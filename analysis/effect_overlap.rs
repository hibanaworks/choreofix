//! Cross-participant overlap in saved, source-bound simulator executions.
use super::{Result, digest};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{BufRead, BufReader, Write},
    path::Path,
};

struct Interval {
    id: u64,
    agent: String,
    tool: String,
    arguments: Option<Value>,
    success: Option<bool>,
    start: u64,
    end: u64,
}

fn overlaps(calls: &[Interval]) -> (Vec<[u64; 2]>, usize) {
    let mut pairs = Vec::new();
    let mut peak = 0;
    for (index, a) in calls.iter().enumerate() {
        let mut active = BTreeSet::new();
        for b in &calls[..=index] {
            if b.start <= a.start && a.start < b.end {
                active.insert(&b.agent);
            }
        }
        peak = peak.max(active.len());
        for b in &calls[index + 1..] {
            if b.start >= a.end {
                break;
            }
            if b.start < b.end && a.agent != b.agent {
                pairs.push([a.id, b.id]);
            }
        }
    }
    (pairs, peak)
}

fn measure(run: &Value) -> Result<(Value, usize, usize)> {
    let directory = Path::new(run["directory"].as_str().ok_or("missing run directory")?);
    let audit = Path::new(run["audit"].as_str().ok_or("missing audit path")?);
    if digest(audit)? != run["audit_sha256"]
        || digest(&directory.join("result.json"))? != run["result_sha256"]
    {
        return Err(format!(
            "{}: selected audit or result hash differs",
            directory.display()
        )
        .into());
    }
    let mut starts = BTreeMap::new();
    let mut ends = BTreeMap::new();
    let mut seen = BTreeSet::new();
    let mut intervals = Vec::new();
    let mut reader = BufReader::new(File::open(directory.join("events.jsonl"))?);
    let mut hash = Sha256::new();
    let mut line = Vec::new();
    while reader.read_until(b'\n', &mut line)? != 0 {
        hash.update(&line);
        let row: Value = serde_json::from_slice(&line)?;
        line.clear();
        let msg = &row["message"];
        match row["direction"].as_str().ok_or("event direction")? {
            "domain_start" | "domain_end" => {
                let id = msg["id"].as_u64().ok_or("domain id")?;
                let agent = msg["agent"].as_str().ok_or("domain participant")?;
                let tool = msg["tool"].as_str().ok_or("domain tool")?;
                if row["direction"] == "domain_start" {
                    let arguments = msg.get("arguments").cloned();
                    if msg["type"] != "execute"
                        || starts
                            .insert(id, (agent.to_owned(), tool.to_owned(), arguments))
                            .is_some()
                    {
                        return Err(format!("invalid or duplicate domain start {id}").into());
                    }
                } else if !starts
                    .get(&id)
                    .is_some_and(|(actor, name, _)| actor == agent && name == tool)
                    || ends
                        .insert(
                            id,
                            msg.get("result").ok_or("missing domain result")?.clone(),
                        )
                        .is_some()
                {
                    return Err(format!("unmatched or duplicate domain end {id}").into());
                }
            }
            "simulator" if msg["type"] == "simulator_result" => {
                let id = msg["id"].as_u64().ok_or("simulator id")?;
                if !seen.insert(id)
                    || ends.remove(&id).as_ref()
                        != Some(msg.get("result").ok_or("missing simulator result")?)
                {
                    return Err(format!("unmatched or duplicate simulator reply {id}").into());
                }
                let start = msg["started_ns"].as_u64().ok_or("interval start")?;
                let end = msg["finished_ns"].as_u64().ok_or("interval end")?;
                if start > end || msg["clock"] != "runner monotonic elapsed" {
                    return Err(format!("invalid simulator clock for {id}").into());
                }
                let (agent, tool, arguments) = starts.get(&id).ok_or("missing request")?;
                intervals.push(Interval {
                    id,
                    agent: agent.clone(),
                    tool: tool.clone(),
                    arguments: arguments.clone(),
                    success: msg["result"]["success"].as_bool(),
                    start,
                    end,
                });
            }
            _ => {}
        }
    }
    if format!("{:x}", hash.finalize()) != run["events_sha256"]
        || intervals.is_empty()
        || starts.len() != intervals.len()
        || !ends.is_empty()
        || Some(intervals.len() as u64) != run["effects"].as_u64()
    {
        return Err(format!("{}: incomplete or changed execution", directory.display()).into());
    }
    intervals.sort_by_key(|c| (c.start, c.id));
    let (pairs, peak) = overlaps(&intervals);
    let mut negative_outcomes = Vec::new();
    for (index, call) in intervals
        .iter()
        .enumerate()
        .filter(|(_, call)| call.success == Some(false))
    {
        let next = intervals[index + 1..].iter().find(|other| {
            other.start > call.end && other.agent == call.agent && other.tool == call.tool
        });
        negative_outcomes.push(json!({
            "id":call.id, "agent":call.agent, "tool":call.tool,
            "next_same_operation":next.map(|other|json!({
                "id":other.id, "success":other.success,
                "same_arguments":other.arguments.as_ref().zip(call.arguments.as_ref()).map(|(a,b)|a == b)
            }))
        }));
    }
    Ok((
        json!({
            "task":run["task"], "depth":run["depth"], "directory":directory, "events_sha256":run["events_sha256"],
            "audit_sha256":run["audit_sha256"], "result_sha256":run["result_sha256"],
            "intervals":intervals.iter().map(|c|json!({"id":c.id,"agent":c.agent,"start_ns":c.start,"end_ns":c.end})).collect::<Vec<_>>(),
            "overlapping_call_ids":pairs, "peak_distinct_participants":peak,
            "calls_with_boolean_outcome":intervals.iter().filter(|call| call.success.is_some()).count(),
            "negative_outcomes":negative_outcomes,
        }),
        intervals.len(),
        pairs.len(),
    ))
}

