//! A real review is independent of the clinical panel and imaging work.
use super::super::global::*;
use crate::{
    imp::{json, patient},
    shell::ShellResult,
};
use hibana::Endpoint;

#[hibana_runtime::scope]
pub async fn run(endpoint: &mut Endpoint<PHARMACIST>) -> ShellResult<()> {
    let result = json::decode(endpoint.recv::<ReviewResult>().await?)?;
    patient::actual_result(&result)?;
    let bytes = json::encode(&result)?;
    endpoint.send::<MedicationReview>(&bytes.as_slice()).await?;
    Ok(())
}
