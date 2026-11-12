//! One research participant; independent authors use disjoint projected fragments.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::ParallelEndpoint;

#[hibana_runtime::scope]
pub async fn run<const AUTHOR: u8, C: ChapterMessages<AUTHOR>>(
    endpoint: &mut ParallelEndpoint<RESEARCH_SERVICE>,
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
