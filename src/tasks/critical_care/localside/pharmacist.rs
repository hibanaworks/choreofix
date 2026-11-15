//! The actual review reply remains independent of the two patient workups.
use super::super::{global::*, imp};
use crate::shell::ShellResult;
use hibana::Endpoint;
#[hibana_runtime::scope]
pub async fn run(endpoint: &mut Endpoint<PHARMACIST>) -> ShellResult<()> {
    let result = imp::decode(endpoint.recv::<ReviewResult>().await?)?;
    imp::actual_result(&result, "patient_A_and_B")?;
    Ok(())
}
