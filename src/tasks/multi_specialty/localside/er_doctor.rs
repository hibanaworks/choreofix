//! ER assessment, joint actual findings and the real conference-room request.
use super::super::global::*;
use crate::imp::{json, patient};
use crate::shell::{
    ShellError, ShellResult,
    global::Exit as IoExit,
    imp::file_wire::{ClosedFile, ResultChunk, ResultChunkMsg, SERVICE_FD},
};
use hibana::g::Message;
use hibana_wasip1_runtime::{FileBoundary, HibanaWasiGuest, WasiImportCompletion, protocol};
use serde_json::json;

#[hibana_runtime::scope]
pub async fn run(
    mut guest: &mut HibanaWasiGuest,
    endpoint: &mut hibana::Endpoint<ER_DOCTOR>,
) -> ShellResult<i32> {
    let [mut business, mut private] = endpoint.par()?;
    let ((pending,mut reply,mut case),review)=async{
  let [mut clinical,mut review]=business.par()?;
  async{
   let assessment=loop{
    match guest.next_file::<ClosedFile,ER_DOCTOR>(&mut private,SERVICE_FD).await?{
     FileBoundary::Exit(_)=>return Err(ShellError::message("ER doctor exited before actual assessment")),
     FileBoundary::Closed{pending,file}=>{
      match patient::file_arguments(&file,b"service/assess_patient"){
       Err(error)=>{
        let reply=json!({"status":"invalid_arguments","error":error.to_string()});
        let bytes=json::encode(&reply)?;
for(index,chunk)in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate(){
 private.send::<ResultChunkMsg>(&ResultChunk::new(index*protocol::WASIP1_IO_CHUNK_CAPACITY,chunk)?).await?;
}
guest=pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
 protocol::FdClosed::new_with_errno(SERVICE_FD,28),
)))?;
       }
       Ok(arguments)=>{
        let bytes=json::encode(&arguments)?;
        clinical.send::<AssessmentRequest>(&bytes.as_slice()).await?;
        let assessment=json::decode(clinical.recv::<AssessmentResult>().await?)?;
        patient::actual_result(&assessment)?;
        let bytes=json::encode(&assessment)?;
for(index,chunk)in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate(){
 private.send::<ResultChunkMsg>(&ResultChunk::new(index*protocol::WASIP1_IO_CHUNK_CAPACITY,chunk)?).await?;
}
guest=pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
 protocol::FdClosed::new_with_errno(SERVICE_FD,0),
)))?;
        break assessment;
       }
      }
     }
    }
   };
   let ((pending,reply,general,_general_panel),_card_panel,_neuro_panel)=async{
    let [mut own,mut card,mut neuro]=clinical.par()?;
    async{
     loop{
      match guest.next_file::<ClosedFile,ER_DOCTOR>(&mut private,SERVICE_FD).await?{
       FileBoundary::Exit(_)=>return Err(ShellError::message("ER doctor exited before general investigation")),
       FileBoundary::Closed{pending,file}=>{
        let arguments=patient::file_arguments(&file,b"service/order_tests").and_then(|arguments|{
         patient::tests(&arguments,&["vitals","basic_labs"])?;Ok(arguments)
        });
        let (reply,errno)=match arguments{
         Err(error)=>(json!({"status":"invalid_arguments","error":error.to_string()}),28),
         Ok(arguments)=>{
          let mut panel=own.recv::<PanelLoan<GENERAL_PANEL>>().await?;
          let bytes=json::encode(&arguments)?;
          panel.send::<TestRequest>(&bytes.as_slice()).await?;
          let attempt=json::decode(panel.recv::<TestResult>().await?)?;
          patient::actual_result(&attempt)?;
          own.send::<PanelReturn<GENERAL_PANEL>>(panel).await?;
          let branch=own.offer().await?;
          match branch.label(){
           PanelPartial::LOGICAL_LABEL =>{
            let receipts=json::decode(branch.recv::<PanelPartial>().await?)?;
            if patient::panel_complete(&receipts,&["vitals","basic_labs"])?{
             return Err(ShellError::message("partial reply contains a complete general panel"));
            }
            (attempt,0)
           }
           PanelCompleted::<GENERAL_PANEL>::LOGICAL_LABEL =>{
            let panel=branch.recv::<PanelCompleted<GENERAL_PANEL>>().await?;
            let receipts=json::decode(own.recv::<PanelComplete>().await?)?;
            if !patient::panel_complete(&receipts,&["vitals","basic_labs"])?{
             return Err(ShellError::message("completed reply lacks actual general results"));
            }
            break Ok::<_,ShellError>((pending,attempt,receipts,panel));
           }
           _=>return Err(ShellError::message("unexpected general panel reply")),
          }
         }
        };
        let bytes=json::encode(&reply)?;
for(index,chunk)in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate(){
 private.send::<ResultChunkMsg>(&ResultChunk::new(index*protocol::WASIP1_IO_CHUNK_CAPACITY,chunk)?).await?;
}
guest=pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
 protocol::FdClosed::new_with_errno(SERVICE_FD,errno),
)))?;
       }
      }
     }
    };
    async{
     let bytes=json::encode(&assessment)?;
     card.send::<CardReferral>(&bytes.as_slice()).await?;
     Ok(card.recv::<PanelCompleted<CARDIAC_PANEL>>().await?)
    };
    async{
     let bytes=json::encode(&assessment)?;
     neuro.send::<NeuroReferral>(&bytes.as_slice()).await?;
     Ok(neuro.recv::<PanelCompleted<NEURO_PANEL>>().await?)
    };
   }.await?;
   let (ct,mri)=async{
    let [mut c,mut m]=clinical.par()?;
    async{json::decode(c.recv::<CtForER>().await?)};
    async{json::decode(m.recv::<MriForER>().await?)};
   }.await?;
   let (cardiac,neuro)=async{
    let [mut c,mut n]=clinical.par()?;
    async{json::decode(c.recv::<CardDiagnosisForER>().await?)};
    async{json::decode(n.recv::<NeuroDiagnosisForER>().await?)};
   }.await?;
   for report in [&ct,&mri,&cardiac["result"],&neuro["result"]]{patient::actual_result(report)?;}
   Ok((pending,reply,json!({"assessment":assessment,"general":general,"CT":ct,"MRI":mri,
    "cardiac":cardiac,"neurology":neuro})))
  };
  async{json::decode(review.recv::<ReviewForER>().await?)};
 }.await?;
    patient::actual_result(&review)?;
    case["review"] = review;
    reply["consultation"] = case.clone();
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
    loop {
        match guest
            .next_file::<ClosedFile, ER_DOCTOR>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Exit(_) => {
                return Err(ShellError::message(
                    "ER doctor exited before the actual case conference",
                ));
            }
            FileBoundary::Closed { pending, file } => {
                let arguments = patient::file_arguments(&file, b"service/conduct_case_conference")
                    .and_then(|arguments| {
                        if arguments.get("attendees").is_some_and(|v| !v.is_string()) {
                            return Err(ShellError::message("attendees must be a string"));
                        }
                        Ok(arguments)
                    });
                let (reply, errno) = match &arguments {
                    Err(error) => (
                        json!({"status":"invalid_arguments","error":error.to_string()}),
                        28,
                    ),
                    Ok(arguments) => {
                        let admitted = json!({"arguments":arguments,"case":case});
                        let bytes = json::encode(&admitted)?;
                        business.send::<ConferenceIntent>(&bytes.as_slice()).await?;
                        let mut room = business.recv::<RoomLoan>().await?;
                        room.send::<RoomRequest>(&bytes.as_slice()).await?;
                        let conference = json::decode(room.recv::<RoomResult>().await?)?;
                        patient::actual_result(&conference)?;
                        business.send::<RoomLoan>(room).await?;
                        let final_case = json::decode(business.recv::<FinalForER>().await?)?;
                        json::same(&final_case["case"], &case)?;
                        json::same(&final_case["conference"], &conference)?;
                        let mut reply = conference;
                        reply["consultation"] = final_case;
                        (reply, 0)
                    }
                };
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
                    protocol::FdClosed::new_with_errno(SERVICE_FD, errno),
                )))?;
                if arguments.is_ok() {
                    break;
                }
            }
        }
    }
    drop(business);
    loop {
        match guest
            .next_file::<ClosedFile, ER_DOCTOR>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Exit(exit) => {
                let code = exit.status() as i32;
                private.send::<IoExit>(&code).await?;
                return Ok(code);
            }
            FileBoundary::Closed { pending, .. } => {
                let reply = json!({"status":"out_of_order","error":"the consultation Endpoint has completed"});
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
