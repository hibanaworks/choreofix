//! Match actual job/report effects to unique request, resource and WASI receipts.
//! This reads immutable observations; EPF alone interprets source execution.
use crate::{
    evidence::{Result, digest, save},
    journal::{Frame, check_close_pairs, close_arguments, records},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

fn take_frame<'a>(
    frames: &'a BTreeMap<u64, Frame>,
    used: &mut BTreeSet<u64>,
    matches: impl Fn(&Frame) -> bool,
) -> Result<(u64, &'a Frame)> {
    let mut found = frames.iter().filter(|(_, f)| matches(f));
    let (&id, frame) = found.next().ok_or("missing manufacturing receipt")?;
    if found.next().is_some() || !used.insert(id) {
        return Err("duplicate or reused manufacturing receipt".into());
    }
    Ok((id, frame))
}
fn check(rows: &[Value], artifact: &Value) -> Result<Value> {
    let compiled = &artifact["compiled"];
    let fixture = &rows.first().ok_or("empty manufacturing evidence")?["message"];
    if compiled["task"] != "11E" || fixture["task"] != "11E" || fixture["type"] != "fixtures" {
        return Err("manufacturing source or fixture differs".into());
    }
    let actors = fixture["fixtures"]
        .as_array()
        .ok_or("manufacturing actors")?;
    if actors
        .iter()
        .map(|a| a["agent"].as_str())
        .collect::<BTreeSet<_>>()
        != BTreeSet::from([
            Some("TECHNICIAN_A"),
            Some("TECHNICIAN_B"),
            Some("SUPERVISOR"),
        ])
        || actors.len() != 3
    {
        return Err("manufacturing participants differ".into());
    }
    let name = |group: &str, key: &str| -> Result<u64> {
        compiled[group][key]
            .as_u64()
            .filter(|v| *v <= 255)
            .ok_or_else(|| format!("compiled {group} {key}").into())
    };
    let supervisor = name("roles", "SUPERVISOR")?;
    let report_service = name("roles", "REPORT_SERVICE")?;
    let labels = crate::projection::frames(artifact, rows)?;
    let crate::journal::Journal {
        frames,
        calls,
        score,
    } = crate::journal::read(rows, &labels, actors)?;
    if calls.len() != 5
        || score["complete"] != true
        || score["violations"] != json!([])
        || frames
            .values()
            .any(|f| f.identity["session"] != compiled["session"])
    {
        return Err("manufacturing completion or effect population differs".into());
    }
    check_close_pairs(
        &frames,
        &[
            (name("roles", "TECHNICIAN_A")?, name("roles", "A_IO")?),
            (name("roles", "TECHNICIAN_B")?, name("roles", "B_IO")?),
            (supervisor, name("roles", "SUPERVISOR_IO")?),
        ],
    )?;
    let mut used = BTreeSet::new();
    let mut attempt_closes = BTreeSet::new();
    let mut effect_closes = BTreeSet::new();
    let mut results = Vec::new();
    let mut intervals = Vec::new();
    let mut done_receipts = Vec::new();
    for (agent, job, machine, io, intent, request, done, query, query_loan, query_return, retry) in [
        (
            "TECHNICIAN_A",
            "lathe_job_a",
            "LATHE",
            "A_IO",
            "LatheIntentA",
            "LatheJobA",
            "LatheDoneA",
            "LATHE_REQUEST_A",
            "LatheRequestLoanA",
            "ReturnLatheRequestA",
            "LatheRetryA",
        ),
        (
            "TECHNICIAN_B",
            "lathe_job_b",
            "LATHE",
            "B_IO",
            "LatheIntentB",
            "LatheJobB",
            "LatheDoneB",
            "LATHE_REQUEST_B",
            "LatheRequestLoanB",
            "ReturnLatheRequestB",
            "LatheRetryB",
        ),
        (
            "TECHNICIAN_A",
            "mill_job_a",
            "MILL",
            "A_IO",
            "MillIntentA",
            "MillJobA",
            "MillDoneA",
            "MILL_REQUEST_A",
            "MillRequestLoanA",
            "ReturnMillRequestA",
            "MillRetryA",
        ),
        (
            "TECHNICIAN_B",
            "mill_job_b",
            "MILL",
            "B_IO",
            "MillIntentB",
            "MillJobB",
            "MillDoneB",
            "MILL_REQUEST_B",
            "MillRequestLoanB",
            "ReturnMillRequestB",
            "MillRetryB",
        ),
    ] {
        let actor = name("roles", agent)?;
        let io = name("roles", io)?;
        let resource = name("roles", machine)?;
        let service = name("roles", &format!("{machine}_SERVICE"))?;
        let query = name("roles", query)?;
        let title = if machine == "LATHE" { "Lathe" } else { "Mill" };
        let matches = calls
            .values()
            .filter(|c| {
                c.request["agent"] == agent
                    && c.request["tool"] == "do_job"
                    && c.request["arguments"]["job_id"] == job
            })
            .collect::<Vec<_>>();
        let [call] = matches.as_slice() else {
            return Err("missing or duplicate actual job effect".into());
        };
        let arguments = json!({"job_id":job});
        let (result_at, result) = call.result()?;
        if call.request["arguments"] != arguments
            || result["tool_name"] != "do_job"
            || result["success"] != true
            || result["data"]
                != json!({"job_id":job,"agent":agent,"machine":machine,"status":"completed"})
        {
            return Err("job request or actual result differs".into());
        }
        let initial_label = name("labels", intent)? as u8;
        let (_, initial) = take_frame(&frames, &mut used, |f| {
            f.direction(actor, service) && f.logical_label == initial_label
        })?;
        if initial.value()? != arguments || initial.received()? >= call.emitted {
            return Err("job intent differs from admitted effect".into());
        }
        let mut initial_close = BTreeSet::new();
        close_arguments(
            &frames,
            &mut initial_close,
            actor,
            io,
            initial.sent,
            "service/do_job",
            &arguments,
        )?;
        attempt_closes.extend(initial_close.iter().copied());
        let request_label = name("labels", request)? as u8;
        let (_, request) = take_frame(&frames, &mut used, |f| {
            f.direction(resource, service) && f.logical_label == request_label
        })?;
        if request.value()? != arguments || request.received()? >= call.emitted {
            return Err("resource job request differs from admitted effect".into());
        }
        let mut final_close = BTreeSet::new();
        close_arguments(
            &frames,
            &mut final_close,
            actor,
            io,
            request.sent,
            "service/do_job",
            &arguments,
        )?;
        if final_close
            .iter()
            .any(|close| !effect_closes.insert(*close))
        {
            return Err("effect reused another job's WASI close".into());
        }
        let result_label = name("labels", &format!("{title}Result"))? as u8;
        let (_, reply) = take_frame(&frames, &mut used, |f| {
            f.direction(service, resource)
                && f.logical_label == result_label
                && f.sent > call.emitted
                && f.value().ok().as_ref() == Some(result)
        })?;
        if reply.sent <= result_at {
            return Err("job reply precedes actual result".into());
        }
        let done_label = name("labels", done)? as u8;
        let (_, done) = take_frame(&frames, &mut used, |f| {
            f.direction(actor, supervisor) && f.logical_label == done_label
        })?;
        if done.value()? != *result || done.sent <= reply.received()? {
            return Err("job completion does not contain its actual result".into());
        }
        done_receipts.push(done.received()?);
        let initial_received = initial.received()?;
        let loan_label = name("labels", &format!("{title}Loan"))? as u8;
        let (_, loan) = take_frame(&frames, &mut used, |f| {
            f.direction(service, actor)
                && f.logical_label == loan_label
                && f.sent > initial_received
        })?;
        if loan.received()? >= request.sent {
            return Err("job effect precedes its workstation loan".into());
        }
        let returned_label = name("labels", &format!("Return{title}"))? as u8;
        let (_, returned) = take_frame(&frames, &mut used, |f| {
            f.direction(actor, service) && f.logical_label == returned_label
        })?;
        if returned.sent <= reply.received()? {
            return Err("workstation returned before actual job result".into());
        }
        returned.received()?;
        let query_loan_label = name("labels", query_loan)? as u8;
        let (_, query_loan) = take_frame(&frames, &mut used, |f| {
            f.direction(service, actor) && f.logical_label == query_loan_label
        })?;
        let ready_label = name("labels", &format!("{title}Ready"))? as u8;
        let (_, ready) = take_frame(&frames, &mut used, |f| {
            f.direction(service, query) && f.logical_label == ready_label
        })?;
        let query_return_label = name("labels", query_return)? as u8;
        let (_, query_return) = take_frame(&frames, &mut used, |f| {
            f.direction(actor, service) && f.logical_label == query_return_label
        })?;
        if !ready.bytes.is_empty()
            || query_loan.sent <= initial.received()?
            || query_loan.sent >= ready.sent
            || query_loan.received()? >= ready.received()?
            || query_return.sent <= ready.received()?
            || query_return.received()? >= loan.sent
        {
            return Err("request continuation or Ready retirement differs".into());
        }
        let busy_label = name("labels", &format!("{title}Busy"))? as u8;
        let retry_label = name("labels", retry)? as u8;
        let mut busies = frames
            .iter()
            .filter(|(_, f)| f.direction(service, query) && f.logical_label == busy_label)
            .collect::<Vec<_>>();
        let mut retries = frames
            .iter()
            .filter(|(_, f)| f.direction(query, service) && f.logical_label == retry_label)
            .collect::<Vec<_>>();
        busies.sort_by_key(|(_, f)| f.sent);
        retries.sort_by_key(|(_, f)| f.sent);
        if (!busies.is_empty() && busies.len() != retries.len() + 1)
            || (busies.is_empty() && !retries.is_empty())
        {
            return Err("missing or extra Busy/retry occurrence".into());
        }
        for (id, busy) in &busies {
            if !busy.bytes.is_empty()
                || busy.sent <= query_loan.sent
                || busy.received()? >= query_return.sent
                || !used.insert(**id)
            {
                return Err("Busy receipt differs or follows query retirement".into());
            }
        }
        if let Some((_, first_busy)) = busies.first()
            && (first_busy.sent >= ready.sent || initial_close == final_close)
        {
            return Err("Busy request reused as the eventual effect's WASI close".into());
        }
        for (index, (id, retry)) in retries.iter().enumerate() {
            if retry.value()? != arguments
                || retry.sent <= busies[index].1.received()?
                || retry.received()? >= busies[index + 1].1.sent
                || !used.insert(**id)
            {
                return Err("retry does not answer its actual Busy occurrence".into());
            }
            let mut close = BTreeSet::new();
            close_arguments(
                &frames,
                &mut close,
                actor,
                io,
                retry.sent,
                "service/do_job",
                &arguments,
            )?;
            if close
                .iter()
                .any(|c| !attempt_closes.insert(*c) || final_close.contains(c))
            {
                return Err("retry reused a previous or successful WASI close".into());
            }
        }
        intervals.push((machine, agent, call.started()?.1, call.ended()?.1));
        results.push(result.clone());
    }
    let report_calls = calls
        .values()
        .filter(|c| c.request["agent"] == "SUPERVISOR" && c.request["tool"] == "collect_reports")
        .collect::<Vec<_>>();
    let [report_call] = report_calls.as_slice() else {
        return Err("missing or duplicate actual report effect".into());
    };
    if report_call.request["arguments"] != json!({}) {
        return Err("collect_reports requires its original empty arguments".into());
    }
    let completed_label = name("labels", "CompletedJobs")? as u8;
    let (_, completed) = take_frame(&frames, &mut used, |f| {
        f.direction(supervisor, report_service) && f.logical_label == completed_label
    })?;
    let mut supplied = completed
        .value()?
        .as_array()
        .cloned()
        .ok_or("missing actual four-job results")?;
    supplied.sort_by_key(|v| v["data"]["job_id"].as_str().unwrap_or("").to_owned());
    results.sort_by_key(|v| v["data"]["job_id"].as_str().unwrap().to_owned());
    if supplied != results
        || done_receipts.iter().any(|at| *at >= completed.sent)
        || completed.received()? >= report_call.emitted
    {
        return Err("report does not consume all four actual completion receipts".into());
    }
    let report_label = name("labels", "CollectReports")? as u8;
    let (_, report_request) = take_frame(&frames, &mut used, |f| {
        f.direction(supervisor, report_service) && f.logical_label == report_label
    })?;
    if report_request.value()? != json!({})
        || report_request.sent <= completed.sent
        || report_request.received()? >= report_call.emitted
    {
        return Err("actual report request differs".into());
    }
    close_arguments(
        &frames,
        &mut effect_closes,
        supervisor,
        name("roles", "SUPERVISOR_IO")?,
        completed.sent,
        "service/collect_reports",
        &json!({}),
    )?;
    let (result_at, actual_report) = report_call.result()?;
    let mut expected_jobs = results
        .iter()
        .map(|r| r["data"].clone())
        .collect::<Vec<_>>();
    expected_jobs.sort_by_key(|v| v["job_id"].as_str().unwrap().to_owned());
    let mut actual_jobs = actual_report["data"]["jobs"]
        .as_array()
        .cloned()
        .ok_or("report job contents")?;
    actual_jobs.sort_by_key(|v| v["job_id"].as_str().unwrap_or("").to_owned());
    let report_started = report_call.started()?.1;
    if actual_report["tool_name"] != "collect_reports"
        || actual_report["success"] != true
        || actual_report["data"]["agent"] != "SUPERVISOR"
        || actual_report["data"]["scope"] != "all_jobs"
        || actual_report["data"]["status"] != "collected"
        || actual_jobs != expected_jobs
        || intervals.iter().any(|(_, _, _, end)| *end > report_started)
    {
        return Err("actual supervisor report differs from completed job data".into());
    }
    let reply_label = name("labels", "Reports")? as u8;
    let (_, reply) = take_frame(&frames, &mut used, |f| {
        f.direction(report_service, supervisor) && f.logical_label == reply_label
    })?;
    if reply.value()? != *actual_report || reply.sent <= result_at {
        return Err("supervisor reply differs from the actual report result".into());
    }
    reply.received()?;
    let mut overlap = 0;
    for (i, (machine, agent, start, end)) in intervals.iter().enumerate() {
        for (other_machine, other_agent, other_start, other_end) in &intervals[i + 1..] {
            if start < other_end && other_start < end {
                if machine == other_machine || agent == other_agent {
                    return Err("actual workstation or technician effects overlap illegally".into());
                }
                overlap += 1;
            }
        }
    }
    let mut scored = score["events"]
        .as_array()
        .cloned()
        .ok_or("original manufacturing score effects")?;
    for call in calls.values() {
        let (_, result) = call.result()?;
        let arguments = if call.tool()? == "collect_reports" {
            json!({"scope":"all_jobs"})
        } else {
            call.request["arguments"].clone()
        };
        let index = scored
            .iter()
            .position(|e| {
                e["agent"] == call.request["agent"]
                    && e["tool"] == call.request["tool"]
                    && e["args"] == arguments
                    && e["success"] == result["success"]
                    && e["result"] == result["data"]
                    && e["violations"] == json!([])
            })
            .ok_or("original score does not match the actual manufacturing effect")?;
        scored.remove(index);
    }
    let business = compiled["labels"]
        .as_object()
        .ok_or("manufacturing message contracts")?
        .values()
        .map(|v| {
            v.as_u64()
                .and_then(|n| u8::try_from(n).ok())
                .ok_or("manufacturing label")
        })
        .collect::<std::result::Result<BTreeSet<_>, _>>()?;
    if !scored.is_empty()
        || frames
            .iter()
            .any(|(id, f)| business.contains(&f.logical_label) && !used.contains(id))
    {
        return Err("unmatched manufacturing effect or message occurrence".into());
    }
    let json_ids = used
        .iter()
        .copied()
        .filter(|id| frames[id].value().is_ok_and(|v| v.is_object()))
        .collect::<Vec<_>>();
    Ok(
        json!({"status":"passed","task":"11E","domain_calls":calls.len(),"simulator_complete":true,"violations":0,"actual_jobs":4,"actual_reports":1,"distinct_machine_overlap_pairs":overlap,"matched_business_frame_ids":used,"matched_json_frame_ids":json_ids,"effect_scope":"unique actual WASI/query/resource/response/report occurrences, content equality, actual resource exclusion and original score"}),
    )
}

