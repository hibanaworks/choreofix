use super::super::global::*;
use crate::shell::{ShellError, ShellResult};
use hibana::{Endpoint, g::Message};

pub async fn run<'run, const AUTHOR: u8, const SECTION: u8, C: ChapterMessages<AUTHOR>>(
    endpoint: &mut Endpoint<'run, AUTHOR>,
    mut section: Endpoint<'run, SECTION>,
) -> ShellResult<()> {
    loop {
        endpoint.send::<Section<SECTION>>(section).await?;
        let branch = endpoint.offer().await?;
        match branch.label() {
            label if label == C::FactRejectionForAuthor::LOGICAL_LABEL => {
                branch.recv::<C::FactRejectionForAuthor>().await?;
            }
            label if label == C::ReviewRejectionForAuthor::LOGICAL_LABEL => {
                branch.recv::<C::ReviewRejectionForAuthor>().await?;
            }
            label if label == C::FinalForAuthor::LOGICAL_LABEL => {
                branch.recv::<C::FinalForAuthor>().await?;
                return Ok(());
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected chapter verdict {label}"
                )));
            }
        }
        section = endpoint.recv::<Section<SECTION>>().await?;
    }
}
