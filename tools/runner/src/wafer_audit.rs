//! Connect each 15E effect to the guest request, instrument and returned result of this run.
use crate::{evidence::Result, journal};
use serde_json::{Value, json};
use std::collections::BTreeSet;

pub(crate) fn check(journal: crate::journal::Journal<'_>) -> Result<Value> {
    let artifact = journal.projection;
    let roles = &artifact["compiled"]["roles"];
    let role = |name: &str| crate::projection::named_id(roles, name).map(u64::from);
    let label = |name: &str| crate::projection::named_id(&artifact["compiled"]["labels"], name);
    let pairs = [
        ("ENGINEER", "WAFER"),
        ("DEP", "DEP_INBOX"),
        ("ETCH", "ETCH_INBOX"),
        ("LITHO", "LITHO_INBOX"),
        ("MEASURER", "MEASURE_FILE"),
        ("CONTROL", "CONTROL_FILE"),
    ]
    .into_iter()
    .map(|(a, b)| Ok((role(a)?, role(b)?)))
    .collect::<Result<Vec<_>>>()?;
    journal::check_close_pairs(&journal.frames, &pairs)?;
    let events = journal.scored_events;
    if !journal.simulator_complete {
        return Err("incomplete wafer score or unmatched effects".into());
    }
    let mut used = BTreeSet::new();
    let mut scores = BTreeSet::new();
    let mut orders = journal.calls.values().collect::<Vec<_>>();
    orders.sort_by_key(|c| c.emitted);
    let mut evidence = Vec::new();
    let mut physical_releases = Vec::new();
    for call in orders {
        let args = &call.request["arguments"];
        let agent = call.request["agent"].as_str().ok_or("wafer agent")?;
        let tool = call.tool;
        let field = match tool {
            "check_monitoring" => "check_type",
            "run_metrology_measurement" => "measurement",
            "run_deposition" | "run_etch_process" | "run_stepper_exposure" => "process",
            _ => return Err("unexpected wafer tool".into()),
        };
        let (returned, result) = call.result;
        let name = result["data"][field]
            .as_str()
            .ok_or("wafer result operation")?;
        if result["data"]["engineer"] != agent
            || args[field].as_str().is_none_or(|s| {
                s.trim()
                    .to_lowercase()
                    .replace([' ', '-'], "_")
                    .trim_matches('_')
                    != name
            })
        {
            return Err("wrong wafer target, agent or operation".into());
        }
        let (file, stage, reply_from, request, reply) = match (agent, tool, name) {
            ("LITHOGRAPHER", "run_stepper_exposure", "photo_exposure") => (
                "LITHO_INBOX",
                "STAGE",
                "STEPPER",
                label("Expose")?,
                label("Exposed")?,
            ),
            ("LITHOGRAPHER", "run_etch_process", "pattern_transfer_etch") => (
                "LITHO_INBOX",
                "PATTERN_WAFER",
                "PLASMA_INPUT",
                label("PatternEtch")?,
                label("PatternResult")?,
            ),
            ("ETCHER", "run_etch_process", "reactive_ion_etch") => (
                "ETCH_INBOX",
                "ETCH_CHAMBER",
                "PLASMA_INPUT",
                label("ReactiveEtch")?,
                label("EtchResult")?,
            ),
            ("DEPOSITOR", "run_deposition", "cvd_growth") => (
                "DEP_INBOX",
                "CVD_STAGE",
                "DEPOSITION_CHAMBER",
                label("Growth")?,
                label("Grown")?,
            ),
            ("LITHOGRAPHER", "check_monitoring", "exposure_dose") => (
                "LITHO_INBOX",
                "STAGE",
                "MONITOR_INPUT",
                label("MonitorRequest_EXPOSURE_DOSE")?,
                label("MonitorReading_EXPOSURE_DOSE")?,
            ),
            ("LITHOGRAPHER", "check_monitoring", "etch_endpoint") => (
                "LITHO_INBOX",
                "PATTERN_WAFER",
                "MONITOR_INPUT",
                label("etch_endpoint")?,
                label("etch_endpoint")?,
            ),
            ("ETCHER", "check_monitoring", "etch_rate") => (
                "ETCH_INBOX",
                "ETCH_CHAMBER",
                "MONITOR_INPUT",
                label("etch_rate")?,
                label("etch_rate")?,
            ),
            ("DEPOSITOR", "check_monitoring", "deposition_rate") => (
                "DEP_INBOX",
                "CVD_STAGE",
                "MONITOR_INPUT",
                label("Rate")?,
                label("RateResult")?,
            ),
            ("PROCESS_CONTROLLER", "check_monitoring", "cross_reference") => (
                "CONTROL_FILE",
                "MONITOR",
                "MONITOR_INPUT",
                label("CrossReference")?,
                label("CrossReferenceResult")?,
            ),
            ("INTEGRATION_ENGINEER", "run_etch_process", "etch_uniformity_check") => (
                "WAFER",
                "ETCH_CHAMBER",
                "ETCH_CHAMBER",
                label("Uniformity")?,
                label("UniformityResult")?,
            ),
            ("INTEGRATION_ENGINEER", "run_deposition", "film_integrity_check") => (
                "WAFER",
                "DEPOSITION_CHAMBER",
                "DEPOSITION_CHAMBER",
                label("Film")?,
                label("FilmResult")?,
            ),
            ("INTEGRATION_ENGINEER", "run_stepper_exposure", "overlay_alignment") => (
                "WAFER",
                "STAGE",
                "STEPPER",
                label("Mount")?,
                label("Mounted")?,
            ),
            ("INTEGRATION_ENGINEER", "run_metrology_measurement", "overlay_alignment") => (
                "WAFER",
                "METROLOGY",
                "STEPPER",
                label("Measure")?,
                label("Measured")?,
            ),
            ("LITHOGRAPHER", "run_metrology_measurement", "alignment_check") => (
                "LITHO_INBOX",
                "METROLOGY",
                "METROLOGY",
                label("alignment_check")?,
                label("alignment_check")?,
            ),
            ("DEPOSITOR", "run_metrology_measurement", "thickness_verification") => (
                "DEP_INBOX",
                "METROLOGY",
                "METROLOGY",
                label("thickness_verification")?,
                label("thickness_verification")?,
            ),
            ("METROLOGY_ENGINEER", "run_metrology_measurement", "cd_measurement") => (
                "MEASURE_FILE",
                "METROLOGY",
                "METROLOGY",
                label("cd")?,
                label("cd")?,
            ),
            ("METROLOGY_ENGINEER", "run_metrology_measurement", "thickness_measurement") => (
                "MEASURE_FILE",
                "METROLOGY",
                "METROLOGY",
                label("thickness")?,
                label("thickness")?,
            ),
            ("METROLOGY_ENGINEER", "run_metrology_measurement", "profile_measurement") => (
                "MEASURE_FILE",
                "METROLOGY",
                "METROLOGY",
                label("profile")?,
                label("profile")?,
            ),
            _ => return Err(format!("unexpected {agent} {tool} {name}").into()),
        };
        let mut take = |from: &str,
                        to: &str,
                        label: u8,
                        value: &Value,
                        earliest: usize,
                        latest: usize|
         -> Result<(u64, usize)> {
            let (a, b) = (role(from)?, role(to)?);
            let (&id, f) = journal
                .frames
                .iter()
                .filter(|(id, f)| {
                    !used.contains(*id) && f.direction(a, b) && f.logical_label == label
                })
                .min_by_key(|(_, f)| f.sent)
                .ok_or_else(|| format!("missing {name} {from}->{to} ({label})"))?;
            let at = f.received;
            if f.sent < earliest || at > latest || f.value()? != *value {
                return Err(format!("wrong occurrence or payload: {name} {from}->{to} ({label}), send {}, receive {at}, allowed {earliest}..{latest}",f.sent).into());
            }
            used.insert(id);
            Ok((id, at))
        };
        let (input, admitted) = take(file, stage, request, args, 0, call.emitted)?;
        let mut instrument = Vec::new();
        let ok = result["success"]
            .as_bool()
            .ok_or("instrument result status")?;
        if matches!(tool, "check_monitoring" | "run_etch_process") {
            let sensor = tool == "check_monitoring";
            let (station, handle, power, permit) = if sensor {
                (
                    "MONITOR_INPUT",
                    "METER",
                    "SENSOR",
                    label("SensorPermit_MONITOR")?,
                )
            } else {
                (
                    "PLASMA_INPUT",
                    "RIE",
                    "ETCH_POWER",
                    label("ChamberPermit_ETCH_CHAMBER")?,
                )
            };
            let (sample, at) = take(stage, station, request, args, admitted, call.emitted)?;
            let (requested, at) = take(handle, power, request, args, at, call.emitted)?;
            let (granted, _) = take(power, handle, permit, args, at, call.emitted)?;
            let end_label = match (sensor, name, ok) {
                (true, _, _) => label("SampleResult")?,
                (false, "etch_uniformity_check", _) => label("UniformityResult")?,
                (false, _, true) => label("LoadPassed")?,
                (false, _, false) => label("LoadFailed")?,
            };
            let (end, _) = take(handle, power, end_label, result, returned, usize::MAX)?;
            instrument.extend([sample, requested, granted, end]);
            let confirmation = match (name, ok) {
                ("exposure_dose", true) => Some(label("DosePassed")?),
                ("exposure_dose", false) => Some(label("DoseFailed")?),
                ("etch_endpoint", true) => Some(label("EndpointPassed")?),
                ("etch_endpoint", false) => Some(label("EndpointFailed")?),
                ("etch_rate", true) => Some(label("EtchRatePassed")?),
                ("etch_rate", false) => Some(label("EtchRateFailed")?),
                ("deposition_rate", true) => Some(label("RatePassed")?),
                ("deposition_rate", false) => Some(label("RateFailed")?),
                ("cross_reference", true) => Some(label("CrossReferencePassed")?),
                ("cross_reference", false) => Some(label("CrossReferenceFailed")?),
                ("pattern_transfer_etch", true) => Some(label("PatternLoaded")?),
                ("pattern_transfer_etch", false) => Some(label("PatternFailed")?),
                ("reactive_ion_etch", true) => Some(label("EtchLoaded")?),
                ("reactive_ion_etch", false) => Some(label("EtchFailed")?),
                _ => None,
            };
            if let Some(confirmed) = confirmation {
                let (frame, _) = take(stage, station, confirmed, result, returned, usize::MAX)?;
                instrument.push(frame);
                if ok && matches!(name, "etch_endpoint" | "etch_rate") {
                    let (reported, at) = take(
                        stage,
                        "PLASMA_INPUT",
                        confirmed,
                        result,
                        returned,
                        usize::MAX,
                    )?;
                    let (release, _) =
                        take("RIE", "ETCH_POWER", confirmed, result, at, usize::MAX)?;
                    instrument.extend([reported, release]);
                    physical_releases.push((release, result["data"]["inputs"].clone(), name));
                }
            }
        } else if tool == "run_metrology_measurement" {
            let (sample, _) = take("PROBE", "READOUT", request, args, admitted, call.emitted)?;
            let (response, _) = take(
                "READOUT",
                "PROBE",
                label("SampleResult")?,
                result,
                returned,
                usize::MAX,
            )?;
            instrument.extend([sample, response]);
        }
        let (response, _) = take(reply_from, file, reply, result, returned, usize::MAX)?;
        let score_index = events
            .iter()
            .enumerate()
            .position(|(i, e)| !scores.contains(&i) && call.matches_score(e, args))
            .ok_or("effect missing from common wafer score")?;
        scores.insert(score_index);
        evidence.push(json!({"agent":agent,"tool":tool,"operation":name,"input_frame":input,"result_frame":response,"instrument_frames":instrument,"success":ok,"receipt":result["data"]["receipt"],"score_index":score_index}));
    }
    for (release, inputs, monitor) in physical_releases {
        let load = if monitor == "etch_endpoint" {
            "pattern_transfer_etch"
        } else {
            "reactive_ion_etch"
        };
        let effect = journal::one(evidence.iter().filter(|e| {
            e["success"] == true && e["operation"] == load && inputs == json!([e["receipt"]])
        }))?;
        let ids = effect["instrument_frames"]
            .as_array()
            .ok_or("load receipts")?;
        let issued = journal.frames[&ids[1].as_u64().ok_or("load request")?].sent;
        let released = journal.frames[&release].received;
        let load_labels = [
            label("reactive_ion_etch")?,
            label("pattern_etch")?,
            label("Uniformity")?,
        ];
        if released <= issued
            || journal.frames.values().any(|f| {
                f.direction(
                    roles["RIE"].as_u64().unwrap(),
                    roles["ETCH_POWER"].as_u64().unwrap(),
                ) && load_labels.contains(&f.logical_label)
                    && f.sent > issued
                    && f.received < released
            })
        {
            return Err("etch chamber reused before its measured wafer was released".into());
        }
    }
    for (owner, peer) in [
        ("METER", "SENSOR"),
        ("RIE", "ETCH_POWER"),
        ("PROBE", "READOUT"),
    ] {
        let (a, b) = (role(owner)?, role(peer)?);
        if journal
            .frames
            .iter()
            .any(|(id, f)| (f.direction(a, b) || f.direction(b, a)) && !used.contains(id))
        {
            return Err(format!("unused instrument operation for {owner}").into());
        }
    }
    let frame_result = |effect: &Value| -> Result<(usize, Value)> {
        let f = &journal.frames[&effect["result_frame"]
            .as_u64()
            .ok_or("effect result frame")?];
        Ok((f.received, f.value()?))
    };
    let notice =
        |from: &str, to: &str, logical: Option<u8>| -> Result<(usize, usize, Value)> {
            let (a, b) = (role(from)?, role(to)?);
            let frame =
                journal::one(journal.frames.values().filter(|f| {
                    f.direction(a, b) && logical.is_none_or(|l| l == f.logical_label)
                }))?;
            Ok((frame.sent, frame.received, frame.value()?))
        };
    let successful = |tool: &str, name: &str| -> Result<&Value> {
        evidence
            .iter()
            .filter(|e| e["success"] == true && e["tool"] == tool && e["operation"] == name)
            .max_by_key(|e| journal.frames[&e["result_frame"].as_u64().unwrap()].sent)
            .ok_or_else(|| format!("missing successful {name}").into())
    };
    let validation = successful("check_monitoring", "cross_reference")?;
    let validation_input = &journal.frames[&validation["input_frame"]
        .as_u64()
        .ok_or("cross-reference frame")?];
    let (validated, actual) = frame_result(validation)?;
    let mut measurements = Vec::new();
    for (file, measure, process_tool, process, check) in [
        (
            "LITHO_INBOX",
            "cd_measurement",
            "run_etch_process",
            "pattern_transfer_etch",
            "etch_endpoint",
        ),
        (
            "DEP_INBOX",
            "thickness_measurement",
            "run_deposition",
            "cvd_growth",
            "deposition_rate",
        ),
        (
            "ETCH_INBOX",
            "profile_measurement",
            "run_etch_process",
            "reactive_ion_etch",
            "etch_rate",
        ),
    ] {
        let material = successful(process_tool, process)?;
        let sample = successful("check_monitoring", check)?;
        let receipts = json!([material["receipt"], sample["receipt"]]);
        let (sent, available, ready) = notice(file, "MEASURE_FILE", Some(role(file)?.try_into()?))?;
        if ready["receipts"] != receipts
            || sent <= frame_result(material)?.0.max(frame_result(sample)?.0)
        {
            return Err(
                format!("ready message does not identify completed {process} and {check}").into(),
            );
        }
        let dimension = successful("run_metrology_measurement", measure)?;
        let request =
            &journal.frames[&dimension["input_frame"].as_u64().ok_or("dimension input")?];
        let (measured, reading) = frame_result(dimension)?;
        let (reported, delivered, report) =
            notice("MEASURE_FILE", "CONTROL_FILE", Some(request.logical_label))?;
        if available >= request.sent
            || reading["data"]["inputs"] != receipts
            || measured >= reported
            || report != reading
            || delivered >= validation_input.sent
        {
            return Err(format!("wrong wafer report for {measure}").into());
        }
        measurements.push(dimension["receipt"].clone());
    }
    if actual["data"]["inputs"] != json!(measurements) {
        return Err("cross-reference checked a different set of measurements".into());
    }
    let mut inspections = serde_json::Map::new();
    let mut inspected = 0usize;
    for (name, tool) in [
        ("film_integrity_check", "run_deposition"),
        ("etch_uniformity_check", "run_etch_process"),
    ] {
        let e = successful(tool, name)?;
        inspected = inspected.max(frame_result(e)?.0);
        inspections.insert(name.to_owned(), e["receipt"].clone());
    }
    let mut disposition = None;
    for inbox in ["LITHO_INBOX", "DEP_INBOX", "ETCH_INBOX", "WAFER"] {
        let (sent, _, confirmed) = notice("CONTROL_FILE", inbox, None)?;
        if validated >= sent || confirmed != actual {
            return Err(format!("incorrect validation for {inbox}").into());
        }
        if inbox != "WAFER" {
            let (sent, _, decision) = notice("WAFER", inbox, Some(role(inbox)?.try_into()?))?;
            if inspected >= sent
                || decision["inspection"] != json!(inspections)
                || decision["lot_disposition"]
                    .as_str()
                    .is_none_or(|value| value.trim().is_empty())
                || disposition.as_ref().is_some_and(|first| first != &decision)
            {
                return Err(
                    "lot notification disagrees with inspections or other recipients".into(),
                );
            }
            disposition = Some(decision);
        }
    }
    Ok(
        json!({"status":"passed","task":"15E","simulator_complete":true,"violations":0,
        "domain_calls":evidence.len(),"effects":evidence,"captured_records":journal.execution["captured_records"],
        }),
    )
}
