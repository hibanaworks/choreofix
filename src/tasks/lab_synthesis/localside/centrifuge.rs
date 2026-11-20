//! Cell preparation and purification may arrive in either order at the physical owner.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::ReceivedEffects};
use hibana::{Endpoint, g::Message};
use serde_json::{Value, json};
#[hibana_runtime::scope]
pub async fn run(
    owner: &mut Endpoint<CENT_OWNER>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    let mut cent = owner.recv::<CentLoan>().await?;
    let mut cells = Value::Null;
    let mut purified = Value::Null;
    for _ in 0..3 {
        let branch = cent.offer().await?;
        match branch.label() {
            CellsOperation::LOGICAL_LABEL => {
                let received = branch.recv::<CellsOperation>().await?;
                let result = effects.apply(received).await?;
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
                let received = branch.recv::<PurifyOperation>().await?;
                let packet = imp::decode(received.as_bytes())?;
                let raw = imp::record(
                    &packet["raw"],
                    ("run_analysis", "CHEMIST", "raw_materials"),
                    &json!({}),
                )?;
                let result = effects.apply(received).await?;
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
                let received = branch.recv::<VerificationOperation>().await?;
                let packet = imp::decode(received.as_bytes())?;
                let (compound, assay) = imp::assay(&packet["assay"])?;
                if packet["assay"]["cells"] != cells || packet["assay"]["compound"] != purified {
                    return Err(ShellError::message(
                        "verification request substituted actual centrifuge results",
                    ));
                }
                let result = effects.apply(received).await?;
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
