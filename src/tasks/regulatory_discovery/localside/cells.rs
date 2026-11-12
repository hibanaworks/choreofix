//! The cell laboratory owns its instrument and returns actual trial outcomes.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::{Endpoint, g::Message};
use serde_json::Value;
use std::cell::Cell;

pub async fn run<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, CELL_SERVICE>,
    (mut instrument, actual): (Endpoint<'r, CELL>, &Cell<Value>),
    domain: &D,
) -> ShellResult<()> {
    loop {
        let admitted = imp::decode(endpoint.recv::<Intent<1>>().await?)?;
        endpoint.send::<Instrument<CELL>>(instrument).await?;
        let request = imp::decode(endpoint.recv::<Request<1>>().await?)?;
        imp::same(&request, &admitted)?;
        instrument = endpoint.recv::<Instrument<CELL>>().await?;
        let mut specimen = endpoint.recv::<Specimen<CULTURE>>().await?;
        let (actor, tool, _, _) = imp::analysis(1)?;
        let result = domain
            .execute(actor, tool, request)
            .await
            .map_err(ShellError::message)?;
        let bytes = imp::encode(&result)?;
        actual.set(result.clone());
        if result["success"] == true {
            specimen
                .send::<Passed<1>>(&imp::AnalysisReply::<1, true>::from_bytes(&bytes)?)
                .await?;
            endpoint.send::<Specimen<CULTURE>>(specimen).await?;
            break;
        }
        specimen
            .send::<Failed<1>>(&imp::AnalysisReply::<1, false>::from_bytes(&bytes)?)
            .await?;
        endpoint.send::<Specimen<CULTURE>>(specimen).await?;
    }
    loop {
        let admitted = imp::decode(endpoint.recv::<Intent<4>>().await?)?;
        endpoint.send::<Instrument<CELL>>(instrument).await?;
        let request = imp::decode(endpoint.recv::<Request<4>>().await?)?;
        imp::same(&request, &admitted)?;
        instrument = endpoint.recv::<Instrument<CELL>>().await?;
        let mut specimen = endpoint.recv::<Specimen<BIOASSAY>>().await?;
        let (actor, tool, _, _) = imp::analysis(4)?;
        let result = domain
            .execute(actor, tool, request)
            .await
            .map_err(ShellError::message)?;
        let bytes = imp::encode(&result)?;
        actual.set(result.clone());
        if result["success"] == true {
            specimen
                .send::<Passed<4>>(&imp::AnalysisReply::<4, true>::from_bytes(&bytes)?)
                .await?;
            endpoint.send::<Specimen<BIOASSAY>>(specimen).await?;
            break;
        }
        specimen
            .send::<Failed<4>>(&imp::AnalysisReply::<4, false>::from_bytes(&bytes)?)
            .await?;
        endpoint.send::<Specimen<BIOASSAY>>(specimen).await?;
    }
    loop {
        let admitted = imp::decode(endpoint.recv::<Intent<7>>().await?)?;
        endpoint.send::<Instrument<CELL>>(instrument).await?;
        let request = imp::decode(endpoint.recv::<Request<7>>().await?)?;
        imp::same(&request, &admitted)?;
        instrument = endpoint.recv::<Instrument<CELL>>().await?;
        let mut specimen = endpoint.recv::<Specimen<TOXICITY>>().await?;
        let (actor, tool, _, _) = imp::analysis(7)?;
        let result = domain
            .execute(actor, tool, request)
            .await
            .map_err(ShellError::message)?;
        let bytes = imp::encode(&result)?;
        actual.set(result.clone());
        if result["success"] == true {
            specimen
                .send::<Passed<7>>(&imp::AnalysisReply::<7, true>::from_bytes(&bytes)?)
                .await?;
            endpoint.send::<Specimen<TOXICITY>>(specimen).await?;
            break;
        }
        specimen
            .send::<Failed<7>>(&imp::AnalysisReply::<7, false>::from_bytes(&bytes)?)
            .await?;
        endpoint.send::<Specimen<TOXICITY>>(specimen).await?;
    }
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            label if label == ApprovedForCells::LOGICAL_LABEL => {
                branch.recv::<ApprovedForCells>().await?;
                break;
            }
            0 => {
                branch.recv::<Plan<0, CELL_SERVICE>>().await?;
            }
            1 => {
                branch.recv::<Plan<1, CELL_SERVICE>>().await?;
                loop {
                    let admitted = imp::decode(endpoint.recv::<Intent<11>>().await?)?;
                    endpoint.send::<Instrument<CELL>>(instrument).await?;
                    let request = imp::decode(endpoint.recv::<Request<11>>().await?)?;
                    imp::same(&request, &admitted)?;
                    instrument = endpoint.recv::<Instrument<CELL>>().await?;
                    let mut specimen = endpoint.recv::<Specimen<BIOASSAY>>().await?;
                    let (actor, tool, _, _) = imp::analysis(11)?;
                    let result = domain
                        .execute(actor, tool, request)
                        .await
                        .map_err(ShellError::message)?;
                    let bytes = imp::encode(&result)?;
                    actual.set(result.clone());
                    if result["success"] == true {
                        specimen
                            .send::<Passed<11>>(&imp::AnalysisReply::<11, true>::from_bytes(
                                &bytes,
                            )?)
                            .await?;
                        endpoint.send::<Specimen<BIOASSAY>>(specimen).await?;
                        break;
                    }
                    specimen
                        .send::<Failed<11>>(&imp::AnalysisReply::<11, false>::from_bytes(&bytes)?)
                        .await?;
                    endpoint.send::<Specimen<BIOASSAY>>(specimen).await?;
                }
            }
            2 => {
                branch.recv::<Plan<2, CELL_SERVICE>>().await?;
                loop {
                    let admitted = imp::decode(endpoint.recv::<Intent<14>>().await?)?;
                    endpoint.send::<Instrument<CELL>>(instrument).await?;
                    let request = imp::decode(endpoint.recv::<Request<14>>().await?)?;
                    imp::same(&request, &admitted)?;
                    instrument = endpoint.recv::<Instrument<CELL>>().await?;
                    let mut specimen = endpoint.recv::<Specimen<TOXICITY>>().await?;
                    let (actor, tool, _, _) = imp::analysis(14)?;
                    let result = domain
                        .execute(actor, tool, request)
                        .await
                        .map_err(ShellError::message)?;
                    let bytes = imp::encode(&result)?;
                    actual.set(result.clone());
                    if result["success"] == true {
                        specimen
                            .send::<Passed<14>>(&imp::AnalysisReply::<14, true>::from_bytes(
                                &bytes,
                            )?)
                            .await?;
                        endpoint.send::<Specimen<TOXICITY>>(specimen).await?;
                        break;
                    }
                    specimen
                        .send::<Failed<14>>(&imp::AnalysisReply::<14, false>::from_bytes(&bytes)?)
                        .await?;
                    endpoint.send::<Specimen<TOXICITY>>(specimen).await?;
                }
            }
            3 => {
                branch.recv::<Plan<3, CELL_SERVICE>>().await?;
                loop {
                    let admitted = imp::decode(endpoint.recv::<Intent<11>>().await?)?;
                    endpoint.send::<Instrument<CELL>>(instrument).await?;
                    let request = imp::decode(endpoint.recv::<Request<11>>().await?)?;
                    imp::same(&request, &admitted)?;
                    instrument = endpoint.recv::<Instrument<CELL>>().await?;
                    let mut specimen = endpoint.recv::<Specimen<BIOASSAY>>().await?;
                    let (actor, tool, _, _) = imp::analysis(11)?;
                    let result = domain
                        .execute(actor, tool, request)
                        .await
                        .map_err(ShellError::message)?;
                    let bytes = imp::encode(&result)?;
                    actual.set(result.clone());
                    if result["success"] == true {
                        specimen
                            .send::<Passed<11>>(&imp::AnalysisReply::<11, true>::from_bytes(
                                &bytes,
                            )?)
                            .await?;
                        endpoint.send::<Specimen<BIOASSAY>>(specimen).await?;
                        break;
                    }
                    specimen
                        .send::<Failed<11>>(&imp::AnalysisReply::<11, false>::from_bytes(&bytes)?)
                        .await?;
                    endpoint.send::<Specimen<BIOASSAY>>(specimen).await?;
                }
                loop {
                    let admitted = imp::decode(endpoint.recv::<Intent<14>>().await?)?;
                    endpoint.send::<Instrument<CELL>>(instrument).await?;
                    let request = imp::decode(endpoint.recv::<Request<14>>().await?)?;
                    imp::same(&request, &admitted)?;
                    instrument = endpoint.recv::<Instrument<CELL>>().await?;
                    let mut specimen = endpoint.recv::<Specimen<TOXICITY>>().await?;
                    let (actor, tool, _, _) = imp::analysis(14)?;
                    let result = domain
                        .execute(actor, tool, request)
                        .await
                        .map_err(ShellError::message)?;
                    let bytes = imp::encode(&result)?;
                    actual.set(result.clone());
                    if result["success"] == true {
                        specimen
                            .send::<Passed<14>>(&imp::AnalysisReply::<14, true>::from_bytes(
                                &bytes,
                            )?)
                            .await?;
                        endpoint.send::<Specimen<TOXICITY>>(specimen).await?;
                        break;
                    }
                    specimen
                        .send::<Failed<14>>(&imp::AnalysisReply::<14, false>::from_bytes(&bytes)?)
                        .await?;
                    endpoint.send::<Specimen<TOXICITY>>(specimen).await?;
                }
            }
            _ => {
                return Err(ShellError::message(
                    "unexpected scientific continuation at the cell laboratory",
                ));
            }
        }
    }
    drop(instrument);
    Ok(())
}