pub fn audit(path: &Path) -> Result<Value> {
    let artifact = crate::projection::load(path)?;
    let rows = records(path)?;
    let mut result = check(&rows, &artifact)?;
    result["dynamic_protocol"] = crate::projection::check_dynamic(&artifact, &rows)?;
    result["source_events_sha256"] = json!(digest(path)?);
    result["runtime_binary_sha256"] = artifact["runtime_binary_sha256"].clone();
    result["compiled_projection_sha256"] = json!(digest(
        &path
            .parent()
            .ok_or("manufacturing event directory")?
            .join("projection.json"),
    )?);
    Ok(result)
}
pub fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
    let artifact = crate::projection::load(path)?;
    let rows = records(path)?;
    let baseline = check(&rows, &artifact)?;
    let ids = baseline["matched_json_frame_ids"]
        .as_array()
        .ok_or("manufacturing JSON ids")?
        .iter()
        .map(|v| v.as_u64().ok_or_else(|| "manufacturing frame id".into()))
        .collect::<Result<Vec<_>>>()?;
    let mut checked =
        crate::journal::json_frame_mutants(&rows, &ids, |changed| check(changed, &artifact))?;
    let first = baseline["matched_business_frame_ids"]
        .as_array()
        .and_then(|v| v.first())
        .and_then(Value::as_u64)
        .ok_or("manufacturing business frame")?;
    for field in ["frame_label", "lane"] {
        let mut changed = rows.clone();
        let mut count = 0;
        for row in &mut changed {
            let m = &mut row["message"];
            if m["type"] == "protocol_frame" && m["frame_id"] == first {
                m[field] = json!(
                    m[field].as_u64().ok_or("wire field")?
                        ^ if field == "lane" { 255 } else { 65535 }
                );
                count += 1;
            }
        }
        if count < 2 || check(&changed, &artifact).is_ok() {
            return Err("coherently altered manufacturing wire header was accepted".into());
        }
        checked.push(
            json!({"kind":format!("coherent_wrong_{field}"),"frame_id":first,"status":"rejected"}),
        );
    }
    fs::create_dir(output)?;
    let result = json!({"status":"passed","baseline":baseline,"checked":checked,"scope":"immutable record mutations, not runtime attacks"});
    save(&output.join("report.json"), &result)?;
    Ok(result)
}
