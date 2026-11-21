//! The same gauge measures every material presented by a workstation.
use super::super::{global::*, imp};
use crate::shell::{
    ShellError, ShellResult,
    global::{FileExit, Loan},
    imp::service::ReceivedEffects,
};
use hibana::{Endpoint, g::Message};

pub async fn bench<'run>(bench: &mut Endpoint<'run, BENCH>) -> ShellResult<i32> {
    let mut gauge = bench.recv::<Loan<GAUGE>>().await?;
    // Physical specimens on the inspection bench; replacements carry a new artifact and attempt.
    let mut specimens: [Option<serde_json::Value>; imp::JOBS.len()] = Default::default();
    loop {
        let ready = bench.offer().await?;
        let job = ready.label();
        let arguments = match job {
            FileExit::LOGICAL_LABEL => {
                let status = ready.recv::<FileExit>().await?;
                bench.send::<Loan<GAUGE>>(gauge).await?;
                return Ok(status);
            }
            ALPHA => {
                specimens[usize::from(WORKER_A)] =
                    Some(imp::decode(ready.recv::<Sample<ALPHA, WORKER_A>>().await?)?);
                continue;
            }
            BETA => {
                specimens[usize::from(WORKER_B)] =
                    Some(imp::decode(ready.recv::<Sample<BETA, WORKER_B>>().await?)?);
                continue;
            }
            GAMMA => {
                specimens[usize::from(WORKER_C)] =
                    Some(imp::decode(ready.recv::<Sample<GAMMA, WORKER_C>>().await?)?);
                continue;
            }
            BETA_A => {
                specimens[usize::from(BETA_JOB_A)] = Some(imp::decode(
                    ready.recv::<Sample<BETA_A, BETA_JOB_A>>().await?,
                )?);
                continue;
            }
            GAMMA_B => {
                specimens[usize::from(GAMMA_JOB_B)] = Some(imp::decode(
                    ready.recv::<Sample<GAMMA_B, GAMMA_JOB_B>>().await?,
                )?);
                continue;
            }
            ALPHA_C => {
                specimens[usize::from(ALPHA_JOB_C)] = Some(imp::decode(
                    ready.recv::<Sample<ALPHA_C, ALPHA_JOB_C>>().await?,
                )?);
                continue;
            }
            WORKER_A => ready
                .recv::<CheckPart<WORKER_A>>()
                .await?
                .as_bytes()
                .to_vec(),
            WORKER_B => ready
                .recv::<CheckPart<WORKER_B>>()
                .await?
                .as_bytes()
                .to_vec(),
            WORKER_C => ready
                .recv::<CheckPart<WORKER_C>>()
                .await?
                .as_bytes()
                .to_vec(),
            BETA_JOB_A => ready
                .recv::<CheckPart<BETA_JOB_A>>()
                .await?
                .as_bytes()
                .to_vec(),
            GAMMA_JOB_B => ready
                .recv::<CheckPart<GAMMA_JOB_B>>()
                .await?
                .as_bytes()
                .to_vec(),
            ALPHA_JOB_C => ready
                .recv::<CheckPart<ALPHA_JOB_C>>()
                .await?
                .as_bytes()
                .to_vec(),
            _ => return Err(ShellError::message("unknown inspection sample")),
        };
        let specimen = specimens[usize::from(job)]
            .as_ref()
            .ok_or_else(|| ShellError::message("inspection without a delivered specimen"))?;
        let requested = imp::decode(&arguments)?;
        imp::inspection_input(specimen, &requested)?;
        match job {
            WORKER_A => {
                gauge
                    .send::<CheckPart<WORKER_A>>(&imp::JobArguments::from_bytes(&arguments)?)
                    .await?
            }
            WORKER_B => {
                gauge
                    .send::<CheckPart<WORKER_B>>(&imp::JobArguments::from_bytes(&arguments)?)
                    .await?
            }
            WORKER_C => {
                gauge
                    .send::<CheckPart<WORKER_C>>(&imp::JobArguments::from_bytes(&arguments)?)
                    .await?
            }
            BETA_JOB_A => {
                gauge
                    .send::<CheckPart<BETA_JOB_A>>(&imp::JobArguments::from_bytes(&arguments)?)
                    .await?
            }
            GAMMA_JOB_B => {
                gauge
                    .send::<CheckPart<GAMMA_JOB_B>>(&imp::JobArguments::from_bytes(&arguments)?)
                    .await?
            }
            ALPHA_JOB_C => {
                gauge
                    .send::<CheckPart<ALPHA_JOB_C>>(&imp::JobArguments::from_bytes(&arguments)?)
                    .await?
            }
            _ => unreachable!(),
        }
        let reading = gauge.recv::<InspectionResult>().await?.to_vec();
        imp::inspected(specimen, &requested, &imp::decode(&reading)?)?;
        gauge.send::<InspectionResult>(&reading.as_slice()).await?;
        match job {
            WORKER_A => {
                if imp::decode(&reading)?["success"] == true {
                    bench
                        .send::<Approved<WORKER_A>>(&imp::InspectionReply::from_bytes(&reading)?)
                        .await?;
                } else {
                    bench
                        .send::<Failed<WORKER_A>>(&imp::InspectionReply::from_bytes(&reading)?)
                        .await?;
                }
            }
            WORKER_B => {
                if imp::decode(&reading)?["success"] == true {
                    bench
                        .send::<Approved<WORKER_B>>(&imp::InspectionReply::from_bytes(&reading)?)
                        .await?;
                } else {
                    bench
                        .send::<Failed<WORKER_B>>(&imp::InspectionReply::from_bytes(&reading)?)
                        .await?;
                }
            }
            WORKER_C => {
                if imp::decode(&reading)?["success"] == true {
                    bench
                        .send::<Approved<WORKER_C>>(&imp::InspectionReply::from_bytes(&reading)?)
                        .await?;
                } else {
                    bench
                        .send::<Failed<WORKER_C>>(&imp::InspectionReply::from_bytes(&reading)?)
                        .await?;
                }
            }
            BETA_JOB_A => {
                if imp::decode(&reading)?["success"] == true {
                    bench
                        .send::<Approved<BETA_JOB_A>>(&imp::InspectionReply::from_bytes(&reading)?)
                        .await?;
                } else {
                    bench
                        .send::<Failed<BETA_JOB_A>>(&imp::InspectionReply::from_bytes(&reading)?)
                        .await?;
                }
            }
            GAMMA_JOB_B => {
                if imp::decode(&reading)?["success"] == true {
                    bench
                        .send::<Approved<GAMMA_JOB_B>>(&imp::InspectionReply::from_bytes(&reading)?)
                        .await?;
                } else {
                    bench
                        .send::<Failed<GAMMA_JOB_B>>(&imp::InspectionReply::from_bytes(&reading)?)
                        .await?;
                }
            }
            ALPHA_JOB_C => {
                if imp::decode(&reading)?["success"] == true {
                    bench
                        .send::<Approved<ALPHA_JOB_C>>(&imp::InspectionReply::from_bytes(&reading)?)
                        .await?;
                } else {
                    bench
                        .send::<Failed<ALPHA_JOB_C>>(&imp::InspectionReply::from_bytes(&reading)?)
                        .await?;
                }
            }
            _ => unreachable!(),
        }
    }
}

