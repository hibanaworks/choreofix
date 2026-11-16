//! Actual approved materials, independent instrument loans and the quality report.
use super::super::{global::*, imp};
use crate::shell::ShellResult;
use hibana::Endpoint;
use serde_json::json;

#[hibana_runtime::scope]
pub async fn run(analyst: &mut Endpoint<ANALYST>) -> ShellResult<()> {
    let approved = imp::decode(analyst.recv::<ApprovedBiology>().await?)?;
    let inputs = imp::approved_inputs(&approved)?;
    let bytes = imp::encode(&approved)?;
    analyst.send::<FileAssay>(&bytes.as_slice()).await?;
    let (molecular, verification) = async {
        let [mut molecular, mut verification] = analyst.par()?;
        async {
            let arguments = molecular.recv::<FileMolecular>().await?.into_value();
            let bytes = imp::encode(&json!({"arguments":arguments,"approved":approved}))?;
            molecular.send::<Molecular>(&bytes.as_slice()).await?;
            let mut spec = molecular.recv::<SpecLoan>().await?;
            spec.send::<SpecRequest>(&bytes.as_slice()).await?;
            let result = imp::decode(spec.recv::<InstrumentResult>().await?)?;
            imp::record(&result, ("run_analysis", "ANALYST", &arguments), &inputs)?;
            molecular.send::<SpecLoan>(spec).await?;
            let bytes = imp::encode(&result)?;
            molecular
                .send::<FileMolecularResult>(&bytes.as_slice())
                .await?;
            Ok(result)
        };
        async {
            let arguments = verification.recv::<FileVerification>().await?.into_value();
            let bytes = imp::encode(&json!({"arguments":arguments,"approved":approved}))?;
            verification.send::<Separation>(&bytes.as_slice()).await?;
            let mut cent = verification.recv::<CentLoan>().await?;
            cent.send::<CentRequest>(&bytes.as_slice()).await?;
            let result = imp::decode(cent.recv::<InstrumentResult>().await?)?;
            imp::record(&result, ("run_separation", "ANALYST", &arguments), &inputs)?;
            verification.send::<CentLoan>(cent).await?;
            let bytes = imp::encode(&result)?;
            verification
                .send::<FileVerificationResult>(&bytes.as_slice())
                .await?;
            Ok(result)
        };
    }
    .await?;
    let mut quality = approved;
    quality["molecular"] = molecular;
    quality["verification"] = verification;
    imp::final_inputs(&quality)?;
    let bytes = imp::encode(&quality)?;
    async {
        let [mut scientist, mut instrument] = analyst.par()?;
        async {
            scientist.send::<Quality>(&bytes.as_slice()).await?;
        };
        async {
            instrument
                .send::<QualityForOwner>(&bytes.as_slice())
                .await?;
        };
        Ok(())
    }
    .await?;
    Ok(())
}
