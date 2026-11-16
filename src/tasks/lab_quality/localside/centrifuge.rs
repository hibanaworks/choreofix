//! Physical centrifuge data and biological candidate custody; no progress table.
use super::super::{global::*, imp};
use crate::shell::{
    ShellError, ShellResult,
    imp::{review_reply::ReviewReply, service::Domain},
};
use hibana::{Endpoint, g::Message};
use serde_json::{Value, json};
use std::cell::Cell;
#[hibana_runtime::scope]
pub async fn run<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, CENT_SERVICE>,
    (candidate, actual): (Endpoint<'r, BIOLOGY>, &Cell<Value>),
    domain: &D,
) -> ShellResult<()> {
    let mut instrument = endpoint.recv::<CentLoan>().await?;
    let admitted = imp::decode(endpoint.recv::<Preparation>().await?)?;
    let arguments = imp::arguments::<3>(&admitted)?;
    endpoint.send::<CentLoan>(instrument).await?;
    let request = imp::decode(endpoint.recv::<CentRequest>().await?)?;
    imp::same_value(&request, &admitted)?;
    let preparation = domain
        .execute("BIOLOGIST", "run_separation", arguments.clone())
        .await
        .map_err(ShellError::message)?;
    let cells_id = imp::record(
        &preparation,
        ("run_separation", "BIOLOGIST", &arguments),
        &json!({}),
    )?;
    let bytes = imp::encode(&preparation)?;
    endpoint.send::<InstrumentResult>(&bytes.as_slice()).await?;
    instrument = endpoint.recv::<CentLoan>().await?;
    let admitted = imp::decode(endpoint.recv::<Purification>().await?)?;
    let arguments = imp::arguments::<2>(&admitted["arguments"])?;
    let synthesis_id = imp::synthesis_id(&admitted["synthesis"])?;
    let analysis_id = imp::record(
        &admitted["analysis"],
        (
            "run_analysis",
            "CHEMIST",
            &json!({"sample":"synthesized_compound"}),
        ),
        &json!({"synthesis":synthesis_id}),
    )?;
    endpoint.send::<CentLoan>(instrument).await?;
    let request = imp::decode(endpoint.recv::<CentRequest>().await?)?;
    imp::same_value(&request, &admitted)?;
    let purification = domain
        .execute("CHEMIST", "run_separation", arguments.clone())
        .await
        .map_err(ShellError::message)?;
    imp::record(
        &purification,
        ("run_separation", "CHEMIST", &arguments),
        &json!({"synthesis":synthesis_id,"analysis":analysis_id}),
    )?;
    let sample = json!({"synthesis":admitted["synthesis"],"initial_analysis":admitted["analysis"],"purification":purification,"analysis":admitted["analysis"]});
    imp::specimen(&sample)?;
    let bytes = imp::encode(&purification)?;
    endpoint.send::<InstrumentResult>(&bytes.as_slice()).await?;
    instrument = endpoint.recv::<CentLoan>().await?;
    let bytes = imp::encode(&sample)?;
    endpoint
        .send::<PurificationProof>(&bytes.as_slice())
        .await?;
    let calibration = imp::decode(endpoint.recv::<CalibrationProof>().await?)?;
    imp::record(
        &calibration,
        (
            "run_analysis",
            "BIOLOGIST",
            &json!({"sample":"fluorescence"}),
        ),
        &json!({"prepared_cells":cells_id}),
    )?;
    endpoint.send::<BioAccess>(candidate).await?;
    let mut biology = loop {
        let request = imp::decode(endpoint.recv::<BioRequest>().await?)?;
        let arguments = imp::arguments::<5>(&request["arguments"])?;
        for key in ["synthesis", "initial_analysis", "purification", "analysis"] {
            imp::same_value(&request["materials"][key], &sample[key])?;
        }
        imp::same_value(&request["materials"]["cells"], &preparation)?;
        imp::same_value(&request["materials"]["calibration"], &calibration)?;
        let mut candidate = endpoint.recv::<BioAccess>().await?;
        let result = if let Err(error) =
            imp::bio_arguments(&arguments, &sample["synthesis"]["data"]["compound"])
        {
            json!({"tool_name":"run_bioassay","success":false,"data":{
   "scientist":"BIOLOGIST","compound":arguments["compound"],
   "cell_type":arguments.get("cell_type").and_then(Value::as_str).unwrap_or(""),
   "inputs":imp::bio_inputs(&request["materials"])?,"artifact_id":null,"status":"invalid_arguments","missing_inputs":[],
   "error":error.to_string()}})
        } else {
            domain
                .execute("BIOLOGIST", "run_bioassay", arguments.clone())
                .await
                .map_err(ShellError::message)?
        };
        imp::bio_result(&result, &arguments, &request["materials"])?;
        let mut materials = request["materials"].clone();
        materials["bioassay"] = result.clone();
        let proof = json!({"success":result["success"],"result":result,"arguments":arguments,"materials":materials});
        let result_bytes = imp::encode(&result)?;
        let proof_bytes = imp::encode(&proof)?;
        actual.set(result.clone());
        match result["success"].as_bool() {
            Some(true) => {
                let reply = ReviewReply::<true>::from_bytes(&result_bytes)?;
                candidate.send::<BioPassed>(&reply).await?;
                let observed = ReviewReply::<true>::from_bytes(&proof_bytes)?;
                async {
                    let [mut fume, mut spec] = endpoint.par()?;
                    async {
                        fume.send::<BioFumePassed>(&observed).await?;
                    };
                    async {
                        spec.send::<BioSpecPassed>(&observed).await?;
                    };
                    Ok(())
                }
                .await?;
                endpoint.send::<BioAccess>(candidate).await?;
            }
            Some(false) => {
                let reply = ReviewReply::<false>::from_bytes(&result_bytes)?;
                candidate.send::<BioFailed>(&reply).await?;
                let observed = ReviewReply::<false>::from_bytes(&proof_bytes)?;
                async {
                    let [mut fume, mut spec] = endpoint.par()?;
                    async {
                        fume.send::<BioFumeFailed>(&observed).await?;
                    };
                    async {
                        spec.send::<BioSpecFailed>(&observed).await?;
                    };
                    Ok(())
                }
                .await?;
                endpoint.send::<BioAccess>(candidate).await?;
            }
            None => return Err(ShellError::message("missing actual biological result")),
        }
        if result["success"] == true {
            break proof;
        }
    };
    let mut chemist_qc = Value::Null;
    let mut biologist_qc = Value::Null;
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            ChemistApproved::LOGICAL_LABEL => {
                chemist_qc = imp::decode(branch.recv::<ChemistApproved>().await?)?;
            }
            BiologistApproved::LOGICAL_LABEL => {
                biologist_qc = imp::decode(branch.recv::<BiologistApproved>().await?)?;
            }
            ChemistRejected::LOGICAL_LABEL => {
                branch.recv::<ChemistRejected>().await?;
            }
            BiologistRejected::LOGICAL_LABEL => {
                branch.recv::<BiologistRejected>().await?;
            }
            LastChemistRejected::LOGICAL_LABEL => {
                branch.recv::<LastChemistRejected>().await?;
            }
            LastBiologistRejected::LOGICAL_LABEL => {
                branch.recv::<LastBiologistRejected>().await?;
            }
            LastChemistApproved::LOGICAL_LABEL => {
                chemist_qc = imp::decode(branch.recv::<LastChemistApproved>().await?)?;
                break;
            }
            LastBiologistApproved::LOGICAL_LABEL => {
                biologist_qc = imp::decode(branch.recv::<LastBiologistApproved>().await?)?;
                break;
            }
            Retest::LOGICAL_LABEL => {
                let request = imp::decode(branch.recv::<Retest>().await?)?;
                let arguments = imp::arguments::<5>(&request["arguments"])?;
                for key in ["synthesis", "initial_analysis", "purification", "analysis"] {
                    imp::same_value(&request["materials"][key], &sample[key])?;
                }
                imp::same_value(&request["materials"]["cells"], &preparation)?;
                imp::same_value(&request["materials"]["calibration"], &calibration)?;
                let mut candidate = endpoint.recv::<BioAccess>().await?;
                let result = if let Err(error) =
                    imp::bio_arguments(&arguments, &sample["synthesis"]["data"]["compound"])
                {
                    json!({"tool_name":"run_bioassay","success":false,"data":{
   "scientist":"BIOLOGIST","compound":arguments["compound"],
   "cell_type":arguments.get("cell_type").and_then(Value::as_str).unwrap_or(""),
   "inputs":imp::bio_inputs(&request["materials"])?,"artifact_id":null,"status":"invalid_arguments","missing_inputs":[],
   "error":error.to_string()}})
                } else {
                    domain
                        .execute("BIOLOGIST", "run_bioassay", arguments.clone())
                        .await
                        .map_err(ShellError::message)?
                };
                imp::bio_result(&result, &arguments, &request["materials"])?;
                let mut materials = request["materials"].clone();
                materials["bioassay"] = result.clone();
                let proof = json!({"success":result["success"],"result":result,"arguments":arguments,"materials":materials});
                let result_bytes = imp::encode(&result)?;
                let proof_bytes = imp::encode(&proof)?;
                actual.set(result.clone());
                match result["success"].as_bool() {
                    Some(true) => {
                        let reply = ReviewReply::<true>::from_bytes(&result_bytes)?;
                        candidate.send::<RetestPassed>(&reply).await?;
                        let observed = ReviewReply::<true>::from_bytes(&proof_bytes)?;
                        async {
                            let [mut fume, mut spec] = endpoint.par()?;
                            async {
                                fume.send::<RetestFumePassed>(&observed).await?;
                            };
                            async {
                                spec.send::<RetestSpecPassed>(&observed).await?;
                            };
                            Ok(())
                        }
                        .await?;
                        endpoint.send::<BioAccess>(candidate).await?;
                    }
                    Some(false) => {
                        let reply = ReviewReply::<false>::from_bytes(&result_bytes)?;
                        candidate.send::<RetestFailed>(&reply).await?;
                        let observed = ReviewReply::<false>::from_bytes(&proof_bytes)?;
                        async {
                            let [mut fume, mut spec] = endpoint.par()?;
                            async {
                                fume.send::<RetestFumeFailed>(&observed).await?;
                            };
                            async {
                                spec.send::<RetestSpecFailed>(&observed).await?;
                            };
                            Ok(())
                        }
                        .await?;
                        endpoint.send::<BioAccess>(candidate).await?;
                    }
                    None => return Err(ShellError::message("missing actual biological result")),
                }
                biology = proof;
            }
            LastRetest::LOGICAL_LABEL => {
                let request = imp::decode(branch.recv::<LastRetest>().await?)?;
                let arguments = imp::arguments::<5>(&request["arguments"])?;
                for key in ["synthesis", "initial_analysis", "purification", "analysis"] {
                    imp::same_value(&request["materials"][key], &sample[key])?;
                }
                imp::same_value(&request["materials"]["cells"], &preparation)?;
                imp::same_value(&request["materials"]["calibration"], &calibration)?;
                let mut candidate = endpoint.recv::<BioAccess>().await?;
                let result = if let Err(error) =
                    imp::bio_arguments(&arguments, &sample["synthesis"]["data"]["compound"])
                {
                    json!({"tool_name":"run_bioassay","success":false,"data":{
   "scientist":"BIOLOGIST","compound":arguments["compound"],
   "cell_type":arguments.get("cell_type").and_then(Value::as_str).unwrap_or(""),
   "inputs":imp::bio_inputs(&request["materials"])?,"artifact_id":null,"status":"invalid_arguments","missing_inputs":[],
   "error":error.to_string()}})
                } else {
                    domain
                        .execute("BIOLOGIST", "run_bioassay", arguments.clone())
                        .await
                        .map_err(ShellError::message)?
                };
                imp::bio_result(&result, &arguments, &request["materials"])?;
                let mut materials = request["materials"].clone();
                materials["bioassay"] = result.clone();
                let proof = json!({"success":result["success"],"result":result,"arguments":arguments,"materials":materials});
                let result_bytes = imp::encode(&result)?;
                let proof_bytes = imp::encode(&proof)?;
                actual.set(result.clone());
                match result["success"].as_bool() {
                    Some(true) => {
                        let reply = ReviewReply::<true>::from_bytes(&result_bytes)?;
                        candidate.send::<LastRetestPassed>(&reply).await?;
                        let observed = ReviewReply::<true>::from_bytes(&proof_bytes)?;
                        async {
                            let [mut fume, mut spec] = endpoint.par()?;
                            async {
                                fume.send::<LastRetestFumePassed>(&observed).await?;
                            };
                            async {
                                spec.send::<LastRetestSpecPassed>(&observed).await?;
                            };
                            Ok(())
                        }
                        .await?;
                        endpoint.send::<BioAccess>(candidate).await?;
                    }
                    Some(false) => {
                        let reply = ReviewReply::<false>::from_bytes(&result_bytes)?;
                        candidate.send::<LastRetestFailed>(&reply).await?;
                        let observed = ReviewReply::<false>::from_bytes(&proof_bytes)?;
                        async {
                            let [mut fume, mut spec] = endpoint.par()?;
                            async {
                                fume.send::<LastRetestFumeFailed>(&observed).await?;
                            };
                            async {
                                spec.send::<LastRetestSpecFailed>(&observed).await?;
                            };
                            Ok(())
                        }
                        .await?;
                        endpoint.send::<BioAccess>(candidate).await?;
                    }
                    None => return Err(ShellError::message("missing actual biological result")),
                }
                biology = proof;
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected centrifuge operation {label}"
                )));
            }
        }
    }
    let admitted = imp::decode(endpoint.recv::<Separation>().await?)?;
    let arguments = imp::arguments::<7>(&admitted["arguments"])?;
    let approved = &admitted["approved"];
    let inputs = imp::approved_inputs(approved)?;
    imp::same_value(&approved["chemist_qc"], &chemist_qc)?;
    imp::same_value(&approved["biologist_qc"], &biologist_qc)?;
    for key in ["synthesis", "initial_analysis", "purification"] {
        imp::same_value(&approved["materials"][key], &sample[key])?;
    }
    imp::same_value(&approved["materials"]["cells"], &preparation)?;
    imp::same_value(&approved["materials"]["calibration"], &calibration)?;
    imp::same_value(&approved["materials"]["bioassay"], &biology["result"])?;
    endpoint.send::<CentLoan>(instrument).await?;
    let request = imp::decode(endpoint.recv::<CentRequest>().await?)?;
    imp::same_value(&request, &admitted)?;
    let result = domain
        .execute("ANALYST", "run_separation", arguments.clone())
        .await
        .map_err(ShellError::message)?;
    imp::record(&result, ("run_separation", "ANALYST", &arguments), &inputs)?;
    let bytes = imp::encode(&result)?;
    endpoint.send::<InstrumentResult>(&bytes.as_slice()).await?;
    let instrument = endpoint.recv::<CentLoan>().await?;
    endpoint
        .send::<VerificationProof>(&bytes.as_slice())
        .await?;
    endpoint.send::<CentLoan>(instrument).await?;
    Ok(())
}
