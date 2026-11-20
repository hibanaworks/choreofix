use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult};
use hibana::Endpoint;
use serde_json::json;

pub async fn run<'run>(endpoint: &mut Endpoint<'run, TOXICOLOGIST>) -> ShellResult<()> {
    let mut instrument = endpoint.recv::<Loan<MASS>>().await?;
    instrument.send::<Use<BASELINE>>(&()).await?;
    let baseline = imp::decode(instrument.recv::<Reply<BASELINE>>().await?)?;
    endpoint.send::<Loan<MASS>>(instrument).await?;
    let bytes = imp::encode(&baseline)?;
    endpoint.send::<Reply<BASELINE>>(&bytes.as_slice()).await?;

    let compound = imp::decode(endpoint.recv::<ToxicologySample>().await?)?;
    let compound_id = imp::compound(&compound)?;
    let bytes = imp::encode(&compound)?;
    endpoint.send::<ToxicologySample>(&bytes.as_slice()).await?;
    let mut instrument = endpoint.recv::<Loan<HPLC>>().await?;
    instrument.send::<Use<PURITY>>(&()).await?;
    let purity = imp::decode(instrument.recv::<Reply<PURITY>>().await?)?;
    imp::inputs(&purity, &json!({"compound":compound_id}))?;
    endpoint.send::<Loan<HPLC>>(instrument).await?;
    let bytes = imp::encode(&purity)?;
    endpoint.send::<Reply<PURITY>>(&bytes.as_slice()).await?;

    let mut instrument = endpoint.recv::<Loan<CELL>>().await?;
    instrument.send::<Use<CYTOTOXICITY>>(&()).await?;
    let toxicity = imp::decode(instrument.recv::<Reply<CYTOTOXICITY>>().await?)?;
    let cells = toxicity["data"]["inputs"]["prepared_cells"]
        .as_u64()
        .ok_or_else(|| ShellError::message("toxicity lacks prepared-cell evidence"))?;
    imp::inputs(
        &toxicity,
        &json!({
            "compound":compound_id,"prepared_cells":cells,
            "baseline":imp::artifact(&baseline)?,"purity":imp::artifact(&purity)?
        }),
    )?;
    endpoint.send::<Loan<CELL>>(instrument).await?;
    let bytes = imp::encode(&toxicity)?;
    endpoint
        .send::<Reply<CYTOTOXICITY>>(&bytes.as_slice())
        .await?;

    let report =
        json!({"compound":compound,"baseline":baseline,"purity":purity,"toxicity":toxicity});
    let bytes = imp::encode(&report)?;
    endpoint.send::<ToxReport>(&bytes.as_slice()).await?;
    Ok(())
}
