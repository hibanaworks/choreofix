use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult};
use hibana::{Endpoint, g::Message};
use serde_json::json;
#[hibana_runtime::scope]
pub async fn run<'run>(
    endpoint: &mut Endpoint<'run, CHEMIST>,
    qc_authority: Endpoint<'run, SYNTHESIS_VIEW>,
) -> ShellResult<()> {
    let mut authority = qc_authority;
    let qc = loop {
        let mut instrument = endpoint.recv::<Loan<HPLC>>().await?;
        instrument.send::<Use<SYNTHESIS_QC>>(&()).await?;
        endpoint.send::<Loan<HPLC>>(instrument).await?;
        endpoint.send::<Loan<SYNTHESIS_VIEW>>(authority).await?;
        let branch = endpoint.offer().await?;
        let result = match branch.label() {
            Passed::LOGICAL_LABEL => imp::decode(branch.recv::<Passed>().await?)?,
            Failed::<ANALYSIS_FAILED>::LOGICAL_LABEL => {
                imp::decode(branch.recv::<Failed<ANALYSIS_FAILED>>().await?)?
            }
            label => return Err(ShellError::message(format!("unexpected qc result {label}"))),
        };

        let bytes = imp::encode(&result)?;
        endpoint
            .send::<Reply<SYNTHESIS_QC>>(&bytes.as_slice())
            .await?;
        if result["success"] == true {
            break result;
        }
        authority = endpoint.recv::<Loan<SYNTHESIS_VIEW>>().await?;
    };

    let mut mass = endpoint.recv::<Loan<MASS>>().await?;
    mass.send::<Use<MOLECULAR_ID>>(&()).await?;
    let identity = imp::decode(mass.recv::<Reply<MOLECULAR_ID>>().await?)?;
    endpoint.send::<Loan<MASS>>(mass).await?;
    let bytes = imp::encode(&identity)?;
    endpoint
        .send::<Reply<MOLECULAR_ID>>(&bytes.as_slice())
        .await?;
    let material = json!({"qc":qc,"identity":identity});
    let compound = imp::compound(&material)?;
    let bytes = imp::encode(&material)?;
    async {
        let [mut bio, mut tox] = endpoint.par()?;
        async {
            bio.send::<BiologicalSample>(&bytes.as_slice()).await?;
        };
        async {
            tox.send::<ToxicologySample>(&bytes.as_slice()).await?;
        };
        Ok(())
    }
    .await?;
    let approval = imp::decode(endpoint.recv::<ApprovedForChemist>().await?)?;
    let regulatory = imp::artifact(&approval)?;
    let decision = approval["data"]["inputs"]["decision"]
        .as_u64()
        .ok_or_else(|| ShellError::message("approval lacks a decision"))?;
    imp::inputs(&approval, &json!({"compound":compound,"decision":decision}))?;
    let bytes = imp::encode(&approval)?;
    endpoint.send::<ApprovalNotice>(&bytes.as_slice()).await?;
    let mut study = endpoint.recv::<Loan<STUDY>>().await?;
    loop {
        study.send::<ApprovalNotice>(&bytes.as_slice()).await?;
        let mut hplc = endpoint.recv::<Loan<HPLC>>().await?;
        hplc.send::<Use<DOCUMENTATION>>(&()).await?;
        endpoint.send::<Loan<HPLC>>(hplc).await?;
        endpoint.send::<Loan<STUDY>>(study).await?;
        let branch = endpoint.offer().await?;
        let receipt = match branch.label() {
            Passed::LOGICAL_LABEL => imp::decode(branch.recv::<Passed>().await?)?,
            DocumentationFailed::LOGICAL_LABEL => {
                imp::decode(branch.recv::<DocumentationFailed>().await?)?
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected documentation result {label}"
                )));
            }
        };
        imp::inputs(
            &receipt,
            &json!({"compound":compound,"decision":decision,"regulatory":regulatory}),
        )?;
        let bytes = imp::encode(&receipt)?;
        endpoint
            .send::<Reply<DOCUMENTATION>>(&bytes.as_slice())
            .await?;
        if receipt["success"] == true {
            return Ok(());
        }
        study = endpoint.recv::<Loan<STUDY>>().await?;
    }
}
