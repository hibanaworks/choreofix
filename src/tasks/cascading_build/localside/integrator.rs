//! Candidate data selects repair; the model owns the actual link request.
use super::super::wire::BuildRequest;
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::review_reply::ReviewReply};
use core::cell::Cell;
use hibana::{Endpoint, g::Message};
use serde_json::Value;
#[hibana_runtime::scope]
pub async fn run(endpoint: &mut Endpoint<INTEGRATOR>, input: &Cell<Value>) -> ShellResult<()> {
    let (objects, tests) = loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            RepairCandidate::LOGICAL_LABEL => {
                let actual = imp::decode(branch.recv::<RepairCandidate>().await?)?;
                let (objects, tests) = imp::candidate_parts(&actual)?;
                let index = imp::needed(&objects, &tests)?
                    .ok_or_else(|| ShellError::message("repair candidate is already current"))?;
                let bytes = imp::encode(&imp::reconsider(&objects, &tests, index)?)?;
                input.set(actual);
                drop(endpoint.offer().await?);
                async {
                    let [mut selected, mut compiler, mut first_peer, mut second_peer] =
                        endpoint.par()?;
                    match index {
                        0 => {
                            async {
                                async {
                                    selected.send::<RetryA>(&bytes.as_slice()).await?;
                                };
                                async {
                                    compiler
                                        .send::<RetryAForCompiler>(&bytes.as_slice())
                                        .await?;
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
                                    compiler
                                        .send::<RetryBForCompiler>(&bytes.as_slice())
                                        .await?;
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
                                    compiler
                                        .send::<RetryCForCompiler>(&bytes.as_slice())
                                        .await?;
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
            }
            ReadyCandidate::LOGICAL_LABEL => {
                let actual = imp::decode(branch.recv::<ReadyCandidate>().await?)?;
                let (objects, tests) = imp::candidate_parts(&actual)?;
                if imp::finish(&objects, &tests)? != actual {
                    return Err(ShellError::message(
                        "ready candidate differs from actual successful checks",
                    ));
                }
                let bytes = imp::encode(&actual)?;
                endpoint
                    .send::<FinishChecks>(&ReviewReply::from_bytes(&bytes)?)
                    .await?;
                break (objects, tests);
            }
            _ => return Err(ShellError::message("unexpected integrator candidate")),
        }
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
