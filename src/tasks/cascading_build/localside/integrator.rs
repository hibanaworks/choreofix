//! The integrator guest selects an eligible rebuild from the actual candidate.
use super::super::wire::{BuildRequest, RepairNotice};
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::review_reply::ReviewReply};
use hibana::Endpoint;
use serde_json::json;
#[hibana_runtime::scope]
pub async fn run(endpoint: &mut Endpoint<INTEGRATOR>) -> ShellResult<()> {
    let (objects, tests) = loop {
        let tests: [serde_json::Value; 3] =
            serde_json::from_slice(endpoint.recv::<AllForIntegrator>().await?)
                .map_err(|e| ShellError::message(e.to_string()))?;
        let objects = tests
            .each_ref()
            .map(|test| test["data"]["input_object"].clone());
        let eligible = imp::rebuild_modules(&objects, &tests)?;
        if eligible.is_empty() {
            let actual = imp::decode(endpoint.recv::<ReadyCandidate>().await?)?;
            if actual != json!({"objects": objects, "success": true}) {
                return Err(ShellError::message(
                    "compiler objects differ from the positively tested inputs",
                ));
            }
            let bytes = imp::encode(&imp::finish(&objects, &tests)?)?;
            endpoint
                .send::<FinishChecks>(&ReviewReply::from_bytes(&bytes)?)
                .await?;
            break (objects, tests);
        }
        let actual = imp::decode(endpoint.recv::<RepairCandidate>().await?)?;
        if actual != json!({"objects": objects}) {
            return Err(ShellError::message(
                "compiler objects differ from the tested repair inputs",
            ));
        }
        let mut notice = imp::candidate(&objects, &tests)?;
        notice["success"] = json!(false);
        notice["rebuild"] = json!(eligible);
        let index = loop {
            let bytes = imp::encode(&notice)?;
            endpoint.send::<FileRepair>(&RepairNotice(&bytes)).await?;
            let choice = endpoint.recv::<ChooseRepair>().await?;
            if let Some(index) = imp::requested_module(choice)
                && eligible.contains(&imp::MODULES[index])
            {
                break index;
            }
            notice["error"] = json!("choose a module listed in rebuild");
        };
        let bytes = imp::encode(&imp::reconsider(&objects, &tests, index)?)?;
        match index {
            0 => {
                endpoint
                    .send::<RetryAForCompiler>(&bytes.as_slice())
                    .await?
            }
            1 => {
                endpoint
                    .send::<RetryBForCompiler>(&bytes.as_slice())
                    .await?
            }
            _ => {
                endpoint
                    .send::<RetryCForCompiler>(&bytes.as_slice())
                    .await?
            }
        }
        async {
            let [mut selected, mut first_peer, mut second_peer] = endpoint.par()?;
            match index {
                0 => {
                    async {
                        async {
                            selected.send::<RetryA>(&bytes.as_slice()).await?;
                        };
                        async {
                            first_peer.send::<RetryAToB>(&bytes.as_slice()).await?;
                        };
                        async {
                            second_peer.send::<RetryAToC>(&bytes.as_slice()).await?;
                        };
                        Ok::<(), ShellError>(())
                    }
                    .await?
                }
                1 => {
                    async {
                        async {
                            selected.send::<RetryB>(&bytes.as_slice()).await?;
                        };
                        async {
                            first_peer.send::<RetryBToA>(&bytes.as_slice()).await?;
                        };
                        async {
                            second_peer.send::<RetryBToC>(&bytes.as_slice()).await?;
                        };
                        Ok::<(), ShellError>(())
                    }
                    .await?
                }
                _ => {
                    async {
                        async {
                            selected.send::<RetryC>(&bytes.as_slice()).await?;
                        };
                        async {
                            first_peer.send::<RetryCToA>(&bytes.as_slice()).await?;
                        };
                        async {
                            second_peer.send::<RetryCToB>(&bytes.as_slice()).await?;
                        };
                        Ok::<(), ShellError>(())
                    }
                    .await?
                }
            }
            Ok::<(), ShellError>(())
        }
        .await?;
    };
    let (_core, _config, _models) = async {
        let [mut library, mut configuration, mut definitions] = endpoint.par()?;
        async { Ok::<_, ShellError>(library.recv::<FrozenCore>().await?) };
        async { Ok::<_, ShellError>(configuration.recv::<FrozenConfig>().await?) };
        async { Ok::<_, ShellError>(definitions.recv::<FrozenModels>().await?) };
    }
    .await?;
    let proof = imp::encode(&imp::finish(&objects, &tests)?)?;
    endpoint
        .send::<FileReady>(&ReviewReply::from_bytes(&proof)?)
        .await?;
    let arguments = endpoint.recv::<LinkFile>().await?.into_value();
    let bytes = imp::encode(&arguments)?;
    endpoint.send::<Link>(&BuildRequest(&bytes)).await?;
    let result = imp::decode(endpoint.recv::<Linked>().await?)?;
    imp::linked(&result, &objects, &tests)?;
    let bytes = imp::encode(&result)?;
    endpoint.send::<FileLinked>(&bytes.as_slice()).await?;
    async {
        let [mut a, mut b, mut c, mut tester, mut service] = endpoint.par()?;
        async {
            a.send::<FinishedA>(&bytes.as_slice()).await?;
        };
        async {
            b.send::<FinishedB>(&bytes.as_slice()).await?;
        };
        async {
            c.send::<FinishedC>(&bytes.as_slice()).await?;
        };
        async {
            tester.send::<FinishedTester>(&bytes.as_slice()).await?;
        };
        async {
            service.send::<FinishedService>(&bytes.as_slice()).await?;
        };
        Ok::<(), ShellError>(())
    }
    .await?;

    Ok(())
}
