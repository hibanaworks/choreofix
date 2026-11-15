//! Use the actual received operating-room continuation for one admitted request.
use super::super::{global::*, imp};
use crate::shell::ShellResult;
use hibana::Endpoint;
#[hibana_runtime::scope]
pub async fn run(endpoint: &mut Endpoint<ANESTHESIOLOGIST>) -> ShellResult<()> {
    let admitted = endpoint.recv::<AnesthesiaAdmitted>().await?;
    let mut room = endpoint.recv::<AnesthesiaLoan>().await?;
    room.send::<AnesthesiaRequest>(&admitted).await?;
    let result = imp::decode(room.recv::<AnesthesiaResult>().await?)?;
    imp::actual_result(&result, "patient_a")?;
    endpoint.send::<AnesthesiaLoan>(room).await?;
    Ok(())
}
