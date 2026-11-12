//! The chemist directly uses the scientific and physical resource Endpoints.
use super::super::{global::*, imp};
use crate::shell::{
    ShellError, ShellResult,
    global::Exit as IoExit,
    imp::file_wire::{ClosedFile, ResultChunk, ResultChunkMsg, SERVICE_FD},
};
use hibana_wasip1_runtime::{FileBoundary, HibanaWasiGuest, WasiImportCompletion, protocol};
use serde_json::json;

#[hibana_runtime::scope]
pub async fn run(
    mut guest: &mut HibanaWasiGuest,
    endpoint: &mut hibana::Endpoint<CHEMIST>,
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    let (mut chemistry, call) = async {
        async { Ok(task.recv::<ChemAccess>().await?) };
        async {
            match guest
                .next_file::<ClosedFile, CHEMIST>(&mut private, SERVICE_FD)
                .await?
            {
                FileBoundary::Exit(_) => Err(ShellError::message(
                    "chemist exited before completing the actual laboratory work",
                )),
                call => Ok(call),
            }
        };
    }
    .await?;
    let synthesis = {
        let mut call = call;
        loop {
            match call {
                FileBoundary::Exit(_) => {
                    return Err(ShellError::message(
                        "chemist exited before completing the actual laboratory work",
                    ));
                }
                FileBoundary::Closed { pending, file } => {
                    let requested = file.arguments(b"service/run_synthesis", "compound");
                    match requested {
                        Ok(arguments) => {
                            let result = {
                                let bytes = imp::encode(&arguments)?;
                                chemistry.send::<Synthesis>(&bytes.as_slice()).await?;
                                let mut instrument = chemistry.recv::<FumeLoan>().await?;
                                instrument
                                    .send::<SynthesisRequest>(&bytes.as_slice())
                                    .await?;
                                let result =
                                    imp::decode(instrument.recv::<InstrumentResult>().await?)?;
                                imp::successful(&result)?;
                                chemistry.send::<FumeLoan>(instrument).await?;
                                result
                            };

                            let bytes = imp::encode(&result)?;
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
                                protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                                    SERVICE_FD, 0,
                                )),
                            ))?;
                            break result;
                        }
                        Err(error) => {
                            let reply =
                                json!({"status":"invalid_arguments","error":error.to_string()});

                            let bytes = imp::encode(&reply)?;
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
                                protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                                    SERVICE_FD, 28,
                                )),
                            ))?;
                        }
                    }
                }
            }
            call = guest
                .next_file::<ClosedFile, CHEMIST>(&mut private, SERVICE_FD)
                .await?;
        }
    };
    let mut analysis = {
        let mut call = guest
            .next_file::<ClosedFile, CHEMIST>(&mut private, SERVICE_FD)
            .await?;
        loop {
            match call {
                FileBoundary::Exit(_) => {
                    return Err(ShellError::message(
                        "chemist exited before completing the actual laboratory work",
                    ));
                }
                FileBoundary::Closed { pending, file } => {
                    let requested =
                        file.arguments(b"service/run_analysis", "sample")
                            .and_then(|value| {
                                imp::assigned(&value, "sample", "synthesized_compound")?;
                                Ok(value)
                            });
                    match requested {
                        Ok(arguments) => {
                            let result = {
                                let bytes = imp::encode(&arguments)?;
                                chemistry.send::<InitialAnalysis>(&bytes.as_slice()).await?;
                                let mut instrument = chemistry.recv::<SpecLoan>().await?;
                                instrument.send::<SpecRequest>(&bytes.as_slice()).await?;
                                let result =
                                    imp::decode(instrument.recv::<InstrumentResult>().await?)?;
                                imp::successful(&result)?;
                                chemistry.send::<SpecLoan>(instrument).await?;
                                result
                            };

                            let bytes = imp::encode(&result)?;
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
                                protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                                    SERVICE_FD, 0,
                                )),
                            ))?;
                            break result;
                        }
                        Err(error) => {
                            let reply =
                                json!({"status":"invalid_arguments","error":error.to_string()});

                            let bytes = imp::encode(&reply)?;
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
                                protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                                    SERVICE_FD, 28,
                                )),
                            ))?;
                        }
                    }
                }
            }
            call = guest
                .next_file::<ClosedFile, CHEMIST>(&mut private, SERVICE_FD)
                .await?;
        }
    };
    let (mut pending, mut operation) = {
        let mut call = guest
            .next_file::<ClosedFile, CHEMIST>(&mut private, SERVICE_FD)
            .await?;
        loop {
            match call {
                FileBoundary::Exit(_) => {
                    return Err(ShellError::message(
                        "chemist exited before completing the actual laboratory work",
                    ));
                }
                FileBoundary::Closed { pending, file } => {
                    let requested = file
                        .arguments(b"service/run_separation", "material")
                        .and_then(|value| {
                            imp::assigned(&value, "material", "compound_purification")?;
                            Ok(value)
                        });
                    match requested {
                        Ok(arguments) => {
                            let result = {
                                let bytes = imp::encode(&arguments)?;
                                chemistry.send::<Purification>(&bytes.as_slice()).await?;
                                let mut instrument = chemistry.recv::<CentLoan>().await?;
                                instrument.send::<CentRequest>(&bytes.as_slice()).await?;
                                let result =
                                    imp::decode(instrument.recv::<InstrumentResult>().await?)?;
                                imp::successful(&result)?;
                                chemistry.send::<CentLoan>(instrument).await?;
                                result
                            };
                            break (pending, result);
                        }
                        Err(error) => {
                            let reply =
                                json!({"status":"invalid_arguments","error":error.to_string()});

                            let bytes = imp::encode(&reply)?;
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
                                protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                                    SERVICE_FD, 28,
                                )),
                            ))?;
                        }
                    }
                }
            }
            call = guest
                .next_file::<ClosedFile, CHEMIST>(&mut private, SERVICE_FD)
                .await?;
        }
    };
    let sample = json!({"synthesis":synthesis,"analysis":analysis,"purification":operation});
    let bytes = imp::encode(&sample)?;
    task.send::<Sample>(&bytes.as_slice()).await?;
    task.send::<ChemAccess>(chemistry).await?;
    // A submitted operation returns its actual QC receipt with the original
    // operation result. The guest can then choose the next reconsideration.
    let (approval, quality, call) = loop {
        let branch = task.offer().await?;
        match branch.label() {
            50 => {
                let selected = imp::decode(branch.recv::<FirstChemist>().await?)?;
                imp::qc_arguments(&selected, "CHEMIST")?;
                let mut report = sample.clone();
                report["analysis"] = analysis.clone();
                let bytes = imp::encode(&report)?;
                task.send::<ChemistReport>(&bytes.as_slice()).await?;
            }
            52 => {
                let selected = imp::decode(branch.recv::<NextChemist>().await?)?;
                imp::qc_arguments(&selected, "CHEMIST")?;
                let mut report = sample.clone();
                report["analysis"] = analysis.clone();
                let bytes = imp::encode(&report)?;
                task.send::<ChemistReport>(&bytes.as_slice()).await?;
            }
            51 => {
                let selected = imp::decode(branch.recv::<FirstBiologist>().await?)?;
                imp::qc_arguments(&selected, "BIOLOGIST")?;
            }
            53 => {
                let selected = imp::decode(branch.recv::<NextBiologist>().await?)?;
                imp::qc_arguments(&selected, "BIOLOGIST")?;
            }
            83 => {
                branch.recv::<BiologistRejected>().await?;
            }
            81 => {
                let verification = imp::decode(branch.recv::<ChemistRejected>().await?)?;

                let reply = json!({"operation":operation,"verification":verification});

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
                    protocol::FdClosed::new_with_errno(SERVICE_FD, 0),
                )))?;

                let (returned, call) = async {
                    async { Ok(task.recv::<ChemAccess>().await?) };
                    async {
                        match guest
                            .next_file::<ClosedFile, CHEMIST>(&mut private, SERVICE_FD)
                            .await?
                        {
                            FileBoundary::Exit(_) => Err(ShellError::message(
                                "chemist exited before completing the actual laboratory work",
                            )),
                            call => Ok(call),
                        }
                    };
                }
                .await?;
                chemistry = returned;
                let (next_pending, measurement) = {
                    let mut call = call;
                    loop {
                        match call {
                            FileBoundary::Exit(_) => {
                                return Err(ShellError::message(
                                    "chemist exited before completing the actual laboratory work",
                                ));
                            }
                            FileBoundary::Closed { pending, file } => {
                                let requested = file
                                    .arguments(b"service/run_analysis", "sample")
                                    .and_then(|value| {
                                        imp::assigned(&value, "sample", "synthesized_compound")?;
                                        Ok(value)
                                    });
                                match requested {
                                    Ok(arguments) => {
                                        let result = {
                                            let bytes = imp::encode(&arguments)?;
                                            chemistry.send::<Reanalysis>(&bytes.as_slice()).await?;
                                            let mut instrument =
                                                chemistry.recv::<SpecLoan>().await?;
                                            instrument
                                                .send::<SpecRequest>(&bytes.as_slice())
                                                .await?;
                                            let result = imp::decode(
                                                instrument.recv::<InstrumentResult>().await?,
                                            )?;
                                            imp::successful(&result)?;
                                            chemistry.send::<SpecLoan>(instrument).await?;
                                            result
                                        };
                                        break (pending, result);
                                    }
                                    Err(error) => {
                                        let reply = json!({"status":"invalid_arguments","error":error.to_string()});

                                        let bytes = imp::encode(&reply)?;
                                        for (index, chunk) in bytes
                                            .chunks(protocol::WASIP1_IO_CHUNK_CAPACITY)
                                            .enumerate()
                                        {
                                            private
                                                .send::<ResultChunkMsg>(&ResultChunk::new(
                                                    index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                                    chunk,
                                                )?)
                                                .await?;
                                        }
                                        guest = pending.complete(WasiImportCompletion::FdClose(
                                            protocol::FdClosedRet(
                                                protocol::FdClosed::new_with_errno(SERVICE_FD, 28),
                                            ),
                                        ))?;
                                    }
                                }
                            }
                        }
                        call = guest
                            .next_file::<ClosedFile, CHEMIST>(&mut private, SERVICE_FD)
                            .await?;
                    }
                };
                pending = next_pending;
                operation = measurement.clone();
                analysis = measurement;
                task.send::<ChemAccess>(chemistry).await?;
            }
            80 => {
                let verification = imp::decode(branch.recv::<ChemistApproved>().await?)?;
                let reply = json!({"operation":operation,"verification":verification});

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
                    protocol::FdClosed::new_with_errno(SERVICE_FD, 0),
                )))?;

                let (quality, call) = async {
                    async {
                        loop {
                            let branch = task.offer().await?;
                            match branch.label() {
                                87 => {
                                    branch.recv::<LastBiologistRejected>().await?;
                                }
                                85 => {
                                    branch.recv::<LastBiologistApproved>().await?;
                                    break;
                                }
                                label => {
                                    return Err(ShellError::message(format!(
                                        "unexpected remaining biological result {label}"
                                    )));
                                }
                            }
                        }
                        imp::decode(task.recv::<Quality>().await?)
                    };
                    async {
                        match guest
                            .next_file::<ClosedFile, CHEMIST>(&mut private, SERVICE_FD)
                            .await?
                        {
                            FileBoundary::Exit(_) => Err(ShellError::message(
                                "chemist exited before receiving actual quality",
                            )),
                            call => Ok(call),
                        }
                    };
                }
                .await?;
                break (verification, quality, call);
            }
            82 => {
                branch.recv::<BiologistApproved>().await?;
                let approval = loop {
                    let branch = task.offer().await?;
                    match branch.label() {
                        54 => {
                            let selected = imp::decode(branch.recv::<LastChemist>().await?)?;
                            imp::qc_arguments(&selected, "CHEMIST")?;
                            let mut report = sample.clone();
                            report["analysis"] = analysis.clone();
                            let bytes = imp::encode(&report)?;
                            task.send::<ChemistReport>(&bytes.as_slice()).await?;
                        }
                        86 => {
                            let verification =
                                imp::decode(branch.recv::<LastChemistRejected>().await?)?;

                            let reply = json!({"operation":operation,"verification":verification});

                            let bytes = imp::encode(&reply)?;
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
                                protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                                    SERVICE_FD, 0,
                                )),
                            ))?;

                            let (returned,call)=async {
 async {Ok(task.recv::<ChemAccess>().await?)};
 async {
match guest.next_file::<ClosedFile, CHEMIST>(&mut private, SERVICE_FD).await? {
 FileBoundary::Exit(_) => Err(ShellError::message("chemist exited before completing the actual laboratory work")),
 call => Ok(call),
}
};
}.await?;
                            chemistry = returned;
                            let (next_pending, measurement) = {
                                let mut call = call;
                                loop {
                                    match call {
                                        FileBoundary::Exit(_) => {
                                            return Err(ShellError::message(
                                                "chemist exited before completing the actual laboratory work",
                                            ));
                                        }
                                        FileBoundary::Closed { pending, file } => {
                                            let requested = file
                                                .arguments(b"service/run_analysis", "sample")
                                                .and_then(|value| {
                                                    imp::assigned(
                                                        &value,
                                                        "sample",
                                                        "synthesized_compound",
                                                    )?;
                                                    Ok(value)
                                                });
                                            match requested {
                                                Ok(arguments) => {
                                                    let result = {
                                                        let bytes = imp::encode(&arguments)?;
                                                        chemistry
                                                            .send::<LastReanalysis>(
                                                                &bytes.as_slice(),
                                                            )
                                                            .await?;
                                                        let mut instrument =
                                                            chemistry.recv::<SpecLoan>().await?;
                                                        instrument
                                                            .send::<SpecRequest>(&bytes.as_slice())
                                                            .await?;
                                                        let result = imp::decode(
                                                            instrument
                                                                .recv::<InstrumentResult>()
                                                                .await?,
                                                        )?;
                                                        imp::successful(&result)?;
                                                        chemistry
                                                            .send::<SpecLoan>(instrument)
                                                            .await?;
                                                        result
                                                    };
                                                    break (pending, result);
                                                }
                                                Err(error) => {
                                                    let reply = json!({"status":"invalid_arguments","error":error.to_string()});

                                                    let bytes = imp::encode(&reply)?;
                                                    for (index, chunk) in bytes
                                                        .chunks(protocol::WASIP1_IO_CHUNK_CAPACITY)
                                                        .enumerate()
                                                    {
                                                        private.send::<ResultChunkMsg>(&ResultChunk::new(index * protocol::WASIP1_IO_CHUNK_CAPACITY, chunk)?).await?;
                                                    }
                                                    guest =
                                                        pending
                                                            .complete(WasiImportCompletion::FdClose(
                                                            protocol::FdClosedRet(
                                                                protocol::FdClosed::new_with_errno(
                                                                    SERVICE_FD, 28,
                                                                ),
                                                            ),
                                                        ))?;
                                                }
                                            }
                                        }
                                    }
                                    call = guest
                                        .next_file::<ClosedFile, CHEMIST>(&mut private, SERVICE_FD)
                                        .await?;
                                }
                            };
                            pending = next_pending;
                            operation = measurement.clone();
                            analysis = measurement;
                            task.send::<ChemAccess>(chemistry).await?;
                        }
                        84 => {
                            let verification =
                                imp::decode(branch.recv::<LastChemistApproved>().await?)?;
                            let reply = json!({"operation":operation,"verification":verification});

                            let bytes = imp::encode(&reply)?;
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
                                protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                                    SERVICE_FD, 0,
                                )),
                            ))?;

                            break verification;
                        }
                        label => {
                            return Err(ShellError::message(format!(
                                "unexpected remaining chemist operation {label}"
                            )));
                        }
                    }
                };
                let (quality, call) = async {
                    async { imp::decode(task.recv::<Quality>().await?) };
                    async {
                        match guest
                            .next_file::<ClosedFile, CHEMIST>(&mut private, SERVICE_FD)
                            .await?
                        {
                            FileBoundary::Exit(_) => Err(ShellError::message(
                                "chemist exited before receiving actual quality",
                            )),
                            call => Ok(call),
                        }
                    };
                }
                .await?;
                break (approval, quality, call);
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected chemist review operation {label}"
                )));
            }
        }
    };
    imp::same(&quality["biology"]["sample"], &sample)?;
    imp::same(&quality["biology"]["qc"]["chemist"], &approval)?;
    let _validation = {
        let mut call = call;
        loop {
            match call {
                FileBoundary::Exit(_) => {
                    return Err(ShellError::message(
                        "chemist exited before final validation",
                    ));
                }
                FileBoundary::Closed { pending, file } => {
                    let requested =
                        file.arguments(b"service/run_analysis", "sample")
                            .and_then(|value| {
                                imp::assigned(&value, "sample", "final_validation")?;
                                Ok(value)
                            });
                    match requested {
                        Ok(arguments) => {
                            let admitted = json!({"arguments":arguments,"quality":quality});
                            let bytes = imp::encode(&admitted)?;
                            task.send::<FinalValidation>(&bytes.as_slice()).await?;
                            let mut instrument = task.recv::<SpecLoan>().await?;
                            instrument.send::<SpecRequest>(&bytes.as_slice()).await?;
                            let result = imp::decode(instrument.recv::<InstrumentResult>().await?)?;
                            imp::successful(&result)?;
                            task.send::<SpecLoan>(instrument).await?;

                            let bytes = imp::encode(&result)?;
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
                                protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                                    SERVICE_FD, 0,
                                )),
                            ))?;

                            break result;
                        }
                        Err(error) => {
                            let reply =
                                json!({"status":"invalid_arguments","error":error.to_string()});

                            let bytes = imp::encode(&reply)?;
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
                                protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                                    SERVICE_FD, 28,
                                )),
                            ))?;
                        }
                    }
                }
            }
            call = guest
                .next_file::<ClosedFile, CHEMIST>(&mut private, SERVICE_FD)
                .await?;
        }
    };
    drop(task);
    loop {
        match guest
            .next_file::<ClosedFile, CHEMIST>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Exit(exit) => {
                let code = exit.status() as i32;
                private.send::<IoExit>(&code).await?;
                return Ok(code);
            }
            FileBoundary::Closed { pending, .. } => {
                let reply = json!({"status":"out_of_order","error":"the chemist's task Endpoint has completed"});

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
                    protocol::FdClosed::new_with_errno(SERVICE_FD, 8),
                )))?;
            }
        }
    }
}
