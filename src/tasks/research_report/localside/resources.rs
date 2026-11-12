//! Physical owners execute only the actual requests carried by their sole Endpoints.
use super::super::global::*;
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::Endpoint;
use serde_json::Value;

pub async fn document<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, DOCUMENT_SERVICE>,
    mut document: Endpoint<'r, DOCUMENT>,
    domain: &D,
) -> ShellResult<()> {
    let admitted = endpoint.recv::<WriteA>().await?.into_value();
    endpoint.send::<DocumentLoan>(document).await?;
    let request = endpoint.recv::<WriteOnDocument>().await?.into_value();
    if request != admitted || request["section_name"] != SECTION_A {
        return Err(ShellError::message("document write arguments differ"));
    }
    let result = domain
        .execute("RESEARCHER_A", "write_section", request)
        .await
        .map_err(ShellError::message)?;
    let bytes =
        serde_json::to_vec(&result).map_err(|error| ShellError::message(error.to_string()))?;
    endpoint.send::<DocumentResult>(&bytes.as_slice()).await?;
    document = endpoint.recv::<DocumentLoan>().await?;

    let admitted = endpoint.recv::<WriteB>().await?.into_value();
    endpoint.send::<DocumentLoan>(document).await?;
    let request = endpoint.recv::<WriteOnDocument>().await?.into_value();
    if request != admitted || request["section_name"] != SECTION_B {
        return Err(ShellError::message("document write arguments differ"));
    }
    let result = domain
        .execute("RESEARCHER_B", "write_section", request)
        .await
        .map_err(ShellError::message)?;
    let bytes =
        serde_json::to_vec(&result).map_err(|error| ShellError::message(error.to_string()))?;
    endpoint.send::<DocumentResult>(&bytes.as_slice()).await?;
    document = endpoint.recv::<DocumentLoan>().await?;

    // These are the actual results received from the review services, not flags.
    let mut review_a = Vec::new();
    let mut review_b = Vec::new();
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            76 | 176 => {
                let (agent, section, admitted, feedback) = match branch.label() {
                    76 => (
                        "RESEARCHER_A",
                        SECTION_A,
                        branch.recv::<ReviseA>().await?,
                        &review_a,
                    ),
                    176 => (
                        "RESEARCHER_B",
                        SECTION_B,
                        branch.recv::<ReviseB>().await?,
                        &review_b,
                    ),
                    _ => unreachable!(),
                };
                let admitted = admitted.into_value();
                let reviewed: Value = serde_json::from_slice(feedback)
                    .map_err(|error| ShellError::message(error.to_string()))?;
                if reviewed["success"] != false {
                    return Err(ShellError::message(
                        "revision has no actual negative review",
                    ));
                }
                endpoint.send::<DocumentLoan>(document).await?;
                let request = endpoint.recv::<ReviseOnDocument>().await?.into_value();
                if request != admitted || request["section_name"] != section {
                    return Err(ShellError::message("document revision arguments differ"));
                }
                let result = domain
                    .execute(agent, "revise_section", request)
                    .await
                    .map_err(ShellError::message)?;
                let bytes = serde_json::to_vec(&result)
                    .map_err(|error| ShellError::message(error.to_string()))?;
                endpoint.send::<DocumentResult>(&bytes.as_slice()).await?;
                document = endpoint.recv::<DocumentLoan>().await?;
            }
            30 => {
                review_a = branch.recv::<ApprovedA>().await?.to_vec();
            }
            31 => {
                review_a = branch.recv::<RejectedA>().await?.to_vec();
            }
            40 => {
                review_b = branch.recv::<ApprovedB>().await?.to_vec();
            }
            41 => {
                review_b = branch.recv::<RejectedB>().await?.to_vec();
            }
            224 => {
                if branch.recv::<ConfirmApprovedA>().await? != review_a {
                    return Err(ShellError::message(
                        "section A approval differs from the actual review",
                    ));
                }
                if endpoint.recv::<ConfirmApprovedB>().await? != review_b {
                    return Err(ShellError::message(
                        "section B approval differs from the actual review",
                    ));
                }
                break;
            }
            80 => {
                branch.recv::<FirstA>().await?;
            }
            81 => {
                branch.recv::<FirstB>().await?;
            }
            82 => {
                branch.recv::<AgainABeforeB>().await?;
            }
            83 => {
                branch.recv::<BDuringARevision>().await?;
            }
            84 => {
                branch.recv::<BOnly>().await?;
            }
            85 => {
                branch.recv::<AgainAAfterB>().await?;
            }
            86 => {
                branch.recv::<AgainBBeforeA>().await?;
            }
            87 => {
                branch.recv::<ADuringBRevision>().await?;
            }
            88 => {
                branch.recv::<AOnly>().await?;
            }
            89 => {
                branch.recv::<AgainBAfterA>().await?;
            }
            92 => {
                branch.recv::<AgainADuringBRevision>().await?;
            }
            93 => {
                branch.recv::<AgainBDuringARevision>().await?;
            }
            94 => {
                branch.recv::<AgainBBeforeAReview>().await?;
            }
            95 => {
                branch.recv::<AgainABeforeBReview>().await?;
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected document operation {label}"
                )));
            }
        }
    }
    let admitted = endpoint.recv::<Combine>().await?.into_value();
    endpoint.send::<DocumentLoan>(document).await?;
    let request = endpoint.recv::<CombineOnDocument>().await?.into_value();
    let sections = request["sections"]
        .as_str()
        .unwrap_or("")
        .split(',')
        .map(str::trim)
        .collect::<Vec<_>>();
    let a: Value = serde_json::from_slice(&review_a)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let b: Value = serde_json::from_slice(&review_b)
        .map_err(|error| ShellError::message(error.to_string()))?;
    if request != admitted
        || sections.len() != 2
        || !sections.contains(&SECTION_A)
        || !sections.contains(&SECTION_B)
        || a["success"] != true
        || b["success"] != true
    {
        return Err(ShellError::message(
            "combine arguments or actual reviews differ",
        ));
    }
    let result = domain
        .execute("EDITOR", "combine_sections", request)
        .await
        .map_err(ShellError::message)?;
    let bytes =
        serde_json::to_vec(&result).map_err(|error| ShellError::message(error.to_string()))?;
    endpoint.send::<DocumentResult>(&bytes.as_slice()).await?;
    let _document = endpoint.recv::<DocumentLoan>().await?;
    endpoint.recv::<DocumentExit>().await?;
    Ok(())
}

