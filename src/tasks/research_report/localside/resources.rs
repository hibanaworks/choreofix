//! Direct receivers own actual report data and apply only admitted requests.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::{Endpoint, g::Message};
use serde_json::{Value, json};

#[hibana_runtime::scope]
pub async fn initial_document<'run>(
    endpoint: &mut Endpoint<'run, INITIAL_DOCUMENT>,
    (mut document, requests): (Endpoint<'run, DOCUMENT>, &mut Endpoint<WRITE_REQUESTS>),
    domain: &impl Domain,
) -> ShellResult<()> {
    let ((document, sections), references) = async {
        let [mut writing, mut material] = endpoint.par()?;
        async {
            let mut sections = json!({});
            for _ in 0..2 {
                let claim = requests.offer().await?;
                let (agent, section, arguments) = match claim.label() {
                    WriteA::LOGICAL_LABEL => {
                        let value = claim.recv::<WriteA>().await?;
                        writing.send::<WriteArgumentsA>(&value).await?;
                        ("RESEARCHER_A", SECTION_A, value.into_value())
                    }
                    WriteB::LOGICAL_LABEL => {
                        let value = claim.recv::<WriteB>().await?;
                        writing.send::<WriteArgumentsB>(&value).await?;
                        ("RESEARCHER_B", SECTION_B, value.into_value())
                    }
                    _ => return Err(ShellError::message("unexpected document submission")),
                };
                writing.send::<DocumentLoan>(document).await?;
                let request = writing.recv::<WriteOnDocument>().await?.into_value();
                imp::same(&arguments, &request)?;
                let result = domain
                    .execute(agent, "write_section", request.clone())
                    .await
                    .map_err(ShellError::message)?;
                sections[section] = imp::material(&result, &request, (agent, "write_section"))?;
                let bytes = imp::encode(&result)?;
                writing.send::<DocumentResult>(&bytes.as_slice()).await?;
                document = writing.recv::<DocumentLoan>().await?;
            }
            Ok::<_, ShellError>((document, sections))
        };
        async {
            let mut references = json!({});
            for _ in 0..2 {
                let receipt = material.offer().await?;
                let (agent, section, result) = match receipt.label() {
                    ReferenceMaterialA::LOGICAL_LABEL => (
                        "RESEARCHER_A",
                        SECTION_A,
                        imp::decode(receipt.recv::<ReferenceMaterialA>().await?)?,
                    ),
                    ReferenceMaterialB::LOGICAL_LABEL => (
                        "RESEARCHER_B",
                        SECTION_B,
                        imp::decode(receipt.recv::<ReferenceMaterialB>().await?)?,
                    ),
                    _ => return Err(ShellError::message("unexpected reference receipt")),
                };
                references[section] = imp::material(
                    &result,
                    &json!({"section_name":section}),
                    (agent, "update_references"),
                )?;
            }
            Ok::<_, ShellError>(references)
        };
    }
    .await?;
    let snapshot = json!({"sections":sections,"references":references});
    let review = endpoint.offer().await?;
    let (section, receipt) = match review.label() {
        ApprovedA::LOGICAL_LABEL => (SECTION_A, imp::decode(review.recv::<ApprovedA>().await?)?),
        RejectedA::LOGICAL_LABEL => (SECTION_A, imp::decode(review.recv::<RejectedA>().await?)?),
        ApprovedB::LOGICAL_LABEL => (SECTION_B, imp::decode(review.recv::<ApprovedB>().await?)?),
        RejectedB::LOGICAL_LABEL => (SECTION_B, imp::decode(review.recv::<RejectedB>().await?)?),
        _ => return Err(ShellError::message("expected review of initial material")),
    };
    imp::reviewed(&receipt, &snapshot, section)?;
    let bytes = imp::encode(&snapshot)?;
    endpoint.send::<ReportMaterial>(&bytes.as_slice()).await?;
    endpoint.send::<DocumentLoan>(document).await?;
    Ok(())
}

