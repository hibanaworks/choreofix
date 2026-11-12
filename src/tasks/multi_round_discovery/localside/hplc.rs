//! The chromatograph owns its measurements, review loan and final archives.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::{Endpoint, g::Message};
use serde_json::json;

#[hibana_runtime::scope]
pub async fn run<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, HPLC_SERVICE>,
    mut instrument: Endpoint<'r, HPLC>,
    domain: &D,
) -> ShellResult<()> {
    let qc = loop {
        let admitted = imp::decode(endpoint.recv::<Intent<0>>().await?)?;
        endpoint.send::<Instrument<HPLC>>(instrument).await?;
        let request = imp::decode(endpoint.recv::<Request<0>>().await?)?;
        imp::same(&request, &admitted)?;
        instrument = endpoint.recv::<Instrument<HPLC>>().await?;
        let mut material = endpoint.recv::<Specimen<QC>>().await?;
        let result = domain
            .execute("CHEMIST", "run_hplc_analysis", request)
            .await
            .map_err(ShellError::message)?;
        imp::reply(0, &result)?;
        imp::same(&result["data"]["inputs"], &json!({}))?;
        let bytes = imp::encode(&result)?;
        if result["success"] == true {
            material
                .send::<Passed<0>>(&imp::AnalysisReply::<0, true>::from_bytes(&bytes)?)
                .await?;
            endpoint.send::<Specimen<QC>>(material).await?;
            endpoint
                .send::<QcEvidence>(&imp::JsonTo::<MASS_SERVICE>::from_bytes(&bytes)?)
                .await?;
            break result;
        }
        material
            .send::<Failed<0>>(&imp::AnalysisReply::<0, false>::from_bytes(&bytes)?)
            .await?;
        endpoint.send::<Specimen<QC>>(material).await?;
    };
    let cells = imp::decode(endpoint.recv::<Cells<HPLC_SERVICE>>().await?)?;
    let cells_id = imp::reply(1, &cells)?;
    if cells["success"] != true {
        return Err(ShellError::message(
            "stability requires actual prepared cells",
        ));
    }
    let mut formulation = imp::decode(endpoint.recv::<Formulation<HPLC_SERVICE>>().await?)?;
    loop {
        async {
            let [mut stable, mut pure] = endpoint.par()?;
            async {
                let material = pure.recv::<Specimen<PURITY>>().await?;
                pure.send::<Specimen<PURITY>>(material).await
            };
            async {
                let material = stable.recv::<Specimen<STABILITY>>().await?;
                stable.send::<Specimen<STABILITY>>(material).await
            };
        }
        .await?;
        let formulation_id = imp::reply(4, &formulation)?;
        if formulation["success"] != true {
            return Err(ShellError::message(
                "analysis requires an actual formulation",
            ));
        }
        let purity = loop {
            let admitted = imp::decode(endpoint.recv::<Intent<7>>().await?)?;
            endpoint.send::<Instrument<HPLC>>(instrument).await?;
            let request = imp::decode(endpoint.recv::<Request<7>>().await?)?;
            imp::same(&request, &admitted)?;
            instrument = endpoint.recv::<Instrument<HPLC>>().await?;
            let mut material = endpoint.recv::<Specimen<PURITY>>().await?;
            let result = domain
                .execute("TOXICOLOGIST", "run_hplc_analysis", request)
                .await
                .map_err(ShellError::message)?;
            imp::reply(7, &result)?;
            imp::same(
                &result["data"]["inputs"],
                &json!({"formulation":formulation_id}),
            )?;
            let bytes = imp::encode(&result)?;
            if result["success"] == true {
                material
                    .send::<Passed<7>>(&imp::AnalysisReply::<7, true>::from_bytes(&bytes)?)
                    .await?;
                endpoint.send::<Specimen<PURITY>>(material).await?;
                endpoint
                    .send::<PurityEvidence<MASS_SERVICE>>(&imp::JsonTo::<MASS_SERVICE>::from_bytes(
                        &bytes,
                    )?)
                    .await?;
                break result;
            }
            material
                .send::<Failed<7>>(&imp::AnalysisReply::<7, false>::from_bytes(&bytes)?)
                .await?;
            endpoint.send::<Specimen<PURITY>>(material).await?;
        };
        let stability = loop {
            let admitted = imp::decode(endpoint.recv::<Intent<6>>().await?)?;
            endpoint.send::<Instrument<HPLC>>(instrument).await?;
            let request = imp::decode(endpoint.recv::<Request<6>>().await?)?;
            imp::same(&request, &admitted)?;
            instrument = endpoint.recv::<Instrument<HPLC>>().await?;
            let mut material = endpoint.recv::<Specimen<STABILITY>>().await?;
            let result = domain
                .execute("BIOLOGIST", "run_hplc_analysis", request)
                .await
                .map_err(ShellError::message)?;
            imp::reply(6, &result)?;
            imp::same(
                &result["data"]["inputs"],
                &json!({"formulation":formulation_id,"cells":cells_id}),
            )?;
            let bytes = imp::encode(&result)?;
            if result["success"] == true {
                material
                    .send::<Passed<6>>(&imp::AnalysisReply::<6, true>::from_bytes(&bytes)?)
                    .await?;
                endpoint.send::<Specimen<STABILITY>>(material).await?;
                endpoint
                    .send::<StabilityEvidence<MASS_SERVICE>>(
                        &imp::JsonTo::<MASS_SERVICE>::from_bytes(&bytes)?,
                    )
                    .await?;
                break result;
            }
            material
                .send::<Failed<6>>(&imp::AnalysisReply::<6, false>::from_bytes(&bytes)?)
                .await?;
            endpoint.send::<Specimen<STABILITY>>(material).await?;
        };
        formulation = loop {
            let branch = endpoint.offer().await?;
            match branch.label() {
                RoutineReview::LOGICAL_LABEL => {
                    imp::decode(branch.recv::<RoutineReview>().await?)?;
                }
                DeepIntent::<HPLC>::LOGICAL_LABEL => {
                    let admitted = imp::decode(branch.recv::<DeepIntent<HPLC>>().await?)?;
                    imp::arguments(11, &admitted)?;
                    endpoint.send::<Instrument<HPLC>>(instrument).await?;
                    instrument = endpoint.recv::<Instrument<HPLC>>().await?;
                }
                Formulation::<HPLC_SERVICE>::LOGICAL_LABEL => {
                    break imp::decode(branch.recv::<Formulation<HPLC_SERVICE>>().await?)?;
                }
                Archived::<FORMULATION>::LOGICAL_LABEL => {
                    drop(branch);
                    let (formulation_cap, bio_cap, stability_cap, purity_cap, tox_cap) = async {
                        let [mut f, mut b, mut s, mut p, mut t] = endpoint.par()?;
                        async { f.recv::<Archived<FORMULATION>>().await };
                        async { b.recv::<Archived<BIOASSAY>>().await };
                        async { s.recv::<Archived<STABILITY>>().await };
                        async { p.recv::<Archived<PURITY>>().await };
                        async { t.recv::<Archived<TOXICITY>>().await };
                    }
                    .await?;
                    let final_result = imp::decode(endpoint.recv::<Final<HPLC_SERVICE>>().await?)?;
                    imp::same(&final_result["formulation"], &formulation)?;
                    for (name, expected) in [("purity", &purity), ("stability", &stability)] {
                        imp::same(
                            &final_result["clinical"]["data"]["inputs"][name],
                            &expected["data"]["event_id"],
                        )?;
                    }
                    imp::same(
                        &final_result["compound"]["data"]["inputs"]["qc"],
                        &qc["data"]["event_id"],
                    )?;
                    let decision =
                        imp::decode(endpoint.recv::<DecisionEvidence<HPLC_SERVICE>>().await?)?;
                    imp::same(&decision, &final_result["decision"])?;
                    loop {
                        let admitted = imp::decode(endpoint.recv::<Intent<15>>().await?)?;
                        endpoint.send::<Instrument<HPLC>>(instrument).await?;
                        let request = imp::decode(endpoint.recv::<Request<15>>().await?)?;
                        imp::same(&request, &admitted)?;
                        instrument = endpoint.recv::<Instrument<HPLC>>().await?;
                        let mut study = endpoint.recv::<Specimen<STUDY>>().await?;
                        let result = domain
                            .execute("CHEMIST", "run_hplc_analysis", request)
                            .await
                            .map_err(ShellError::message)?;
                        imp::reply(15, &result)?;
                        let expected = json!({"compound":imp::reply(3,&final_result["compound"])?,
                            "formulation":formulation_id,"clinical":imp::reply(9,&final_result["clinical"])?,
                            "regulatory":imp::reply(if final_result["regulatory"]["data"]["review_type"]=="initial_review" {10}else{11},&final_result["regulatory"])?,
                            "decision":imp::reply(if final_result["outcome"]=="continue" {12}else{14},&decision)?});
                        imp::same(&result["data"]["inputs"], &expected)?;
                        let bytes = imp::encode(&result)?;
                        if result["success"] == true {
                            imp::documentation_receipt(&final_result, &result)?;
                            study
                                .send::<Passed<15>>(&imp::AnalysisReply::<15, true>::from_bytes(
                                    &bytes,
                                )?)
                                .await?;
                            endpoint.send::<Specimen<STUDY>>(study).await?;
                            // Archive custody extends through the actual effect.
                            drop((formulation_cap, bio_cap, stability_cap, purity_cap, tox_cap));
                            return Ok(());
                        }
                        study
                            .send::<Failed<15>>(&imp::AnalysisReply::<15, false>::from_bytes(
                                &bytes,
                            )?)
                            .await?;
                        endpoint.send::<Specimen<STUDY>>(study).await?;
                    }
                }
                _ => return Err(ShellError::message("unexpected chromatograph operation")),
            }
        };
    }
}
