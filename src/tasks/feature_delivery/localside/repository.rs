//! The repository owns actual candidate, review and publication data.
//! Known continuations use recv; offer is reserved for projected choices.
use super::super::{
    global::{delivery::*, *},
    imp,
};
use crate::shell::{
    ShellError, ShellResult,
    imp::{review_reply::ReviewReply, service::Domain},
};
use hibana::{Endpoint, g::Message};
use serde_json::Value;

#[hibana_runtime::scope]
pub async fn run<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, REPOSITORY>,
    (mut repository, environment, feature_a, feature_b): (
        Endpoint<'r, REPO>,
        Endpoint<'r, TEST_ENV>,
        Endpoint<'r, AUTH_A>,
        Endpoint<'r, AUTH_B>,
    ),
    domain: &D,
) -> ShellResult<(
    Endpoint<'r, REPO>,
    Endpoint<'r, TEST_ENV>,
    Endpoint<'r, AUTH_A>,
    Endpoint<'r, AUTH_B>,
)> {
    endpoint.send::<Feature<AUTH_A>>(feature_a).await?;
    endpoint.send::<Feature<AUTH_B>>(feature_b).await?;
    endpoint.send::<TestLoan>(environment).await?;
    let branch = endpoint.offer().await?;
    let first_review = match branch.label() {
        ReviewFirstA::LOGICAL_LABEL => {
            let arguments = branch.recv::<ReviewFirstA>().await?.into_value();
            imp::arguments(&arguments, ("pr_id", FEATURE_A))?;
            arguments
        }
        ReviewFirstB::LOGICAL_LABEL => {
            let arguments = branch.recv::<ReviewFirstB>().await?.into_value();
            imp::arguments(&arguments, ("pr_id", FEATURE_B))?;
            arguments
        }
        _ => return Err(ShellError::message("expected the selected review request")),
    };
    let order = if first_review["pr_id"] == FEATURE_A {
        [FEATURE_A, FEATURE_B]
    } else {
        [FEATURE_B, FEATURE_A]
    };
    let mut candidates = [Value::Null, Value::Null];
    let mut reviews = [Value::Null, Value::Null];
    let mut feature_a = None;
    let mut feature_b = None;
    for feature in order {
        let (index, agent) = if feature == FEATURE_A {
            (0, "DEVELOPER_A")
        } else {
            (1, "DEVELOPER_B")
        };
        let admitted = if feature == FEATURE_A {
            endpoint.recv::<ImplementA>().await?.into_value()
        } else {
            endpoint.recv::<ImplementB>().await?.into_value()
        };
        imp::arguments(&admitted, ("feature_name", feature))?;
        endpoint.send::<RepoLoan>(repository).await?;
        let (authority, arguments) = async {
            let [mut authority, mut repo] = endpoint.par()?;
            async { Ok::<_, ShellError>(authority.recv::<Implement>().await?.into_value()) };
            async { Ok::<_, ShellError>(repo.recv::<ImplementOnRepo>().await?.into_value()) };
        }
        .await?;
        imp::same(&authority, &admitted)?;
        imp::same(&arguments, &admitted)?;
        let result = domain
            .execute(agent, "implement_feature", arguments)
            .await
            .map_err(ShellError::message)?;
        imp::candidate(&result, feature)?;
        let bytes = imp::encode(&result)?;
        endpoint.send::<RepoResult>(&bytes.as_slice()).await?;
        repository = endpoint.recv::<RepoReturn>().await?;
        if feature == FEATURE_A {
            endpoint
                .send::<CandidateForReviewA>(&bytes.as_slice())
                .await?;
            endpoint
                .send::<CandidateForTestsA>(&bytes.as_slice())
                .await?;
        } else {
            endpoint
                .send::<CandidateForReviewB>(&bytes.as_slice())
                .await?;
            endpoint
                .send::<CandidateForTestsB>(&bytes.as_slice())
                .await?;
        }
        candidates[index] = result;
    }
    for feature in order {
        let (index, agent) = if feature == FEATURE_A {
            (0, "DEVELOPER_A")
        } else {
            (1, "DEVELOPER_B")
        };
        loop {
            let result = if feature == FEATURE_A {
                imp::decode(endpoint.recv::<ReviewResultA>().await?)?
            } else {
                imp::decode(endpoint.recv::<ReviewResultB>().await?)?
            };
            imp::reviewed(&result, &candidates[index], feature)?;
            let branch = endpoint.offer().await?;
            match branch.label() {
                ReviewApproved::LOGICAL_LABEL => {
                    imp::same(
                        &imp::decode(branch.recv::<ReviewApproved>().await?)?,
                        &result,
                    )?;
                    if feature == FEATURE_A {
                        feature_a = Some(endpoint.recv::<Feature<AUTH_A>>().await?);
                    } else {
                        feature_b = Some(endpoint.recv::<Feature<AUTH_B>>().await?);
                    }
                    reviews[index] = result;
                    break;
                }
                ReviewRejected::LOGICAL_LABEL => {
                    imp::same(
                        &imp::decode(branch.recv::<ReviewRejected>().await?)?,
                        &result,
                    )?;
                    reviews[index] = result;
                }
                _ => {
                    return Err(ShellError::message(
                        "expected this candidate's initial review outcome",
                    ));
                }
            }
            let admitted = if feature == FEATURE_A {
                endpoint.recv::<ImplementA>().await?.into_value()
            } else {
                endpoint.recv::<ImplementB>().await?.into_value()
            };
            imp::arguments(&admitted, ("feature_name", feature))?;
            endpoint.send::<RepoLoan>(repository).await?;
            let (authority, arguments) = async {
                let [mut authority, mut repo] = endpoint.par()?;
                async { Ok::<_, ShellError>(authority.recv::<Implement>().await?.into_value()) };
                async { Ok::<_, ShellError>(repo.recv::<ImplementOnRepo>().await?.into_value()) };
            }
            .await?;
            imp::same(&authority, &admitted)?;
            imp::same(&arguments, &admitted)?;
            let result = domain
                .execute(agent, "implement_feature", arguments)
                .await
                .map_err(ShellError::message)?;
            imp::candidate(&result, feature)?;
            let bytes = imp::encode(&result)?;
            endpoint.send::<RepoResult>(&bytes.as_slice()).await?;
            repository = endpoint.recv::<RepoReturn>().await?;
            if feature == FEATURE_A {
                endpoint
                    .send::<CandidateForReviewA>(&bytes.as_slice())
                    .await?;
                endpoint
                    .send::<CandidateForTestsA>(&bytes.as_slice())
                    .await?;
            } else {
                endpoint
                    .send::<CandidateForReviewB>(&bytes.as_slice())
                    .await?;
                endpoint
                    .send::<CandidateForTestsB>(&bytes.as_slice())
                    .await?;
            }
            candidates[index] = result;
        }
    }
    let mut prior_ci = Value::Null;
    for feature in order {
        let (index, agent) = if feature == FEATURE_A {
            (0, "DEVELOPER_A")
        } else {
            (1, "DEVELOPER_B")
        };
        if feature == FEATURE_A {
            endpoint
                .send::<MergeLoanA>(
                    feature_a.take().ok_or_else(|| {
                        ShellError::message("missing approved feature A Endpoint")
                    })?,
                )
                .await?;
        } else {
            endpoint
                .send::<MergeLoanB>(
                    feature_b.take().ok_or_else(|| {
                        ShellError::message("missing approved feature B Endpoint")
                    })?,
                )
                .await?;
        }
        loop {
            let admitted = if feature == FEATURE_A {
                endpoint.recv::<MergeAdmissionA>().await?.into_value()
            } else {
                endpoint.recv::<MergeAdmissionB>().await?.into_value()
            };
            imp::arguments(&admitted, ("branch", feature))?;
            endpoint.send::<RepoLoan>(repository).await?;
            let (authority, arguments) = async {
                let [mut authority, mut repo] = endpoint.par()?;
                async { Ok::<_, ShellError>(authority.recv::<Merge>().await?.into_value()) };
                async { Ok::<_, ShellError>(repo.recv::<MergeOnRepo>().await?.into_value()) };
            }
            .await?;
            imp::merge_inputs(
                &admitted,
                (&authority, &arguments),
                (feature, &candidates[index], &reviews[index]),
            )?;
            let publication = domain
                .execute(agent, "merge_code", arguments)
                .await
                .map_err(ShellError::message)?;
            imp::merge_result(
                &publication,
                (feature, &candidates[index], &reviews[index]),
                &prior_ci,
            )?;
            let bytes = imp::encode(&publication)?;
            endpoint.send::<Merged>(&bytes.as_slice()).await?;
            repository = endpoint.recv::<RepoReturn>().await?;
            endpoint.send::<Merged>(&bytes.as_slice()).await?;
            let result = imp::decode(endpoint.recv::<CiResult>().await?)?;
            imp::ci_publication(&result, &publication)?;
            let bytes = imp::encode(&result)?;
            let branch = endpoint.offer().await?;
            match branch.label() {
                IntegratedA::LOGICAL_LABEL => {
                    imp::arguments(&publication["data"], ("feature", FEATURE_A))?;
                    imp::same(&imp::decode(branch.recv::<IntegratedA>().await?)?, &result)?;
                    let mut authority = endpoint.recv::<Feature<AUTH_A>>().await?;
                    let reply = ReviewReply::<true>::from_bytes(&bytes)?;
                    if first_review["pr_id"] == FEATURE_A {
                        authority.send::<CiPassedA>(&reply).await?;
                        authority.send::<CiPassedA>(&reply).await?;
                        authority.send::<CiPassedA>(&reply).await?;
                        authority.send::<CiPassedA>(&reply).await?;
                        authority.send::<CiPassedA>(&reply).await?;
                        authority.send::<CiPassedA>(&reply).await?;
                        authority.send::<CiPassedA>(&reply).await?;
                        authority.send::<CiPassedA>(&reply).await?;
                    } else {
                        authority.send::<LastCiPassedA>(&reply).await?;
                        authority.send::<LastCiPassedA>(&reply).await?;
                        authority.send::<LastCiPassedA>(&reply).await?;
                        authority.send::<LastCiPassedA>(&reply).await?;
                        authority.send::<LastCiPassedA>(&reply).await?;
                        authority.send::<LastCiPassedA>(&reply).await?;
                        authority.send::<LastCiPassedA>(&reply).await?;
                        authority.send::<LastCiPassedA>(&reply).await?;
                    }
                    feature_a = Some(authority);
                    prior_ci = result;
                    break;
                }
                IntegrationFailedA::LOGICAL_LABEL => {
                    imp::arguments(&publication["data"], ("feature", FEATURE_A))?;
                    imp::same(
                        &imp::decode(branch.recv::<IntegrationFailedA>().await?)?,
                        &result,
                    )?;
                }
                IntegratedB::LOGICAL_LABEL => {
                    imp::arguments(&publication["data"], ("feature", FEATURE_B))?;
                    imp::same(&imp::decode(branch.recv::<IntegratedB>().await?)?, &result)?;
                    let mut authority = endpoint.recv::<Feature<AUTH_B>>().await?;
                    let reply = ReviewReply::<true>::from_bytes(&bytes)?;
                    if first_review["pr_id"] == FEATURE_B {
                        authority.send::<CiPassedB>(&reply).await?;
                        authority.send::<CiPassedB>(&reply).await?;
                        authority.send::<CiPassedB>(&reply).await?;
                        authority.send::<CiPassedB>(&reply).await?;
                        authority.send::<CiPassedB>(&reply).await?;
                        authority.send::<CiPassedB>(&reply).await?;
                        authority.send::<CiPassedB>(&reply).await?;
                        authority.send::<CiPassedB>(&reply).await?;
                    } else {
                        authority.send::<LastCiPassedB>(&reply).await?;
                        authority.send::<LastCiPassedB>(&reply).await?;
                        authority.send::<LastCiPassedB>(&reply).await?;
                        authority.send::<LastCiPassedB>(&reply).await?;
                        authority.send::<LastCiPassedB>(&reply).await?;
                        authority.send::<LastCiPassedB>(&reply).await?;
                        authority.send::<LastCiPassedB>(&reply).await?;
                        authority.send::<LastCiPassedB>(&reply).await?;
                    }
                    feature_b = Some(authority);
                    prior_ci = result;
                    break;
                }
                IntegrationFailedB::LOGICAL_LABEL => {
                    imp::arguments(&publication["data"], ("feature", FEATURE_B))?;
                    imp::same(
                        &imp::decode(branch.recv::<IntegrationFailedB>().await?)?,
                        &result,
                    )?;
                }
                _ => return Err(ShellError::message("expected the actual CI outcome")),
            }
            prior_ci = result;
            loop {
                let admitted = if feature == FEATURE_A {
                    endpoint.recv::<ImplementA>().await?.into_value()
                } else {
                    endpoint.recv::<ImplementB>().await?.into_value()
                };
                imp::arguments(&admitted, ("feature_name", feature))?;
                endpoint.send::<RepoLoan>(repository).await?;
                let (authority, arguments) = async {
                    let [mut authority, mut repo] = endpoint.par()?;
                    async {
                        Ok::<_, ShellError>(authority.recv::<Implement>().await?.into_value())
                    };
                    async {
                        Ok::<_, ShellError>(repo.recv::<ImplementOnRepo>().await?.into_value())
                    };
                }
                .await?;
                imp::same(&authority, &admitted)?;
                imp::same(&arguments, &admitted)?;
                let result = domain
                    .execute(agent, "implement_feature", arguments)
                    .await
                    .map_err(ShellError::message)?;
                imp::candidate(&result, feature)?;
                let bytes = imp::encode(&result)?;
                endpoint.send::<RepoResult>(&bytes.as_slice()).await?;
                repository = endpoint.recv::<RepoReturn>().await?;
                if feature == FEATURE_A {
                    endpoint
                        .send::<CandidateForReviewA>(&bytes.as_slice())
                        .await?;
                    endpoint
                        .send::<CandidateForTestsA>(&bytes.as_slice())
                        .await?;
                } else {
                    endpoint
                        .send::<CandidateForReviewB>(&bytes.as_slice())
                        .await?;
                    endpoint
                        .send::<CandidateForTestsB>(&bytes.as_slice())
                        .await?;
                }
                candidates[index] = result;
                let result = if feature == FEATURE_A {
                    imp::decode(endpoint.recv::<ReviewResultA>().await?)?
                } else {
                    imp::decode(endpoint.recv::<ReviewResultB>().await?)?
                };
                imp::reviewed(&result, &candidates[index], feature)?;
                let branch = endpoint.offer().await?;
                match branch.label() {
                    RecoveryReviewApproved::LOGICAL_LABEL => {
                        imp::same(
                            &imp::decode(branch.recv::<RecoveryReviewApproved>().await?)?,
                            &result,
                        )?;
                        reviews[index] = result;
                        break;
                    }
                    RecoveryReviewRejected::LOGICAL_LABEL => {
                        imp::same(
                            &imp::decode(branch.recv::<RecoveryReviewRejected>().await?)?,
                            &result,
                        )?;
                        reviews[index] = result;
                    }
                    _ => {
                        return Err(ShellError::message(
                            "expected this candidate's recovery review outcome",
                        ));
                    }
                }
            }
        }
    }
    let environment = endpoint.recv::<TestReturn>().await?;
    Ok((
        repository,
        environment,
        feature_a
            .take()
            .ok_or_else(|| ShellError::message("missing final feature A Endpoint"))?,
        feature_b
            .take()
            .ok_or_else(|| ShellError::message("missing final feature B Endpoint"))?,
    ))
}
