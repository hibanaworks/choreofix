//! One actual worker receives a tool, performs its request and returns the tool.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult};
use hibana::{Endpoint, g::Message};

#[hibana_runtime::scope]
pub async fn run<const WORKER: u8>(endpoint: &mut Endpoint<WORKER>) -> ShellResult<()> {
    let material = {
        let [mut intent, mut work] = endpoint.par()?;
        let station = intent.recv::<StationJob<WORKER>>().await?;
        let arguments = imp::JobArguments::<WORKER>::from_bytes(station.as_bytes())?;
        let admitted = imp::submission::<WORKER>(arguments.value());
        intent.send::<Intent<WORKER>>(&arguments).await?;
        let branch = work.offer().await?;
        match branch.label() {
            Loan0::LOGICAL_LABEL => {
                async {
                    let mut tool = branch.recv::<Loan0>().await?;
                    tool.send::<Request0<WORKER>>(&arguments.into()).await?;
                    let result = imp::decode(tool.recv::<Reply0>().await?)?;
                    imp::completed(&admitted, &result, None)?;
                    let bytes = imp::encode(&result)?;
                    let material = imp::encode(&result["data"]["part"])?;
                    let [mut returned, mut part] = work.par()?;
                    async {
                        returned.send::<Return0>(tool).await?;
                    };
                    async {
                        part.send::<JobResult>(&bytes.as_slice()).await?;
                    };
                    Ok(material)
                }
                .await
            }
            Loan1::LOGICAL_LABEL => {
                async {
                    let mut tool = branch.recv::<Loan1>().await?;
                    tool.send::<Request1<WORKER>>(&arguments.into()).await?;
                    let result = imp::decode(tool.recv::<Reply1>().await?)?;
                    imp::completed(&admitted, &result, None)?;
                    let bytes = imp::encode(&result)?;
                    let material = imp::encode(&result["data"]["part"])?;
                    let [mut returned, mut part] = work.par()?;
                    async {
                        returned.send::<Return1>(tool).await?;
                    };
                    async {
                        part.send::<JobResult>(&bytes.as_slice()).await?;
                    };
                    Ok(material)
                }
                .await
            }
            _ => Err(ShellError::message("unexpected tool loan")),
        }?
    };
    endpoint
        .send::<FinishedPart<WORKER>>(&material.as_slice())
        .await?;
    Ok(())
}
