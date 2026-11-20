use super::super::{global::*, imp};
use crate::shell::{
    ShellError, ShellResult,
    imp::{review_reply::ReviewReply, service::Domain},
};
use hibana::Endpoint;
use serde_json::json;

#[hibana_runtime::scope]
pub async fn run(
    endpoint: &mut Endpoint<'_, REVIEW_OWNER>,
    domain: &impl Domain,
) -> ShellResult<()> {
    loop {
        let arguments = endpoint.recv::<Request<COMPLIANCE>>().await?.into_value();
        let submitted = imp::decode(endpoint.recv::<ReviewInput>().await?)?;
        let decision = imp::artifact(&submitted)?;
        let compound = submitted["data"]["inputs"]["compound"]
            .as_u64()
            .ok_or_else(|| ShellError::message("candidate decision lacks compound"))?;
        if submitted["tool_name"] != "submit_decision"
            || submitted["data"]["scientist"] != "LEAD_SCIENTIST"
            || !submitted["data"]["go"].is_boolean()
        {
            return Err(ShellError::message(
                "compliance request names an invalid candidate decision",
            ));
        }
        let mut study = endpoint.recv::<Loan<STUDY>>().await?;
        let receipt = domain
            .execute(
                "REGULATORY_SPECIALIST",
                "verify_regulatory_compliance",
                arguments.clone(),
            )
            .await
            .map_err(ShellError::message)?;
        imp::result(
            &receipt,
            (
                "REGULATORY_SPECIALIST",
                "verify_regulatory_compliance",
                &arguments,
            ),
        )?;
        imp::inputs(&receipt, &json!({"compound":compound,"decision":decision}))?;
        let bytes = imp::encode(&receipt)?;
        if receipt["success"] == true {
            let reply = ReviewReply::<true>::from_bytes(&bytes)?;
            study.send::<Passed>(&reply).await?;
            study.send::<ApprovedForLead>(&reply).await?;
            async {
                let [mut bio, mut tox, mut hplc, mut cells] = study.par()?;
                async {
                    bio.send::<ApprovedForBio>(&reply).await?;
                };
                async {
                    tox.send::<ApprovedForTox>(&reply).await?;
                };
                async {
                    hplc.send::<ApprovedForHplc>(&reply).await?;
                };
                async {
                    cells.send::<ApprovedForCells>(&reply).await?;
                };
                Ok(())
            }
            .await?;
            study.send::<ApprovedForChemist>(&reply).await?;
            endpoint.send::<Loan<STUDY>>(study).await?;
            return Ok(());
        }
        let reply = ReviewReply::<false>::from_bytes(&bytes)?;
        study.send::<ComplianceRejected>(&reply).await?;
        study.send::<ComplianceRejected>(&reply).await?;
        endpoint.send::<Loan<STUDY>>(study).await?;
    }
}
