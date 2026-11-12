//! One database Endpoint serializes its actual physical updates, not other work.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::{Endpoint, g::Message};
use serde_json::{Value, json};

#[hibana_runtime::scope]
pub async fn run<'r>(
    mut database: Endpoint<'r, DATABASE>,
    domain: &impl Domain,
) -> ShellResult<()> {
    let mut research: [Option<Value>; 3] = [None, None, None];
    let mut references = json!({});
    loop {
        let branch = database.offer().await?;
        match branch.label() {
            label if label == a::ResearchForDatabase::LOGICAL_LABEL => {
                let actual = branch.recv::<a::ResearchForDatabase>().await?.into_value();
                imp::research(AUTHOR_A, &actual)?;
                research[0] = Some(actual);
            }
            label if label == a::References::LOGICAL_LABEL => {
                let arguments = branch.recv::<a::References>().await?.into_value();
                let researched = research[0].as_ref().ok_or_else(|| {
                    ShellError::message("reference request has no received research")
                })?;
                let result = domain
                    .execute(imp::AUTHORS[0], "update_references", arguments.clone())
                    .await
                    .map_err(ShellError::message)?;
                imp::material(AUTHOR_A, 4, &result, (&arguments, researched))?;
                references[imp::SECTIONS[0]] = result["data"]["artifact"].clone();
                let result = imp::Reply::<AUTHOR_A, 4>::new(result)?;
                database.send::<a::Referenced>(&result).await?;
                database.send::<a::ReferencesForFact>(&result).await?;
            }

            label if label == b::ResearchForDatabase::LOGICAL_LABEL => {
                let actual = branch.recv::<b::ResearchForDatabase>().await?.into_value();
                imp::research(AUTHOR_B, &actual)?;
                research[1] = Some(actual);
            }
            label if label == b::References::LOGICAL_LABEL => {
                let arguments = branch.recv::<b::References>().await?.into_value();
                let researched = research[1].as_ref().ok_or_else(|| {
                    ShellError::message("reference request has no received research")
                })?;
                let result = domain
                    .execute(imp::AUTHORS[1], "update_references", arguments.clone())
                    .await
                    .map_err(ShellError::message)?;
                imp::material(AUTHOR_B, 4, &result, (&arguments, researched))?;
                references[imp::SECTIONS[1]] = result["data"]["artifact"].clone();
                let result = imp::Reply::<AUTHOR_B, 4>::new(result)?;
                database.send::<b::Referenced>(&result).await?;
                database.send::<b::ReferencesForFact>(&result).await?;
            }

            label if label == c::ResearchForDatabase::LOGICAL_LABEL => {
                let actual = branch.recv::<c::ResearchForDatabase>().await?.into_value();
                imp::research(AUTHOR_C, &actual)?;
                research[2] = Some(actual);
            }
            label if label == c::References::LOGICAL_LABEL => {
                let arguments = branch.recv::<c::References>().await?.into_value();
                let researched = research[2].as_ref().ok_or_else(|| {
                    ShellError::message("reference request has no received research")
                })?;
                let result = domain
                    .execute(imp::AUTHORS[2], "update_references", arguments.clone())
                    .await
                    .map_err(ShellError::message)?;
                imp::material(AUTHOR_C, 4, &result, (&arguments, researched))?;
                references[imp::SECTIONS[2]] = result["data"]["artifact"].clone();
                let result = imp::Reply::<AUTHOR_C, 4>::new(result)?;
                database.send::<c::Referenced>(&result).await?;
                database.send::<c::ReferencesForFact>(&result).await?;
            }

            ReferencesForPublication::LOGICAL_LABEL => {
                branch.recv::<ReferencesForPublication>().await?;
                database
                    .send::<FrozenReferences>(&imp::encode(&references)?.as_slice())
                    .await?;
                let published = imp::decode(database.recv::<PublishedForDatabase>().await?)?;
                imp::same(
                    &published["data"]["input_snapshot"]["references"],
                    &references,
                )?;
                return Ok(());
            }
            _ => {
                return Err(ShellError::message(
                    "unexpected reference database operation",
                ));
            }
        }
    }
}
