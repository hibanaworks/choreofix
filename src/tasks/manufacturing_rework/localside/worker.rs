//! One actual worker receives a tool, performs its request and returns the tool.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult};
use hibana::{Endpoint, g::Message};
use serde_json::Value;

#[hibana_runtime::scope]
pub async fn run<const WORKER: u8>(
    endpoint: &mut Endpoint<WORKER>,
    bytes: &[u8],
) -> ShellResult<Value> {
    let arguments = imp::JobArguments::<WORKER>::from_bytes(bytes)?;
    let admitted = imp::admitted(&arguments)?;
    let [mut intent, mut work] = endpoint.par()?;
    intent.send::<Intent<WORKER>>(&arguments).await?;
    let branch = work.offer().await?;
    let result = match branch.label() {
        Loan0::LOGICAL_LABEL => {
            let mut tool = branch.recv::<Loan0>().await?;
            tool.send::<Request0<WORKER>>(&arguments).await?;
            let result = imp::decode(tool.recv::<Reply0>().await?)?;
            imp::completed(&admitted, &result, None)?;
            work.send::<Return0>(tool).await?;
            result
        }
        Loan1::LOGICAL_LABEL => {
            let mut tool = branch.recv::<Loan1>().await?;
            tool.send::<Request1<WORKER>>(&arguments).await?;
            let result = imp::decode(tool.recv::<Reply1>().await?)?;
            imp::completed(&admitted, &result, None)?;
            work.send::<Return1>(tool).await?;
            result
        }
        _ => return Err(ShellError::message("unexpected tool loan")),
    };
    Ok(result)
}
