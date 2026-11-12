//! The CI guest operates the actual BUILD_SERVER and delegated feature.
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
    endpoint: &mut Endpoint<CI_RUNNER>,
    actual_results: (&Cell<Value>, &Cell<Value>),
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    let mut build = task.recv::<BuildLoan>().await?;
    loop {
        let (publication,(mut arguments,mut pending,returned_private)) = async{
 async{imp::decode(task.recv::<Merged>().await?)};
 async move {
   let mut guest=guest;
   let mut private=private;
   let (arguments,pending)=loop {match guest.next_file::<ClosedFile,CI_RUNNER>(&mut private,SERVICE_FD).await?{
 FileBoundary::Exit(_)=>return Err(ShellError::message("CI runner exited before the final actual result")),
 FileBoundary::Closed{pending,file}=>{
 let requested=imp::decode(&file.bytes).and_then(|value|match value["suite"].as_str(){
 Some(SUITE_A)=>super::wasi_inputs::arguments::<5>(&file,b"service/run_tests",("suite",SUITE_A)),
 Some(SUITE_B)=>super::wasi_inputs::arguments::<5>(&file,b"service/run_tests",("suite",SUITE_B)),
 _=>Err(ShellError::message("unknown integrated test suite"))
 });
 match requested{Ok(arguments)=>break(arguments,pending),Err(error)=>{let bytes=imp::encode(&json!({"status":"invalid_arguments","error":error.to_string()}))?;
 for(index,chunk)in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate(){private.send::<ResultChunkMsg>(&ResultChunk::new(index*protocol::WASIP1_IO_CHUNK_CAPACITY,chunk)?).await?;}
 guest=pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(protocol::FdClosed::new_with_errno(SERVICE_FD,28))))?;}}
 }}};
   Ok::<_,ShellError>((arguments,pending,private))
  };
 }.await?;
        private = returned_private;
        let suite = match publication["data"]["feature"].as_str() {
            Some(FEATURE_A) => SUITE_A,
            Some(FEATURE_B) => SUITE_B,
            _ => return Err(ShellError::message("publication names an unknown feature")),
        };
        loop {
            let value = arguments.into_value();
            match imp::arguments(&value, ("suite", suite)) {
                Ok(()) => {
                    arguments = RepositoryArguments::from_bytes(&imp::encode(&value)?)?;
                    break;
                }
                Err(error) => {
                    let bytes = imp::encode(
                        &json!({"status":"invalid_arguments","error":error.to_string(),"publication":publication}),
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
                        protocol::FdClosedRet(protocol::FdClosed::new_with_errno(SERVICE_FD, 28)),
                    ))?;
                    (arguments, pending) = loop {
                        match guest
                            .next_file::<ClosedFile, CI_RUNNER>(&mut private, SERVICE_FD)
                            .await?
                        {
                            FileBoundary::Exit(_) => {
                                return Err(ShellError::message(
                                    "CI runner exited before the final actual result",
                                ));
                            }
                            FileBoundary::Closed { pending, file } => {
                                let requested =
                                    imp::decode(&file.bytes).and_then(|value| match value["suite"]
                                        .as_str()
                                    {
                                        Some(SUITE_A) => super::wasi_inputs::arguments::<5>(
                                            &file,
                                            b"service/run_tests",
                                            ("suite", SUITE_A),
                                        ),
                                        Some(SUITE_B) => super::wasi_inputs::arguments::<5>(
                                            &file,
                                            b"service/run_tests",
                                            ("suite", SUITE_B),
                                        ),
                                        _ => Err(ShellError::message(
                                            "unknown integrated test suite",
                                        )),
                                    });
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
                }
            }
        }
        if publication["data"]["feature"] == FEATURE_A {
            let mut feature = task.recv::<Feature<AUTH_A>>().await?;
            task.send::<CiIntentA>(&arguments).await?;
            async {
                async {
                    feature.send::<RunCiOnFeatureA>(&arguments).await?;
                };
                async {
                    build.send::<RunCiA>(&arguments).await?;
                };
                Ok::<_, ShellError>(())
            }
            .await?;
            let result = imp::decode(feature.recv::<CiResult>().await?)?;
            imp::same(&result, &imp::decode(build.recv::<CiResult>().await?)?)?;
            imp::ci_publication(&result, &publication)?;
            actual_results.0.set(result.clone());
            let bytes = imp::encode(&result)?;
            if result["success"] == true {
                let response = ReviewReply::<true>::from_bytes(&bytes)?;
                feature.send::<IntegratedA>(&response).await?;
                task.send::<Feature<AUTH_A>>(feature).await?;
                let branch = task.offer().await?;
                match branch.label() {
                    CiPassedA::LOGICAL_LABEL => {
                        imp::same(&imp::decode(branch.recv::<CiPassedA>().await?)?, &result)?;
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
                    LastCiPassedA::LOGICAL_LABEL => {
                        imp::same(
                            &imp::decode(branch.recv::<LastCiPassedA>().await?)?,
                            &result,
                        )?;
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
                        task.send::<BuildReturn>(build).await?;
                        loop {
                            match guest
                                .next_file::<ClosedFile, CI_RUNNER>(&mut private, SERVICE_FD)
                                .await?
                            {
                                FileBoundary::Exit(exit) => {
                                    let code = exit.status() as i32;
                                    private.send::<IoExit>(&code).await?;
                                    return Ok(code);
                                }
                                FileBoundary::Closed { pending, .. } => {
                                    let bytes = imp::encode(
                                        &json!({"status":"out_of_order","error":"the final integrated CI has completed"}),
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
                    _ => {
                        return Err(ShellError::message(
                            "expected this publication's successful CI feedback",
                        ));
                    }
                }
            } else {
                let response = ReviewReply::<false>::from_bytes(&bytes)?;
                feature.send::<IntegrationFailedA>(&response).await?;
                feature.send::<CiFailedA>(&response).await?;
                feature.send::<CiFailedA>(&response).await?;
                feature.send::<CiFailedA>(&response).await?;
                feature.send::<CiFailedA>(&response).await?;
                feature.send::<CiFailedA>(&response).await?;
                feature.send::<CiFailedA>(&response).await?;
                feature.send::<CiFailedA>(&response).await?;
                feature.send::<CiFailedA>(&response).await?;
                imp::same(&imp::decode(task.recv::<CiFailedA>().await?)?, &result)?;
                task.send::<Feature<AUTH_A>>(feature).await?;
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
            }
        } else {
            let mut feature = task.recv::<Feature<AUTH_B>>().await?;
            task.send::<CiIntentB>(&arguments).await?;
            async {
                async {
                    feature.send::<RunCiOnFeatureB>(&arguments).await?;
                };
                async {
                    build.send::<RunCiB>(&arguments).await?;
                };
                Ok::<_, ShellError>(())
            }
            .await?;
            let result = imp::decode(feature.recv::<CiResult>().await?)?;
            imp::same(&result, &imp::decode(build.recv::<CiResult>().await?)?)?;
            imp::ci_publication(&result, &publication)?;
            actual_results.1.set(result.clone());
            let bytes = imp::encode(&result)?;
            if result["success"] == true {
                let response = ReviewReply::<true>::from_bytes(&bytes)?;
                feature.send::<IntegratedB>(&response).await?;
                task.send::<Feature<AUTH_B>>(feature).await?;
                let branch = task.offer().await?;
                match branch.label() {
                    CiPassedB::LOGICAL_LABEL => {
                        imp::same(&imp::decode(branch.recv::<CiPassedB>().await?)?, &result)?;
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
                    LastCiPassedB::LOGICAL_LABEL => {
                        imp::same(
                            &imp::decode(branch.recv::<LastCiPassedB>().await?)?,
                            &result,
                        )?;
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
                        task.send::<BuildReturn>(build).await?;
                        loop {
                            match guest
                                .next_file::<ClosedFile, CI_RUNNER>(&mut private, SERVICE_FD)
                                .await?
                            {
                                FileBoundary::Exit(exit) => {
                                    let code = exit.status() as i32;
                                    private.send::<IoExit>(&code).await?;
                                    return Ok(code);
                                }
                                FileBoundary::Closed { pending, .. } => {
                                    let bytes = imp::encode(
                                        &json!({"status":"out_of_order","error":"the final integrated CI has completed"}),
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
                    _ => {
                        return Err(ShellError::message(
                            "expected this publication's successful CI feedback",
                        ));
                    }
                }
            } else {
                let response = ReviewReply::<false>::from_bytes(&bytes)?;
                feature.send::<IntegrationFailedB>(&response).await?;
                feature.send::<CiFailedB>(&response).await?;
                feature.send::<CiFailedB>(&response).await?;
                feature.send::<CiFailedB>(&response).await?;
                feature.send::<CiFailedB>(&response).await?;
                feature.send::<CiFailedB>(&response).await?;
                feature.send::<CiFailedB>(&response).await?;
                feature.send::<CiFailedB>(&response).await?;
                feature.send::<CiFailedB>(&response).await?;
                imp::same(&imp::decode(task.recv::<CiFailedB>().await?)?, &result)?;
                task.send::<Feature<AUTH_B>>(feature).await?;
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
            }
        }
    }
}
