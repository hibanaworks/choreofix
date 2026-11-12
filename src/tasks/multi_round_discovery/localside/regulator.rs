//! The regulator uses its actual study loan for routine/deep review and submission.
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
    endpoint: &mut Endpoint<REGULATOR>,
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    let mut previous: Option<serde_json::Value> = None;
    loop {
        let branch = task.offer().await?;
        if branch.label() == Final::<REGULATOR>::LOGICAL_LABEL {
            let final_outcome = imp::decode(branch.recv::<Final<REGULATOR>>().await?)?;
            let mut actual = previous
                .take()
                .ok_or_else(|| ShellError::message("missing actual review submission"))?;
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
        let mut study = branch.recv::<Specimen<STUDY>>().await?;
        let clinical = imp::decode(task.recv::<ClinicalEvidence<REGULATOR>>().await?)?;
        imp::reply(9, &clinical)?;
        let displayed = match previous.take() {
            Some(mut actual) => {
                actual["coordination"] = json!({"clinical":clinical});
                actual
            }
            None => clinical.clone(),
        };
        let bytes = imp::encode(&displayed)?;
        for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
            private
                .send::<ResultChunkMsg>(&ResultChunk::new(
                    index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                    chunk,
                )?)
                .await?;
        }
        let mut last_review = None;
        loop {
            let FileBoundary::Closed { pending, file } = guest
                .next_file::<ClosedFile, REGULATOR>(&mut private, SERVICE_FD)
                .await?
            else {
                return Err(ShellError::message(
                    "regulator exited with its study Endpoint",
                ));
            };
            if imp::evidence_file(&file, "regulatory").is_ok() && last_review.is_some() {
                task.send::<Specimen<STUDY>>(study).await?;
                let actual = json!({"evidence":"regulatory","result":last_review});
                let bytes = imp::encode(&actual)?;
                previous = Some(actual);
                for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
                    private
                        .send::<ResultChunkMsg>(&ResultChunk::new(
                            index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                            chunk,
                        )?)
                        .await?;
                }
                guest = pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
                    protocol::FdClosed::new_with_errno(SERVICE_FD, 0),
                )))?;
                break;
            }
            let reply = if let Ok(arguments) = imp::analysis_file::<10>(&file) {
                let bytes = imp::encode(&arguments)?;
                let input = imp::AnalysisArguments::<10>::from_bytes(&bytes)?;
                study.send::<RoutineReview>(&input).await?;
                study.send::<RoutineReview>(&input).await?;
                study.send::<RoutineReview>(&input).await?;
                task.send::<Specimen<STUDY>>(study).await?;
                let branch = task.offer().await?;
                let actual = if branch.label() == Passed::<10>::LOGICAL_LABEL {
                    imp::decode(branch.recv::<Passed<10>>().await?)?
                } else {
                    imp::decode(branch.recv::<Failed<10>>().await?)?
                };
                imp::regulatory_receipt(&arguments, &clinical, &actual)?;
                study = task.recv::<Specimen<STUDY>>().await?;
                Ok(actual)
            } else if let Ok(arguments) = imp::analysis_file::<11>(&file) {
                let bytes = imp::encode(&arguments)?;
                study
                    .send::<DeepIntent<HPLC>>(&imp::JsonTo::<HPLC>::from_bytes(&bytes)?)
                    .await?;
                let mut hplc = task.recv::<Instrument<HPLC>>().await?;
                study
                    .send::<DeepIntent<MASS>>(&imp::JsonTo::<MASS>::from_bytes(&bytes)?)
                    .await?;
                let mut mass = task.recv::<Instrument<MASS>>().await?;
                let input = imp::AnalysisArguments::<11>::from_bytes(&bytes)?;
                study.send::<DeepReview>(&input).await?;
                async {
                    async { hplc.send::<Request<11>>(&input).await };
                    async { mass.send::<Request<11>>(&input).await };
                }
                .await?;
                async {
                    let [mut chromatography, mut spectrometry] = task.par()?;
                    async { chromatography.send::<Instrument<HPLC>>(hplc).await };
                    async { spectrometry.send::<Instrument<MASS>>(mass).await };
                }
                .await?;
                task.send::<Specimen<STUDY>>(study).await?;
                let branch = task.offer().await?;
                let actual = if branch.label() == Passed::<11>::LOGICAL_LABEL {
                    imp::decode(branch.recv::<Passed<11>>().await?)?
                } else {
                    imp::decode(branch.recv::<Failed<11>>().await?)?
                };
                imp::regulatory_receipt(&arguments, &clinical, &actual)?;
                study = task.recv::<Specimen<STUDY>>().await?;
                let (hplc, mass) = async {
                    let [mut chromatography, mut spectrometry] = task.par()?;
                    async { chromatography.recv::<Instrument<HPLC>>().await };
                    async { spectrometry.recv::<Instrument<MASS>>().await };
                }
                .await?;
                task.send::<Instrument<MASS>>(mass).await?;
                task.send::<Instrument<HPLC>>(hplc).await?;
                Ok(actual)
            } else {
                Err(ShellError::message(
                    "invalid regulatory review or evidence file",
                ))
            };
            let (reply, errno) = match reply {
                Ok(value) => {
                    last_review = Some(value.clone());
                    (value, 0)
                }
                Err(error) => (
                    json!({"status":"invalid_arguments","error":error.to_string()}),
                    28,
                ),
            };
            let bytes = imp::encode(&reply)?;
            for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
                private
                    .send::<ResultChunkMsg>(&ResultChunk::new(
                        index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                        chunk,
                    )?)
                    .await?;
            }
            guest = pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
                protocol::FdClosed::new_with_errno(SERVICE_FD, errno),
            )))?;
        }
    }
    drop(task);
    loop {
        match guest
            .next_file::<ClosedFile, REGULATOR>(&mut private, SERVICE_FD)
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
