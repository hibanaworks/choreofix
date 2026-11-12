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
    (endpoint, mut purity_specimen, mut toxicity_specimen): (
        &mut Endpoint<'r, TOXICOLOGIST>,
        Endpoint<'r, PURITY>,
        Endpoint<'r, TOXICITY>,
    ),
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    let baseline = {
        loop {
            match guest
                .next_file::<ClosedFile, TOXICOLOGIST>(&mut private, SERVICE_FD)
                .await?
            {
                FileBoundary::Exit(_) => {
                    return Err(ShellError::message(
                        "scientist exited before completing mass spectrometry",
                    ));
                }
                FileBoundary::Closed { pending, file } => match imp::analysis_file::<2>(&file) {
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
                        let input = imp::AnalysisArguments::<2>::from_bytes(&bytes)?;
                        task.send::<MassIntent<2>>(&input).await?;
                        let mut instrument = task.recv::<Instrument<MASS>>().await?;
                        instrument.send::<MassRequest<2>>(&input).await?;
                        let result = imp::decode(instrument.recv::<MassResult>().await?)?;
                        imp::successful(&result)?;
                        task.send::<Instrument<MASS>>(instrument).await?;
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
                },
            }
        }
    };
    let compound = imp::decode(task.recv::<CompoundTox>().await?)?;
    let mut purity = {
        loop {
            match guest
                .next_file::<ClosedFile, TOXICOLOGIST>(&mut private, SERVICE_FD)
                .await?
            {
                FileBoundary::Exit(_) => {
                    return Err(ShellError::message(
                        "scientist exited before completing an analysis",
                    ));
                }
                FileBoundary::Closed { pending, file } => match imp::analysis_file::<6>(&file) {
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
                        let input = imp::AnalysisArguments::<6>::from_bytes(&bytes)?;
                        purity_specimen.send::<Intent<6>>(&input).await?;
                        let mut instrument = task.recv::<Instrument<HPLC>>().await?;
                        instrument.send::<Request<6>>(&input).await?;
                        task.send::<Instrument<HPLC>>(instrument).await?;
                        task.send::<Specimen<PURITY>>(purity_specimen).await?;

                        let branch = task.offer().await?;
                        match branch.label() {
                            label if label == Passed::<6>::LOGICAL_LABEL => {
                                let result = imp::decode(branch.recv::<Passed<6>>().await?)?;
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
                            label if label == Failed::<6>::LOGICAL_LABEL => {
                                let result = imp::decode(branch.recv::<Failed<6>>().await?)?;
                                purity_specimen = task.recv::<Specimen<PURITY>>().await?;
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
    let mut cytotoxicity = {
        loop {
            match guest
                .next_file::<ClosedFile, TOXICOLOGIST>(&mut private, SERVICE_FD)
                .await?
            {
                FileBoundary::Exit(_) => {
                    return Err(ShellError::message(
                        "scientist exited before completing an analysis",
                    ));
                }
                FileBoundary::Closed { pending, file } => match imp::analysis_file::<7>(&file) {
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
                        let input = imp::AnalysisArguments::<7>::from_bytes(&bytes)?;
                        toxicity_specimen.send::<Intent<7>>(&input).await?;
                        let mut instrument = task.recv::<Instrument<CELL>>().await?;
                        instrument.send::<Request<7>>(&input).await?;
                        task.send::<Instrument<CELL>>(instrument).await?;
                        task.send::<Specimen<TOXICITY>>(toxicity_specimen).await?;

                        let branch = task.offer().await?;
                        match branch.label() {
                            label if label == Passed::<7>::LOGICAL_LABEL => {
                                let result = imp::decode(branch.recv::<Passed<7>>().await?)?;
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
                            label if label == Failed::<7>::LOGICAL_LABEL => {
                                let result = imp::decode(branch.recv::<Failed<7>>().await?)?;
                                toxicity_specimen = task.recv::<Specimen<TOXICITY>>().await?;
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
        &json!({"compound":compound,"baseline":baseline,"purity":purity,"cytotoxicity":cytotoxicity}),
    )?;
    task.send::<ToxReport>(&bytes.as_slice()).await?;
    loop {
        let branch = task.offer().await?;
        let plan = match branch.label() {
            label if label == ApprovedForTox::LOGICAL_LABEL => {
                let approved = imp::decode(branch.recv::<ApprovedForTox>().await?)?;
                // The pending query returns the actual final review, then the guest may exit.
                loop {
                    match guest
                        .next_file::<ClosedFile, TOXICOLOGIST>(&mut private, SERVICE_FD)
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
            0 => imp::decode(branch.recv::<Plan<0, TOXICOLOGIST>>().await?)?,
            1 => imp::decode(branch.recv::<Plan<1, TOXICOLOGIST>>().await?)?,
            2 => imp::decode(branch.recv::<Plan<2, TOXICOLOGIST>>().await?)?,
            3 => imp::decode(branch.recv::<Plan<3, TOXICOLOGIST>>().await?)?,
            _ => {
                return Err(ShellError::message(
                    "unexpected scientific review continuation",
                ));
            }
        };
        loop {
            match guest
                .next_file::<ClosedFile, TOXICOLOGIST>(&mut private, SERVICE_FD)
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
        if plan["toxicology"] == true {
            let mut purity_specimen = task.recv::<Specimen<PURITY>>().await?;
            let mut toxicity_specimen = task.recv::<Specimen<TOXICITY>>().await?;
            let replacement_purity = {
                loop {
                    match guest
                        .next_file::<ClosedFile, TOXICOLOGIST>(&mut private, SERVICE_FD)
                        .await?
                    {
                        FileBoundary::Exit(_) => {
                            return Err(ShellError::message(
                                "scientist exited before completing an analysis",
                            ));
                        }
                        FileBoundary::Closed { pending, file } => {
                            match imp::analysis_file::<13>(&file) {
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
                                    let input = imp::AnalysisArguments::<13>::from_bytes(&bytes)?;
                                    purity_specimen.send::<Intent<13>>(&input).await?;
                                    let mut instrument = task.recv::<Instrument<HPLC>>().await?;
                                    instrument.send::<Request<13>>(&input).await?;
                                    task.send::<Instrument<HPLC>>(instrument).await?;
                                    task.send::<Specimen<PURITY>>(purity_specimen).await?;

                                    let branch = task.offer().await?;
                                    match branch.label() {
                                        label if label == Passed::<13>::LOGICAL_LABEL => {
                                            let result =
                                                imp::decode(branch.recv::<Passed<13>>().await?)?;
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
                                        label if label == Failed::<13>::LOGICAL_LABEL => {
                                            let result =
                                                imp::decode(branch.recv::<Failed<13>>().await?)?;
                                            purity_specimen =
                                                task.recv::<Specimen<PURITY>>().await?;
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
            purity = replacement_purity;
            let replacement_cytotoxicity = {
                loop {
                    match guest
                        .next_file::<ClosedFile, TOXICOLOGIST>(&mut private, SERVICE_FD)
                        .await?
                    {
                        FileBoundary::Exit(_) => {
                            return Err(ShellError::message(
                                "scientist exited before completing an analysis",
                            ));
                        }
                        FileBoundary::Closed { pending, file } => {
                            match imp::analysis_file::<14>(&file) {
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
                                    let input = imp::AnalysisArguments::<14>::from_bytes(&bytes)?;
                                    toxicity_specimen.send::<Intent<14>>(&input).await?;
                                    let mut instrument = task.recv::<Instrument<CELL>>().await?;
                                    instrument.send::<Request<14>>(&input).await?;
                                    task.send::<Instrument<CELL>>(instrument).await?;
                                    task.send::<Specimen<TOXICITY>>(toxicity_specimen).await?;

                                    let branch = task.offer().await?;
                                    match branch.label() {
                                        label if label == Passed::<14>::LOGICAL_LABEL => {
                                            let result =
                                                imp::decode(branch.recv::<Passed<14>>().await?)?;
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
                                        label if label == Failed::<14>::LOGICAL_LABEL => {
                                            let result =
                                                imp::decode(branch.recv::<Failed<14>>().await?)?;
                                            toxicity_specimen =
                                                task.recv::<Specimen<TOXICITY>>().await?;
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
            cytotoxicity = replacement_cytotoxicity;
        }
        let bytes = imp::encode(
            &json!({"compound":compound,"baseline":baseline,"purity":purity,"cytotoxicity":cytotoxicity}),
        )?;
        task.send::<ToxReport>(&bytes.as_slice()).await?;
    }
    drop(task);
    loop {
        match guest
            .next_file::<ClosedFile, TOXICOLOGIST>(&mut private, SERVICE_FD)
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
