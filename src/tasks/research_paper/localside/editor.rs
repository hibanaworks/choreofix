use super::super::{global::*, imp::Material};
use crate::shell::{ShellError, ShellResult};
use hibana::{EndpointBorrow, g::Message};
use serde_json::json;

pub async fn run<const AUTHOR: u8, const SECTION: u8, C: ChapterMessages<AUTHOR>>(
    endpoint: &mut EndpointBorrow<'_, '_, EDITOR>,
) -> ShellResult<()> {
    loop {
        let branch = endpoint.offer().await?;
        let facts = match branch.label() {
            label if label == C::FactsForEditor::LOGICAL_LABEL => {
                branch.recv::<C::FactsForEditor>().await?.into_value()
            }
            label if label == C::FailedFactsForEditor::LOGICAL_LABEL => {
                branch.recv::<C::FailedFactsForEditor>().await?;
                continue;
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected fact evidence {label}"
                )));
            }
        };
        let section = endpoint.recv::<Section<SECTION>>().await?;
        let material =
            Material::new(json!({"snapshot":facts["data"]["input_snapshot"],"facts":facts}))?;
        endpoint.send::<C::ReviewInput>(&material).await?;
        endpoint.send::<C::ReviewMaterial>(&material).await?;
        endpoint.send::<Section<SECTION>>(section).await?;
        let branch = endpoint.offer().await?;
        match branch.label() {
            label if label == C::ReviewPassed::LOGICAL_LABEL => {
                branch.recv::<C::ReviewPassed>().await?;
                return Ok(());
            }
            label if label == C::ReviewRejected::LOGICAL_LABEL => {
                branch.recv::<C::ReviewRejected>().await?;
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected editorial result {label}"
                )));
            }
        }
    }
}
