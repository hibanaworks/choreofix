//! The fork owner receives each real meal request and transfers its actual right.
use super::super::global::{Eat, ForkRequest, Loan, Meal};
use crate::shell::{ShellError, ShellResult};
use hibana::Endpoint;
#[hibana_runtime::scope]
pub async fn run<'run, const NEIGHBOR: u8, const ROLE: u8, const RESOURCE: u8>(
    endpoint: &mut Endpoint<'run, ROLE>,
    mut resource: Endpoint<'run, RESOURCE>,
) -> ShellResult<()> {
    for _ in 0..2 {
        let request = endpoint.offer().await?;
        match request.label() {
            label if label == ROLE => {
                let admitted = request.recv::<Eat<ROLE>>().await?;
                resource
                    .send::<ForkRequest<RESOURCE, ROLE>>(&super::super::imp::ForkRequest(&admitted))
                    .await?;
            }
            label if label == RESOURCE => {
                let admitted = request.recv::<Eat<RESOURCE>>().await?;
                resource
                    .send::<ForkRequest<RESOURCE, NEIGHBOR>>(&super::super::imp::ForkRequest(
                        &admitted,
                    ))
                    .await?;
            }
            _ => return Err(ShellError::message("unexpected fork request")),
        }
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
