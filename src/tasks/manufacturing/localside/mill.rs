//! The mill's source order is independent from the lathe's source order.
use super::super::global::*;
use crate::shell::ShellResult;
use hibana::Endpoint;

#[hibana_runtime::scope]
pub async fn run<'run>(
    endpoint: &mut Endpoint<'run, MILL_SERVICE>,
    mill: Endpoint<'run, MILL>,
) -> ShellResult<()> {
    let mut resource = mill;
    for _ in 0..2 {
        drop(endpoint.offer().await?);
        endpoint.send::<MillLoan>(resource).await?;
        resource = endpoint.recv::<ReturnMill>().await?;
    }
    Ok(())
}
