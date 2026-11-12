//! A wash guest owns its private I/O; feature authority follows the global.
use super::super::{
    global::{delivery::*, *},
    imp,
};
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
    mut guest: &mut HibanaWasiGuest,
    endpoint: &mut Endpoint<DEVELOPER_B>,
    actual_review: &Cell<Value>,
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    let ((mut feature,selected),(mut arguments,mut pending,returned_private)) = async{
 async{
 let feature=task.recv::<Feature<AUTH_B>>().await?;
 let branch=task.offer().await?;
 let selected=match branch.label(){
 ReviewFirstA::LOGICAL_LABEL=>branch.recv::<ReviewFirstA>().await?.into_value(),
 ReviewFirstB::LOGICAL_LABEL=>branch.recv::<ReviewFirstB>().await?.into_value(),
 _=>return Err(ShellError::message("missing actual first review"))};
 Ok::<_,ShellError>((feature,selected))
 };
 async move {
   let mut guest=guest;
   let mut private=private;
   let (arguments,pending)=loop {
 match guest.next_file::<ClosedFile,DEVELOPER_B>(&mut private,SERVICE_FD).await?{
 FileBoundary::Exit(_)=>return Err(ShellError::message("developer exited before its required request")),
 FileBoundary::Closed{pending,file}=>{
 let requested=super::wasi_inputs::arguments::<1>(&file,b"service/implement_feature",("feature_name",FEATURE_B));
 match requested {
 Ok(arguments)=>break (arguments,pending),
 Err(error)=>{
 let bytes=imp::encode(&json!({"status":"invalid_arguments","error":error.to_string()}))?;
 for(index,chunk)in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate(){
 private.send::<ResultChunkMsg>(&ResultChunk::new(index*protocol::WASIP1_IO_CHUNK_CAPACITY,chunk)?).await?;
 }
 guest=pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(protocol::FdClosed::new_with_errno(SERVICE_FD,28))))?;
 }
 }}
 }
};
   Ok::<_,ShellError>((arguments,pending,private))
  };
 }.await?;
    private = returned_private;
    let mut peer = Value::Null;
    if selected["pr_id"] == FEATURE_A {
        peer = imp::decode(task.recv::<InterfaceChangedA>().await?)?;
        imp::candidate(&peer, FEATURE_A)?;
    }
    task.send::<ImplementB>(&arguments).await?;
    let mut repository = task.recv::<RepoLoan>().await?;
    async {
        async {
            feature.send::<Implement>(&arguments).await?;
        };
        async {
            repository.send::<ImplementOnRepo>(&arguments).await?;
        };
        Ok::<_, ShellError>(())
    }
    .await?;
    let candidate = imp::decode(repository.recv::<RepoResult>().await?)?;
    imp::candidate(&candidate, FEATURE_B)?;
    task.send::<RepoReturn>(repository).await?;
    let changed = imp::encode(&candidate)?;
    feature
        .send::<InterfaceChangedB>(&changed.as_slice())
        .await?;
    let mut implemented = candidate;
    task.send::<Feature<AUTH_B>>(feature).await?;
    if selected["pr_id"] == FEATURE_B {
        peer = imp::decode(task.recv::<InterfaceChangedA>().await?)?;
        imp::candidate(&peer, FEATURE_A)?;
    }
    loop {
        let tests = imp::decode(task.recv::<TestedB>().await?)?;
        imp::tested(&tests, &implemented, FEATURE_B)?;
        let review = loop {
            let branch = task.offer().await?;
            match branch.label() {
                InterfaceChangedA::LOGICAL_LABEL => {
                    peer = imp::decode(branch.recv::<InterfaceChangedA>().await?)?;
                    imp::candidate(&peer, FEATURE_A)?;
                }
                ReviewApproved::LOGICAL_LABEL => {
                    let result = imp::decode(branch.recv::<ReviewApproved>().await?)?;
                    imp::reviewed(&result, &peer, FEATURE_A)?;
                }
                ReviewRejected::LOGICAL_LABEL => {
                    let result = imp::decode(branch.recv::<ReviewRejected>().await?)?;
                    imp::reviewed(&result, &peer, FEATURE_A)?;
                }
                RecoveryReviewApproved::LOGICAL_LABEL => {
                    let result = imp::decode(branch.recv::<RecoveryReviewApproved>().await?)?;
                    imp::reviewed(&result, &peer, FEATURE_A)?;
                }
                RecoveryReviewRejected::LOGICAL_LABEL => {
                    let result = imp::decode(branch.recv::<RecoveryReviewRejected>().await?)?;
                    imp::reviewed(&result, &peer, FEATURE_A)?;
                }
                Approved::LOGICAL_LABEL => break imp::decode(branch.recv::<Approved>().await?)?,
                Rejected::LOGICAL_LABEL => break imp::decode(branch.recv::<Rejected>().await?)?,
                _ => {
                    return Err(ShellError::message(
                        "expected this feature's review continuation",
                    ));
                }
            }
        };
        imp::reviewed(&review, &implemented, FEATURE_B)?;
        let mut result = implemented.clone();
        result["coordination"] = json!({"tests":tests,"code_review":review,"peer_candidate":peer});
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
        if review["success"] == true {
            break;
        }
        feature = task.recv::<Feature<AUTH_B>>().await?;
        (arguments, pending) = loop {
            match guest
                .next_file::<ClosedFile, DEVELOPER_B>(&mut private, SERVICE_FD)
                .await?
            {
                FileBoundary::Exit(_) => {
                    return Err(ShellError::message(
                        "developer exited before its required request",
                    ));
                }
                FileBoundary::Closed { pending, file } => {
                    let requested = super::wasi_inputs::arguments::<1>(
                        &file,
                        b"service/implement_feature",
                        ("feature_name", FEATURE_B),
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
        task.send::<ImplementB>(&arguments).await?;
        let mut repository = task.recv::<RepoLoan>().await?;
        async {
            async {
                feature.send::<Implement>(&arguments).await?;
            };
            async {
                repository.send::<ImplementOnRepo>(&arguments).await?;
            };
            Ok::<_, ShellError>(())
        }
        .await?;
        let candidate = imp::decode(repository.recv::<RepoResult>().await?)?;
        imp::candidate(&candidate, FEATURE_B)?;
        task.send::<RepoReturn>(repository).await?;
        let changed = imp::encode(&candidate)?;
        feature
            .send::<InterfaceChangedB>(&changed.as_slice())
            .await?;
        implemented = candidate;
        task.send::<Feature<AUTH_B>>(feature).await?;
    }
    let (mut feature,(mut merge_arguments,mut merge_pending,returned_private)) = async{
 async{
 loop{let branch=task.offer().await?;match branch.label(){
 InterfaceChangedA::LOGICAL_LABEL=>{peer=imp::decode(branch.recv::<InterfaceChangedA>().await?)?;imp::candidate(&peer,FEATURE_A)?;}
 ReviewApproved::LOGICAL_LABEL=>{let result=imp::decode(branch.recv::<ReviewApproved>().await?)?;imp::reviewed(&result,&peer,FEATURE_A)?;}
 ReviewRejected::LOGICAL_LABEL=>{let result=imp::decode(branch.recv::<ReviewRejected>().await?)?;imp::reviewed(&result,&peer,FEATURE_A)?;}
 RecoveryReviewApproved::LOGICAL_LABEL=>{let result=imp::decode(branch.recv::<RecoveryReviewApproved>().await?)?;imp::reviewed(&result,&peer,FEATURE_A)?;}
 RecoveryReviewRejected::LOGICAL_LABEL=>{let result=imp::decode(branch.recv::<RecoveryReviewRejected>().await?)?;imp::reviewed(&result,&peer,FEATURE_A)?;}
 CiPassedA::LOGICAL_LABEL=>{branch.recv::<CiPassedA>().await?;}
 CiFailedA::LOGICAL_LABEL=>{branch.recv::<CiFailedA>().await?;}
 MergeLoanB::LOGICAL_LABEL=>break Ok::<_,ShellError>(branch.recv::<MergeLoanB>().await?),
 _=>return Err(ShellError::message("expected this feature's merge authority"))
 }}
 };
 async move {
   let mut guest=guest;
   let mut private=private;
   let (arguments,pending)=loop {
 match guest.next_file::<ClosedFile,DEVELOPER_B>(&mut private,SERVICE_FD).await?{
 FileBoundary::Exit(_)=>return Err(ShellError::message("developer exited before its required request")),
 FileBoundary::Closed{pending,file}=>{
 let requested=super::wasi_inputs::arguments::<2>(&file,b"service/merge_code",("branch",FEATURE_B));
 match requested {
 Ok(arguments)=>break (arguments,pending),
 Err(error)=>{
 let bytes=imp::encode(&json!({"status":"invalid_arguments","error":error.to_string()}))?;
 for(index,chunk)in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate(){
 private.send::<ResultChunkMsg>(&ResultChunk::new(index*protocol::WASIP1_IO_CHUNK_CAPACITY,chunk)?).await?;
 }
 guest=pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(protocol::FdClosed::new_with_errno(SERVICE_FD,28))))?;
 }
 }}
 }
};
   Ok::<_,ShellError>((arguments,pending,private))
  };
 }.await?;
    private = returned_private;
    loop {
        task.send::<MergeAdmissionB>(&merge_arguments).await?;
        let mut repository = task.recv::<RepoLoan>().await?;
        async {
            async {
                feature.send::<Merge>(&merge_arguments).await?;
            };
            async {
                repository.send::<MergeOnRepo>(&merge_arguments).await?;
            };
            Ok::<_, ShellError>(())
        }
        .await?;
        let publication = imp::decode(feature.recv::<Merged>().await?)?;
        imp::merged(&publication, FEATURE_B)?;
        let bytes = imp::encode(&publication)?;
        task.send::<RepoReturn>(repository).await?;
        task.send::<Merged>(&bytes.as_slice()).await?;
        task.send::<Feature<AUTH_B>>(feature).await?;
        let ci = loop {
            let branch = task.offer().await?;
            match branch.label() {
                InterfaceChangedA::LOGICAL_LABEL => {
                    peer = imp::decode(branch.recv::<InterfaceChangedA>().await?)?;
                    imp::candidate(&peer, FEATURE_A)?;
                }
                ReviewApproved::LOGICAL_LABEL => {
                    let result = imp::decode(branch.recv::<ReviewApproved>().await?)?;
                    imp::reviewed(&result, &peer, FEATURE_A)?;
                }
                ReviewRejected::LOGICAL_LABEL => {
                    let result = imp::decode(branch.recv::<ReviewRejected>().await?)?;
                    imp::reviewed(&result, &peer, FEATURE_A)?;
                }
                RecoveryReviewApproved::LOGICAL_LABEL => {
                    let result = imp::decode(branch.recv::<RecoveryReviewApproved>().await?)?;
                    imp::reviewed(&result, &peer, FEATURE_A)?;
                }
                RecoveryReviewRejected::LOGICAL_LABEL => {
                    let result = imp::decode(branch.recv::<RecoveryReviewRejected>().await?)?;
                    imp::reviewed(&result, &peer, FEATURE_A)?;
                }
                CiPassedB::LOGICAL_LABEL => break imp::decode(branch.recv::<CiPassedB>().await?)?,
                CiFailedB::LOGICAL_LABEL => break imp::decode(branch.recv::<CiFailedB>().await?)?,
                LastCiPassedB::LOGICAL_LABEL => {
                    let ci = imp::decode(branch.recv::<LastCiPassedB>().await?)?;
                    imp::ci_publication(&ci, &publication)?;
                    let mut result = publication;
                    result["coordination"] = json!({"ci":ci,"peer_candidate":peer});
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
                    guest = merge_pending.complete(WasiImportCompletion::FdClose(
                        protocol::FdClosedRet(protocol::FdClosed::new_with_errno(SERVICE_FD, 0)),
                    ))?;
                    return loop {
                        match guest
                            .next_file::<ClosedFile, DEVELOPER_B>(&mut private, SERVICE_FD)
                            .await?
                        {
                            FileBoundary::Exit(exit) => {
                                let code = exit.status() as i32;
                                private.send::<IoExit>(&code).await?;
                                break Ok::<_, ShellError>(code);
                            }
                            FileBoundary::Closed { pending, .. } => {
                                let bytes = imp::encode(
                                    &json!({"status":"out_of_order","error":"the assigned feature has passed its final CI"}),
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
                    };
                }
                _ => {
                    return Err(ShellError::message(
                        "expected this feature's actual CI outcome",
                    ));
                }
            }
        };
        imp::ci_publication(&ci, &publication)?;
        let mut result = publication;
        result["coordination"] = json!({"ci":ci,"peer_candidate":peer});
        let bytes = imp::encode(&result)?;
        for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
            private
                .send::<ResultChunkMsg>(&ResultChunk::new(
                    index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                    chunk,
                )?)
                .await?;
        }
        guest = merge_pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
            protocol::FdClosed::new_with_errno(SERVICE_FD, 0),
        )))?;
        if ci["success"] == true {
            break;
        }
        feature = task.recv::<Feature<AUTH_B>>().await?;
        loop {
            let (arguments, pending) = loop {
                match guest
                    .next_file::<ClosedFile, DEVELOPER_B>(&mut private, SERVICE_FD)
                    .await?
                {
                    FileBoundary::Exit(_) => {
                        return Err(ShellError::message(
                            "developer exited before its required request",
                        ));
                    }
                    FileBoundary::Closed { pending, file } => {
                        let requested = super::wasi_inputs::arguments::<1>(
                            &file,
                            b"service/implement_feature",
                            ("feature_name", FEATURE_B),
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
            task.send::<ImplementB>(&arguments).await?;
            let mut repository = task.recv::<RepoLoan>().await?;
            async {
                async {
                    feature.send::<Implement>(&arguments).await?;
                };
                async {
                    repository.send::<ImplementOnRepo>(&arguments).await?;
                };
                Ok::<_, ShellError>(())
            }
            .await?;
            let candidate = imp::decode(repository.recv::<RepoResult>().await?)?;
            imp::candidate(&candidate, FEATURE_B)?;
            task.send::<RepoReturn>(repository).await?;
            let changed = imp::encode(&candidate)?;
            feature
                .send::<InterfaceChangedB>(&changed.as_slice())
                .await?;
            let tests = imp::decode(task.recv::<TestedB>().await?)?;
            imp::tested(&tests, &candidate, FEATURE_B)?;
            let review_args =
                crate::shell::imp::repository_arguments::RepositoryArguments::from_bytes(
                    br#"{"pr_id":"payment_processing"}"#,
                )?;
            feature.send::<Review>(&review_args).await?;
            let review = imp::decode(feature.recv::<ReviewResultB>().await?)?;
            imp::reviewed(&review, &candidate, FEATURE_B)?;
            actual_review.set(review.clone());
            let bytes = imp::encode(&review)?;
            if review["success"] == true {
                let reply = ReviewReply::<true>::from_bytes(&bytes)?;
                feature.send::<RecoveryApproved>(&reply).await?;
                feature.send::<RecoveryReviewApproved>(&reply).await?;
                feature.send::<RecoveryReviewApproved>(&reply).await?;
                feature.send::<RecoveryReviewApproved>(&reply).await?;
                feature.send::<RecoveryReviewApproved>(&reply).await?;
                feature.send::<RecoveryReviewApproved>(&reply).await?;
                feature.send::<RecoveryReviewApproved>(&reply).await?;
            } else {
                let reply = ReviewReply::<false>::from_bytes(&bytes)?;
                feature.send::<RecoveryRejected>(&reply).await?;
                feature.send::<RecoveryReviewRejected>(&reply).await?;
                feature.send::<RecoveryReviewRejected>(&reply).await?;
                feature.send::<RecoveryReviewRejected>(&reply).await?;
                feature.send::<RecoveryReviewRejected>(&reply).await?;
                feature.send::<RecoveryReviewRejected>(&reply).await?;
                feature.send::<RecoveryReviewRejected>(&reply).await?;
            }
            let mut result = candidate;
            result["coordination"] =
                json!({"tests":tests,"code_review":review,"peer_candidate":peer});
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
            if review["success"] == true {
                break;
            }
        }
        (merge_arguments, merge_pending) = loop {
            match guest
                .next_file::<ClosedFile, DEVELOPER_B>(&mut private, SERVICE_FD)
                .await?
            {
                FileBoundary::Exit(_) => {
                    return Err(ShellError::message(
                        "developer exited before its required request",
                    ));
                }
                FileBoundary::Closed { pending, file } => {
                    let requested = super::wasi_inputs::arguments::<2>(
                        &file,
                        b"service/merge_code",
                        ("branch", FEATURE_B),
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
    }
    let(_,exit)=async{
 async{
 loop{let branch=task.offer().await?;match branch.label(){
 InterfaceChangedA::LOGICAL_LABEL=>{peer=imp::decode(branch.recv::<InterfaceChangedA>().await?)?;imp::candidate(&peer,FEATURE_A)?;}
 ReviewApproved::LOGICAL_LABEL=>{let result=imp::decode(branch.recv::<ReviewApproved>().await?)?;imp::reviewed(&result,&peer,FEATURE_A)?;}
 ReviewRejected::LOGICAL_LABEL=>{let result=imp::decode(branch.recv::<ReviewRejected>().await?)?;imp::reviewed(&result,&peer,FEATURE_A)?;}
 RecoveryReviewApproved::LOGICAL_LABEL=>{let result=imp::decode(branch.recv::<RecoveryReviewApproved>().await?)?;imp::reviewed(&result,&peer,FEATURE_A)?;}
 RecoveryReviewRejected::LOGICAL_LABEL=>{let result=imp::decode(branch.recv::<RecoveryReviewRejected>().await?)?;imp::reviewed(&result,&peer,FEATURE_A)?;}
 CiFailedA::LOGICAL_LABEL=>{branch.recv::<CiFailedA>().await?;}
 LastCiPassedA::LOGICAL_LABEL=>{branch.recv::<LastCiPassedA>().await?;break;}
 _=>return Err(ShellError::message("expected the final peer CI result"))
 }}Ok::<_,ShellError>(())
 };
 async{loop{
 match guest.next_file::<ClosedFile,DEVELOPER_B>(&mut private,SERVICE_FD).await?{
 FileBoundary::Exit(exit)=>{let code=exit.status() as i32;private.send::<IoExit>(&code).await?;break Ok::<_,ShellError>(code);}
 FileBoundary::Closed{pending,..}=>{
 let bytes=imp::encode(&json!({"status":"out_of_order","error":"the assigned feature has passed its final CI"}))?;
 for(index,chunk)in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate(){
 private.send::<ResultChunkMsg>(&ResultChunk::new(index*protocol::WASIP1_IO_CHUNK_CAPACITY,chunk)?).await?;
 }
 guest=pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(protocol::FdClosed::new_with_errno(SERVICE_FD,8))))?;
 }
 }
}};
 }.await?;
    Ok(exit)
}
