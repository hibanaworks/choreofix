//! Direct operation files and actual affine resource requests.
use super::super::wire::BuildRequest;
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult};
use hibana::{Endpoint, g::Message};
use serde_json::Value;
#[hibana_runtime::scope]
pub async fn run<const ROLE: u8>(endpoint: &mut Endpoint<ROLE>) -> ShellResult<()> {
    let index = match ROLE {
        A => 0,
        B => 1,
        C => 2,
        _ => return Err(ShellError::message("unknown cascading builder")),
    };
    let mut built = Value::Null;
    loop {
        let branch = endpoint.offer().await?;
        let bytes = match branch.label() {
            CompileFileA::LOGICAL_LABEL if ROLE == A => {
                let arguments = branch.recv::<CompileFileA>().await?.into_value();
                let (mut core, mut config) = async {
                    let [mut library, mut configuration] = endpoint.par()?;
                    async { Ok::<_, ShellError>(library.recv::<LoanCoreA>().await?) };
                    async { Ok::<_, ShellError>(configuration.recv::<LoanConfigA>().await?) };
                }
                .await?;
                let bytes = imp::encode(&arguments)?;
                endpoint.send::<CompileA>(&bytes.as_slice()).await?;
                let (first, second) = async {
                    async {
                        core.send::<CompileCore<A>>(&BuildRequest(&bytes)).await?;
                        imp::decode(core.recv::<CoreBuilt>().await?)
                    };
                    async {
                        config
                            .send::<CompileConfig<A>>(&BuildRequest(&bytes))
                            .await?;
                        imp::decode(config.recv::<ConfigBuilt>().await?)
                    };
                }
                .await?;
                if first != second {
                    return Err(ShellError::message(
                        "compile resources returned different results",
                    ));
                }
                imp::object(&first, index)?;
                async {
                    let [mut library, mut configuration] = endpoint.par()?;
                    async {
                        library.send::<ReturnCore>(core).await?;
                    };
                    async {
                        configuration.send::<ReturnConfig>(config).await?;
                    };
                    Ok::<(), ShellError>(())
                }
                .await?;
                built = first;
                let bytes = imp::encode(&built)?;
                endpoint.send::<FileCompiled>(&bytes.as_slice()).await?;
                continue;
            }
            CompileFileB::LOGICAL_LABEL if ROLE == B => {
                let arguments = branch.recv::<CompileFileB>().await?.into_value();
                let (mut core, mut models) = async {
                    let [mut library, mut definitions] = endpoint.par()?;
                    async { Ok::<_, ShellError>(library.recv::<LoanCoreB>().await?) };
                    async { Ok::<_, ShellError>(definitions.recv::<LoanModelsB>().await?) };
                }
                .await?;
                let bytes = imp::encode(&arguments)?;
                endpoint.send::<CompileB>(&bytes.as_slice()).await?;
                let (first, second) = async {
                    async {
                        core.send::<CompileCore<B>>(&BuildRequest(&bytes)).await?;
                        imp::decode(core.recv::<CoreBuilt>().await?)
                    };
                    async {
                        models
                            .send::<CompileModels<B>>(&BuildRequest(&bytes))
                            .await?;
                        imp::decode(models.recv::<ModelsBuilt>().await?)
                    };
                }
                .await?;
                if first != second {
                    return Err(ShellError::message(
                        "compile resources returned different results",
                    ));
                }
                imp::object(&first, index)?;
                async {
                    let [mut library, mut definitions] = endpoint.par()?;
                    async {
                        library.send::<ReturnCore>(core).await?;
                    };
                    async {
                        definitions.send::<ReturnModels>(models).await?;
                    };
                    Ok::<(), ShellError>(())
                }
                .await?;
                built = first;
                let bytes = imp::encode(&built)?;
                endpoint.send::<FileCompiled>(&bytes.as_slice()).await?;
                continue;
            }
            CompileFileC::LOGICAL_LABEL if ROLE == C => {
                let arguments = branch.recv::<CompileFileC>().await?.into_value();
                let (mut config, mut models) = async {
                    let [mut configuration, mut definitions] = endpoint.par()?;
                    async { Ok::<_, ShellError>(configuration.recv::<LoanConfigC>().await?) };
                    async { Ok::<_, ShellError>(definitions.recv::<LoanModelsC>().await?) };
                }
                .await?;
                let bytes = imp::encode(&arguments)?;
                endpoint.send::<CompileC>(&bytes.as_slice()).await?;
                let (first, second) = async {
                    async {
                        config
                            .send::<CompileConfig<C>>(&BuildRequest(&bytes))
                            .await?;
                        imp::decode(config.recv::<ConfigBuilt>().await?)
                    };
                    async {
                        models
                            .send::<CompileModels<C>>(&BuildRequest(&bytes))
                            .await?;
                        imp::decode(models.recv::<ModelsBuilt>().await?)
                    };
                }
                .await?;
                if first != second {
                    return Err(ShellError::message(
                        "compile resources returned different results",
                    ));
                }
                imp::object(&first, index)?;
                async {
                    let [mut configuration, mut definitions] = endpoint.par()?;
                    async {
                        configuration.send::<ReturnConfig>(config).await?;
                    };
                    async {
                        definitions.send::<ReturnModels>(models).await?;
                    };
                    Ok::<(), ShellError>(())
                }
                .await?;
                built = first;
                let bytes = imp::encode(&built)?;
                endpoint.send::<FileCompiled>(&bytes.as_slice()).await?;
                continue;
            }

            Feedback::LOGICAL_LABEL => {
                let result = imp::decode(branch.recv::<Feedback>().await?)?;
                imp::test_result(&result, index)?;
                if result["data"]["input_object"] != built["data"]["object"] {
                    return Err(ShellError::message(
                        "feedback names a different built object",
                    ));
                }
                let bytes = imp::encode(&result)?;
                endpoint.send::<FileFeedback>(&bytes.as_slice()).await?;
                continue;
            }
            UpdateA::LOGICAL_LABEL => branch.recv::<UpdateA>().await?.to_vec(),
            UpdateB::LOGICAL_LABEL => branch.recv::<UpdateB>().await?.to_vec(),
            UpdateC::LOGICAL_LABEL => branch.recv::<UpdateC>().await?.to_vec(),
            RetryAToB::LOGICAL_LABEL => branch.recv::<RetryAToB>().await?.to_vec(),
            RetryAToC::LOGICAL_LABEL => branch.recv::<RetryAToC>().await?.to_vec(),
            RetryBToA::LOGICAL_LABEL => branch.recv::<RetryBToA>().await?.to_vec(),
            RetryBToC::LOGICAL_LABEL => branch.recv::<RetryBToC>().await?.to_vec(),
            RetryCToA::LOGICAL_LABEL => branch.recv::<RetryCToA>().await?.to_vec(),
            RetryCToB::LOGICAL_LABEL => branch.recv::<RetryCToB>().await?.to_vec(),
            RetryA::LOGICAL_LABEL => {
                let bytes = branch.recv::<RetryA>().await?.to_vec();
                endpoint.send::<FileReconsidered>(&bytes.as_slice()).await?;
                continue;
            }
            RetryB::LOGICAL_LABEL => {
                let bytes = branch.recv::<RetryB>().await?.to_vec();
                endpoint.send::<FileReconsidered>(&bytes.as_slice()).await?;
                continue;
            }
            RetryC::LOGICAL_LABEL => {
                let bytes = branch.recv::<RetryC>().await?.to_vec();
                endpoint.send::<FileReconsidered>(&bytes.as_slice()).await?;
                continue;
            }
            FinishedA::LOGICAL_LABEL if ROLE == A => {
                let bytes = branch.recv::<FinishedA>().await?.to_vec();
                endpoint.send::<FileFinished>(&bytes.as_slice()).await?;
                return Ok(());
            }
            FinishedB::LOGICAL_LABEL if ROLE == B => {
                let bytes = branch.recv::<FinishedB>().await?.to_vec();
                endpoint.send::<FileFinished>(&bytes.as_slice()).await?;
                return Ok(());
            }
            FinishedC::LOGICAL_LABEL if ROLE == C => {
                let bytes = branch.recv::<FinishedC>().await?.to_vec();
                endpoint.send::<FileFinished>(&bytes.as_slice()).await?;
                return Ok(());
            }
            _ => {
                return Err(ShellError::message(
                    "unexpected cascading builder operation",
                ));
            }
        };
        endpoint.send::<FileUpdated>(&bytes.as_slice()).await?;
    }
}
