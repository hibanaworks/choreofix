//! The data owner receives admitted arguments and actual resource requests.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::{Endpoint, g::Message, runtime::resolver::DecisionArm};
use serde_json::json;

#[hibana_runtime::scope]
pub async fn run<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, DOCUMENT_SERVICE>,
    (document, first): (Endpoint<'r, DOCUMENT>, DecisionArm),
    domain: &D,
) -> ShellResult<Endpoint<'r, DOCUMENT>> {
    let (sections, references_a, references_b) = async {
        let [mut writing, mut references_a, mut references_b] = endpoint.par()?;
        async {
            // The same immutable allocation input is registered with
            // FIRST_WRITER. It is not an application progress/owner flag.
            writing.send::<InitialDocument>(document).await?;
            match first {
                DecisionArm::Left => {
                    let admitted = writing.recv::<WriteA>().await?.into_value();
                    if imp::chapter(&admitted)? != CHAPTER_A {
                        return Err(ShellError::message("initial chapter A differs"));
                    }
                    let arguments = writing.recv::<WriteOnDocument>().await?.into_value();
                    imp::same(&admitted, &arguments)?;
                    let result_a = domain
                        .execute("WRITER_A", "write_section", arguments.clone())
                        .await
                        .map_err(ShellError::message)?;
                    let material_a =
                        imp::material(&result_a, &arguments, ("WRITER_A", "write_section"))?;
                    let bytes_a = imp::encode(&result_a)?;
                    writing.send::<Written>(&bytes_a.as_slice()).await?;
                    let material_b = async {
                        let [mut notification, mut continuation] = writing.par()?;
                        async {
                            notification.send::<MaterialA>(&bytes_a.as_slice()).await?;
                        };
                        let admitted = continuation.recv::<WriteB>().await?.into_value();
                        if imp::chapter(&admitted)? != CHAPTER_B {
                            return Err(ShellError::message("initial chapter B differs"));
                        }
                        let arguments = continuation.recv::<WriteOnDocument>().await?.into_value();
                        imp::same(&admitted, &arguments)?;
                        let result = domain
                            .execute("WRITER_B", "write_section", arguments.clone())
                            .await
                            .map_err(ShellError::message)?;
                        let material =
                            imp::material(&result, &arguments, ("WRITER_B", "write_section"))?;
                        let bytes = imp::encode(&result)?;
                        continuation.send::<Written>(&bytes.as_slice()).await?;
                        continuation.send::<MaterialB>(&bytes.as_slice()).await?;
                        Ok(material)
                    }
                    .await?;
                    Ok([material_a, material_b])
                }
                DecisionArm::Right => {
                    let admitted = writing.recv::<WriteB>().await?.into_value();
                    if imp::chapter(&admitted)? != CHAPTER_B {
                        return Err(ShellError::message("initial chapter B differs"));
                    }
                    let arguments = writing.recv::<WriteOnDocument>().await?.into_value();
                    imp::same(&admitted, &arguments)?;
                    let result_b = domain
                        .execute("WRITER_B", "write_section", arguments.clone())
                        .await
                        .map_err(ShellError::message)?;
                    let material_b =
                        imp::material(&result_b, &arguments, ("WRITER_B", "write_section"))?;
                    let bytes_b = imp::encode(&result_b)?;
                    writing.send::<Written>(&bytes_b.as_slice()).await?;
                    let material_a = async {
                        let [mut notification, mut continuation] = writing.par()?;
                        async {
                            notification.send::<MaterialB>(&bytes_b.as_slice()).await?;
                        };
                        let admitted = continuation.recv::<WriteA>().await?.into_value();
                        if imp::chapter(&admitted)? != CHAPTER_A {
                            return Err(ShellError::message("initial chapter A differs"));
                        }
                        let arguments = continuation.recv::<WriteOnDocument>().await?.into_value();
                        imp::same(&admitted, &arguments)?;
                        let result = domain
                            .execute("WRITER_A", "write_section", arguments.clone())
                            .await
                            .map_err(ShellError::message)?;
                        let material =
                            imp::material(&result, &arguments, ("WRITER_A", "write_section"))?;
                        let bytes = imp::encode(&result)?;
                        continuation.send::<Written>(&bytes.as_slice()).await?;
                        continuation.send::<MaterialA>(&bytes.as_slice()).await?;
                        Ok(material)
                    }
                    .await?;
                    Ok([material_a, material_b])
                }
            }
        };
        async {
            let arguments = references_a.recv::<ReferencesA>().await?.into_value();
            if imp::chapter(&arguments)? != CHAPTER_A {
                return Err(ShellError::message("references A differ"));
            }
            let result = domain
                .execute("WRITER_A", "update_references", arguments.clone())
                .await
                .map_err(ShellError::message)?;
            let material = imp::material(&result, &arguments, ("WRITER_A", "update_references"))?;
            let bytes = imp::encode(&result)?;
            references_a.send::<ReferencedA>(&bytes.as_slice()).await?;
            references_a
                .send::<ReferenceMaterialA>(&bytes.as_slice())
                .await?;
            Ok(material)
        };
        async {
            let arguments = references_b.recv::<ReferencesB>().await?.into_value();
            if imp::chapter(&arguments)? != CHAPTER_B {
                return Err(ShellError::message("references B differ"));
            }
            let result = domain
                .execute("WRITER_B", "update_references", arguments.clone())
                .await
                .map_err(ShellError::message)?;
            let material = imp::material(&result, &arguments, ("WRITER_B", "update_references"))?;
            let bytes = imp::encode(&result)?;
            references_b.send::<ReferencedB>(&bytes.as_slice()).await?;
            references_b
                .send::<ReferenceMaterialB>(&bytes.as_slice())
                .await?;
            Ok(material)
        };
    }
    .await?;
    let [a, b] = sections;
    let mut snapshot = json!({"sections":{CHAPTER_A:a,CHAPTER_B:b},
        "references":{CHAPTER_A:references_a,CHAPTER_B:references_b}});
    let reviews = loop {
        let (a, b) = async {
            let [mut a, mut b] = endpoint.par()?;
            async {
                let arguments = a.recv::<ReviewA>().await?.into_value();
                if imp::chapter(&arguments)? != CHAPTER_A {
                    return Err(ShellError::message("review A changed scope"));
                }
                let result = domain
                    .execute("EDITOR", "review_sections", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::reviewed(&result, &snapshot, CHAPTER_A)?;
                let bytes = imp::encode(&result)?;
                a.send::<ReviewedA>(&bytes.as_slice()).await?;
                Ok(result)
            };
            async {
                let arguments = b.recv::<ReviewB>().await?.into_value();
                if imp::chapter(&arguments)? != CHAPTER_B {
                    return Err(ShellError::message("review B changed scope"));
                }
                let result = domain
                    .execute("EDITOR", "review_sections", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::reviewed(&result, &snapshot, CHAPTER_B)?;
                let bytes = imp::encode(&result)?;
                b.send::<ReviewedB>(&bytes.as_slice()).await?;
                Ok(result)
            };
        }
        .await?;
        let branch = endpoint.offer().await?;
        let (admitted, agent) = match branch.label() {
            FinalApprovedA::LOGICAL_LABEL => {
                imp::same(&imp::decode(branch.recv::<FinalApprovedA>().await?)?, &a)?;
                imp::same(&imp::decode(endpoint.recv::<FinalApprovedB>().await?)?, &b)?;
                break [a, b];
            }
            ReconsiderA::LOGICAL_LABEL => {
                imp::same(&imp::decode(branch.recv::<ReconsiderA>().await?)?, &a)?;
                (endpoint.recv::<ReviseA>().await?.into_value(), "WRITER_A")
            }
            ReconsiderB::LOGICAL_LABEL => {
                imp::same(&imp::decode(branch.recv::<ReconsiderB>().await?)?, &b)?;
                (endpoint.recv::<ReviseB>().await?.into_value(), "WRITER_B")
            }
            _ => {
                return Err(ShellError::message(
                    "expected an actual rejected review or both final approvals",
                ));
            }
        };
        let section = imp::chapter(&admitted)?.to_owned();
        let arguments = endpoint.recv::<ReviseOnDocument>().await?.into_value();
        imp::same(&admitted, &arguments)?;
        let result = domain
            .execute(agent, "revise_section", arguments.clone())
            .await
            .map_err(ShellError::message)?;
        snapshot["sections"][&section] =
            imp::material(&result, &arguments, (agent, "revise_section"))?;
        let bytes = imp::encode(&result)?;
        endpoint.send::<Revised>(&bytes.as_slice()).await?;
        endpoint.send::<UpdatedDocument>(&bytes.as_slice()).await?;
    };
    let admitted = endpoint.recv::<Combine>().await?.into_value();
    let arguments = endpoint.recv::<CombineOnDocument>().await?.into_value();
    imp::combine_inputs(&admitted, &arguments, (&snapshot, &reviews))?;
    let result = domain
        .execute("EDITOR", "combine_sections", arguments)
        .await
        .map_err(ShellError::message)?;
    imp::combined(&result, &snapshot, &reviews)?;
    let bytes = imp::encode(&result)?;
    endpoint.send::<Combined>(&bytes.as_slice()).await?;
    endpoint.recv::<DocumentReturn>().await.map_err(Into::into)
}