pub(super) fn write_report(input: &Path, output: &Path) -> Result<()> {
    let bytes = std::fs::read(input)?;
    let cohort: Value = serde_json::from_slice(&bytes)?;
    let runs = cohort["runs"].as_array().ok_or("missing cohort runs")?;
    if runs.is_empty() || cohort["conditions"].as_u64() != Some(runs.len() as u64) {
        return Err("empty or incomplete cohort".into());
    }
    let mut conditions = BTreeSet::new();
    let mut events = BTreeSet::new();
    let mut tasks = BTreeSet::new();
    let (mut calls, mut pairs, mut overlapping) = (0, 0, 0);
    let mut cases = Vec::new();
    for run in runs {
        let task = run["task"].as_str().ok_or("missing task")?;
        let depth = run["depth"].as_u64().ok_or("missing depth")?;
        if !conditions.insert((task, depth))
            || !events.insert(run["events_sha256"].as_str().ok_or("missing events hash")?)
        {
            return Err("repeated condition or trace in cohort".into());
        }
        let (case, n_calls, n_pairs) = measure(run)?;
        if n_pairs > 0 {
            overlapping += 1;
            tasks.insert(task);
        }
        pairs += n_pairs;
        calls += n_calls;
        cases.push(case);
    }
    let report = json!({
        "cohort_sha256":format!("{:x}",Sha256::digest(bytes)),
        "analysis_sha256":format!("{:x}",Sha256::digest(include_bytes!("effect_overlap.rs"))),
        "scope":"Selected development runs; different participants' simulator request intervals on the same runner clock, including simulated delays and callback overhead.",
        "conditions":cases.len(), "calls":calls, "conditions_with_overlap":overlapping,
        "tasks_with_overlap":tasks, "cross_participant_pairs":pairs, "cases":cases,
    });
    let mut file = File::create_new(output)?;
    serde_json::to_writer_pretty(&mut file, &report)?;
    file.write_all(b"\n")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn touching_and_empty_intervals_do_not_overlap_and_peak_counts_participants() {
        let calls = [
            (1, "A", 0, 10),
            (2, "A", 1, 6),
            (3, "B", 1, 1),
            (4, "C", 4, 8),
            (5, "D", 8, 10),
            (6, "B", 10, 12),
        ]
        .map(|(id, agent, start, end)| Interval {
            id,
            agent: agent.into(),
            tool: String::from("fixture"),
            arguments: None,
            success: None,
            start,
            end,
        });
        assert_eq!(overlaps(&calls), (vec![[1, 4], [1, 5], [2, 4]], 2));
    }
}
