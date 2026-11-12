//! Clinical assessment consumes the actual material views from both assay lanes.
use super::super::{global::*, imp};
use crate::shell::{
    ShellError, ShellResult,
    global::Exit as IoExit,
    imp::file_wire::{ClosedFile, ResultChunk, ResultChunkMsg, SERVICE_FD},
};
use hibana::{Endpoint, g::Message};
use hibana_wasip1_runtime::{FileBoundary, HibanaWasiGuest, WasiImportCompletion, protocol};
use serde_json::json;

#[hibana_runtime::scope]
pub async fn run(
    mut guest: &mut HibanaWasiGuest,
    endpoint: &mut Endpoint<CLINICIAN>,
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    let _culture = task.recv::<Specimen<CULTURE>>().await?;
    let mut previous: Option<serde_json::Value> = None;
    loop {
        let branch = task.offer().await?;
        if branch.label() == Final::<CLINICIAN>::LOGICAL_LABEL {
            let final_outcome = imp::decode(branch.recv::<Final<CLINICIAN>>().await?)?;
            let mut actual = previous
                .take()
                .ok_or_else(|| ShellError::message("missing actual clinical reply"))?;
            actual["coordination"] = final_outcome;
            let bytes = imp::encode(&actual)?;
            for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
                private
                    .send::<ResultChunkMsg>(&ResultChunk::new(
                        index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                        chunk,
                    )?)
                    .await?;
            }
            break;
        }
        let formulation = branch.recv::<Specimen<FORMULATION>>().await?;
        let mut study = task.recv::<Specimen<STUDY>>().await?;
        if let Some(mut actual) = previous.take() {
            actual["coordination"] = json!({"received_specimens":["STUDY"]});
            let bytes = imp::encode(&actual)?;
            for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
                private
                    .send::<ResultChunkMsg>(&ResultChunk::new(
                        index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                        chunk,
                    )?)
                    .await?;
            }
        }
        let (mut call, (purity, bioassay, toxicity, stability)) = async {
            async {
                Ok(guest
                    .next_file::<ClosedFile, CLINICIAN>(&mut private, SERVICE_FD)
                    .await?)
            };
            async {
                let purity = task.recv::<Specimen<PURITY>>().await?;
                let ((bioassay, toxicity), stability) = async {
                    let [mut cells, mut hplc] = task.par()?;
                    async {
                        let bioassay = cells.recv::<Specimen<BIOASSAY>>().await?;
                        let toxicity = cells.recv::<Specimen<TOXICITY>>().await?;
                        Ok((bioassay, toxicity))
                    };
                    async { hplc.recv::<Specimen<STABILITY>>().await };
                }
                .await?;
                Ok((purity, bioassay, toxicity, stability))
            };
        }
        .await?;
        loop {
            let FileBoundary::Closed { pending, file } = call else {
                return Err(ShellError::message(
                    "clinical lead exited before the current assessment",
                ));
            };
            match imp::analysis_file::<9>(&file) {
                Err(error) => {
                    let bytes = imp::encode(
                        &json!({"status":"invalid_arguments","error":error.to_string()}),
                    )?;
                    for (index, chunk) in
                        bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                    {
                        private
                            .send::<ResultChunkMsg>(&ResultChunk::new(
                                index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                chunk,
                            )?)
                            .await?;
                    }
                    guest = pending.complete(WasiImportCompletion::FdClose(
                        protocol::FdClosedRet(protocol::FdClosed::new_with_errno(SERVICE_FD, 28)),
                    ))?;
                }
                Ok(arguments) => {
                    let bytes = imp::encode(&arguments)?;
                    let input = imp::AnalysisArguments::<9>::from_bytes(&bytes)?;
                    study.send::<Intent<9>>(&input).await?;
                    let mut instrument = task.recv::<Instrument<MASS>>().await?;
                    instrument.send::<Request<9>>(&input).await?;
                    task.send::<Instrument<MASS>>(instrument).await?;
                    task.send::<Specimen<STUDY>>(study).await?;
                    let actual = imp::decode(task.recv::<Passed<9>>().await?)?;
                    async {
                        let [mut form, mut bio, mut stable, mut pure, mut toxic] = task.par()?;
                        async { form.send::<Specimen<FORMULATION>>(formulation).await };
                        async { bio.send::<Specimen<BIOASSAY>>(bioassay).await };
                        async { stable.send::<Specimen<STABILITY>>(stability).await };
                        async { pure.send::<Specimen<PURITY>>(purity).await };
                        async { toxic.send::<Specimen<TOXICITY>>(toxicity).await };
                    }
                    .await?;
                    let bytes = imp::encode(&actual)?;
                    previous = Some(actual);
                    for (index, chunk) in
                        bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                    {
                        private
                            .send::<ResultChunkMsg>(&ResultChunk::new(
                                index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                chunk,
                            )?)
                            .await?;
                    }
                    guest = pending.complete(WasiImportCompletion::FdClose(
                        protocol::FdClosedRet(protocol::FdClosed::new_with_errno(SERVICE_FD, 0)),
                    ))?;
                    break;
                }
            }
            call = guest
                .next_file::<ClosedFile, CLINICIAN>(&mut private, SERVICE_FD)
                .await?;
        }
    }
    drop(task);
    loop {
        match guest
            .next_file::<ClosedFile, CLINICIAN>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Exit(exit) => {
                let code = exit.status() as i32;
                private.send::<IoExit>(&code).await?;
                return Ok(code);
            }
            FileBoundary::Closed { pending, .. } => {
                let bytes = imp::encode(
                    &json!({"status":"out_of_order","error":"the project has a final outcome"}),
                )?;
                for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
                    private
                        .send::<ResultChunkMsg>(&ResultChunk::new(
                            index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                            chunk,
                        )?)
                        .await?;
                }
                guest = pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
                    protocol::FdClosed::new_with_errno(SERVICE_FD, 8),
                )))?;
            }
        }
    }
}
