//! Real trial outcomes and actual QC receipts drive biological reconsideration.
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
    endpoint: &mut hibana::Endpoint<BIOLOGIST>,
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    let preparation = {
        let mut call = guest
            .next_file::<ClosedFile, BIOLOGIST>(&mut private, SERVICE_FD)
            .await?;
        loop {
            match call {
                FileBoundary::Exit(_) => {
                    return Err(ShellError::message(
                        "biologist exited before preparing the actual trial",
                    ));
                }
                FileBoundary::Closed { pending, file } => {
                    let requested = file
                        .arguments(b"service/run_separation", "material")
                        .and_then(|value| {
                            imp::assigned(&value, "material", "cell_cultures")?;
                            Ok(value)
                        });
                    match requested {
                        Ok(arguments) => {
                            let bytes = imp::encode(&arguments)?;
                            task.send::<Preparation>(&bytes.as_slice()).await?;
                            let mut instrument = task.recv::<CentLoan>().await?;
                            instrument.send::<CentRequest>(&bytes.as_slice()).await?;
                            let result = imp::decode(instrument.recv::<InstrumentResult>().await?)?;
                            imp::successful(&result)?;
                            task.send::<CentLoan>(instrument).await?;

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
                .next_file::<ClosedFile, BIOLOGIST>(&mut private, SERVICE_FD)
                .await?;
        }
    };
    let calibration = {
        let mut call = guest
            .next_file::<ClosedFile, BIOLOGIST>(&mut private, SERVICE_FD)
            .await?;
        loop {
            match call {
                FileBoundary::Exit(_) => {
                    return Err(ShellError::message(
                        "biologist exited before preparing the actual trial",
                    ));
                }
                FileBoundary::Closed { pending, file } => {
                    let requested =
                        file.arguments(b"service/run_analysis", "sample")
                            .and_then(|value| {
                                imp::assigned(&value, "sample", "fluorescence")?;
                                Ok(value)
                            });
                    match requested {
                        Ok(arguments) => {
                            let bytes = imp::encode(&arguments)?;
                            task.send::<Calibration>(&bytes.as_slice()).await?;
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
                .next_file::<ClosedFile, BIOLOGIST>(&mut private, SERVICE_FD)
                .await?;
        }
    };
    let ((sample, mut biology, mut selected), call) = async {
        async {
            let sample = imp::decode(task.recv::<Sample>().await?)?;
            let biology = task.recv::<BioAccess>().await?;
            let branch = task.offer().await?;
            let selected = match branch.label() {
                50 => {
                    let value = imp::decode(branch.recv::<FirstChemist>().await?)?;
                    imp::qc_arguments(&value, "CHEMIST")?;
                    value
                }
                51 => {
                    let value = imp::decode(branch.recv::<FirstBiologist>().await?)?;
                    imp::qc_arguments(&value, "BIOLOGIST")?;
                    value
                }
                label => {
                    return Err(ShellError::message(format!(
                        "unexpected initial QC request {label}"
                    )));
                }
            };
            Ok((sample, biology, selected))
        };
        async {
            match guest
                .next_file::<ClosedFile, BIOLOGIST>(&mut private, SERVICE_FD)
                .await?
            {
                FileBoundary::Exit(_) => Err(ShellError::message(
                    "biologist exited before actual approval",
                )),
                call => Ok(call),
            }
        };
    }
    .await?;
    let (mut pending, mut operation, mut arguments) = {
        let mut call = call;
        loop {
            match call {
                FileBoundary::Exit(_) => {
                    return Err(ShellError::message(
                        "biologist exited before the actual trial",
                    ));
                }
                FileBoundary::Closed { pending, file } => {
                    let requested =
                        imp::assay_file(&file, &sample["synthesis"]["data"]["compound"]);
                    match requested {
                        Ok(arguments) => {
                            let admitted = json!({"arguments":arguments,"purification":sample["purification"],
          "preparation":preparation,"calibration":calibration});
                            let bytes = imp::encode(&admitted)?;
                            biology.send::<BioRequest>(&bytes.as_slice()).await?;
                            task.send::<BioAccess>(biology).await?;
                            let branch = task.offer().await?;
                            match branch.label() {
                                25 => {
                                    let result = imp::decode(branch.recv::<BioPassed>().await?)?;
                                    break (pending, result, arguments);
                                }
                                26 => {
                                    let result = imp::decode(branch.recv::<BioFailed>().await?)?;

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

                                    let (returned, next_call) = async {
                                        async { Ok(task.recv::<BioAccess>().await?) };
                                        async {
                                            match guest
                                                .next_file::<ClosedFile, BIOLOGIST>(
                                                    &mut private,
                                                    SERVICE_FD,
                                                )
                                                .await?
                                            {
                                                FileBoundary::Exit(_) => Err(ShellError::message(
                                                    "biologist exited before actual approval",
                                                )),
                                                call => Ok(call),
                                            }
                                        };
                                    }
                                    .await?;
                                    biology = returned;
                                    call = next_call;
                                    continue;
                                }
                                label => {
                                    return Err(ShellError::message(format!(
                                        "unexpected actual trial result {label}"
                                    )));
                                }
                            }
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
                .next_file::<ClosedFile, BIOLOGIST>(&mut private, SERVICE_FD)
                .await?;
        }
    };
    if selected["scientist"] == "BIOLOGIST" {
        let report = json!({"sample":sample,"preparation":preparation,"calibration":calibration,
 "arguments":arguments,"result":operation});
        let bytes = imp::encode(&report)?;
        task.send::<BiologyReport>(&bytes.as_slice()).await?;
    }
    let approval = loop {
        let branch = task.offer().await?;
        match branch.label() {
            52 => {
                let selected = imp::decode(branch.recv::<NextChemist>().await?)?;
                imp::qc_arguments(&selected, "CHEMIST")?;
            }
            53 => {
                let selected = imp::decode(branch.recv::<NextBiologist>().await?)?;
                imp::qc_arguments(&selected, "BIOLOGIST")?;

                let report = json!({"sample":sample,"preparation":preparation,"calibration":calibration,
 "arguments":arguments,"result":operation});
                let bytes = imp::encode(&report)?;
                task.send::<BiologyReport>(&bytes.as_slice()).await?;
            }
            81 => {
                branch.recv::<ChemistRejected>().await?;
            }
            83 => {
                let verification = imp::decode(branch.recv::<BiologistRejected>().await?)?;
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

                let ((returned, next_selected), call) = async {
                    async {
                        let returned = task.recv::<BioAccess>().await?;
                        let branch = task.offer().await?;
                        let selected = match branch.label() {
                            52 => {
                                let value = imp::decode(branch.recv::<NextChemist>().await?)?;
                                imp::qc_arguments(&value, "CHEMIST")?;
                                value
                            }
                            53 => {
                                let value = imp::decode(branch.recv::<NextBiologist>().await?)?;
                                imp::qc_arguments(&value, "BIOLOGIST")?;
                                value
                            }
                            label => {
                                return Err(ShellError::message(format!(
                                    "unexpected reconsidered QC subject {label}"
                                )));
                            }
                        };
                        Ok((returned, selected))
                    };
                    async {
                        match guest
                            .next_file::<ClosedFile, BIOLOGIST>(&mut private, SERVICE_FD)
                            .await?
                        {
                            FileBoundary::Exit(_) => Err(ShellError::message(
                                "biologist exited before actual approval",
                            )),
                            call => Ok(call),
                        }
                    };
                }
                .await?;
                biology = returned;
                selected = next_selected;
                (pending, operation, arguments) = {
                    let mut call = call;
                    loop {
                        match call {
                            FileBoundary::Exit(_) => {
                                return Err(ShellError::message(
                                    "biologist exited before the actual trial",
                                ));
                            }
                            FileBoundary::Closed { pending, file } => {
                                let requested = imp::assay_file(
                                    &file,
                                    &sample["synthesis"]["data"]["compound"],
                                );
                                match requested {
                                    Ok(arguments) => {
                                        let admitted = json!({"arguments":arguments,"purification":sample["purification"],
          "preparation":preparation,"calibration":calibration});
                                        let bytes = imp::encode(&admitted)?;
                                        biology.send::<Retest>(&bytes.as_slice()).await?;
                                        task.send::<BioAccess>(biology).await?;
                                        let branch = task.offer().await?;
                                        match branch.label() {
                                            101 => {
                                                let result = imp::decode(
                                                    branch.recv::<RetestPassed>().await?,
                                                )?;
                                                break (pending, result, arguments);
                                            }
                                            102 => {
                                                let result = imp::decode(
                                                    branch.recv::<RetestFailed>().await?,
                                                )?;

                                                let bytes = imp::encode(&result)?;
                                                for (index, chunk) in bytes
                                                    .chunks(protocol::WASIP1_IO_CHUNK_CAPACITY)
                                                    .enumerate()
                                                {
                                                    private.send::<ResultChunkMsg>(&ResultChunk::new(index*protocol::WASIP1_IO_CHUNK_CAPACITY,chunk)?).await?;
                                                }
                                                guest = pending.complete(
                                                    WasiImportCompletion::FdClose(
                                                        protocol::FdClosedRet(
                                                            protocol::FdClosed::new_with_errno(
                                                                SERVICE_FD, 0,
                                                            ),
                                                        ),
                                                    ),
                                                )?;

                                                let(returned,next_call)=async {
         async {Ok(task.recv::<BioAccess>().await?)};
         async {
match guest.next_file::<ClosedFile,BIOLOGIST>(&mut private,SERVICE_FD).await?{
 FileBoundary::Exit(_)=>Err(ShellError::message("biologist exited before actual approval")),
 call=>Ok(call),
}
};
        }.await?;
                                                biology = returned;
                                                call = next_call;
                                                continue;
                                            }
                                            label => {
                                                return Err(ShellError::message(format!(
                                                    "unexpected actual trial result {label}"
                                                )));
                                            }
                                        }
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
                            .next_file::<ClosedFile, BIOLOGIST>(&mut private, SERVICE_FD)
                            .await?;
                    }
                };
                if selected["scientist"] == "BIOLOGIST" {
                    let report = json!({"sample":sample,"preparation":preparation,"calibration":calibration,
 "arguments":arguments,"result":operation});
                    let bytes = imp::encode(&report)?;
                    task.send::<BiologyReport>(&bytes.as_slice()).await?;
                }
            }
            82 => {
                let verification = imp::decode(branch.recv::<BiologistApproved>().await?)?;
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

                return async {
                    async {
                        loop {
                            let branch = task.offer().await?;
                            match branch.label() {
                                86 => { branch.recv::<LastChemistRejected>().await?; }
                                84 => { branch.recv::<LastChemistApproved>().await?; break; }
                                label => return Err(ShellError::message(format!("unexpected remaining chemistry result {label}"))),
                            }
                        }
                        let approved = imp::decode(task.recv::<ApprovedResults>().await?)?;
                        imp::same(&approved["biologist"], &verification)?;
                        let proof = json!({"sample":sample,"preparation":preparation,"calibration":calibration,
                            "arguments":arguments,"result":operation,"qc":approved});
                        let bytes = imp::encode(&proof)?;
                        task.send::<ApprovedBiology>(&bytes.as_slice()).await?;
                        Ok(())
                    };
                    loop {
                        match guest.next_file::<ClosedFile, BIOLOGIST>(&mut private, SERVICE_FD).await? {
                            FileBoundary::Exit(exit) => {
                                let code = exit.status() as i32;
                                private.send::<IoExit>(&code).await?;
                                break Ok(code);
                            }
                            FileBoundary::Closed { pending, .. } => {
                                let reply = json!({"status":"out_of_order","error":"the biological candidate is already approved"});
                                let bytes = imp::encode(&reply)?;
                                for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
                                    private.send::<ResultChunkMsg>(&ResultChunk::new(index * protocol::WASIP1_IO_CHUNK_CAPACITY, chunk)?).await?;
                                }
                                guest = pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(protocol::FdClosed::new_with_errno(SERVICE_FD, 8))))?;
                            }
                        }
                    }
                }.await;
            }
            80 => {
                branch.recv::<ChemistApproved>().await?;
                let approval = loop {
                    let branch = task.offer().await?;
                    match branch.label() {
                        55 => {
                            let selected = imp::decode(branch.recv::<LastBiologist>().await?)?;
                            imp::qc_arguments(&selected, "BIOLOGIST")?;

                            let report = json!({"sample":sample,"preparation":preparation,"calibration":calibration,
 "arguments":arguments,"result":operation});
                            let bytes = imp::encode(&report)?;
                            task.send::<BiologyReport>(&bytes.as_slice()).await?;
                        }
                        87 => {
                            let verification =
                                imp::decode(branch.recv::<LastBiologistRejected>().await?)?;
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

                            let (returned, call) = async {
                                async { Ok(task.recv::<BioAccess>().await?) };
                                async {
                                    match guest
                                        .next_file::<ClosedFile, BIOLOGIST>(
                                            &mut private,
                                            SERVICE_FD,
                                        )
                                        .await?
                                    {
                                        FileBoundary::Exit(_) => Err(ShellError::message(
                                            "biologist exited before actual approval",
                                        )),
                                        call => Ok(call),
                                    }
                                };
                            }
                            .await?;
                            biology = returned;
                            (pending, operation, arguments) = {
                                let mut call = call;
                                loop {
                                    match call {
                                        FileBoundary::Exit(_) => {
                                            return Err(ShellError::message(
                                                "biologist exited before the actual trial",
                                            ));
                                        }
                                        FileBoundary::Closed { pending, file } => {
                                            let requested = imp::assay_file(
                                                &file,
                                                &sample["synthesis"]["data"]["compound"],
                                            );
                                            match requested {
                                                Ok(arguments) => {
                                                    let admitted = json!({"arguments":arguments,"purification":sample["purification"],
          "preparation":preparation,"calibration":calibration});
                                                    let bytes = imp::encode(&admitted)?;
                                                    biology
                                                        .send::<LastRetest>(&bytes.as_slice())
                                                        .await?;
                                                    task.send::<BioAccess>(biology).await?;
                                                    let branch = task.offer().await?;
                                                    match branch.label() {
                                                        111 => {
                                                            let result = imp::decode(
                                                                branch
                                                                    .recv::<LastRetestPassed>()
                                                                    .await?,
                                                            )?;
                                                            break (pending, result, arguments);
                                                        }
                                                        112 => {
                                                            let result = imp::decode(
                                                                branch
                                                                    .recv::<LastRetestFailed>()
                                                                    .await?,
                                                            )?;

                                                            let bytes = imp::encode(&result)?;
                                                            for(index,chunk)in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate(){
 private.send::<ResultChunkMsg>(&ResultChunk::new(index*protocol::WASIP1_IO_CHUNK_CAPACITY,chunk)?).await?;
}
                                                            guest=pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(protocol::FdClosed::new_with_errno(SERVICE_FD,0))))?;

                                                            let(returned,next_call)=async {
         async {Ok(task.recv::<BioAccess>().await?)};
         async {
match guest.next_file::<ClosedFile,BIOLOGIST>(&mut private,SERVICE_FD).await?{
 FileBoundary::Exit(_)=>Err(ShellError::message("biologist exited before actual approval")),
 call=>Ok(call),
}
};
        }.await?;
                                                            biology = returned;
                                                            call = next_call;
                                                            continue;
                                                        }
                                                        label => {
                                                            return Err(ShellError::message(
                                                                format!(
                                                                    "unexpected actual trial result {label}"
                                                                ),
                                                            ));
                                                        }
                                                    }
                                                }
                                                Err(error) => {
                                                    let reply = json!({"status":"invalid_arguments","error":error.to_string()});

                                                    let bytes = imp::encode(&reply)?;
                                                    for (index, chunk) in bytes
                                                        .chunks(protocol::WASIP1_IO_CHUNK_CAPACITY)
                                                        .enumerate()
                                                    {
                                                        private.send::<ResultChunkMsg>(&ResultChunk::new(index*protocol::WASIP1_IO_CHUNK_CAPACITY,chunk)?).await?;
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
                                        .next_file::<ClosedFile, BIOLOGIST>(
                                            &mut private,
                                            SERVICE_FD,
                                        )
                                        .await?;
                                }
                            };
                        }
                        85 => {
                            let verification =
                                imp::decode(branch.recv::<LastBiologistApproved>().await?)?;
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
                                "unexpected remaining biological QC result {label}"
                            )));
                        }
                    }
                };
                break approval;
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected biological QC result {label}"
                )));
            }
        }
    };
    // Biological domain work is complete. Keep private I/O live while the real
    // remaining approval arrives and this participant forwards the actual data.
    async {
  async {
   let approved=imp::decode(task.recv::<ApprovedResults>().await?)?;
   imp::same(&approved["biologist"],&approval)?;
   let proof=json!({"sample":sample,"preparation":preparation,"calibration":calibration,
    "arguments":arguments,"result":operation,"qc":approved});
   let bytes=imp::encode(&proof)?;
   task.send::<ApprovedBiology>(&bytes.as_slice()).await?;
   Ok(())
  };
  loop {
   match guest.next_file::<ClosedFile,BIOLOGIST>(&mut private,SERVICE_FD).await?{
    FileBoundary::Exit(exit)=>{
     let code=exit.status() as i32;
     private.send::<IoExit>(&code).await?;
     break Ok(code);
    }
    FileBoundary::Closed{pending,..}=>{
     let reply=json!({"status":"out_of_order","error":"the biological candidate is already approved"});

let bytes=imp::encode(&reply)?;
for(index,chunk)in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate(){
 private.send::<ResultChunkMsg>(&ResultChunk::new(index*protocol::WASIP1_IO_CHUNK_CAPACITY,chunk)?).await?;
}
guest=pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(protocol::FdClosed::new_with_errno(SERVICE_FD,8))))?;

    }
   }
  }
 }.await
}
