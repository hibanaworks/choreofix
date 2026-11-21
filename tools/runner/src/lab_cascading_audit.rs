//! Bind 13H effects to committed file and instrument requests and actual replies.
use crate::{
    evidence::Result,
    journal::{self, one},
    projection,
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, ops::Range};

type Edge = (&'static str, &'static str, &'static str);
type Instrument = (&'static str, &'static str, &'static str, &'static str);

pub(crate) fn check(rows: &[Value], artifact: &Value, dynamic: &Value) -> Result<Value> {
    let compiled = &artifact["compiled"];
    let fixture = &rows.first().ok_or("empty scientific history")?["message"];
    let actors = crate::journal::participants(
        fixture,
        include_str!("../../../benchmark/descriptions/13H/metadata.json"),
    )?;
    if compiled["task"] != "13H" || fixture["type"] != "fixtures" || fixture["task"] != "13H" {
        return Err("foreign task fixture or projection".into());
    }
    let role = |name: &str| {
        compiled["roles"][name]
            .as_u64()
            .ok_or_else(|| format!("compiled role {name}"))
    };
    let label = |name: &str| {
        compiled["labels"][name]
            .as_u64()
            .ok_or_else(|| format!("compiled message {name}"))
    };
    let headers = projection::frames(artifact, rows)?;
    let journal = journal::read(rows, &headers, actors, dynamic)?;
    let journal::Journal {
        frames,
        calls,
        score,
    } = journal;
    let score_events = score["events"]
        .as_array()
        .ok_or("scientific score events")?;
    if calls.len() != score_events.len()
        || score["violations"] != json!([])
        || frames
            .values()
            .any(|frame| frame.identity["session"] != compiled["session"])
    {
        return Err("incomplete scientific history or physical resource violation".into());
    }
    let pairs = [
        ("CHEM_VM", "CHEMIST_IO"),
        ("BIO_VM", "BIOLOGIST_IO"),
        ("ANALYST_VM", "ANALYST_IO"),
        ("QC_VM", "QC_IO"),
        ("SAFETY_VM", "SAFETY_IO"),
    ]
    .into_iter()
    .map(|(vm, io)| Ok((role(vm)?, role(io)?)))
    .collect::<Result<Vec<_>>>()?;
    journal::check_close_pairs(&frames, &pairs)?;

    let mut used = BTreeSet::new();
    let mut relevant = BTreeSet::new();
    let mut take = |(from, to, msg): Edge, interval: Range<usize>| -> Result<(usize, Value)> {
        let key = (role(from)?, role(to)?, label(msg)?);
        relevant.insert(key);
        let (id, frame) = frames
            .iter()
            .filter(|(id, f)| {
                !used.contains(*id)
                    && f.direction(key.0, key.1)
                    && u64::from(f.logical_label) == key.2
                    && interval.contains(&f.received)
            })
            .min_by_key(|(_, frame)| frame.received)
            .ok_or_else(|| {
                format!("missing committed scientific {msg} {from}->{to} in {interval:?}")
            })?;
        used.insert(*id);
        Ok((frame.received, frame.value()?))
    };
    let mut scored = BTreeSet::new();
    let mut ordered = calls.values().collect::<Vec<_>>();
    ordered.sort_by_key(|call| call.emitted);
    let mut inspection = None;
    let mut terminal = None;
    let mut final_validation = None;
    for call in ordered {
        let who = call.request["agent"].as_str().ok_or("scientific agent")?;
        let input = &call.request["arguments"];
        let tool = call.tool;
        let (returned, output) = call.result;
        let success = output["success"]
            .as_bool()
            .ok_or("scientific result status")?;
        let (file, carrier, request, reply, instruments): (_, _, _, _, &[Instrument]) =
            match (who, tool) {
                ("CHEMIST", "run_synthesis") => {
                    if output["data"]["status"] == "reagent_exhausted" && !success {
                        terminal = Some((returned, output));
                    }
                    (
                        "CHEM_FILE",
                        "CHEMIST",
                        "FileSynthesis",
                        "FileSynthesisResult",
                        &[
                            (
                                "FUME_HOOD",
                                "FUME_OWNER",
                                "FumeSynthesis",
                                "FumeSynthesisResult",
                            ),
                            (
                                "REAGENT_SUPPLY",
                                "FUME_OWNER",
                                "ReagentSynthesis",
                                "ReagentSynthesisResult",
                            ),
                        ],
                    )
                }
                ("SAFETY_OFFICER", "perform_inspection") if success => {
                    if inspection.replace(returned).is_some() {
                        return Err("multiple inspections".into());
                    }
                    (
                        "SAFETY_FILE",
                        "SAFETY_OFFICER",
                        "FileInspection",
                        "FileInspectionResult",
                        &[
                            (
                                "SPECTROMETER",
                                "INSPECTION_SERVICE",
                                "SpecInspection",
                                "SpecResult",
                            ),
                            (
                                "CENTRIFUGE",
                                "INSPECTION_SERVICE",
                                "CentInspection",
                                "CentResult",
                            ),
                            (
                                "FUME_HOOD",
                                "INSPECTION_SERVICE",
                                "FumeInspection",
                                "FumeResult",
                            ),
                            (
                                "CHROMATOGRAPH",
                                "INSPECTION_SERVICE",
                                "ChromaInspection",
                                "ChromaResult",
                            ),
                        ],
                    )
                }
                ("QC_CONTROLLER", "verify_result")
                    if input["scientist"] == "BIOLOGIST" && input["data_type"] == "bioassay" =>
                {
                    if success {
                        terminal = Some((returned, output));
                    }
                    let resources: &[Instrument] = if success {
                        &[
                            (
                                "FUME_HOOD",
                                "FUME_OWNER",
                                "FumeQuality",
                                "FumeQualityApproved",
                            ),
                            (
                                "SPECTROMETER",
                                "FUME_OWNER",
                                "SpecQuality",
                                "SpecQualityApproved",
                            ),
                        ]
                    } else {
                        &[
                            (
                                "FUME_HOOD",
                                "FUME_OWNER",
                                "FumeQuality",
                                "FumeQualityRejected",
                            ),
                            (
                                "SPECTROMETER",
                                "FUME_OWNER",
                                "SpecQuality",
                                "SpecQualityRejected",
                            ),
                        ]
                    };
                    (
                        "QC_FILE",
                        "QC",
                        "FileQuality",
                        "FileQualityResult",
                        resources,
                    )
                }
                ("CHEMIST", "run_analysis") if input["sample"] == "synthesized_compound" => (
                    "CHEM_FILE",
                    "CHEM_CONTROL",
                    "FileAnalysis",
                    "FileAnalysisResult",
                    &[("SPECTROMETER", "SPEC_OWNER", "Analysis", "AnalysisResult")],
                ),
                ("CHEMIST", "run_analysis") if input["sample"] == "final_validation" && success => {
                    if final_validation.replace((returned, output)).is_some() {
                        return Err("multiple final validations".into());
                    }
                    (
                        "CHEM_FILE",
                        "CHEM_CONTROL",
                        "FileFinal",
                        "FileFinalResult",
                        &[(
                            "SPECTROMETER",
                            "SPEC_OWNER",
                            "FinalValidation",
                            "FinalResult",
                        )],
                    )
                }
                ("CHEMIST", "run_separation") if input["material"] == "compound_purification" => (
                    "CHEM_FILE",
                    "CHEM_CONTROL",
                    "FilePurification",
                    "FilePurificationResult",
                    &[(
                        "CENTRIFUGE",
                        "CENT_OWNER",
                        "Purification",
                        "PurificationResult",
                    )],
                ),
                ("BIOLOGIST", "run_separation") if input["material"] == "cell_cultures" => (
                    "BIO_FILE",
                    "BIO_CONTROL",
                    "FileCells",
                    "FileCellsResult",
                    &[("CENTRIFUGE", "CENT_OWNER", "Cells", "CellsResult")],
                ),
                ("BIOLOGIST", "run_analysis") if input["sample"] == "fluorescence" => (
                    "BIO_FILE",
                    "BIO_CONTROL",
                    "FileCalibration",
                    "FileCalibrationResult",
                    &[(
                        "SPECTROMETER",
                        "SPEC_OWNER",
                        "Calibration",
                        "CalibrationResult",
                    )],
                ),
                ("BIOLOGIST", "run_bioassay") => (
                    "BIO_FILE",
                    "BIOLOGIST",
                    "FileBioassay",
                    "FileBioassayResult",
                    &[("BIOLOGIST", "CENT_OWNER", "Assay", "AssayResult")],
                ),
                ("ANALYST", "run_chromatography") => (
                    "ANALYST_FILE",
                    "ANALYST",
                    "FileChromatography",
                    "FileChromatographyResult",
                    &[(
                        "CHROMATOGRAPH",
                        "CHROMA_OWNER",
                        "Chromatography",
                        "ChromatographyResult",
                    )],
                ),
                ("ANALYST", "run_analysis") if input["sample"] == "molecular_detail" => (
                    "ANALYST_FILE",
                    "ANALYST",
                    "FileMolecular",
                    "FileMolecularResult",
                    &[("SPECTROMETER", "SPEC_OWNER", "Molecular", "MolecularResult")],
                ),
                ("ANALYST", "run_separation") if input["material"] == "verification" => (
                    "ANALYST_FILE",
                    "ANALYST",
                    "FileVerification",
                    "FileVerificationResult",
                    &[(
                        "CENTRIFUGE",
                        "CENT_OWNER",
                        "Verification",
                        "VerificationResult",
                    )],
                ),
                _ => return Err(format!("unknown scientific effect {who} {tool}").into()),
            };
        let (admitted, args) = take((file, carrier, request), 0..call.emitted)?;
        if &args != input {
            return Err(format!("{request} arguments differ from the actual effect").into());
        }
        let mut packet = None;
        let mut replied = returned;
        for &(resource, owner, request, reply) in instruments {
            let (_, actual) = take((resource, owner, request), admitted + 1..call.emitted)?;
            let args = if matches!(tool, "run_synthesis" | "perform_inspection") {
                &actual
            } else {
                &actual["arguments"]
            };
            if args != input || packet.as_ref().is_some_and(|packet| packet != &actual) {
                return Err(format!("{request} differs from the admitted scientific input").into());
            }
            packet = Some(actual);
            let (at, actual) = take((owner, resource, reply), returned + 1..rows.len())?;
            if &actual != output {
                return Err(
                    format!("{reply} differs from the actual scientific effect result").into(),
                );
            }
            replied = replied.max(at);
        }
        let (_, displayed) = take((carrier, file, reply), replied + 1..rows.len())?;
        if &displayed != output {
            return Err(format!("{reply} differs from the actual scientific effect result").into());
        }
        let material = match (who, tool, success) {
            ("CHEMIST", "run_separation", true) if input["material"] == "compound_purification" => {
                Some((("BIO_CONTROL", "BIO_FILE", "FileBioBatch"), "purification"))
            }
            ("BIOLOGIST", "run_bioassay", true) => {
                Some((("ANALYST", "ANALYST_FILE", "FileAnalystBatch"), "bioassay"))
            }
            _ => None,
        };
        if let Some((edge, key)) = material {
            let (_, batch) = take(edge, returned + 1..rows.len())?;
            if &batch[key] != output {
                return Err(format!("{} differs from the actual {key} result", edge.2).into());
            }
        }
        let notices: &[Edge] = match (tool, success, output["data"]["status"].as_str()) {
            ("run_synthesis", false, Some("synthesis_failed")) => &[
                ("BIO_CONTROL", "BIO_FILE", "FileBioBatch"),
                ("ANALYST", "ANALYST_FILE", "FileAnalystBatch"),
            ],
            ("verify_result", false, _) => &[
                ("CHEM_CONTROL", "CHEM_FILE", "FileChemQuality"),
                ("BIO_CONTROL", "BIO_FILE", "FileBioBatch"),
                ("ANALYST", "ANALYST_FILE", "FileAnalystBatch"),
            ],
            ("verify_result", true, _) => &[
                ("CHEM_CONTROL", "CHEM_FILE", "FileChemQuality"),
                ("BIO_CONTROL", "BIO_FILE", "FileBioBatch"),
                ("ANALYST", "ANALYST_FILE", "FileAnalystBatch"),
            ],
            _ => &[],
        };
        for &edge in notices {
            let (_, notice) = take(edge, returned + 1..rows.len())?;
            if &notice != output {
                return Err(
                    format!("{} differs from the actual QC or synthesis result", edge.2).into(),
                );
            }
        }
        // The common world's inspection method records its default argument.
        // The file, resource and effect comparisons above still use the submitted JSON.
        let default_inspection;
        let scored_input = if tool == "perform_inspection"
            && input.as_object().is_some_and(serde_json::Map::is_empty)
        {
            default_inspection = json!({"notes":""});
            &default_inspection
        } else {
            input
        };
        let (index, _) = one(score_events.iter().enumerate().filter(|(index, event)| {
            !scored.contains(index) && call.matches_score(event, scored_input)
        }))?;
        scored.insert(index);
    }
    let (at, terminal) = terminal.ok_or("missing actual scientific verdict")?;
    let complete = terminal["success"] == true;
    if inspection.is_none()
        || score["complete"] != complete
        || complete != final_validation.is_some()
        || final_validation.is_some_and(|(validation, result)| {
            validation <= at
                || result["data"]["inputs"]["bioassay_qc"] != terminal["data"]["artifact_id"]
        })
    {
        return Err("terminal scientific outcome differs from the common score".into());
    }
    let expected = if let Some((returned, validation)) = final_validation {
        let (_, received) = take(
            ("CHEM_CONTROL", "QC", "FinalValidated"),
            returned + 1..rows.len(),
        )?;
        if &received != validation {
            return Err("QC received a different final validation result".into());
        }
        validation
    } else {
        for (file, name) in [
            ("BIO_FILE", "FileBioBatch"),
            ("ANALYST_FILE", "FileAnalystBatch"),
            ("QC_FILE", "FileQcQuality"),
        ] {
            let (_, received) = take(("QC", file, name), at + 1..rows.len())?;
            if &received != terminal {
                return Err(
                    format!("{name} differs from the actual exhausted-stock result").into(),
                );
            }
        }
        terminal
    };
    for (&id, frame) in &frames {
        let from = frame.identity["source"]
            .as_u64()
            .ok_or("scientific source role")?;
        let to = frame.identity["target"]
            .as_u64()
            .ok_or("scientific target role")?;
        if relevant.contains(&(from, to, u64::from(frame.logical_label))) && !used.contains(&id) {
            return Err(format!("unmatched scientific request or result frame {id}").into());
        }
    }
    Ok(
        json!({"status":"passed","task":"13H","domain_calls":calls.len(),"violations":0,
        "simulator_complete":complete,"terminal_result":expected,"guest_exits":actors.len(),
        "effect_receives_bound_to_checked_occurrences":true,"dynamic_protocol":dynamic,"scored_benchmark":false}),
    )
}
