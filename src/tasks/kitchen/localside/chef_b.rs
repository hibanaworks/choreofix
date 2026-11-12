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
pub async fn run(guest: &mut HibanaWasiGuest, endpoint: &mut Endpoint<B>) -> ShellResult<i32> {
    let [mut business, mut private] = endpoint.par()?;
    let ((pending, arguments), ingredient) = {
        let [mut intent, mut work, mut own, mut received] = business.par()?;
        let private_io = &mut private;
        async {
            async move {
                let mut guest = guest;
                // The two distinct one-shot requests are independently enabled.
                // Repeats are rejected by Endpoint; there is no local done table.
                for _ in [();2] {
                    loop {
                        let FileBoundary::Closed {pending,file} = guest.next_file::<ClosedFile,B>(private_io,SERVICE_FD).await?
                            else { return Err(ShellError::message("chef exited before completing its preparation")); };
                        let reply: ShellResult<Value> = async {
                            match file.path.as_slice() {
                                b"service/prepare_base_dish" => {
                                    let arguments = match imp::Arguments::<1>::from_close(&file,b"service/prepare_base_dish") {
                                        Ok(arguments)=>arguments,
                                        Err(error)=>return Ok(json!({"status":"invalid_arguments","error":error.to_string()})),
                                    };
                                    if let Err(error)=intent.send::<BaseIntentB>(&arguments).await {
                                        if error.is_unavailable() { return Ok(json!({"status":"out_of_order","error":"the base request has completed"})); }
                                        return Err(error.into());
                                    }
                                    drop(work.offer().await?);
                                    let (mut oven,mut stove)=async {
                                        let [mut o,mut s]=work.par()?;
                                        async {o.recv::<OvenLoan>().await};
                                        async {s.recv::<StoveLoan>().await};
                                    }.await?;
                                    async {
                                        async {oven.send::<OvenRequestB>(&arguments).await?;};
                                        async {stove.send::<StoveRequestB>(&arguments).await?;};
                                        Ok::<(),ShellError>(())
                                    }.await?;
                                    let (left,right)=async {
                                        async {imp::decode(oven.recv::<OvenResult>().await?)};
                                        async {imp::decode(stove.recv::<StoveResult>().await?)};
                                    }.await?;
                                    imp::completed("prepare_base_dish","CHEF_B",&arguments.value(),&left)?;
                                    if left != right {return Err(ShellError::message("equipment results differ for the same actual dish"));}
                                    async {
                                        let [mut o,mut s]=work.par()?;
                                        async {o.send::<ReturnOven>(oven).await?;};
                                        async {s.send::<ReturnStove>(stove).await?;};
                                        Ok::<(),ShellError>(())
                                    }.await?;
                                    Ok(left)
                                }
                                b"service/prepare_ingredient" => {
                                    let arguments=match imp::Arguments::<4>::from_close(&file,b"service/prepare_ingredient") {
                                        Ok(arguments)=>arguments,
                                        Err(error)=>return Ok(json!({"status":"invalid_arguments","error":error.to_string()})),
                                    };
                                    if let Err(error)=own.send::<PrepareGarnish>(&arguments).await {
                                        if error.is_unavailable() {return Ok(json!({"status":"out_of_order","error":"the ingredient request has completed"}));}
                                        return Err(error.into());
                                    }
                                    let result=imp::decode(own.recv::<GarnishPrepared>().await?)?;
                                    imp::completed("prepare_ingredient","CHEF_B",&arguments.value(),&result)?;
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
                    let FileBoundary::Closed { pending, file } = guest.next_file::<ClosedFile, B>(private_io, SERVICE_FD).await?
                        else { return Err(ShellError::message("chef exited before combining its dish")); };
                    let (reply, errno) = if file.path == b"service/combine_dish" {
                        match imp::Arguments::<7>::from_close(&file, b"service/combine_dish") {
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
                let result=imp::decode(received.recv::<Glaze>().await?)?;
                imp::completed("prepare_ingredient","CHEF_C",&imp::expected(5).expect("known ingredient"),&result)?;
                Ok::<_,ShellError>(result)
            };
        }.await?
    };
    if arguments.value()["ingredient"] != ingredient["data"]["ingredient"] {
        return Err(ShellError::message(
            "combination differs from the actual delivered ingredient",
        ));
    }
    business.send::<CombineB>(&arguments).await?;
    let result = imp::decode(business.recv::<DishB>().await?)?;
    imp::completed("combine_dish", "CHEF_B", &arguments.value(), &result)?;
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
            .next_file::<ClosedFile, B>(&mut private, SERVICE_FD)
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
