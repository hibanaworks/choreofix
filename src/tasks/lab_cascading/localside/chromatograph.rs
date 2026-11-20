//! The chromatography owner holds its actual instrument for the whole run.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::{Endpoint, g::Message};
use serde_json::Value;

#[hibana_runtime::scope]
pub async fn run<D: Domain>(endpoint: &mut Endpoint<CHROMA_OWNER>, domain: &D) -> ShellResult<()> {
    let mut instrument = endpoint.recv::<Loan<CHROMATOGRAPH>>().await?;
    let mut batch = Value::Null;
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            ChromaInspection::LOGICAL_LABEL => {
                branch.recv::<ChromaInspection>().await?;
                endpoint.send::<Loan<CHROMATOGRAPH>>(instrument).await?;
                instrument = endpoint.recv::<Loan<CHROMATOGRAPH>>().await?;
            }
            ChromaBatch::LOGICAL_LABEL => batch = branch.recv::<ChromaBatch>().await?.into_value(),
            ChromaSynthesisFailed::LOGICAL_LABEL => {
                batch = imp::decode(branch.recv::<ChromaSynthesisFailed>().await?)?
            }
            ChromaReagentsExhausted::LOGICAL_LABEL => {
                batch = branch.recv::<ChromaReagentsExhausted>().await?.into_value()
            }
            Chromatography::LOGICAL_LABEL => {
                let admitted = imp::decode(branch.recv::<Chromatography>().await?)?;
                imp::same_value(&admitted["material"]["synthesis"], &batch)?;
                imp::instrument_inputs::<6>(&admitted)?;
                endpoint.send::<Loan<CHROMATOGRAPH>>(instrument).await?;
                let received = imp::decode(endpoint.recv::<Chromatography>().await?)?;
                imp::same_value(&received, &admitted)?;
                let actual = domain
                    .execute(
                        "ANALYST",
                        "run_chromatography",
                        admitted["arguments"].clone(),
                    )
                    .await
                    .map_err(ShellError::message)?;
                imp::instrument_result::<6>(&actual, &admitted)?;
                let bytes = imp::encode(&actual)?;
                endpoint
                    .send::<ChromatographyResult>(&bytes.as_slice())
                    .await?;
                instrument = endpoint.recv::<Loan<CHROMATOGRAPH>>().await?;
            }
            ChromaQualityApproved::LOGICAL_LABEL => {
                let result = imp::decode(branch.recv::<ChromaQualityApproved>().await?)?;
                imp::batch_cause(&result, &batch)?;
                let bytes = imp::encode(&result)?;
                endpoint
                    .send::<ChromaQualityReceipt>(&bytes.as_slice())
                    .await?;
            }
            ChromaQualityRejected::LOGICAL_LABEL => {
                let result = imp::decode(branch.recv::<ChromaQualityRejected>().await?)?;
                imp::batch_cause(&result, &batch)?;
                let bytes = imp::encode(&result)?;
                endpoint
                    .send::<ChromaQualityReceipt>(&bytes.as_slice())
                    .await?;
            }
            ChromaReSynthesize::LOGICAL_LABEL => {
                let result = imp::decode(branch.recv::<ChromaReSynthesize>().await?)?;
                imp::batch_cause(&result, &batch)?;
            }
            ChromaFinalApproved::LOGICAL_LABEL => {
                let result = imp::decode(branch.recv::<ChromaFinalApproved>().await?)?;
                imp::batch_cause(&result, &batch)?;
                return Ok(());
            }
            ChromaFinalExhausted::LOGICAL_LABEL => {
                imp::same_value(
                    &branch.recv::<ChromaFinalExhausted>().await?.into_value(),
                    &batch,
                )?;
                return Ok(());
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected chromatography operation {label}"
                )));
            }
        }
    }
}
