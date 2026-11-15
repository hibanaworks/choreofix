//! Each actual file request selects the next borrower of this one machine.
use super::super::global::*;
use crate::shell::ShellResult;
use hibana::Endpoint;

#[hibana_runtime::scope]
pub async fn run<'run>(
    endpoint: &mut Endpoint<'run, LATHE_SERVICE>,
    lathe: Endpoint<'run, LATHE>,
) -> ShellResult<()> {
    let mut resource = lathe;
    for _ in 0..2 {
        drop(endpoint.offer().await?);
        endpoint.send::<LatheLoan>(resource).await?;
        resource = endpoint.recv::<ReturnLathe>().await?;
    }
    Ok(())
}
