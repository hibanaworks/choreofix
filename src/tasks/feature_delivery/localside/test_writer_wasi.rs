//! Test definitions use the real TEST_ENV and actual source/result messages.
use super::super::{
    global::{delivery::*, *},
    imp,
};
use crate::shell::{
    ShellError, ShellResult,
    global::Exit as IoExit,
    imp::file_wire::{ClosedFile, ResultChunk, ResultChunkMsg, SERVICE_FD},
};
use hibana::{Endpoint, g::Message};
use hibana_wasip1_runtime::{FileBoundary, HibanaWasiGuest, WasiImportCompletion, protocol};
use serde_json::json;

#[hibana_runtime::scope]
pub async fn run(
    mut guest: &mut HibanaWasiGuest,
    endpoint: &mut Endpoint<TEST_WRITER>,
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    let (mut environment,(arguments,pending,returned_private)) = async{
 async{
 let environment=task.recv::<TestLoan>().await?;
 let branch=task.offer().await?;match branch.label(){
 ReviewFirstA::LOGICAL_LABEL=>{branch.recv::<ReviewFirstA>().await?;}
 ReviewFirstB::LOGICAL_LABEL=>{branch.recv::<ReviewFirstB>().await?;}
 _=>return Err(ShellError::message("expected the actual review order"))}
 Ok::<_,ShellError>(environment)
 };
 async move {
   let mut guest=guest;
   let mut private=private;
   let (arguments,pending)=loop {match guest.next_file::<ClosedFile,TEST_WRITER>(&mut private,SERVICE_FD).await?{
 FileBoundary::Exit(_)=>return Err(ShellError::message("test writer exited before final CI")),
 FileBoundary::Closed{pending,file}=>{
 let requested=imp::decode(&file.bytes).and_then(|value|match value["target"].as_str(){
 Some(FEATURE_A)=>super::wasi_inputs::arguments::<4>(&file,b"service/write_tests",("target",FEATURE_A)),
 Some(FEATURE_B)=>super::wasi_inputs::arguments::<4>(&file,b"service/write_tests",("target",FEATURE_B)),
 _=>Err(ShellError::message("tests name neither assigned feature"))
 });
 match requested{
 Ok(arguments)=>break(arguments,pending),
 Err(error)=>{let bytes=imp::encode(&json!({"status":"invalid_arguments","error":error.to_string()}))?;
 for(index,chunk)in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate(){
 private.send::<ResultChunkMsg>(&ResultChunk::new(index*protocol::WASIP1_IO_CHUNK_CAPACITY,chunk)?).await?;}
 guest=pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(protocol::FdClosed::new_with_errno(SERVICE_FD,28))))?;}
 }}
 }};
   Ok::<_,ShellError>((arguments,pending,private))
  };
 }.await?;
    private = returned_private;
    let selected = arguments.into_value();
    let feature = if selected["target"] == FEATURE_A {
        FEATURE_A
    } else {
        FEATURE_B
    };
    let arguments = crate::shell::imp::repository_arguments::RepositoryArguments::from_bytes(
        &imp::encode(&selected)?,
    )?;
    let result = if feature == FEATURE_A {
        task.send::<TestIntentA>(&arguments).await?;
        let candidate = imp::decode(task.recv::<CandidateForTestsA>().await?)?;
        imp::candidate(&candidate, FEATURE_A)?;
        environment.send::<TestRequestA>(&arguments).await?;
        let result = imp::decode(environment.recv::<TestResultA>().await?)?;
        imp::tested(&result, &candidate, FEATURE_A)?;
        let bytes = imp::encode(&result)?;
        task.send::<TestsForCiA>(&bytes.as_slice()).await?;
        result
    } else {
        task.send::<TestIntentB>(&arguments).await?;
        let candidate = imp::decode(task.recv::<CandidateForTestsB>().await?)?;
        imp::candidate(&candidate, FEATURE_B)?;
        environment.send::<TestRequestB>(&arguments).await?;
        let result = imp::decode(environment.recv::<TestResultB>().await?)?;
        imp::tested(&result, &candidate, FEATURE_B)?;
        let bytes = imp::encode(&result)?;
        task.send::<TestsForCiB>(&bytes.as_slice()).await?;
        result
    };
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
    let feature = if feature == FEATURE_A {
        FEATURE_B
    } else {
        FEATURE_A
    };
    let (arguments, mut pending) = loop {
        match guest
            .next_file::<ClosedFile, TEST_WRITER>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Exit(_) => {
                return Err(ShellError::message("test writer exited before final CI"));
            }
            FileBoundary::Closed { pending, file } => {
                let requested = super::wasi_inputs::arguments::<4>(
                    &file,
                    b"service/write_tests",
                    ("target", feature),
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
    let mut prepared = if feature == FEATURE_A {
        task.send::<TestIntentA>(&arguments).await?;
        let candidate = imp::decode(task.recv::<CandidateForTestsA>().await?)?;
        imp::candidate(&candidate, FEATURE_A)?;
        environment.send::<TestRequestA>(&arguments).await?;
        let result = imp::decode(environment.recv::<TestResultA>().await?)?;
        imp::tested(&result, &candidate, FEATURE_A)?;
        let bytes = imp::encode(&result)?;
        task.send::<TestsForCiA>(&bytes.as_slice()).await?;
        result
    } else {
        task.send::<TestIntentB>(&arguments).await?;
        let candidate = imp::decode(task.recv::<CandidateForTestsB>().await?)?;
        imp::candidate(&candidate, FEATURE_B)?;
        environment.send::<TestRequestB>(&arguments).await?;
        let result = imp::decode(environment.recv::<TestResultB>().await?)?;
        imp::tested(&result, &candidate, FEATURE_B)?;
        let bytes = imp::encode(&result)?;
        task.send::<TestsForCiB>(&bytes.as_slice()).await?;
        result
    };
    let mut outcomes = Vec::new();
    loop {
        let continuation = loop {
            let branch = task.offer().await?;
            match branch.label() {
                ReviewApproved::LOGICAL_LABEL => {
                    outcomes.push(imp::decode(branch.recv::<ReviewApproved>().await?)?);
                }
                RecoveryReviewApproved::LOGICAL_LABEL => {
                    outcomes.push(imp::decode(branch.recv::<RecoveryReviewApproved>().await?)?);
                }
                ReviewRejected::LOGICAL_LABEL => {
                    break imp::decode(branch.recv::<ReviewRejected>().await?)?;
                }
                RecoveryReviewRejected::LOGICAL_LABEL => {
                    break imp::decode(branch.recv::<RecoveryReviewRejected>().await?)?;
                }
                CiFailedA::LOGICAL_LABEL => break imp::decode(branch.recv::<CiFailedA>().await?)?,
                CiFailedB::LOGICAL_LABEL => break imp::decode(branch.recv::<CiFailedB>().await?)?,
                CiPassedA::LOGICAL_LABEL => {
                    outcomes.push(imp::decode(branch.recv::<CiPassedA>().await?)?);
                }
                CiPassedB::LOGICAL_LABEL => {
                    outcomes.push(imp::decode(branch.recv::<CiPassedB>().await?)?);
                }
                LastCiPassedA::LOGICAL_LABEL => {
                    let completed = imp::decode(branch.recv::<LastCiPassedA>().await?)?;
                    let mut result = prepared;
                    result["coordination"] = json!({"outcomes":outcomes,"ci":completed});
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
                        protocol::FdClosedRet(protocol::FdClosed::new_with_errno(SERVICE_FD, 0)),
                    ))?;
                    task.send::<TestReturn>(environment).await?;
                    loop {
                        match guest
                            .next_file::<ClosedFile, TEST_WRITER>(&mut private, SERVICE_FD)
                            .await?
                        {
                            FileBoundary::Exit(exit) => {
                                let code = exit.status() as i32;
                                private.send::<IoExit>(&code).await?;
                                return Ok(code);
                            }
                            FileBoundary::Closed { pending, .. } => {
                                let bytes = imp::encode(
                                    &json!({"status":"out_of_order","error":"both features have passed CI"}),
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
                    let mut result = prepared;
                    result["coordination"] = json!({"outcomes":outcomes,"ci":completed});
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
                        protocol::FdClosedRet(protocol::FdClosed::new_with_errno(SERVICE_FD, 0)),
                    ))?;
                    task.send::<TestReturn>(environment).await?;
                    loop {
                        match guest
                            .next_file::<ClosedFile, TEST_WRITER>(&mut private, SERVICE_FD)
                            .await?
                        {
                            FileBoundary::Exit(exit) => {
                                let code = exit.status() as i32;
                                private.send::<IoExit>(&code).await?;
                                return Ok(code);
                            }
                            FileBoundary::Closed { pending, .. } => {
                                let bytes = imp::encode(
                                    &json!({"status":"out_of_order","error":"both features have passed CI"}),
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
                        "expected an actual review or CI outcome",
                    ));
                }
            }
        };
        let feature = match continuation["data"]["target"]
            .as_str()
            .or_else(|| continuation["data"]["feature"].as_str())
        {
            Some(FEATURE_A) => FEATURE_A,
            Some(FEATURE_B) => FEATURE_B,
            _ => return Err(ShellError::message("retry result names another feature")),
        };
        outcomes.push(continuation);
        let mut result = prepared;
        result["coordination"] = json!({"outcomes":outcomes});
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
        let (arguments, next) = loop {
            match guest
                .next_file::<ClosedFile, TEST_WRITER>(&mut private, SERVICE_FD)
                .await?
            {
                FileBoundary::Exit(_) => {
                    return Err(ShellError::message("test writer exited before final CI"));
                }
                FileBoundary::Closed { pending, file } => {
                    let requested = super::wasi_inputs::arguments::<4>(
                        &file,
                        b"service/write_tests",
                        ("target", feature),
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
        pending = next;
        prepared = if feature == FEATURE_A {
            task.send::<TestIntentA>(&arguments).await?;
            let candidate = imp::decode(task.recv::<CandidateForTestsA>().await?)?;
            imp::candidate(&candidate, FEATURE_A)?;
            environment.send::<TestRequestA>(&arguments).await?;
            let result = imp::decode(environment.recv::<TestResultA>().await?)?;
            imp::tested(&result, &candidate, FEATURE_A)?;
            let bytes = imp::encode(&result)?;
            task.send::<TestsForCiA>(&bytes.as_slice()).await?;
            result
        } else {
            task.send::<TestIntentB>(&arguments).await?;
            let candidate = imp::decode(task.recv::<CandidateForTestsB>().await?)?;
            imp::candidate(&candidate, FEATURE_B)?;
            environment.send::<TestRequestB>(&arguments).await?;
            let result = imp::decode(environment.recv::<TestResultB>().await?)?;
            imp::tested(&result, &candidate, FEATURE_B)?;
            let bytes = imp::encode(&result)?;
            task.send::<TestsForCiB>(&bytes.as_slice()).await?;
            result
        };
        outcomes.clear();
    }
}
