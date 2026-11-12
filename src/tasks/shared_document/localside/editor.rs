//! The editor owns actual material and review replies, never an approval table.
use super::super::{global::*, imp};
use crate::{
    shell::imp::report_arguments::ReportArguments,
    shell::{
        ShellError, ShellResult,
        global::Exit as IoExit,
        imp::{
            file_wire::{ClosedFile, ResultChunk, ResultChunkMsg, SERVICE_FD},
            review_reply::ReviewReply,
        },
    },
};
use hibana::{Endpoint, g::Message};
use hibana_wasip1_runtime::{FileBoundary, HibanaWasiGuest, WasiImportCompletion, protocol};
use serde_json::json;

#[hibana_runtime::scope]
pub async fn run(guest: &mut HibanaWasiGuest, endpoint: &mut Endpoint<EDITOR>) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    let ((mut document, mut snapshot), mut boundary) = async {
        async {
            let ((document, a, b), references_a, references_b) = async {
                let [mut writing, mut references_a, mut references_b] = task.par()?;
                async {
                    drop(writing.offer().await?);
                    let (first, (second, document)) = async {
                        let [mut first, mut remaining] = writing.par().map_err(|error| {
                            ShellError::message(format!("editor writing fork: {error:?}"))
                        })?;
                        async {
                            let branch = first.offer().await?;
                            match branch.label() {
                                MaterialA::LOGICAL_LABEL => {
                                    imp::decode(branch.recv::<MaterialA>().await?)
                                }
                                MaterialB::LOGICAL_LABEL => {
                                    imp::decode(branch.recv::<MaterialB>().await?)
                                }
                                _ => Err(ShellError::message("unexpected first chapter material")),
                            }
                        };
                        async {
                            // The second writer's unseen prefix still separates
                            // this fork from the first material's source fork.
                            drop(remaining.offer().await?);
                            async {
                                let [mut material, mut loan] =
                                    remaining.par().map_err(|error| {
                                        ShellError::message(format!(
                                            "editor remaining inputs fork: {error:?}"
                                        ))
                                    })?;
                                async {
                                    let branch = material.offer().await?;
                                    match branch.label() {
                                        MaterialA::LOGICAL_LABEL => {
                                            imp::decode(branch.recv::<MaterialA>().await?)
                                        }
                                        MaterialB::LOGICAL_LABEL => {
                                            imp::decode(branch.recv::<MaterialB>().await?)
                                        }
                                        _ => Err(ShellError::message(
                                            "unexpected second chapter material",
                                        )),
                                    }
                                };
                                async {
                                    loan.recv::<InitialDocument>()
                                        .await
                                        .map_err(ShellError::from)
                                };
                            }
                            .await
                        };
                    }
                    .await?;
                    let [a, b] = match (
                        first["data"]["section"].as_str(),
                        second["data"]["section"].as_str(),
                    ) {
                        (Some(CHAPTER_A), Some(CHAPTER_B)) => [first, second],
                        (Some(CHAPTER_B), Some(CHAPTER_A)) => [second, first],
                        _ => {
                            return Err(ShellError::message(
                                "document preparation has different chapters",
                            ));
                        }
                    };
                    Ok((document, a, b))
                };
                async { imp::decode(references_a.recv::<ReferenceMaterialA>().await?) };
                async { imp::decode(references_b.recv::<ReferenceMaterialB>().await?) };
            }
            .await?;
            let a = imp::material(
                &a,
                &json!({"section_name":CHAPTER_A}),
                ("WRITER_A", "write_section"),
            )?;
            let b = imp::material(
                &b,
                &json!({"section_name":CHAPTER_B}),
                ("WRITER_B", "write_section"),
            )?;
            let references_a = imp::material(
                &references_a,
                &json!({"section_name":CHAPTER_A}),
                ("WRITER_A", "update_references"),
            )?;
            let references_b = imp::material(
                &references_b,
                &json!({"section_name":CHAPTER_B}),
                ("WRITER_B", "update_references"),
            )?;
            Ok((
                document,
                json!({"sections":{CHAPTER_A:a,CHAPTER_B:b},
                "references":{CHAPTER_A:references_a,CHAPTER_B:references_b}}),
            ))
        };
        async {
            Ok::<_, ShellError>(
                guest
                    .next_file::<ClosedFile, EDITOR>(&mut private, SERVICE_FD)
                    .await?,
            )
        };
    }
    .await?;
    let (mut pending, mut arguments, mut scope) = loop {
        match boundary {
            FileBoundary::Exit(_) => {
                return Err(ShellError::message("editor exited before document review"));
            }
            FileBoundary::Closed { pending, file } => {
                let request = ReportArguments::<5>::from_close(&file).and_then(|arguments| {
                    let value = imp::decode(&file.bytes)?;
                    Ok((arguments, imp::chapter(&value)?.to_owned()))
                });
                let error = match request {
                    Ok((arguments, scope)) => break (pending, arguments, scope),
                    Err(error) => error,
                };
                let bytes =
                    imp::encode(&json!({"status":"invalid_arguments","error":error.to_string()}))?;
                for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
                    private
                        .send::<ResultChunkMsg>(&ResultChunk::new(
                            index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                            chunk,
                        )?)
                        .await?;
                }
                pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
                    protocol::FdClosed::new_with_errno(SERVICE_FD, 28),
                )))?;
            }
        }
        boundary = guest
            .next_file::<ClosedFile, EDITOR>(&mut private, SERVICE_FD)
            .await?;
    };

    let reviews = loop {
        let first = if scope == CHAPTER_A {
            task.send::<ReviewA>(&arguments).await?;
            imp::decode(task.recv::<ReviewedA>().await?)?
        } else {
            task.send::<ReviewB>(&arguments).await?;
            imp::decode(task.recv::<ReviewedB>().await?)?
        };
        imp::reviewed(&first, &snapshot, &scope)?;
        let bytes = imp::encode(&first)?;
        for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
            private
                .send::<ResultChunkMsg>(&ResultChunk::new(
                    index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                    chunk,
                )?)
                .await?;
        }
        pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
            protocol::FdClosed::new_with_errno(SERVICE_FD, 0),
        )))?;

        let (second_pending, second_arguments, second_scope) = loop {
            match guest
                .next_file::<ClosedFile, EDITOR>(&mut private, SERVICE_FD)
                .await?
            {
                FileBoundary::Exit(_) => {
                    return Err(ShellError::message(
                        "editor exited before both complete-document reviews",
                    ));
                }
                FileBoundary::Closed { pending, file } => {
                    let request = ReportArguments::<5>::from_close(&file).and_then(|arguments| {
                        let value = imp::decode(&file.bytes)?;
                        let second = imp::chapter(&value)?.to_owned();
                        Ok((arguments, second))
                    });
                    let error = match request {
                        Ok((arguments, scope)) => break (pending, arguments, scope),
                        Err(error) => error,
                    };
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
                    pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
                        protocol::FdClosed::new_with_errno(SERVICE_FD, 28),
                    )))?;
                }
            }
        };
        let second = if second_scope == CHAPTER_A {
            task.send::<ReviewA>(&second_arguments).await?;
            imp::decode(task.recv::<ReviewedA>().await?)?
        } else {
            task.send::<ReviewB>(&second_arguments).await?;
            imp::decode(task.recv::<ReviewedB>().await?)?
        };
        imp::reviewed(&second, &snapshot, &second_scope)?;
        let bytes = imp::encode(&second)?;
        for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
            private
                .send::<ResultChunkMsg>(&ResultChunk::new(
                    index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                    chunk,
                )?)
                .await?;
        }
        second_pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
            protocol::FdClosed::new_with_errno(SERVICE_FD, 0),
        )))?;

        if first["success"] == true && second["success"] == true {
            break if scope == CHAPTER_A {
                [first, second]
            } else {
                [second, first]
            };
        }
        // Forward an actual negative review. If both are negative, preserve
        // the guest's first review order instead of inventing a workflow flag.
        let rejected = if first["success"] == false {
            first
        } else {
            second
        };
        let bytes = imp::encode(&rejected)?;
        let reply = ReviewReply::<false>::from_bytes(&bytes)?;
        let agent = if rejected["data"]["scope"] == CHAPTER_A {
            // This typed entry selects the intrinsic retry and writer choices.
            // par() only borrows a selected branch; it does not choose one.
            task.send::<ReconsiderA>(&reply).await?;
            async {
                let [service, mut writer, mut other] = task.par()?;
                drop(service); // The selected branch's first operation is complete.
                async {
                    writer.send::<RejectedA>(&reply).await?;
                };
                async {
                    other.send::<RejectedAForB>(&reply).await?;
                };
                Ok::<(), ShellError>(())
            }
            .await?;
            task.send::<RevisionDocument>(document).await?;
            "WRITER_A"
        } else {
            task.send::<ReconsiderB>(&reply).await?;
            async {
                let [service, mut writer, mut other] = task.par()?;
                drop(service);
                async {
                    writer.send::<RejectedB>(&reply).await?;
                };
                async {
                    other.send::<RejectedBForA>(&reply).await?;
                };
                Ok::<(), ShellError>(())
            }
            .await?;
            task.send::<RevisionDocument>(document).await?;
            "WRITER_B"
        };
        let ((returned, updated), next) = async {
            async {
                let [mut updated, mut returned] = task.par()?;
                async {
                    returned
                        .recv::<RevisionDocument>()
                        .await
                        .map_err(ShellError::from)
                };
                async { imp::decode(updated.recv::<UpdatedDocument>().await?) };
            };
            async {
                Ok::<_, ShellError>(
                    guest
                        .next_file::<ClosedFile, EDITOR>(&mut private, SERVICE_FD)
                        .await?,
                )
            };
        }
        .await?;
        document = returned;
        let chapter = updated["data"]["section"]
            .as_str()
            .ok_or_else(|| ShellError::message("revision lost its actual chapter"))?;
        let material = imp::material(
            &updated,
            &json!({"section_name":chapter}),
            (agent, "revise_section"),
        )?;
        snapshot["sections"][chapter] = material;
        boundary = next;
        (pending, arguments, scope) = loop {
            match boundary {
                FileBoundary::Exit(_) => {
                    return Err(ShellError::message(
                        "editor exited before fresh post-revision reviews",
                    ));
                }
                FileBoundary::Closed { pending, file } => {
                    let request = ReportArguments::<5>::from_close(&file).and_then(|arguments| {
                        let value = imp::decode(&file.bytes)?;
                        Ok((arguments, imp::chapter(&value)?.to_owned()))
                    });
                    let error = match request {
                        Ok((arguments, scope)) => break (pending, arguments, scope),
                        Err(error) => error,
                    };
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
                    pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
                        protocol::FdClosed::new_with_errno(SERVICE_FD, 28),
                    )))?;
                }
            }
            boundary = guest
                .next_file::<ClosedFile, EDITOR>(&mut private, SERVICE_FD)
                .await?;
        };
    };

    // The resource moves before publication is announced. The old editing
    // continuation has no authority left at this editor.
    task.send::<FrozenDocument>(document).await?;
    let bytes_a = imp::encode(&reviews[0])?;
    let bytes_b = imp::encode(&reviews[1])?;
    task.send::<FinalApprovedA>(&ReviewReply::<true>::from_bytes(&bytes_a)?)
        .await?;
    task.send::<FinalApprovedB>(&ReviewReply::<true>::from_bytes(&bytes_b)?)
        .await?;
    loop {
        match guest
            .next_file::<ClosedFile, EDITOR>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Exit(_) => {
                return Err(ShellError::message(
                    "editor exited before actual document combination",
                ));
            }
            FileBoundary::Closed { pending, file } => {
                let request = ReportArguments::<6>::from_close(&file).and_then(|arguments| {
                    let value = imp::decode(&file.bytes)?;
                    imp::combine_inputs(&value, &value, (&snapshot, &reviews))?;
                    Ok(arguments)
                });
                let (reply, errno) = match request {
                    Err(error) => (
                        json!({"status":"invalid_arguments","error":error.to_string()}),
                        28,
                    ),
                    Ok(arguments) => {
                        async {
                            let [mut service, mut writer] = task.par()?;
                            async {
                                service.send::<Combine>(&arguments).await?;
                            };
                            async {
                                writer.send::<CombineForward>(&arguments).await?;
                            };
                            Ok::<(), ShellError>(())
                        }
                        .await?;
                        let result = imp::decode(task.recv::<CombinedForEditor>().await?)?;
                        imp::combined(&result, &snapshot, &reviews)?;
                        let bytes = imp::encode(&result)?;
                        task.send::<FinishedB>(&bytes.as_slice()).await?;
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
                        pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
                            protocol::FdClosed::new_with_errno(SERVICE_FD, 0),
                        )))?;
                        break;
                    }
                };
                let bytes = imp::encode(&reply)?;
                for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
                    private
                        .send::<ResultChunkMsg>(&ResultChunk::new(
                            index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                            chunk,
                        )?)
                        .await?;
                }
                pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
                    protocol::FdClosed::new_with_errno(SERVICE_FD, errno),
                )))?;
            }
        }
    }
    drop(task);
    loop {
        match guest
            .next_file::<ClosedFile, EDITOR>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Exit(exit) => {
                let status = exit.status() as i32;
                private.send::<IoExit>(&status).await?;
                return Ok(status);
            }
            FileBoundary::Closed { pending, .. } => {
                let bytes = imp::encode(
                    &json!({"status":"out_of_order","error":"the actual document has completed"}),
                )?;
                for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
                    private
                        .send::<ResultChunkMsg>(&ResultChunk::new(
                            index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                            chunk,
                        )?)
                        .await?;
                }
                pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
                    protocol::FdClosed::new_with_errno(SERVICE_FD, 8),
                )))?;
            }
        }
    }
}
