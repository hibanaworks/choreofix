//! Each actual wash close sends its own admitted and delegated resource requests.
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
pub async fn run<const ROLE: u8>(
    mut guest: &mut HibanaWasiGuest,
    endpoint: &mut Endpoint<ROLE>,
) -> ShellResult<i32> {
    if ROLE != BUILDER_A && ROLE != BUILDER_B {
        return Err(ShellError::message("shared build has two builder roles"));
    }
    let module = if ROLE == BUILDER_A {
        "module_a"
    } else {
        "module_b"
    };
    let agent = if ROLE == BUILDER_A {
        "BUILDER_A"
    } else {
        "BUILDER_B"
    };
    let [mut task, mut private] = endpoint.par()?;
    loop {
        match guest
            .next_file::<ClosedFile, ROLE>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Exit(_) => {
                return Err(ShellError::message(
                    "builder exited before its current build",
                ));
            }
            FileBoundary::Closed { pending, file } => {
                let arguments = file
                    .arguments(b"service/compile_module", "module_name")
                    .and_then(|value| imp::compile_arguments(value, module));
                let arguments = match arguments {
                    Ok(value) => value,
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
                        continue;
                    }
                };
                let incoming = task.offer().await?;
                if ROLE == BUILDER_B && incoming.label() == TypesUpdated::LOGICAL_LABEL {
                    let update = imp::decode(incoming.recv::<TypesUpdated>().await?)?;
                    imp::compiled(&update, "BUILDER_A", "module_a")?;
                } else {
                    drop(incoming);
                }
                let (mut core, mut types) = async {
                    let [mut library, mut definitions] = task.par()?;
                    async { Ok::<_, ShellError>(library.recv::<CoreLoan>().await?) };
                    async { Ok::<_, ShellError>(definitions.recv::<TypesLoan>().await?) };
                }
                .await?;
                let bytes = imp::encode(&arguments)?;
                if ROLE == BUILDER_A {
                    task.send::<CompileA>(&bytes.as_slice()).await?;
                } else {
                    task.send::<CompileB>(&bytes.as_slice()).await?;
                }
                let (result, other) = async {
                    async {
                        core.send::<CompileCore>(&bytes.as_slice()).await?;
                        imp::decode(core.recv::<CoreCompiled>().await?)
                    };
                    async {
                        types.send::<CompileTypes>(&bytes.as_slice()).await?;
                        imp::decode(types.recv::<TypesCompiled>().await?)
                    };
                }
                .await?;
                if result != other {
                    return Err(ShellError::message(
                        "build resources returned different actual results",
                    ));
                }
                imp::compiled(&result, agent, module)?;
                let bytes = imp::encode(&result)?;
                if ROLE == BUILDER_A {
                    async {
                        let [mut notification, mut handoff] = task.par()?;
                        async {
                            notification.send::<BuiltA>(&bytes.as_slice()).await?;
                        };
                        async {
                            handoff.send::<TypesUpdated>(&bytes.as_slice()).await?;
                            let [mut library, mut definitions] = handoff.par()?;
                            async {
                                library.send::<CoreLoan>(core).await?;
                            };
                            async {
                                definitions.send::<TypesLoan>(types).await?;
                            };
                            Ok::<(), ShellError>(())
                        };
                        Ok(())
                    }
                    .await?;
                } else {
                    let next = task.offer().await?;
                    match next.label() {
                        CoreLoan::LOGICAL_LABEL | TypesLoan::LOGICAL_LABEL => {
                            drop(next);
                            async {
                                let [mut library, mut definitions] = task.par()?;
                                async {
                                    library.send::<CoreLoan>(core).await?;
                                };
                                async {
                                    definitions.send::<TypesLoan>(types).await?;
                                };
                                Ok::<(), ShellError>(())
                            }
                            .await?;
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
                            // The global requires another actual build after A's
                            // type update. No local retry/done flag authorizes it.
                            continue;
                        }
                        BuiltB::LOGICAL_LABEL
                        | ReturnCore::LOGICAL_LABEL
                        | ReturnTypes::LOGICAL_LABEL => {
                            drop(next);
                            async {
                                let [mut notification, mut core_return, mut types_return] =
                                    task.par()?;
                                async {
                                    notification.send::<BuiltB>(&bytes.as_slice()).await?;
                                };
                                async {
                                    core_return.send::<ReturnCore>(core).await?;
                                };
                                async {
                                    types_return.send::<ReturnTypes>(types).await?;
                                };
                                Ok::<(), ShellError>(())
                            }
                            .await?;
                        }
                        _ => return Err(ShellError::message("unexpected builder continuation")),
                    }
                }
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
                    &json!({"status":"out_of_order","error":"this builder has completed its current object"}),
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