pub async fn readout<'run>(
    mut readout: Endpoint<'run, READOUT>,
    gauge: Endpoint<'run, GAUGE>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    readout.send::<Loan<GAUGE>>(gauge).await?;
    loop {
        let request = readout.offer().await?;
        let reading = match request.label() {
            <Loan<GAUGE> as hibana::g::Message>::LOGICAL_LABEL => {
                let _gauge = request.recv::<Loan<GAUGE>>().await?;
                return Ok(());
            }
            WORKER_A => {
                effects
                    .apply(request.recv::<CheckPart<WORKER_A>>().await?)
                    .await?
            }
            WORKER_B => {
                effects
                    .apply(request.recv::<CheckPart<WORKER_B>>().await?)
                    .await?
            }
            WORKER_C => {
                effects
                    .apply(request.recv::<CheckPart<WORKER_C>>().await?)
                    .await?
            }
            BETA_JOB_A => {
                effects
                    .apply(request.recv::<CheckPart<BETA_JOB_A>>().await?)
                    .await?
            }
            GAMMA_JOB_B => {
                effects
                    .apply(request.recv::<CheckPart<GAMMA_JOB_B>>().await?)
                    .await?
            }
            ALPHA_JOB_C => {
                effects
                    .apply(request.recv::<CheckPart<ALPHA_JOB_C>>().await?)
                    .await?
            }
            _ => return Err(ShellError::message("unknown gauge measurement")),
        };
        readout
            .send::<InspectionResult>(&imp::encode(&reading)?.as_slice())
            .await?;
    }
}
