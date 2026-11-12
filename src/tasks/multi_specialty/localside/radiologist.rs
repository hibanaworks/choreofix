//! The radiologist chooses the request order; each real operation returns its resource.
use super::super::global::*;
use crate::imp::{json, patient};
use crate::shell::{
    ShellError, ShellResult,
    global::Exit as IoExit,
    imp::file_wire::{ClosedFile, ResultChunk, ResultChunkMsg, SERVICE_FD},
};
use hibana_wasip1_runtime::{FileBoundary, HibanaWasiGuest, WasiImportCompletion, protocol};
use serde_json::json;

#[hibana_runtime::scope]
pub async fn run(
    mut guest: &mut HibanaWasiGuest,
    endpoint: &mut hibana::Endpoint<RADIOLOGIST>,
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    let (pending,mut reply,data)=async{
  let [mut ct,mut mri]=task.par()?;
  // These are the actual image and interpretation receipts, used to check
  // data correspondence and to return the completed case to the guest.
  let mut data=json!({"images":{},"reports":{}});
  loop{
   match guest.next_file::<ClosedFile,RADIOLOGIST>(&mut private,SERVICE_FD).await?{
    FileBoundary::Exit(_)=>return Err(ShellError::message("radiologist exited before both actual reports")),
    FileBoundary::Closed{pending,file}=>{
     let arguments=match file.path.as_slice(){
      b"service/perform_imaging" => patient::file_arguments(&file,b"service/perform_imaging"),
      b"service/interpret_imaging" => patient::file_arguments(&file,b"service/interpret_imaging"),
      _=>Err(ShellError::message("expected an imaging or interpretation request")),
     }.and_then(|arguments|{
      if !arguments["modality"].as_str().is_some_and(|m|m.eq_ignore_ascii_case("CT")||m.eq_ignore_ascii_case("MRI")){
       return Err(ShellError::message("unknown imaging modality"));
      }Ok(arguments)
     });
     let (reply,errno)=match arguments{
      Err(error)=>(json!({"status":"invalid_arguments","error":error.to_string()}),28),
      Ok(arguments)=>{
       let modality=arguments["modality"].as_str().expect("validated modality").to_ascii_uppercase();
       match (file.path.as_slice(),modality.as_str()){
        (b"service/perform_imaging",_) if !data["images"][&modality].is_null()=>
          (json!({"status":"out_of_order","error":"this case already has that actual image"}),8),
        (b"service/interpret_imaging",_) if data["images"][&modality].is_null()=>
          (json!({"status":"invalid_arguments","error":"the actual image is absent"}),28),
        (b"service/interpret_imaging",_) if !data["reports"][&modality].is_null()=>
          (json!({"status":"out_of_order","error":"this case already has that actual interpretation"}),8),
        (b"service/perform_imaging","CT")=>{patient::actual_result(&json::decode(ct.recv::<CtOrder>().await?)?)?;
let bytes=json::encode(&arguments)?;
ct.send::<CtPerformIntent>(&bytes.as_slice()).await?;
let mut instrument=ct.recv::<CtLoan>().await?;
instrument.send::<CtPerformRequest>(&bytes.as_slice()).await?;
let result=json::decode(instrument.recv::<CtImageResult>().await?)?;
patient::actual_result(&result)?;
ct.send::<CtLoan>(instrument).await?;
data["images"][&modality]=result.clone();
(result,0)},
        (b"service/perform_imaging","MRI")=>{patient::actual_result(&json::decode(mri.recv::<MriOrder>().await?)?)?;
let bytes=json::encode(&arguments)?;
mri.send::<MriPerformIntent>(&bytes.as_slice()).await?;
let mut instrument=mri.recv::<MriLoan>().await?;
instrument.send::<MriPerformRequest>(&bytes.as_slice()).await?;
let result=json::decode(instrument.recv::<MriImageResult>().await?)?;
patient::actual_result(&result)?;
mri.send::<MriLoan>(instrument).await?;
data["images"][&modality]=result.clone();
(result,0)},
        (b"service/interpret_imaging","CT")=>{let bytes=json::encode(&arguments)?;
ct.send::<CtInterpretIntent>(&bytes.as_slice()).await?;
let mut instrument=ct.recv::<CtLoan>().await?;
instrument.send::<CtInterpretRequest>(&bytes.as_slice()).await?;
let result=json::decode(instrument.recv::<CtReportResult>().await?)?;
patient::actual_result(&result)?;
json::same(&result["data"]["inputs"]["image"],&data["images"][&modality]["data"]["artifact_id"])?;
ct.send::<CtLoan>(instrument).await?;
data["reports"][&modality]=result.clone();
(result,0)},
        (b"service/interpret_imaging","MRI")=>{let bytes=json::encode(&arguments)?;
mri.send::<MriInterpretIntent>(&bytes.as_slice()).await?;
let mut instrument=mri.recv::<MriLoan>().await?;
instrument.send::<MriInterpretRequest>(&bytes.as_slice()).await?;
let result=json::decode(instrument.recv::<MriReportResult>().await?)?;
patient::actual_result(&result)?;
json::same(&result["data"]["inputs"]["image"],&data["images"][&modality]["data"]["artifact_id"])?;
mri.send::<MriLoan>(instrument).await?;
data["reports"][&modality]=result.clone();
(result,0)},
        _=>return Err(ShellError::message("validated imaging request has another operation")),
       }
      }
     };
     if !data["reports"]["CT"].is_null() && !data["reports"]["MRI"].is_null(){
      break Ok::<_,ShellError>((pending,reply,data));
     }
     let bytes=json::encode(&reply)?;
for(index,chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate(){
 private.send::<ResultChunkMsg>(&ResultChunk::new(index*protocol::WASIP1_IO_CHUNK_CAPACITY,chunk)?).await?;
}
guest=pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
 protocol::FdClosed::new_with_errno(SERVICE_FD,errno),
)))?;
    }
   }
  }
 }.await?;
    let final_case = json::decode(task.recv::<FinalForRad>().await?)?;
    json::same(&final_case["case"]["CT"], &data["reports"]["CT"])?;
    json::same(&final_case["case"]["MRI"], &data["reports"]["MRI"])?;
    patient::actual_result(&final_case["conference"])?;
    reply["consultation"] = final_case;
    let bytes = json::encode(&reply)?;
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
            .next_file::<ClosedFile, RADIOLOGIST>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Exit(exit) => {
                let code = exit.status() as i32;
                private.send::<IoExit>(&code).await?;
                return Ok(code);
            }
            FileBoundary::Closed { pending, .. } => {
                let reply =
                    json!({"status":"out_of_order","error":"the imaging Endpoint has completed"});
                let bytes = json::encode(&reply)?;
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
