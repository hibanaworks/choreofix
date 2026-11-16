//! Cell preparation and arrival of the real specimen are independent.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::Endpoint;
use serde_json::json;
#[hibana_runtime::scope]
pub async fn run<D: Domain>(biologist: &mut Endpoint<BIOLOGIST>, domain: &D) -> ShellResult<()> {
    let ((cells, calibrated), sample) = async {
        let [mut specimen, mut prepared] = biologist.par()?;
        async {
            let arguments = prepared.recv::<Cells>().await?.into_value();
            let bytes = imp::encode(&json!({"arguments":arguments}))?;
            prepared.send::<CellsOperation>(&bytes.as_slice()).await?;
            let cells = imp::decode(prepared.recv::<CellsResult>().await?)?;
            let cells_id = imp::record(
                &cells,
                ("run_separation", "BIOLOGIST", "cell_cultures"),
                &json!({}),
            )?;
            let bytes = imp::encode(&cells)?;
            prepared.send::<FileCellsResult>(&bytes.as_slice()).await?;
            let arguments = prepared.recv::<Calibrate>().await?.into_value();
            let bytes = imp::encode(&json!({"arguments":arguments,"cells":cells}))?;
            prepared
                .send::<CalibrationOperation>(&bytes.as_slice())
                .await?;
            let calibrated = imp::decode(prepared.recv::<CalibrationResult>().await?)?;
            imp::record(
                &calibrated,
                ("run_analysis", "BIOLOGIST", "fluorescence"),
                &json!({"prepared_cells":cells_id}),
            )?;
            let bytes = imp::encode(&calibrated)?;
            prepared
                .send::<FileCalibrationResult>(&bytes.as_slice())
                .await?;
            Ok((cells, calibrated))
        };
        async { imp::decode(specimen.recv::<Purified>().await?) };
    }
    .await?;
    let compound_id = imp::compound(&sample)?;
    let cells_id = imp::record(
        &cells,
        ("run_separation", "BIOLOGIST", "cell_cultures"),
        &json!({}),
    )?;
    let calibrated_id = imp::record(
        &calibrated,
        ("run_analysis", "BIOLOGIST", "fluorescence"),
        &json!({"prepared_cells":cells_id}),
    )?;
    let bytes = imp::encode(&sample)?;
    biologist.send::<Sample>(&bytes.as_slice()).await?;
    let arguments = biologist.recv::<Bioassay>().await?.into_value();
    let culture = arguments["cell_type"].as_str().unwrap_or("").to_owned();
    let assay = domain
        .execute("BIOLOGIST", "run_bioassay", arguments)
        .await
        .map_err(ShellError::message)?;
    imp::record(
        &assay,
        ("run_bioassay", "BIOLOGIST", "compound_purification"),
        &json!({"compound":compound_id,"prepared_cells":cells_id,"calibration":calibrated_id}),
    )?;
    if assay["data"]["cell_type"] != culture {
        return Err(ShellError::message(
            "assay culture differs from its admitted argument",
        ));
    }
    let bytes = imp::encode(&assay)?;
    biologist.send::<BioassayResult>(&bytes.as_slice()).await?;
    let inputs = json!({"raw":sample["raw"],"compound":sample["compound"],"cells":cells,"calibration":calibrated,"bioassay":assay});
    imp::assay(&inputs)?;
    let bytes = imp::encode(&inputs)?;
    biologist.send::<AssayNotice>(&bytes.as_slice()).await?;
    Ok(())
}
