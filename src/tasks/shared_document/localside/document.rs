//! The data owner receives admitted arguments and actual resource requests.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::ReceivedEffects};
use hibana::{Endpoint, g::Message};
use serde_json::json;

#[hibana_runtime::scope]
pub async fn run<'r>(
    endpoint: &mut Endpoint<'r, DOCUMENT_SERVICE>,
    document: Endpoint<'r, DOCUMENT>,
    effects: &impl ReceivedEffects,
) -> ShellResult<Endpoint<'r, DOCUMENT>> {
    let (sections, references_a, references_b) = async {
        let [mut writing, mut references_a, mut references_b] = endpoint.par()?;
        async {
            let first = writing.recv::<Allocate>().await?.into_value();
            let accepted_bytes = imp::encode(&first)?;
            async {
                let [mut accepted, mut writing] = writing.par()?;
                async {
                    accepted
                        .send::<AllocationAccepted>(&accepted_bytes.as_slice())
                        .await
                        .map_err(ShellError::from)
                };
                async {
                    match first["first_writer"].as_str() {
                        Some("WRITER_A") => {
                            writing.send::<InitialDocument>(document).await?;
                            let admitted = writing.recv::<WriteA>().await?;
                            writing.send::<WriteArgumentsA>(&admitted).await?;
                            let admitted = admitted.into_value();
                            if imp::chapter(&admitted)? != CHAPTER_A {
                                return Err(ShellError::message("initial chapter A differs"));
                            }
                            let request = writing.recv::<WriteOnDocumentA>().await?;
                            let arguments = imp::decode(request.as_bytes())?;
                            imp::same(&admitted, &arguments)?;
                            let result_a = effects.apply(request).await?;
                            let material_a = imp::material(
                                &result_a,
                                &arguments,
                                ("WRITER_A", "write_section"),
                            )?;
                            let bytes_a = imp::encode(&result_a)?;
                            writing.send::<Written>(&bytes_a.as_slice()).await?;
                            let material_b = async {
                                let [mut notification, mut continuation] = writing.par()?;
                                async {
                                    notification.send::<MaterialA>(&bytes_a.as_slice()).await?;
                                };
                                let admitted = continuation.recv::<WriteB>().await?;
                                continuation.send::<WriteArgumentsB>(&admitted).await?;
                                let admitted = admitted.into_value();
                                if imp::chapter(&admitted)? != CHAPTER_B {
                                    return Err(ShellError::message("initial chapter B differs"));
                                }
                                let request = continuation.recv::<WriteOnDocumentB>().await?;
                                let arguments = imp::decode(request.as_bytes())?;
                                imp::same(&admitted, &arguments)?;
                                let result = effects.apply(request).await?;
                                let material = imp::material(
                                    &result,
                                    &arguments,
                                    ("WRITER_B", "write_section"),
                                )?;
                                let bytes = imp::encode(&result)?;
                                continuation.send::<Written>(&bytes.as_slice()).await?;
                                continuation.send::<MaterialB>(&bytes.as_slice()).await?;
                                Ok(material)
                            }
                            .await?;
                            Ok([material_a, material_b])
                        }
                        Some("WRITER_B") => {
                            writing.send::<InitialDocumentB>(document).await?;
                            let admitted = writing.recv::<WriteB>().await?;
                            writing.send::<WriteArgumentsB>(&admitted).await?;
                            let admitted = admitted.into_value();
                            if imp::chapter(&admitted)? != CHAPTER_B {
                                return Err(ShellError::message("initial chapter B differs"));
                            }
                            let request = writing.recv::<WriteOnDocumentB>().await?;
                            let arguments = imp::decode(request.as_bytes())?;
                            imp::same(&admitted, &arguments)?;
                            let result_b = effects.apply(request).await?;
                            let material_b = imp::material(
                                &result_b,
                                &arguments,
                                ("WRITER_B", "write_section"),
                            )?;
                            let bytes_b = imp::encode(&result_b)?;
                            writing.send::<Written>(&bytes_b.as_slice()).await?;
                            let material_a = async {
                                let [mut notification, mut continuation] = writing.par()?;
                                async {
                                    notification.send::<MaterialB>(&bytes_b.as_slice()).await?;
                                };
                                let admitted = continuation.recv::<WriteA>().await?;
                                continuation.send::<WriteArgumentsA>(&admitted).await?;
                                let admitted = admitted.into_value();
                                if imp::chapter(&admitted)? != CHAPTER_A {
                                    return Err(ShellError::message("initial chapter A differs"));
                                }
                                let request = continuation.recv::<WriteOnDocumentA>().await?;
                                let arguments = imp::decode(request.as_bytes())?;
                                imp::same(&admitted, &arguments)?;
                                let result = effects.apply(request).await?;
                                let material = imp::material(
                                    &result,
                                    &arguments,
                                    ("WRITER_A", "write_section"),
                                )?;
                                let bytes = imp::encode(&result)?;
                                continuation.send::<Written>(&bytes.as_slice()).await?;
                                continuation.send::<MaterialA>(&bytes.as_slice()).await?;
                                Ok(material)
                            }
                            .await?;
                            Ok([material_a, material_b])
                        }
                        _ => unreachable!("validated first writer"),
                    }
                };
            }
            .await
            .map(|(_, sections)| sections)
        };
        async {
            let request = references_a.recv::<ReferencesA>().await?;
            let arguments = imp::decode(request.as_bytes())?;
            if imp::chapter(&arguments)? != CHAPTER_A {
                return Err(ShellError::message("references A differ"));
            }
            let result = effects.apply(request).await?;
            let material = imp::material(&result, &arguments, ("WRITER_A", "update_references"))?;
            let bytes = imp::encode(&result)?;
            references_a.send::<ReferencedA>(&bytes.as_slice()).await?;
            references_a
                .send::<ReferenceMaterialA>(&bytes.as_slice())
                .await?;
            Ok(material)
        };
        async {
            let request = references_b.recv::<ReferencesB>().await?;
            let arguments = imp::decode(request.as_bytes())?;
            if imp::chapter(&arguments)? != CHAPTER_B {
                return Err(ShellError::message("references B differ"));
            }
            let result = effects.apply(request).await?;
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
                let request = a.recv::<ReviewA>().await?;
                let arguments = imp::decode(request.as_bytes())?;
                if imp::chapter(&arguments)? != CHAPTER_A {
                    return Err(ShellError::message("review A changed scope"));
                }
                let result = effects.apply(request).await?;
                imp::reviewed(&result, &snapshot, CHAPTER_A)?;
                let bytes = imp::encode(&result)?;
                async {
                    let [mut response, mut evidence] = a.par()?;
                    async {
                        response.send::<ReviewedA>(&bytes.as_slice()).await?;
                    };
                    async {
                        evidence.send::<ReviewEvidenceA>(&bytes.as_slice()).await?;
                    };
                    Ok::<(), ShellError>(())
                }
                .await?;
                Ok(result)
            };
            async {
                let request = b.recv::<ReviewB>().await?;
                let arguments = imp::decode(request.as_bytes())?;
                if imp::chapter(&arguments)? != CHAPTER_B {
                    return Err(ShellError::message("review B changed scope"));
                }
                let result = effects.apply(request).await?;
                imp::reviewed(&result, &snapshot, CHAPTER_B)?;
                let bytes = imp::encode(&result)?;
                async {
                    let [mut response, mut evidence] = b.par()?;
                    async {
                        response.send::<ReviewedB>(&bytes.as_slice()).await?;
                    };
                    async {
                        evidence.send::<ReviewEvidenceB>(&bytes.as_slice()).await?;
                    };
                    Ok::<(), ShellError>(())
                }
                .await?;
                Ok(result)
            };
        }
        .await?;
        let branch = endpoint.offer().await?;
        let (arguments, result, agent) = match branch.label() {
            FinalApprovedA::LOGICAL_LABEL => {
                imp::same(&imp::decode(branch.recv::<FinalApprovedA>().await?)?, &a)?;
                imp::same(&imp::decode(endpoint.recv::<FinalApprovedB>().await?)?, &b)?;
                break [a, b];
            }
            ReconsiderA::LOGICAL_LABEL => {
                imp::same(&branch.recv::<ReconsiderA>().await?.into_value(), &a)?;
                let admitted = endpoint.recv::<ReviseA>().await?;
                endpoint.send::<RevisionArgumentsA>(&admitted).await?;
                let request = endpoint.recv::<ReviseOnDocumentA>().await?;
                let arguments = imp::decode(request.as_bytes())?;
                imp::same(&admitted.into_value(), &arguments)?;
                (arguments, effects.apply(request).await?, "WRITER_A")
            }
            ReconsiderB::LOGICAL_LABEL => {
                imp::same(&branch.recv::<ReconsiderB>().await?.into_value(), &b)?;
                let admitted = endpoint.recv::<ReviseB>().await?;
                endpoint.send::<RevisionArgumentsB>(&admitted).await?;
                let request = endpoint.recv::<ReviseOnDocumentB>().await?;
                let arguments = imp::decode(request.as_bytes())?;
                imp::same(&admitted.into_value(), &arguments)?;
                (arguments, effects.apply(request).await?, "WRITER_B")
            }
            _ => {
                return Err(ShellError::message(
                    "expected an actual reconsideration request or both final approvals",
                ));
            }
        };
        let section = imp::chapter(&arguments)?;
        snapshot["sections"][section] =
            imp::material(&result, &arguments, (agent, "revise_section"))?;
        let bytes = imp::encode(&result)?;
        endpoint.send::<Revised>(&bytes.as_slice()).await?;
        endpoint.send::<UpdatedDocument>(&bytes.as_slice()).await?;
    };
    let admitted = endpoint.recv::<Combine>().await?.into_value();
    let request = endpoint.recv::<CombineOnDocument>().await?;
    let arguments = imp::decode(request.as_bytes())?;
    imp::combine_inputs(&admitted, &arguments, (&snapshot, &reviews))?;
    let result = effects.apply(request).await?;
    imp::combined(&result, &snapshot, &reviews)?;
    let bytes = imp::encode(&result)?;
    endpoint.send::<Combined>(&bytes.as_slice()).await?;
    endpoint.recv::<DocumentReturn>().await.map_err(Into::into)
}
