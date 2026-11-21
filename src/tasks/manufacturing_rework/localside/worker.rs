//! One actual worker receives a tool, performs its request and returns the tool.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::ReceivedEffects};
use hibana::{Endpoint, g::Message};

#[hibana_runtime::scope]
pub async fn run<const WORKER: u8>(
    endpoint: &mut Endpoint<WORKER>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
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
                    let material = result["data"]["part"].clone();
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
                    let material = result["data"]["part"].clone();
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
    let (result, bytes) = async {
        let [mut delivery, mut check] = endpoint.par()?;
        async {
            delivery
                .send::<FinishedPart<WORKER>>(&imp::encode(&material)?.as_slice())
                .await?;
        };
        let request = check.recv::<CheckPart<WORKER>>().await?;
        let arguments = imp::decode(request.as_bytes())?;
        imp::inspection_input(&material, &arguments)?;
        let result = effects.apply(request).await?;
        imp::inspected(&material, &arguments, &result)?;
        let bytes = imp::encode(&result)?;
        check.send::<InspectionResult>(&bytes.as_slice()).await?;
        Ok((result, bytes))
    }
    .await?;
    if result["success"] == true {
        endpoint
            .send::<Approved<WORKER>>(&imp::InspectionReply::from_bytes(&bytes)?)
            .await?;
        endpoint.send::<JobUpdate>(&bytes.as_slice()).await?;
    } else {
        endpoint
            .send::<Failed<WORKER>>(&imp::InspectionReply::from_bytes(&bytes)?)
            .await?;
        let request = endpoint.recv::<DispatchRequest<WORKER>>().await?;
        let arguments = imp::decode(request.as_bytes())?;
        imp::rework_input(&result, &arguments)?;
        let order = effects.apply(request).await?;
        imp::dispatched(&result, &arguments, &order)?;
        let bytes = imp::encode(&order)?;
        async {
            let [mut dispatcher, mut station] = endpoint.par()?;
            async {
                dispatcher
                    .send::<WorkOrder<WORKER>>(&bytes.as_slice())
                    .await?;
            };
            async {
                station.send::<JobUpdate>(&bytes.as_slice()).await?;
            };
            Ok(())
        }
        .await?;
    }
    Ok(())
}
