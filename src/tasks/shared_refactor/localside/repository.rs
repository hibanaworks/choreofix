//! The resource owner performs effects; independent views carry actual validation data.
use super::super::arguments::Arguments;
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::ReceivedEffects};
use hibana::{Endpoint, g::Message};
use serde_json::{Value, json};

#[hibana_runtime::scope]
pub async fn run(
    owner: &mut Endpoint<REPO_OWNER>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    let mut repo = owner.recv::<RepoLoan>().await?;
    async {
        async {
            async {
                let [mut a, mut b] = owner.par()?;
                async {
                    a.send::<ReadCandidateA>(&()).await?;
                    let current = imp::decode(a.recv::<CandidateA>().await?)?;
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
                    let arguments = imp::decode(a.recv::<MergeA>().await?.as_bytes())?;
                    imp::publication_inputs(&arguments, &current, ("module_a", &review, &test))?;
                    let bytes =
                        imp::encode(&json!({"arguments":arguments,"review":review,"test":test}))?;
                    a.send::<PublicationA>(&Arguments::try_from(bytes.as_slice())?)
                        .await?;
                    let value = imp::decode(a.recv::<PublishedA>().await?)?;
                    imp::published(&value, &current, "module_a")?;
                    let bytes = imp::encode(&value)?;
                    a.send::<MergedA>(&bytes.as_slice()).await?;
                };
                async {
                    b.send::<ReadCandidateB>(&()).await?;
                    let current = imp::decode(b.recv::<CandidateB>().await?)?;
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
                    let arguments = imp::decode(b.recv::<MergeB>().await?.as_bytes())?;
                    imp::publication_inputs(&arguments, &current, ("module_b", &review, &test))?;
                    let bytes =
                        imp::encode(&json!({"arguments":arguments,"review":review,"test":test}))?;
                    b.send::<PublicationB>(&Arguments::try_from(bytes.as_slice())?)
                        .await?;
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
            owner.send::<RepoExit>(&()).await?;
        };
        async {
            // This Endpoint's single I/O owner retains the physical repository.
            let mut current = Value::Null;
            loop {
                let branch = repo.offer().await?;
                match branch.label() {
                    RefactorA::LOGICAL_LABEL => {
                        let received = branch.recv::<RefactorA>().await?;
                        let value = effects.apply(received).await?;
                        imp::changed(&value, "module_a")?;
                        current = value["data"]["working_candidates"].clone();
                        let bytes = imp::encode(&value)?;
                        repo.send::<RefactoredA>(&bytes.as_slice()).await?;
                    }
                    RefactorSharedA::LOGICAL_LABEL => {
                        let received = branch.recv::<RefactorSharedA>().await?;
                        let value = effects.apply(received).await?;
                        imp::changed(&value, "SHARED_LIBRARY")?;
                        current = value["data"]["working_candidates"].clone();
                        let bytes = imp::encode(&value)?;
                        repo.send::<SharedChangedA>(&bytes.as_slice()).await?;
                    }
                    RefactorB::LOGICAL_LABEL => {
                        let received = branch.recv::<RefactorB>().await?;
                        let value = effects.apply(received).await?;
                        imp::changed(&value, "module_b")?;
                        current = value["data"]["working_candidates"].clone();
                        let bytes = imp::encode(&value)?;
                        repo.send::<RefactoredB>(&bytes.as_slice()).await?;
                    }
                    RefactorSharedB::LOGICAL_LABEL => {
                        let received = branch.recv::<RefactorSharedB>().await?;
                        let value = effects.apply(received).await?;
                        imp::changed(&value, "SHARED_LIBRARY")?;
                        current = value["data"]["working_candidates"].clone();
                        let bytes = imp::encode(&value)?;
                        repo.send::<SharedChangedB>(&bytes.as_slice()).await?;
                    }
                    ReadCandidateA::LOGICAL_LABEL => {
                        branch.recv::<ReadCandidateA>().await?;
                        let bytes = imp::encode(&current)?;
                        repo.send::<CandidateA>(&bytes.as_slice()).await?;
                    }
                    ReadCandidateB::LOGICAL_LABEL => {
                        branch.recv::<ReadCandidateB>().await?;
                        let bytes = imp::encode(&current)?;
                        repo.send::<CandidateB>(&bytes.as_slice()).await?;
                    }
                    RepoExit::LOGICAL_LABEL => {
                        branch.recv::<RepoExit>().await?;
                        break;
                    }
                    PublicationA::LOGICAL_LABEL => {
                        let received = branch.recv::<PublicationA>().await?;
                        let admitted = imp::decode(received.as_bytes())?;
                        let arguments = &admitted["arguments"];
                        imp::publication_inputs(
                            arguments,
                            &current,
                            ("module_a", &admitted["review"], &admitted["test"]),
                        )?;
                        let value = effects.apply(received).await?;
                        imp::published(&value, &current, "module_a")?;
                        let bytes = imp::encode(&value)?;
                        repo.send::<PublishedA>(&bytes.as_slice()).await?;
                    }
                    PublicationB::LOGICAL_LABEL => {
                        let received = branch.recv::<PublicationB>().await?;
                        let admitted = imp::decode(received.as_bytes())?;
                        let arguments = &admitted["arguments"];
                        imp::publication_inputs(
                            arguments,
                            &current,
                            ("module_b", &admitted["review"], &admitted["test"]),
                        )?;
                        let value = effects.apply(received).await?;
                        imp::published(&value, &current, "module_b")?;
                        let bytes = imp::encode(&value)?;
                        repo.send::<PublishedB>(&bytes.as_slice()).await?;
                    }
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected repository operation {label}"
                        )));
                    }
                }
            }
        };
        Ok(())
    }
    .await
}
