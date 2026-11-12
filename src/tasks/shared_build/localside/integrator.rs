//! The linker consumes both actual current build messages before its effect.
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
pub async fn run(
    mut guest: &mut HibanaWasiGuest,
    endpoint: &mut Endpoint<INTEGRATOR>,
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    loop {
        match guest
            .next_file::<ClosedFile, INTEGRATOR>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Exit(_) => {
                return Err(ShellError::message("integrator exited before linking"));
            }
            FileBoundary::Closed { pending, file } => {
                let arguments = file
                    .arguments(b"service/link_modules", "modules")
                    .and_then(imp::link_arguments);
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
                drop(task.offer().await?);
                let (a, b) = async {
                    let [mut a, mut b] = task.par()?;
                    async { imp::decode(a.recv::<BuiltA>().await?) };
                    async { imp::decode(b.recv::<BuiltB>().await?) };
                }
                .await?;
                let objects = [
                    imp::compiled(&a, "BUILDER_A", "module_a")?,
                    imp::compiled(&b, "BUILDER_B", "module_b")?,
                ];
                let bytes = imp::encode(&arguments)?;
                task.send::<Link>(&bytes.as_slice()).await?;
                let result = imp::decode(task.recv::<Linked>().await?)?;
                imp::linked(&result, &objects)?;
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
        }
    }
    drop(task);
    loop {
        match guest
            .next_file::<ClosedFile, INTEGRATOR>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Exit(exit) => {
                let status = exit.status() as i32;
                private.send::<IoExit>(&status).await?;
                return Ok(status);
            }
            FileBoundary::Closed { pending, .. } => {
                let bytes = imp::encode(
                    &json!({"status":"out_of_order","error":"the current binary is already linked"}),
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
