//! Scripted requests through real WASI guests and the unchanged 3E simulator.
//! These inputs test the implementation; they cannot grant a protocol step.
use crate::{
    evidence::{Result, Trace, archive_build_receipt, digest, save},
    oracle, process,
};
use serde_json::{Value, json};
use std::{
    collections::VecDeque,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Schedule {
    FinishFirst,
    Alternate,
    BothRejectedThenFirst,
    BothRejectedThenOther,
}
struct Case {
    depth: u8,
    first: &'static str,
    schedule: Schedule,
    delay: f64,
    extra: &'static [u32],
    adversarial_inputs: bool,
}

fn response(reply: &Value) -> Result<Value> {
    let text = reply["stdout"].as_str().ok_or("report terminal output")?;
    let at = text.find('{').ok_or("missing report service response")?;
    serde_json::Deserializer::from_str(&text[at..])
        .into_iter::<Value>()
        .next()
        .ok_or("empty report response")?
        .map_err(Into::into)
}

async fn researcher(
    guest: &process::Guest,
    agent: &str,
    section: &str,
    adversarial_inputs: bool,
) -> Result<Vec<Value>> {
    let topic = if section == "section_a" {
        "subtopic_a"
    } else {
        "subtopic_b"
    };
    if adversarial_inputs {
        let other = if section == "section_a" {
            "section_b"
        } else {
            "section_a"
        };
        for (tool, arguments) in [
            ("research_topic", json!({})),
            ("research_topic", json!({"topic":"unassigned_topic"})),
            ("write_section", json!({"section_name":other})),
        ] {
            let denied = guest
                .command(
                    agent,
                    format!("echo '{arguments}' > /service/{tool}; cat /service/result"),
                )
                .await?;
            assert_eq!(response(&denied)?["status"], "invalid_arguments");
        }
    }
    let research = guest
        .command(
            agent,
            format!(
                "echo '{{\"topic\":\"{topic}\"}}' > /service/research_topic; cat /service/result"
            ),
        )
        .await?;
    assert_eq!(response(&research)?["success"], true);
    let notes = "independently selected research notes";
    let write = guest
        .command(
            agent,
            format!(
                "printf '%s' ' {}' > /service/write_section; cat /service/result",
                json!({"section_name":section,"content_notes":notes})
            ),
        )
        .await?;
    assert_eq!(response(&write)?["success"], true);
    let references = guest.command(agent,format!("echo '{{\"section_name\":\"{section}\"}}' > /service/update_references; cat /service/result")).await?;
    let mut review = response(&references)?["coordination"]["review_sections"].clone();
    let mut feedback = Vec::new();
    loop {
        let accepted = review["success"].as_bool().ok_or("actual section review")?;
        feedback.push(review.clone());
        if accepted {
            break;
        }
        let padding = " ".repeat(512);
        let revised = guest.command(agent,format!("printf '%s' '{padding}{}' > /service/revise_section; cat /service/result",json!({"section_name":section,"feedback":review["message"].as_str().ok_or("original review feedback")?}))).await?;
        let revised = response(&revised)?;
        assert_eq!(revised["success"], true);
        review = revised["coordination"]["review_sections"].clone();
    }
    if adversarial_inputs {
        for (tool, arguments) in [
            ("research_topic", json!({"topic":topic})),
            ("write_section", json!({"section_name":section})),
            ("update_references", json!({"section_name":section})),
            (
                "revise_section",
                json!({"section_name":section,"feedback":"after approval"}),
            ),
        ] {
            let denied = hibana_runtime::race(
                guest.command(agent, format!("echo '{arguments}' > /service/{tool}")),
                hibana_runtime::host::deadline(Duration::from_secs(3)),
            )
            .await?;
            assert!(
                denied["stdout"]
                    .as_str()
                    .ok_or("post-approval stdout")?
                    .contains("wash:"),
                "approved section accepted another revision: {denied}"
            );
            let result = guest.command(agent, "cat /service/result".into()).await?;
            assert_eq!(response(&result)?["status"], "out_of_order");
        }
    }
    if !adversarial_inputs {
        assert_eq!(
            guest.command(agent, "exit 0".into()).await?["exit_status"],
            0
        );
    }
    Ok(feedback)
}

async fn editor(
    guest: &process::Guest,
    first: &str,
    schedule: Schedule,
    adversarial_inputs: bool,
) -> Result<()> {
    if adversarial_inputs {
        let denied = guest.command("EDITOR", "echo '{\"sections\":\"section_a,section_b\"}' > /service/combine_sections; cat /service/result".into()).await?;
        assert_eq!(response(&denied)?["status"], "out_of_order");
    }
    let other = if first == "section_a" {
        "section_b"
    } else {
        "section_a"
    };
    let mut requests = VecDeque::from([first, other]);
    let mut results = Vec::new();
    while let Some(section) = requests.pop_front() {
        let reply = guest.command("EDITOR",format!("echo '{{\"section_name\":\"{section}\"}}' > /service/review_sections; cat /service/result")).await?;
        let reply = response(&reply)?;
        let accepted = reply["success"].as_bool().ok_or("original review result")?;
        results.push((section, accepted));
        if !accepted {
            match schedule {
                Schedule::FinishFirst => requests.push_front(section),
                Schedule::Alternate => requests.push_back(section),
                Schedule::BothRejectedThenFirst | Schedule::BothRejectedThenOther => {
                    if results.len() == 1 {
                        requests.push_back(section);
                    } else {
                        let selected = if matches!(schedule, Schedule::BothRejectedThenFirst) {
                            first
                        } else {
                            other
                        };
                        requests.retain(|pending| *pending != section);
                        if section == selected {
                            requests.push_front(section);
                        } else {
                            requests.push_back(section);
                        }
                    }
                }
            }
        }
    }
    for section in [first, other] {
        assert!(
            results
                .iter()
                .rev()
                .find(|(target, _)| *target == section)
                .ok_or("missing final section review")?
                .1
        );
    }
    let result = guest.command("EDITOR","echo '{\"sections\":\" section_b, section_a \"}' > /service/combine_sections; cat /service/result".into()).await?;
    assert_eq!(response(&result)?["success"], true);
    if !adversarial_inputs {
        assert_eq!(
            guest.command("EDITOR", "exit 0".into()).await?["exit_status"],
            0
        );
    }
    Ok(())
}

fn ordinary() -> Vec<Case> {
    let mut cases = Vec::new();
    for depth in 0..=2 {
        for first in ["section_a", "section_b"] {
            for schedule in [Schedule::FinishFirst, Schedule::Alternate] {
                cases.push(Case {
                    depth,
                    first,
                    schedule,
                    delay: 0.01,
                    extra: &[],
                    adversarial_inputs: false,
                });
                if (depth == 0 && matches!(schedule, Schedule::FinishFirst))
                    || (depth > 0 && matches!(schedule, Schedule::Alternate))
                {
                    cases.push(Case {
                        depth,
                        first,
                        schedule,
                        delay: 1.0,
                        extra: &[],
                        adversarial_inputs: false,
                    });
                }
            }
        }
    }
    cases
}
fn repeated() -> Vec<Case> {
    let mut cases = Vec::new();
    for first in ["section_a", "section_b"] {
        for schedule in [
            Schedule::BothRejectedThenFirst,
            Schedule::BothRejectedThenOther,
        ] {
            for extra in [&[2][..], &[2, 3][..]] {
                cases.push(Case {
                    depth: 2,
                    first,
                    schedule,
                    delay: 0.01,
                    extra,
                    adversarial_inputs: false,
                });
            }
        }
    }
    cases
}

#[hibana_runtime::main]
#[test]
async fn report_input_domain_is_complete_unique_and_cannot_run_empty() -> Result<()> {
    use std::collections::BTreeSet;
    for (cases, expected, extended) in [(ordinary(), 18, false), (repeated(), 8, true)] {
        let identities = cases
            .iter()
            .map(|case| {
                (
                    case.depth,
                    case.first,
                    case.schedule,
                    case.delay.to_bits(),
                    case.extra,
                )
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(cases.len(), expected);
        assert_eq!(identities.len(), expected);
        assert!(cases.iter().all(|case| {
            !case.adversarial_inputs
                && matches!(case.first, "section_a" | "section_b")
                && if extended {
                    case.depth == 2
                        && case.delay == 0.01
                        && matches!(
                            case.schedule,
                            Schedule::BothRejectedThenFirst | Schedule::BothRejectedThenOther
                        )
                        && matches!(case.extra, [2] | [2, 3])
                } else {
                    case.depth <= 2
                        && case.extra.is_empty()
                        && matches!(case.schedule, Schedule::FinishFirst | Schedule::Alternate)
                        && (case.delay == 0.01
                            || case.delay == 1.0
                                && case.schedule
                                    == if case.depth == 0 {
                                        Schedule::FinishFirst
                                    } else {
                                        Schedule::Alternate
                                    })
                }
        }));
    }
    assert_eq!(
        run(Vec::new()).await.unwrap_err().to_string(),
        "report qualification requires at least one actual input"
    );
    Ok(())
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires the current report WASI binary, receipt, wash, auditor and original simulator"]
async fn actual_wasi_report_original_conditions_and_review_orders() -> Result<()> {
    run(ordinary()).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires the current report WASI binary, receipt, wash, auditor and original simulator"]
async fn actual_wasi_report_third_and_fourth_rejections() -> Result<()> {
    run(repeated()).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires the current report WASI binary, receipt, wash, auditor and original simulator"]
async fn actual_wasi_report_rejects_invalid_and_post_completion_requests() -> Result<()> {
    run(vec![Case {
        depth: 2,
        first: "section_a",
        schedule: Schedule::FinishFirst,
        delay: 0.01,
        extra: &[],
        adversarial_inputs: true,
    }])
    .await
}

#[hibana_runtime::scope]
async fn run(cases: Vec<Case>) -> Result<()> {
    if cases.is_empty() {
        return Err("report qualification requires at least one actual input".into());
    }
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    )?;
    let root =
        PathBuf::from(std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("report evidence root")?);
    fs::create_dir_all(&root)?;
    let directory = root.join(format!(
        "report-rust-boundary-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir(&directory)?;
    let binary =
        PathBuf::from(std::env::var_os("CHOREOFIX_REPORT_WASI_BINARY").ok_or("report binary")?)
            .canonicalize()?;
    let receipt = PathBuf::from(
        std::env::var_os("CHOREOFIX_REPORT_BUILD_RECEIPT").ok_or("report build receipt")?,
    );
    let receipt = archive_build_receipt(&binary, &receipt, &directory)?;
    let auditor = PathBuf::from(std::env::var_os("CHOREOFIX_AUDITOR").ok_or("report auditor")?)
        .canonicalize()?;
    let module = Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let assignments = cases.iter().map(|case| json!({"depth":case.depth,"first":case.first,"schedule":format!("{:?}",case.schedule),"delay":case.delay,"extra_failures":case.extra,"adversarial_inputs":case.adversarial_inputs})).collect::<Vec<_>>();
    save(&directory.join("assignments.json"), &json!(assignments))?;
    let mut outcomes = Vec::new();
    for case in cases {
        let output = directory.join(format!(
            "depth{}-{}-{:?}-delay{}-extra{:?}",
            case.depth, case.first, case.schedule, case.delay, case.extra
        ));
        fs::create_dir(&output)?;
        crate::projection::archive(&binary, &output, "3E")?;
        save(
            &output.join("case.json"),
            &json!({"task":"3E","depth":case.depth,"review_first":case.first,"schedule":format!("{:?}",case.schedule),"delay_multiplier":case.delay,
            "kind":"deterministic_rust_wasi_regression","model_calls":0,"scored_benchmark":false,
            "test_barriers":if case.adversarial_inputs { "keep the three actual guests alive until combine completes, then attempt another edit" } else { "none" },"build":receipt,
            "runtime_binary_sha256":digest(&binary)?,"guest_wasm_sha256":digest(module)?,"compiled_projection_sha256":digest(&output.join("projection.json"))?,
            "harness_binary_sha256":digest(&std::env::current_exe()?)?,"harness_source_sha256":digest(Path::new("tools/runner/src/report_boundary.rs"))?,
            "audit_source_sha256":digest(Path::new("tools/runner/src/report_audit.rs"))?,"journal_source_sha256":digest(Path::new("tools/runner/src/journal.rs"))?,
            "failure_overrides":case.extra.iter().map(|&index|("review_sections",index)).collect::<Vec<_>>(),
            "adversarial_inputs":case.adversarial_inputs}),
        )?;
        let trace = Trace::create(&output.join("events.jsonl"))?;
        let failures = case
            .extra
            .iter()
            .map(|&index| ("review_sections", index))
            .collect::<Vec<_>>();
        let (simulator, _) =
            oracle::Simulator::new("3E", case.depth, case.delay, trace.clone(), &failures)?;
        let (guest, runtime) = process::guest(&binary, module, &["3E"], &output, trace, simulator)?;
        let (a, b, result) = hibana_runtime::race(
            async {
                if case.adversarial_inputs {
                    let ((a, b), result) = async {
                        async {
                            let (a, b, ()) = async {
                                async { researcher(&guest, "RESEARCHER_A", "section_a", true).await };
                                async { researcher(&guest, "RESEARCHER_B", "section_b", true).await };
                                async { editor(&guest, case.first, case.schedule, true).await };
                            }.await?;
                            // All three real replies have arrived, including combine.
                            for (agent, section) in [("RESEARCHER_A", "section_a"), ("RESEARCHER_B", "section_b")] {
                                let denied = hibana_runtime::race(
                                    guest.command(agent, format!("echo '{}' > /service/revise_section", json!({"section_name":section,"feedback":"after the combined report completed"}))),
                                    hibana_runtime::host::deadline(Duration::from_secs(3)),
                                ).await?;
                                assert!(denied["stdout"].as_str().ok_or("post-combine stdout")?.contains("wash:"), "completed report accepted another edit: {denied}");
                                let reply = guest.command(agent, "cat /service/result".into()).await?;
                                assert_eq!(response(&reply)?["status"], "out_of_order");
                            }
                            async {
                                let guest = &guest;
                                for agent in ["RESEARCHER_A", "RESEARCHER_B", "EDITOR"] {
                                    async {
                                        assert_eq!(guest.command(agent, "exit 0".into()).await?["exit_status"], 0);
                                        Ok(())
                                    };
                                }
                                Ok(())
                            }.await?;
                            Ok((a, b))
                        };
                        async { runtime.await };
                    }.await?;
                    Ok((a, b, result))
                } else {
                    async {
                async {
                    editor(&guest, case.first, case.schedule, false).await?;
                };
                let (a, b, result) = async {
                    async {
                        researcher(
                            &guest,
                            "RESEARCHER_A",
                            "section_a",
                            case.adversarial_inputs,
                        )
                        .await
                    };
                    async {
                        researcher(
                            &guest,
                            "RESEARCHER_B",
                            "section_b",
                            case.adversarial_inputs,
                        )
                        .await
                    };
                    async { runtime.await };
                }
                .await?;
                Ok((a, b, result))
                    }.await
                }
            },
            hibana_runtime::host::deadline(Duration::from_secs(120)),
        )
        .await?;
        assert_eq!(
            result["protocol"],
            json!({"type":"run_result","status":"complete"})
        );
        assert_eq!(result["runtime_exit_code"], 0);
        assert_eq!(result["backend_errors"], json!([]));
        assert_eq!(result["score"]["complete"], true);
        assert_eq!(result["score"]["violations"], json!([]));
        let rejected = usize::from(case.depth) + case.extra.len();
        assert_eq!(
            a.iter()
                .chain(&b)
                .filter(|reply| reply["success"] == false)
                .count(),
            rejected
        );
        assert_eq!(
            result["score"]["events"]
                .as_array()
                .ok_or("original report event list")?
                .len(),
            9 + 2 * rejected
        );
        let status = Command::new(&auditor)
            .arg("--audit-report")
            .arg(output.join("events.jsonl"))
            .stdout(Stdio::from(fs::File::create(output.join("audit.json"))?))
            .stderr(Stdio::from(fs::File::create(
                output.join("audit-stderr.log"),
            )?))
            .status()?;
        if !status.success() {
            return Err(format!("report audit failed ({status}): {}", output.display()).into());
        }
        let audit: Value = serde_json::from_slice(&fs::read(output.join("audit.json"))?)?;
        assert_eq!(audit["status"], "passed");
        assert_eq!(audit["dynamic_protocol"]["status"], "passed");
        assert_eq!(audit["actual_resource_endpoint_requests"], 5 + rejected);
        assert_eq!(audit["actual_review_requests"], 2 + rejected);
        let status = Command::new(&auditor)
            .arg("--check-report-mutants")
            .arg(output.join("events.jsonl"))
            .arg(output.join("resource-mutants"))
            .stdout(Stdio::from(fs::File::create(output.join("mutants.log"))?))
            .stderr(Stdio::from(fs::File::create(
                output.join("mutants-stderr.log"),
            )?))
            .status()?;
        if !status.success() {
            return Err(format!("report mutations failed ({status}): {}", output.display()).into());
        }
        let mutants: Value =
            serde_json::from_slice(&fs::read(output.join("resource-mutants/summary.json"))?)?;
        assert_eq!(mutants["status"], "passed");
        assert!(mutants["count"].as_u64().is_some_and(|count| count > 0));
        if case.delay == 1.0 {
            for kind in [
                "research_topic+research_topic",
                "update_references+write_section",
            ] {
                assert!(
                    audit["overlapping_effect_pairs"][kind]
                        .as_u64()
                        .is_some_and(|count| count > 0),
                    "missing actual {kind} overlap"
                );
            }
            if case.depth > 0 && matches!(case.schedule, Schedule::Alternate) {
                assert!(
                    audit["overlapping_effect_pairs"]["review_sections+revise_section"]
                        .as_u64()
                        .is_some_and(|count| count > 0)
                );
            }
        }
        outcomes.push(
            json!({"case":output.file_name().and_then(|name| name.to_str()).ok_or("report case name")?,"audit":audit,"record_mutants":mutants["count"],"original_score":result["score"]}),
        );
    }
    assert_eq!(outcomes.len(), assignments.len());
    save(
        &directory.join("result.json"),
        &json!({"status":"passed","cases":outcomes,"model_calls":0,"scored_benchmark":false}),
    )?;
    Ok(())
}
