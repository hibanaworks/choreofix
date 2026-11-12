//! QC chooses requests from its own Wasm guest and forwards actual results.
use super::super::{
    global::*,
    imp::{self, QcReply},
};
use crate::shell::{
    ShellError, ShellResult,
    global::Exit as IoExit,
    imp::file_wire::{ClosedFile, ResultChunk, ResultChunkMsg, SERVICE_FD},
};
use hibana_wasip1_runtime::{FileBoundary, HibanaWasiGuest, WasiImportCompletion, protocol};
use serde_json::{Value, json};
use std::cell::Cell;

#[hibana_runtime::scope]
pub async fn run(
    mut guest: &mut HibanaWasiGuest,
    endpoint: &mut hibana::Endpoint<QC>,
    selection: &Cell<Value>,
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    let (mut chemistry, call) = async {
        async { Ok(task.recv::<ChemAccess>().await?) };
        async {
            match guest
                .next_file::<ClosedFile, QC>(&mut private, SERVICE_FD)
                .await?
            {
                FileBoundary::Exit(_) => Err(ShellError::message(
                    "QC exited before both actual approvals",
                )),
                call => Ok(call),
            }
        };
    }
    .await?;
    let (mut pending, mut selected) = {
        let mut call = call;
        loop {
            match call {
                FileBoundary::Exit(_) => {
                    return Err(ShellError::message(
                        "QC exited before both actual approvals",
                    ));
                }
                FileBoundary::Closed { pending, file } => {
                    let requested = imp::qc_file(&file);
                    match requested {
                        Ok(arguments) => break (pending, arguments),
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
                .next_file::<ClosedFile, QC>(&mut private, SERVICE_FD)
                .await?;
        }
    };
    selection.set(selected.clone());
    let (mut biology, mut checked) = if selected["scientist"] == "CHEMIST" {
        {
            let bytes = imp::encode(&selected)?;
            task.send::<FirstChemist>(&bytes.as_slice()).await?;
            task.send::<FirstChemist>(&bytes.as_slice()).await?;
            task.send::<FirstChemist>(&bytes.as_slice()).await?;
            task.send::<FirstChemist>(&bytes.as_slice()).await?;
        }
        async {
            let [mut trials, mut verification] = task.par()?;
            async { Ok(trials.recv::<BioAccess>().await?) };
            async {
                Ok({
                    let report = imp::decode(verification.recv::<ChemistReport>().await?)?;
                    let admitted = json!({"arguments":selected,"sample":report});
                    let bytes = imp::encode(&admitted)?;
                    chemistry.send::<QcAdmission>(&bytes.as_slice()).await?;
                    let (mut spec, mut fume) = async {
                        let [mut s, mut f] = verification.par()?;
                        async { Ok(s.recv::<SpecLoan>().await?) };
                        async { Ok(f.recv::<FumeLoan>().await?) };
                    }
                    .await?;
                    let bytes = imp::encode(&admitted)?;
                    let (spec_result, fume_result) = async {
                        async {
                            spec.send::<QcSpecRequest>(&bytes.as_slice()).await?;
                            imp::decode(spec.recv::<QcSpecResult>().await?)
                        };
                        async {
                            fume.send::<QcFumeRequest>(&bytes.as_slice()).await?;
                            imp::decode(fume.recv::<QcFumeResult>().await?)
                        };
                    }
                    .await?;
                    imp::same(&spec_result, &fume_result)?;
                    imp::qc_result(&spec_result, &selected)?;
                    async {
                        let [mut s, mut f] = verification.par()?;
                        async {
                            s.send::<SpecLoan>(spec).await?;
                        };
                        async {
                            f.send::<FumeLoan>(fume).await?;
                        };
                        Ok(())
                    }
                    .await?;
                    spec_result
                })
            };
        }
        .await?
    } else {
        {
            let bytes = imp::encode(&selected)?;
            task.send::<FirstBiologist>(&bytes.as_slice()).await?;
            task.send::<FirstBiologist>(&bytes.as_slice()).await?;
            task.send::<FirstBiologist>(&bytes.as_slice()).await?;
            task.send::<FirstBiologist>(&bytes.as_slice()).await?;
        }
        let mut returned = task.recv::<BioAccess>().await?;
        let checked = {
            let report = imp::decode(task.recv::<BiologyReport>().await?)?;
            let admitted =
                json!({"arguments":selected,"sample":report["sample"],"result":report["result"]});
            let bytes = imp::encode(&admitted)?;
            returned.send::<QcAdmission>(&bytes.as_slice()).await?;
            let (mut spec, mut fume) = async {
                let [mut s, mut f] = task.par()?;
                async { Ok(s.recv::<SpecLoan>().await?) };
                async { Ok(f.recv::<FumeLoan>().await?) };
            }
            .await?;
            let bytes = imp::encode(&admitted)?;
            let (spec_result, fume_result) = async {
                async {
                    spec.send::<QcSpecRequest>(&bytes.as_slice()).await?;
                    imp::decode(spec.recv::<QcSpecResult>().await?)
                };
                async {
                    fume.send::<QcFumeRequest>(&bytes.as_slice()).await?;
                    imp::decode(fume.recv::<QcFumeResult>().await?)
                };
            }
            .await?;
            imp::same(&spec_result, &fume_result)?;
            imp::qc_result(&spec_result, &selected)?;
            async {
                let [mut s, mut f] = task.par()?;
                async {
                    s.send::<SpecLoan>(spec).await?;
                };
                async {
                    f.send::<FumeLoan>(fume).await?;
                };
                Ok(())
            }
            .await?;
            spec_result
        };
        (returned, checked)
    };
    let first_approval = loop {
        async {
            let [mut c, mut b] = task.par()?;
            async {
                c.send::<ChemAccess>(chemistry).await?;
            };
            async {
                b.send::<BioAccess>(biology).await?;
            };
            Ok(())
        }
        .await?;
        let branch = task.offer().await?;
        let result = match branch.label() {
            80 => imp::decode(branch.recv::<ChemistApproved>().await?)?,
            81 => imp::decode(branch.recv::<ChemistRejected>().await?)?,
            82 => imp::decode(branch.recv::<BiologistApproved>().await?)?,
            83 => imp::decode(branch.recv::<BiologistRejected>().await?)?,
            label => return Err(ShellError::message(format!("unexpected QC result {label}"))),
        };
        imp::same(&result, &checked)?;
        (chemistry, biology) = async {
            let [mut c, mut b] = task.par()?;
            async { Ok(c.recv::<ChemAccess>().await?) };
            async { Ok(b.recv::<BioAccess>().await?) };
        }
        .await?;
        match (
            result["success"].as_bool(),
            result["data"]["scientist"].as_str(),
        ) {
            (Some(true), Some("CHEMIST")) => {
                let bytes = imp::encode(&result)?;
                let reply = QcReply::<CHEMIST, true>::from_bytes(&bytes)?;
                task.send::<ChemistApproved>(&reply).await?;
                task.send::<ChemistApproved>(&reply).await?;
                task.send::<ChemistApproved>(&reply).await?;
                task.send::<ChemistApproved>(&reply).await?;
            }
            (Some(false), Some("CHEMIST")) => {
                let bytes = imp::encode(&result)?;
                let reply = QcReply::<CHEMIST, false>::from_bytes(&bytes)?;
                task.send::<ChemistRejected>(&reply).await?;
                task.send::<ChemistRejected>(&reply).await?;
                task.send::<ChemistRejected>(&reply).await?;
                task.send::<ChemistRejected>(&reply).await?;
            }
            (Some(true), Some("BIOLOGIST")) => {
                let bytes = imp::encode(&result)?;
                let reply = QcReply::<BIOLOGIST, true>::from_bytes(&bytes)?;
                task.send::<BiologistApproved>(&reply).await?;
                task.send::<BiologistApproved>(&reply).await?;
                task.send::<BiologistApproved>(&reply).await?;
                task.send::<BiologistApproved>(&reply).await?;
            }
            (Some(false), Some("BIOLOGIST")) => {
                let bytes = imp::encode(&result)?;
                let reply = QcReply::<BIOLOGIST, false>::from_bytes(&bytes)?;
                task.send::<BiologistRejected>(&reply).await?;
                task.send::<BiologistRejected>(&reply).await?;
                task.send::<BiologistRejected>(&reply).await?;
                task.send::<BiologistRejected>(&reply).await?;
            }
            _ => {
                return Err(ShellError::message(
                    "actual QC result has no subject or verdict",
                ));
            }
        }

        let bytes = imp::encode(&result)?;
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

        if result["success"] == true {
            break result;
        }
        match result["data"]["scientist"].as_str() {
            Some("CHEMIST") => {
                task.send::<ChemAccess>(chemistry).await?;
                let (returned, call) = async {
                    async { Ok(task.recv::<ChemAccess>().await?) };
                    async {
                        match guest
                            .next_file::<ClosedFile, QC>(&mut private, SERVICE_FD)
                            .await?
                        {
                            FileBoundary::Exit(_) => Err(ShellError::message(
                                "QC exited before both actual approvals",
                            )),
                            call => Ok(call),
                        }
                    };
                }
                .await?;
                chemistry = returned;
                (pending, selected) = {
                    let mut call = call;
                    loop {
                        match call {
                            FileBoundary::Exit(_) => {
                                return Err(ShellError::message(
                                    "QC exited before both actual approvals",
                                ));
                            }
                            FileBoundary::Closed { pending, file } => {
                                let requested = imp::qc_file(&file);
                                match requested {
                                    Ok(arguments) => break (pending, arguments),
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
                            .next_file::<ClosedFile, QC>(&mut private, SERVICE_FD)
                            .await?;
                    }
                };
                selection.set(selected.clone());
                if selected["scientist"] == "CHEMIST" {
                    {
                        let bytes = imp::encode(&selected)?;
                        task.send::<NextChemist>(&bytes.as_slice()).await?;
                        task.send::<NextChemist>(&bytes.as_slice()).await?;
                        task.send::<NextChemist>(&bytes.as_slice()).await?;
                        task.send::<NextChemist>(&bytes.as_slice()).await?;
                    }
                    checked = {
                        let report = imp::decode(task.recv::<ChemistReport>().await?)?;
                        let admitted = json!({"arguments":selected,"sample":report});
                        let bytes = imp::encode(&admitted)?;
                        chemistry.send::<QcAdmission>(&bytes.as_slice()).await?;
                        let (mut spec, mut fume) = async {
                            let [mut s, mut f] = task.par()?;
                            async { Ok(s.recv::<SpecLoan>().await?) };
                            async { Ok(f.recv::<FumeLoan>().await?) };
                        }
                        .await?;
                        let bytes = imp::encode(&admitted)?;
                        let (spec_result, fume_result) = async {
                            async {
                                spec.send::<QcSpecRequest>(&bytes.as_slice()).await?;
                                imp::decode(spec.recv::<QcSpecResult>().await?)
                            };
                            async {
                                fume.send::<QcFumeRequest>(&bytes.as_slice()).await?;
                                imp::decode(fume.recv::<QcFumeResult>().await?)
                            };
                        }
                        .await?;
                        imp::same(&spec_result, &fume_result)?;
                        imp::qc_result(&spec_result, &selected)?;
                        async {
                            let [mut s, mut f] = task.par()?;
                            async {
                                s.send::<SpecLoan>(spec).await?;
                            };
                            async {
                                f.send::<FumeLoan>(fume).await?;
                            };
                            Ok(())
                        }
                        .await?;
                        spec_result
                    };
                } else {
                    {
                        let bytes = imp::encode(&selected)?;
                        task.send::<NextBiologist>(&bytes.as_slice()).await?;
                        task.send::<NextBiologist>(&bytes.as_slice()).await?;
                        task.send::<NextBiologist>(&bytes.as_slice()).await?;
                        task.send::<NextBiologist>(&bytes.as_slice()).await?;
                    }
                    checked = {
                        let report = imp::decode(task.recv::<BiologyReport>().await?)?;
                        let admitted = json!({"arguments":selected,"sample":report["sample"],"result":report["result"]});
                        let bytes = imp::encode(&admitted)?;
                        biology.send::<QcAdmission>(&bytes.as_slice()).await?;
                        let (mut spec, mut fume) = async {
                            let [mut s, mut f] = task.par()?;
                            async { Ok(s.recv::<SpecLoan>().await?) };
                            async { Ok(f.recv::<FumeLoan>().await?) };
                        }
                        .await?;
                        let bytes = imp::encode(&admitted)?;
                        let (spec_result, fume_result) = async {
                            async {
                                spec.send::<QcSpecRequest>(&bytes.as_slice()).await?;
                                imp::decode(spec.recv::<QcSpecResult>().await?)
                            };
                            async {
                                fume.send::<QcFumeRequest>(&bytes.as_slice()).await?;
                                imp::decode(fume.recv::<QcFumeResult>().await?)
                            };
                        }
                        .await?;
                        imp::same(&spec_result, &fume_result)?;
                        imp::qc_result(&spec_result, &selected)?;
                        async {
                            let [mut s, mut f] = task.par()?;
                            async {
                                s.send::<SpecLoan>(spec).await?;
                            };
                            async {
                                f.send::<FumeLoan>(fume).await?;
                            };
                            Ok(())
                        }
                        .await?;
                        spec_result
                    };
                }
            }
            Some("BIOLOGIST") => {
                task.send::<BioAccess>(biology).await?;
                (pending, selected) = {
                    let mut call = guest
                        .next_file::<ClosedFile, QC>(&mut private, SERVICE_FD)
                        .await?;
                    loop {
                        match call {
                            FileBoundary::Exit(_) => {
                                return Err(ShellError::message(
                                    "QC exited before both actual approvals",
                                ));
                            }
                            FileBoundary::Closed { pending, file } => {
                                let requested = imp::qc_file(&file);
                                match requested {
                                    Ok(arguments) => break (pending, arguments),
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
                            .next_file::<ClosedFile, QC>(&mut private, SERVICE_FD)
                            .await?;
                    }
                };
                selection.set(selected.clone());
                let (returned, next_checked) = if selected["scientist"] == "CHEMIST" {
                    {
                        let bytes = imp::encode(&selected)?;
                        task.send::<NextChemist>(&bytes.as_slice()).await?;
                        task.send::<NextChemist>(&bytes.as_slice()).await?;
                        task.send::<NextChemist>(&bytes.as_slice()).await?;
                        task.send::<NextChemist>(&bytes.as_slice()).await?;
                    }
                    async {
                        let [mut trials, mut verification] = task.par()?;
                        async { Ok(trials.recv::<BioAccess>().await?) };
                        async {
                            Ok({
                                let report =
                                    imp::decode(verification.recv::<ChemistReport>().await?)?;
                                let admitted = json!({"arguments":selected,"sample":report});
                                let bytes = imp::encode(&admitted)?;
                                chemistry.send::<QcAdmission>(&bytes.as_slice()).await?;
                                let (mut spec, mut fume) = async {
                                    let [mut s, mut f] = verification.par()?;
                                    async { Ok(s.recv::<SpecLoan>().await?) };
                                    async { Ok(f.recv::<FumeLoan>().await?) };
                                }
                                .await?;
                                let bytes = imp::encode(&admitted)?;
                                let (spec_result, fume_result) = async {
                                    async {
                                        spec.send::<QcSpecRequest>(&bytes.as_slice()).await?;
                                        imp::decode(spec.recv::<QcSpecResult>().await?)
                                    };
                                    async {
                                        fume.send::<QcFumeRequest>(&bytes.as_slice()).await?;
                                        imp::decode(fume.recv::<QcFumeResult>().await?)
                                    };
                                }
                                .await?;
                                imp::same(&spec_result, &fume_result)?;
                                imp::qc_result(&spec_result, &selected)?;
                                async {
                                    let [mut s, mut f] = verification.par()?;
                                    async {
                                        s.send::<SpecLoan>(spec).await?;
                                    };
                                    async {
                                        f.send::<FumeLoan>(fume).await?;
                                    };
                                    Ok(())
                                }
                                .await?;
                                spec_result
                            })
                        };
                    }
                    .await?
                } else {
                    {
                        let bytes = imp::encode(&selected)?;
                        task.send::<NextBiologist>(&bytes.as_slice()).await?;
                        task.send::<NextBiologist>(&bytes.as_slice()).await?;
                        task.send::<NextBiologist>(&bytes.as_slice()).await?;
                        task.send::<NextBiologist>(&bytes.as_slice()).await?;
                    }
                    let mut returned = task.recv::<BioAccess>().await?;
                    let checked = {
                        let report = imp::decode(task.recv::<BiologyReport>().await?)?;
                        let admitted = json!({"arguments":selected,"sample":report["sample"],"result":report["result"]});
                        let bytes = imp::encode(&admitted)?;
                        returned.send::<QcAdmission>(&bytes.as_slice()).await?;
                        let (mut spec, mut fume) = async {
                            let [mut s, mut f] = task.par()?;
                            async { Ok(s.recv::<SpecLoan>().await?) };
                            async { Ok(f.recv::<FumeLoan>().await?) };
                        }
                        .await?;
                        let bytes = imp::encode(&admitted)?;
                        let (spec_result, fume_result) = async {
                            async {
                                spec.send::<QcSpecRequest>(&bytes.as_slice()).await?;
                                imp::decode(spec.recv::<QcSpecResult>().await?)
                            };
                            async {
                                fume.send::<QcFumeRequest>(&bytes.as_slice()).await?;
                                imp::decode(fume.recv::<QcFumeResult>().await?)
                            };
                        }
                        .await?;
                        imp::same(&spec_result, &fume_result)?;
                        imp::qc_result(&spec_result, &selected)?;
                        async {
                            let [mut s, mut f] = task.par()?;
                            async {
                                s.send::<SpecLoan>(spec).await?;
                            };
                            async {
                                f.send::<FumeLoan>(fume).await?;
                            };
                            Ok(())
                        }
                        .await?;
                        spec_result
                    };
                    (returned, checked)
                };
                biology = returned;
                checked = next_checked;
            }
            _ => return Err(ShellError::message("unknown rejected QC subject")),
        }
    };
    let approved = match first_approval["data"]["scientist"].as_str() {
        Some("CHEMIST") => {
            let mut call = guest
                .next_file::<ClosedFile, QC>(&mut private, SERVICE_FD)
                .await?;
            let last_approval = loop {
                let (pending, selected) = {
                    let mut call = call;
                    loop {
                        match call {
                            FileBoundary::Exit(_) => {
                                return Err(ShellError::message(
                                    "QC exited before both actual approvals",
                                ));
                            }
                            FileBoundary::Closed { pending, file } => {
                                let requested = imp::qc_file(&file).and_then(|arguments| {
                                    imp::qc_arguments(&arguments, "BIOLOGIST")?;
                                    Ok(arguments)
                                });
                                match requested {
                                    Ok(arguments) => break (pending, arguments),
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
                            .next_file::<ClosedFile, QC>(&mut private, SERVICE_FD)
                            .await?;
                    }
                };
                {
                    let bytes = imp::encode(&selected)?;
                    task.send::<LastBiologist>(&bytes.as_slice()).await?;
                    task.send::<LastBiologist>(&bytes.as_slice()).await?;
                    task.send::<LastBiologist>(&bytes.as_slice()).await?;
                }
                let checked = {
                    let report = imp::decode(task.recv::<BiologyReport>().await?)?;
                    let admitted = json!({"arguments":selected,"sample":report["sample"],"result":report["result"]});
                    let bytes = imp::encode(&admitted)?;
                    biology.send::<QcAdmission>(&bytes.as_slice()).await?;
                    let (mut spec, mut fume) = async {
                        let [mut s, mut f] = task.par()?;
                        async { Ok(s.recv::<SpecLoan>().await?) };
                        async { Ok(f.recv::<FumeLoan>().await?) };
                    }
                    .await?;
                    let bytes = imp::encode(&admitted)?;
                    let (spec_result, fume_result) = async {
                        async {
                            spec.send::<QcSpecRequest>(&bytes.as_slice()).await?;
                            imp::decode(spec.recv::<QcSpecResult>().await?)
                        };
                        async {
                            fume.send::<QcFumeRequest>(&bytes.as_slice()).await?;
                            imp::decode(fume.recv::<QcFumeResult>().await?)
                        };
                    }
                    .await?;
                    imp::same(&spec_result, &fume_result)?;
                    imp::qc_result(&spec_result, &selected)?;
                    async {
                        let [mut s, mut f] = task.par()?;
                        async {
                            s.send::<SpecLoan>(spec).await?;
                        };
                        async {
                            f.send::<FumeLoan>(fume).await?;
                        };
                        Ok(())
                    }
                    .await?;
                    spec_result
                };
                task.send::<BioAccess>(biology).await?;
                let branch = task.offer().await?;
                let result = match branch.label() {
                    85 => imp::decode(branch.recv::<LastBiologistApproved>().await?)?,
                    87 => imp::decode(branch.recv::<LastBiologistRejected>().await?)?,
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected remaining QC result {label}"
                        )));
                    }
                };
                imp::same(&result, &checked)?;
                biology = task.recv::<BioAccess>().await?;
                match result["success"].as_bool() {
                    Some(true) => {
                        let bytes = imp::encode(&result)?;
                        let reply = QcReply::<BIOLOGIST, true>::from_bytes(&bytes)?;
                        task.send::<LastBiologistApproved>(&reply).await?;
                        task.send::<LastBiologistApproved>(&reply).await?;
                        task.send::<LastBiologistApproved>(&reply).await?;
                        task.send::<LastBiologistApproved>(&reply).await?;
                    }
                    Some(false) => {
                        let bytes = imp::encode(&result)?;
                        let reply = QcReply::<BIOLOGIST, false>::from_bytes(&bytes)?;
                        task.send::<LastBiologistRejected>(&reply).await?;
                        task.send::<LastBiologistRejected>(&reply).await?;
                        task.send::<LastBiologistRejected>(&reply).await?;
                        task.send::<LastBiologistRejected>(&reply).await?;
                    }
                    None => return Err(ShellError::message("actual QC result has no verdict")),
                }

                let bytes = imp::encode(&result)?;
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

                if result["success"] == true {
                    break result;
                }
                task.send::<BioAccess>(biology).await?;
                let (returned, next_call) = async {
                    async { Ok(task.recv::<BioAccess>().await?) };
                    async {
                        match guest
                            .next_file::<ClosedFile, QC>(&mut private, SERVICE_FD)
                            .await?
                        {
                            FileBoundary::Exit(_) => Err(ShellError::message(
                                "QC exited before both actual approvals",
                            )),
                            call => Ok(call),
                        }
                    };
                }
                .await?;
                biology = returned;
                call = next_call;
            };
            json!({"chemist":first_approval,"biologist":last_approval})
        }
        Some("BIOLOGIST") => {
            let mut call = guest
                .next_file::<ClosedFile, QC>(&mut private, SERVICE_FD)
                .await?;
            let last_approval = loop {
                let (pending, selected) = {
                    let mut call = call;
                    loop {
                        match call {
                            FileBoundary::Exit(_) => {
                                return Err(ShellError::message(
                                    "QC exited before both actual approvals",
                                ));
                            }
                            FileBoundary::Closed { pending, file } => {
                                let requested = imp::qc_file(&file).and_then(|arguments| {
                                    imp::qc_arguments(&arguments, "CHEMIST")?;
                                    Ok(arguments)
                                });
                                match requested {
                                    Ok(arguments) => break (pending, arguments),
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
                            .next_file::<ClosedFile, QC>(&mut private, SERVICE_FD)
                            .await?;
                    }
                };
                {
                    let bytes = imp::encode(&selected)?;
                    task.send::<LastChemist>(&bytes.as_slice()).await?;
                    task.send::<LastChemist>(&bytes.as_slice()).await?;
                    task.send::<LastChemist>(&bytes.as_slice()).await?;
                }
                let checked = {
                    let report = imp::decode(task.recv::<ChemistReport>().await?)?;
                    let admitted = json!({"arguments":selected,"sample":report});
                    let bytes = imp::encode(&admitted)?;
                    chemistry.send::<QcAdmission>(&bytes.as_slice()).await?;
                    let (mut spec, mut fume) = async {
                        let [mut s, mut f] = task.par()?;
                        async { Ok(s.recv::<SpecLoan>().await?) };
                        async { Ok(f.recv::<FumeLoan>().await?) };
                    }
                    .await?;
                    let bytes = imp::encode(&admitted)?;
                    let (spec_result, fume_result) = async {
                        async {
                            spec.send::<QcSpecRequest>(&bytes.as_slice()).await?;
                            imp::decode(spec.recv::<QcSpecResult>().await?)
                        };
                        async {
                            fume.send::<QcFumeRequest>(&bytes.as_slice()).await?;
                            imp::decode(fume.recv::<QcFumeResult>().await?)
                        };
                    }
                    .await?;
                    imp::same(&spec_result, &fume_result)?;
                    imp::qc_result(&spec_result, &selected)?;
                    async {
                        let [mut s, mut f] = task.par()?;
                        async {
                            s.send::<SpecLoan>(spec).await?;
                        };
                        async {
                            f.send::<FumeLoan>(fume).await?;
                        };
                        Ok(())
                    }
                    .await?;
                    spec_result
                };
                task.send::<ChemAccess>(chemistry).await?;
                let branch = task.offer().await?;
                let result = match branch.label() {
                    84 => imp::decode(branch.recv::<LastChemistApproved>().await?)?,
                    86 => imp::decode(branch.recv::<LastChemistRejected>().await?)?,
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected remaining QC result {label}"
                        )));
                    }
                };
                imp::same(&result, &checked)?;
                chemistry = task.recv::<ChemAccess>().await?;
                match result["success"].as_bool() {
                    Some(true) => {
                        let bytes = imp::encode(&result)?;
                        let reply = QcReply::<CHEMIST, true>::from_bytes(&bytes)?;
                        task.send::<LastChemistApproved>(&reply).await?;
                        task.send::<LastChemistApproved>(&reply).await?;
                        task.send::<LastChemistApproved>(&reply).await?;
                        task.send::<LastChemistApproved>(&reply).await?;
                    }
                    Some(false) => {
                        let bytes = imp::encode(&result)?;
                        let reply = QcReply::<CHEMIST, false>::from_bytes(&bytes)?;
                        task.send::<LastChemistRejected>(&reply).await?;
                        task.send::<LastChemistRejected>(&reply).await?;
                        task.send::<LastChemistRejected>(&reply).await?;
                        task.send::<LastChemistRejected>(&reply).await?;
                    }
                    None => return Err(ShellError::message("actual QC result has no verdict")),
                }

                let bytes = imp::encode(&result)?;
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

                if result["success"] == true {
                    break result;
                }
                task.send::<ChemAccess>(chemistry).await?;
                let (returned, next_call) = async {
                    async { Ok(task.recv::<ChemAccess>().await?) };
                    async {
                        match guest
                            .next_file::<ClosedFile, QC>(&mut private, SERVICE_FD)
                            .await?
                        {
                            FileBoundary::Exit(_) => Err(ShellError::message(
                                "QC exited before both actual approvals",
                            )),
                            call => Ok(call),
                        }
                    };
                }
                .await?;
                chemistry = returned;
                call = next_call;
            };
            json!({"chemist":last_approval,"biologist":first_approval})
        }
        _ => return Err(ShellError::message("unknown approved QC subject")),
    };
    let bytes = imp::encode(&approved)?;
    task.send::<ApprovedResults>(&bytes.as_slice()).await?;
    drop((task, chemistry, biology));
    loop {
        match guest
            .next_file::<ClosedFile, QC>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Exit(exit) => {
                let code = exit.status() as i32;
                private.send::<IoExit>(&code).await?;
                return Ok(code);
            }
            FileBoundary::Closed { pending, .. } => {
                let reply = json!({"status":"out_of_order","error":"both scientific candidates are already approved"});

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
