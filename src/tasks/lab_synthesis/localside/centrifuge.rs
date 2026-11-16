//! Cell preparation and purification may arrive in either order at the physical owner.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::{Endpoint, g::Message};
use serde_json::{Value, json};
#[hibana_runtime::scope]
pub async fn run<D: Domain>(owner: &mut Endpoint<CENT_OWNER>, domain: &D) -> ShellResult<()> {
    let mut cent = owner.recv::<CentLoan>().await?;
    let mut cells = Value::Null;
    let mut purified = Value::Null;
    for _ in 0..3 {
        let branch = cent.offer().await?;
        match branch.label() {
            CellsOperation::LOGICAL_LABEL => {
                let packet = imp::decode(branch.recv::<CellsOperation>().await?)?;
                let arguments = imp::arguments::<2>(&packet)?;
                let result = domain
                    .execute("BIOLOGIST", "run_separation", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::record(
                    &result,
                    ("run_separation", "BIOLOGIST", "cell_cultures"),
                    &json!({}),
                )?;
                cells = result.clone();
                let bytes = imp::encode(&result)?;
                cent.send::<CellsResult>(&bytes.as_slice()).await?;
            }
            PurifyOperation::LOGICAL_LABEL => {
                let packet = imp::decode(branch.recv::<PurifyOperation>().await?)?;
                let arguments = imp::arguments::<1>(&packet)?;
                let raw = imp::record(
                    &packet["raw"],
                    ("run_analysis", "CHEMIST", "raw_materials"),
                    &json!({}),
                )?;
                let result = domain
                    .execute("CHEMIST", "run_separation", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::record(
                    &result,
                    ("run_separation", "CHEMIST", "compound_purification"),
                    &json!({"raw":raw}),
                )?;
                purified = result.clone();
                let bytes = imp::encode(&result)?;
                cent.send::<PurifiedResult>(&bytes.as_slice()).await?;
            }
            VerificationOperation::LOGICAL_LABEL => {
                let packet = imp::decode(branch.recv::<VerificationOperation>().await?)?;
                let arguments = imp::arguments::<6>(&packet)?;
                let (compound, assay) = imp::assay(&packet["assay"])?;
                if packet["assay"]["cells"] != cells || packet["assay"]["compound"] != purified {
                    return Err(ShellError::message(
                        "verification request substituted actual centrifuge results",
                    ));
                }
                let result = domain
                    .execute("ANALYST", "run_separation", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::record(
                    &result,
                    ("run_separation", "ANALYST", "verification"),
                    &json!({"compound":compound,"bioassay":assay}),
                )?;
                let bytes = imp::encode(&result)?;
                cent.send::<VerificationResult>(&bytes.as_slice()).await?;
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected centrifuge request {label}"
                )));
            }
        }
    }
    owner.send::<CentLoan>(cent).await?;
    Ok(())
}
