//! One actual worker receives a tool, performs its request and returns the tool.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::ReceivedEffects};
use hibana::{Endpoint, g::Message};

#[hibana_runtime::scope]
pub async fn run<const WORKER: u8, const MACHINE: u8>(
    endpoint: &mut Endpoint<WORKER>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    let [mut intent, mut work] = endpoint.par()?;
    let material = {
        let station = intent.recv::<StationJob<WORKER>>().await?;
        let arguments = imp::JobArguments::<WORKER>::from_bytes(station.as_bytes())?;
        let admitted = imp::submission::<WORKER>(arguments.value());
        let mut machine = intent.recv::<JobLoan<MACHINE>>().await?;
        machine.send::<Intent<WORKER>>(&arguments).await?;
        let branch = work.offer().await?;
        let material = match branch.label() {
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
        }?;
        let actual = imp::decode(machine.recv::<StationResult<MACHINE>>().await?)?;
        imp::completed(&admitted, &actual, None)?;
        if actual["data"]["part"] != material {
            return Err(ShellError::message(
                "station and tool returned different material",
            ));
        }
        intent.send::<JobLoan<MACHINE>>(machine).await?;
        material
    };
    let (result, bytes) = async {
        let [mut delivery, mut check] = work.par()?;
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
        work.send::<Approved<WORKER>>(&imp::InspectionReply::from_bytes(&bytes)?)
            .await?;
        work.send::<JobUpdate>(&bytes.as_slice()).await?;
    } else {
        work.send::<Failed<WORKER>>(&imp::InspectionReply::from_bytes(&bytes)?)
            .await?;
        let request = work.recv::<DispatchRequest<WORKER>>().await?;
        let arguments = imp::decode(request.as_bytes())?;
        imp::rework_input(&result, &arguments)?;
        let order = effects.apply(request).await?;
        imp::dispatched(&result, &arguments, &order)?;
        let bytes = imp::encode(&order)?;
        async {
            let [mut dispatcher, mut station] = work.par()?;
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

/// Manufacture the job that needs only this workstation.
#[hibana_runtime::scope]
pub async fn station_job<const SERVICE: u8, const MACHINE: u8, const JOB: u8>(
    endpoint: &mut Endpoint<SERVICE>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    let submission = endpoint.recv::<StationJob<JOB>>().await?;
    let arguments = imp::JobArguments::<JOB>::from_bytes(submission.as_bytes())?;
    let admitted = imp::submission::<JOB>(arguments.value());
    let mut machine = endpoint.recv::<JobLoan<MACHINE>>().await?;
    let received = async {
        async {
            machine.send::<JobIntent<JOB>>(&arguments.into()).await?;
        };
        Ok::<_, ShellError>(endpoint.recv::<JobIntent<JOB>>().await?)
    }
    .await?;
    if imp::decode(received.as_bytes())? != admitted["arguments"] {
        return Err(ShellError::message(
            "workstation request differs from the submitted job",
        ));
    }
    let result = effects.apply(received).await?;
    imp::completed(&admitted, &result, None)?;
    let bytes = imp::encode(&result)?;
    async {
        async {
            endpoint.send::<JobResult>(&bytes.as_slice()).await?;
        };
        let confirmed = machine.recv::<JobResult>().await?;
        if confirmed[..] != bytes {
            return Err(ShellError::message(
                "workstation reply differs from its actual effect",
            ));
        }
        Ok::<(), ShellError>(())
    }
    .await?;
    let [mut returned, mut file, mut inspector] = endpoint.par()?;
    async {
        returned.send::<JobLoan<MACHINE>>(machine).await?;
    };
    async {
        file.send::<JobResult>(&bytes.as_slice()).await?;
    };
    async {
        let part = imp::encode(&result["data"]["part"])?;
        inspector
            .send::<FinishedPart<JOB>>(&part.as_slice())
            .await?;
    };
    Ok(())
}
