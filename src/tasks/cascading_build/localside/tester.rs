//! The model's test arguments must name the actual object received from its builder.
use super::super::{global::*, imp};
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
    endpoint: &mut Endpoint<TESTER>,
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    let branch = task.offer().await?;
    let mut report = match branch.label() {
        TestObjectA::LOGICAL_LABEL => imp::decode(branch.recv::<TestObjectA>().await?)?,
        TestObjectB::LOGICAL_LABEL => imp::decode(branch.recv::<TestObjectB>().await?)?,
        TestObjectC::LOGICAL_LABEL => imp::decode(branch.recv::<TestObjectC>().await?)?,
        _ => {
            return Err(ShellError::message(
                "missing the first actual build to test",
            ));
        }
    };
    loop {
        let index = imp::module(&report["data"]["module"])?;
        let object = imp::object(&report, index)?;
        let (arguments, pending) = loop {
            match guest
                .next_file::<ClosedFile, TESTER>(&mut private, SERVICE_FD)
                .await?
            {
                FileBoundary::Exit(_) => {
                    return Err(ShellError::message(
                        "tester exited before the linked result",
                    ));
                }
                FileBoundary::Closed { pending, file } => {
                    let requested = file
                        .arguments(b"service/run_build_tests", "module_name")
                        .and_then(|value| imp::arguments(value, index));
                    match requested {
                        Ok(arguments) => break (arguments, pending),
                        Err(error) => {
                            let bytes = imp::encode(
                                &json!({"status":"invalid_arguments","error":error.to_string(),
                                "actual_build":report}),
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
        let bytes = imp::encode(&json!({"arguments":arguments,"object":object}))?;
        match index {
            0 => task.send::<TestA>(&bytes.as_slice()).await?,
            1 => task.send::<TestB>(&bytes.as_slice()).await?,
            _ => task.send::<TestC>(&bytes.as_slice()).await?,
        }
        let mut result = imp::decode(task.recv::<Tested>().await?)?;
        imp::test_result(&result, index)?;
        if result["data"]["input_object"] != object {
            return Err(ShellError::message(
                "actual tested object differs from the received build",
            ));
        }
        // The next actual build (or final linked result) teaches the model what
        // data exists next. No done map or expected-operation cursor is added.
        let branch = task.offer().await?;
        let next = match branch.label() {
            TestObjectA::LOGICAL_LABEL => imp::decode(branch.recv::<TestObjectA>().await?)?,
            TestObjectB::LOGICAL_LABEL => imp::decode(branch.recv::<TestObjectB>().await?)?,
            TestObjectC::LOGICAL_LABEL => imp::decode(branch.recv::<TestObjectC>().await?)?,
            FinishedTester::LOGICAL_LABEL => imp::decode(branch.recv::<FinishedTester>().await?)?,
            _ => return Err(ShellError::message("unexpected actual test continuation")),
        };
        result["coordination"] = json!({"next":next});
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
        if next["tool_name"] == "link_modules" {
            if next["success"] != true || next["data"]["status"] != "linked" {
                return Err(ShellError::message(
                    "tester received no successful linked result",
                ));
            }
            break;
        }
        report = next;
    }
    drop(task);
    loop {
        match guest
            .next_file::<ClosedFile, TESTER>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Exit(exit) => {
                let status = exit.status() as i32;
                private.send::<IoExit>(&status).await?;
                return Ok(status);
            }
            FileBoundary::Closed { pending, .. } => {
                let bytes = imp::encode(
                    &json!({"status":"out_of_order","error":"the actual linked result has closed testing"}),
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
