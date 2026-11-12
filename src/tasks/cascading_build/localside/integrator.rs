//! Reconsideration follows received real objects/tests; linking uses the model's own arguments.
use super::super::{global::*, imp};
use crate::shell::{
    ShellError, ShellResult,
    global::Exit as IoExit,
    imp::{
        file_wire::{ClosedFile, ResultChunk, ResultChunkMsg, SERVICE_FD},
        review_reply::ReviewReply,
    },
};
use core::cell::Cell;
use hibana::{Endpoint, g::Message};
use hibana_wasip1_runtime::{FileBoundary, HibanaWasiGuest, WasiImportCompletion, protocol};
use serde_json::{Value, json};

#[hibana_runtime::scope]
pub async fn run(
    guest: &mut HibanaWasiGuest,
    endpoint: &mut Endpoint<INTEGRATOR>,
    input: &Cell<Value>,
) -> ShellResult<i32> {
    let [mut task, private] = endpoint.par()?;
    let ((objects, tests), (arguments, pending, mut private)) = async {
        async {
            loop {
                let branch = task.offer().await?;
                match branch.label() {
                    RepairCandidate::LOGICAL_LABEL => {
                        let actual = imp::decode(branch.recv::<RepairCandidate>().await?)?;
                        let (objects, tests) = imp::candidate_parts(&actual)?;
                        let index = imp::needed(&objects, &tests)?.ok_or_else(||
                            ShellError::message("repair candidate is already current"))?;
                        let bytes = imp::encode(&imp::reconsider(&objects, &tests, index)?)?;
                        input.set(actual);
                        drop(task.offer().await?);
                        async {
                            let [mut selected, mut compiler, mut first_peer, mut second_peer] = task.par()?;
                            match index {
                                0 => {
                                    async {
                                        async { selected.send::<RetryA>(&bytes.as_slice()).await?; };
                                        async { compiler.send::<RetryAForCompiler>(&bytes.as_slice()).await?; };
                                        async { first_peer.send::<RetryAToB>(&bytes.as_slice()).await?; };
                                        async { second_peer.send::<RetryAToC>(&bytes.as_slice()).await?; };
                                        Ok::<(), ShellError>(())
                                    }.await?
                                }
                                1 => {
                                    async {
                                        async { selected.send::<RetryB>(&bytes.as_slice()).await?; };
                                        async { compiler.send::<RetryBForCompiler>(&bytes.as_slice()).await?; };
                                        async { first_peer.send::<RetryBToA>(&bytes.as_slice()).await?; };
                                        async { second_peer.send::<RetryBToC>(&bytes.as_slice()).await?; };
                                        Ok::<(), ShellError>(())
                                    }.await?
                                }
                                _ => {
                                    async {
                                        async { selected.send::<RetryC>(&bytes.as_slice()).await?; };
                                        async { compiler.send::<RetryCForCompiler>(&bytes.as_slice()).await?; };
                                        async { first_peer.send::<RetryCToA>(&bytes.as_slice()).await?; };
                                        async { second_peer.send::<RetryCToB>(&bytes.as_slice()).await?; };
                                        Ok::<(), ShellError>(())
                                    }.await?
                                }
                            }
                            Ok::<(), ShellError>(())
                        }.await?;
                    }
                    ReadyCandidate::LOGICAL_LABEL => {
                        let actual = imp::decode(branch.recv::<ReadyCandidate>().await?)?;
                        let (objects, tests) = imp::candidate_parts(&actual)?;
                        if imp::finish(&objects, &tests)? != actual {
                            return Err(ShellError::message("ready candidate differs from actual successful checks"));
                        }
                        let bytes = imp::encode(&actual)?;
                        task.send::<FinishChecks>(&ReviewReply::from_bytes(&bytes)?).await?;
                        break Ok::<_, ShellError>((objects, tests));
                    }
                    _ => return Err(ShellError::message("unexpected integrator candidate")),
                }
            }
        };
        async move {
            let mut private = private;
            let mut guest = guest;
            loop {
                match guest.next_file::<ClosedFile, INTEGRATOR>(&mut private, SERVICE_FD).await? {
                    FileBoundary::Exit(_) => return Err(ShellError::message("integrator exited before linking")),
                    FileBoundary::Closed { pending, file } => {
                        let requested = file.arguments(b"service/link_modules", "modules").and_then(imp::link_arguments);
                        match requested {
                            Ok(arguments) => break Ok::<_, ShellError>((arguments, pending, private)),
                            Err(error) => {
                                let bytes = imp::encode(&json!({"status":"invalid_arguments","error":error.to_string()}))?;
                                for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
                                    private.send::<ResultChunkMsg>(&ResultChunk::new(index * protocol::WASIP1_IO_CHUNK_CAPACITY, chunk)?).await?;
                                }
                                guest = pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
                                    protocol::FdClosed::new_with_errno(SERVICE_FD, 28))))?;
                            }
                        }
                    }
                }
            }
        };
    }.await?;
    let (_core, _config, _models) = async {
        let [mut library, mut configuration, mut definitions] = task.par()?;
        async { Ok::<_, ShellError>(library.recv::<FrozenCore>().await?) };
        async { Ok::<_, ShellError>(configuration.recv::<FrozenConfig>().await?) };
        async { Ok::<_, ShellError>(definitions.recv::<FrozenModels>().await?) };
    }
    .await?;
    let bytes = imp::encode(&arguments)?;
    task.send::<Link>(&bytes.as_slice()).await?;
    let result = imp::decode(task.recv::<Linked>().await?)?;
    imp::linked(&result, &objects, &tests)?;
    let bytes = imp::encode(&result)?;
    async {
        let [mut a, mut b, mut c, mut tester, mut service] = task.par()?;
        async {
            a.send::<FinishedA>(&bytes.as_slice()).await?;
        };
        async {
            b.send::<FinishedB>(&bytes.as_slice()).await?;
        };
        async {
            c.send::<FinishedC>(&bytes.as_slice()).await?;
        };
        async {
            tester.send::<FinishedTester>(&bytes.as_slice()).await?;
        };
        async {
            service.send::<FinishedService>(&bytes.as_slice()).await?;
        };
        Ok::<(), ShellError>(())
    }
    .await?;
    for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
        private
            .send::<ResultChunkMsg>(&ResultChunk::new(
                index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                chunk,
            )?)
            .await?;
    }
    let mut guest = pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
        protocol::FdClosed::new_with_errno(SERVICE_FD, 0),
    )))?;
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
