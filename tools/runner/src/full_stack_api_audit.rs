//! Check actual full-stack effects against EPF-committed requests and results.
use crate::{
    evidence::{Result, digest},
    journal::{self, one},
    projection,
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, ops::Range, path::Path};

pub fn audit(path: &Path) -> Result<Value> {
    let rows = journal::records(path)?;
    let artifact = projection::load(path)?;
    let compiled = &artifact["compiled"];
    let fixture = &rows.first().ok_or("empty full-stack evidence")?["message"];
    let participants = fixture["fixtures"]
        .as_array()
        .ok_or("full-stack participants")?;
    if compiled["task"] != "8M"
        || fixture["task"] != "8M"
        || fixture["type"] != "fixtures"
        || participants.len() != 5
        || participants
            .iter()
            .filter_map(|p| p["agent"].as_str())
            .collect::<BTreeSet<_>>()
            != BTreeSet::from([
                "DB_ADMIN",
                "BACKEND_DEV",
                "FRONTEND_DEV",
                "REVIEWER",
                "TESTER",
            ])
    {
        return Err("foreign full-stack task or participant set".into());
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
    let headers = projection::frames(&artifact, &rows)?;
    let dynamic = projection::check_dynamic(&artifact, &rows)?;
    let mut journal = journal::read(&rows, &headers, participants)?;
    let receipts = journal.bind_receives(&dynamic)?;
    let frames = &journal.frames;
    let events = journal.score["events"]
        .as_array()
        .ok_or("common score events")?;
    if journal.score["complete"] != true
        || journal.score["violations"] != json!([])
        || events.len() != journal.calls.len()
        || frames
            .values()
            .any(|frame| frame.identity["session"] != compiled["session"])
    {
        return Err("incomplete full-stack score or resource violation".into());
    }
    let pairs = ["DB", "BACKEND", "FRONTEND", "REVIEW", "TEST"]
        .map(|prefix| {
            Ok((
                role(&format!("{prefix}_VM"))?,
                role(&format!("{prefix}_IO"))?,
            ))
        })
        .into_iter()
        .collect::<Result<Vec<_>>>()?;
    journal::check_close_pairs(frames, &pairs)?;

    let mut used = BTreeSet::new();
    let mut take = |(from, to, message): (&str, &str, &str),
                    actor: Option<&str>,
                    interval: Range<usize>,
                    expected: Option<&Value>|
     -> Result<(u64, Value)> {
        let (from_id, to_id, logical) = (role(from)?, role(to)?, label(message)?);
        let actor = actor.map(role).transpose()?;
        let (&id, frame) = one(frames.iter().filter(|(id, frame)| {
            !used.contains(*id)
                && frame.direction(from_id, to_id)
                && u64::from(frame.logical_label) == logical
                && frame.received.is_some_and(|at| interval.contains(&at))
                && actor.is_none_or(|actor| receipts[id]["receive_commit"]["actor"] == actor)
                && expected.is_none_or(|value| frame.value().is_ok_and(|payload| payload == *value))
        }))
        .map_err(|error| format!("{message} {from}->{to} in {interval:?}: {error}"))?;
        used.insert(id);
        Ok((id, frame.value()?))
    };

    let mut ordered = journal.calls.values().collect::<Vec<_>>();
    ordered.sort_by_key(|call| call.emitted);
    let latest = |tool: &str, target: Option<&str>, before: usize| -> Result<&Value> {
        ordered
            .iter()
            .filter(|call| {
                call.tool().ok() == Some(tool)
                    && target.is_none_or(|target| call.request["arguments"]["pr_id"] == target)
            })
            .filter_map(|call| call.result().ok())
            .filter(|(at, _)| *at < before)
            .max_by_key(|(at, _)| *at)
            .map(|(_, value)| value)
            .ok_or_else(|| format!("no completed {tool} before {before}").into())
    };

    let mut seen_scores = BTreeSet::new();
    let mut effects = Vec::new();
    let mut early_requests = Vec::new();
    let mut previous_attempt = None;
    for (index, &call) in ordered.iter().enumerate() {
        let tool = call.tool()?;
        let agent = call.request["agent"]
            .as_str()
            .ok_or("full-stack effect agent")?;
        let args = &call.request["arguments"];
        let (returned, result) = call.result()?;
        let success = result["success"]
            .as_bool()
            .ok_or("full-stack effect success")?;
        let data = &result["data"];
        let first_backend = tool == "implement_backend"
            && !ordered[..index]
                .iter()
                .any(|earlier| earlier.tool().ok() == Some(tool));
        let in_recovery = previous_attempt.is_some();

        let mut replies = Vec::new();
        let mut returns = Vec::new();
        let (source, target, operation, performer) = match (agent, tool) {
            ("DB_ADMIN", "migrate_schema") if success => {
                replies.extend([
                    ("DATABASE", "DB_FILE", "SchemaMigrated"),
                    ("DATABASE", "BACKEND_SERVICE", "SchemaReady"),
                    ("BACKEND_SERVICE", "BACKEND_FILE", "SchemaReady"),
                ]);
                if !ordered[..index]
                    .iter()
                    .any(|earlier| earlier.tool().ok() == Some("implement_backend"))
                    && let Ok(before) = latest(tool, None, call.emitted)
                {
                    let (request, reason) = take(
                        ("BACKEND_FILE", "BACKEND_SERVICE", "NeedSchema"),
                        Some("BACKEND_SERVICE"),
                        0..call.emitted,
                        None,
                    )?;
                    if reason.as_object().is_none_or(|fields| fields.len() != 1)
                        || reason["reason"]
                            .as_str()
                            .is_none_or(|reason| reason.trim().is_empty())
                    {
                        return Err("invalid observed reason for early migration".into());
                    }
                    let request_args = json!({"reason":reason["reason"],
                        "schema":before["data"]["revision"],"status":"migration_requested"});
                    let (grant, _) = take(
                        ("BACKEND_SERVICE", "DB_SERVICE", "RequestMigration"),
                        Some("DB_SERVICE"),
                        frames[&request].received()?..call.emitted,
                        Some(&request_args),
                    )?;
                    let (notice, _) = take(
                        ("DB_SERVICE", "DB_FILE", "SchemaStatus"),
                        Some("DB_VM"),
                        frames[&grant].received()?..call.emitted,
                        Some(&request_args),
                    )?;
                    early_requests.push(
                        json!({"request_frame":request,"grant_frame":grant,"notice_frame":notice,
                        "old_schema":before["data"]["revision"],"new_schema":data["revision"]}),
                    );
                }
                ("DB_FILE", "DATABASE", "MigrateSchema", "DB_SERVICE")
            }
            ("BACKEND_DEV", "define_api") if success => {
                replies.extend([
                    ("API_GATEWAY", "BACKEND_FILE", "ApiDefined"),
                    ("API_GATEWAY", "REVIEW_SERVICE", "ApiContract"),
                ]);
                ("BACKEND_FILE", "API_GATEWAY", "DefineApi", "API_SERVICE")
            }
            ("BACKEND_DEV", "implement_backend") if success => {
                if data["inputs"]["schema"]
                    != latest("migrate_schema", None, call.emitted)?["data"]["revision"]
                {
                    return Err("backend implementation did not use the current schema".into());
                }
                replies.extend([
                    ("BACKEND_SERVICE", "BACKEND_FILE", "BackendImplemented"),
                    ("BACKEND_SERVICE", "REVIEW_SERVICE", "BackendCode"),
                ]);
                if first_backend {
                    replies.extend([
                        ("BACKEND_SERVICE", "DB_SERVICE", "SchemaInUse"),
                        ("DB_SERVICE", "DB_FILE", "SchemaStatus"),
                    ]);
                }
                (
                    "BACKEND_FILE",
                    "BACKEND_SERVICE",
                    "ImplementBackend",
                    "BACKEND_SERVICE",
                )
            }
            ("FRONTEND_DEV", "implement_frontend") if success => {
                let backend = latest("code_review", Some("backend_api"), call.emitted)?;
                if backend["success"] != true
                    || data["inputs"]
                        != json!({
                            "schema":backend["data"]["inputs"]["schema"],
                            "api":backend["data"]["inputs"]["api"],"backend_review":backend["data"]["revision"]
                        })
                {
                    return Err(
                        "frontend implementation did not use the current backend approval".into(),
                    );
                }
                replies.extend([
                    ("FRONTEND_SERVICE", "FRONTEND_FILE", "FrontendImplemented"),
                    ("FRONTEND_SERVICE", "REVIEW_SERVICE", "UiCode"),
                ]);
                (
                    "FRONTEND_FILE",
                    "FRONTEND_SERVICE",
                    "ImplementFrontend",
                    "FRONTEND_SERVICE",
                )
            }
            ("REVIEWER", "code_review") if success || data["status"] == "changes_requested" => {
                let (implementation, request, ready) = match args["pr_id"].as_str() {
                    Some("backend_api") => {
                        let status = if success {
                            "BackendAccepted"
                        } else if in_recovery {
                            "BackendChangesAfterTest"
                        } else {
                            "BackendChanges"
                        };
                        replies.extend([
                            ("REVIEW_SERVICE", "BACKEND_SERVICE", status),
                            ("REVIEW_SERVICE", "REVIEW_FILE", "BackendReviewed"),
                            ("REVIEW_SERVICE", "BACKEND_FILE", "BackendOutcome"),
                        ]);
                        if success {
                            replies.push(("REVIEW_SERVICE", "FRONTEND_SERVICE", "BackendApproval"));
                        }
                        ("implement_backend", "ReviewBackend", "BackendCandidate")
                    }
                    Some("frontend_ui") => {
                        let status = if success {
                            "UiAccepted"
                        } else if in_recovery {
                            "UiChangesAfterTest"
                        } else {
                            "UiChanges"
                        };
                        replies.extend([
                            ("REVIEW_SERVICE", "FRONTEND_SERVICE", status),
                            ("REVIEW_SERVICE", "REVIEW_FILE", "FrontendReviewed"),
                            ("REVIEW_SERVICE", "FRONTEND_FILE", "UiOutcome"),
                        ]);
                        ("implement_frontend", "ReviewFrontend", "UiCandidate")
                    }
                    _ => return Err("foreign review target".into()),
                };
                let revision = latest(implementation, None, call.emitted)?;
                let expected = json!({"schema":revision["data"]["inputs"]["schema"],
                    "api":latest("define_api", None, call.emitted)?["data"]["contract"],"code":revision["data"]["revision"]});
                if data["inputs"] != expected {
                    return Err("review did not examine the current candidate".into());
                }
                take(
                    ("REVIEW_SERVICE", "REVIEW_FILE", ready),
                    Some("REVIEW_VM"),
                    0..call.emitted,
                    Some(&expected),
                )?;
                ("REVIEW_FILE", "REVIEW_SERVICE", request, "REVIEW_SERVICE")
            }
            ("TESTER", "run_integration_tests") if success || data["status"] == "tests_failed" => {
                let backend = latest("code_review", Some("backend_api"), call.emitted)?;
                let frontend = latest("code_review", Some("frontend_ui"), call.emitted)?;
                let backend_code = latest("implement_backend", None, call.emitted)?;
                let frontend_code = latest("implement_frontend", None, call.emitted)?;
                let schema = &latest("migrate_schema", None, call.emitted)?["data"]["revision"];
                let api = &latest("define_api", None, call.emitted)?["data"]["contract"];
                for (review, code) in [(backend, backend_code), (frontend, frontend_code)] {
                    if review["success"] != true
                        || review["data"]["inputs"]
                            != json!({"schema":schema,"api":api,"code":code["data"]["revision"]})
                    {
                        return Err(
                            "integration lacks approval of the current implementation".into()
                        );
                    }
                }
                let inputs = json!({
                    "candidate":{"schema":schema,"api":api,"backend":backend_code["data"]["revision"],"frontend":frontend_code["data"]["revision"]},
                    "backend_review":backend["data"]["revision"],"frontend_review":frontend["data"]["revision"],
                });
                if data["inputs"] != inputs {
                    return Err("integration used stale inputs".into());
                }
                let (grant, _) = take(
                    ("REVIEW_SERVICE", "TEST_ENV", "TestInputs"),
                    Some("TEST_SERVICE"),
                    0..call.emitted,
                    Some(&inputs),
                )?;
                take(
                    ("TEST_ENV", "TEST_FILE", "TestInputs"),
                    Some("TEST_VM"),
                    frames[&grant].received()?..call.emitted,
                    Some(&inputs),
                )?;

                if let Some((old, component, decision_at)) = &previous_attempt {
                    let old: &Value = old;
                    let changed = &inputs["candidate"];
                    let stable = &old["candidate"];
                    if match *component {
                        "frontend" => {
                            changed["frontend"] == stable["frontend"]
                                || changed["backend"] != stable["backend"]
                                || inputs["backend_review"] != old["backend_review"]
                        }
                        "backend" => {
                            changed["backend"] == stable["backend"]
                                || changed["frontend"] != stable["frontend"]
                                || inputs["frontend_review"] != old["frontend_review"]
                        }
                        "schema" => {
                            changed["schema"] == stable["schema"]
                                || changed["api"] != stable["api"]
                                || changed["backend"] == stable["backend"]
                                || changed["frontend"] == stable["frontend"]
                        }
                        "api" => {
                            changed["schema"] != stable["schema"]
                                || changed["frontend"] == stable["frontend"]
                        }
                        _ => true,
                    } {
                        return Err("retest did not reflect the actual selected correction".into());
                    }
                    let corrected = match *component {
                        "frontend" => "implement_frontend",
                        "backend" => "implement_backend",
                        "schema" => "migrate_schema",
                        "api" => "define_api",
                        _ => unreachable!(),
                    };
                    if !ordered[..index].iter().any(|prior| {
                        prior.emitted > *decision_at && prior.tool().ok() == Some(corrected)
                    }) {
                        return Err("chosen correction did not execute before the retest".into());
                    }
                }
                replies.extend([
                    ("TEST_ENV", "TEST_FILE", "IntegrationResult"),
                    ("TEST_OWNER", "REVIEW_SERVICE", "IntegrationReport"),
                    ("REVIEW_SERVICE", "REVIEW_FILE", "ReviewOutcome"),
                ]);
                let developer_outcomes = [
                    ("REVIEW_SERVICE", "DB_FILE", "DbOutcome"),
                    ("REVIEW_SERVICE", "FRONTEND_FILE", "UiOutcome"),
                    ("REVIEW_SERVICE", "BACKEND_FILE", "BackendOutcome"),
                ];
                if success {
                    replies.extend(developer_outcomes);
                    replies.extend([
                        ("TEST_ENV", "TEST_OWNER", "IntegrationPassed"),
                        ("REVIEW_SERVICE", "FRONTEND_SERVICE", "IntegrationPassed"),
                        (
                            "REVIEW_SERVICE",
                            "BACKEND_SERVICE",
                            "BackendIntegrationPassed",
                        ),
                        ("TEST_OWNER", "DB_SERVICE", "ReleaseDatabase"),
                        ("TEST_OWNER", "API_SERVICE", "ReleaseGateway"),
                    ]);
                    if index + 1 != ordered.len() {
                        return Err("effects after final approved integration".into());
                    }
                } else {
                    let next = ordered[index + 1..]
                        .iter()
                        .find(|next| next.tool().ok() == Some(tool))
                        .ok_or("failed test has no retest")?;
                    let (decision, selected) = take(
                        ("REVIEW_FILE", "REVIEW_SERVICE", "RepairDecision"),
                        Some("REVIEW_SERVICE"),
                        returned + 1..next.emitted,
                        None,
                    )?;
                    let component = match selected["component"].as_str() {
                        Some("frontend") => "frontend",
                        Some("backend") => "backend",
                        Some("schema") => "schema",
                        Some("api") => "api",
                        _ => return Err("foreign repair component".into()),
                    };
                    if selected.as_object().is_none_or(|fields| fields.len() != 1) {
                        return Err("invalid repair receipt".into());
                    }
                    let decision_at = frames[&decision].received()?;
                    let notice = json!({"repair":selected,"test":result});
                    for edge in developer_outcomes {
                        let (id, _) = take(edge, None, decision_at + 1..rows.len(), Some(&notice))?;
                        returns.push(id);
                    }
                    let (ui, backend, db, api) = match component {
                        "frontend" => (
                            "IntegrationFailed",
                            "BackendRetained",
                            "SchemaRetained",
                            "ApiRetained",
                        ),
                        "backend" => (
                            "UiRetained",
                            "BackendRepair",
                            "SchemaRetained",
                            "ApiRetained",
                        ),
                        "api" => (
                            "UiInvalidated",
                            "BackendRetained",
                            "SchemaRetained",
                            "ChangeApi",
                        ),
                        "schema" => (
                            "UiInvalidated",
                            "BackendSchemaInvalidated",
                            "ChangeSchema",
                            "ApiRetained",
                        ),
                        _ => unreachable!(),
                    };
                    replies.extend([
                        ("TEST_ENV", "TEST_OWNER", "IntegrationFailed"),
                        ("REVIEW_SERVICE", "FRONTEND_SERVICE", ui),
                        ("REVIEW_SERVICE", "BACKEND_SERVICE", backend),
                        ("REVIEW_SERVICE", "DB_SERVICE", db),
                        ("REVIEW_SERVICE", "API_SERVICE", api),
                    ]);
                    previous_attempt = Some((inputs, component, decision_at));
                }
                ("TEST_FILE", "TEST_ENV", "RunIntegration", "TEST_SERVICE")
            }
            _ => return Err(format!("foreign or invalid effect {agent} {tool}").into()),
        };
        let (request, _) = take(
            (source, target, operation),
            Some(performer),
            0..call.emitted,
            Some(args),
        )?;
        for edge in replies {
            let (id, _) = take(edge, None, returned + 1..rows.len(), Some(result))?;
            returns.push(id);
        }
        let (score_id, _) = one(events.iter().enumerate().filter(|(index, event)| {
            !seen_scores.contains(index)
                && event["agent"] == agent
                && event["tool"] == tool
                && event["args"] == *args
                && event["success"] == success
                && event["result"] == *data
                && event["violations"] == json!([])
        }))?;
        seen_scores.insert(score_id);
        effects.push(json!({"agent":agent,"tool":tool,"request_frame":request,
            "request_occurrence":receipts[&request]["occurrence"],"reply_frames":returns,"success":success}));
    }

    if effects
        .last()
        .is_none_or(|last| last["tool"] != "run_integration_tests" || last["success"] != true)
        || frames.iter().any(|(id, frame)| {
            [
                "MigrateSchema",
                "DefineApi",
                "ImplementBackend",
                "ImplementFrontend",
                "ReviewBackend",
                "ReviewFrontend",
                "RunIntegration",
                "NeedSchema",
                "RepairDecision",
            ]
            .iter()
            .filter_map(|name| label(name).ok())
            .any(|entry| u64::from(frame.logical_label) == entry)
                && !used.contains(id)
        })
    {
        return Err("missing terminal result or an unconsumed effect or decision request".into());
    }

    let mut transfers = Vec::new();
    for (resource, owner, holder) in [
        ("DATABASE", "DB_OWNER", "DB_SERVICE"),
        ("API_GATEWAY", "API_OWNER", "API_SERVICE"),
        ("TEST_ENV", "TEST_OWNER", "TEST_SERVICE"),
    ] {
        let (role_id, owner, holder) = (role(resource)?, role(owner)?, role(holder)?);
        let belongs = |frame: &journal::Frame| {
            u64::from(frame.logical_label) == label("Loan").unwrap_or(256)
                && frame
                    .bytes
                    .first()
                    .is_some_and(|&byte| u64::from(byte) == role_id)
        };
        let (&out, outgoing) = one(frames
            .iter()
            .filter(|(_, frame)| belongs(frame) && frame.direction(owner, holder)))?;
        let (&back, incoming) = one(frames
            .iter()
            .filter(|(_, frame)| belongs(frame) && frame.direction(holder, owner)))?;
        if outgoing.received()? >= incoming.received()? {
            return Err("resource was not returned after its acquisition".into());
        }
        transfers.push(json!({"resource":resource,"acquired":out,"returned":back}));
    }
    Ok(json!({
        "status":"passed","task":"8M","depth":fixture["depth"],"domain_calls":effects.len(),
        "effects":effects,"early_migration_requests":early_requests,"resource_transfers":transfers,
        "captured_records":dynamic["captured_records"],"simulator_complete":true,"violations":0,
        "source_events_sha256":digest(path)?,
        "compiled_projection_sha256":digest(&path.parent().ok_or("evidence directory")?.join("projection.json"))?,
        "runtime_binary_sha256":artifact["runtime_binary_sha256"],
    }))
}
