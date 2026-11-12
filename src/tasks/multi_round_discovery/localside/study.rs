//! The study owner keeps actual scientific evidence and owns deep-review tools.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::{Endpoint, g::Message};
use serde_json::json;

#[hibana_runtime::scope]
pub async fn run<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, STUDY_SERVICE>,
    domain: &D,
) -> ShellResult<()> {
    let identity = imp::decode(endpoint.recv::<Identity<STUDY_SERVICE>>().await?)?;
    imp::reply(3, &identity)?;
    let mut formulation = imp::decode(endpoint.recv::<Formulation<STUDY_SERVICE>>().await?)?;
    loop {
        imp::formulation_receipt(&identity, &formulation)?;
        if formulation["success"] != true {
            return Err(ShellError::message(
                "study requires an actual successful formulation",
            ));
        }
        let clinical = imp::decode(endpoint.recv::<ClinicalEvidence<STUDY_SERVICE>>().await?)?;
        imp::reply(9, &clinical)?;
        if clinical["data"]["inputs"]["formulation"] != imp::reply(4, &formulation)?
            || !matches!(
                clinical["data"]["assessment"].as_str(),
                Some("supported" | "insufficient_data")
            )
        {
            return Err(ShellError::message(
                "clinical assessment belongs to another formulation",
            ));
        }

        // This owns the last actual domain result, not an approval/progress flag.
        // The next received Endpoint operation chooses review or decision.
        let mut actual_review = None;
        let (mut study, regulatory) = loop {
            let branch = endpoint.offer().await?;
            match branch.label() {
                RoutineReview::LOGICAL_LABEL => {
                    let admitted = imp::decode(branch.recv::<RoutineReview>().await?)?;
                    let mut study = endpoint.recv::<Specimen<STUDY>>().await?;
                    let result = domain
                        .execute(
                            "REGULATORY_SPECIALIST",
                            "verify_regulatory_compliance",
                            admitted.clone(),
                        )
                        .await
                        .map_err(ShellError::message)?;
                    imp::regulatory_receipt(&admitted, &clinical, &result)?;
                    let bytes = imp::encode(&result)?;
                    if result["success"] == true {
                        study
                            .send::<Passed<10>>(&imp::AnalysisReply::<10, true>::from_bytes(
                                &bytes,
                            )?)
                            .await?;
                    } else {
                        study
                            .send::<Failed<10>>(&imp::AnalysisReply::<10, false>::from_bytes(
                                &bytes,
                            )?)
                            .await?;
                    }
                    endpoint.send::<Specimen<STUDY>>(study).await?;
                    actual_review = Some(result);
                }
                DeepReview::LOGICAL_LABEL => {
                    let admitted = imp::decode(branch.recv::<DeepReview>().await?)?;
                    let (hplc_request, mass_request) = async {
                        let [mut hplc, mut mass] = endpoint.par()?;
                        async { imp::decode(hplc.recv::<Request<11>>().await?) };
                        async { imp::decode(mass.recv::<Request<11>>().await?) };
                    }
                    .await?;
                    imp::same(&hplc_request, &admitted)?;
                    imp::same(&mass_request, &admitted)?;
                    let (hplc, mass) = async {
                        let [mut hplc, mut mass] = endpoint.par()?;
                        async { hplc.recv::<Instrument<HPLC>>().await };
                        async { mass.recv::<Instrument<MASS>>().await };
                    }
                    .await?;
                    let mut study = endpoint.recv::<Specimen<STUDY>>().await?;
                    let result = domain
                        .execute(
                            "REGULATORY_SPECIALIST",
                            "verify_regulatory_compliance",
                            admitted.clone(),
                        )
                        .await
                        .map_err(ShellError::message)?;
                    imp::regulatory_receipt(&admitted, &clinical, &result)?;
                    let bytes = imp::encode(&result)?;
                    if result["success"] == true {
                        study
                            .send::<Passed<11>>(&imp::AnalysisReply::<11, true>::from_bytes(
                                &bytes,
                            )?)
                            .await?;
                    } else {
                        study
                            .send::<Failed<11>>(&imp::AnalysisReply::<11, false>::from_bytes(
                                &bytes,
                            )?)
                            .await?;
                    }
                    endpoint.send::<Specimen<STUDY>>(study).await?;
                    async {
                        let [mut hplc_return, mut mass_return] = endpoint.par()?;
                        async { hplc_return.send::<Instrument<HPLC>>(hplc).await };
                        async { mass_return.send::<Instrument<MASS>>(mass).await };
                    }
                    .await?;
                    actual_review = Some(result);
                }
                Specimen::<STUDY>::LOGICAL_LABEL => {
                    let study = branch.recv::<Specimen<STUDY>>().await?;
                    let regulatory = actual_review.take().ok_or_else(|| {
                        ShellError::message("decision has no actual regulatory result")
                    })?;
                    break (study, regulatory);
                }
                _ => return Err(ShellError::message("unexpected study operation")),
            }
        };
        let bytes = imp::encode(&regulatory)?;
        study
            .send::<RegulatoryEvidence<DIRECTOR>>(&imp::JsonTo::<DIRECTOR>::from_bytes(&bytes)?)
            .await?;
        let input = imp::decode(endpoint.recv::<DecisionRequest>().await?)?;
        let step = match input["outcome"].as_str() {
            Some("continue") => 12,
            Some("retry") => 13,
            Some("abort") => 14,
            _ => return Err(ShellError::message("unknown requested project outcome")),
        };
        imp::arguments(step, &input)?;
        if step == 12 && regulatory["success"] != true {
            return Err(ShellError::message(
                "continuation requires actual positive approval",
            ));
        }
        let mut candidate = json!({"outcome":input["outcome"],"compound":identity,
            "formulation":formulation,"clinical":clinical,"regulatory":regulatory});
        let result = domain
            .execute("PROJECT_DIRECTOR", "submit_project_decision", input)
            .await
            .map_err(ShellError::message)?;
        imp::decision_receipt(&candidate, &result)?;
        // The director receives the actual complete outcome, not enough IDs
        // to manufacture a second copy of the scientific record elsewhere.
        candidate["decision"] = result;
        let bytes = imp::encode(&candidate)?;
        study
            .send::<DecisionEvidence<DIRECTOR>>(&imp::JsonTo::<DIRECTOR>::from_bytes(&bytes)?)
            .await?;
        endpoint.send::<Specimen<STUDY>>(study).await?;
        let branch = endpoint.offer().await?;
        if branch.label() == Final::<STUDY_SERVICE>::LOGICAL_LABEL {
            let final_result = imp::decode(branch.recv::<Final<STUDY_SERVICE>>().await?)?;
            imp::same(&final_result, &candidate)?;
            let bytes = imp::encode(&candidate["decision"])?;
            endpoint
                .send::<DecisionEvidence<HPLC_SERVICE>>(&imp::JsonTo::<HPLC_SERVICE>::from_bytes(
                    &bytes,
                )?)
                .await?;
            return Ok(());
        }
        formulation = imp::decode(branch.recv::<Formulation<STUDY_SERVICE>>().await?)?;
    }
}
