//! Receive the actual independent medication review.
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
    Ok(())
}