pub async fn document<'run>(
    endpoint: &mut Endpoint<'run, DOCUMENT_SERVICE>,
    domain: &impl Domain,
) -> ShellResult<()> {
    let first_request = endpoint.offer().await?;
    let section = match first_request.label() {
        ReviewA::LOGICAL_LABEL => {
            first_request.recv::<ReviewA>().await?;
            SECTION_A
        }
        ReviewB::LOGICAL_LABEL => {
            first_request.recv::<ReviewB>().await?;
            SECTION_B
        }
        _ => return Err(ShellError::message("expected first section review")),
    };
    let result = endpoint.offer().await?;
    let receipt = match (section, result.label()) {
        (SECTION_A, ApprovedA::LOGICAL_LABEL) => {
            let bytes = result.recv::<ApprovedA>().await?.to_vec();
            let reply = crate::shell::imp::review_reply::ReviewReply::<true>::from_bytes(&bytes)?;
            endpoint.send::<ApprovedA>(&reply).await?;
            imp::decode(&bytes)?
        },
        (SECTION_A, RejectedA::LOGICAL_LABEL) => {
            let bytes = result.recv::<RejectedA>().await?.to_vec();
            let reply = crate::shell::imp::review_reply::ReviewReply::<false>::from_bytes(&bytes)?;
            endpoint.send::<RejectedA>(&reply).await?;
            imp::decode(&bytes)?
        },
        (SECTION_B, ApprovedB::LOGICAL_LABEL) => {
            let bytes = result.recv::<ApprovedB>().await?.to_vec();
            let reply = crate::shell::imp::review_reply::ReviewReply::<true>::from_bytes(&bytes)?;
            endpoint.send::<ApprovedB>(&reply).await?;
            imp::decode(&bytes)?
        },
        (SECTION_B, RejectedB::LOGICAL_LABEL) => {
            let bytes = result.recv::<RejectedB>().await?.to_vec();
            let reply = crate::shell::imp::review_reply::ReviewReply::<false>::from_bytes(&bytes)?;
            endpoint.send::<RejectedB>(&reply).await?;
            imp::decode(&bytes)?
        },
        _ => return Err(ShellError::message("review result does not match section")),
    };
    let mut snapshot = imp::decode(endpoint.recv::<ReportMaterial>().await?)?;
    let mut document = endpoint.recv::<DocumentLoan>().await?;
    imp::reviewed(&receipt, &snapshot, section)?;
    let (mut review_a, mut review_b) = if section == SECTION_A {
        (receipt, Value::Null)
    } else {
        (Value::Null, receipt)
    };
    let admitted = loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            ReviewA::LOGICAL_LABEL => {
                branch.recv::<ReviewA>().await?;
            }
            ReviewB::LOGICAL_LABEL => {
                branch.recv::<ReviewB>().await?;
            }
            DocumentLoan::LOGICAL_LABEL => {
                branch.send::<DocumentLoan>(document).await?;
                let admitted = endpoint.offer().await?;
                let (arguments, section, agent, feedback) = match admitted.label() {
                    ReviseA::LOGICAL_LABEL => {
                        let admitted = admitted.recv::<ReviseA>().await?;
                        endpoint.send::<RevisionArgumentsA>(&admitted).await?;
                        let value = admitted.into_value();
                        (value, SECTION_A, "RESEARCHER_A", &review_a)
                    }
                    ReviseB::LOGICAL_LABEL => {
                        let admitted = admitted.recv::<ReviseB>().await?;
                        endpoint.send::<RevisionArgumentsB>(&admitted).await?;
                        let value = admitted.into_value();
                        (value, SECTION_B, "RESEARCHER_B", &review_b)
                    }
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected revision request {label}"
                        )));
                    }
                };
                let request = endpoint.recv::<ReviseOnDocument>().await?.into_value();
                imp::same(&arguments, &request)?;
                imp::reviewed(feedback, &snapshot, section)?;
                if feedback["success"] != false {
                    return Err(ShellError::message(
                        "revision has no current negative review",
                    ));
                }
                let result = domain
                    .execute(agent, "revise_section", request.clone())
                    .await
                    .map_err(ShellError::message)?;
                snapshot["sections"][section] =
                    imp::material(&result, &request, (agent, "revise_section"))?;
                let bytes = imp::encode(&result)?;
                endpoint.send::<DocumentResult>(&bytes.as_slice()).await?;
                document = endpoint.recv::<DocumentLoan>().await?;
            }
            ApprovedA::LOGICAL_LABEL => {
                review_a = imp::decode(branch.recv::<ApprovedA>().await?)?;
                imp::reviewed(&review_a, &snapshot, SECTION_A)?;
            }
            RejectedA::LOGICAL_LABEL => {
                review_a = imp::decode(branch.recv::<RejectedA>().await?)?;
                imp::reviewed(&review_a, &snapshot, SECTION_A)?;
            }
            ApprovedB::LOGICAL_LABEL => {
                review_b = imp::decode(branch.recv::<ApprovedB>().await?)?;
                imp::reviewed(&review_b, &snapshot, SECTION_B)?;
            }
            RejectedB::LOGICAL_LABEL => {
                review_b = imp::decode(branch.recv::<RejectedB>().await?)?;
                imp::reviewed(&review_b, &snapshot, SECTION_B)?;
            }
            Combine::LOGICAL_LABEL => {
                break branch.recv::<Combine>().await?.into_value();
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected document operation {label}"
                )));
            }
        }
    };
    let a = imp::decode(endpoint.recv::<ConfirmApprovedA>().await?)?;
    let b = imp::decode(endpoint.recv::<ConfirmApprovedB>().await?)?;
    imp::same(&a, &review_a)?;
    imp::same(&b, &review_b)?;
    endpoint.send::<DocumentLoan>(document).await?;
    let request = endpoint.recv::<CombineOnDocument>().await?.into_value();
    let reviews = [review_a, review_b];
    imp::combine_inputs(&admitted, &request, (&snapshot, &reviews))?;
    let result = domain
        .execute("EDITOR", "combine_sections", request)
        .await
        .map_err(ShellError::message)?;
    imp::combined(&result, &snapshot, &reviews)?;
    let bytes = imp::encode(&result)?;
    endpoint.send::<DocumentResult>(&bytes.as_slice()).await?;
    Ok(())
}

