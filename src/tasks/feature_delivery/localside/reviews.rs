//! The service owns the candidate and original review outcome, not a phase flag.
use super::super::{
    global::{delivery::*, *},
    imp,
};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::{Endpoint, g::Message};

pub async fn feature_a<D: Domain>(
    endpoint: &mut Endpoint<'_, REVIEW_A>,
    domain: &D,
) -> ShellResult<()> {
    loop {
        let candidate = imp::decode(endpoint.recv::<CandidateForReviewA>().await?)?;
        imp::candidate(&candidate, FEATURE_A)?;
        let arguments = endpoint.recv::<Review>().await?.into_value();
        imp::arguments(&arguments, ("pr_id", FEATURE_A))?;
        let result = domain
            .execute("REVIEWER", "code_review", arguments)
            .await
            .map_err(ShellError::message)?;
        imp::reviewed(&result, &candidate, FEATURE_A)?;
        let bytes = imp::encode(&result)?;
        endpoint.send::<ReviewResultA>(&bytes.as_slice()).await?;
        endpoint.send::<ReviewResultA>(&bytes.as_slice()).await?;
        let branch = endpoint.offer().await?;
        match branch.label() {
            ReviewApproved::LOGICAL_LABEL => {
                imp::same(
                    &imp::decode(branch.recv::<ReviewApproved>().await?)?,
                    &result,
                )?;
                break;
            }
            ReviewRejected::LOGICAL_LABEL => {
                imp::same(
                    &imp::decode(branch.recv::<ReviewRejected>().await?)?,
                    &result,
                )?;
            }
            _ => {
                return Err(ShellError::message(
                    "expected this review's original outcome",
                ));
            }
        }
    }
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            LastCiPassedA::LOGICAL_LABEL => {
                branch.recv::<LastCiPassedA>().await?;
                return Ok(());
            }
            CiPassedA::LOGICAL_LABEL => {
                branch.recv::<CiPassedA>().await?;
                return Ok(());
            }
            CiFailedA::LOGICAL_LABEL => {
                branch.recv::<CiFailedA>().await?;
            }
            _ => return Err(ShellError::message("expected feature A's CI outcome")),
        }
        loop {
            let candidate = imp::decode(endpoint.recv::<CandidateForReviewA>().await?)?;
            imp::candidate(&candidate, FEATURE_A)?;
            let admitted = endpoint.recv::<ReviewRequestA>().await?.into_value();
            imp::arguments(&admitted, ("pr_id", FEATURE_A))?;
            let arguments = endpoint.recv::<Review>().await?.into_value();
            imp::same(&arguments, &admitted)?;
            let result = domain
                .execute("REVIEWER", "code_review", arguments)
                .await
                .map_err(ShellError::message)?;
            imp::reviewed(&result, &candidate, FEATURE_A)?;
            let bytes = imp::encode(&result)?;
            endpoint.send::<ReviewResultA>(&bytes.as_slice()).await?;
            endpoint.send::<ReviewResultA>(&bytes.as_slice()).await?;
            let branch = endpoint.offer().await?;
            match branch.label() {
                RecoveryReviewApproved::LOGICAL_LABEL => {
                    imp::same(
                        &imp::decode(branch.recv::<RecoveryReviewApproved>().await?)?,
                        &result,
                    )?;
                    break;
                }
                RecoveryReviewRejected::LOGICAL_LABEL => {
                    imp::same(
                        &imp::decode(branch.recv::<RecoveryReviewRejected>().await?)?,
                        &result,
                    )?;
                }
                _ => {
                    return Err(ShellError::message(
                        "expected the recovery review's original outcome",
                    ));
                }
            }
        }
    }
}

pub async fn feature_b<D: Domain>(
    endpoint: &mut Endpoint<'_, REVIEW_B>,
    domain: &D,
) -> ShellResult<()> {
    loop {
        let candidate = imp::decode(endpoint.recv::<CandidateForReviewB>().await?)?;
        imp::candidate(&candidate, FEATURE_B)?;
        let arguments = endpoint.recv::<Review>().await?.into_value();
        imp::arguments(&arguments, ("pr_id", FEATURE_B))?;
        let result = domain
            .execute("REVIEWER", "code_review", arguments)
            .await
            .map_err(ShellError::message)?;
        imp::reviewed(&result, &candidate, FEATURE_B)?;
        let bytes = imp::encode(&result)?;
        endpoint.send::<ReviewResultB>(&bytes.as_slice()).await?;
        endpoint.send::<ReviewResultB>(&bytes.as_slice()).await?;
        let branch = endpoint.offer().await?;
        match branch.label() {
            ReviewApproved::LOGICAL_LABEL => {
                imp::same(
                    &imp::decode(branch.recv::<ReviewApproved>().await?)?,
                    &result,
                )?;
                break;
            }
            ReviewRejected::LOGICAL_LABEL => {
                imp::same(
                    &imp::decode(branch.recv::<ReviewRejected>().await?)?,
                    &result,
                )?;
            }
            _ => {
                return Err(ShellError::message(
                    "expected this review's original outcome",
                ));
            }
        }
    }
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            LastCiPassedB::LOGICAL_LABEL => {
                branch.recv::<LastCiPassedB>().await?;
                return Ok(());
            }
            CiPassedB::LOGICAL_LABEL => {
                branch.recv::<CiPassedB>().await?;
                return Ok(());
            }
            CiFailedB::LOGICAL_LABEL => {
                branch.recv::<CiFailedB>().await?;
            }
            _ => return Err(ShellError::message("expected feature B's CI outcome")),
        }
        loop {
            let candidate = imp::decode(endpoint.recv::<CandidateForReviewB>().await?)?;
            imp::candidate(&candidate, FEATURE_B)?;
            let admitted = endpoint.recv::<ReviewRequestB>().await?.into_value();
            imp::arguments(&admitted, ("pr_id", FEATURE_B))?;
            let arguments = endpoint.recv::<Review>().await?.into_value();
            imp::same(&arguments, &admitted)?;
            let result = domain
                .execute("REVIEWER", "code_review", arguments)
                .await
                .map_err(ShellError::message)?;
            imp::reviewed(&result, &candidate, FEATURE_B)?;
            let bytes = imp::encode(&result)?;
            endpoint.send::<ReviewResultB>(&bytes.as_slice()).await?;
            endpoint.send::<ReviewResultB>(&bytes.as_slice()).await?;
            let branch = endpoint.offer().await?;
            match branch.label() {
                RecoveryReviewApproved::LOGICAL_LABEL => {
                    imp::same(
                        &imp::decode(branch.recv::<RecoveryReviewApproved>().await?)?,
                        &result,
                    )?;
                    break;
                }
                RecoveryReviewRejected::LOGICAL_LABEL => {
                    imp::same(
                        &imp::decode(branch.recv::<RecoveryReviewRejected>().await?)?,
                        &result,
                    )?;
                }
                _ => {
                    return Err(ShellError::message(
                        "expected the recovery review's original outcome",
                    ));
                }
            }
        }
    }
}
