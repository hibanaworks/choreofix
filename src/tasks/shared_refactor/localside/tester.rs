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
    for _ in 0..2 {
        let (candidate, mut call) = async {
            async { imp::decode(task.recv::<TestPlan>().await?) };
            async {
                match guest
                    .next_file::<ClosedFile, TESTER>(&mut private, SERVICE_FD)
                    .await?
                {
                    FileBoundary::Exit(_) => Err(ShellError::message(
                        "tester exited before a dependent test round",
                    )),
                    call => Ok::<_, ShellError>(call),
                }
            };
        }
        .await?;
        let (mut repo, mut environment) = async {
            let [mut r, mut e] = task.par()?;
            async { Ok::<_, ShellError>(r.recv::<RepoLoan>().await?) };
            async { Ok::<_, ShellError>(e.recv::<TestLoan>().await?) };
        }
        .await?;
        loop {
            match call {
                FileBoundary::Exit(_) => {
                    return Err(ShellError::message(
                        "tester exited before both dependent suites",
                    ));
                }
                FileBoundary::Closed { pending, file } => {
                    let requested = file.arguments(b"service/run_tests", "suite").and_then(|v| {
                        if matches!(
                            v["suite"].as_str(),
                            Some("module_a_tests" | "module_b_tests")
                        ) {
                            Ok(v)
                        } else {
                            Err(ShellError::message("unknown dependent suite"))
                        }
                    });
                    let (reply, errno) = match &requested {
                        Err(error) => (
                            json!({"status":"invalid_arguments","error":error.to_string()}),
                            28,
                        ),
                        Ok(arguments) => {
                            let bytes = imp::encode(arguments)?;
                            if arguments["suite"] == "module_a_tests" {
                                task.send::<TestA>(&bytes.as_slice()).await?;
                            } else {
                                task.send::<TestB>(&bytes.as_slice()).await?;
                            }
                            let bytes =
                                imp::encode(&json!({"candidate":candidate,"arguments":arguments}))?;
                            let (read, tested) = async {
                                async {
                                    repo.send::<ReadSnapshot>(&bytes.as_slice()).await?;
                                    imp::decode(repo.recv::<RepositoryTested>().await?)
                                };
                                async {
                                    environment.send::<RunTests>(&bytes.as_slice()).await?;
                                    imp::decode(environment.recv::<EnvironmentTested>().await?)
                                };
                            }
                            .await?;
                            imp::same(&read, &tested)?;
                            imp::same(&read["candidate"], &candidate)?;
                            (read["result"].clone(), 0)
                        }
                    };
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
                    guest =
                        pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
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

        let mut call = guest
            .next_file::<ClosedFile, TESTER>(&mut private, SERVICE_FD)
            .await?;
        loop {
            match call {
                FileBoundary::Exit(_) => {
                    return Err(ShellError::message(
                        "tester exited before both dependent suites",
                    ));
                }
                FileBoundary::Closed { pending, file } => {
                    let requested = file.arguments(b"service/run_tests", "suite").and_then(|v| {
                        if matches!(
                            v["suite"].as_str(),
                            Some("module_a_tests" | "module_b_tests")
                        ) {
                            Ok(v)
                        } else {
                            Err(ShellError::message("unknown dependent suite"))
                        }
                    });
                    let (reply, errno) = match &requested {
                        Err(error) => (
                            json!({"status":"invalid_arguments","error":error.to_string()}),
                            28,
                        ),
                        Ok(arguments) => {
                            let bytes = imp::encode(arguments)?;
                            if arguments["suite"] == "module_a_tests" {
                                task.send::<TestA>(&bytes.as_slice()).await?;
                            } else {
                                task.send::<TestB>(&bytes.as_slice()).await?;
                            }
                            let bytes =
                                imp::encode(&json!({"candidate":candidate,"arguments":arguments}))?;
                            let (read, tested) = async {
                                async {
                                    repo.send::<ReadSnapshot>(&bytes.as_slice()).await?;
                                    imp::decode(repo.recv::<RepositoryTested>().await?)
                                };
                                async {
                                    environment.send::<RunTests>(&bytes.as_slice()).await?;
                                    imp::decode(environment.recv::<EnvironmentTested>().await?)
                                };
                            }
                            .await?;
                            imp::same(&read, &tested)?;
                            imp::same(&read["candidate"], &candidate)?;
                            (read["result"].clone(), 0)
                        }
                    };
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
                    guest =
                        pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
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
    }
    drop(task);
    loop {
        match guest
            .next_file::<ClosedFile, TESTER>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Exit(code) => {
                let code = code.status() as i32;
                private.send::<IoExit>(&code).await?;
                return Ok(code);
            }
            FileBoundary::Closed { pending, .. } => {
                let reply =
                    json!({"status":"out_of_order","error":"the task Endpoint has completed"});
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
