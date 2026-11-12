//! The reviewer keeps its actual request/reply while following delegated authority.
use super::super::{
    global::{delivery::*, *},
    imp,
};
use crate::shell::{
    ShellError, ShellResult,
    global::Exit as IoExit,
    imp::{
        file_wire::{ClosedFile, ResultChunk, ResultChunkMsg, SERVICE_FD},
        repository_arguments::RepositoryArguments,
        review_reply::ReviewReply,
    },
};
use core::cell::Cell;
use hibana::{Endpoint, g::Message};
use hibana_wasip1_runtime::{FileBoundary, HibanaWasiGuest, WasiImportCompletion, protocol};
use serde_json::{Value, json};

#[hibana_runtime::scope]
pub async fn run(
    mut guest: &mut HibanaWasiGuest,
    endpoint: &mut Endpoint<REVIEWER>,
    actual_results: (&Cell<Value>, &Cell<Value>),
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    let (arguments, mut pending) = loop {
        match guest
            .next_file::<ClosedFile, REVIEWER>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Exit(_) => {
                return Err(ShellError::message(
                    "reviewer exited before the actual final CI",
                ));
            }
            FileBoundary::Closed { pending, file } => {
                let requested =
                    imp::decode(&file.bytes).and_then(|value| match value["pr_id"].as_str() {
                        Some(FEATURE_A) => super::wasi_inputs::arguments::<3>(
                            &file,
                            b"service/code_review",
                            ("pr_id", FEATURE_A),
                        ),
                        Some(FEATURE_B) => super::wasi_inputs::arguments::<3>(
                            &file,
                            b"service/code_review",
                            ("pr_id", FEATURE_B),
                        ),
                        _ => Err(ShellError::message("review names neither feature")),
                    });
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
    let first_request = arguments.into_value();
    let arguments = RepositoryArguments::from_bytes(&imp::encode(&first_request)?)?;
    let (mut feature_a, mut feature_b, mut result_a, mut result_b, mut current_review) =
        if first_request["pr_id"] == FEATURE_B {
            task.send::<ReviewFirstB>(&arguments).await?;
            task.send::<ReviewFirstB>(&arguments).await?;
            task.send::<ReviewFirstB>(&arguments).await?;
            task.send::<ReviewFirstB>(&arguments).await?;
            let mut feature_b = task.recv::<Feature<AUTH_B>>().await?;
            feature_b.send::<Review>(&arguments).await?;
            let result_b = imp::decode(feature_b.recv::<ReviewResultB>().await?)?;
            imp::arguments(&result_b["data"], ("target", FEATURE_B))?;
            let bytes = imp::encode(&result_b)?;
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

            let (arguments, next_pending) = loop {
                match guest
                    .next_file::<ClosedFile, REVIEWER>(&mut private, SERVICE_FD)
                    .await?
                {
                    FileBoundary::Exit(_) => {
                        return Err(ShellError::message(
                            "reviewer exited before the actual final CI",
                        ));
                    }
                    FileBoundary::Closed { pending, file } => {
                        let requested = super::wasi_inputs::arguments::<3>(
                            &file,
                            b"service/code_review",
                            ("pr_id", FEATURE_A),
                        );
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
            pending = next_pending;
            let mut feature_a = task.recv::<Feature<AUTH_A>>().await?;
            feature_a.send::<Review>(&arguments).await?;
            let result_a = imp::decode(feature_a.recv::<ReviewResultA>().await?)?;
            imp::arguments(&result_a["data"], ("target", FEATURE_A))?;

            let current_review = result_a.clone();
            (feature_a, feature_b, result_a, result_b, current_review)
        } else {
            task.send::<ReviewFirstA>(&arguments).await?;
            task.send::<ReviewFirstA>(&arguments).await?;
            task.send::<ReviewFirstA>(&arguments).await?;
            task.send::<ReviewFirstA>(&arguments).await?;
            let mut feature_a = task.recv::<Feature<AUTH_A>>().await?;
            feature_a.send::<Review>(&arguments).await?;
            let result_a = imp::decode(feature_a.recv::<ReviewResultA>().await?)?;
            imp::arguments(&result_a["data"], ("target", FEATURE_A))?;
            let bytes = imp::encode(&result_a)?;
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

            let (arguments, next_pending) = loop {
                match guest
                    .next_file::<ClosedFile, REVIEWER>(&mut private, SERVICE_FD)
                    .await?
                {
                    FileBoundary::Exit(_) => {
                        return Err(ShellError::message(
                            "reviewer exited before the actual final CI",
                        ));
                    }
                    FileBoundary::Closed { pending, file } => {
                        let requested = super::wasi_inputs::arguments::<3>(
                            &file,
                            b"service/code_review",
                            ("pr_id", FEATURE_B),
                        );
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
            pending = next_pending;
            let mut feature_b = task.recv::<Feature<AUTH_B>>().await?;
            feature_b.send::<Review>(&arguments).await?;
            let result_b = imp::decode(feature_b.recv::<ReviewResultB>().await?)?;
            imp::arguments(&result_b["data"], ("target", FEATURE_B))?;

            let current_review = result_b.clone();
            (feature_a, feature_b, result_a, result_b, current_review)
        };
    if first_request["pr_id"] == FEATURE_B {
        loop {
            actual_results.1.set(result_b.clone());
            let bytes = imp::encode(&result_b)?;
            if result_b["success"] == true {
                let response = ReviewReply::<true>::from_bytes(&bytes)?;
                feature_b.send::<Approved>(&response).await?;
                feature_b.send::<ReviewApproved>(&response).await?;
                feature_b.send::<ReviewApproved>(&response).await?;
                feature_b.send::<ReviewApproved>(&response).await?;
                feature_b.send::<ReviewApproved>(&response).await?;
                feature_b.send::<ReviewApproved>(&response).await?;
                feature_b.send::<ReviewApproved>(&response).await?;
                task.send::<Feature<AUTH_B>>(feature_b).await?;
                break;
            } else {
                let response = ReviewReply::<false>::from_bytes(&bytes)?;
                feature_b.send::<Rejected>(&response).await?;
                feature_b.send::<ReviewRejected>(&response).await?;
                feature_b.send::<ReviewRejected>(&response).await?;
                feature_b.send::<ReviewRejected>(&response).await?;
                feature_b.send::<ReviewRejected>(&response).await?;
                feature_b.send::<ReviewRejected>(&response).await?;
                feature_b.send::<ReviewRejected>(&response).await?;
                task.send::<Feature<AUTH_B>>(feature_b).await?;
                let mut response = current_review.clone();
                response["coordination"] = json!({"code_review":result_b});
                let bytes = imp::encode(&response)?;
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
                let(next_feature,(arguments,next_pending,returned_private))=async{
 async{Ok::<_,ShellError>(task.recv::<Feature<AUTH_B>>().await?)};
 async move{
 let mut guest=guest;let mut private=private;
 let(arguments,pending)=loop{match guest.next_file::<ClosedFile,REVIEWER>(&mut private,SERVICE_FD).await?{
 FileBoundary::Exit(_)=>return Err(ShellError::message("reviewer exited before the actual final CI")),
 FileBoundary::Closed{pending,file}=>{
 let requested=super::wasi_inputs::arguments::<3>(&file,b"service/code_review",("pr_id",FEATURE_B));
 match requested{Ok(arguments)=>break(arguments,pending),Err(error)=>{let bytes=imp::encode(&json!({"status":"invalid_arguments","error":error.to_string()}))?;
 for(index,chunk)in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate(){private.send::<ResultChunkMsg>(&ResultChunk::new(index*protocol::WASIP1_IO_CHUNK_CAPACITY,chunk)?).await?;}
 guest=pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(protocol::FdClosed::new_with_errno(SERVICE_FD,28))))?;}}
 }}};
 Ok::<_,ShellError>((arguments,pending,private))
 };
 }.await?;
                private = returned_private;
                pending = next_pending;
                feature_b = next_feature;
                feature_b.send::<Review>(&arguments).await?;
                result_b = imp::decode(feature_b.recv::<ReviewResultB>().await?)?;
                imp::arguments(&result_b["data"], ("target", FEATURE_B))?;
                current_review = result_b.clone();
            }
        }
        loop {
            actual_results.0.set(result_a.clone());
            let bytes = imp::encode(&result_a)?;
            if result_a["success"] == true {
                let response = ReviewReply::<true>::from_bytes(&bytes)?;
                feature_a.send::<Approved>(&response).await?;
                feature_a.send::<ReviewApproved>(&response).await?;
                feature_a.send::<ReviewApproved>(&response).await?;
                feature_a.send::<ReviewApproved>(&response).await?;
                feature_a.send::<ReviewApproved>(&response).await?;
                feature_a.send::<ReviewApproved>(&response).await?;
                feature_a.send::<ReviewApproved>(&response).await?;
                task.send::<Feature<AUTH_A>>(feature_a).await?;
                break;
            } else {
                let response = ReviewReply::<false>::from_bytes(&bytes)?;
                feature_a.send::<Rejected>(&response).await?;
                feature_a.send::<ReviewRejected>(&response).await?;
                feature_a.send::<ReviewRejected>(&response).await?;
                feature_a.send::<ReviewRejected>(&response).await?;
                feature_a.send::<ReviewRejected>(&response).await?;
                feature_a.send::<ReviewRejected>(&response).await?;
                feature_a.send::<ReviewRejected>(&response).await?;
                task.send::<Feature<AUTH_A>>(feature_a).await?;
                let mut response = current_review.clone();
                response["coordination"] = json!({"code_review":result_a});
                let bytes = imp::encode(&response)?;
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
                let(next_feature,(arguments,next_pending,returned_private))=async{
 async{Ok::<_,ShellError>(task.recv::<Feature<AUTH_A>>().await?)};
 async move{
 let mut guest=guest;let mut private=private;
 let(arguments,pending)=loop{match guest.next_file::<ClosedFile,REVIEWER>(&mut private,SERVICE_FD).await?{
 FileBoundary::Exit(_)=>return Err(ShellError::message("reviewer exited before the actual final CI")),
 FileBoundary::Closed{pending,file}=>{
 let requested=super::wasi_inputs::arguments::<3>(&file,b"service/code_review",("pr_id",FEATURE_A));
 match requested{Ok(arguments)=>break(arguments,pending),Err(error)=>{let bytes=imp::encode(&json!({"status":"invalid_arguments","error":error.to_string()}))?;
 for(index,chunk)in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate(){private.send::<ResultChunkMsg>(&ResultChunk::new(index*protocol::WASIP1_IO_CHUNK_CAPACITY,chunk)?).await?;}
 guest=pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(protocol::FdClosed::new_with_errno(SERVICE_FD,28))))?;}}
 }}};
 Ok::<_,ShellError>((arguments,pending,private))
 };
 }.await?;
                private = returned_private;
                pending = next_pending;
                feature_a = next_feature;
                feature_a.send::<Review>(&arguments).await?;
                result_a = imp::decode(feature_a.recv::<ReviewResultA>().await?)?;
                imp::arguments(&result_a["data"], ("target", FEATURE_A))?;
                current_review = result_a.clone();
            }
        }
    } else {
        loop {
            actual_results.0.set(result_a.clone());
            let bytes = imp::encode(&result_a)?;
            if result_a["success"] == true {
                let response = ReviewReply::<true>::from_bytes(&bytes)?;
                feature_a.send::<Approved>(&response).await?;
                feature_a.send::<ReviewApproved>(&response).await?;
                feature_a.send::<ReviewApproved>(&response).await?;
                feature_a.send::<ReviewApproved>(&response).await?;
                feature_a.send::<ReviewApproved>(&response).await?;
                feature_a.send::<ReviewApproved>(&response).await?;
                feature_a.send::<ReviewApproved>(&response).await?;
                task.send::<Feature<AUTH_A>>(feature_a).await?;
                break;
            } else {
                let response = ReviewReply::<false>::from_bytes(&bytes)?;
                feature_a.send::<Rejected>(&response).await?;
                feature_a.send::<ReviewRejected>(&response).await?;
                feature_a.send::<ReviewRejected>(&response).await?;
                feature_a.send::<ReviewRejected>(&response).await?;
                feature_a.send::<ReviewRejected>(&response).await?;
                feature_a.send::<ReviewRejected>(&response).await?;
                feature_a.send::<ReviewRejected>(&response).await?;
                task.send::<Feature<AUTH_A>>(feature_a).await?;
                let mut response = current_review.clone();
                response["coordination"] = json!({"code_review":result_a});
                let bytes = imp::encode(&response)?;
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
                let(next_feature,(arguments,next_pending,returned_private))=async{
 async{Ok::<_,ShellError>(task.recv::<Feature<AUTH_A>>().await?)};
 async move{
 let mut guest=guest;let mut private=private;
 let(arguments,pending)=loop{match guest.next_file::<ClosedFile,REVIEWER>(&mut private,SERVICE_FD).await?{
 FileBoundary::Exit(_)=>return Err(ShellError::message("reviewer exited before the actual final CI")),
 FileBoundary::Closed{pending,file}=>{
 let requested=super::wasi_inputs::arguments::<3>(&file,b"service/code_review",("pr_id",FEATURE_A));
 match requested{Ok(arguments)=>break(arguments,pending),Err(error)=>{let bytes=imp::encode(&json!({"status":"invalid_arguments","error":error.to_string()}))?;
 for(index,chunk)in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate(){private.send::<ResultChunkMsg>(&ResultChunk::new(index*protocol::WASIP1_IO_CHUNK_CAPACITY,chunk)?).await?;}
 guest=pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(protocol::FdClosed::new_with_errno(SERVICE_FD,28))))?;}}
 }}};
 Ok::<_,ShellError>((arguments,pending,private))
 };
 }.await?;
                private = returned_private;
                pending = next_pending;
                feature_a = next_feature;
                feature_a.send::<Review>(&arguments).await?;
                result_a = imp::decode(feature_a.recv::<ReviewResultA>().await?)?;
                imp::arguments(&result_a["data"], ("target", FEATURE_A))?;
                current_review = result_a.clone();
            }
        }
        loop {
            actual_results.1.set(result_b.clone());
            let bytes = imp::encode(&result_b)?;
            if result_b["success"] == true {
                let response = ReviewReply::<true>::from_bytes(&bytes)?;
                feature_b.send::<Approved>(&response).await?;
                feature_b.send::<ReviewApproved>(&response).await?;
                feature_b.send::<ReviewApproved>(&response).await?;
                feature_b.send::<ReviewApproved>(&response).await?;
                feature_b.send::<ReviewApproved>(&response).await?;
                feature_b.send::<ReviewApproved>(&response).await?;
                feature_b.send::<ReviewApproved>(&response).await?;
                task.send::<Feature<AUTH_B>>(feature_b).await?;
                break;
            } else {
                let response = ReviewReply::<false>::from_bytes(&bytes)?;
                feature_b.send::<Rejected>(&response).await?;
                feature_b.send::<ReviewRejected>(&response).await?;
                feature_b.send::<ReviewRejected>(&response).await?;
                feature_b.send::<ReviewRejected>(&response).await?;
                feature_b.send::<ReviewRejected>(&response).await?;
                feature_b.send::<ReviewRejected>(&response).await?;
                feature_b.send::<ReviewRejected>(&response).await?;
                task.send::<Feature<AUTH_B>>(feature_b).await?;
                let mut response = current_review.clone();
                response["coordination"] = json!({"code_review":result_b});
                let bytes = imp::encode(&response)?;
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
                let(next_feature,(arguments,next_pending,returned_private))=async{
 async{Ok::<_,ShellError>(task.recv::<Feature<AUTH_B>>().await?)};
 async move{
 let mut guest=guest;let mut private=private;
 let(arguments,pending)=loop{match guest.next_file::<ClosedFile,REVIEWER>(&mut private,SERVICE_FD).await?{
 FileBoundary::Exit(_)=>return Err(ShellError::message("reviewer exited before the actual final CI")),
 FileBoundary::Closed{pending,file}=>{
 let requested=super::wasi_inputs::arguments::<3>(&file,b"service/code_review",("pr_id",FEATURE_B));
 match requested{Ok(arguments)=>break(arguments,pending),Err(error)=>{let bytes=imp::encode(&json!({"status":"invalid_arguments","error":error.to_string()}))?;
 for(index,chunk)in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate(){private.send::<ResultChunkMsg>(&ResultChunk::new(index*protocol::WASIP1_IO_CHUNK_CAPACITY,chunk)?).await?;}
 guest=pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(protocol::FdClosed::new_with_errno(SERVICE_FD,28))))?;}}
 }}};
 Ok::<_,ShellError>((arguments,pending,private))
 };
 }.await?;
                private = returned_private;
                pending = next_pending;
                feature_b = next_feature;
                feature_b.send::<Review>(&arguments).await?;
                result_b = imp::decode(feature_b.recv::<ReviewResultB>().await?)?;
                imp::arguments(&result_b["data"], ("target", FEATURE_B))?;
                current_review = result_b.clone();
            }
        }
    }
    let mut outcomes = Vec::new();
    loop {
        let branch = task.offer().await?;
        match branch.label() {
            CiPassedA::LOGICAL_LABEL => {
                outcomes.push(imp::decode(branch.recv::<CiPassedA>().await?)?);
            }
            CiPassedB::LOGICAL_LABEL => {
                outcomes.push(imp::decode(branch.recv::<CiPassedB>().await?)?);
            }
            LastCiPassedA::LOGICAL_LABEL => {
                let completed = imp::decode(branch.recv::<LastCiPassedA>().await?)?;
                let mut response = current_review;
                response["coordination"] = json!({"outcomes":outcomes,"ci":completed});
                let bytes = imp::encode(&response)?;
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
                loop {
                    match guest
                        .next_file::<ClosedFile, REVIEWER>(&mut private, SERVICE_FD)
                        .await?
                    {
                        FileBoundary::Exit(exit) => {
                            let code = exit.status() as i32;
                            private.send::<IoExit>(&code).await?;
                            return Ok(code);
                        }
                        FileBoundary::Closed { pending, .. } => {
                            let bytes = imp::encode(
                                &json!({"status":"out_of_order","error":"both features have passed their actual CI"}),
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
                                    SERVICE_FD, 8,
                                )),
                            ))?;
                        }
                    }
                }
            }
            LastCiPassedB::LOGICAL_LABEL => {
                let completed = imp::decode(branch.recv::<LastCiPassedB>().await?)?;
                let mut response = current_review;
                response["coordination"] = json!({"outcomes":outcomes,"ci":completed});
                let bytes = imp::encode(&response)?;
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
                loop {
                    match guest
                        .next_file::<ClosedFile, REVIEWER>(&mut private, SERVICE_FD)
                        .await?
                    {
                        FileBoundary::Exit(exit) => {
                            let code = exit.status() as i32;
                            private.send::<IoExit>(&code).await?;
                            return Ok(code);
                        }
                        FileBoundary::Closed { pending, .. } => {
                            let bytes = imp::encode(
                                &json!({"status":"out_of_order","error":"both features have passed their actual CI"}),
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
                                    SERVICE_FD, 8,
                                )),
                            ))?;
                        }
                    }
                }
            }
            CiFailedA::LOGICAL_LABEL => {
                let failed = imp::decode(branch.recv::<CiFailedA>().await?)?;
                outcomes.push(failed);
                let mut response = current_review.clone();
                response["coordination"] = json!({"outcomes":outcomes});
                let bytes = imp::encode(&response)?;
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
                outcomes.clear();
                loop {
                    let (arguments, next_pending) = loop {
                        match guest
                            .next_file::<ClosedFile, REVIEWER>(&mut private, SERVICE_FD)
                            .await?
                        {
                            FileBoundary::Exit(_) => {
                                return Err(ShellError::message(
                                    "reviewer exited before the actual final CI",
                                ));
                            }
                            FileBoundary::Closed { pending, file } => {
                                let requested = super::wasi_inputs::arguments::<3>(
                                    &file,
                                    b"service/code_review",
                                    ("pr_id", FEATURE_A),
                                );
                                match requested {
                                    Ok(arguments) => break (arguments, pending),
                                    Err(error) => {
                                        let bytes = imp::encode(
                                            &json!({"status":"invalid_arguments","error":error.to_string()}),
                                        )?;
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
                                }
                            }
                        }
                    };
                    pending = next_pending;
                    task.send::<ReviewRequestA>(&arguments).await?;
                    let branch = task.offer().await?;
                    current_review = match branch.label() {
                        RecoveryApproved::LOGICAL_LABEL => {
                            imp::decode(branch.recv::<RecoveryApproved>().await?)?
                        }
                        RecoveryRejected::LOGICAL_LABEL => {
                            imp::decode(branch.recv::<RecoveryRejected>().await?)?
                        }
                        _ => {
                            return Err(ShellError::message("expected the actual recovery review"));
                        }
                    };
                    imp::arguments(&current_review["data"], ("target", FEATURE_A))?;
                    if current_review["success"] == true {
                        break;
                    }
                    let response = current_review.clone();
                    let bytes = imp::encode(&response)?;
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
                        protocol::FdClosedRet(protocol::FdClosed::new_with_errno(SERVICE_FD, 0)),
                    ))?;
                }
            }
            CiFailedB::LOGICAL_LABEL => {
                let failed = imp::decode(branch.recv::<CiFailedB>().await?)?;
                outcomes.push(failed);
                let mut response = current_review.clone();
                response["coordination"] = json!({"outcomes":outcomes});
                let bytes = imp::encode(&response)?;
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
                outcomes.clear();
                loop {
                    let (arguments, next_pending) = loop {
                        match guest
                            .next_file::<ClosedFile, REVIEWER>(&mut private, SERVICE_FD)
                            .await?
                        {
                            FileBoundary::Exit(_) => {
                                return Err(ShellError::message(
                                    "reviewer exited before the actual final CI",
                                ));
                            }
                            FileBoundary::Closed { pending, file } => {
                                let requested = super::wasi_inputs::arguments::<3>(
                                    &file,
                                    b"service/code_review",
                                    ("pr_id", FEATURE_B),
                                );
                                match requested {
                                    Ok(arguments) => break (arguments, pending),
                                    Err(error) => {
                                        let bytes = imp::encode(
                                            &json!({"status":"invalid_arguments","error":error.to_string()}),
                                        )?;
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
                                }
                            }
                        }
                    };
                    pending = next_pending;
                    task.send::<ReviewRequestB>(&arguments).await?;
                    let branch = task.offer().await?;
                    current_review = match branch.label() {
                        RecoveryApproved::LOGICAL_LABEL => {
                            imp::decode(branch.recv::<RecoveryApproved>().await?)?
                        }
                        RecoveryRejected::LOGICAL_LABEL => {
                            imp::decode(branch.recv::<RecoveryRejected>().await?)?
                        }
                        _ => {
                            return Err(ShellError::message("expected the actual recovery review"));
                        }
                    };
                    imp::arguments(&current_review["data"], ("target", FEATURE_B))?;
                    if current_review["success"] == true {
                        break;
                    }
                    let response = current_review.clone();
                    let bytes = imp::encode(&response)?;
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
                        protocol::FdClosedRet(protocol::FdClosed::new_with_errno(SERVICE_FD, 0)),
                    ))?;
                }
            }
            _ => return Err(ShellError::message("expected the next actual CI outcome")),
        }
    }
}
