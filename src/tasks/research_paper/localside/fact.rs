use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult};
use hibana::{EndpointBorrow, g::Message};

#[hibana_runtime::scope]
pub async fn run<const AUTHOR: u8, const SECTION: u8, C: ChapterMessages<AUTHOR>>(
    endpoint: &mut EndpointBorrow<'_, '_, FACT_CHECKER>,
) -> ShellResult<()> {
    let (mut written, references) = async {
        let [mut document, mut database] = endpoint.par()?;
        async { Ok::<_, ShellError>(document.recv::<C::MaterialForFact>().await?.into_value()) };
        async { Ok::<_, ShellError>(database.recv::<C::ReferencesForFact>().await?.into_value()) };
    }
    .await?;
    let mut section = endpoint.recv::<Section<SECTION>>().await?;
    loop {
        let snapshot = imp::snapshot(AUTHOR, &written, &references)?;
        let material = imp::Material::new(snapshot.clone())?;
        endpoint.send::<C::FactInput>(&material).await?;
        endpoint.send::<C::FactMaterial>(&material).await?;
        endpoint.send::<Section<SECTION>>(section).await?;
        let branch = endpoint.offer().await?;
        match branch.label() {
            label if label == C::FactRejected::LOGICAL_LABEL => {
                let actual = branch.recv::<C::FactRejected>().await?.into_value();
                imp::checked(AUTHOR, &actual, &snapshot)?;
                written = endpoint.recv::<C::RevisedForFact>().await?.into_value();
            }
            label if label == C::FactPassed::LOGICAL_LABEL => {
                let facts = branch.recv::<C::FactPassed>().await?.into_value();
                imp::checked(AUTHOR, &facts, &snapshot)?;
                let branch = endpoint.offer().await?;
                match branch.label() {
                    label if label == Section::<SECTION>::LOGICAL_LABEL => {
                        let section = branch.recv::<Section<SECTION>>().await?;
                        let reviewed = endpoint.recv::<C::FinalForFact>().await?.into_value();
                        imp::reviewed(AUTHOR, &reviewed, (&snapshot, &facts))?;
                        let approval = imp::Verdict::<AUTHOR, 6, true>::new(reviewed)?;
                        async {
                            let [mut archive, mut file] = endpoint.par()?;
                            async {
                                archive.send::<Section<SECTION>>(section).await?;
                            };
                            async {
                                file.send::<C::FactFinished>(&approval).await?;
                            };
                            Ok::<(), ShellError>(())
                        }
                        .await?;
                        return Ok(());
                    }
                    label if label == C::RevisedForFact::LOGICAL_LABEL => {
                        written = branch.recv::<C::RevisedForFact>().await?.into_value();
                    }
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected factual continuation {label}"
                        )));
                    }
                }
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected factual verdict {label}"
                )));
            }
        }
        section = endpoint.recv::<Section<SECTION>>().await?;
    }
}
