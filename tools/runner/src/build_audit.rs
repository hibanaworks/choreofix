//! Actual compile/link data and unique effect receipts; EPF owns source ordering.
use crate::{
    evidence::{Result, digest, save},
    journal::{check_close_pairs, close_arguments, records},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

fn check(rows: &[Value], artifact: &Value) -> Result<Value> {
    let compiled = &artifact["compiled"];
    let fixture = &rows.first().ok_or("empty build evidence")?["message"];
    if compiled["task"] != "10E" || fixture["task"] != "10E" || fixture["type"] != "fixtures" {
        return Err("shared build task differs".into());
    }
    let actors = fixture["fixtures"].as_array().ok_or("build participants")?;
    if actors.len() != 3
        || actors
            .iter()
            .map(|a| a["agent"].as_str())
            .collect::<BTreeSet<_>>()
            != BTreeSet::from([Some("BUILDER_A"), Some("BUILDER_B"), Some("INTEGRATOR")])
    {
        return Err("shared build participants differ".into());
    }
    let name = |group: &str, key: &str| -> Result<u64> {
        compiled[group][key]
            .as_u64()
            .filter(|n| *n <= 255)
            .ok_or_else(|| format!("compiled {group} {key}").into())
    };
    let service = name("roles", "BUILD_SERVICE")?;
    let core = name("roles", "CORE_LIB")?;
    let types = name("roles", "SHARED_TYPES")?;
    let session = compiled["session"].as_u64().ok_or("build session")?;
    let labels = crate::projection::frames(artifact, rows)?;
    let crate::journal::Journal {
        frames,
        calls,
        score,
    } = crate::journal::read(rows, &labels, actors)?;
    if calls.is_empty()
        || frames.values().any(|f| f.identity["session"] != session)
        || score["complete"] != true
        || score["violations"] != json!([])
    {
        return Err("shared build completion or session differs".into());
    }
    let pairs = [
        (name("roles", "BUILDER_A")?, name("roles", "A_IO")?),
        (name("roles", "BUILDER_B")?, name("roles", "B_IO")?),
        (
            name("roles", "INTEGRATOR")?,
            name("roles", "INTEGRATOR_IO")?,
        ),
    ];
    check_close_pairs(&frames, &pairs)?;
    let mut used = BTreeSet::new();
    let mut closes = BTreeSet::new();
    let mut scored = score["events"]
        .as_array()
        .ok_or("build score effects")?
        .clone();
    let mut objects = BTreeMap::new();
    let mut ordered: Vec<_> = calls.values().collect();
    ordered.sort_by_key(|call| call.emitted);
    let mut links = 0;
    for call in &ordered {
        let agent = call.request["agent"].as_str().ok_or("build effect agent")?;
        let tool = call.tool()?;
        let arguments = &call.request["arguments"];
        let (at, result) = call.result()?;
        let (request, reply, io, module) = match (agent, tool) {
            ("BUILDER_A", "compile_module") => ("CompileA", "", "A_IO", Some("module_a")),
            ("BUILDER_B", "compile_module") => ("CompileB", "", "B_IO", Some("module_b")),
            ("INTEGRATOR", "link_modules") => ("Link", "Linked", "INTEGRATOR_IO", None),
            _ => return Err("unassigned build effect".into()),
        };
        if arguments.as_object().is_none_or(|fields| fields.len() != 1) || result["success"] != true
        {
            return Err("build arguments or result differ".into());
        }
        let index = scored
            .iter()
            .position(|event| {
                event["agent"] == agent
                    && event["tool"] == tool
                    && event["args"] == *arguments
                    && event["success"] == result["success"]
                    && event["result"] == result["data"]
                    && event["violations"] == json!([])
            })
            .ok_or("original build score differs from actual effect")?;
        scored.remove(index);
        let actor = name("roles", agent)?;
        let label = name("labels", request)? as u8;
        let (id, admission) = frames
            .iter()
            .filter(|(_, frame)| {
                frame.direction(actor, service)
                    && frame.logical_label == label
                    && frame.received.is_some_and(|before| before < call.emitted)
            })
            .max_by_key(|(_, frame)| frame.received)
            .ok_or("build effect lacks its actual admitted input")?;
        if admission.value()? != *arguments || !used.insert(*id) {
            return Err("admitted build input altered or reused".into());
        }
        close_arguments(
            &frames,
            &mut closes,
            actor,
            name("roles", io)?,
            admission.sent,
            &format!("service/{tool}"),
            arguments,
        )?;
        let mut requests = Vec::new();
        let mut replies = Vec::new();
        if let Some(module) = module {
            let supplied = arguments["module_name"]
                .as_str()
                .ok_or("compile module")?
                .trim()
                .to_lowercase()
                .replace(' ', "_");
            if supplied != module {
                return Err("compile names an unassigned module".into());
            }
            requests.extend([(core, "CompileCore"), (types, "CompileTypes")]);
            replies.extend([(core, "CoreCompiled"), (types, "TypesCompiled")]);
            let object = &result["data"]["object"];
            let content: Value =
                serde_json::from_str(object["bytes"].as_str().ok_or("actual object bytes")?)?;
            if result["data"]["status"] != "compiled"
                || result["data"]["agent"] != agent
                || result["data"]["module"] != module
                || object["builder"] != agent
                || object["module"] != module
                || !object["id"].is_string()
                || content["module"] != module
                || content["inputs"] != object["inputs"]
            {
                return Err("actual object data differs".into());
            }
            objects.insert(module, object.clone());
            for (report, target) in if module == "module_a" {
                vec![
                    ("BuiltA", name("roles", "INTEGRATOR")?),
                    ("TypesUpdated", name("roles", "BUILDER_B")?),
                ]
            } else {
                vec![("BuiltB", name("roles", "INTEGRATOR")?)]
            } {
                let report_label = name("labels", report)? as u8;
                let reports: Vec<_> = frames
                    .iter()
                    .filter(|(_, frame)| {
                        frame.direction(actor, target)
                            && frame.logical_label == report_label
                            && frame.sent > at
                            && frame.value().ok().as_ref() == Some(result)
                    })
                    .collect();
                // The stale initial B object is a real result, but only the
                // current B object has a BuiltB delivery to the linker.
                if reports.len() != usize::from(module == "module_a" || !reports.is_empty()) {
                    return Err("build result notification missing or duplicated".into());
                }
                for (id, frame) in reports {
                    frame.received()?;
                    if !used.insert(*id) {
                        return Err("build result notification reused".into());
                    }
                }
            }
        } else {
            links += 1;
            let mut modules: Vec<_> = arguments["modules"]
                .as_str()
                .ok_or("link modules")?
                .split(',')
                .map(|s| s.trim().to_lowercase().replace(' ', "_"))
                .collect();
            modules.sort_unstable();
            let expected = json!(objects);
            let data = &result["data"];
            let binary: Value =
                serde_json::from_str(data["binary"]["bytes"].as_str().ok_or("linked bytes")?)?;
            if modules != ["module_a", "module_b"]
                || objects.len() != 2
                || objects["module_a"]["inputs"]["SHARED_TYPES"]
                    != objects["module_b"]["inputs"]["SHARED_TYPES"]
                || data["agent"] != agent
                || data["status"] != "linked"
                || data["issues"] != json!([])
                || data["inputs"] != expected
                || binary != expected
            {
                return Err("linked binary lacks the actual current matching objects".into());
            }
            replies.push((actor, reply));
        }
        for (resource, request) in requests {
            let label = name("labels", request)? as u8;
            let (id, frame) = frames
                .iter()
                .filter(|(_, frame)| {
                    frame.direction(resource, service)
                        && frame.logical_label == label
                        && frame.received.is_some_and(|before| before < call.emitted)
                })
                .max_by_key(|(_, frame)| frame.received)
                .ok_or("compile lacks one actual resource receive")?;
            if frame.value()? != *arguments || !used.insert(*id) {
                return Err("compile resource input altered or reused".into());
            }
        }
        for (target, reply) in replies {
            let label = name("labels", reply)? as u8;
            let (id, frame) = frames
                .iter()
                .filter(|(_, frame)| {
                    frame.direction(service, target)
                        && frame.logical_label == label
                        && frame.sent > at
                })
                .min_by_key(|(_, frame)| frame.sent)
                .ok_or("actual build result reply missing")?;
            if frame.value()? != *result || !used.insert(*id) {
                return Err("actual build reply altered or reused".into());
            }
            frame.received()?;
        }
    }
    let business_labels = [
        "CompileA",
        "CompileB",
        "CompileCore",
        "CompileTypes",
        "CoreCompiled",
        "TypesCompiled",
        "BuiltA",
        "BuiltB",
        "TypesUpdated",
        "Link",
        "Linked",
    ]
    .into_iter()
    .map(|label| name("labels", label).map(|n| n as u8))
    .collect::<Result<BTreeSet<_>>>()?;
    if !scored.is_empty()
        || links != 1
        || objects.len() != 2
        || frames
            .iter()
            .any(|(id, f)| business_labels.contains(&f.logical_label) && !used.contains(id))
    {
        return Err("unmatched build effect, data delivery or result".into());
    }
    for label in ["BuiltA", "BuiltB", "TypesUpdated"] {
        let label = name("labels", label)? as u8;
        if frames
            .values()
            .filter(|frame| frame.logical_label == label)
            .count()
            != 1
        {
            return Err("current object or type update report is missing".into());
        }
    }
    Ok(
        json!({"status":"passed","task":"10E","domain_calls":calls.len(),"simulator_complete":true,
        "violations":0,"matched_effect_frame_ids":used,
        "effect_scope":"unique admitted/WASI/resource receipts, actual objects and current linked inputs"}),
    )
}
pub fn audit(path: &Path) -> Result<Value> {
    let artifact = crate::projection::load(path)?;
    let rows = records(path)?;
    let mut result = check(&rows, &artifact)?;
    result["dynamic_protocol"] = crate::projection::check_dynamic(&artifact, &rows)?;
    result["source_events_sha256"] = json!(digest(path)?);
    result["compiled_projection_sha256"] = json!(digest(
        &path
            .parent()
            .ok_or("evidence directory")?
            .join("projection.json")
    )?);
    result["runtime_binary_sha256"] = artifact["runtime_binary_sha256"].clone();
    Ok(result)
}
pub fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
    let artifact = crate::projection::load(path)?;
    let rows = records(path)?;
    let baseline = check(&rows, &artifact)?;
    let ids = baseline["matched_effect_frame_ids"]
        .as_array()
        .ok_or("build effect frame ids")?
        .iter()
        .map(|id| id.as_u64().ok_or_else(|| "build effect frame id".into()))
        .collect::<Result<Vec<_>>>()?;
    let checked =
        crate::journal::json_frame_mutants(&rows, &ids, |changed| check(changed, &artifact))?;
    fs::create_dir(output)?;
    let result = json!({"status":"passed","baseline":baseline,"checked":checked,
        "scope":"immutable record mutations, not runtime attacks"});
    save(&output.join("report.json"), &result)?;
    Ok(result)
}
