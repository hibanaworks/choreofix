//! The actual spectrum input, instrument loan, effect and return stay together.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::{Endpoint, g::Message};
use serde_json::Value;

#[hibana_runtime::scope]
pub async fn run<D: Domain>(endpoint: &mut Endpoint<SPEC_OWNER>, domain: &D) -> ShellResult<()> {
    let mut instrument = endpoint.recv::<Loan<SPECTROMETER>>().await?;
    let mut batch = Value::Null;
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            Calibration::LOGICAL_LABEL => {
                let admitted = imp::decode(branch.recv::<Calibration>().await?)?;
                imp::instrument_inputs::<4>(&admitted)?;
                endpoint.send::<Loan<SPECTROMETER>>(instrument).await?;
                let received = imp::decode(endpoint.recv::<Calibration>().await?)?;
                imp::same_value(&received, &admitted)?;
                let actual = domain
                    .execute("BIOLOGIST", "run_analysis", admitted["arguments"].clone())
                    .await
                    .map_err(ShellError::message)?;
                imp::instrument_result::<4>(&actual, &admitted)?;
                let bytes = imp::encode(&actual)?;
                endpoint
                    .send::<CalibrationResult>(&bytes.as_slice())
                    .await?;
                instrument = endpoint.recv::<Loan<SPECTROMETER>>().await?;
            }
            Analysis::LOGICAL_LABEL => {
                let admitted = imp::decode(branch.recv::<Analysis>().await?)?;
                imp::same_value(&admitted["material"]["synthesis"], &batch)?;
                imp::instrument_inputs::<1>(&admitted)?;
                endpoint.send::<Loan<SPECTROMETER>>(instrument).await?;
                let received = imp::decode(endpoint.recv::<Analysis>().await?)?;
                imp::same_value(&received, &admitted)?;
                let actual = domain
                    .execute("CHEMIST", "run_analysis", admitted["arguments"].clone())
                    .await
                    .map_err(ShellError::message)?;
                imp::instrument_result::<1>(&actual, &admitted)?;
                let bytes = imp::encode(&actual)?;
                endpoint.send::<AnalysisResult>(&bytes.as_slice()).await?;
                instrument = endpoint.recv::<Loan<SPECTROMETER>>().await?;
            }
            Molecular::LOGICAL_LABEL => {
                let admitted = imp::decode(branch.recv::<Molecular>().await?)?;
                imp::same_value(&admitted["material"]["synthesis"], &batch)?;
                imp::instrument_inputs::<7>(&admitted)?;
                endpoint.send::<Loan<SPECTROMETER>>(instrument).await?;
                let received = imp::decode(endpoint.recv::<Molecular>().await?)?;
                imp::same_value(&received, &admitted)?;
                let actual = domain
                    .execute("ANALYST", "run_analysis", admitted["arguments"].clone())
                    .await
                    .map_err(ShellError::message)?;
                imp::instrument_result::<7>(&actual, &admitted)?;
                let bytes = imp::encode(&actual)?;
                endpoint.send::<MolecularResult>(&bytes.as_slice()).await?;
                instrument = endpoint.recv::<Loan<SPECTROMETER>>().await?;
            }
            SpecQualityAdmission::LOGICAL_LABEL => {
                let admitted = imp::decode(branch.recv::<SpecQualityAdmission>().await?)?;
                imp::same_value(&admitted["material"]["synthesis"], &batch)?;
                imp::quality_input(&admitted)?;
                endpoint.send::<Loan<SPECTROMETER>>(instrument).await?;
                instrument = endpoint.recv::<Loan<SPECTROMETER>>().await?;
            }
            SpecInspection::LOGICAL_LABEL => {
                branch.recv::<SpecInspection>().await?;
                endpoint.send::<Loan<SPECTROMETER>>(instrument).await?;
                instrument = endpoint.recv::<Loan<SPECTROMETER>>().await?;
            }
            SpecBatch::LOGICAL_LABEL => batch = branch.recv::<SpecBatch>().await?.into_value(),
            SpecSynthesisFailed::LOGICAL_LABEL => {
                batch = imp::decode(branch.recv::<SpecSynthesisFailed>().await?)?
            }
            SpecReagentsExhausted::LOGICAL_LABEL => {
                batch = branch.recv::<SpecReagentsExhausted>().await?.into_value()
            }
            SpecReSynthesize::LOGICAL_LABEL => {
                let result = imp::decode(branch.recv::<SpecReSynthesize>().await?)?;
                imp::batch_cause(&result, &batch)?;
            }
            SpecFinalExhausted::LOGICAL_LABEL => {
                imp::same_value(
                    &branch.recv::<SpecFinalExhausted>().await?.into_value(),
                    &batch,
                )?;
                return Ok(());
            }
            SpecFinalApproved::LOGICAL_LABEL => {
                let result = imp::decode(branch.recv::<SpecFinalApproved>().await?)?;
                imp::batch_cause(&result, &batch)?;
                let admitted = imp::decode(endpoint.recv::<FinalValidation>().await?)?;
                imp::same_value(&admitted["report"]["material"]["synthesis"], &batch)?;
                imp::same_value(&admitted["report"]["quality"], &result)?;
                imp::instrument_inputs::<10>(&admitted)?;
                endpoint.send::<Loan<SPECTROMETER>>(instrument).await?;
                let received = imp::decode(endpoint.recv::<FinalValidation>().await?)?;
                imp::same_value(&received, &admitted)?;
                let actual = domain
                    .execute("CHEMIST", "run_analysis", admitted["arguments"].clone())
                    .await
                    .map_err(ShellError::message)?;
                imp::instrument_result::<10>(&actual, &admitted)?;
                let bytes = imp::encode(&actual)?;
                endpoint.send::<FinalResult>(&bytes.as_slice()).await?;
                let instrument = endpoint.recv::<Loan<SPECTROMETER>>().await?;
                drop(instrument);
                return Ok(());
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected spectrometer operation {label}"
                )));
            }
        }
    }
}
