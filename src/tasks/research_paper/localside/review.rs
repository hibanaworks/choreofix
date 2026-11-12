//! Receive a real chapter authority before each actual factual/editorial effect.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::{ParallelEndpoint, g::Message};
use serde_json::Value;

#[hibana_runtime::scope]
pub async fn run<const AUTHOR: u8, const SECTION: u8, C: ChapterMessages<AUTHOR>>(
    endpoint: &mut ParallelEndpoint<REVIEW_SERVICE>,
    domain: &impl Domain,
) -> ShellResult<(Value, Value)> {
    // This owns the actual latest fact-check reply, not a separate phase flag.
    let mut factual_result = None;
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            label if label == C::FactMaterial::LOGICAL_LABEL => {
                let snapshot = branch.recv::<C::FactMaterial>().await?.into_value();
                let arguments = endpoint.recv::<C::FactRequest>().await?.into_value();
                let mut section = endpoint.recv::<Section<SECTION>>().await?;
                let result = domain
                    .execute("FACT_CHECKER", "fact_check", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::checked(AUTHOR, &result, &snapshot)?;
                factual_result = Some(result.clone());
                if result["success"] == true {
                    let result = imp::Verdict::<AUTHOR, 5, true>::new(result)?;
                    section.send::<C::FactPassed>(&result).await?;
                    section.send::<C::FactsForEditor>(&result).await?;
                    endpoint.send::<Section<SECTION>>(section).await?;
                } else {
                    let result = imp::Verdict::<AUTHOR, 5, false>::new(result)?;
                    section.send::<C::FactRejected>(&result).await?;
                    async {
                        let [mut editor, mut returned] = endpoint.par()?;
                        async {
                            editor.send::<C::FailedFactsForEditor>(&result).await?;
                        };
                        async {
                            section.send::<C::FactRejectionForAuthor>(&result).await?;
                            returned.send::<Section<SECTION>>(section).await?;
                        };
                        Ok::<(), ShellError>(())
                    }
                    .await?;
                }
            }
            label if label == C::ReviewMaterial::LOGICAL_LABEL => {
                let material = branch.recv::<C::ReviewMaterial>().await?.into_value();
                let arguments = endpoint.recv::<C::ReviewRequest>().await?.into_value();
                let mut section = endpoint.recv::<Section<SECTION>>().await?;
                let facts = factual_result
                    .as_ref()
                    .ok_or_else(|| ShellError::message("review has no actual factual result"))?;
                imp::same(&material["facts"], facts)?;
                imp::checked(AUTHOR, facts, &material["snapshot"])?;
                if facts["success"] != true {
                    return Err(ShellError::message(
                        "editorial input has no positive factual evidence",
                    ));
                }
                let result = domain
                    .execute("EDITOR", "review_sections", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::reviewed(AUTHOR, &result, (&material["snapshot"], facts))?;
                if result["success"] == true {
                    let actual = result.clone();
                    let result = imp::Verdict::<AUTHOR, 6, true>::new(result)?;
                    section.send::<C::ReviewPassed>(&result).await?;
                    section.send::<C::FinalForAuthor>(&result).await?;
                    section.send::<C::FinalForDocument>(&result).await?;
                    endpoint.send::<Section<SECTION>>(section).await?;
                    endpoint.send::<C::FinalForFact>(&result).await?;
                    return Ok((facts.clone(), actual));
                } else {
                    let result = imp::Verdict::<AUTHOR, 6, false>::new(result)?;
                    section.send::<C::ReviewRejected>(&result).await?;
                    section.send::<C::ReviewRejectionForAuthor>(&result).await?;
                    endpoint.send::<Section<SECTION>>(section).await?;
                }
            }
            _ => return Err(ShellError::message("unexpected paper review request")),
        }
    }
}
