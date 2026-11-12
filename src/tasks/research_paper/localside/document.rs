//! The unique document Endpoint and its real material stay with this I/O owner.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::{Endpoint, g::Message};
use serde_json::{Value, json};

#[hibana_runtime::scope]
pub async fn run(mut document: Endpoint<DOCUMENT>, domain: &impl Domain) -> ShellResult<()> {
    // Actual received research/material only; neither is an owner or progress map.
    let mut research: [Option<Value>; 3] = [None, None, None];
    let mut sections = json!({});
    loop {
        let branch = document.offer().await?;
        match branch.label() {
            label if label == a::ResearchForDocument::LOGICAL_LABEL => {
                let actual = branch.recv::<a::ResearchForDocument>().await?.into_value();
                imp::research(AUTHOR_A, &actual)?;
                research[0] = Some(actual);
            }

            label if label == a::Write::LOGICAL_LABEL => {
                let arguments = branch.recv::<a::Write>().await?.into_value();
                let researched = research[0].as_ref().ok_or_else(|| {
                    ShellError::message("document request has no received research")
                })?;
                let result = domain
                    .execute(imp::AUTHORS[0], "write_section", arguments.clone())
                    .await
                    .map_err(ShellError::message)?;
                imp::material(AUTHOR_A, 2, &result, (&arguments, researched))?;
                sections[imp::SECTIONS[0]] = result["data"]["artifact"].clone();
                let result = imp::Reply::<AUTHOR_A, 2>::new(result)?;
                document.send::<a::Written>(&result).await?;
                document.send::<a::MaterialForFact>(&result).await?;
            }

            label if label == a::Revise::LOGICAL_LABEL => {
                let arguments = branch.recv::<a::Revise>().await?.into_value();
                let researched = research[0].as_ref().ok_or_else(|| {
                    ShellError::message("document request has no received research")
                })?;
                let result = domain
                    .execute(imp::AUTHORS[0], "revise_section", arguments.clone())
                    .await
                    .map_err(ShellError::message)?;
                imp::material(AUTHOR_A, 3, &result, (&arguments, researched))?;
                sections[imp::SECTIONS[0]] = result["data"]["artifact"].clone();
                let result = imp::Reply::<AUTHOR_A, 3>::new(result)?;
                document.send::<a::Revised>(&result).await?;
                document.send::<a::RevisedForFact>(&result).await?;
            }

            label if label == a::FinalForDocument::LOGICAL_LABEL => {
                let reviewed = branch.recv::<a::FinalForDocument>().await?.into_value();
                imp::same(
                    &reviewed["data"]["input_snapshot"]["sections"][imp::SECTIONS[0]],
                    &sections[imp::SECTIONS[0]],
                )?;
            }

            label if label == b::ResearchForDocument::LOGICAL_LABEL => {
                let actual = branch.recv::<b::ResearchForDocument>().await?.into_value();
                imp::research(AUTHOR_B, &actual)?;
                research[1] = Some(actual);
            }

            label if label == b::Write::LOGICAL_LABEL => {
                let arguments = branch.recv::<b::Write>().await?.into_value();
                let researched = research[1].as_ref().ok_or_else(|| {
                    ShellError::message("document request has no received research")
                })?;
                let result = domain
                    .execute(imp::AUTHORS[1], "write_section", arguments.clone())
                    .await
                    .map_err(ShellError::message)?;
                imp::material(AUTHOR_B, 2, &result, (&arguments, researched))?;
                sections[imp::SECTIONS[1]] = result["data"]["artifact"].clone();
                let result = imp::Reply::<AUTHOR_B, 2>::new(result)?;
                document.send::<b::Written>(&result).await?;
                document.send::<b::MaterialForFact>(&result).await?;
            }

            label if label == b::Revise::LOGICAL_LABEL => {
                let arguments = branch.recv::<b::Revise>().await?.into_value();
                let researched = research[1].as_ref().ok_or_else(|| {
                    ShellError::message("document request has no received research")
                })?;
                let result = domain
                    .execute(imp::AUTHORS[1], "revise_section", arguments.clone())
                    .await
                    .map_err(ShellError::message)?;
                imp::material(AUTHOR_B, 3, &result, (&arguments, researched))?;
                sections[imp::SECTIONS[1]] = result["data"]["artifact"].clone();
                let result = imp::Reply::<AUTHOR_B, 3>::new(result)?;
                document.send::<b::Revised>(&result).await?;
                document.send::<b::RevisedForFact>(&result).await?;
            }

            label if label == b::FinalForDocument::LOGICAL_LABEL => {
                let reviewed = branch.recv::<b::FinalForDocument>().await?.into_value();
                imp::same(
                    &reviewed["data"]["input_snapshot"]["sections"][imp::SECTIONS[1]],
                    &sections[imp::SECTIONS[1]],
                )?;
            }

            label if label == c::ResearchForDocument::LOGICAL_LABEL => {
                let actual = branch.recv::<c::ResearchForDocument>().await?.into_value();
                imp::research(AUTHOR_C, &actual)?;
                research[2] = Some(actual);
            }

            label if label == c::Write::LOGICAL_LABEL => {
                let arguments = branch.recv::<c::Write>().await?.into_value();
                let researched = research[2].as_ref().ok_or_else(|| {
                    ShellError::message("document request has no received research")
                })?;
                let result = domain
                    .execute(imp::AUTHORS[2], "write_section", arguments.clone())
                    .await
                    .map_err(ShellError::message)?;
                imp::material(AUTHOR_C, 2, &result, (&arguments, researched))?;
                sections[imp::SECTIONS[2]] = result["data"]["artifact"].clone();
                let result = imp::Reply::<AUTHOR_C, 2>::new(result)?;
                document.send::<c::Written>(&result).await?;
                document.send::<c::MaterialForFact>(&result).await?;
            }

            label if label == c::Revise::LOGICAL_LABEL => {
                let arguments = branch.recv::<c::Revise>().await?.into_value();
                let researched = research[2].as_ref().ok_or_else(|| {
                    ShellError::message("document request has no received research")
                })?;
                let result = domain
                    .execute(imp::AUTHORS[2], "revise_section", arguments.clone())
                    .await
                    .map_err(ShellError::message)?;
                imp::material(AUTHOR_C, 3, &result, (&arguments, researched))?;
                sections[imp::SECTIONS[2]] = result["data"]["artifact"].clone();
                let result = imp::Reply::<AUTHOR_C, 3>::new(result)?;
                document.send::<c::Revised>(&result).await?;
                document.send::<c::RevisedForFact>(&result).await?;
            }

            label if label == c::FinalForDocument::LOGICAL_LABEL => {
                let reviewed = branch.recv::<c::FinalForDocument>().await?.into_value();
                imp::same(
                    &reviewed["data"]["input_snapshot"]["sections"][imp::SECTIONS[2]],
                    &sections[imp::SECTIONS[2]],
                )?;
            }

            Combine::LOGICAL_LABEL => {
                let admitted = branch.recv::<Combine>().await?.into_value();
                let (_a, _b, _c, references, reviews) = async {
                    let [mut a, mut b, mut c, mut references, mut reviews] = document.par()?;
                    async { Ok::<_, ShellError>(a.recv::<Section<SECTION_A>>().await?) };
                    async { Ok::<_, ShellError>(b.recv::<Section<SECTION_B>>().await?) };
                    async { Ok::<_, ShellError>(c.recv::<Section<SECTION_C>>().await?) };
                    async {
                        references.send::<ReferencesForPublication>(&()).await?;
                        imp::decode(references.recv::<FrozenReferences>().await?)
                    };
                    async { imp::decode(reviews.recv::<FrozenReviews>().await?) };
                }
                .await?;
                let snapshot = imp::publication(&admitted, (&sections, &references), &reviews)?;
                let actual = domain
                    .execute("EDITOR", "combine_sections", admitted.clone())
                    .await
                    .map_err(ShellError::message)?;
                imp::published(&actual, (&admitted, &snapshot), &reviews)?;
                let bytes = imp::encode(&actual)?;
                async {
                    let [mut editor, mut database] = document.par()?;
                    async {
                        editor.send::<Published>(&bytes.as_slice()).await?;
                    };
                    async {
                        database
                            .send::<PublishedForDatabase>(&bytes.as_slice())
                            .await?;
                    };
                    Ok::<(), ShellError>(())
                }
                .await?;
                return Ok(());
            }
            _ => return Err(ShellError::message("unexpected document operation")),
        }
    }
}
