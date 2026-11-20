//! Actual objects and resource Endpoints have one owner. No permission table.
use super::super::{global::*, imp};
use crate::shell::{
    ShellError, ShellResult,
    imp::{review_reply::ReviewReply, service::PairedEffects},
};
use hibana::{Endpoint, g::Message};
use serde_json::Value;

#[hibana_runtime::scope]
pub async fn run<'r>(
    endpoint: &mut Endpoint<'r, COMPILER>,
    (mut core, mut config, mut models): (
        Endpoint<'r, CORE>,
        Endpoint<'r, CONFIG>,
        Endpoint<'r, MODELS>,
    ),
    effects: &impl PairedEffects,
) -> ShellResult<()> {
    let mut builds = [Value::Null, Value::Null, Value::Null];
    let (objects, tests) = loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            LoanCoreA::LOGICAL_LABEL | LoanConfigA::LOGICAL_LABEL => {
                drop(branch);
                async {
                    let [mut library, mut configuration] = endpoint.par()?;
                    async {
                        library.send::<LoanCoreA>(core).await?;
                    };
                    async {
                        configuration.send::<LoanConfigA>(config).await?;
                    };
                    Ok::<(), ShellError>(())
                }
                .await?;
                let admitted = imp::arguments(imp::decode(endpoint.recv::<CompileA>().await?)?, 0)?;
                let result = {
                    let [mut library, mut configuration] = endpoint.par()?;
                    let (first, second) = async {
                        async { library.recv::<CompileCore<A>>().await };
                        async { configuration.recv::<CompileConfig<A>>().await };
                    }
                    .await?;
                    if imp::decode(first.as_bytes())? != admitted
                        || imp::decode(second.as_bytes())? != admitted
                    {
                        return Err(ShellError::message(
                            "compile resources differ from admitted module A input",
                        ));
                    }
                    effects.apply_pair(first, second).await?
                };
                imp::object(&result, 0)?;
                builds[0] = result.clone();
                let bytes = imp::encode(&result)?;
                async {
                    let [mut library, mut configuration] = endpoint.par()?;
                    async {
                        library.send::<CoreBuilt>(&bytes.as_slice()).await?;
                    };
                    async {
                        configuration.send::<ConfigBuilt>(&bytes.as_slice()).await?;
                    };
                    Ok::<(), ShellError>(())
                }
                .await?;
                (core, config) = async {
                    let [mut library, mut configuration] = endpoint.par()?;
                    async { Ok::<_, ShellError>(library.recv::<ReturnCore>().await?) };
                    async { Ok::<_, ShellError>(configuration.recv::<ReturnConfig>().await?) };
                }
                .await?;
                if result["data"]["artifacts_modified"] == true {
                    async {
                        let [mut b, mut c] = endpoint.par()?;
                        async {
                            b.send::<UpdateB>(&bytes.as_slice()).await?;
                        };
                        async {
                            c.send::<UpdateC>(&bytes.as_slice()).await?;
                        };
                        Ok::<(), ShellError>(())
                    }
                    .await?;
                }
            }
            LoanCoreB::LOGICAL_LABEL | LoanModelsB::LOGICAL_LABEL => {
                drop(branch);
                async {
                    let [mut library, mut definitions] = endpoint.par()?;
                    async {
                        library.send::<LoanCoreB>(core).await?;
                    };
                    async {
                        definitions.send::<LoanModelsB>(models).await?;
                    };
                    Ok::<(), ShellError>(())
                }
                .await?;
                let admitted = imp::arguments(imp::decode(endpoint.recv::<CompileB>().await?)?, 1)?;
                let result = {
                    let [mut library, mut definitions] = endpoint.par()?;
                    let (first, second) = async {
                        async { library.recv::<CompileCore<B>>().await };
                        async { definitions.recv::<CompileModels<B>>().await };
                    }
                    .await?;
                    if imp::decode(first.as_bytes())? != admitted
                        || imp::decode(second.as_bytes())? != admitted
                    {
                        return Err(ShellError::message(
                            "compile resources differ from admitted module B input",
                        ));
                    }
                    effects.apply_pair(first, second).await?
                };
                imp::object(&result, 1)?;
                builds[1] = result.clone();
                let bytes = imp::encode(&result)?;
                async {
                    let [mut library, mut definitions] = endpoint.par()?;
                    async {
                        library.send::<CoreBuilt>(&bytes.as_slice()).await?;
                    };
                    async {
                        definitions.send::<ModelsBuilt>(&bytes.as_slice()).await?;
                    };
                    Ok::<(), ShellError>(())
                }
                .await?;
                (core, models) = async {
                    let [mut library, mut definitions] = endpoint.par()?;
                    async { Ok::<_, ShellError>(library.recv::<ReturnCore>().await?) };
                    async { Ok::<_, ShellError>(definitions.recv::<ReturnModels>().await?) };
                }
                .await?;
                if result["data"]["artifacts_modified"] == true {
                    async {
                        let [mut a, mut c] = endpoint.par()?;
                        async {
                            a.send::<UpdateA>(&bytes.as_slice()).await?;
                        };
                        async {
                            c.send::<UpdateC>(&bytes.as_slice()).await?;
                        };
                        Ok::<(), ShellError>(())
                    }
                    .await?;
                }
            }
            LoanConfigC::LOGICAL_LABEL | LoanModelsC::LOGICAL_LABEL => {
                drop(branch);
                async {
                    let [mut configuration, mut definitions] = endpoint.par()?;
                    async {
                        configuration.send::<LoanConfigC>(config).await?;
                    };
                    async {
                        definitions.send::<LoanModelsC>(models).await?;
                    };
                    Ok::<(), ShellError>(())
                }
                .await?;
                let admitted = imp::arguments(imp::decode(endpoint.recv::<CompileC>().await?)?, 2)?;
                let result = {
                    let [mut configuration, mut definitions] = endpoint.par()?;
                    let (first, second) = async {
                        async { configuration.recv::<CompileConfig<C>>().await };
                        async { definitions.recv::<CompileModels<C>>().await };
                    }
                    .await?;
                    if imp::decode(first.as_bytes())? != admitted
                        || imp::decode(second.as_bytes())? != admitted
                    {
                        return Err(ShellError::message(
                            "compile resources differ from admitted module C input",
                        ));
                    }
                    effects.apply_pair(first, second).await?
                };
                imp::object(&result, 2)?;
                builds[2] = result.clone();
                let bytes = imp::encode(&result)?;
                async {
                    let [mut configuration, mut definitions] = endpoint.par()?;
                    async {
                        configuration.send::<ConfigBuilt>(&bytes.as_slice()).await?;
                    };
                    async {
                        definitions.send::<ModelsBuilt>(&bytes.as_slice()).await?;
                    };
                    Ok::<(), ShellError>(())
                }
                .await?;
                (config, models) = async {
                    let [mut configuration, mut definitions] = endpoint.par()?;
                    async { Ok::<_, ShellError>(configuration.recv::<ReturnConfig>().await?) };
                    async { Ok::<_, ShellError>(definitions.recv::<ReturnModels>().await?) };
                }
                .await?;
                if result["data"]["artifacts_modified"] == true {
                    async {
                        let [mut a, mut b] = endpoint.par()?;
                        async {
                            a.send::<UpdateA>(&bytes.as_slice()).await?;
                        };
                        async {
                            b.send::<UpdateB>(&bytes.as_slice()).await?;
                        };
                        Ok::<(), ShellError>(())
                    }
                    .await?;
                }
            }
            TestObjectA::LOGICAL_LABEL => {
                drop(branch);
                let bytes = imp::encode(&builds[0])?;
                endpoint.send::<TestObjectA>(&bytes.as_slice()).await?;
            }
            TestObjectB::LOGICAL_LABEL => {
                drop(branch);
                let bytes = imp::encode(&builds[1])?;
                endpoint.send::<TestObjectB>(&bytes.as_slice()).await?;
            }
            TestObjectC::LOGICAL_LABEL => {
                drop(branch);
                let bytes = imp::encode(&builds[2])?;
                endpoint.send::<TestObjectC>(&bytes.as_slice()).await?;
            }
            AllForCompiler::LOGICAL_LABEL => {
                let tests: [Value; 3] = imp::decode(branch.recv::<AllForCompiler>().await?)?
                    .as_array()
                    .and_then(|items| items.clone().try_into().ok())
                    .ok_or_else(|| {
                        ShellError::message("actual test receipts must cover three modules")
                    })?;
                let objects = [
                    imp::object(&builds[0], 0)?,
                    imp::object(&builds[1], 1)?,
                    imp::object(&builds[2], 2)?,
                ];
                if imp::rebuild_modules(&objects, &tests)?.is_empty() {
                    let proof = imp::finish(&objects, &tests)?;
                    let bytes =
                        imp::encode(&serde_json::json!({"objects":objects,"success":true}))?;
                    endpoint
                        .send::<ReadyCandidate>(&ReviewReply::from_bytes(&bytes)?)
                        .await?;
                    let accepted = imp::decode(endpoint.recv::<FinishChecks>().await?)?;
                    if accepted != proof {
                        return Err(ShellError::message(
                            "integrator changed the current verified candidate",
                        ));
                    }
                    break (objects, tests);
                }
                let bytes = imp::encode(&serde_json::json!({"objects":objects}))?;
                endpoint.send::<RepairCandidate>(&bytes.as_slice()).await?;
                let repair = endpoint.offer().await?;
                let (index, request) = match repair.label() {
                    RetryAForCompiler::LOGICAL_LABEL => {
                        (0, imp::decode(repair.recv::<RetryAForCompiler>().await?)?)
                    }
                    RetryBForCompiler::LOGICAL_LABEL => {
                        (1, imp::decode(repair.recv::<RetryBForCompiler>().await?)?)
                    }
                    RetryCForCompiler::LOGICAL_LABEL => {
                        (2, imp::decode(repair.recv::<RetryCForCompiler>().await?)?)
                    }
                    _ => return Err(ShellError::message("unexpected rebuild selection")),
                };
                if request != imp::reconsider(&objects, &tests, index)? {
                    return Err(ShellError::message(
                        "rebuild request does not match the current actual object and test",
                    ));
                }
            }
            _ => return Err(ShellError::message("unexpected compiler operation")),
        }
    };
    async {
        let [mut library, mut configuration, mut definitions] = endpoint.par()?;
        async {
            library.send::<FrozenCore>(core).await?;
        };
        async {
            configuration.send::<FrozenConfig>(config).await?;
        };
        async {
            definitions.send::<FrozenModels>(models).await?;
        };
        Ok::<(), ShellError>(())
    }
    .await?;
    let received = endpoint.recv::<Link>().await?;
    let result = effects.apply(received).await?;
    imp::linked(&result, &objects, &tests)?;
    let bytes = imp::encode(&result)?;
    endpoint.send::<Linked>(&bytes.as_slice()).await?;
    Ok(())
}
