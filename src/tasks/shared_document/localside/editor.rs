//! Actual material and review receipts; choices arrive from the editor's files.
use super::super::{global::*, imp};
use crate::shell::{
    ShellError, ShellResult,
    imp::{report_arguments::ReportArguments, review_reply::ReviewReply},
};
use hibana::{Endpoint, g::Message};
use serde_json::json;

#[hibana_runtime::scope]
pub async fn run(endpoint: &mut Endpoint<EDITOR>) -> ShellResult<()> {
    let ((mut document, a, b), references_a, references_b) = async {
        let [mut writing, mut references_a, mut references_b] = endpoint.par()?;
        async {
            drop(writing.offer().await?);
            let (first, (second, document)) = async {
                let [mut first, mut remaining] = writing.par()?;
                async {
                    let branch = first.offer().await?;
                    match branch.label() {
                        MaterialA::LOGICAL_LABEL => imp::decode(branch.recv::<MaterialA>().await?),
                        MaterialB::LOGICAL_LABEL => imp::decode(branch.recv::<MaterialB>().await?),
                        _ => Err(ShellError::message("unexpected first chapter material")),
                    }
                };
                async {
                    drop(remaining.offer().await?);
                    async {
                        let [mut material, mut loan] = remaining.par()?;
                        async {
                            let branch = material.offer().await?;
                            match branch.label() {
                                MaterialA::LOGICAL_LABEL => {
                                    imp::decode(branch.recv::<MaterialA>().await?)
                                }
                                MaterialB::LOGICAL_LABEL => {
                                    imp::decode(branch.recv::<MaterialB>().await?)
                                }
                                _ => Err(ShellError::message("unexpected second chapter material")),
                            }
                        };
                        async {
                            loan.recv::<InitialDocument>()
                                .await
                                .map_err(ShellError::from)
                        };
                    }
                    .await
                };
            }
            .await?;
            let [a, b] = match (
                first["data"]["section"].as_str(),
                second["data"]["section"].as_str(),
            ) {
                (Some(CHAPTER_A), Some(CHAPTER_B)) => [first, second],
                (Some(CHAPTER_B), Some(CHAPTER_A)) => [second, first],
                _ => {
                    return Err(ShellError::message(
                        "document preparation has different chapters",
                    ));
                }
            };
            Ok((document, a, b))
        };
        async { imp::decode(references_a.recv::<ReferenceMaterialA>().await?) };
        async { imp::decode(references_b.recv::<ReferenceMaterialB>().await?) };
    }
    .await?;
    let a = imp::material(
        &a,
        &json!({"section_name":CHAPTER_A}),
        ("WRITER_A", "write_section"),
    )?;
    let b = imp::material(
        &b,
        &json!({"section_name":CHAPTER_B}),
        ("WRITER_B", "write_section"),
    )?;
    let references_a = imp::material(
        &references_a,
        &json!({"section_name":CHAPTER_A}),
        ("WRITER_A", "update_references"),
    )?;
    let references_b = imp::material(
        &references_b,
        &json!({"section_name":CHAPTER_B}),
        ("WRITER_B", "update_references"),
    )?;
    let mut snapshot = json!({"sections":{CHAPTER_A:a,CHAPTER_B:b},"references":{CHAPTER_A:references_a,CHAPTER_B:references_b}});
    let (reviews, value) = loop {
        let (a, b) = async {
            let [mut a, mut b] = endpoint.par()?;
            async { imp::decode(a.recv::<ReviewEvidenceA>().await?) };
            async { imp::decode(b.recv::<ReviewEvidenceB>().await?) };
        }
        .await?;
        imp::reviewed(&a, &snapshot, CHAPTER_A)?;
        imp::reviewed(&b, &snapshot, CHAPTER_B)?;
        let branch = endpoint.offer().await?;
        match branch.label() {
            CombineInput::LOGICAL_LABEL => {
                let requested = branch.recv::<CombineInput>().await?.into_value();
                let reviews = [a, b];
                imp::combine_inputs(&requested, &requested, (&snapshot, &reviews))?;
                break (reviews, requested);
            }
            RequestReconsiderA::LOGICAL_LABEL => {
                let requested = branch.recv::<RequestReconsiderA>().await?.into_value();
                if requested["review_id"] != a["data"]["review_id"] {
                    return Err(ShellError::message(
                        "reconsideration differs from the current chapter A review",
                    ));
                }
                let reply = ReportArguments::<17>::from_bytes(&imp::encode(&a)?)?;
                async {
                    let [mut service, mut writer, mut other] = endpoint.par()?;
                    async {
                        service.send::<ReconsiderA>(&reply).await?;
                    };
                    async {
                        writer.send::<RevisionReviewA>(&reply).await?;
                    };
                    async {
                        other.send::<RevisionReviewAForB>(&reply).await?;
                    };
                    Ok::<(), ShellError>(())
                }
                .await?;
                endpoint.send::<RevisionDocument>(document).await?;
                let (returned, updated) = async {
                    let [mut material, mut loan] = endpoint.par()?;
                    async {
                        loan.recv::<RevisionDocument>()
                            .await
                            .map_err(ShellError::from)
                    };
                    async { imp::decode(material.recv::<UpdatedDocument>().await?) };
                }
                .await?;
                document = returned;
                let chapter =
                    imp::chapter(&json!({"section_name":updated["data"]["section"]}))?.to_owned();
                snapshot["sections"][&chapter] = imp::material(
                    &updated,
                    &json!({"section_name":chapter}),
                    ("WRITER_A", "revise_section"),
                )?;
                let bytes = imp::encode(&updated)?;
                endpoint.send::<RevisionAckA>(&bytes.as_slice()).await?;
            }
            RequestReconsiderB::LOGICAL_LABEL => {
                let requested = branch.recv::<RequestReconsiderB>().await?.into_value();
                if requested["review_id"] != b["data"]["review_id"] {
                    return Err(ShellError::message(
                        "reconsideration differs from the current chapter B review",
                    ));
                }
                let reply = ReportArguments::<17>::from_bytes(&imp::encode(&b)?)?;
                async {
                    let [mut service, mut writer, mut other] = endpoint.par()?;
                    async {
                        service.send::<ReconsiderB>(&reply).await?;
                    };
                    async {
                        writer.send::<RevisionReviewB>(&reply).await?;
                    };
                    async {
                        other.send::<RevisionReviewBForA>(&reply).await?;
                    };
                    Ok::<(), ShellError>(())
                }
                .await?;
                endpoint.send::<RevisionDocument>(document).await?;
                let (returned, updated) = async {
                    let [mut material, mut loan] = endpoint.par()?;
                    async {
                        loan.recv::<RevisionDocument>()
                            .await
                            .map_err(ShellError::from)
                    };
                    async { imp::decode(material.recv::<UpdatedDocument>().await?) };
                }
                .await?;
                document = returned;
                let chapter =
                    imp::chapter(&json!({"section_name":updated["data"]["section"]}))?.to_owned();
                snapshot["sections"][&chapter] = imp::material(
                    &updated,
                    &json!({"section_name":chapter}),
                    ("WRITER_B", "revise_section"),
                )?;
                let bytes = imp::encode(&updated)?;
                endpoint.send::<RevisionAckB>(&bytes.as_slice()).await?;
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected editor choice {label}"
                )));
            }
        }
    };
    // Moving the actual document closes editing before approval is announced.
    endpoint.send::<FrozenDocument>(document).await?;
    let bytes_a = imp::encode(&reviews[0])?;
    let bytes_b = imp::encode(&reviews[1])?;
    endpoint
        .send::<FinalApprovedA>(&ReviewReply::<true>::from_bytes(&bytes_a)?)
        .await?;
    endpoint
        .send::<FinalApprovedB>(&ReviewReply::<true>::from_bytes(&bytes_b)?)
        .await?;
    let arguments = ReportArguments::<6>::from_bytes(&imp::encode(&value)?)?;
    async {
        let [mut service, mut writer] = endpoint.par()?;
        async {
            service.send::<Combine>(&arguments).await?;
        };
        async {
            writer.send::<CombineForward>(&arguments).await?;
        };
        Ok::<(), ShellError>(())
    }
    .await?;
    let result = imp::decode(endpoint.recv::<CombinedForEditor>().await?)?;
    imp::combined(&result, &snapshot, &reviews)?;
    let bytes = imp::encode(&result)?;
    async {
        let [mut response, mut writer] = endpoint.par()?;
        async {
            response.send::<CombinedReply>(&bytes.as_slice()).await?;
        };
        async {
            writer.send::<FinishedB>(&bytes.as_slice()).await?;
        };
        Ok::<(), ShellError>(())
    }
    .await?;
    Ok(())
}
