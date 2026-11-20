//! One research participant; independent authors use disjoint projected fragments.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::{EndpointBorrow, g::Message};

#[hibana_runtime::scope]
pub async fn run<'borrow, 'run, const AUTHOR: u8, C: ChapterMessages<AUTHOR>>(
    endpoint: &mut EndpointBorrow<'borrow, 'run, RESEARCH_SERVICE>,
    domain: &impl Domain,
) -> ShellResult<()> {
    let arguments = endpoint.recv::<C::Research>().await?.into_value();
    let result = domain
        .execute(
            imp::AUTHORS[usize::from(AUTHOR)],
            "research_topic",
            arguments.clone(),
        )
        .await
        .map_err(ShellError::message)?;
    imp::research(AUTHOR, &result)?;
    imp::same(&result["data"]["topic"], &arguments["topic"])?;
    let result = imp::Reply::<AUTHOR, 1>::new(result)?;
    async {
        let [mut author, mut document, mut database] = endpoint.par()?;
        async {
            author.send::<C::Researched>(&result).await?;
        };
        async {
            document.send::<C::ResearchForDocument>(&result).await?;
        };
        async {
            database.send::<C::ResearchForDatabase>(&result).await?;
        };
        Ok::<(), ShellError>(())
    }
    .await
}

pub async fn supplemental<const AUTHOR: u8, C: ChapterMessages<AUTHOR>>(
    endpoint: &mut EndpointBorrow<'_, '_, RESEARCH_SERVICE>,
    domain: &impl Domain,
) -> ShellResult<()> {
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            label if label == C::ResearchDone::LOGICAL_LABEL => {
                branch.recv::<C::ResearchDone>().await?;
                return Ok(());
            }
            label if label == C::Supplement::LOGICAL_LABEL => {
                let arguments = branch.recv::<C::Supplement>().await?.into_value();
                let actual = domain
                    .execute(
                        imp::AUTHORS[usize::from(AUTHOR)],
                        "research_topic",
                        arguments.clone(),
                    )
                    .await
                    .map_err(ShellError::message)?;
                imp::same(&actual["data"]["topic"], &arguments["topic"])?;
                endpoint
                    .send::<C::Supplemented>(&imp::Reply::<AUTHOR, 7>::new(actual)?)
                    .await?;
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected supplemental research operation {label}"
                )));
            }
        }
    }
}
