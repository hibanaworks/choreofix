//! Physical centrifuge effects and the actual biological assay receiver.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::{Endpoint, g::Message};
use serde_json::Value;

#[hibana_runtime::scope]
pub async fn run<D: Domain>(endpoint: &mut Endpoint<CENT_OWNER>, domain: &D) -> ShellResult<()> {
    let mut instrument = endpoint.recv::<Loan<CENTRIFUGE>>().await?;
    let mut batch = Value::Null;
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            Cells::LOGICAL_LABEL => {
                let admitted = imp::decode(branch.recv::<Cells>().await?)?;
                imp::instrument_inputs::<3>(&admitted)?;
                endpoint.send::<Loan<CENTRIFUGE>>(instrument).await?;
                let received = imp::decode(endpoint.recv::<Cells>().await?)?;
                imp::same_value(&received, &admitted)?;
                let actual = domain
                    .execute("BIOLOGIST", "run_separation", admitted["arguments"].clone())
                    .await
                    .map_err(ShellError::message)?;
                imp::instrument_result::<3>(&actual, &admitted)?;
                let bytes = imp::encode(&actual)?;
                endpoint.send::<CellsResult>(&bytes.as_slice()).await?;
                instrument = endpoint.recv::<Loan<CENTRIFUGE>>().await?;
            }
            Purification::LOGICAL_LABEL => {
                let admitted = imp::decode(branch.recv::<Purification>().await?)?;
                imp::same_value(&admitted["material"]["synthesis"], &batch)?;
                imp::instrument_inputs::<2>(&admitted)?;
                endpoint.send::<Loan<CENTRIFUGE>>(instrument).await?;
                let received = imp::decode(endpoint.recv::<Purification>().await?)?;
                imp::same_value(&received, &admitted)?;
                let actual = domain
                    .execute("CHEMIST", "run_separation", admitted["arguments"].clone())
                    .await
                    .map_err(ShellError::message)?;
                imp::instrument_result::<2>(&actual, &admitted)?;
                let bytes = imp::encode(&actual)?;
                endpoint
                    .send::<PurificationResult>(&bytes.as_slice())
                    .await?;
                instrument = endpoint.recv::<Loan<CENTRIFUGE>>().await?;
            }
            Verification::LOGICAL_LABEL => {
                let admitted = imp::decode(branch.recv::<Verification>().await?)?;
                imp::same_value(&admitted["material"]["synthesis"], &batch)?;
                imp::instrument_inputs::<8>(&admitted)?;
                endpoint.send::<Loan<CENTRIFUGE>>(instrument).await?;
                let received = imp::decode(endpoint.recv::<Verification>().await?)?;
                imp::same_value(&received, &admitted)?;
                let actual = domain
                    .execute("ANALYST", "run_separation", admitted["arguments"].clone())
                    .await
                    .map_err(ShellError::message)?;
                imp::instrument_result::<8>(&actual, &admitted)?;
                let bytes = imp::encode(&actual)?;
                endpoint
                    .send::<VerificationResult>(&bytes.as_slice())
                    .await?;
                instrument = endpoint.recv::<Loan<CENTRIFUGE>>().await?;
            }
            Assay::LOGICAL_LABEL => {
                let admitted = imp::decode(branch.recv::<Assay>().await?)?;
                imp::same_value(&admitted["material"]["synthesis"], &batch)?;
                imp::instrument_inputs::<5>(&admitted)?;
                let actual = domain
                    .execute("BIOLOGIST", "run_bioassay", admitted["arguments"].clone())
                    .await
                    .map_err(ShellError::message)?;
                imp::instrument_result::<5>(&actual, &admitted)?;
                let bytes = imp::encode(&actual)?;
                endpoint.send::<AssayResult>(&bytes.as_slice()).await?;
            }
            CentInspection::LOGICAL_LABEL => {
                branch.recv::<CentInspection>().await?;
                endpoint.send::<Loan<CENTRIFUGE>>(instrument).await?;
                instrument = endpoint.recv::<Loan<CENTRIFUGE>>().await?;
            }
            CentBatch::LOGICAL_LABEL => batch = branch.recv::<CentBatch>().await?.into_value(),
            CentSynthesisFailed::LOGICAL_LABEL => {
                batch = imp::decode(branch.recv::<CentSynthesisFailed>().await?)?
            }
            CentReagentsExhausted::LOGICAL_LABEL => {
                batch = branch.recv::<CentReagentsExhausted>().await?.into_value()
            }
            CentReSynthesize::LOGICAL_LABEL => {
                let result = imp::decode(branch.recv::<CentReSynthesize>().await?)?;
                imp::batch_cause(&result, &batch)?;
            }
            CentFinalApproved::LOGICAL_LABEL => {
                let result = imp::decode(branch.recv::<CentFinalApproved>().await?)?;
                imp::batch_cause(&result, &batch)?;
                return Ok(());
            }
            CentFinalExhausted::LOGICAL_LABEL => {
                imp::same_value(&branch.recv::<CentFinalExhausted>().await?.into_value(), &batch)?;
                return Ok(());
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected centrifuge operation {label}"
                )));
            }
        }
    }
}
