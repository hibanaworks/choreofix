//! Endpoint operations are the participant's scientific procedure.
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
pub async fn run<'r>(
    mut guest: &mut HibanaWasiGuest,
    (endpoint, mut culture_specimen, mut bioassay_specimen, mut stability_specimen): (
        &mut Endpoint<'r, BIOLOGIST>,
        Endpoint<'r, CULTURE>,
        Endpoint<'r, BIOASSAY>,
        Endpoint<'r, STABILITY>,
    ),
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    let preparation = {
        loop {
            match guest
                .next_file::<ClosedFile, BIOLOGIST>(&mut private, SERVICE_FD)
                .await?
            {
                FileBoundary::Exit(_) => {
                    return Err(ShellError::message(
                        "scientist exited before completing an analysis",
                    ));
                }
                FileBoundary::Closed { pending, file } => match imp::analysis_file::<1>(&file) {
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
                    Ok(arguments) => {
                        let bytes = imp::encode(&arguments)?;
                        let input = imp::AnalysisArguments::<1>::from_bytes(&bytes)?;
                        culture_specimen.send::<Intent<1>>(&input).await?;
                        let mut instrument = task.recv::<Instrument<CELL>>().await?;
                        instrument.send::<Request<1>>(&input).await?;
                        task.send::<Instrument<CELL>>(instrument).await?;
                        task.send::<Specimen<CULTURE>>(culture_specimen).await?;

                        let branch = task.offer().await?;
                        match branch.label() {
                            label if label == Passed::<1>::LOGICAL_LABEL => {
                                let result = imp::decode(branch.recv::<Passed<1>>().await?)?;
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
                            label if label == Failed::<1>::LOGICAL_LABEL => {
                                let result = imp::decode(branch.recv::<Failed<1>>().await?)?;
                                culture_specimen = task.recv::<Specimen<CULTURE>>().await?;
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
                            }
                            _ => {
                                return Err(ShellError::message(
                                    "unexpected actual analysis outcome",
                                ));
                            }
                        }
                    }
                },
            }
        }
    };
    let compound = imp::decode(task.recv::<CompoundBio>().await?)?;
    let mut assay = {
        loop {
            match guest
                .next_file::<ClosedFile, BIOLOGIST>(&mut private, SERVICE_FD)
                .await?
            {
                FileBoundary::Exit(_) => {
                    return Err(ShellError::message(
                        "scientist exited before completing an analysis",
                    ));
                }
                FileBoundary::Closed { pending, file } => match imp::analysis_file::<4>(&file) {
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
                    Ok(arguments) => {
                        let bytes = imp::encode(&arguments)?;
                        let input = imp::AnalysisArguments::<4>::from_bytes(&bytes)?;
                        bioassay_specimen.send::<Intent<4>>(&input).await?;
                        let mut instrument = task.recv::<Instrument<CELL>>().await?;
                        instrument.send::<Request<4>>(&input).await?;
                        task.send::<Instrument<CELL>>(instrument).await?;
                        task.send::<Specimen<BIOASSAY>>(bioassay_specimen).await?;

                        let branch = task.offer().await?;
                        match branch.label() {
                            label if label == Passed::<4>::LOGICAL_LABEL => {
                                let result = imp::decode(branch.recv::<Passed<4>>().await?)?;
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
                            label if label == Failed::<4>::LOGICAL_LABEL => {
                                let result = imp::decode(branch.recv::<Failed<4>>().await?)?;
                                bioassay_specimen = task.recv::<Specimen<BIOASSAY>>().await?;
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
                            }
                            _ => {
                                return Err(ShellError::message(
                                    "unexpected actual analysis outcome",
                                ));
                            }
                        }
                    }
                },
            }
        }
    };
    let mut stability = {
        loop {
            match guest
                .next_file::<ClosedFile, BIOLOGIST>(&mut private, SERVICE_FD)
                .await?
            {
                FileBoundary::Exit(_) => {
                    return Err(ShellError::message(
                        "scientist exited before completing an analysis",
                    ));
                }
                FileBoundary::Closed { pending, file } => match imp::analysis_file::<5>(&file) {
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
                    Ok(arguments) => {
                        let bytes = imp::encode(&arguments)?;
                        let input = imp::AnalysisArguments::<5>::from_bytes(&bytes)?;
                        stability_specimen.send::<Intent<5>>(&input).await?;
                        let mut instrument = task.recv::<Instrument<HPLC>>().await?;
                        instrument.send::<Request<5>>(&input).await?;
                        task.send::<Instrument<HPLC>>(instrument).await?;
                        task.send::<Specimen<STABILITY>>(stability_specimen).await?;

                        let branch = task.offer().await?;
                        match branch.label() {
                            label if label == Passed::<5>::LOGICAL_LABEL => {
                                let result = imp::decode(branch.recv::<Passed<5>>().await?)?;
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
                            label if label == Failed::<5>::LOGICAL_LABEL => {
                                let result = imp::decode(branch.recv::<Failed<5>>().await?)?;
                                stability_specimen = task.recv::<Specimen<STABILITY>>().await?;
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
                            }
                            _ => {
                                return Err(ShellError::message(
                                    "unexpected actual analysis outcome",
                                ));
                            }
                        }
                    }
                },
            }
        }
    };
    let bytes = imp::encode(
        &json!({"compound":compound,"preparation":preparation,"assay":assay,"stability":stability}),
    )?;
    task.send::<BioReport>(&bytes.as_slice()).await?;
    loop {
        let branch = task.offer().await?;
        let plan = match branch.label() {
            label if label == ApprovedForBio::LOGICAL_LABEL => {
                let approved = imp::decode(branch.recv::<ApprovedForBio>().await?)?;
                // The pending query returns the actual final review, then the guest may exit.
                loop {
                    match guest
                        .next_file::<ClosedFile, BIOLOGIST>(&mut private, SERVICE_FD)
                        .await?
                    {
                        FileBoundary::Exit(exit) => {
                            let code = exit.status() as i32;
                            private.send::<IoExit>(&code).await?;
                            return Ok(code);
                        }
                        FileBoundary::Closed { pending, file } => {
                            match imp::input_file(&file, b"service/await_review").and_then(|v| {
                                if v == json!({}) {
                                    Ok(v)
                                } else {
                                    Err(ShellError::message("review query takes no arguments"))
                                }
                            }) {
                                Ok(_) => {
                                    let bytes = imp::encode(&approved)?;
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
                                    break;
                                }
                                Err(error) => {
                                    let result = json!({"status":"invalid_arguments","error":error.to_string()});
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
                }
                break;
            }
            0 => imp::decode(branch.recv::<Plan<0, BIOLOGIST>>().await?)?,
            1 => imp::decode(branch.recv::<Plan<1, BIOLOGIST>>().await?)?,
            2 => imp::decode(branch.recv::<Plan<2, BIOLOGIST>>().await?)?,
            3 => imp::decode(branch.recv::<Plan<3, BIOLOGIST>>().await?)?,
            _ => {
                return Err(ShellError::message(
                    "unexpected scientific review continuation",
                ));
            }
        };
        loop {
            match guest
                .next_file::<ClosedFile, BIOLOGIST>(&mut private, SERVICE_FD)
                .await?
            {
                FileBoundary::Exit(_) => {
                    return Err(ShellError::message(
                        "scientist exited before regulatory approval",
                    ));
                }
                FileBoundary::Closed { pending, file } => {
                    match imp::input_file(&file, b"service/await_review").and_then(|v| {
                        if v == json!({}) {
                            Ok(v)
                        } else {
                            Err(ShellError::message("review query takes no arguments"))
                        }
                    }) {
                        Ok(_) => {
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
                            break;
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
                    }
                }
            }
        }
        if plan["biology"] == true {
            let mut bioassay_specimen = task.recv::<Specimen<BIOASSAY>>().await?;
            let mut stability_specimen = task.recv::<Specimen<STABILITY>>().await?;
            let replacement_assay = {
                loop {
                    match guest
                        .next_file::<ClosedFile, BIOLOGIST>(&mut private, SERVICE_FD)
                        .await?
                    {
                        FileBoundary::Exit(_) => {
                            return Err(ShellError::message(
                                "scientist exited before completing an analysis",
                            ));
                        }
                        FileBoundary::Closed { pending, file } => {
                            match imp::analysis_file::<11>(&file) {
                                Err(error) => {
                                    let result = json!({"status":"invalid_arguments","error":error.to_string()});
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
                                Ok(arguments) => {
                                    let bytes = imp::encode(&arguments)?;
                                    let input = imp::AnalysisArguments::<11>::from_bytes(&bytes)?;
                                    bioassay_specimen.send::<Intent<11>>(&input).await?;
                                    let mut instrument = task.recv::<Instrument<CELL>>().await?;
                                    instrument.send::<Request<11>>(&input).await?;
                                    task.send::<Instrument<CELL>>(instrument).await?;
                                    task.send::<Specimen<BIOASSAY>>(bioassay_specimen).await?;

                                    let branch = task.offer().await?;
                                    match branch.label() {
                                        label if label == Passed::<11>::LOGICAL_LABEL => {
                                            let result =
                                                imp::decode(branch.recv::<Passed<11>>().await?)?;
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
                                            guest =
                                                pending.complete(WasiImportCompletion::FdClose(
                                                    protocol::FdClosedRet(
                                                        protocol::FdClosed::new_with_errno(
                                                            SERVICE_FD, 0,
                                                        ),
                                                    ),
                                                ))?;
                                            break result;
                                        }
                                        label if label == Failed::<11>::LOGICAL_LABEL => {
                                            let result =
                                                imp::decode(branch.recv::<Failed<11>>().await?)?;
                                            bioassay_specimen =
                                                task.recv::<Specimen<BIOASSAY>>().await?;
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
                                            guest =
                                                pending.complete(WasiImportCompletion::FdClose(
                                                    protocol::FdClosedRet(
                                                        protocol::FdClosed::new_with_errno(
                                                            SERVICE_FD, 0,
                                                        ),
                                                    ),
                                                ))?;
                                        }
                                        _ => {
                                            return Err(ShellError::message(
                                                "unexpected actual analysis outcome",
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            };
            assay = replacement_assay;
            let replacement_stability = {
                loop {
                    match guest
                        .next_file::<ClosedFile, BIOLOGIST>(&mut private, SERVICE_FD)
                        .await?
                    {
                        FileBoundary::Exit(_) => {
                            return Err(ShellError::message(
                                "scientist exited before completing an analysis",
                            ));
                        }
                        FileBoundary::Closed { pending, file } => {
                            match imp::analysis_file::<12>(&file) {
                                Err(error) => {
                                    let result = json!({"status":"invalid_arguments","error":error.to_string()});
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
                                Ok(arguments) => {
                                    let bytes = imp::encode(&arguments)?;
                                    let input = imp::AnalysisArguments::<12>::from_bytes(&bytes)?;
                                    stability_specimen.send::<Intent<12>>(&input).await?;
                                    let mut instrument = task.recv::<Instrument<HPLC>>().await?;
                                    instrument.send::<Request<12>>(&input).await?;
                                    task.send::<Instrument<HPLC>>(instrument).await?;
                                    task.send::<Specimen<STABILITY>>(stability_specimen).await?;

                                    let branch = task.offer().await?;
                                    match branch.label() {
                                        label if label == Passed::<12>::LOGICAL_LABEL => {
                                            let result =
                                                imp::decode(branch.recv::<Passed<12>>().await?)?;
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
                                            guest =
                                                pending.complete(WasiImportCompletion::FdClose(
                                                    protocol::FdClosedRet(
                                                        protocol::FdClosed::new_with_errno(
                                                            SERVICE_FD, 0,
                                                        ),
                                                    ),
                                                ))?;
                                            break result;
                                        }
                                        label if label == Failed::<12>::LOGICAL_LABEL => {
                                            let result =
                                                imp::decode(branch.recv::<Failed<12>>().await?)?;
                                            stability_specimen =
                                                task.recv::<Specimen<STABILITY>>().await?;
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
                                            guest =
                                                pending.complete(WasiImportCompletion::FdClose(
                                                    protocol::FdClosedRet(
                                                        protocol::FdClosed::new_with_errno(
                                                            SERVICE_FD, 0,
                                                        ),
                                                    ),
                                                ))?;
                                        }
                                        _ => {
                                            return Err(ShellError::message(
                                                "unexpected actual analysis outcome",
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            };
            stability = replacement_stability;
        }
        let bytes = imp::encode(
            &json!({"compound":compound,"preparation":preparation,"assay":assay,"stability":stability}),
        )?;
        task.send::<BioReport>(&bytes.as_slice()).await?;
    }
    drop(task);
    loop {
        match guest
            .next_file::<ClosedFile, BIOLOGIST>(&mut private, SERVICE_FD)
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
