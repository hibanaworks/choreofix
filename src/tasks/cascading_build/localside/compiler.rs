//! Actual objects and resource Endpoints have one owner. No permission table.
use super::super::{global::*, imp};
use crate::shell::{
    ShellError, ShellResult,
    imp::{review_reply::ReviewReply, service::Domain},
};
use hibana::{Endpoint, g::Message};
use serde_json::Value;

#[hibana_runtime::scope]
pub async fn run<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, COMPILER>,
    (mut core, mut config, mut models): (
        Endpoint<'r, CORE>,
        Endpoint<'r, CONFIG>,
        Endpoint<'r, MODELS>,
    ),
    domain: &D,
) -> ShellResult<()> {
    let mut objects = [Value::Null, Value::Null, Value::Null];
    // Three initial builds precede the known aggregate receive. Rebuilds
    // below repeat only the selected resource operation, not this prefix.
    for _ in 0..3 {
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
                let (library, configuration) = async {
                    let [mut library, mut configuration] = endpoint.par()?;
                    async { imp::decode(library.recv::<CompileCore>().await?) };
                    async { imp::decode(configuration.recv::<CompileConfig>().await?) };
                }
                .await?;
                if library != admitted || configuration != admitted {
                    return Err(ShellError::message(
                        "compile resources differ from admitted module A input",
                    ));
                }
                let result = domain
                    .execute("BUILDER_A", "compile_module", admitted)
                    .await
                    .map_err(ShellError::message)?;
                objects[0] = imp::object(&result, 0)?;
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
                endpoint.send::<TestObjectA>(&bytes.as_slice()).await?;
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
                let (library, definitions) = async {
                    let [mut library, mut definitions] = endpoint.par()?;
                    async { imp::decode(library.recv::<CompileCore>().await?) };
                    async { imp::decode(definitions.recv::<CompileModels>().await?) };
                }
                .await?;
                if library != admitted || definitions != admitted {
                    return Err(ShellError::message(
                        "compile resources differ from admitted module B input",
                    ));
                }
                let result = domain
                    .execute("BUILDER_B", "compile_module", admitted)
                    .await
                    .map_err(ShellError::message)?;
                objects[1] = imp::object(&result, 1)?;
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
                endpoint.send::<TestObjectB>(&bytes.as_slice()).await?;
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
                let (configuration, definitions) = async {
                    let [mut configuration, mut definitions] = endpoint.par()?;
                    async { imp::decode(configuration.recv::<CompileConfig>().await?) };
                    async { imp::decode(definitions.recv::<CompileModels>().await?) };
                }
                .await?;
                if configuration != admitted || definitions != admitted {
                    return Err(ShellError::message(
                        "compile resources differ from admitted module C input",
                    ));
                }
                let result = domain
                    .execute("BUILDER_C", "compile_module", admitted)
                    .await
                    .map_err(ShellError::message)?;
                objects[2] = imp::object(&result, 2)?;
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
                endpoint.send::<TestObjectC>(&bytes.as_slice()).await?;
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
            _ => return Err(ShellError::message("unexpected initial compilation")),
        }
    }
    let tests = loop {
        let tests: [Value; 3] = imp::decode(endpoint.recv::<AllForCompiler>().await?)?
            .as_array()
            .and_then(|items| items.clone().try_into().ok())
            .ok_or_else(|| ShellError::message("actual test receipts must cover three modules"))?;
        if imp::needed(&objects, &tests)?.is_none() {
            let proof = imp::finish(&objects, &tests)?;
            let bytes = imp::encode(&proof)?;
            endpoint
                .send::<ReadyCandidate>(&ReviewReply::from_bytes(&bytes)?)
                .await?;
            let accepted = imp::decode(endpoint.recv::<FinishChecks>().await?)?;
            if accepted != proof {
                return Err(ShellError::message(
                    "integrator changed the current verified candidate",
                ));
            }
            break tests;
        }
        let bytes = imp::encode(&imp::candidate(&objects, &tests)?)?;
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
                let (library, configuration) = async {
                    let [mut library, mut configuration] = endpoint.par()?;
                    async { imp::decode(library.recv::<CompileCore>().await?) };
                    async { imp::decode(configuration.recv::<CompileConfig>().await?) };
                }
                .await?;
                if library != admitted || configuration != admitted {
                    return Err(ShellError::message(
                        "compile resources differ from admitted module A input",
                    ));
                }
                let result = domain
                    .execute("BUILDER_A", "compile_module", admitted)
                    .await
                    .map_err(ShellError::message)?;
                objects[0] = imp::object(&result, 0)?;
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
                endpoint.send::<TestObjectA>(&bytes.as_slice()).await?;
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
                let (library, definitions) = async {
                    let [mut library, mut definitions] = endpoint.par()?;
                    async { imp::decode(library.recv::<CompileCore>().await?) };
                    async { imp::decode(definitions.recv::<CompileModels>().await?) };
                }
                .await?;
                if library != admitted || definitions != admitted {
                    return Err(ShellError::message(
                        "compile resources differ from admitted module B input",
                    ));
                }
                let result = domain
                    .execute("BUILDER_B", "compile_module", admitted)
                    .await
                    .map_err(ShellError::message)?;
                objects[1] = imp::object(&result, 1)?;
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
                endpoint.send::<TestObjectB>(&bytes.as_slice()).await?;
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
                let (configuration, definitions) = async {
                    let [mut configuration, mut definitions] = endpoint.par()?;
                    async { imp::decode(configuration.recv::<CompileConfig>().await?) };
                    async { imp::decode(definitions.recv::<CompileModels>().await?) };
                }
                .await?;
                if configuration != admitted || definitions != admitted {
                    return Err(ShellError::message(
                        "compile resources differ from admitted module C input",
                    ));
                }
                let result = domain
                    .execute("BUILDER_C", "compile_module", admitted)
                    .await
                    .map_err(ShellError::message)?;
                objects[2] = imp::object(&result, 2)?;
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
                endpoint.send::<TestObjectC>(&bytes.as_slice()).await?;
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
            _ => return Err(ShellError::message("unexpected compiler continuation")),
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
    let arguments = imp::link_arguments(imp::decode(endpoint.recv::<Link>().await?)?)?;
    let result = domain
        .execute("INTEGRATOR", "link_modules", arguments)
        .await
        .map_err(ShellError::message)?;
    imp::linked(&result, &objects, &tests)?;
    let bytes = imp::encode(&result)?;
    endpoint.send::<Linked>(&bytes.as_slice()).await?;
    Ok(())
}
