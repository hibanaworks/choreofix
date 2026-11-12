//! This chef chooses its own shell commands; Endpoint admits actual effects.
use super::super::{global::*, imp};
use crate::shell::{
    ShellError, ShellResult,
    global::Exit,
    imp::file_wire::{ClosedFile, ResultChunk, ResultChunkMsg, SERVICE_FD},
};
use hibana::Endpoint;
use hibana_wasip1_runtime::{FileBoundary, HibanaWasiGuest, WasiImportCompletion, protocol};
use serde_json::{Value, json};

#[hibana_runtime::scope]
pub async fn run(guest: &mut HibanaWasiGuest, endpoint: &mut Endpoint<C>) -> ShellResult<i32> {
    let [mut business, mut private] = endpoint.par()?;
    let ((pending, arguments), ingredient) = {
        let [mut intent, mut work, mut received, mut own] = business.par()?;
        let private_io = &mut private;
        async {
            async move {
                let mut guest = guest;
                // The two distinct one-shot requests are independently enabled.
                // Repeats are rejected by Endpoint; there is no local done table.
                for _ in [();2] {
                    loop {
                        let FileBoundary::Closed {pending,file} = guest.next_file::<ClosedFile,C>(private_io,SERVICE_FD).await?
                            else { return Err(ShellError::message("chef exited before completing its preparation")); };
                        let reply: ShellResult<Value> = async {
                            match file.path.as_slice() {
                                b"service/prepare_base_dish" => {
                                    let arguments = match imp::Arguments::<2>::from_close(&file,b"service/prepare_base_dish") {
                                        Ok(arguments)=>arguments,
                                        Err(error)=>return Ok(json!({"status":"invalid_arguments","error":error.to_string()})),
                                    };
                                    if let Err(error)=intent.send::<BaseIntentC>(&arguments).await {
                                        if error.is_unavailable() { return Ok(json!({"status":"out_of_order","error":"the base request has completed"})); }
                                        return Err(error.into());
                                    }
                                    let mut oven=work.recv::<OvenLoan>().await?;
                                    oven.send::<OvenRequestC>(&arguments).await?;
                                    let result=imp::decode(oven.recv::<OvenResult>().await?)?;
                                    imp::completed("prepare_base_dish","CHEF_C",&arguments.value(),&result)?;
                                    work.send::<ReturnOven>(oven).await?;
                                    Ok(result)
                                }
                                b"service/prepare_ingredient" => {
                                    let arguments=match imp::Arguments::<5>::from_close(&file,b"service/prepare_ingredient") {
                                        Ok(arguments)=>arguments,
                                        Err(error)=>return Ok(json!({"status":"invalid_arguments","error":error.to_string()})),
                                    };
                                    if let Err(error)=own.send::<PrepareGlaze>(&arguments).await {
                                        if error.is_unavailable() {return Ok(json!({"status":"out_of_order","error":"the ingredient request has completed"}));}
                                        return Err(error.into());
                                    }
                                    let result=imp::decode(own.recv::<GlazePrepared>().await?)?;
                                    imp::completed("prepare_ingredient","CHEF_C",&arguments.value(),&result)?;
                                    Ok(result)
                                }
                                _=>Ok(json!({"status":"out_of_order","error":"complete the assigned base and ingredient before combining"})),
                            }
                        }.await;
                        let reply=reply?;
                        let errno=match reply["status"].as_str() {Some("invalid_arguments")=>28,Some("out_of_order")=>8,_=>0};
                        let bytes=imp::encode(&reply)?;
                        for (index,chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
                            private_io.send::<ResultChunkMsg>(&ResultChunk::new(index*protocol::WASIP1_IO_CHUNK_CAPACITY,chunk)?).await?;
                        }
                        guest=pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
                            protocol::FdClosed::new_with_errno(SERVICE_FD,errno))))?;
                        if reply["success"]==true {break;}
                    }
                }
                loop {
                    let FileBoundary::Closed { pending, file } = guest.next_file::<ClosedFile, C>(private_io, SERVICE_FD).await?
                        else { return Err(ShellError::message("chef exited before combining its dish")); };
                    let (reply, errno) = if file.path == b"service/combine_dish" {
                        match imp::Arguments::<8>::from_close(&file, b"service/combine_dish") {
                            Ok(arguments) => break Ok::<_, ShellError>((pending, arguments)),
                            Err(error) => (json!({"status":"invalid_arguments","error":error.to_string()}), 28),
                        }
                    } else {
                        (json!({"status":"out_of_order","error":"both preparation requests have completed"}), 8)
                    };
                    let bytes = imp::encode(&reply)?;
                    for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
                        private_io.send::<ResultChunkMsg>(&ResultChunk::new(index * protocol::WASIP1_IO_CHUNK_CAPACITY, chunk)?).await?;
                    }
                    guest = pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
                        protocol::FdClosed::new_with_errno(SERVICE_FD, errno))))?;
                }
            };
            async {
                let result=imp::decode(received.recv::<Sauce>().await?)?;
                imp::completed("prepare_ingredient","CHEF_A",&imp::expected(3).expect("known ingredient"),&result)?;
                Ok::<_,ShellError>(result)
            };
        }.await?
    };
    if arguments.value()["ingredient"] != ingredient["data"]["ingredient"] {
        return Err(ShellError::message(
            "combination differs from the actual delivered ingredient",
        ));
    }
    business.send::<CombineC>(&arguments).await?;
    let result = imp::decode(business.recv::<DishC>().await?)?;
    imp::completed("combine_dish", "CHEF_C", &arguments.value(), &result)?;
    let bytes = imp::encode(&result)?;
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
    drop(business);
    loop {
        match guest
            .next_file::<ClosedFile, C>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Exit(exit) => {
                let status = exit.status() as i32;
                private.send::<Exit>(&status).await?;
                return Ok(status);
            }
            FileBoundary::Closed { pending, .. } => {
                let bytes = imp::encode(
                    &json!({"status":"out_of_order","error":"this chef has completed its dish"}),
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
