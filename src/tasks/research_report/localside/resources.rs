//! Direct receivers own actual report data and apply only admitted requests.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::{Endpoint, g::Message};
use serde_json::{Value, json};

#[hibana_runtime::scope]
pub async fn document<'run>(
    endpoint: &mut Endpoint<'run, DOCUMENT_SERVICE>,
    document: Endpoint<'run, DOCUMENT>,
    domain: &impl Domain,
) -> ShellResult<()> {
    endpoint.send::<DocumentLoan>(document).await?;
    let admitted = endpoint.recv::<WriteA>().await?;
    endpoint.send::<WriteArgumentsA>(&admitted).await?;
    let arguments = admitted.into_value();
    let request = endpoint.recv::<WriteOnDocument>().await?.into_value();
    imp::same(&arguments, &request)?;
    let result = domain
        .execute("RESEARCHER_A", "write_section", request.clone())
        .await
        .map_err(ShellError::message)?;
    let a = imp::material(&result, &request, ("RESEARCHER_A", "write_section"))?;
    let bytes = imp::encode(&result)?;
    endpoint.send::<DocumentResult>(&bytes.as_slice()).await?;
    let document = endpoint.recv::<DocumentLoan>().await?;
    let (references_a, (mut document, b)) = async {
        let [mut references, mut writing] = endpoint.par()?;
        async {
            let result = imp::decode(references.recv::<ReferenceMaterialA>().await?)?;
            imp::material(
                &result,
                &json!({"section_name":SECTION_A}),
                ("RESEARCHER_A", "update_references"),
            )
        };
        async {
            writing.send::<DocumentLoan>(document).await?;
            let admitted = writing.recv::<WriteB>().await?;
            writing.send::<WriteArgumentsB>(&admitted).await?;
            let arguments = admitted.into_value();
            let request = writing.recv::<WriteOnDocument>().await?.into_value();
            imp::same(&arguments, &request)?;
            let result = domain
                .execute("RESEARCHER_B", "write_section", request.clone())
                .await
                .map_err(ShellError::message)?;
            let b = imp::material(&result, &request, ("RESEARCHER_B", "write_section"))?;
            let bytes = imp::encode(&result)?;
            writing.send::<DocumentResult>(&bytes.as_slice()).await?;
            let document = writing.recv::<DocumentLoan>().await?;
            Ok((document, b))
        };
    }
    .await?;
    let result = imp::decode(endpoint.recv::<ReferenceMaterialB>().await?)?;
    let references_b = imp::material(
        &result,
        &json!({"section_name":SECTION_B}),
        ("RESEARCHER_B", "update_references"),
    )?;
    let mut snapshot = json!({"sections":{SECTION_A:a,SECTION_B:b},"references":{SECTION_A:references_a,SECTION_B:references_b}});
    let mut review_a = Value::Null;
    let mut review_b = Value::Null;
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
    database: Endpoint<'run, DATABASE>,
    domain: &impl Domain,
) -> ShellResult<()> {
    endpoint.send::<DatabaseLoan>(database).await?;
    let admitted = endpoint.recv::<ReferencesA>().await?;
    endpoint.send::<ReferenceArgumentsA>(&admitted).await?;
    let arguments = admitted.into_value();
    let request = endpoint.recv::<ReferencesOnDatabase>().await?.into_value();
    imp::same(&arguments, &request)?;
    let result = domain
        .execute("RESEARCHER_A", "update_references", request.clone())
        .await
        .map_err(ShellError::message)?;
    imp::material(&result, &request, ("RESEARCHER_A", "update_references"))?;
    let bytes = imp::encode(&result)?;
    async {
        let [mut returned, mut material] = endpoint.par()?;
        async {
            returned.send::<DatabaseResult>(&bytes.as_slice()).await?;
        };
        async {
            material
                .send::<ReferenceMaterialA>(&bytes.as_slice())
                .await?;
        };
        Ok(())
    }
    .await?;
    let database = endpoint.recv::<DatabaseLoan>().await?;
    endpoint.send::<DatabaseLoan>(database).await?;
    let admitted = endpoint.recv::<ReferencesB>().await?;
    endpoint.send::<ReferenceArgumentsB>(&admitted).await?;
    let arguments = admitted.into_value();
    let request = endpoint.recv::<ReferencesOnDatabase>().await?.into_value();
    imp::same(&arguments, &request)?;
    let result = domain
        .execute("RESEARCHER_B", "update_references", request.clone())
        .await
        .map_err(ShellError::message)?;
    imp::material(&result, &request, ("RESEARCHER_B", "update_references"))?;
    let bytes = imp::encode(&result)?;
    async {
        let [mut returned, mut material] = endpoint.par()?;
        async {
            returned.send::<DatabaseResult>(&bytes.as_slice()).await?;
        };
        async {
            material
                .send::<ReferenceMaterialB>(&bytes.as_slice())
                .await?;
        };
        Ok(())
    }
    .await?;
    let _database = endpoint.recv::<DatabaseLoan>().await?;
    Ok(())
}
