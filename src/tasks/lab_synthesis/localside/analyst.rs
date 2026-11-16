//! Independent measurements use the same actual assay and different instrument Endpoints.
use super::super::{global::*, imp};
use crate::shell::ShellResult;
use hibana::Endpoint;
use serde_json::json;
#[hibana_runtime::scope]
pub async fn run(analyst: &mut Endpoint<ANALYST>) -> ShellResult<()> {
    let assay = imp::decode(analyst.recv::<AssayNotice>().await?)?;
    let (compound_id, assay_id) = imp::assay(&assay)?;
    let inputs = json!({"compound":compound_id,"bioassay":assay_id});
    let bytes = imp::encode(&assay)?;
    analyst.send::<FileAssay>(&bytes.as_slice()).await?;
    let (molecular, verification) = async {
        let [mut molecular, mut verification] = analyst.par()?;
        async {
            let arguments = molecular.recv::<Molecular>().await?.into_value();
            let bytes = imp::encode(&json!({"arguments":arguments,"assay":assay}))?;
            molecular
                .send::<MolecularOperation>(&bytes.as_slice())
                .await?;
            let result = imp::decode(molecular.recv::<MolecularResult>().await?)?;
            imp::record(
                &result,
                ("run_analysis", "ANALYST", "molecular_detail"),
                &inputs,
            )?;
            let bytes = imp::encode(&result)?;
            molecular
                .send::<FileMolecularResult>(&bytes.as_slice())
                .await?;
            Ok(result)
        };
        async {
            let arguments = verification.recv::<Verify>().await?.into_value();
            let bytes = imp::encode(&json!({"arguments":arguments,"assay":assay}))?;
            verification
                .send::<VerificationOperation>(&bytes.as_slice())
                .await?;
            let result = imp::decode(verification.recv::<VerificationResult>().await?)?;
            imp::record(
                &result,
                ("run_separation", "ANALYST", "verification"),
                &inputs,
            )?;
            let bytes = imp::encode(&result)?;
            verification
                .send::<FileVerificationResult>(&bytes.as_slice())
                .await?;
            Ok(result)
        };
    }
    .await?;
    let quality = json!({"assay":assay,"molecular":molecular,"verification":verification});
    imp::quality(&quality)?;
    let bytes = imp::encode(&quality)?;
    analyst.send::<Quality>(&bytes.as_slice()).await?;
    Ok(())
}
