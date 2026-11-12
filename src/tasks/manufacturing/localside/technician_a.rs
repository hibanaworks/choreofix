//! The wash guest chooses either actual job; affine request Endpoints retain retries.
use super::super::{global::*, imp};
use crate::shell::{
    ShellError, ShellResult,
    global::Exit as IoExit,
    imp::file_wire::{ClosedFile, ResultChunk, ResultChunkMsg, SERVICE_FD},
};
use hibana::{Endpoint, g::Message};
use hibana_wasip1_runtime::{FileBoundary, HibanaWasiGuest, WasiImportCompletion, protocol};
use serde_json::{Value, json};

#[hibana_runtime::scope]
pub async fn run(
    mut guest: &mut HibanaWasiGuest,
    endpoint: &mut Endpoint<TECHNICIAN_A>,
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    let mut lathe_request = None;
    let mut mill_request = None;
    {
        let [
            mut lathe_intent,
            mut lathe_work,
            mut mill_intent,
            mut mill_work,
        ] = task.par()?;
        // Two actual completed jobs, with arbitrary invalid inputs and busy retries between them.
        for _ in [(); 2] {
            loop {
                let FileBoundary::Closed { pending, file } = guest
                    .next_file::<ClosedFile, TECHNICIAN_A>(&mut private, SERVICE_FD)
                    .await?
                else {
                    return Err(ShellError::message(
                        "technician exited before completing both jobs",
                    ));
                };
                let reply: ShellResult<Value> = async {
                let arguments = match file.arguments(b"service/do_job", "job_id") {
                    Ok(arguments) => arguments,
                    Err(error) => return Ok(json!({"status":"invalid_arguments","error":error.to_string()})),
                };
                match arguments["job_id"].as_str().unwrap() {
"lathe_job_a" => {
    let arguments = imp::JobArguments::<0>::from_bytes(&file.bytes)?;
    let mut request = if let Some(request) = lathe_request.take() {
        request
    } else {
        if let Err(error) = lathe_intent.send::<LatheIntentA>(&arguments).await {
            if error.is_unavailable() {
                return Ok(json!({"status":"out_of_order","error":"the assigned job request has completed"}));
            }
            return Err(error.into());
        }
        lathe_work.recv::<LatheRequestLoanA>().await?
    };
    let branch = request.offer().await?;
    match branch.label() {
        LatheBusy::LOGICAL_LABEL => {
            branch.recv::<LatheBusy>().await?;
            lathe_request = Some(request);
            return Ok(json!({"status":"busy","job_id":"lathe_job_a","machine":"LATHE"}));
        }
        LatheRetryA::LOGICAL_LABEL => {
            branch.send::<LatheRetryA>(&arguments).await?;
            // A queued Ready is retained while this in-flight retry is answered.
            request.recv::<LatheBusy>().await?;
            lathe_request = Some(request);
            return Ok(json!({"status":"busy","job_id":"lathe_job_a","machine":"LATHE"}));
        }
        LatheReady::LOGICAL_LABEL => { branch.recv::<LatheReady>().await?; }
        _ => return Err(ShellError::message("unexpected lathe availability")),
    }
    lathe_work.send::<ReturnLatheRequestA>(request).await?;
    let mut resource = lathe_work.recv::<LatheLoan>().await?;
    resource.send::<LatheJobA>(&arguments).await?;
    let result = imp::decode(resource.recv::<LatheResult>().await?)?;
    imp::completed(&result, "TECHNICIAN_A", "LATHE", &arguments.into_value())?;
    let bytes = imp::encode(&result)?;
    async {
        let [mut returned, mut report] = lathe_work.par()?;
        async { returned.send::<ReturnLathe>(resource).await?; };
        async { report.send::<LatheDoneA>(&bytes.as_slice()).await?; };
        Ok::<(), ShellError>(())
    }.await?;
    Ok(result)
}
"mill_job_a" => {
    let arguments = imp::JobArguments::<2>::from_bytes(&file.bytes)?;
    let mut request = if let Some(request) = mill_request.take() {
        request
    } else {
        if let Err(error) = mill_intent.send::<MillIntentA>(&arguments).await {
            if error.is_unavailable() {
                return Ok(json!({"status":"out_of_order","error":"the assigned job request has completed"}));
            }
            return Err(error.into());
        }
        mill_work.recv::<MillRequestLoanA>().await?
    };
    let branch = request.offer().await?;
    match branch.label() {
        MillBusy::LOGICAL_LABEL => {
            branch.recv::<MillBusy>().await?;
            mill_request = Some(request);
            return Ok(json!({"status":"busy","job_id":"mill_job_a","machine":"MILL"}));
        }
        MillRetryA::LOGICAL_LABEL => {
            branch.send::<MillRetryA>(&arguments).await?;
            // A queued Ready is retained while this in-flight retry is answered.
            request.recv::<MillBusy>().await?;
            mill_request = Some(request);
            return Ok(json!({"status":"busy","job_id":"mill_job_a","machine":"MILL"}));
        }
        MillReady::LOGICAL_LABEL => { branch.recv::<MillReady>().await?; }
        _ => return Err(ShellError::message("unexpected mill availability")),
    }
    mill_work.send::<ReturnMillRequestA>(request).await?;
    let mut resource = mill_work.recv::<MillLoan>().await?;
    resource.send::<MillJobA>(&arguments).await?;
    let result = imp::decode(resource.recv::<MillResult>().await?)?;
    imp::completed(&result, "TECHNICIAN_A", "MILL", &arguments.into_value())?;
    let bytes = imp::encode(&result)?;
    async {
        let [mut returned, mut report] = mill_work.par()?;
        async { returned.send::<ReturnMill>(resource).await?; };
        async { report.send::<MillDoneA>(&bytes.as_slice()).await?; };
        Ok::<(), ShellError>(())
    }.await?;
    Ok(result)
}

                    _ => Ok(json!({"status":"invalid_arguments","error":"the job belongs to another technician or machine"})),
                }
            }.await;
                let reply = reply?;
                let bytes = imp::encode(&reply)?;
                for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
                    private
                        .send::<ResultChunkMsg>(&ResultChunk::new(
                            index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                            chunk,
                        )?)
                        .await?;
                }
                let errno = match reply["status"].as_str() {
                    Some("busy") => 6,
                    Some("out_of_order") => 8,
                    Some("invalid_arguments") => 28,
                    _ => 0,
                };
                guest = pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
                    protocol::FdClosed::new_with_errno(SERVICE_FD, errno),
                )))?;
                if reply["tool_name"] == "do_job" {
                    break;
                }
            }
        }
    }
    drop(task);
    loop {
        match guest
            .next_file::<ClosedFile, TECHNICIAN_A>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Exit(exit) => {
                let status = exit.status() as i32;
                private.send::<IoExit>(&status).await?;
                return Ok(status);
            }
            FileBoundary::Closed { pending, .. } => {
                let bytes = imp::encode(
                    &json!({"status":"out_of_order","error":"both assigned jobs have completed"}),
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
