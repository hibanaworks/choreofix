//! The reviewer owns the submitted study while the real regulatory effect runs.
use super::super::{global::*, imp};
use crate::shell::{
    ShellError, ShellResult,
    imp::{review_reply::ReviewReply, service::Domain},
};
use hibana::Endpoint;
use serde_json::{Value, json};
use std::cell::Cell;

pub async fn run<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, REVIEW_SERVICE>,
    actual: &Cell<Value>,
    domain: &D,
) -> ShellResult<()> {
    loop {
        let admitted = imp::decode(endpoint.recv::<Review>().await?)?;
        imp::same(&admitted["arguments"], &json!({"decision":"go_no_go"}))?;
        imp::decision_receipt(&admitted["decision"], &admitted["receipt"])?;
        let mut study = endpoint.recv::<Specimen<STUDY>>().await?;
        let result = domain
            .execute(
                "REGULATORY_SPECIALIST",
                "verify_regulatory_compliance",
                admitted["arguments"].clone(),
            )
            .await
            .map_err(ShellError::message)?;
        imp::regulatory_receipt(&admitted, &result)?;
        let bytes = imp::encode(&result)?;
        actual.set(result.clone());
        if result["success"] == true {
            let reply = ReviewReply::<true>::from_bytes(&bytes)?;
            study.send::<Approved>(&reply).await?;
            study.send::<ApprovedForLead>(&reply).await?;
            study.send::<ApprovedForBio>(&reply).await?;
            study.send::<ApprovedForTox>(&reply).await?;
            study.send::<ApprovedForHplc>(&reply).await?;
            study.send::<ApprovedForCells>(&reply).await?;
            let approved = imp::encode(&json!({"success":result["success"], "approval":result,
                "decision":admitted["decision"],"receipt":admitted["receipt"]}))?;
            study
                .send::<ApprovedForChemist>(&ReviewReply::<true>::from_bytes(&approved)?)
                .await?;
            endpoint.send::<Specimen<STUDY>>(study).await?;
            break;
        }
        let reply = ReviewReply::<false>::from_bytes(&bytes)?;
        study.send::<Rejected>(&reply).await?;
        study.send::<Rejected>(&reply).await?;
        endpoint.send::<Specimen<STUDY>>(study).await?;
    }
    Ok(())
}
