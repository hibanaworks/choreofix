//! Use the actual received operating-room continuation for one admitted request.
use super::super::{global::*, imp};
use crate::shell::ShellResult;
use hibana::Endpoint;
#[hibana_runtime::scope]
pub async fn run(endpoint: &mut Endpoint<SURGEON>) -> ShellResult<()> {
    let admitted = endpoint.recv::<SurgeryAdmitted>().await?;
    let mut room = endpoint.recv::<SurgeryLoan>().await?;
    room.send::<SurgeryRequest>(&admitted).await?;
    let result = imp::decode(room.recv::<SurgeryResult>().await?)?;
    imp::actual_result(&result, "patient_a")?;
    endpoint.send::<SurgeryLoan>(room).await?;
    Ok(())
}
