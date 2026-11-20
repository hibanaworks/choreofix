use super::super::{global::*, imp};
use crate::shell::ShellResult;
use hibana::Endpoint;
use serde_json::json;

pub async fn run<'run>(endpoint: &mut Endpoint<'run, BIOLOGIST>) -> ShellResult<()> {
    let mut cell = endpoint.recv::<Loan<CELL>>().await?;
    cell.send::<Use<CULTURE_PREP>>(&()).await?;
    let cells = imp::decode(cell.recv::<Reply<CULTURE_PREP>>().await?)?;
    let cells_id = imp::artifact(&cells)?;
    let bytes = imp::encode(&cells)?;
    endpoint
        .send::<Reply<CULTURE_PREP>>(&bytes.as_slice())
        .await?;

    let compound = imp::decode(endpoint.recv::<BiologicalSample>().await?)?;
    let compound_id = imp::compound(&compound)?;
    let bytes = imp::encode(&compound)?;
    endpoint.send::<BiologicalSample>(&bytes.as_slice()).await?;
    cell.send::<Use<BIOASSAY>>(&()).await?;
    let assay = imp::decode(cell.recv::<Reply<BIOASSAY>>().await?)?;
    imp::inputs(
        &assay,
        &json!({"compound":compound_id,"prepared_cells":cells_id}),
    )?;
    let assay_id = imp::artifact(&assay)?;
    let bytes = imp::encode(&assay)?;
    endpoint.send::<Reply<BIOASSAY>>(&bytes.as_slice()).await?;

    let mut instrument = endpoint.recv::<Loan<HPLC>>().await?;
    instrument.send::<Use<STABILITY>>(&()).await?;
    let stability = imp::decode(instrument.recv::<Reply<STABILITY>>().await?)?;
    imp::inputs(
        &stability,
        &json!({"compound":compound_id,"medium":assay_id}),
    )?;
    endpoint.send::<Loan<HPLC>>(instrument).await?;
    let bytes = imp::encode(&stability)?;
    endpoint.send::<Reply<STABILITY>>(&bytes.as_slice()).await?;

    endpoint.send::<Loan<CELL>>(cell).await?;
    let report = json!({"compound":compound,"cells":cells,"assay":assay,"stability":stability});
    let bytes = imp::encode(&report)?;
    endpoint.send::<BioReport>(&bytes.as_slice()).await?;
    Ok(())
}
