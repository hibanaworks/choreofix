//! The HPLC owner receives both the real request and its scientific continuation.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::{Endpoint, g::Message};
use serde_json::{Value, json};
use std::cell::Cell;

pub async fn run<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, HPLC_SERVICE>,
    (mut instrument, actual): (Endpoint<'r, HPLC>, &Cell<Value>),
    domain: &D,
) -> ShellResult<()> {
    let qc = loop {
        let admitted = imp::decode(endpoint.recv::<Intent<0>>().await?)?;
        endpoint.send::<Instrument<HPLC>>(instrument).await?;
        let request = imp::decode(endpoint.recv::<Request<0>>().await?)?;
        imp::same(&request, &admitted)?;
        instrument = endpoint.recv::<Instrument<HPLC>>().await?;
        let mut specimen = endpoint.recv::<Specimen<QC>>().await?;
        let (actor, tool, _, _) = imp::analysis(0)?;
        let result = domain
            .execute(actor, tool, request)
            .await
            .map_err(ShellError::message)?;
        let bytes = imp::encode(&result)?;
        actual.set(result.clone());
        if result["success"] == true {
            specimen
                .send::<Passed<0>>(&imp::AnalysisReply::<0, true>::from_bytes(&bytes)?)
                .await?;
            endpoint.send::<Specimen<QC>>(specimen).await?;
            break result;
        }
        specimen
            .send::<Failed<0>>(&imp::AnalysisReply::<0, false>::from_bytes(&bytes)?)
            .await?;
        endpoint.send::<Specimen<QC>>(specimen).await?;
    };
    let mut purity = loop {
        let admitted = imp::decode(endpoint.recv::<Intent<6>>().await?)?;
        endpoint.send::<Instrument<HPLC>>(instrument).await?;
        let request = imp::decode(endpoint.recv::<Request<6>>().await?)?;
        imp::same(&request, &admitted)?;
        instrument = endpoint.recv::<Instrument<HPLC>>().await?;
        let mut specimen = endpoint.recv::<Specimen<PURITY>>().await?;
        let (actor, tool, _, _) = imp::analysis(6)?;
        let result = domain
            .execute(actor, tool, request)
            .await
            .map_err(ShellError::message)?;
        let bytes = imp::encode(&result)?;
        actual.set(result.clone());
        if result["success"] == true {
            specimen
                .send::<Passed<6>>(&imp::AnalysisReply::<6, true>::from_bytes(&bytes)?)
                .await?;
            endpoint.send::<Specimen<PURITY>>(specimen).await?;
            break result;
        }
        specimen
            .send::<Failed<6>>(&imp::AnalysisReply::<6, false>::from_bytes(&bytes)?)
            .await?;
        endpoint.send::<Specimen<PURITY>>(specimen).await?;
    };
    let mut stability = loop {
        let admitted = imp::decode(endpoint.recv::<Intent<5>>().await?)?;
        endpoint.send::<Instrument<HPLC>>(instrument).await?;
        let request = imp::decode(endpoint.recv::<Request<5>>().await?)?;
        imp::same(&request, &admitted)?;
        instrument = endpoint.recv::<Instrument<HPLC>>().await?;
        let mut specimen = endpoint.recv::<Specimen<STABILITY>>().await?;
        let (actor, tool, _, _) = imp::analysis(5)?;
        let result = domain
            .execute(actor, tool, request)
            .await
            .map_err(ShellError::message)?;
        let bytes = imp::encode(&result)?;
        actual.set(result.clone());
        if result["success"] == true {
            specimen
                .send::<Passed<5>>(&imp::AnalysisReply::<5, true>::from_bytes(&bytes)?)
                .await?;
            endpoint.send::<Specimen<STABILITY>>(specimen).await?;
            break result;
        }
        specimen
            .send::<Failed<5>>(&imp::AnalysisReply::<5, false>::from_bytes(&bytes)?)
            .await?;
        endpoint.send::<Specimen<STABILITY>>(specimen).await?;
    };
    let structural = loop {
        let admitted = imp::decode(endpoint.recv::<Intent<9>>().await?)?;
        endpoint.send::<Instrument<HPLC>>(instrument).await?;
        let request = imp::decode(endpoint.recv::<Request<9>>().await?)?;
        imp::same(&request, &admitted)?;
        instrument = endpoint.recv::<Instrument<HPLC>>().await?;
        let mut specimen = endpoint.recv::<Specimen<STRUCTURAL>>().await?;
        let (actor, tool, _, _) = imp::analysis(9)?;
        let result = domain
            .execute(actor, tool, request)
            .await
            .map_err(ShellError::message)?;
        let bytes = imp::encode(&result)?;
        actual.set(result.clone());
        if result["success"] == true {
            specimen
                .send::<Passed<9>>(&imp::AnalysisReply::<9, true>::from_bytes(&bytes)?)
                .await?;
            endpoint.send::<Specimen<STRUCTURAL>>(specimen).await?;
            break result;
        }
        specimen
            .send::<Failed<9>>(&imp::AnalysisReply::<9, false>::from_bytes(&bytes)?)
            .await?;
        endpoint.send::<Specimen<STRUCTURAL>>(specimen).await?;
    };
    let (approval, decision, recorded) = loop {
        let decision = imp::decode(endpoint.recv::<Decision>().await?)?;
        imp::same(&decision["compound"]["qc"], &qc)?;
        imp::same(&decision["bio"]["stability"], &stability)?;
        imp::same(&decision["tox"]["purity"], &purity)?;
        imp::same(&decision["analyses"]["structural"], &structural)?;
        let recorded = domain
            .execute(
                "LEAD_SCIENTIST",
                "submit_decision",
                json!({"go":decision["go"]}),
            )
            .await
            .map_err(ShellError::message)?;
        imp::decision_receipt(&decision, &recorded)?;
        let bytes = imp::encode(&recorded)?;
        endpoint.send::<DecisionRecorded>(&bytes.as_slice()).await?;
        let branch = endpoint.offer().await?;
        match branch.label() {
            label if label == ApprovedForHplc::LOGICAL_LABEL => {
                let approved = imp::decode(branch.recv::<ApprovedForHplc>().await?)?;
                imp::same(
                    &approved["data"]["inputs"]["decision"],
                    &recorded["data"]["artifact_id"],
                )?;
                break (approved, decision, recorded);
            }
            0 => {
                branch.recv::<Plan<0, HPLC_SERVICE>>().await?;
            }
            1 => {
                branch.recv::<Plan<1, HPLC_SERVICE>>().await?;
                stability = loop {
                    let admitted = imp::decode(endpoint.recv::<Intent<12>>().await?)?;
                    endpoint.send::<Instrument<HPLC>>(instrument).await?;
                    let request = imp::decode(endpoint.recv::<Request<12>>().await?)?;
                    imp::same(&request, &admitted)?;
                    instrument = endpoint.recv::<Instrument<HPLC>>().await?;
                    let mut specimen = endpoint.recv::<Specimen<STABILITY>>().await?;
                    let (actor, tool, _, _) = imp::analysis(12)?;
                    let result = domain
                        .execute(actor, tool, request)
                        .await
                        .map_err(ShellError::message)?;
                    let bytes = imp::encode(&result)?;
                    actual.set(result.clone());
                    if result["success"] == true {
                        specimen
                            .send::<Passed<12>>(&imp::AnalysisReply::<12, true>::from_bytes(
                                &bytes,
                            )?)
                            .await?;
                        endpoint.send::<Specimen<STABILITY>>(specimen).await?;
                        break result;
                    }
                    specimen
                        .send::<Failed<12>>(&imp::AnalysisReply::<12, false>::from_bytes(&bytes)?)
                        .await?;
                    endpoint.send::<Specimen<STABILITY>>(specimen).await?;
                };
            }
            2 => {
                branch.recv::<Plan<2, HPLC_SERVICE>>().await?;
                purity = loop {
                    let admitted = imp::decode(endpoint.recv::<Intent<13>>().await?)?;
                    endpoint.send::<Instrument<HPLC>>(instrument).await?;
                    let request = imp::decode(endpoint.recv::<Request<13>>().await?)?;
                    imp::same(&request, &admitted)?;
                    instrument = endpoint.recv::<Instrument<HPLC>>().await?;
                    let mut specimen = endpoint.recv::<Specimen<PURITY>>().await?;
                    let (actor, tool, _, _) = imp::analysis(13)?;
                    let result = domain
                        .execute(actor, tool, request)
                        .await
                        .map_err(ShellError::message)?;
                    let bytes = imp::encode(&result)?;
                    actual.set(result.clone());
                    if result["success"] == true {
                        specimen
                            .send::<Passed<13>>(&imp::AnalysisReply::<13, true>::from_bytes(
                                &bytes,
                            )?)
                            .await?;
                        endpoint.send::<Specimen<PURITY>>(specimen).await?;
                        break result;
                    }
                    specimen
                        .send::<Failed<13>>(&imp::AnalysisReply::<13, false>::from_bytes(&bytes)?)
                        .await?;
                    endpoint.send::<Specimen<PURITY>>(specimen).await?;
                };
            }
            3 => {
                branch.recv::<Plan<3, HPLC_SERVICE>>().await?;
                purity = loop {
                    let admitted = imp::decode(endpoint.recv::<Intent<13>>().await?)?;
                    endpoint.send::<Instrument<HPLC>>(instrument).await?;
                    let request = imp::decode(endpoint.recv::<Request<13>>().await?)?;
                    imp::same(&request, &admitted)?;
                    instrument = endpoint.recv::<Instrument<HPLC>>().await?;
                    let mut specimen = endpoint.recv::<Specimen<PURITY>>().await?;
                    let (actor, tool, _, _) = imp::analysis(13)?;
                    let result = domain
                        .execute(actor, tool, request)
                        .await
                        .map_err(ShellError::message)?;
                    let bytes = imp::encode(&result)?;
                    actual.set(result.clone());
                    if result["success"] == true {
                        specimen
                            .send::<Passed<13>>(&imp::AnalysisReply::<13, true>::from_bytes(
                                &bytes,
                            )?)
                            .await?;
                        endpoint.send::<Specimen<PURITY>>(specimen).await?;
                        break result;
                    }
                    specimen
                        .send::<Failed<13>>(&imp::AnalysisReply::<13, false>::from_bytes(&bytes)?)
                        .await?;
                    endpoint.send::<Specimen<PURITY>>(specimen).await?;
                };
                stability = loop {
                    let admitted = imp::decode(endpoint.recv::<Intent<12>>().await?)?;
                    endpoint.send::<Instrument<HPLC>>(instrument).await?;
                    let request = imp::decode(endpoint.recv::<Request<12>>().await?)?;
                    imp::same(&request, &admitted)?;
                    instrument = endpoint.recv::<Instrument<HPLC>>().await?;
                    let mut specimen = endpoint.recv::<Specimen<STABILITY>>().await?;
                    let (actor, tool, _, _) = imp::analysis(12)?;
                    let result = domain
                        .execute(actor, tool, request)
                        .await
                        .map_err(ShellError::message)?;
                    let bytes = imp::encode(&result)?;
                    actual.set(result.clone());
                    if result["success"] == true {
                        specimen
                            .send::<Passed<12>>(&imp::AnalysisReply::<12, true>::from_bytes(
                                &bytes,
                            )?)
                            .await?;
                        endpoint.send::<Specimen<STABILITY>>(specimen).await?;
                        break result;
                    }
                    specimen
                        .send::<Failed<12>>(&imp::AnalysisReply::<12, false>::from_bytes(&bytes)?)
                        .await?;
                    endpoint.send::<Specimen<STABILITY>>(specimen).await?;
                };
            }
            _ => {
                return Err(ShellError::message(
                    "unexpected scientific continuation at the HPLC owner",
                ));
            }
        }
    };
    loop {
        let admitted = imp::decode(endpoint.recv::<Document>().await?)?;
        imp::same(&admitted["arguments"], &json!({"sample":"documentation"}))?;
        imp::same(&admitted["approval"], &approval)?;
        imp::same(&admitted["decision"], &decision)?;
        imp::same(&admitted["receipt"], &recorded)?;
        endpoint.send::<Instrument<HPLC>>(instrument).await?;
        let request = imp::decode(endpoint.recv::<Document>().await?)?;
        imp::same(&request, &admitted)?;
        instrument = endpoint.recv::<Instrument<HPLC>>().await?;
        let mut study = endpoint.recv::<Specimen<STUDY>>().await?;
        let result = domain
            .execute("CHEMIST", "run_hplc_analysis", request["arguments"].clone())
            .await
            .map_err(ShellError::message)?;
        imp::same(
            &result["data"]["inputs"]["regulatory"],
            &approval["data"]["artifact_id"],
        )?;
        imp::same(
            &result["data"]["inputs"]["decision"],
            &recorded["data"]["artifact_id"],
        )?;
        let bytes = imp::encode(&result)?;
        actual.set(result.clone());
        if result["success"] == true {
            study
                .send::<Passed<15>>(&imp::AnalysisReply::<15, true>::from_bytes(&bytes)?)
                .await?;
            endpoint.send::<Specimen<STUDY>>(study).await?;
            break;
        }
        study
            .send::<Failed<15>>(&imp::AnalysisReply::<15, false>::from_bytes(&bytes)?)
            .await?;
        endpoint.send::<Specimen<STUDY>>(study).await?;
    }
    drop(instrument);
    Ok(())
}
