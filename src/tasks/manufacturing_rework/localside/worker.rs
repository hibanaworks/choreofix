//! One actual worker receives a tool, performs its request and returns the tool.
use super::{
    super::{global::*, imp},
    tool_depot::Tool,
};
use crate::shell::{
    ShellError, ShellResult,
    global::FileExit,
    imp::service::{PairedEffects, ReceivedEffects, ResourceReceipt},
};
use hibana::{Endpoint, g::Message};

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

pub async fn drive<
    'run,
    const DRIVE: u8,
    const PRIMARY: u8,
    const OTHER: u8,
    const FIRST: u8,
    const SECOND: u8,
>(
    mut drive: Endpoint<'run, DRIVE>,
    effects: &impl PairedEffects,
) -> ShellResult<i32> {
    let [mut machine, mut first, mut second, mut close] = drive.par()?;
    loop {
        let request = hibana_runtime::race(machine.offer(), close.offer()).await?;
        match request.label() {
            label if label == DRIVE => {
                let status = request.recv::<StationClosed<DRIVE>>().await?;
                close.send::<FileExit>(&status).await?;
                return Ok(status);
            }
            JobIntent::<OTHER>::LOGICAL_LABEL => {
                let result = effects
                    .apply(request.recv::<JobIntent<OTHER>>().await?)
                    .await?;
                machine
                    .send::<JobResult>(&imp::encode(&result)?.as_slice())
                    .await?;
            }
            MachineToolWork::<PRIMARY>::LOGICAL_LABEL => {
                let station =
                    ResourceReceipt::retain(request.recv::<MachineToolWork<PRIMARY>>().await?);
                let demand = imp::JobArguments::from_bytes(station.as_bytes())?;
                let arguments = imp::JobArguments::from_bytes(station.as_bytes())?;
                machine.send::<MachineRequest<PRIMARY>>(&demand).await?;
                let grant = {
                    hibana_runtime::race(
                        async {
                            Ok::<_, ShellError>(Tool::First(
                                first.recv::<SlotLoan<TOOL_0, FIRST>>().await?,
                            ))
                        },
                        async {
                            Ok(Tool::Second(
                                second.recv::<SlotLoan<TOOL_1, SECOND>>().await?,
                            ))
                        },
                    )
                    .await?
                };
                let result = match grant {
                    Tool::First(mut tool) => {
                        tool.send::<ToolWork<TOOL_0, PRIMARY>>(&arguments).await?;
                        let received = first.recv::<ToolWork<TOOL_0, PRIMARY>>().await?;
                        let result = effects.apply_pair(station, received).await?;
                        first
                            .send::<ToolReply<TOOL_0>>(&imp::encode(&result)?.as_slice())
                            .await?;
                        let actual = imp::decode(tool.recv::<ToolReply<TOOL_0>>().await?)?;
                        first.send::<SlotLoan<TOOL_0, TOOL_0>>(tool).await?;
                        actual
                    }
                    Tool::Second(mut tool) => {
                        tool.send::<ToolWork<TOOL_1, PRIMARY>>(&arguments).await?;
                        let received = second.recv::<ToolWork<TOOL_1, PRIMARY>>().await?;
                        let result = effects.apply_pair(station, received).await?;
                        second
                            .send::<ToolReply<TOOL_1>>(&imp::encode(&result)?.as_slice())
                            .await?;
                        let actual = imp::decode(tool.recv::<ToolReply<TOOL_1>>().await?)?;
                        second.send::<SlotLoan<TOOL_1, TOOL_1>>(tool).await?;
                        actual
                    }
                };
                machine
                    .send::<JobResult>(&imp::encode(&result)?.as_slice())
                    .await?;
            }
            _ => return Err(ShellError::message("unknown workstation operation")),
        }
    }
}
/// The submitting worker reacquires the machine for every accepted work order.
#[hibana_runtime::scope]
pub async fn station_job<const SERVICE: u8, const MACHINE: u8, const JOB: u8, const FILE: u8>(
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
        if JOB < 3 {
            machine
                .send::<MachineToolWork<JOB>>(&arguments.into())
                .await?;
        } else {
            machine.send::<JobIntent<JOB>>(&arguments.into()).await?;
        }
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
        let material = imp::encode(part)?;
        let material = imp::Part::from_bytes(&material)?;
        async {
            let [mut report, mut bench] = job.par()?;
            async {
                leases.send::<FinishedPart<JOB>>(&material).await?;
            };
            async {
                bench.send::<Sample<FILE, JOB>>(&material).await?;
            };
            report.send::<JobResult>(&bytes.as_slice()).await?;
            Ok(())
        }
        .await?;
        let measurement = job.offer().await?;
        let bytes = match measurement.label() {
            Approved::<JOB>::LOGICAL_LABEL => measurement.recv::<Approved<JOB>>().await?.to_vec(),
            Failed::<JOB>::LOGICAL_LABEL => measurement.recv::<Failed<JOB>>().await?.to_vec(),
            _ => return Err(ShellError::message("unexpected inspection response")),
        };
        let verdict = imp::decode(&bytes)?;
        imp::inspected(part, &admitted["arguments"], &verdict)?;
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
