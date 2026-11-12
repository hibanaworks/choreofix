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
    endpoint: &mut hibana::Endpoint<TESTER>,
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    let (_candidate, mut call) = async {
        async { Ok::<_, ShellError>(task.recv::<Ready>().await?.to_vec()) };
        async {
            match guest
                .next_file::<ClosedFile, TESTER>(&mut private, SERVICE_FD)
                .await?
            {
                FileBoundary::Exit(_) => Err(ShellError::message("tester exited before testing")),
                call => Ok(call),
            }
        };
    }
    .await?;
    loop {
        match call {
            FileBoundary::Exit(_) => {
                return Err(ShellError::message("tester exited before testing"));
            }
            FileBoundary::Closed { pending, file } => {
                let requested = file.arguments(b"service/run_tests", "suite").and_then(|v| {
                    imp::arguments(
                        &serde_json::to_vec(&v).map_err(|e| ShellError::message(e.to_string()))?,
                        "suite",
                        "integration_tests",
                    )
                });
                let (reply, errno) = match &requested {
                    Err(error) => (
                        json!({"status":"invalid_arguments","error":error.to_string()}),
                        28,
                    ),
                    Ok(arguments) => {
                        let (mut repo, mut environment) = async {
                            let [mut r, mut e] = task.par()?;
                            async { Ok::<_, ShellError>(r.recv::<RepoLoan>().await?) };
                            async { Ok::<_, ShellError>(e.recv::<TestLoan>().await?) };
                        }
                        .await?;
                        let bytes = serde_json::to_vec(arguments)
                            .map_err(|e| ShellError::message(e.to_string()))?;
                        let (from_repo, from_environment) = async {
                            async {
                                repo.send::<ReadForTests>(&bytes.as_slice()).await?;
                                Ok::<_, ShellError>(repo.recv::<RepositoryTested>().await?.to_vec())
                            };
                            async {
                                environment.send::<RunTests>(&bytes.as_slice()).await?;
                                Ok::<_, ShellError>(
                                    environment.recv::<EnvironmentTested>().await?.to_vec(),
                                )
                            };
                        }
                        .await?;
                        if from_repo != from_environment {
                            return Err(ShellError::message("resource test results differ"));
                        }
                        async {
                            let [mut r, mut e] = task.par()?;
                            async {
                                r.send::<RepoLoan>(repo).await?;
                            };
                            async {
                                e.send::<TestLoan>(environment).await?;
                            };
                            Ok(())
                        }
                        .await?;
                        (
                            serde_json::from_slice(&from_repo)
                                .map_err(|e| ShellError::message(e.to_string()))?,
                            0,
                        )
                    }
                };

                let bytes =
                    serde_json::to_vec(&reply).map_err(|e| ShellError::message(e.to_string()))?;
                for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
                    private
                        .send::<ResultChunkMsg>(&ResultChunk::new(
                            index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                            chunk,
                        )?)
                        .await?;
                }
                guest = pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
                    protocol::FdClosed::new_with_errno(SERVICE_FD, errno),
                )))?;

                if requested.is_ok() {
                    break;
                }
            }
        }
        call = guest
            .next_file::<ClosedFile, TESTER>(&mut private, SERVICE_FD)
            .await?;
    }
    drop(task);
    loop {
        match guest
            .next_file::<ClosedFile, TESTER>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Closed { pending, .. } => {
                let reply =
                    json!({"status":"out_of_order","error":"the test Endpoint has completed"});
                let errno = 8;

                let bytes =
                    serde_json::to_vec(&reply).map_err(|e| ShellError::message(e.to_string()))?;
                for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
                    private
                        .send::<ResultChunkMsg>(&ResultChunk::new(
                            index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                            chunk,
                        )?)
                        .await?;
                }
                guest = pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
                    protocol::FdClosed::new_with_errno(SERVICE_FD, errno),
                )))?;
            }
            FileBoundary::Exit(exit) => {
                let status = exit.status() as i32;
                private.send::<IoExit>(&status).await?;
                return Ok(status);
            }
        }
    }
}
