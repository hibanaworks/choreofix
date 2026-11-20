use super::super::{global::*, imp};
use crate::shell::ShellResult;
use hibana::Endpoint;
use serde_json::json;

#[hibana_runtime::scope]
pub async fn run<'run>(endpoint: &mut Endpoint<'run, CHEMIST>) -> ShellResult<()> {
    let mut instrument = endpoint.recv::<Loan<HPLC>>().await?;
    instrument.send::<Use<SYNTHESIS_QC>>(&()).await?;
    let qc = imp::decode(instrument.recv::<Reply<SYNTHESIS_QC>>().await?)?;
    endpoint.send::<Loan<HPLC>>(instrument).await?;
    let bytes = imp::encode(&qc)?;
    endpoint
        .send::<Reply<SYNTHESIS_QC>>(&bytes.as_slice())
        .await?;
    let mut instrument = endpoint.recv::<Loan<MASS>>().await?;
    instrument.send::<Use<MOLECULAR_ID>>(&()).await?;
    let identity = imp::decode(instrument.recv::<Reply<MOLECULAR_ID>>().await?)?;
    imp::inputs(&identity, &json!({"qc":imp::artifact(&qc)?}))?;
    endpoint.send::<Loan<MASS>>(instrument).await?;
    let bytes = imp::encode(&identity)?;
    endpoint
        .send::<Reply<MOLECULAR_ID>>(&bytes.as_slice())
        .await?;
    let compound = json!({"qc":qc,"identity":identity});
    let compound_id = imp::compound(&compound)?;
    let bytes = imp::encode(&compound)?;
    async {
        let [mut biology, mut toxicology] = endpoint.par()?;
        async {
            biology.send::<BiologicalSample>(&bytes.as_slice()).await?;
        };
        async {
            toxicology
                .send::<ToxicologySample>(&bytes.as_slice())
                .await?;
        };
        Ok(())
    }
    .await?;

    let decision = imp::decode(endpoint.recv::<DecisionReceipt>().await?)?;
    let decision_id = imp::artifact(&decision)?;
    if decision["data"]["inputs"]["compound"] != compound_id {
        return Err(crate::shell::ShellError::message(
            "decision belongs to different material",
        ));
    }
    let bytes = imp::encode(&decision)?;
    endpoint.send::<DecisionReceipt>(&bytes.as_slice()).await?;
    let mut instrument = endpoint.recv::<Loan<HPLC>>().await?;
    instrument.send::<Use<DOCUMENTATION>>(&()).await?;
    let documentation = imp::decode(instrument.recv::<Reply<DOCUMENTATION>>().await?)?;
    imp::inputs(
        &documentation,
        &json!({"compound":compound_id,"decision":decision_id}),
    )?;
    endpoint.send::<Loan<HPLC>>(instrument).await?;
    let bytes = imp::encode(&documentation)?;
    endpoint
        .send::<Reply<DOCUMENTATION>>(&bytes.as_slice())
        .await?;
    Ok(())
}
