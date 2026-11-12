//! A wash request uses both actual resource Endpoints and keeps its real feedback.
use super::super::{global::*, imp};
use crate::shell::{
    ShellError, ShellResult,
    global::Exit as IoExit,
    imp::file_wire::{ClosedFile, ResultChunk, ResultChunkMsg, SERVICE_FD},
};
use hibana::{Endpoint, g::Message};
use hibana_wasip1_runtime::{FileBoundary, HibanaWasiGuest, WasiImportCompletion, protocol};
use serde_json::{Value, json};

#[hibana_runtime::scope]
pub async fn run<const ROLE: u8>(
    mut guest: &mut HibanaWasiGuest,
    endpoint: &mut Endpoint<ROLE>,
) -> ShellResult<i32> {
    let index = match ROLE {
        A => 0,
        B => 1,
        C => 2,
        _ => return Err(ShellError::message("unknown cascading builder")),
    };
    let [mut task, mut private] = endpoint.par()?;
    loop {
        let (arguments, pending) = loop {
            match guest
                .next_file::<ClosedFile, ROLE>(&mut private, SERVICE_FD)
                .await?
            {
                FileBoundary::Exit(_) => {
                    return Err(ShellError::message(
                        "builder exited before the linked result",
                    ));
                }
                FileBoundary::Closed { pending, file } => {
                    let requested = file
                        .arguments(b"service/compile_module", "module_name")
                        .and_then(|value| imp::arguments(value, index));
                    match requested {
                        Ok(arguments) => break (arguments, pending),
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
                                protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                                    SERVICE_FD, 28,
                                )),
                            ))?;
                        }
                    }
                }
            }
        };
        let mut updates = Vec::new();
        let mut result = loop {
            let branch = task.offer().await?;
            match branch.label() {
                LoanCoreA::LOGICAL_LABEL | LoanConfigA::LOGICAL_LABEL if ROLE == A => {
                    drop(branch);
                    let (mut core, mut config) = async {
                        let [mut library, mut configuration] = task.par()?;
                        async { Ok::<_, ShellError>(library.recv::<LoanCoreA>().await?) };
                        async { Ok::<_, ShellError>(configuration.recv::<LoanConfigA>().await?) };
                    }
                    .await?;
                    let bytes = imp::encode(&arguments)?;
                    task.send::<CompileA>(&bytes.as_slice()).await?;
                    let (first, second) = async {
                        async {
                            core.send::<CompileCore>(&bytes.as_slice()).await?;
                            imp::decode(core.recv::<CoreBuilt>().await?)
                        };
                        async {
                            config.send::<CompileConfig>(&bytes.as_slice()).await?;
                            imp::decode(config.recv::<ConfigBuilt>().await?)
                        };
                    }
                    .await?;
                    if first != second {
                        return Err(ShellError::message(
                            "compile resources returned different results",
                        ));
                    }
                    imp::object(&first, index)?;
                    async {
                        let [mut library, mut configuration] = task.par()?;
                        async {
                            library.send::<ReturnCore>(core).await?;
                        };
                        async {
                            configuration.send::<ReturnConfig>(config).await?;
                        };
                        Ok::<(), ShellError>(())
                    }
                    .await?;
                    break first;
                }
                LoanCoreB::LOGICAL_LABEL | LoanModelsB::LOGICAL_LABEL if ROLE == B => {
                    drop(branch);
                    let (mut core, mut models) = async {
                        let [mut library, mut definitions] = task.par()?;
                        async { Ok::<_, ShellError>(library.recv::<LoanCoreB>().await?) };
                        async { Ok::<_, ShellError>(definitions.recv::<LoanModelsB>().await?) };
                    }
                    .await?;
                    let bytes = imp::encode(&arguments)?;
                    task.send::<CompileB>(&bytes.as_slice()).await?;
                    let (first, second) = async {
                        async {
                            core.send::<CompileCore>(&bytes.as_slice()).await?;
                            imp::decode(core.recv::<CoreBuilt>().await?)
                        };
                        async {
                            models.send::<CompileModels>(&bytes.as_slice()).await?;
                            imp::decode(models.recv::<ModelsBuilt>().await?)
                        };
                    }
                    .await?;
                    if first != second {
                        return Err(ShellError::message(
                            "compile resources returned different results",
                        ));
                    }
                    imp::object(&first, index)?;
                    async {
                        let [mut library, mut definitions] = task.par()?;
                        async {
                            library.send::<ReturnCore>(core).await?;
                        };
                        async {
                            definitions.send::<ReturnModels>(models).await?;
                        };
                        Ok::<(), ShellError>(())
                    }
                    .await?;
                    break first;
                }
                LoanConfigC::LOGICAL_LABEL | LoanModelsC::LOGICAL_LABEL if ROLE == C => {
                    drop(branch);
                    let (mut config, mut models) = async {
                        let [mut configuration, mut definitions] = task.par()?;
                        async { Ok::<_, ShellError>(configuration.recv::<LoanConfigC>().await?) };
                        async { Ok::<_, ShellError>(definitions.recv::<LoanModelsC>().await?) };
                    }
                    .await?;
                    let bytes = imp::encode(&arguments)?;
                    task.send::<CompileC>(&bytes.as_slice()).await?;
                    let (first, second) = async {
                        async {
                            config.send::<CompileConfig>(&bytes.as_slice()).await?;
                            imp::decode(config.recv::<ConfigBuilt>().await?)
                        };
                        async {
                            models.send::<CompileModels>(&bytes.as_slice()).await?;
                            imp::decode(models.recv::<ModelsBuilt>().await?)
                        };
                    }
                    .await?;
                    if first != second {
                        return Err(ShellError::message(
                            "compile resources returned different results",
                        ));
                    }
                    imp::object(&first, index)?;
                    async {
                        let [mut configuration, mut definitions] = task.par()?;
                        async {
                            configuration.send::<ReturnConfig>(config).await?;
                        };
                        async {
                            definitions.send::<ReturnModels>(models).await?;
                        };
                        Ok::<(), ShellError>(())
                    }
                    .await?;
                    break first;
                }
                UpdateA::LOGICAL_LABEL => {
                    updates.push(imp::decode(branch.recv::<UpdateA>().await?)?)
                }
                UpdateB::LOGICAL_LABEL => {
                    updates.push(imp::decode(branch.recv::<UpdateB>().await?)?)
                }
                UpdateC::LOGICAL_LABEL => {
                    updates.push(imp::decode(branch.recv::<UpdateC>().await?)?)
                }
                _ => {
                    return Err(ShellError::message(
                        "unexpected build input before its actual compile",
                    ));
                }
            }
        };
        let mut feedback = Value::Null;
        let continuation = loop {
            let branch = task.offer().await?;
            match branch.label() {
                Feedback::LOGICAL_LABEL => {
                    feedback = imp::decode(branch.recv::<Feedback>().await?)?;
                    imp::test_result(&feedback, index)?;
                    if feedback["data"]["input_object"] != result["data"]["object"] {
                        return Err(ShellError::message(
                            "feedback tested a different actual object",
                        ));
                    }
                }
                UpdateA::LOGICAL_LABEL => {
                    updates.push(imp::decode(branch.recv::<UpdateA>().await?)?)
                }
                UpdateB::LOGICAL_LABEL => {
                    updates.push(imp::decode(branch.recv::<UpdateB>().await?)?)
                }
                UpdateC::LOGICAL_LABEL => {
                    updates.push(imp::decode(branch.recv::<UpdateC>().await?)?)
                }
                RetryAToB::LOGICAL_LABEL => {
                    updates.push(imp::decode(branch.recv::<RetryAToB>().await?)?)
                }
                RetryAToC::LOGICAL_LABEL => {
                    updates.push(imp::decode(branch.recv::<RetryAToC>().await?)?)
                }
                RetryBToA::LOGICAL_LABEL => {
                    updates.push(imp::decode(branch.recv::<RetryBToA>().await?)?)
                }
                RetryBToC::LOGICAL_LABEL => {
                    updates.push(imp::decode(branch.recv::<RetryBToC>().await?)?)
                }
                RetryCToA::LOGICAL_LABEL => {
                    updates.push(imp::decode(branch.recv::<RetryCToA>().await?)?)
                }
                RetryCToB::LOGICAL_LABEL => {
                    updates.push(imp::decode(branch.recv::<RetryCToB>().await?)?)
                }
                RetryA::LOGICAL_LABEL if ROLE == A => {
                    break imp::decode(branch.recv::<RetryA>().await?)?;
                }
                RetryB::LOGICAL_LABEL if ROLE == B => {
                    break imp::decode(branch.recv::<RetryB>().await?)?;
                }
                RetryC::LOGICAL_LABEL if ROLE == C => {
                    break imp::decode(branch.recv::<RetryC>().await?)?;
                }
                FinishedA::LOGICAL_LABEL if ROLE == A => {
                    break imp::decode(branch.recv::<FinishedA>().await?)?;
                }
                FinishedB::LOGICAL_LABEL if ROLE == B => {
                    break imp::decode(branch.recv::<FinishedB>().await?)?;
                }
                FinishedC::LOGICAL_LABEL if ROLE == C => {
                    break imp::decode(branch.recv::<FinishedC>().await?)?;
                }
                _ => {
                    return Err(ShellError::message(
                        "unexpected builder feedback continuation",
                    ));
                }
            }
        };
        imp::test_result(&feedback, index)?;
        result["coordination"] =
            json!({"test":feedback,"shared_updates":updates,"continuation":continuation});
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
        if continuation["tool_name"] == "link_modules" {
            if continuation["success"] != true || continuation["data"]["status"] != "linked" {
                return Err(ShellError::message(
                    "builder received no successful linked result",
                ));
            }
            break;
        }
        if continuation["module"] != imp::MODULES[index]
            || continuation["object"] != result["data"]["object"]
        {
            return Err(ShellError::message(
                "repair does not identify this actual build",
            ));
        }
    }
    drop(task);
    loop {
        match guest
            .next_file::<ClosedFile, ROLE>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Exit(exit) => {
                let status = exit.status() as i32;
                private.send::<IoExit>(&status).await?;
                return Ok(status);
            }
            FileBoundary::Closed { pending, .. } => {
                let bytes = imp::encode(
                    &json!({"status":"out_of_order","error":"the actual linked result has closed editing"}),
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
