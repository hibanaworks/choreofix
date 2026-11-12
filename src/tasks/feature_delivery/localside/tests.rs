//! Candidate/test data belong to this service; the tester owns its environment.
use super::super::{
    global::{delivery::*, *},
    imp,
};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::{Endpoint, g::Message};

pub async fn feature_a<D: Domain>(
    endpoint: &mut Endpoint<'_, TEST_DATA_A>,
    domain: &D,
) -> ShellResult<()> {
    loop {
        let candidate = imp::decode(endpoint.recv::<CandidateForTestsA>().await?)?;
        imp::candidate(&candidate, FEATURE_A)?;
        let admitted = endpoint.recv::<TestIntentA>().await?.into_value();
        imp::arguments(&admitted, ("target", FEATURE_A))?;
        let bytes = imp::encode(&candidate)?;
        endpoint
            .send::<CandidateForTestsA>(&bytes.as_slice())
            .await?;
        let arguments = endpoint.recv::<TestRequestA>().await?.into_value();
        imp::same(&arguments, &admitted)?;
        let result = domain
            .execute("TEST_WRITER", "write_tests", arguments)
            .await
            .map_err(ShellError::message)?;
        imp::tested(&result, &candidate, FEATURE_A)?;
        let bytes = imp::encode(&result)?;
        endpoint.send::<TestResultA>(&bytes.as_slice()).await?;
        endpoint.send::<TestedA>(&bytes.as_slice()).await?;
        let branch = endpoint.offer().await?;
        match branch.label() {
            ReviewApproved::LOGICAL_LABEL => {
                branch.recv::<ReviewApproved>().await?;
                break;
            }
            ReviewRejected::LOGICAL_LABEL => {
                branch.recv::<ReviewRejected>().await?;
            }
            _ => return Err(ShellError::message("expected the reviewed test candidate")),
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
            _ => return Err(ShellError::message("expected this feature's CI outcome")),
        }
        loop {
            let candidate = imp::decode(endpoint.recv::<CandidateForTestsA>().await?)?;
            imp::candidate(&candidate, FEATURE_A)?;
            let admitted = endpoint.recv::<TestIntentA>().await?.into_value();
            imp::arguments(&admitted, ("target", FEATURE_A))?;
            let bytes = imp::encode(&candidate)?;
            endpoint
                .send::<CandidateForTestsA>(&bytes.as_slice())
                .await?;
            let arguments = endpoint.recv::<TestRequestA>().await?.into_value();
            imp::same(&arguments, &admitted)?;
            let result = domain
                .execute("TEST_WRITER", "write_tests", arguments)
                .await
                .map_err(ShellError::message)?;
            imp::tested(&result, &candidate, FEATURE_A)?;
            let bytes = imp::encode(&result)?;
            endpoint.send::<TestResultA>(&bytes.as_slice()).await?;
            endpoint.send::<TestedA>(&bytes.as_slice()).await?;
            let branch = endpoint.offer().await?;
            match branch.label() {
                RecoveryReviewApproved::LOGICAL_LABEL => {
                    branch.recv::<RecoveryReviewApproved>().await?;
                    break;
                }
                RecoveryReviewRejected::LOGICAL_LABEL => {
                    branch.recv::<RecoveryReviewRejected>().await?;
                }
                _ => {
                    return Err(ShellError::message(
                        "expected the recovery candidate's review",
                    ));
                }
            }
        }
    }
}

pub async fn feature_b<D: Domain>(
    endpoint: &mut Endpoint<'_, TEST_DATA_B>,
    domain: &D,
) -> ShellResult<()> {
    loop {
        let candidate = imp::decode(endpoint.recv::<CandidateForTestsB>().await?)?;
        imp::candidate(&candidate, FEATURE_B)?;
        let admitted = endpoint.recv::<TestIntentB>().await?.into_value();
        imp::arguments(&admitted, ("target", FEATURE_B))?;
        let bytes = imp::encode(&candidate)?;
        endpoint
            .send::<CandidateForTestsB>(&bytes.as_slice())
            .await?;
        let arguments = endpoint.recv::<TestRequestB>().await?.into_value();
        imp::same(&arguments, &admitted)?;
        let result = domain
            .execute("TEST_WRITER", "write_tests", arguments)
            .await
            .map_err(ShellError::message)?;
        imp::tested(&result, &candidate, FEATURE_B)?;
        let bytes = imp::encode(&result)?;
        endpoint.send::<TestResultB>(&bytes.as_slice()).await?;
        endpoint.send::<TestedB>(&bytes.as_slice()).await?;
        let branch = endpoint.offer().await?;
        match branch.label() {
            ReviewApproved::LOGICAL_LABEL => {
                branch.recv::<ReviewApproved>().await?;
                break;
            }
            ReviewRejected::LOGICAL_LABEL => {
                branch.recv::<ReviewRejected>().await?;
            }
            _ => return Err(ShellError::message("expected the reviewed test candidate")),
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
            _ => return Err(ShellError::message("expected this feature's CI outcome")),
        }
        loop {
            let candidate = imp::decode(endpoint.recv::<CandidateForTestsB>().await?)?;
            imp::candidate(&candidate, FEATURE_B)?;
            let admitted = endpoint.recv::<TestIntentB>().await?.into_value();
            imp::arguments(&admitted, ("target", FEATURE_B))?;
            let bytes = imp::encode(&candidate)?;
            endpoint
                .send::<CandidateForTestsB>(&bytes.as_slice())
                .await?;
            let arguments = endpoint.recv::<TestRequestB>().await?.into_value();
            imp::same(&arguments, &admitted)?;
            let result = domain
                .execute("TEST_WRITER", "write_tests", arguments)
                .await
                .map_err(ShellError::message)?;
            imp::tested(&result, &candidate, FEATURE_B)?;
            let bytes = imp::encode(&result)?;
            endpoint.send::<TestResultB>(&bytes.as_slice()).await?;
            endpoint.send::<TestedB>(&bytes.as_slice()).await?;
            let branch = endpoint.offer().await?;
            match branch.label() {
                RecoveryReviewApproved::LOGICAL_LABEL => {
                    branch.recv::<RecoveryReviewApproved>().await?;
                    break;
                }
                RecoveryReviewRejected::LOGICAL_LABEL => {
                    branch.recv::<RecoveryReviewRejected>().await?;
                }
                _ => {
                    return Err(ShellError::message(
                        "expected the recovery candidate's review",
                    ));
                }
            }
        }
    }
}
