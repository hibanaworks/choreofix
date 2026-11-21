//! Actual scientific requests/effects/results. Dynamic source execution belongs to EPF.
use crate::{
    evidence::{Result, digest},
    journal::{self, Call, Frame, one},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
type Calls<'a> = BTreeMap<u64, Call<'a>>;
type Frames<'a> = BTreeMap<u64, Frame<'a>>;
fn target<'a>(call: &Call<'a>) -> Result<&'a str> {
    let key = match call.tool {
        "run_analysis" => "sample",
        "run_separation" => "material",
        "run_synthesis" | "run_bioassay" => "compound",
        "verify_result" => "scientist",
        _ => return Err("unknown scientific effect".into()),
    };
    call.request["arguments"][key]
        .as_str()
        .ok_or_else(|| "scientific target".into())
}
fn shape(value: &Value, required: &[&str], optional: &[&str]) -> Result<()> {
    let object = value.as_object().ok_or("scientific object")?;
    if required.iter().any(|k| !object.contains_key(*k))
        || object
            .keys()
            .any(|k| !required.contains(&k.as_str()) && !optional.contains(&k.as_str()))
    {
        return Err("scientific object has missing or unadmitted fields".into());
    }
    Ok(())
}
fn frame<'a>(
    (frames, used): (&'a Frames<'a>, &mut BTreeSet<usize>),
    (from, to, labels): (u64, u64, &[u64]),
    (expected, after, before): (Option<&Value>, usize, usize),
) -> Result<&'a Frame<'a>> {
    let found = frames
        .values()
        .filter(|f| {
            f.direction(from, to)
                && labels.contains(&u64::from(f.logical_label))
                && !used.contains(&f.sent)
                && after < f.sent
                && f.sent < before
                && expected.is_none_or(|v| f.value().is_ok_and(|x| x == *v))
        })
        .min_by_key(|f| f.sent)
        .ok_or_else(|| {
            format!(
                "missing scientific frame {from}->{to} labels={labels:?} interval={after}..{before}"
            )
        })?;
    if found.received >= before {
        return Err(format!("scientific receive {from}->{to} labels={labels:?}, send={}, commit={}, required_before={before}",found.sent,found.received).into());
    }
    used.insert(found.sent);
    Ok(found)
}
fn known(
    (calls, before): (&Calls, usize),
    value: &Value,
    (actor, tool, name): (&str, &str, &str),
) -> Result<u64> {
    let call = one(calls.values().filter(|c| {
        c.request["agent"] == actor
            && c.tool == tool
            && target(c).ok() == Some(name)
            && c.result.0 < before
            && c.result.1 == value
            && c.result.1["success"] == true
    }))?;
    call.result.1["data"]["artifact_id"]
        .as_u64()
        .ok_or_else(|| "missing actual positive scientific receipt".into())
}
fn specimen(context: (&Calls, usize), materials: &Value) -> Result<Value> {
    let compound = materials["synthesis"]["data"]["compound"]
        .as_str()
        .ok_or("actual synthesis name")?;
    let synthesis = known(
        context,
        &materials["synthesis"],
        ("CHEMIST", "run_synthesis", compound),
    )?;
    let initial = known(
        context,
        &materials["initial_analysis"],
        ("CHEMIST", "run_analysis", "synthesized_compound"),
    )?;
    let analysis = known(
        context,
        &materials["analysis"],
        ("CHEMIST", "run_analysis", "synthesized_compound"),
    )?;
    let purification = known(
        context,
        &materials["purification"],
        ("CHEMIST", "run_separation", "compound_purification"),
    )?;
    if materials["synthesis"]["data"]["inputs"] != json!({})
        || materials["initial_analysis"]["data"]["inputs"] != json!({"synthesis":synthesis})
        || materials["analysis"]["data"]["inputs"] != json!({"synthesis":synthesis})
        || materials["purification"]["data"]["inputs"]
            != json!({"synthesis":synthesis,"analysis":initial})
        || materials["purification"]["data"]["compound"] != compound
    {
        return Err("scientific specimen mixes batches or measurements".into());
    }
    Ok(json!({"synthesis":synthesis,"purification":purification,"analysis":analysis}))
}
fn biological(context: (&Calls, usize), materials: &Value) -> Result<Value> {
    shape(
        materials,
        &[
            "synthesis",
            "initial_analysis",
            "purification",
            "analysis",
            "cells",
            "calibration",
        ],
        &["bioassay"],
    )?;
    let chemical = specimen(context, materials)?;
    let cells = known(
        context,
        &materials["cells"],
        ("BIOLOGIST", "run_separation", "cell_cultures"),
    )?;
    let calibration = known(
        context,
        &materials["calibration"],
        ("BIOLOGIST", "run_analysis", "fluorescence"),
    )?;
    if materials["cells"]["data"]["inputs"] != json!({})
        || materials["calibration"]["data"]["inputs"] != json!({"prepared_cells":cells})
    {
        return Err("biological trial substituted cell/calibration receipts".into());
    }
    Ok(
        json!({"synthesis":chemical["synthesis"],"purification":chemical["purification"],
  "prepared_cells":cells,"calibration":calibration}),
    )
}
fn qc_inputs(context: (&Calls, usize), args: &Value, materials: &Value) -> Result<Value> {
    let mut inputs = specimen(context, materials)?;
    match args["scientist"].as_str() {
        Some("CHEMIST") if args["data_type"] == "purification" => {
            shape(
                materials,
                &["synthesis", "initial_analysis", "purification", "analysis"],
                &[],
            )?;
        }
        Some("BIOLOGIST") if args["data_type"] == "bioassay" => {
            let expected = biological(context, materials)?;
            let compound = materials["synthesis"]["data"]["compound"]
                .as_str()
                .ok_or("compound name")?;
            let assay = known(
                context,
                &materials["bioassay"],
                ("BIOLOGIST", "run_bioassay", compound),
            )?;
            if materials["bioassay"]["data"]["inputs"] != expected {
                return Err("QC uses foreign biological materials".into());
            }
            inputs
                .as_object_mut()
                .ok_or("QC inputs")?
                .remove("analysis");
            inputs["bioassay"] = json!(assay);
        }
        _ => return Err("unknown QC subject/data".into()),
    }
    Ok(inputs)
}
fn approved(context: (&Calls, usize), value: &Value) -> Result<Value> {
    shape(value, &["materials", "chemist_qc", "biologist_qc"], &[])?;
    let materials = &value["materials"];
    let inputs = biological(context, materials)?;
    let compound = materials["synthesis"]["data"]["compound"]
        .as_str()
        .ok_or("compound name")?;
    let assay = known(
        context,
        &materials["bioassay"],
        ("BIOLOGIST", "run_bioassay", compound),
    )?;
    if materials["bioassay"]["data"]["inputs"] != inputs {
        return Err("approved assay mixes specimens".into());
    }
    for (key, subject, datatype) in [
        ("chemist_qc", "CHEMIST", "purification"),
        ("biologist_qc", "BIOLOGIST", "bioassay"),
    ] {
        known(
            context,
            &value[key],
            ("QC_CONTROLLER", "verify_result", subject),
        )?;
        if value[key]["data"]["inputs"]
            != qc_inputs(
                context,
                &json!({"scientist":subject,"data_type":datatype}),
                &materials_for_subject(materials, subject),
            )?
        {
            return Err("approval describes another actual material or trial".into());
        }
    }
    Ok(
        json!({"synthesis":inputs["synthesis"],"purification":inputs["purification"],"bioassay":assay}),
    )
}
fn materials_for_subject(materials: &Value, subject: &str) -> Value {
    if subject == "CHEMIST" {
        json!({"synthesis":materials["synthesis"],"initial_analysis":materials["initial_analysis"],
  "purification":materials["purification"],"analysis":materials["analysis"]})
    } else {
        materials.clone()
    }
}
fn quality(context: (&Calls, usize), value: &Value) -> Result<Value> {
    shape(
        value,
        &[
            "materials",
            "chemist_qc",
            "biologist_qc",
            "molecular",
            "verification",
        ],
        &[],
    )?;
    let approved_value = json!({"materials":value["materials"],"chemist_qc":value["chemist_qc"],"biologist_qc":value["biologist_qc"]});
    let mut inputs = approved(context, &approved_value)?;
    let molecular = known(
        context,
        &value["molecular"],
        ("ANALYST", "run_analysis", "molecular_detail"),
    )?;
    let verification = known(
        context,
        &value["verification"],
        ("ANALYST", "run_separation", "verification"),
    )?;
    if value["molecular"]["data"]["inputs"] != inputs
        || value["verification"]["data"]["inputs"] != inputs
    {
        return Err("quality measurements describe another actual trial".into());
    }
    inputs["purification_qc"] = value["chemist_qc"]["data"]["artifact_id"].clone();
    inputs["bioassay_qc"] = value["biologist_qc"]["data"]["artifact_id"].clone();
    inputs["molecular"] = json!(molecular);
    inputs["verification"] = json!(verification);
    Ok(inputs)
}
pub(crate) fn check(rows: &[Value], artifact: &Value, dynamic: &Value) -> Result<Value> {
    let compiled = &artifact["compiled"];
    let fixture = &rows.first().ok_or("empty QC history")?["message"];
    if compiled["task"] != "13M" || fixture["type"] != "fixtures" || fixture["task"] != "13M" {
        return Err("foreign QC task/source".into());
    }
    let role = |name: &str| -> Result<u64> {
        compiled["roles"][name]
            .as_u64()
            .ok_or_else(|| format!("compiled QC role {name}").into())
    };
    let label = |name: &str| -> Result<u64> {
        compiled["labels"][name]
            .as_u64()
            .ok_or_else(|| format!("compiled QC message {name}").into())
    };
    let labels = |names: &[&str]| -> Result<Vec<u64>> { names.iter().map(|n| label(n)).collect() };
    let actors = crate::journal::participants(
        fixture,
        include_str!("../../../benchmark/descriptions/13M/metadata.json"),
    )?;

    let contracts = crate::projection::frames(artifact, rows)?;
    let journal = journal::read(rows, &contracts, actors, dynamic)?;
    let journal::Journal {
        frames,
        calls,
        score,
    } = journal;
    let events = score["events"]
        .as_array()
        .ok_or("scientific score events")?;
    if calls.len() != events.len()
        || score["complete"] != true
        || score["violations"] != json!([])
        || frames
            .values()
            .any(|f| f.identity["session"] != compiled["session"])
    {
        return Err("incomplete QC world or physical resource violation".into());
    }
    let pairs = [
        ("CHEM_VM", "CHEMIST_IO"),
        ("BIO_VM", "BIOLOGIST_IO"),
        ("ANALYST_VM", "ANALYST_IO"),
        ("QC_VM", "QC_IO"),
    ]
    .into_iter()
    .map(|(vm, io)| Ok((role(vm)?, role(io)?)))
    .collect::<Result<Vec<_>>>()?;
    journal::check_close_pairs(&frames, &pairs)?;
    let mut used = BTreeSet::new();
    let mut scored = BTreeSet::new();
    let mut ordered = calls.values().collect::<Vec<_>>();
    ordered.sort_by_key(|c| c.emitted);
    for call in ordered {
        let actor = call.request["agent"].as_str().ok_or("scientific actor")?;
        let tool = call.tool;
        let name = target(call)?;
        let args = &call.request["arguments"];
        let (ended, value) = call.result;
        let data = &value["data"];
        let success = value["success"]
            .as_bool()
            .ok_or("actual scientific verdict")?;
        let (required, optional): (&[&str], &[&str]) = match tool {
            "run_analysis" => (&["sample"], &[]),
            "run_separation" => (&["material"], &[]),
            "run_synthesis" => (&["compound"], &[]),
            "run_bioassay" => (&["compound"], &["cell_type"]),
            "verify_result" => (&["scientist", "data_type"], &[]),
            _ => return Err("unknown scientific tool".into()),
        };
        shape(args, required, optional)?;
        if args.get("cell_type").is_some_and(|v| !v.is_string())
            || value["tool_name"] != tool
            || data["missing_inputs"] != json!([])
            || data["scientist"]
                != if tool == "verify_result" {
                    args["scientist"].clone()
                } else {
                    json!(actor)
                }
            || required.iter().any(|k| data[*k] != args[*k])
            || data["status"]
                != if success {
                    "complete"
                } else {
                    match tool {
                        "run_bioassay" => "bioassay_failed",
                        "verify_result" => "rejected",
                        _ => return Err("unexpected failed measurement".into()),
                    }
                }
            || (tool == "verify_result" && data["qc_agent"] != "QC_CONTROLLER")
        {
            return Err("scientific result differs from its actual request/verdict".into());
        }
        let inputs = if tool == "verify_result" {
            if actor != "QC_CONTROLLER" {
                return Err("non-QC verification effect".into());
            }
            let (native, candidate, file_request, file_reply, selection_names, verdict_names) =
                match (name, args["data_type"].as_str()) {
                    ("CHEMIST", Some("purification")) => (
                        "CHEMIST",
                        "CHEMISTRY",
                        "FileQcChemist",
                        "FileQcChemistResult",
                        ["FirstChemist", "NextChemist", "LastChemist"],
                        if success {
                            ["ChemistApproved", "LastChemistApproved"]
                        } else {
                            ["ChemistRejected", "LastChemistRejected"]
                        },
                    ),
                    ("BIOLOGIST", Some("bioassay")) => (
                        "BIOLOGIST",
                        "BIOLOGY",
                        "FileQcBiologist",
                        "FileQcBiologistResult",
                        ["FirstBiologist", "NextBiologist", "LastBiologist"],
                        if success {
                            ["BiologistApproved", "LastBiologistApproved"]
                        } else {
                            ["BiologistRejected", "LastBiologistRejected"]
                        },
                    ),
                    _ => return Err("unassigned QC subject".into()),
                };
            let admitted = frame(
                (&frames, &mut used),
                (
                    role("QC_FILE")?,
                    role("QC_CONTROLLER")?,
                    &[label(file_request)?],
                ),
                (Some(args), 0, call.emitted),
            )?;
            let selected = frame(
                (&frames, &mut used),
                (
                    role("QC_CONTROLLER")?,
                    role("SPEC_SERVICE")?,
                    &labels(&selection_names)?,
                ),
                (Some(args), admitted.received, call.emitted),
            )?;
            let selection = u64::from(selected.logical_label);
            for peer in ["QC_SERVICE", native] {
                frame(
                    (&frames, &mut used),
                    (role("QC_CONTROLLER")?, role(peer)?, &[selection]),
                    (Some(args), admitted.received, call.emitted),
                )?;
            }
            if selection != label(selection_names[2])? {
                // The other scientist's independent work need not consume
                // this notification before the selected scientist's effect.
                let other = frame(
                    (&frames, &mut used),
                    (
                        role("QC_CONTROLLER")?,
                        role(if native == "CHEMIST" {
                            "BIOLOGIST"
                        } else {
                            "CHEMIST"
                        })?,
                        &[selection],
                    ),
                    (Some(args), admitted.received, usize::MAX),
                )?;
                if other.sent >= call.emitted {
                    return Err("QC selection was sent after its effect".into());
                }
            }
            let admitted = frame(
                (&frames, &mut used),
                (
                    role(candidate)?,
                    role("QC_SERVICE")?,
                    &[label("QcAdmission")?],
                ),
                (None, admitted.received, call.emitted),
            )?;
            let packet = admitted.value()?;
            shape(&packet, &["arguments", "materials"], &[])?;
            if packet["arguments"] != *args {
                return Err("QC admission substituted selected arguments".into());
            }
            let report = frame(
                (&frames, &mut used),
                (
                    role(native)?,
                    role("QC_CONTROLLER")?,
                    &[label(if native == "CHEMIST" {
                        "ChemistReport"
                    } else {
                        "BiologyReport"
                    })?],
                ),
                (Some(&packet["materials"]), 0, admitted.received),
            )?;
            if report.received >= admitted.received {
                return Err("QC admission preceded its actual scientific report".into());
            }
            for (resource, message) in [
                ("SPECTROMETER", "QcSpecRequest"),
                ("FUME_HOOD", "QcFumeRequest"),
            ] {
                frame(
                    (&frames, &mut used),
                    (role(resource)?, role("QC_SERVICE")?, &[label(message)?]),
                    (Some(&packet), admitted.received, call.emitted),
                )?;
            }
            for (resource, message) in [
                ("SPECTROMETER", "QcSpecResult"),
                ("FUME_HOOD", "QcFumeResult"),
            ] {
                frame(
                    (&frames, &mut used),
                    (role("QC_SERVICE")?, role(resource)?, &[label(message)?]),
                    (Some(value), ended, usize::MAX),
                )?;
            }
            let verdict = if selection == label(selection_names[2])? {
                verdict_names[1]
            } else {
                verdict_names[0]
            };
            frame(
                (&frames, &mut used),
                (
                    role("QC_SERVICE")?,
                    role("QC_CONTROLLER")?,
                    &[label(verdict)?],
                ),
                (Some(value), ended, usize::MAX),
            )?;
            for peer in ["CHEMIST", "BIOLOGIST", "SPEC_SERVICE", "CENT_SERVICE"] {
                frame(
                    (&frames, &mut used),
                    (role("QC_CONTROLLER")?, role(peer)?, &[label(verdict)?]),
                    (Some(value), ended, usize::MAX),
                )?;
            }
            frame(
                (&frames, &mut used),
                (
                    role("QC_CONTROLLER")?,
                    role("QC_FILE")?,
                    &[label(file_reply)?],
                ),
                (Some(value), ended, usize::MAX),
            )?;
            frame(
                (&frames, &mut used),
                (
                    role(native)?,
                    role(if native == "CHEMIST" {
                        "CHEM_FILE"
                    } else {
                        "BIO_FILE"
                    })?,
                    &[label("FileFeedback")?],
                ),
                (Some(value), ended, usize::MAX),
            )?;
            qc_inputs((&calls, call.emitted), args, &packet["materials"])?
        } else {
            let (file, view, normal, owner, resource, file_request, intents, file_reply) =
                match (actor, tool, name) {
                    ("CHEMIST", "run_synthesis", _) => (
                        "CHEM_FILE",
                        "CHEMISTRY",
                        "CHEMISTRY",
                        "QC_SERVICE",
                        Some("FUME_HOOD"),
                        "FileSynthesis",
                        vec!["Synthesis"],
                        "FileSynthesisResult",
                    ),
                    ("CHEMIST", "run_analysis", "synthesized_compound") => (
                        "CHEM_FILE",
                        "CHEMISTRY",
                        "CHEMISTRY",
                        "SPEC_SERVICE",
                        Some("SPECTROMETER"),
                        "FileAnalysis",
                        vec!["InitialAnalysis", "Reanalysis", "LastReanalysis"],
                        "FileAnalysisResult",
                    ),
                    ("CHEMIST", "run_separation", "compound_purification") => (
                        "CHEM_FILE",
                        "CHEMISTRY",
                        "CHEMISTRY",
                        "CENT_SERVICE",
                        Some("CENTRIFUGE"),
                        "FilePurification",
                        vec!["Purification"],
                        "FilePurificationResult",
                    ),
                    ("BIOLOGIST", "run_separation", "cell_cultures") => (
                        "BIO_FILE",
                        "BIOLOGIST",
                        "BIOLOGIST",
                        "CENT_SERVICE",
                        Some("CENTRIFUGE"),
                        "FileCells",
                        vec!["Preparation"],
                        "FileCellsResult",
                    ),
                    ("BIOLOGIST", "run_analysis", "fluorescence") => (
                        "BIO_FILE",
                        "BIOLOGIST",
                        "BIOLOGIST",
                        "SPEC_SERVICE",
                        Some("SPECTROMETER"),
                        "FileCalibration",
                        vec!["Calibration"],
                        "FileCalibrationResult",
                    ),
                    ("BIOLOGIST", "run_bioassay", _) => (
                        "BIO_FILE",
                        "BIOLOGY",
                        "BIOLOGIST",
                        "CENT_SERVICE",
                        None,
                        "FileBioassay",
                        vec!["BioRequest", "Retest", "LastRetest"],
                        "FileBioassayResult",
                    ),
                    ("ANALYST", "run_analysis", "molecular_detail") => (
                        "ANALYST_FILE",
                        "ANALYST",
                        "ANALYST",
                        "SPEC_SERVICE",
                        Some("SPECTROMETER"),
                        "FileMolecular",
                        vec!["Molecular"],
                        "FileMolecularResult",
                    ),
                    ("ANALYST", "run_separation", "verification") => (
                        "ANALYST_FILE",
                        "ANALYST",
                        "ANALYST",
                        "CENT_SERVICE",
                        Some("CENTRIFUGE"),
                        "FileVerification",
                        vec!["Separation"],
                        "FileVerificationResult",
                    ),
                    ("CHEMIST", "run_analysis", "final_validation") => (
                        "CHEM_FILE",
                        "CHEMIST",
                        "CHEMIST",
                        "SPEC_SERVICE",
                        Some("SPECTROMETER"),
                        "FileFinal",
                        vec!["FinalValidation"],
                        "FileFinalResult",
                    ),
                    _ => return Err("unassigned scientific effect".into()),
                };
            let admitted = frame(
                (&frames, &mut used),
                (role(file)?, role(view)?, &[label(file_request)?]),
                (Some(args), 0, call.emitted),
            )?;
            let intent = frame(
                (&frames, &mut used),
                (role(normal)?, role(owner)?, &labels(&intents)?),
                (None, admitted.received, call.emitted),
            )?;
            let packet = intent.value()?;
            let context = (&calls, call.emitted);
            let (inputs, expected) = match (actor, tool, name) {
                ("CHEMIST", "run_synthesis", _) => (json!({}), args.clone()),
                ("BIOLOGIST", "run_separation", "cell_cultures") => (json!({}), args.clone()),
                ("CHEMIST", "run_analysis", "synthesized_compound") => {
                    let id = known(
                        context,
                        &packet["synthesis"],
                        (
                            "CHEMIST",
                            "run_synthesis",
                            packet["synthesis"]["data"]["compound"]
                                .as_str()
                                .ok_or("synthesis name")?,
                        ),
                    )?;
                    (
                        json!({"synthesis":id}),
                        json!({"arguments":args,"synthesis":packet["synthesis"]}),
                    )
                }
                ("CHEMIST", "run_separation", "compound_purification") => {
                    let id = known(
                        context,
                        &packet["synthesis"],
                        (
                            "CHEMIST",
                            "run_synthesis",
                            packet["synthesis"]["data"]["compound"]
                                .as_str()
                                .ok_or("synthesis name")?,
                        ),
                    )?;
                    let analysis = known(
                        context,
                        &packet["analysis"],
                        ("CHEMIST", "run_analysis", "synthesized_compound"),
                    )?;
                    if packet["analysis"]["data"]["inputs"] != json!({"synthesis":id}) {
                        return Err("purification uses a foreign analysis".into());
                    }
                    (
                        json!({"synthesis":id,"analysis":analysis}),
                        json!({"arguments":args,"synthesis":packet["synthesis"],"analysis":packet["analysis"]}),
                    )
                }
                ("BIOLOGIST", "run_analysis", "fluorescence") => {
                    let id = known(
                        context,
                        &packet["cells"],
                        ("BIOLOGIST", "run_separation", "cell_cultures"),
                    )?;
                    (
                        json!({"prepared_cells":id}),
                        json!({"arguments":args,"cells":packet["cells"]}),
                    )
                }
                ("BIOLOGIST", "run_bioassay", _) => {
                    if packet["materials"]["synthesis"]["data"]["compound"] != name {
                        return Err("trial uses a foreign compound".into());
                    }
                    frame(
                        (&frames, &mut used),
                        (
                            role("BIOLOGIST")?,
                            role("CENT_SERVICE")?,
                            &[label("BioAccess")?],
                        ),
                        // The sender may queue the loan before the service has
                        // received the preceding intent. Both real receives
                        // must still commit before this effect.
                        (None, intent.sent, call.emitted),
                    )?;
                    (
                        biological(context, &packet["materials"])?,
                        json!({"arguments":args,"materials":packet["materials"]}),
                    )
                }
                ("ANALYST", _, _) => (
                    approved(context, &packet["approved"])?,
                    json!({"arguments":args,"approved":packet["approved"]}),
                ),
                ("CHEMIST", "run_analysis", "final_validation") => (
                    quality(context, &packet["quality"])?,
                    json!({"arguments":args,"quality":packet["quality"]}),
                ),
                _ => return Err("missing scientific input relation".into()),
            };
            if packet != expected {
                return Err("instrument intent substituted its admitted request".into());
            }
            if let Some(resource) = resource {
                frame(
                    (&frames, &mut used),
                    (
                        role(resource)?,
                        role(owner)?,
                        &[label(match tool {
                            "run_synthesis" => "SynthesisRequest",
                            _ => {
                                if resource == "SPECTROMETER" {
                                    "SpecRequest"
                                } else {
                                    "CentRequest"
                                }
                            }
                        })?],
                    ),
                    (Some(&packet), intent.received, call.emitted),
                )?;
                frame(
                    (&frames, &mut used),
                    (role(owner)?, role(resource)?, &[label("InstrumentResult")?]),
                    (Some(value), ended, usize::MAX),
                )?;
            }
            let reply_view = if tool == "run_bioassay" {
                "BIOLOGIST"
            } else {
                view
            };
            frame(
                (&frames, &mut used),
                (role(reply_view)?, role(file)?, &[label(file_reply)?]),
                (Some(value), ended, usize::MAX),
            )?;
            if actor == "CHEMIST" && tool == "run_analysis" && name == "synthesized_compound" {
                frame(
                    (&frames, &mut used),
                    (
                        role("SPEC_SERVICE")?,
                        role("QC_SERVICE")?,
                        &[label("AnalysisProof")?],
                    ),
                    (Some(value), ended, usize::MAX),
                )?;
            }
            if actor == "CHEMIST" && tool == "run_separation" {
                let sample = json!({"synthesis":packet["synthesis"],"initial_analysis":packet["analysis"],"purification":value,"analysis":packet["analysis"]});
                for (from, to, message) in [
                    ("CENT_SERVICE", "QC_SERVICE", "PurificationProof"),
                    ("CHEMIST", "BIOLOGIST", "Sample"),
                    ("BIOLOGIST", "BIO_FILE", "FileSample"),
                ] {
                    frame(
                        (&frames, &mut used),
                        (role(from)?, role(to)?, &[label(message)?]),
                        (Some(&sample), ended, usize::MAX),
                    )?;
                }
            }
            if actor == "BIOLOGIST" && tool == "run_analysis" {
                frame(
                    (&frames, &mut used),
                    (
                        role("SPEC_SERVICE")?,
                        role("CENT_SERVICE")?,
                        &[label("CalibrationProof")?],
                    ),
                    (Some(value), ended, usize::MAX),
                )?;
            }
            if tool == "run_bioassay" {
                let family = if u64::from(intent.logical_label) == label("BioRequest")? {
                    "Bio"
                } else if u64::from(intent.logical_label) == label("Retest")? {
                    "Retest"
                } else {
                    "LastRetest"
                };
                let mut materials = packet["materials"].clone();
                materials["bioassay"] = value.clone();
                let proof = json!({"success":success,"result":value,"arguments":args,"materials":materials});
                let verdict = if success { "Passed" } else { "Failed" };
                frame(
                    (&frames, &mut used),
                    (
                        role("BIOLOGY")?,
                        role("BIOLOGIST")?,
                        &[label(&format!("{family}{verdict}"))?],
                    ),
                    (Some(value), ended, usize::MAX),
                )?;
                for (owner, suffix) in [("QC_SERVICE", "Fume"), ("SPEC_SERVICE", "Spec")] {
                    frame(
                        (&frames, &mut used),
                        (
                            role("CENT_SERVICE")?,
                            role(owner)?,
                            &[label(&format!("{family}{suffix}{verdict}"))?],
                        ),
                        (Some(&proof), ended, usize::MAX),
                    )?;
                }
            }
            if actor == "ANALYST" && tool == "run_separation" {
                frame(
                    (&frames, &mut used),
                    (
                        role("CENT_SERVICE")?,
                        role("SPEC_SERVICE")?,
                        &[label("VerificationProof")?],
                    ),
                    (Some(value), ended, usize::MAX),
                )?;
            }
            inputs
        };
        if data["inputs"] != inputs {
            return Err(format!(
                "scientific effect uses stale/foreign inputs: {actor}/{tool}/{name}"
            )
            .into());
        }
        for id in inputs.as_object().ok_or("scientific receipts")?.values() {
            let previous = one(calls.values().filter(|c| {
                c.result.1["success"] == true && c.result.1["data"]["artifact_id"] == *id
            }))?;
            if previous.ended.0 >= call.started.0 {
                return Err("scientific input was incomplete before the effect".into());
            }
        }
        let mut score_args = args.clone();
        if tool == "run_bioassay" && score_args.get("cell_type").is_none() {
            score_args["cell_type"] = json!("");
        }
        let (index, _) = events
            .iter()
            .enumerate()
            .find(|(i, e)| !scored.contains(i) && call.matches_score(e, &score_args))
            .ok_or("missing corresponding scored effect")?;
        if if success {
            data["artifact_id"] != index
        } else {
            !data["artifact_id"].is_null()
        } {
            return Err("scientific artifact identifies another world occurrence".into());
        }
        scored.insert(index);
    }
    // Input refusals have real file/request/reply occurrences, but no domain effect.
    let mut refusals = 0;
    let bio_file = role("BIO_FILE")?;
    let bio_view = role("BIOLOGY")?;
    let bio_request = label("FileBioassay")?;
    let pending = frames
        .values()
        .filter(|f| {
            f.direction(bio_file, bio_view)
                && u64::from(f.logical_label) == bio_request
                && !used.contains(&f.sent)
        })
        .collect::<Vec<_>>();
    for admitted in pending {
        let args = admitted.value()?;
        used.insert(admitted.sent);
        let intent = frame(
            (&frames, &mut used),
            (
                role("BIOLOGIST")?,
                role("CENT_SERVICE")?,
                &labels(&["BioRequest", "Retest", "LastRetest"])?,
            ),
            (None, admitted.received, usize::MAX),
        )?;
        let packet = intent.value()?;
        shape(&packet, &["arguments", "materials"], &[])?;
        if packet["arguments"] != args
            || args["compound"] == packet["materials"]["synthesis"]["data"]["compound"]
        {
            return Err("unassigned bioassay request is not a dynamic input refusal".into());
        }
        let expected_inputs = biological((&calls, intent.received), &packet["materials"])?;
        let family = if u64::from(intent.logical_label) == label("BioRequest")? {
            "Bio"
        } else if u64::from(intent.logical_label) == label("Retest")? {
            "Retest"
        } else {
            "LastRetest"
        };
        let reply = frame(
            (&frames, &mut used),
            (
                role("BIOLOGY")?,
                role("BIOLOGIST")?,
                &[label(&format!("{family}Failed"))?],
            ),
            (None, intent.received, usize::MAX),
        )?;
        let value = reply.value()?;
        if value["tool_name"] != "run_bioassay"
            || value["success"] != false
            || value["data"]["status"] != "invalid_arguments"
            || value["data"]["scientist"] != "BIOLOGIST"
            || value["data"]["compound"] != args["compound"]
            || value["data"]["cell_type"] != args.get("cell_type").cloned().unwrap_or(json!(""))
            || value["data"]["inputs"] != expected_inputs
            || !value["data"]["artifact_id"].is_null()
            || value["data"]["missing_inputs"] != json!([])
            || !value["data"]["error"]
                .as_str()
                .is_some_and(|s| !s.is_empty())
        {
            return Err("input refusal impersonates a physical trial failure or artifact".into());
        }
        frame(
            (&frames, &mut used),
            (
                role("BIOLOGIST")?,
                role("CENT_SERVICE")?,
                &[label("BioAccess")?],
            ),
            (None, intent.sent, reply.sent),
        )?;
        frame(
            (&frames, &mut used),
            (
                role("BIOLOGIST")?,
                role("BIO_FILE")?,
                &[label("FileBioassayResult")?],
            ),
            (Some(&value), reply.sent, usize::MAX),
        )?;
        let mut materials = packet["materials"].clone();
        materials["bioassay"] = value.clone();
        let proof = json!({"success":false,"result":value,"arguments":args,"materials":materials});
        for (owner, suffix) in [("QC_SERVICE", "Fume"), ("SPEC_SERVICE", "Spec")] {
            frame(
                (&frames, &mut used),
                (
                    role("CENT_SERVICE")?,
                    role(owner)?,
                    &[label(&format!("{family}{suffix}Failed"))?],
                ),
                (Some(&proof), reply.sent, usize::MAX),
            )?;
        }
        refusals += 1;
    }
    let final_call = one(calls.values().filter(|c| {
        c.request["agent"] == "CHEMIST"
            && c.tool == "run_analysis"
            && target(c).ok() == Some("final_validation")
    }))?;
    let chemist = role("CHEMIST")?;
    let spec_service = role("SPEC_SERVICE")?;
    let final_label = label("FinalValidation")?;
    let final_intent = one(frames.values().filter(|f| {
        f.direction(chemist, spec_service) && u64::from(f.logical_label) == final_label
    }))?;
    let final_packet = final_intent.value()?;
    let quality_value = &final_packet["quality"];
    let approved_value = json!({"materials":quality_value["materials"],"chemist_qc":quality_value["chemist_qc"],"biologist_qc":quality_value["biologist_qc"]});
    let chemical_value = json!({"materials":materials_for_subject(&quality_value["materials"],"CHEMIST"),
  "chemist_qc":quality_value["chemist_qc"],"biologist_qc":quality_value["biologist_qc"]});
    let qc_complete = ["chemist_qc", "biologist_qc"]
        .into_iter()
        .map(|key| {
            Ok(
                one(calls.values().filter(|c| c.result.1 == &quality_value[key]))?
                    .ended
                    .0,
            )
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .max()
        .ok_or("actual approvals")?;
    let quality_calls = calls
        .values()
        .filter(|c| c.request["agent"] == "ANALYST")
        .collect::<Vec<_>>();
    if quality_calls.len() != 2 {
        return Err("missing or extra actual quality measurement".into());
    }
    let quality_start = quality_calls
        .iter()
        .map(|c| c.emitted)
        .min()
        .ok_or("quality operations")?;
    let quality_end = quality_calls
        .iter()
        .map(|c| c.ended.0)
        .max()
        .ok_or("quality results")?;
    for (from, to, message, value, after, before) in [
        (
            "QC_CONTROLLER",
            "BIOLOGIST",
            "ApprovedResults",
            &chemical_value,
            qc_complete,
            quality_start,
        ),
        (
            "BIOLOGIST",
            "ANALYST",
            "ApprovedBiology",
            &approved_value,
            qc_complete,
            quality_start,
        ),
        (
            "ANALYST",
            "ANALYST_FILE",
            "FileAssay",
            &approved_value,
            qc_complete,
            quality_start,
        ),
        (
            "ANALYST",
            "CHEMIST",
            "Quality",
            quality_value,
            quality_end,
            final_call.emitted,
        ),
        (
            "ANALYST",
            "SPEC_SERVICE",
            "QualityForOwner",
            quality_value,
            quality_end,
            final_call.emitted,
        ),
        (
            "CHEMIST",
            "CHEM_FILE",
            "FileQuality",
            quality_value,
            quality_end,
            final_call.emitted,
        ),
    ] {
        frame(
            (&frames, &mut used),
            (role(from)?, role(to)?, &[label(message)?]),
            (Some(value), after, before),
        )?;
    }
    let business = [
        "CHEMIST",
        "BIOLOGIST",
        "ANALYST",
        "QC_CONTROLLER",
        "SPEC_SERVICE",
        "CENT_SERVICE",
        "QC_SERVICE",
        "SPECTROMETER",
        "CENTRIFUGE",
        "FUME_HOOD",
        "CHEMISTRY",
        "BIOLOGY",
        "CHEM_FILE",
        "BIO_FILE",
        "ANALYST_FILE",
        "QC_FILE",
    ]
    .into_iter()
    .map(role)
    .collect::<Result<BTreeSet<_>>>()?;
    let business_labels = compiled["labels"]
        .as_object()
        .ok_or("business contracts")?
        .iter()
        .filter(|(name, _)| {
            ![
                "SpecLoan",
                "CentLoan",
                "FumeLoan",
                "ChemAccess",
                "BioAccess",
            ]
            .contains(&name.as_str())
        })
        .map(|(_, v)| v.as_u64().ok_or_else(|| "business label".into()))
        .collect::<Result<BTreeSet<_>>>()?;
    for f in frames.values() {
        if business.contains(&f.identity["source"].as_u64().ok_or("frame source")?)
            && business.contains(&f.identity["target"].as_u64().ok_or("frame target")?)
            && business_labels.contains(&u64::from(f.logical_label))
            && !used.contains(&f.sent)
        {
            return Err(format!(
                "unused scientific request/result {} -> {} label {}",
                f.identity["source"], f.identity["target"], f.logical_label
            )
            .into());
        }
    }
    for (from, message) in [
        ("SPEC_SERVICE", "SpecLoan"),
        ("CENT_SERVICE", "CentLoan"),
        ("QC_SERVICE", "FumeLoan"),
        ("QC_CONTROLLER", "ChemAccess"),
        ("QC_CONTROLLER", "BioAccess"),
    ] {
        let from = role(from)?;
        let init = role("INIT")?;
        let message = label(message)?;
        one(frames
            .values()
            .filter(|f| f.direction(from, init) && u64::from(f.logical_label) == message))?;
    }
    Ok(
        json!({"status":"passed","task":"13M","domain_calls":calls.len(),"input_refusals":refusals,
  "violations":0,"simulator_complete":true,"scientific_receipts_match_actual_completed_inputs":true,
  "effect_receives_bound_to_checked_occurrences":true,"actual_instrument_endpoint_returns":3,
  "actual_candidate_retirements":2,"guest_exits":4,"dynamic_protocol":dynamic,"scored_benchmark":false}),
    )
}
pub fn check_mutants(path: &Path, output: &Path) -> Result<(Value, Value)> {
    let evidence = crate::journal::Evidence::open(path)?;
    let baseline = evidence.audit(check)?;
    let (rows, artifact) = (evidence.rows, evidence.projection);
    let dynamic = &baseline["dynamic_protocol"];
    let effect = rows
        .iter()
        .position(|r| {
            r["direction"] == "runtime"
                && r["message"]["type"] == "execute"
                && r["message"]["arguments"]["sample"] == "final_validation"
        })
        .ok_or("actual final effect")?;
    let id = rows[effect]["message"]["id"].clone();
    let ended = rows
        .iter()
        .position(|r| r["direction"] == "domain_end" && r["message"]["id"] == id)
        .ok_or("actual final result")?;
    let score = rows
        .iter()
        .position(|r| r["message"]["type"] == "score")
        .ok_or("world score")?;
    let contract = crate::projection::contracts(&artifact["compiled"])?;
    let final_wires = contract
        .iter()
        .filter(|(_, h)| h[3] == artifact["compiled"]["labels"]["SpecRequest"])
        .map(|(wire, _)| *wire)
        .collect::<BTreeSet<_>>();
    let final_frame = rows
        .iter()
        .position(|r| {
            r["message"]["type"] == "protocol_frame"
                && r["message"]["frame_label"]
                    .as_u64()
                    .is_some_and(|w| final_wires.contains(&(w as u16)))
                && serde_json::from_value::<Vec<u8>>(r["message"]["payload"].clone())
                    .ok()
                    .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
                    .is_some_and(|p| p["arguments"]["sample"] == "final_validation")
        })
        .ok_or("actual final resource request")?;
    let mut reports = Vec::new();
    for name in [
        "changed_final_target",
        "negative_final_result",
        "stale_final_inputs",
        "foreign_approval",
        "missing_resource_request",
        "missing_actual_trial",
        "forged_world_receipt",
        "missing_world_score",
        "missing_qc_file_request",
    ] {
        let mut changed = rows.clone();
        match name {
            "changed_final_target" => {
                changed[effect]["message"]["arguments"]["sample"] = json!("synthesized_compound")
            }
            "negative_final_result" => {
                changed[ended]["message"]["result"]["success"] = json!(false)
            }
            "stale_final_inputs" => {
                changed[ended]["message"]["result"]["data"]["inputs"]["bioassay"] = json!(999)
            }
            "foreign_approval" => {
                let mut packet: Value =
                    serde_json::from_slice(&serde_json::from_value::<Vec<u8>>(
                        changed[final_frame]["message"]["payload"].clone(),
                    )?)?;
                packet["quality"]["chemist_qc"]["data"]["artifact_id"] = json!(999);
                changed[final_frame]["message"]["payload"] = json!(serde_json::to_vec(&packet)?);
            }
            "missing_resource_request" => {
                changed.remove(final_frame);
            }
            "missing_actual_trial" => {
                let at = changed
                    .iter()
                    .position(|r| {
                        r["direction"] == "domain_end" && r["message"]["tool"] == "run_bioassay"
                    })
                    .ok_or("actual trial result")?;
                changed.remove(at);
            }
            "forged_world_receipt" => {
                let event = changed[score]["message"]["events"]
                    .as_array_mut()
                    .ok_or("world events")?
                    .iter_mut()
                    .find(|e| e["args"]["sample"] == "final_validation")
                    .ok_or("scored final")?;
                event["result"]["inputs"]["purification_qc"] = json!(999);
            }
            "missing_world_score" => {
                changed.remove(score);
            }
            "missing_qc_file_request" => {
                let wires = contract
                    .iter()
                    .filter(|(_, h)| h[3] == artifact["compiled"]["labels"]["FileQcChemist"])
                    .map(|(wire, _)| *wire)
                    .collect::<BTreeSet<_>>();
                let at = changed
                    .iter()
                    .rposition(|r| {
                        r["message"]["type"] == "protocol_frame"
                            && r["message"]["frame_label"]
                                .as_u64()
                                .is_some_and(|w| wires.contains(&(w as u16)))
                    })
                    .ok_or("QC request occurrence")?;
                changed.remove(at);
            }
            _ => unreachable!(),
        }
        let error = check(&changed, &artifact, dynamic)
            .err()
            .ok_or_else(|| format!("QC auditor accepted {name}"))?;
        reports.push(json!({"name":name,"rejected":true,"error":error.to_string()}));
    }
    let report = json!({"status":"passed","task":"13M","record_counterexamples":reports,"original_events_sha256":digest(path)?,
  "dynamic_capture_checked_once":true,"runtime_attacks":false,"model_calls":0,"scored_runs":0});
    crate::evidence::save(output, &report)?;
    Ok((baseline, report))
}
