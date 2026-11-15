//! Actual research and review effects; finalization moves the review Endpoint.
use super::super::global::*;
use crate::shell::global::Loan;
use crate::shell::imp::review_reply::ReviewReply;
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::{Endpoint, g::Message};

pub async fn run<'run>(
    mut endpoint: Endpoint<'run, REVIEW_B>,
    mut research: Endpoint<'run, RESEARCH_B>,
    domain: &impl Domain,
) -> ShellResult<()> {
    let request = research.recv::<ResearchB>().await?.into_value();
    if request["topic"] != "subtopic_b" {
        return Err(ShellError::message(
            "research topic differs from this participant",
        ));
    }
    let result = domain
        .execute("RESEARCHER_B", "research_topic", request)
        .await
        .map_err(ShellError::message)?;
    let bytes =
        serde_json::to_vec(&result).map_err(|error| ShellError::message(error.to_string()))?;
    research.send::<ResearchResult>(&bytes.as_slice()).await?;
    loop {
        let branch = research.offer().await?;
        let arguments = match branch.label() {
            ConfirmApprovedB::LOGICAL_LABEL => {
                let bytes = branch.recv::<ConfirmApprovedB>().await?.to_vec();
                let receipt = ReviewReply::<true>::from_bytes(&bytes)?;
                research.send::<ConfirmApprovedB>(&receipt).await?;
                break;
            }
            ReviewB::LOGICAL_LABEL => branch.recv::<ReviewB>().await?.into_value(),
            label => {
                return Err(ShellError::message(format!(
                    "unexpected report service request {label}"
                )));
            }
        };
        if arguments["section_name"] != SECTION_B {
            return Err(ShellError::message("review target differs"));
        }
        let result = domain
            .execute("EDITOR", "review_sections", arguments)
            .await
            .map_err(ShellError::message)?;
        let bytes =
            serde_json::to_vec(&result).map_err(|error| ShellError::message(error.to_string()))?;
        match result["success"].as_bool() {
            Some(true) => {
                let reply = ReviewReply::<true>::from_bytes(&bytes)?;
                endpoint.send::<ApprovedB>(&reply).await?;
                endpoint.send::<ApprovedB>(&reply).await?;
                endpoint.send::<ApprovedB>(&reply).await?;
                endpoint.send::<ApprovedB>(&reply).await?;
            }
            Some(false) => {
                let reply = ReviewReply::<false>::from_bytes(&bytes)?;
                endpoint.send::<RejectedB>(&reply).await?;
                endpoint.send::<RejectedB>(&reply).await?;
                endpoint.send::<RejectedB>(&reply).await?;
                endpoint.send::<RejectedB>(&reply).await?;
            }
            None => return Err(ShellError::message("review result has no success field")),
        }
    }
    research.send::<Loan<REVIEW_B>>(endpoint).await?;
    Ok(())
}
