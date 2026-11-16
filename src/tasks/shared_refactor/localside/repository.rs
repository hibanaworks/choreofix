//! The resource owner performs effects; independent views carry actual validation data.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::{Endpoint, g::Message};
use serde_json::{Value, json};

#[hibana_runtime::scope]
pub async fn run<D: Domain>(owner: &mut Endpoint<REPO_OWNER>, domain: &D) -> ShellResult<()> {
    let mut repo = owner.recv::<RepoLoan>().await?;
    let mut current = Value::Null;
    // Four finite edits, in whichever order the actual file requests arrive.
    for _ in 0..4 {
        let branch = repo.offer().await?;
        match branch.label() {
            RefactorA::LOGICAL_LABEL => {
                let args = branch.recv::<RefactorA>().await?.into_value();
                let value = domain
                    .execute("DEVELOPER_A", "refactor_code", args)
                    .await
                    .map_err(ShellError::message)?;
                imp::changed(&value, "module_a")?;
                current = value["data"]["working_candidates"].clone();
                let bytes = imp::encode(&value)?;
                repo.send::<RefactoredA>(&bytes.as_slice()).await?;
            }
            RefactorSharedA::LOGICAL_LABEL => {
                let args = branch.recv::<RefactorSharedA>().await?.into_value();
                let value = domain
                    .execute("DEVELOPER_A", "refactor_code", args)
                    .await
                    .map_err(ShellError::message)?;
                imp::changed(&value, "SHARED_LIBRARY")?;
                current = value["data"]["working_candidates"].clone();
                let bytes = imp::encode(&value)?;
                repo.send::<SharedChangedA>(&bytes.as_slice()).await?;
            }
            RefactorB::LOGICAL_LABEL => {
                let args = branch.recv::<RefactorB>().await?.into_value();
                let value = domain
                    .execute("DEVELOPER_B", "refactor_code", args)
                    .await
                    .map_err(ShellError::message)?;
                imp::changed(&value, "module_b")?;
                current = value["data"]["working_candidates"].clone();
                let bytes = imp::encode(&value)?;
                repo.send::<RefactoredB>(&bytes.as_slice()).await?;
            }
            RefactorSharedB::LOGICAL_LABEL => {
                let args = branch.recv::<RefactorSharedB>().await?.into_value();
                let value = domain
                    .execute("DEVELOPER_B", "refactor_code", args)
                    .await
                    .map_err(ShellError::message)?;
                imp::changed(&value, "SHARED_LIBRARY")?;
                current = value["data"]["working_candidates"].clone();
                let bytes = imp::encode(&value)?;
                repo.send::<SharedChangedB>(&bytes.as_slice()).await?;
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected repository edit {label}"
                )));
            }
        }
    }
    async {
        async {
            async {
                let [mut a, mut b] = owner.par()?;
                async {
                    let (review, test) = async {
                        let [mut review, mut test] = a.par()?;
                        async {
                            let bytes = imp::encode(&current)?;
                            review.send::<ReadyReviewA>(&bytes.as_slice()).await?;
                            let value = imp::decode(review.recv::<ReviewedA>().await?)?;
                            imp::validation(&value, &current, "module_a")?;
                            Ok(value)
                        };
                        async {
                            let bytes = imp::encode(&current)?;
                            test.send::<ReadyTestA>(&bytes.as_slice()).await?;
                            let value = imp::decode(test.recv::<TestedA>().await?)?;
                            imp::validation(&value, &current, "module_a")?;
                            Ok(value)
                        };
                    }
                    .await?;
                    async {
                        let [mut reviewed, mut tested] = a.par()?;
                        async {
                            let bytes = imp::encode(&review)?;
                            reviewed.send::<ReviewFeedbackA>(&bytes.as_slice()).await?;
                        };
                        async {
                            let bytes = imp::encode(&test)?;
                            tested.send::<TestFeedbackA>(&bytes.as_slice()).await?;
                        };
                        Ok(())
                    }
                    .await?;
                    let arguments = a.recv::<MergeA>().await?.into_value();
                    imp::publication_inputs(&arguments, &current, ("module_a", &review, &test))?;
                    let bytes =
                        imp::encode(&json!({"arguments":arguments,"review":review,"test":test}))?;
                    a.send::<PublicationA>(&bytes.as_slice()).await?;
                    let value = imp::decode(a.recv::<PublishedA>().await?)?;
                    imp::published(&value, &current, "module_a")?;
                    let bytes = imp::encode(&value)?;
                    a.send::<MergedA>(&bytes.as_slice()).await?;
                };
                async {
                    let (review, test) = async {
                        let [mut review, mut test] = b.par()?;
                        async {
                            let bytes = imp::encode(&current)?;
                            review.send::<ReadyReviewB>(&bytes.as_slice()).await?;
                            let value = imp::decode(review.recv::<ReviewedB>().await?)?;
                            imp::validation(&value, &current, "module_b")?;
                            Ok(value)
                        };
                        async {
                            let bytes = imp::encode(&current)?;
                            test.send::<ReadyTestB>(&bytes.as_slice()).await?;
                            let value = imp::decode(test.recv::<TestedB>().await?)?;
                            imp::validation(&value, &current, "module_b")?;
                            Ok(value)
                        };
                    }
                    .await?;
                    async {
                        let [mut reviewed, mut tested] = b.par()?;
                        async {
                            let bytes = imp::encode(&review)?;
                            reviewed.send::<ReviewFeedbackB>(&bytes.as_slice()).await?;
                        };
                        async {
                            let bytes = imp::encode(&test)?;
                            tested.send::<TestFeedbackB>(&bytes.as_slice()).await?;
                        };
                        Ok(())
                    }
                    .await?;
                    let arguments = b.recv::<MergeB>().await?.into_value();
                    imp::publication_inputs(&arguments, &current, ("module_b", &review, &test))?;
                    let bytes =
                        imp::encode(&json!({"arguments":arguments,"review":review,"test":test}))?;
                    b.send::<PublicationB>(&bytes.as_slice()).await?;
                    let value = imp::decode(b.recv::<PublishedB>().await?)?;
                    imp::published(&value, &current, "module_b")?;
                    let bytes = imp::encode(&value)?;
                    b.send::<MergedB>(&bytes.as_slice()).await?;
                };
                Ok(())
            }
            .await?;
            owner.send::<ReviewExit>(&0).await?;
            owner.send::<TestExit>(&0).await?;
        };
        async {
            // This Endpoint's single I/O owner retains the physical repository.
            for _ in 0..2 {
                let branch = repo.offer().await?;
                match branch.label() {
                    PublicationA::LOGICAL_LABEL => {
                        let admitted = imp::decode(branch.recv::<PublicationA>().await?)?;
                        let arguments = &admitted["arguments"];
                        imp::publication_inputs(
                            arguments,
                            &current,
                            ("module_a", &admitted["review"], &admitted["test"]),
                        )?;
                        let value = domain
                            .execute("DEVELOPER_A", "merge_code", arguments.clone())
                            .await
                            .map_err(ShellError::message)?;
                        imp::published(&value, &current, "module_a")?;
                        let bytes = imp::encode(&value)?;
                        repo.send::<PublishedA>(&bytes.as_slice()).await?;
                    }
                    PublicationB::LOGICAL_LABEL => {
                        let admitted = imp::decode(branch.recv::<PublicationB>().await?)?;
                        let arguments = &admitted["arguments"];
                        imp::publication_inputs(
                            arguments,
                            &current,
                            ("module_b", &admitted["review"], &admitted["test"]),
                        )?;
                        let value = domain
                            .execute("DEVELOPER_B", "merge_code", arguments.clone())
                            .await
                            .map_err(ShellError::message)?;
                        imp::published(&value, &current, "module_b")?;
                        let bytes = imp::encode(&value)?;
                        repo.send::<PublishedB>(&bytes.as_slice()).await?;
                    }
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected publication {label}"
                        )));
                    }
                }
            }
        };
        Ok(())
    }
    .await
}
