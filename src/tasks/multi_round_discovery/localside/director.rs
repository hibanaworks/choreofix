//! The director transfers the actual material and forwards the checked outcome.
use super::super::{global::*, imp};
use crate::shell::{
    ShellError, ShellResult,
    global::Exit as IoExit,
    imp::file_wire::{ClosedFile, ResultChunk, ResultChunkMsg, SERVICE_FD},
};
use hibana::Endpoint;
use hibana_wasip1_runtime::{FileBoundary, HibanaWasiGuest, WasiImportCompletion, protocol};
use serde_json::json;

#[hibana_runtime::scope]
pub async fn run<'r>(
    mut guest: &mut HibanaWasiGuest,
    (endpoint, mut study, mut formulation, mut bioassay, mut stability, mut purity, mut toxicity): (
        &mut Endpoint<'r, DIRECTOR>,
        Endpoint<'r, STUDY>,
        Endpoint<'r, FORMULATION>,
        Endpoint<'r, BIOASSAY>,
        Endpoint<'r, STABILITY>,
        Endpoint<'r, PURITY>,
        Endpoint<'r, TOXICITY>,
    ),
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    let mut previous: Option<serde_json::Value> = None;
    loop {
        study.send::<Specimen<FORMULATION>>(formulation).await?;
        async {
            let [mut bio, mut stable, mut pure, mut toxic] = task.par()?;
            async { bio.send::<Specimen<BIOASSAY>>(bioassay).await };
            async { stable.send::<Specimen<STABILITY>>(stability).await };
            async { pure.send::<Specimen<PURITY>>(purity).await };
            async { toxic.send::<Specimen<TOXICITY>>(toxicity).await };
        }
        .await?;
        task.send::<Specimen<STUDY>>(study).await?;
        (formulation, bioassay, stability, purity, toxicity) = async {
            let [mut form, mut bio, mut stable, mut pure, mut toxic] = task.par()?;
            async { form.recv::<Specimen<FORMULATION>>().await };
            async { bio.recv::<Specimen<BIOASSAY>>().await };
            async { stable.recv::<Specimen<STABILITY>>().await };
            async { pure.recv::<Specimen<PURITY>>().await };
            async { toxic.recv::<Specimen<TOXICITY>>().await };
        }
        .await?;
        let regulatory = imp::decode(task.recv::<RegulatoryEvidence<DIRECTOR>>().await?)?;
        let displayed = match previous.take() {
            Some(mut actual) => {
                actual["coordination"] = json!({"regulatory":regulatory});
                actual
            }
            None => regulatory.clone(),
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
        let outcome = loop {
            let FileBoundary::Closed { pending, file } = guest
                .next_file::<ClosedFile, DIRECTOR>(&mut private, SERVICE_FD)
                .await?
            else {
                return Err(ShellError::message(
                    "director exited before deciding the study",
                ));
            };
            let requested = file
                .arguments(b"service/submit_project_decision", "outcome")
                .and_then(|value| {
                    let step = match value["outcome"].as_str() {
                        Some("continue") => 12,
                        Some("retry") => 13,
                        Some("abort") => 14,
                        _ => return Err(ShellError::message("unknown project outcome")),
                    };
                    imp::arguments(step, &value)?;
                    if step == 12 && regulatory["success"] != true {
                        return Err(ShellError::message("continuation requires actual approval"));
                    }
                    Ok(value)
                });
            match requested {
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
                    task.send::<DecisionRequest>(&bytes.as_slice()).await?;
                    let actual = imp::decode(task.recv::<DecisionEvidence<DIRECTOR>>().await?)?;
                    imp::same(&actual["outcome"], &arguments["outcome"])?;
                    imp::same(&actual["regulatory"], &regulatory)?;
                    imp::decision_receipt(&actual, &actual["decision"])?;
                    study = task.recv::<Specimen<STUDY>>().await?;
                    let bytes = imp::encode(&actual["decision"])?;
                    previous = Some(actual["decision"].clone());
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
                    break actual;
                }
            }
        };
        if outcome["outcome"] == "retry" {
            continue;
        }
        task.send::<Specimen<STUDY>>(study).await?;
        async {
            let [mut form, mut bio, mut stable, mut pure, mut toxic] = task.par()?;
            async { form.send::<Archived<FORMULATION>>(formulation).await };
            async { bio.send::<Archived<BIOASSAY>>(bioassay).await };
            async { stable.send::<Archived<STABILITY>>(stability).await };
            async { pure.send::<Archived<PURITY>>(purity).await };
            async { toxic.send::<Archived<TOXICITY>>(toxicity).await };
        }
        .await?;
        let bytes = imp::encode(&outcome)?;
        async {
            let [
                mut chemist,
                mut biologist,
                mut toxicologist,
                mut formulator,
                mut clinician,
                mut regulator,
                mut hplc,
                mut mass,
                mut cells,
                mut form,
                mut review,
            ] = task.par()?;
            async {
                chemist
                    .send::<Final<CHEMIST>>(&imp::FinalOutcome::<CHEMIST>::from_bytes(&bytes)?)
                    .await
            };
            async {
                biologist
                    .send::<Final<BIOLOGIST>>(&imp::FinalOutcome::<BIOLOGIST>::from_bytes(&bytes)?)
                    .await
            };
            async {
                toxicologist
                    .send::<Final<TOXICOLOGIST>>(&imp::FinalOutcome::<TOXICOLOGIST>::from_bytes(
                        &bytes,
                    )?)
                    .await
            };
            async {
                formulator
                    .send::<Final<FORMULATOR>>(&imp::FinalOutcome::<FORMULATOR>::from_bytes(
                        &bytes,
                    )?)
                    .await
            };
            async {
                clinician
                    .send::<Final<CLINICIAN>>(&imp::FinalOutcome::<CLINICIAN>::from_bytes(&bytes)?)
                    .await
            };
            async {
                regulator
                    .send::<Final<REGULATOR>>(&imp::FinalOutcome::<REGULATOR>::from_bytes(&bytes)?)
                    .await
            };
            async {
                hplc.send::<Final<HPLC_SERVICE>>(&imp::FinalOutcome::<HPLC_SERVICE>::from_bytes(
                    &bytes,
                )?)
                .await
            };
            async {
                mass.send::<Final<MASS_SERVICE>>(&imp::FinalOutcome::<MASS_SERVICE>::from_bytes(
                    &bytes,
                )?)
                .await
            };
            async {
                cells
                    .send::<Final<CELL_SERVICE>>(&imp::FinalOutcome::<CELL_SERVICE>::from_bytes(
                        &bytes,
                    )?)
                    .await
            };
            async {
                form.send::<Final<FORM_SERVICE>>(&imp::FinalOutcome::<FORM_SERVICE>::from_bytes(
                    &bytes,
                )?)
                .await
            };
            async {
                review
                    .send::<Final<STUDY_SERVICE>>(&imp::FinalOutcome::<STUDY_SERVICE>::from_bytes(
                        &bytes,
                    )?)
                    .await
            };
        }
        .await?;
        break;
    }
    drop(task);
    loop {
        match guest
            .next_file::<ClosedFile, DIRECTOR>(&mut private, SERVICE_FD)
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