#[hibana_runtime::scope]
pub async fn database<'run>(
    endpoint: &mut Endpoint<'run, DATABASE_SERVICE>,
    (mut database, requests): (Endpoint<'run, DATABASE>, &mut Endpoint<REFERENCE_REQUESTS>),
    domain: &impl Domain,
) -> ShellResult<()> {
    for _ in 0..2 {
        let claim = requests.offer().await?;
        let (agent, arguments) = match claim.label() {
            ReferencesA::LOGICAL_LABEL => {
                let value = claim.recv::<ReferencesA>().await?;
                endpoint.send::<ReferenceArgumentsA>(&value).await?;
                ("RESEARCHER_A", value.into_value())
            }
            ReferencesB::LOGICAL_LABEL => {
                let value = claim.recv::<ReferencesB>().await?;
                endpoint.send::<ReferenceArgumentsB>(&value).await?;
                ("RESEARCHER_B", value.into_value())
            }
            _ => return Err(ShellError::message("unexpected reference submission")),
        };
        endpoint.send::<DatabaseLoan>(database).await?;
        let request = endpoint.recv::<ReferencesOnDatabase>().await?.into_value();
        imp::same(&arguments, &request)?;
        let result = domain
            .execute(agent, "update_references", request.clone())
            .await
            .map_err(ShellError::message)?;
        imp::material(&result, &request, (agent, "update_references"))?;
        let bytes = imp::encode(&result)?;
        async {
            let [mut returned, mut material] = endpoint.par()?;
            async {
                returned.send::<DatabaseResult>(&bytes.as_slice()).await?;
            };
            async {
                if agent == "RESEARCHER_A" {
                    material
                        .send::<ReferenceMaterialA>(&bytes.as_slice())
                        .await?;
                } else {
                    material
                        .send::<ReferenceMaterialB>(&bytes.as_slice())
                        .await?;
                }
                Ok(())
            };
            Ok(())
        }
        .await?;
        database = endpoint.recv::<DatabaseLoan>().await?;
    }
    Ok(())
}
