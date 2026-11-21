//! One actual worker receives a tool, performs its request and returns the tool.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, global::FileExit, imp::service::ReceivedEffects};
use hibana::{Endpoint, g::Message};

#[hibana_runtime::scope]
pub async fn run<const WORKER: u8, const MACHINE: u8>(
    endpoint: &mut Endpoint<WORKER>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    let [mut leases, mut file, mut tools] = endpoint.par()?;
    let station = file.recv::<StationJob<WORKER>>().await?;
    let arguments = imp::JobArguments::<WORKER>::from_bytes(station.as_bytes())?;
    let admitted = imp::submission::<WORKER>(arguments.value());
    leases.send::<MachineRequest<WORKER>>(&arguments).await?;
    let mut machine = leases.recv::<JobLoan<MACHINE>>().await?;
    machine.send::<Intent<WORKER>>(&arguments).await?;
    let material = {
        let (tool_result, station_result) = async {
            async {
                let branch = tools.offer().await?;
                let result = match branch.label() {
                    Loan0::LOGICAL_LABEL => {
                        let mut tool = branch.recv::<Loan0>().await?;
                        tool.send::<Request0<WORKER>>(&arguments.into()).await?;
                        let result = imp::decode(tool.recv::<Reply0>().await?)?;
                        tools.send::<Return0>(tool).await?;
                        result
                    }
                    Loan1::LOGICAL_LABEL => {
                        let mut tool = branch.recv::<Loan1>().await?;
                        tool.send::<Request1<WORKER>>(&arguments.into()).await?;
                        let result = imp::decode(tool.recv::<Reply1>().await?)?;
                        tools.send::<Return1>(tool).await?;
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
                Ok::<_, ShellError>(actual)
            };
        }
        .await?;
        leases.send::<JobLoan<MACHINE>>(machine).await?;
        if tool_result != station_result {
            return Err(ShellError::message(
                "station and tool returned different manufacturing results",
            ));
        }
        let bytes = imp::encode(&station_result)?;
        leases.send::<JobResult>(&bytes.as_slice()).await?;
        station_result["data"]["part"].clone()
    };
    let (result, bytes) = async {
        let [mut delivery, mut check] = tools.par()?;
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
        tools.send::<JobUpdate>(&bytes.as_slice()).await?;
        let closed = imp::FailureItem::closed(imp::InspectionReply::from_bytes(&bytes)?);
        tools.send::<FailureStream<WORKER>>(&closed).await?;
    } else {
        let failure = imp::FailureItem::rejected(imp::InspectionReply::from_bytes(&bytes)?);
        tools.send::<FailureStream<WORKER>>(&failure).await?;
        let request = tools.recv::<DispatchRequest<WORKER>>().await?;
        let arguments = imp::decode(request.as_bytes())?;
        imp::rework_input(&result, &arguments)?;
        let order = effects.apply(request).await?;
        imp::dispatched(&result, &arguments, &order)?;
        let bytes = imp::encode(&order)?;
        async {
            let [mut dispatcher, mut station] = tools.par()?;
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

/// Each file request competes for the same machine. Its owner regains the
/// physical Endpoint on return and retires the drive after the files close.
pub async fn station<
    'run,
    const OWNER: u8,
    const MACHINE: u8,
    const PRIMARY: u8,
    const OTHER: u8,
>(
    owner: &mut Endpoint<'run, OWNER>,
    mut machine: Endpoint<'run, MACHINE>,
) -> ShellResult<()> {
    loop {
        let claim = owner.offer().await?;
        match claim.label() {
            label if label == PRIMARY => {
                claim.recv::<MachineRequest<PRIMARY>>().await?;
            }
            label if label == OTHER => {
                claim.recv::<MachineRequest<OTHER>>().await?;
            }
            FileExit::LOGICAL_LABEL => {
                claim.recv::<FileExit>().await?;
                return Ok(());
            }
            _ => return Err(ShellError::message("unknown workstation claimant")),
        }
        owner.send::<JobLoan<MACHINE>>(machine).await?;
        machine = owner.recv::<JobLoan<MACHINE>>().await?;
    }
}

pub async fn drive<'run, const DRIVE: u8, const JOB: u8>(
    mut drive: Endpoint<'run, DRIVE>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    loop {
        let request = drive.offer().await?;
        match request.label() {
            FileExit::LOGICAL_LABEL => {
                let status = request.recv::<FileExit>().await?;
                drive.send::<FileExit>(&status).await?;
                return Ok(());
            }
            JobIntent::<JOB>::LOGICAL_LABEL => {
                let result = effects
                    .apply(request.recv::<JobIntent<JOB>>().await?)
                    .await?;
                drive
                    .send::<JobResult>(&imp::encode(&result)?.as_slice())
                    .await?;
            }
            _ => return Err(ShellError::message("unknown workstation operation")),
        }
    }
}

/// Manufacture and inspect this station-only job again for each actual dispatch.
#[hibana_runtime::scope]
pub async fn station_job<const SERVICE: u8, const MACHINE: u8, const JOB: u8>(
    endpoint: &mut Endpoint<SERVICE>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    let [mut leases, mut job] = endpoint.par()?;
    let mut previous_order = None;
    loop {
        let submission = job.recv::<StationJob<JOB>>().await?;
        let arguments = imp::JobArguments::<JOB>::from_bytes(submission.as_bytes())?;
        let admitted = imp::submission::<JOB>(arguments.value());
        leases.send::<MachineRequest<JOB>>(&arguments).await?;
        let mut machine = leases.recv::<JobLoan<MACHINE>>().await?;
        machine.send::<JobIntent<JOB>>(&arguments.into()).await?;
        let result = imp::decode(machine.recv::<JobResult>().await?)?;
        imp::completed(
            &admitted,
            &result,
            previous_order
                .as_ref()
                .map(|(reading, args, order)| (reading, args, order)),
        )?;
        leases.send::<JobLoan<MACHINE>>(machine).await?;
        let bytes = imp::encode(&result)?;
        let part = &result["data"]["part"];
        async {
            let [mut file, mut inspector] = job.par()?;
            async {
                file.send::<JobResult>(&bytes.as_slice()).await?;
            };
            async {
                inspector
                    .send::<FinishedPart<JOB>>(&imp::encode(part)?.as_slice())
                    .await?;
            };
            Ok(())
        }
        .await?;
        let request = job.recv::<CheckPart<JOB>>().await?;
        let arguments = imp::decode(request.as_bytes())?;
        imp::inspection_input(part, &arguments)?;
        let sample = imp::JobArguments::from_bytes(request.as_bytes())?;
        job.send::<CheckPart<JOB>>(&sample).await?;
        let measurement = job.offer().await?;
        let bytes = match measurement.label() {
            Approved::<JOB>::LOGICAL_LABEL => measurement.recv::<Approved<JOB>>().await?.to_vec(),
            Failed::<JOB>::LOGICAL_LABEL => measurement.recv::<Failed<JOB>>().await?.to_vec(),
            _ => return Err(ShellError::message("unexpected inspection response")),
        };
        let verdict = imp::decode(&bytes)?;
        imp::inspected(part, &arguments, &verdict)?;
        if verdict["success"] == true {
            job.send::<JobUpdate>(&bytes.as_slice()).await?;
            let closed = imp::FailureItem::closed(imp::InspectionReply::from_bytes(&bytes)?);
            job.send::<FailureStream<JOB>>(&closed).await?;
            return Ok(());
        }
        let failure = imp::FailureItem::rejected(imp::InspectionReply::from_bytes(&bytes)?);
        job.send::<FailureStream<JOB>>(&failure).await?;
        let request = job.recv::<DispatchRequest<JOB>>().await?;
        let arguments = imp::decode(request.as_bytes())?;
        imp::rework_input(&verdict, &arguments)?;
        let order = effects.apply(request).await?;
        imp::dispatched(&verdict, &arguments, &order)?;
        let bytes = imp::encode(&order)?;
        async {
            let [mut dispatcher, mut station] = job.par()?;
            async {
                dispatcher.send::<WorkOrder<JOB>>(&bytes.as_slice()).await?;
            };
            async {
                station.send::<JobUpdate>(&bytes.as_slice()).await?;
            };
            Ok(())
        }
        .await?;
        previous_order = Some((verdict, arguments, order));
    }
}
