//! The fork owner receives each real meal request and transfers its actual right.
use super::super::global::{Eat, Loan, Meal};
use crate::shell::{ShellError, ShellResult};
use hibana::Endpoint;
#[hibana_runtime::scope]
pub async fn run<'run, const ROLE: u8, const RESOURCE: u8>(
    endpoint: &mut Endpoint<'run, ROLE>,
    mut resource: Endpoint<'run, RESOURCE>,
) -> ShellResult<()> {
    for _ in 0..2 {
        let admitted = endpoint.recv::<Eat<RESOURCE>>().await?;
        resource.send::<Eat<RESOURCE>>(&admitted).await?;
        endpoint.send::<Loan<RESOURCE>>(resource).await?;
        let result = endpoint.recv::<Meal<RESOURCE>>().await?;
        let result: serde_json::Value =
            serde_json::from_slice(result).map_err(|e| ShellError::message(e.to_string()))?;
        if !result.is_object() {
            return Err(ShellError::message("invalid actual meal result"));
        }
        resource = endpoint.recv::<Loan<RESOURCE>>().await?;
    }
    Ok(())
}
