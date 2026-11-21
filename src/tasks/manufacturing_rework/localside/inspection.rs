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
    loop {
        let ready = bench.offer().await?;
        let job = ready.label();
        match job {
            FileExit::LOGICAL_LABEL => {
                let status = ready.recv::<FileExit>().await?;
                bench.send::<Loan<GAUGE>>(gauge).await?;
                return Ok(status);
            }
            WORKER_A => {
                let sample = ready.recv::<CheckPart<WORKER_A>>().await?;
                gauge
                    .send::<CheckPart<WORKER_A>>(&imp::JobArguments::from_bytes(sample.as_bytes())?)
                    .await?;
            }
            WORKER_B => {
                let sample = ready.recv::<CheckPart<WORKER_B>>().await?;
                gauge
                    .send::<CheckPart<WORKER_B>>(&imp::JobArguments::from_bytes(sample.as_bytes())?)
                    .await?;
            }
            WORKER_C => {
                let sample = ready.recv::<CheckPart<WORKER_C>>().await?;
                gauge
                    .send::<CheckPart<WORKER_C>>(&imp::JobArguments::from_bytes(sample.as_bytes())?)
                    .await?;
            }
            BETA_JOB_A => {
                let sample = ready.recv::<CheckPart<BETA_JOB_A>>().await?;
                gauge
                    .send::<CheckPart<BETA_JOB_A>>(&imp::JobArguments::from_bytes(
                        sample.as_bytes(),
                    )?)
                    .await?;
            }
            GAMMA_JOB_B => {
                let sample = ready.recv::<CheckPart<GAMMA_JOB_B>>().await?;
                gauge
                    .send::<CheckPart<GAMMA_JOB_B>>(&imp::JobArguments::from_bytes(
                        sample.as_bytes(),
                    )?)
                    .await?;
            }
            ALPHA_JOB_C => {
                let sample = ready.recv::<CheckPart<ALPHA_JOB_C>>().await?;
                gauge
                    .send::<CheckPart<ALPHA_JOB_C>>(&imp::JobArguments::from_bytes(
                        sample.as_bytes(),
                    )?)
                    .await?;
            }
            _ => return Err(ShellError::message("unknown inspection sample")),
        }
        let reading = gauge.recv::<InspectionResult>().await?.to_vec();
        gauge.send::<InspectionResult>(&reading.as_slice()).await?;
        match job {
            WORKER_A => {
                bench
                    .send::<Measurement<WORKER_A>>(&reading.as_slice())
                    .await?
            }
            WORKER_B => {
                bench
                    .send::<Measurement<WORKER_B>>(&reading.as_slice())
                    .await?
            }
            WORKER_C => {
                bench
                    .send::<Measurement<WORKER_C>>(&reading.as_slice())
                    .await?
            }
            BETA_JOB_A => {
                bench
                    .send::<Measurement<BETA_JOB_A>>(&reading.as_slice())
                    .await?
            }
            GAMMA_JOB_B => {
                bench
                    .send::<Measurement<GAMMA_JOB_B>>(&reading.as_slice())
                    .await?
            }
            ALPHA_JOB_C => {
                bench
                    .send::<Measurement<ALPHA_JOB_C>>(&reading.as_slice())
                    .await?
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
