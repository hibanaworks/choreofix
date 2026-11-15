//! CT and MRI each run directly through their actual loaned Endpoint.
use super::super::global::*;
use crate::imp::{json, patient};
use crate::shell::{ShellError, ShellResult};
use hibana::Endpoint;
#[hibana_runtime::scope]
pub async fn run(endpoint: &mut Endpoint<RADIOLOGIST>) -> ShellResult<()> {
    async {
        let [mut ct, mut mri] = endpoint.par()?;
        async {
            let assessment = json::decode(ct.recv::<CtOrder>().await?)?;
            patient::actual_result(&assessment)?;
            let arguments = ct.recv::<CtPerformAdmitted>().await?;
            let mut instrument = ct.recv::<CtLoan>().await?;
            instrument.send::<CtPerformRequest>(&arguments).await?;
            let image = json::decode(instrument.recv::<CtImageResult>().await?)?;
            patient::actual_result(&image)?;
            ct.send::<CtLoan>(instrument).await?;
            let arguments = ct.recv::<CtInterpretAdmitted>().await?;
            let mut instrument = ct.recv::<CtLoan>().await?;
            instrument.send::<CtInterpretRequest>(&arguments).await?;
            let report = json::decode(instrument.recv::<CtReportResult>().await?)?;
            patient::actual_result(&report)?;
            json::same(
                &report["data"]["inputs"]["image"],
                &image["data"]["artifact_id"],
            )?;
            ct.send::<CtLoan>(instrument).await?;
        };
        async {
            let assessment = json::decode(mri.recv::<MriOrder>().await?)?;
            patient::actual_result(&assessment)?;
            let arguments = mri.recv::<MriPerformAdmitted>().await?;
            let mut instrument = mri.recv::<MriLoan>().await?;
            instrument.send::<MriPerformRequest>(&arguments).await?;
            let image = json::decode(instrument.recv::<MriImageResult>().await?)?;
            patient::actual_result(&image)?;
            mri.send::<MriLoan>(instrument).await?;
            let arguments = mri.recv::<MriInterpretAdmitted>().await?;
            let mut instrument = mri.recv::<MriLoan>().await?;
            instrument.send::<MriInterpretRequest>(&arguments).await?;
            let report = json::decode(instrument.recv::<MriReportResult>().await?)?;
            patient::actual_result(&report)?;
            json::same(
                &report["data"]["inputs"]["image"],
                &image["data"]["artifact_id"],
            )?;
            mri.send::<MriLoan>(instrument).await?;
        };
        Ok::<(), ShellError>(())
    }
    .await?;
    Ok(())
}
