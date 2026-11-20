//! Actual file requests, specimen data and resource requests remain in one localside.
use super::super::arguments::InstrumentRequest;
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult};
use hibana::Endpoint;
use serde_json::json;
#[hibana_runtime::scope]
pub async fn run(chemist: &mut Endpoint<CHEMIST>) -> ShellResult<()> {
    let arguments = imp::decode(chemist.recv::<Raw>().await?.as_bytes())?;
    let bytes = imp::encode(&json!({"arguments":arguments}))?;
    chemist
        .send::<RawOperation>(&InstrumentRequest(&bytes))
        .await?;
    let raw = imp::decode(chemist.recv::<RawResult>().await?)?;
    let raw_id = imp::record(
        &raw,
        ("run_analysis", "CHEMIST", "raw_materials"),
        &json!({}),
    )?;
    let bytes = imp::encode(&raw)?;
    chemist.send::<FileRawResult>(&bytes.as_slice()).await?;
    let arguments = imp::decode(chemist.recv::<Purify>().await?.as_bytes())?;
    let bytes = imp::encode(&json!({"arguments":arguments,"raw":raw}))?;
    chemist
        .send::<PurifyOperation>(&InstrumentRequest(&bytes))
        .await?;
    let purified = imp::decode(chemist.recv::<PurifiedResult>().await?)?;
    imp::record(
        &purified,
        ("run_separation", "CHEMIST", "compound_purification"),
        &json!({"raw":raw_id}),
    )?;
    let bytes = imp::encode(&purified)?;
    chemist
        .send::<FilePurifiedResult>(&bytes.as_slice())
        .await?;
    let specimen = json!({"raw":raw,"compound":purified});
    let bytes = imp::encode(&specimen)?;
    chemist.send::<Purified>(&bytes.as_slice()).await?;
    let quality = imp::decode(chemist.recv::<Quality>().await?)?;
    let inputs = imp::quality(&quality)?;
    if quality["assay"]["raw"] != specimen["raw"]
        || quality["assay"]["compound"] != specimen["compound"]
    {
        return Err(ShellError::message(
            "quality report describes another actual specimen",
        ));
    }
    let bytes = imp::encode(&quality)?;
    chemist.send::<FileQuality>(&bytes.as_slice()).await?;
    let arguments = imp::decode(chemist.recv::<Final>().await?.as_bytes())?;
    let bytes = imp::encode(&json!({"arguments":arguments,"quality":quality}))?;
    chemist
        .send::<FinalOperation>(&InstrumentRequest(&bytes))
        .await?;
    let result = imp::decode(chemist.recv::<FinalResult>().await?)?;
    imp::record(
        &result,
        ("run_analysis", "CHEMIST", "final_validation"),
        &inputs,
    )?;
    let bytes = imp::encode(&result)?;
    chemist.send::<FileFinalResult>(&bytes.as_slice()).await?;
    Ok(())
}
