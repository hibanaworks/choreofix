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
        let (tool_result, station_result) = async {
            async {
                let branch = work.offer().await?;
                let result = match branch.label() {
                    Loan0::LOGICAL_LABEL => {
                        let mut tool = branch.recv::<Loan0>().await?;
                        tool.send::<Request0<WORKER>>(&arguments.into()).await?;
                        let result = imp::decode(tool.recv::<Reply0>().await?)?;
                        work.send::<Return0>(tool).await?;
                        result
                    }
                    Loan1::LOGICAL_LABEL => {
                        let mut tool = branch.recv::<Loan1>().await?;
                        tool.send::<Request1<WORKER>>(&arguments.into()).await?;
                        let result = imp::decode(tool.recv::<Reply1>().await?)?;
                        work.send::<Return1>(tool).await?;
                        result
                    }
                    _ => return Err(ShellError::message("unexpected tool loan")),
                };
                imp::completed(&admitted, &result, None)?;
                Ok::<_, ShellError>(result)
            };
            async {
                let actual = imp::decode(machine.recv::<StationResult<MACHINE>>().await?)?;
                imp::completed(&admitted, &actual, None)?;
                intent.send::<JobLoan<MACHINE>>(machine).await?;
                Ok::<_, ShellError>(actual)
            };
        }
        .await?;
        if tool_result != station_result {
            return Err(ShellError::message(
                "station and tool returned different manufacturing results",
            ));
        }
        let bytes = imp::encode(&station_result)?;
        intent.send::<JobResult>(&bytes.as_slice()).await?;
        station_result["data"]["part"].clone()
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
        let sample = imp::JobArguments::from_bytes(request.as_bytes())?;
        check.send::<CheckPart<WORKER>>(&sample).await?;
        let result = imp::decode(check.recv::<Measurement<WORKER>>().await?)?;
        imp::inspected(&material, &arguments, &result)?;
        let bytes = imp::encode(&result)?;
        Ok((result, bytes))
    }
    .await?;
    if result["success"] == true {
        work.send::<JobUpdate>(&bytes.as_slice()).await?;
        let closed = imp::FailureItem::closed(imp::InspectionReply::from_bytes(&bytes)?);
        work.send::<FailureStream<WORKER>>(&closed).await?;
    } else {
        let failure = imp::FailureItem::rejected(imp::InspectionReply::from_bytes(&bytes)?);
        work.send::<FailureStream<WORKER>>(&failure).await?;
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
    async {
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
    .await?;
    let request = endpoint.recv::<CheckPart<JOB>>().await?;
    let arguments = imp::decode(request.as_bytes())?;
    let material = &result["data"]["part"];
    imp::inspection_input(material, &arguments)?;
    let sample = imp::JobArguments::from_bytes(request.as_bytes())?;
    endpoint.send::<CheckPart<JOB>>(&sample).await?;
    let verdict = imp::decode(endpoint.recv::<Measurement<JOB>>().await?)?;
    imp::inspected(material, &arguments, &verdict)?;
    let bytes = imp::encode(&verdict)?;
    if verdict["success"] == true {
        endpoint.send::<JobUpdate>(&bytes.as_slice()).await?;
        let closed = imp::FailureItem::closed(imp::InspectionReply::from_bytes(&bytes)?);
        endpoint.send::<FailureStream<JOB>>(&closed).await?;
    } else {
        let failure = imp::FailureItem::rejected(imp::InspectionReply::from_bytes(&bytes)?);
        endpoint.send::<FailureStream<JOB>>(&failure).await?;
        let request = endpoint.recv::<DispatchRequest<JOB>>().await?;
        let arguments = imp::decode(request.as_bytes())?;
        imp::rework_input(&verdict, &arguments)?;
        let order = effects.apply(request).await?;
        imp::dispatched(&verdict, &arguments, &order)?;
        let bytes = imp::encode(&order)?;
        async {
            let [mut dispatcher, mut station] = endpoint.par()?;
            async {
                dispatcher.send::<WorkOrder<JOB>>(&bytes.as_slice()).await?;
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
