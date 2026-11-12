//! The scientist writes Endpoint operations directly, using actual guest input.
use super::super::{global::*, imp};
use crate::shell::{
    ShellError, ShellResult,
    global::Exit as IoExit,
    imp::file_wire::{ClosedFile, ResultChunk, ResultChunkMsg, SERVICE_FD},
};
use hibana::{Endpoint, g::Message};
use hibana_wasip1_runtime::{FileBoundary, HibanaWasiGuest, WasiImportCompletion, protocol};
use serde_json::Value;
use serde_json::json;
use std::cell::Cell;
#[hibana_runtime::scope]
pub async fn run<'r>(
    guest: &mut HibanaWasiGuest,
    (endpoint, mut structural_specimen, mut study): (
        &mut Endpoint<'r, LEAD>,
        Endpoint<'r, STRUCTURAL>,
        Endpoint<'r, STUDY>,
    ),
    selected: &Cell<Value>,
) -> ShellResult<i32> {
    let [mut task, private] = endpoint.par()?;
    let (_qc,_culture,(reference,mut guest,mut private))=async{
 let [mut qc_task,mut culture_task,mut reference_task]=task.par()?;
 async{Ok(qc_task.recv::<Specimen<QC>>().await?)};
 async{Ok(culture_task.recv::<Specimen<CULTURE>>().await?)};
 async move{
let mut guest=guest;
let mut private=private;
let reference={let mut call=guest.next_file::<ClosedFile,LEAD>(&mut private,SERVICE_FD).await?;loop{match call{
 FileBoundary::Exit(_)=>return Err(ShellError::message("lead exited before completing the study")),
 FileBoundary::Closed{pending,file}=>{match imp::analysis_file::<3>(&file){
 Err(error)=>{let result=json!({"status":"invalid_arguments","error":error.to_string()});let bytes=imp::encode(&result)?;
for (index,chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate(){ private.send::<ResultChunkMsg>(&ResultChunk::new(index*protocol::WASIP1_IO_CHUNK_CAPACITY,chunk)?).await?;}
guest=pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(protocol::FdClosed::new_with_errno(SERVICE_FD,28))))?;
},
 Ok(arguments)=>{
 let bytes=imp::encode(&arguments)?;let input=imp::AnalysisArguments::<3>::from_bytes(&bytes)?;
 reference_task.send::<MassIntent<3>>(&input).await?;
 let mut instrument=reference_task.recv::<Instrument<MASS>>().await?;
 instrument.send::<MassRequest<3>>(&input).await?;
 let result=imp::decode(instrument.recv::<MassResult>().await?)?;imp::successful(&result)?;
 reference_task.send::<Instrument<MASS>>(instrument).await?;
 let bytes=imp::encode(&result)?;
for (index,chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate(){ private.send::<ResultChunkMsg>(&ResultChunk::new(index*protocol::WASIP1_IO_CHUNK_CAPACITY,chunk)?).await?;}
guest=pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(protocol::FdClosed::new_with_errno(SERVICE_FD,0))))?;
break result;
 }
 }}
 }call=guest.next_file::<ClosedFile,LEAD>(&mut private,SERVICE_FD).await?;
}};
Ok((reference,guest,private))
 };
}.await?;
    let (mut bioassay, mut purity) = async {
        let [mut bio, mut tox] = task.par()?;
        async { Ok(bio.recv::<Specimen<BIOASSAY>>().await?) };
        async { Ok(tox.recv::<Specimen<PURITY>>().await?) };
    }
    .await?;
    let (mut stability, mut toxicity) = async {
        let [mut bio, mut tox] = task.par()?;
        async { Ok(bio.recv::<Specimen<STABILITY>>().await?) };
        async { Ok(tox.recv::<Specimen<TOXICITY>>().await?) };
    }
    .await?;
    let (mut bio_report, mut tox_report) = async {
        let [mut bio, mut tox] = task.par()?;
        async { imp::decode(bio.recv::<BioReport>().await?) };
        async { imp::decode(tox.recv::<ToxReport>().await?) };
    }
    .await?;
    imp::same(&bio_report["compound"], &tox_report["compound"])?;
    let (structural, characterization, _structural_continuation) = {
        let [mut hplc_task, mut mass_task] = task.par()?;
        let mut next = guest
            .next_file::<ClosedFile, LEAD>(&mut private, SERVICE_FD)
            .await?;
        let first = loop {
            match next {
                FileBoundary::Exit(_) => {
                    return Err(ShellError::message(
                        "lead exited before its independent analyses",
                    ));
                }
                FileBoundary::Closed { pending, file } => {
                    if imp::analysis_file::<9>(&file).is_ok()
                        || imp::analysis_file::<10>(&file).is_ok()
                    {
                        break FileBoundary::Closed { pending, file };
                    }
                    let result = json!({"status":"invalid_arguments","error":"expected structural_confirmation or characterization"});
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
                        protocol::FdClosedRet(protocol::FdClosed::new_with_errno(SERVICE_FD, 28)),
                    ))?;
                }
            }
            next = guest
                .next_file::<ClosedFile, LEAD>(&mut private, SERVICE_FD)
                .await?;
        };
        match first {
            FileBoundary::Closed { pending, file } if file.path == b"service/run_hplc_analysis" => {
                let (structural, structural_continuation) = {
                    let mut call = FileBoundary::Closed { pending, file };
                    loop {
                        match call {
                            FileBoundary::Exit(_) => {
                                return Err(ShellError::message(
                                    "lead exited before the actual structural analysis",
                                ));
                            }
                            FileBoundary::Closed { pending, file } => {
                                match imp::analysis_file::<9>(&file) {
                                    Err(error) => {
                                        let result = json!({"status":"invalid_arguments","error":error.to_string()});
                                        let bytes = imp::encode(&result)?;
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
                                    Ok(arguments) => {
                                        let bytes = imp::encode(&arguments)?;
                                        let input =
                                            imp::AnalysisArguments::<9>::from_bytes(&bytes)?;
                                        structural_specimen.send::<Intent<9>>(&input).await?;
                                        let mut instrument =
                                            hplc_task.recv::<Instrument<HPLC>>().await?;
                                        instrument.send::<Request<9>>(&input).await?;
                                        hplc_task.send::<Instrument<HPLC>>(instrument).await?;
                                        hplc_task
                                            .send::<Specimen<STRUCTURAL>>(structural_specimen)
                                            .await?;
                                        let branch = hplc_task.offer().await?;
                                        match branch.label() {
                                            label if label == Passed::<9>::LOGICAL_LABEL => {
                                                let result =
                                                    imp::decode(branch.recv::<Passed<9>>().await?)?;
                                                let returned = hplc_task
                                                    .recv::<Specimen<STRUCTURAL>>()
                                                    .await?;
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
                                                break (result, returned);
                                            }
                                            label if label == Failed::<9>::LOGICAL_LABEL => {
                                                let result =
                                                    imp::decode(branch.recv::<Failed<9>>().await?)?;
                                                structural_specimen = hplc_task
                                                    .recv::<Specimen<STRUCTURAL>>()
                                                    .await?;
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
                                            }
                                            _ => {
                                                return Err(ShellError::message(
                                                    "unexpected structural result",
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        call = guest
                            .next_file::<ClosedFile, LEAD>(&mut private, SERVICE_FD)
                            .await?;
                    }
                };
                let characterization = {
                    let mut call = guest
                        .next_file::<ClosedFile, LEAD>(&mut private, SERVICE_FD)
                        .await?;
                    loop {
                        match call {
                            FileBoundary::Exit(_) => {
                                return Err(ShellError::message(
                                    "lead exited before completing the study",
                                ));
                            }
                            FileBoundary::Closed { pending, file } => {
                                match imp::analysis_file::<10>(&file) {
                                    Err(error) => {
                                        let result = json!({"status":"invalid_arguments","error":error.to_string()});
                                        let bytes = imp::encode(&result)?;
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
                                    Ok(arguments) => {
                                        let bytes = imp::encode(&arguments)?;
                                        let input =
                                            imp::AnalysisArguments::<10>::from_bytes(&bytes)?;
                                        mass_task.send::<MassIntent<10>>(&input).await?;
                                        let mut instrument =
                                            mass_task.recv::<Instrument<MASS>>().await?;
                                        instrument.send::<MassRequest<10>>(&input).await?;
                                        let result =
                                            imp::decode(instrument.recv::<MassResult>().await?)?;
                                        imp::successful(&result)?;
                                        mass_task.send::<Instrument<MASS>>(instrument).await?;
                                        let bytes = imp::encode(&result)?;
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
                                                protocol::FdClosed::new_with_errno(SERVICE_FD, 0),
                                            ),
                                        ))?;
                                        break result;
                                    }
                                }
                            }
                        }
                        call = guest
                            .next_file::<ClosedFile, LEAD>(&mut private, SERVICE_FD)
                            .await?;
                    }
                };
                (structural, characterization, structural_continuation)
            }
            FileBoundary::Closed { pending, file } => {
                let characterization = {
                    let mut call = FileBoundary::Closed { pending, file };
                    loop {
                        match call {
                            FileBoundary::Exit(_) => {
                                return Err(ShellError::message(
                                    "lead exited before completing the study",
                                ));
                            }
                            FileBoundary::Closed { pending, file } => {
                                match imp::analysis_file::<10>(&file) {
                                    Err(error) => {
                                        let result = json!({"status":"invalid_arguments","error":error.to_string()});
                                        let bytes = imp::encode(&result)?;
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
                                    Ok(arguments) => {
                                        let bytes = imp::encode(&arguments)?;
                                        let input =
                                            imp::AnalysisArguments::<10>::from_bytes(&bytes)?;
                                        mass_task.send::<MassIntent<10>>(&input).await?;
                                        let mut instrument =
                                            mass_task.recv::<Instrument<MASS>>().await?;
                                        instrument.send::<MassRequest<10>>(&input).await?;
                                        let result =
                                            imp::decode(instrument.recv::<MassResult>().await?)?;
                                        imp::successful(&result)?;
                                        mass_task.send::<Instrument<MASS>>(instrument).await?;
                                        let bytes = imp::encode(&result)?;
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
                                                protocol::FdClosed::new_with_errno(SERVICE_FD, 0),
                                            ),
                                        ))?;
                                        break result;
                                    }
                                }
                            }
                        }
                        call = guest
                            .next_file::<ClosedFile, LEAD>(&mut private, SERVICE_FD)
                            .await?;
                    }
                };
                let (structural, structural_continuation) = {
                    let mut call = guest
                        .next_file::<ClosedFile, LEAD>(&mut private, SERVICE_FD)
                        .await?;
                    loop {
                        match call {
                            FileBoundary::Exit(_) => {
                                return Err(ShellError::message(
                                    "lead exited before the actual structural analysis",
                                ));
                            }
                            FileBoundary::Closed { pending, file } => {
                                match imp::analysis_file::<9>(&file) {
                                    Err(error) => {
                                        let result = json!({"status":"invalid_arguments","error":error.to_string()});
                                        let bytes = imp::encode(&result)?;
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
                                    Ok(arguments) => {
                                        let bytes = imp::encode(&arguments)?;
                                        let input =
                                            imp::AnalysisArguments::<9>::from_bytes(&bytes)?;
                                        structural_specimen.send::<Intent<9>>(&input).await?;
                                        let mut instrument =
                                            hplc_task.recv::<Instrument<HPLC>>().await?;
                                        instrument.send::<Request<9>>(&input).await?;
                                        hplc_task.send::<Instrument<HPLC>>(instrument).await?;
                                        hplc_task
                                            .send::<Specimen<STRUCTURAL>>(structural_specimen)
                                            .await?;
                                        let branch = hplc_task.offer().await?;
                                        match branch.label() {
                                            label if label == Passed::<9>::LOGICAL_LABEL => {
                                                let result =
                                                    imp::decode(branch.recv::<Passed<9>>().await?)?;
                                                let returned = hplc_task
                                                    .recv::<Specimen<STRUCTURAL>>()
                                                    .await?;
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
                                                break (result, returned);
                                            }
                                            label if label == Failed::<9>::LOGICAL_LABEL => {
                                                let result =
                                                    imp::decode(branch.recv::<Failed<9>>().await?)?;
                                                structural_specimen = hplc_task
                                                    .recv::<Specimen<STRUCTURAL>>()
                                                    .await?;
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
                                            }
                                            _ => {
                                                return Err(ShellError::message(
                                                    "unexpected structural result",
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        call = guest
                            .next_file::<ClosedFile, LEAD>(&mut private, SERVICE_FD)
                            .await?;
                    }
                };
                (structural, characterization, structural_continuation)
            }
            FileBoundary::Exit(_) => return Err(ShellError::message("missing scientific request")),
        }
    };
    loop {
        let (pending, go) = loop {
            match guest
                .next_file::<ClosedFile, LEAD>(&mut private, SERVICE_FD)
                .await?
            {
                FileBoundary::Exit(_) => {
                    return Err(ShellError::message(
                        "lead exited before regulatory approval",
                    ));
                }
                FileBoundary::Closed { pending, file } => {
                    match imp::input_file(&file, b"service/submit_decision").and_then(|v| {
                        if v.as_object().is_some_and(|o| o.len() == 1) && v["go"].is_boolean() {
                            Ok(v["go"].clone())
                        } else {
                            Err(ShellError::message("expected one boolean go/no-go choice"))
                        }
                    }) {
                        Ok(go) => break (pending, go),
                        Err(error) => {
                            let result =
                                json!({"status":"invalid_arguments","error":error.to_string()});
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
                                    SERVICE_FD, 28,
                                )),
                            ))?;
                        }
                    }
                }
            }
        };
        let candidate = json!({"go":go,"compound":bio_report["compound"],"bio":bio_report,"tox":tox_report,
  "analyses":{"reference":reference,"structural":structural,"characterization":characterization}});
        let bytes = imp::encode(&candidate)?;
        study.send::<Decision>(&bytes.as_slice()).await?;
        let recorded = imp::decode(study.recv::<DecisionRecorded>().await?)?;
        imp::decision_receipt(&candidate, &recorded)?;
        let submission = json!({"decision":candidate,"receipt":recorded});
        let bytes = imp::encode(&submission)?;
        study.send::<Decision>(&bytes.as_slice()).await?;
        task.send::<Specimen<STUDY>>(study).await?;
        let branch = task.offer().await?;
        match branch.label() {
            label if label == ApprovedForLead::LOGICAL_LABEL => {
                let approved = imp::decode(branch.recv::<ApprovedForLead>().await?)?;
                imp::same(
                    &approved["data"]["inputs"]["decision"],
                    &recorded["data"]["artifact_id"],
                )?;
                let mut result = recorded.clone();
                result["regulatory_review"] = approved;
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
                break;
            }
            label if label == Rejected::LOGICAL_LABEL => {
                let rejected = imp::decode(branch.recv::<Rejected>().await?)?;
                imp::same(
                    &rejected["data"]["inputs"]["decision"],
                    &recorded["data"]["artifact_id"],
                )?;
                study = task.recv::<Specimen<STUDY>>().await?;
                let mut result = recorded.clone();
                result["regulatory_review"] = rejected;
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
            }
            _ => return Err(ShellError::message("unexpected lead regulatory result")),
        }
        let plan = loop {
            match guest
                .next_file::<ClosedFile, LEAD>(&mut private, SERVICE_FD)
                .await?
            {
                FileBoundary::Exit(_) => {
                    return Err(ShellError::message(
                        "lead exited instead of reconsidering the rejected decision",
                    ));
                }
                FileBoundary::Closed { pending, file } => match imp::plan_file(&file) {
                    Ok(plan) => {
                        let result = json!({"success":true,"reanalysis":plan});
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
                        break plan;
                    }
                    Err(error) => {
                        let result =
                            json!({"status":"invalid_arguments","error":error.to_string()});
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
                                SERVICE_FD, 28,
                            )),
                        ))?;
                    }
                },
            }
        };
        selected.set(plan.clone());
        let mask = u8::from(plan["biology"] == true) | (u8::from(plan["toxicology"] == true) << 1);
        let bytes = imp::encode(&plan)?;
        let branch = study.offer().await?;
        if branch.label() != mask {
            return Err(ShellError::message(
                "reanalysis branch differs from actual plan",
            ));
        }
        drop(branch);
        match mask {
            0 => {
                async {
                    let [mut bio, mut tox, mut hplc, mut cell] = study.par()?;
                    async {
                        bio.send::<Plan<0, BIOLOGIST>>(
                            &imp::ReanalysisPlan::<0, BIOLOGIST>::from_bytes(&bytes)?,
                        )
                        .await?;
                    };
                    async {
                        tox.send::<Plan<0, TOXICOLOGIST>>(
                            &imp::ReanalysisPlan::<0, TOXICOLOGIST>::from_bytes(&bytes)?,
                        )
                        .await?;
                    };
                    async {
                        hplc.send::<Plan<0, HPLC_SERVICE>>(
                            &imp::ReanalysisPlan::<0, HPLC_SERVICE>::from_bytes(&bytes)?,
                        )
                        .await?;
                    };
                    async {
                        cell.send::<Plan<0, CELL_SERVICE>>(
                            &imp::ReanalysisPlan::<0, CELL_SERVICE>::from_bytes(&bytes)?,
                        )
                        .await?;
                    };
                    Ok(())
                }
                .await?;
            }
            1 => {
                async {
                    let [mut bio, mut tox, mut hplc, mut cell] = study.par()?;
                    async {
                        bio.send::<Plan<1, BIOLOGIST>>(
                            &imp::ReanalysisPlan::<1, BIOLOGIST>::from_bytes(&bytes)?,
                        )
                        .await?;
                    };
                    async {
                        tox.send::<Plan<1, TOXICOLOGIST>>(
                            &imp::ReanalysisPlan::<1, TOXICOLOGIST>::from_bytes(&bytes)?,
                        )
                        .await?;
                    };
                    async {
                        hplc.send::<Plan<1, HPLC_SERVICE>>(
                            &imp::ReanalysisPlan::<1, HPLC_SERVICE>::from_bytes(&bytes)?,
                        )
                        .await?;
                    };
                    async {
                        cell.send::<Plan<1, CELL_SERVICE>>(
                            &imp::ReanalysisPlan::<1, CELL_SERVICE>::from_bytes(&bytes)?,
                        )
                        .await?;
                    };
                    Ok(())
                }
                .await?;
                async {
                    let [mut f0, mut f1] = task.par()?;
                    async {
                        f0.send::<Specimen<BIOASSAY>>(bioassay).await?;
                    };
                    async {
                        f1.send::<Specimen<STABILITY>>(stability).await?;
                    };
                    Ok(())
                }
                .await?;
                bioassay = task.recv::<Specimen<BIOASSAY>>().await?;
                stability = task.recv::<Specimen<STABILITY>>().await?;
            }
            2 => {
                async {
                    let [mut bio, mut tox, mut hplc, mut cell] = study.par()?;
                    async {
                        bio.send::<Plan<2, BIOLOGIST>>(
                            &imp::ReanalysisPlan::<2, BIOLOGIST>::from_bytes(&bytes)?,
                        )
                        .await?;
                    };
                    async {
                        tox.send::<Plan<2, TOXICOLOGIST>>(
                            &imp::ReanalysisPlan::<2, TOXICOLOGIST>::from_bytes(&bytes)?,
                        )
                        .await?;
                    };
                    async {
                        hplc.send::<Plan<2, HPLC_SERVICE>>(
                            &imp::ReanalysisPlan::<2, HPLC_SERVICE>::from_bytes(&bytes)?,
                        )
                        .await?;
                    };
                    async {
                        cell.send::<Plan<2, CELL_SERVICE>>(
                            &imp::ReanalysisPlan::<2, CELL_SERVICE>::from_bytes(&bytes)?,
                        )
                        .await?;
                    };
                    Ok(())
                }
                .await?;
                async {
                    let [mut f0, mut f1] = task.par()?;
                    async {
                        f0.send::<Specimen<PURITY>>(purity).await?;
                    };
                    async {
                        f1.send::<Specimen<TOXICITY>>(toxicity).await?;
                    };
                    Ok(())
                }
                .await?;
                purity = task.recv::<Specimen<PURITY>>().await?;
                toxicity = task.recv::<Specimen<TOXICITY>>().await?;
            }
            3 => {
                async {
                    let [mut bio, mut tox, mut hplc, mut cell] = study.par()?;
                    async {
                        bio.send::<Plan<3, BIOLOGIST>>(
                            &imp::ReanalysisPlan::<3, BIOLOGIST>::from_bytes(&bytes)?,
                        )
                        .await?;
                    };
                    async {
                        tox.send::<Plan<3, TOXICOLOGIST>>(
                            &imp::ReanalysisPlan::<3, TOXICOLOGIST>::from_bytes(&bytes)?,
                        )
                        .await?;
                    };
                    async {
                        hplc.send::<Plan<3, HPLC_SERVICE>>(
                            &imp::ReanalysisPlan::<3, HPLC_SERVICE>::from_bytes(&bytes)?,
                        )
                        .await?;
                    };
                    async {
                        cell.send::<Plan<3, CELL_SERVICE>>(
                            &imp::ReanalysisPlan::<3, CELL_SERVICE>::from_bytes(&bytes)?,
                        )
                        .await?;
                    };
                    Ok(())
                }
                .await?;
                async {
                    let [mut f0, mut f1, mut f2, mut f3] = task.par()?;
                    async {
                        f0.send::<Specimen<BIOASSAY>>(bioassay).await?;
                    };
                    async {
                        f1.send::<Specimen<STABILITY>>(stability).await?;
                    };
                    async {
                        f2.send::<Specimen<PURITY>>(purity).await?;
                    };
                    async {
                        f3.send::<Specimen<TOXICITY>>(toxicity).await?;
                    };
                    Ok(())
                }
                .await?;
                (bioassay, purity) = async {
                    let [mut a, mut b] = task.par()?;
                    async { Ok(a.recv::<Specimen<BIOASSAY>>().await?) };
                    async { Ok(b.recv::<Specimen<PURITY>>().await?) };
                }
                .await?;
                (stability, toxicity) = async {
                    let [mut a, mut b] = task.par()?;
                    async { Ok(a.recv::<Specimen<STABILITY>>().await?) };
                    async { Ok(b.recv::<Specimen<TOXICITY>>().await?) };
                }
                .await?;
            }
            _ => return Err(ShellError::message("invalid actual reanalysis selection")),
        }
        (bio_report, tox_report) = async {
            let [mut bio, mut tox] = task.par()?;
            async { imp::decode(bio.recv::<BioReport>().await?) };
            async { imp::decode(tox.recv::<ToxReport>().await?) };
        }
        .await?;
        imp::same(&bio_report["compound"], &tox_report["compound"])?;
    }
    drop((bioassay, stability, purity, toxicity));
    drop(task);
    loop {
        match guest
            .next_file::<ClosedFile, LEAD>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Exit(exit) => {
                let code = exit.status() as i32;
                private.send::<IoExit>(&code).await?;
                return Ok(code);
            }
            FileBoundary::Closed { pending, .. } => {
                let result =
                    json!({"status":"out_of_order","error":"the scientific task has completed"});
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
                    protocol::FdClosed::new_with_errno(SERVICE_FD, 8),
                )))?;
            }
        }
    }
}