pub async fn database<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, DATABASE_SERVICE>,
    mut database: Endpoint<'r, DATABASE>,
    domain: &D,
) -> ShellResult<()> {
    let admitted = endpoint.recv::<ReferencesA>().await?.into_value();
    endpoint.send::<DatabaseLoan>(database).await?;
    let request = endpoint.recv::<ReferencesOnDatabase>().await?.into_value();
    if request != admitted || request["section_name"] != SECTION_A {
        return Err(ShellError::message("reference arguments differ"));
    }
    let result = domain
        .execute("RESEARCHER_A", "update_references", request)
        .await
        .map_err(ShellError::message)?;
    let bytes =
        serde_json::to_vec(&result).map_err(|error| ShellError::message(error.to_string()))?;
    endpoint.send::<DatabaseResult>(&bytes.as_slice()).await?;
    database = endpoint.recv::<DatabaseLoan>().await?;

    let admitted = endpoint.recv::<ReferencesB>().await?.into_value();
    endpoint.send::<DatabaseLoan>(database).await?;
    let request = endpoint.recv::<ReferencesOnDatabase>().await?.into_value();
    if request != admitted || request["section_name"] != SECTION_B {
        return Err(ShellError::message("reference arguments differ"));
    }
    let result = domain
        .execute("RESEARCHER_B", "update_references", request)
        .await
        .map_err(ShellError::message)?;
    let bytes =
        serde_json::to_vec(&result).map_err(|error| ShellError::message(error.to_string()))?;
    endpoint.send::<DatabaseResult>(&bytes.as_slice()).await?;
    let _database = endpoint.recv::<DatabaseLoan>().await?;
    endpoint.recv::<DatabaseExit>().await?;
    Ok(())
}

pub async fn research<'r, const ROLE: u8>(
    endpoint: &mut Endpoint<'r, ROLE>,
    domain: &impl Domain,
) -> ShellResult<()> {
    let (agent, topic) = match ROLE {
        RESEARCH_A => ("RESEARCHER_A", "subtopic_a"),
        RESEARCH_B => ("RESEARCHER_B", "subtopic_b"),
        _ => return Err(ShellError::message("unknown research service")),
    };
    let request = endpoint.recv::<Research>().await?.into_value();
    if request["topic"] != topic {
        return Err(ShellError::message(
            "research topic differs from this participant",
        ));
    }
    let result = domain
        .execute(agent, "research_topic", request)
        .await
        .map_err(ShellError::message)?;
    let bytes =
        serde_json::to_vec(&result).map_err(|error| ShellError::message(error.to_string()))?;
    endpoint.send::<ResearchResult>(&bytes.as_slice()).await?;
    if ROLE == RESEARCH_A {
        endpoint.recv::<ResearchExitA>().await?;
    } else {
        endpoint.recv::<ResearchExitB>().await?;
    }
    Ok(())
}
