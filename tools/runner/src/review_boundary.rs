//! Deterministic shell stimuli for the actual 6E WASI runtime.
//! These tests neither authorize an operation nor replace a localside.
use crate::{
    evidence::{Result, Trace, archive_build_receipt, digest, save},
    oracle, process,
};
use serde_json::{Value, json};
use std::{
    collections::VecDeque,
    fs,
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Copy, Debug)]
enum ReviewSchedule {
    FinishFirst,
    Alternate,
    AlternateThenRepeat,
    BothRejectedThenFirst,
}

fn response(reply: &Value) -> Result<Value> {
    let text = reply["stdout"].as_str().ok_or("terminal stdout")?;
    let start = text
        .find('{')
        .ok_or_else(|| format!("service reply missing: {reply}"))?;
    serde_json::Deserializer::from_str(&text[start..])
        .into_iter::<Value>()
        .next()
        .ok_or("empty service reply")?
        .map_err(Into::into)
}

fn count(events: &[Value], agent: &str, tool: &str) -> usize {
    events
        .iter()
        .filter(|event| event["agent"] == agent && event["tool"] == tool)
        .count()
}

async fn developer(
    guest: &process::Guest,
    simulator: &oracle::Simulator,
    agent: &str,
    feature: &str,
    branch: &str,
) -> Result<Vec<Value>> {
    let early = guest
        .command(
            agent,
            format!("echo '{{\"branch\":\"{branch}\"}}' > /service/merge_code"),
        )
        .await?;
    assert!(
        early["stdout"]
            .as_str()
            .ok_or("early stdout")?
            .contains("wash:")
    );
    assert_eq!(
        count(&simulator.observed_events().await?, agent, "merge_code"),
        0
    );
    guest
        .command(agent, "echo '{}' > /service/implement_feature".into())
        .await?;
    let invalid = guest.command(agent, "cat /service/result".into()).await?;
    assert_eq!(response(&invalid)?["status"], "invalid_arguments");
    let (other_feature, other_branch) = if agent == "DEVELOPER_A" {
        ("notification_feature", "dev_b_feature")
    } else {
        ("user_authentication", "dev_a_feature")
    };
    let wrong = guest.command(agent, format!("echo '{{\"feature_name\":\"{other_feature}\"}}' > /service/implement_feature; cat /service/result")).await?;
    assert_eq!(response(&wrong)?["status"], "invalid_arguments");
    assert_eq!(
        count(
            &simulator.observed_events().await?,
            agent,
            "implement_feature"
        ),
        0
    );
    let mut feedback = Vec::new();
    loop {
        let padding = " ".repeat(512);
        let reply = guest.command(agent, format!("printf '%s' '{padding}{{\"feature_name\":\"{feature}\"}}' > /service/implement_feature; cat /service/result")).await?;
        let result = response(&reply)?;
        assert_eq!(result["success"], true);
        let review = result["coordination"]["code_review"].clone();
        let accepted = review["success"].as_bool().ok_or("actual review result")?;
        feedback.push(review);
        if accepted {
            break;
        }
        let denied = guest
            .command(
                agent,
                format!("echo '{{\"branch\":\"{branch}\"}}' > /service/merge_code"),
            )
            .await?;
        assert!(
            denied["stdout"]
                .as_str()
                .ok_or("rejected merge stdout")?
                .contains("wash:")
        );
        assert_eq!(
            count(&simulator.observed_events().await?, agent, "merge_code"),
            0
        );
    }
    let wrong = guest.command(agent, format!("echo '{{\"branch\":\"{other_branch}\"}}' > /service/merge_code; cat /service/result")).await?;
    assert_eq!(response(&wrong)?["status"], "invalid_arguments");
    assert_eq!(
        count(&simulator.observed_events().await?, agent, "merge_code"),
        0
    );
    let reply = guest
        .command(
            agent,
            format!(
                "echo '{{\"branch\":\"{branch}\"}}' > /service/merge_code; cat /service/result"
            ),
        )
        .await?;
    assert_eq!(response(&reply)?["success"], true);
    let before = simulator.observed_events().await?;
    for (tool, arguments) in [
        ("implement_feature", json!({"feature_name":feature})),
        ("merge_code", json!({"branch":branch})),
    ] {
        let reply = guest
            .command(agent, format!("echo '{arguments}' > /service/{tool}"))
            .await?;
        assert!(
            reply["stdout"]
                .as_str()
                .ok_or("duplicate stdout")?
                .contains("os error 8")
        );
        assert_eq!(
            count(&simulator.observed_events().await?, agent, tool),
            count(&before, agent, tool)
        );
    }
    assert_eq!(
        guest.command(agent, "exit 0".into()).await?["exit_status"],
        0
    );
    Ok(feedback)
}

async fn reviewer(
    guest: &process::Guest,
    simulator: &oracle::Simulator,
    first: &str,
    schedule: ReviewSchedule,
) -> Result<()> {
    let other = if first == "user_authentication" {
        "notification_feature"
    } else {
        "user_authentication"
    };
    let mut requests = VecDeque::from([first, other]);
    while let Some(feature) = requests.pop_front() {
        let other_feature = if feature == "user_authentication" {
            "notification_feature"
        } else {
            "user_authentication"
        };
        let before = simulator.observed_events().await?;
        let wrong = guest.command("REVIEWER", format!("echo '{{\"pr_id\":\"{other_feature}\"}}' > /service/code_review/{feature}; cat /service/result")).await?;
        assert_eq!(response(&wrong)?["status"], "invalid_arguments");
        let after = simulator.observed_events().await?;
        assert!(!after[before.len()..].iter().any(
            |event| event["tool"] == "code_review" && event["args"]["target"] == other_feature
        ));
        let reply = guest.command("REVIEWER", format!("echo '{{\"pr_id\":\"{feature}\"}}' > /service/code_review/{feature}; cat /service/result")).await?;
        if !response(&reply)?["success"]
            .as_bool()
            .ok_or("review outcome")?
        {
            match schedule {
                ReviewSchedule::FinishFirst => requests.push_front(feature),
                ReviewSchedule::Alternate => requests.push_back(feature),
                ReviewSchedule::BothRejectedThenFirst => {
                    if before
                        .iter()
                        .filter(|event| event["tool"] == "code_review" && event["success"] == false)
                        .count()
                        < 2
                    {
                        requests.push_back(feature);
                    } else {
                        requests.push_front(feature);
                    }
                }
                ReviewSchedule::AlternateThenRepeat => {
                    if before
                        .iter()
                        .any(|event| event["tool"] == "code_review" && event["success"] == false)
                    {
                        requests.push_front(feature);
                    } else {
                        requests.push_back(feature);
                    }
                }
            }
            continue;
        }
        let before = simulator.observed_events().await?;
        let duplicate = guest
            .command(
                "REVIEWER",
                format!("echo '{{\"pr_id\":\"{feature}\"}}' > /service/code_review/{feature}"),
            )
            .await?;
        assert!(
            duplicate["stdout"]
                .as_str()
                .ok_or("duplicate review stdout")?
                .contains("os error 8")
        );
        assert_eq!(
            count(
                &simulator.observed_events().await?,
                "REVIEWER",
                "code_review"
            ),
            count(&before, "REVIEWER", "code_review")
        );
    }
    assert_eq!(
        guest.command("REVIEWER", "exit 0".into()).await?["exit_status"],
        0
    );
    Ok(())
}

fn ordinary_cases() -> Vec<ReviewCase> {
    let mut cases = Vec::new();
    for depth in [0, 1, 2] {
        for first in ["user_authentication", "notification_feature"] {
            for schedule in [
                ReviewSchedule::FinishFirst,
                ReviewSchedule::Alternate,
                ReviewSchedule::AlternateThenRepeat,
            ] {
                let delays: &[f64] = if (depth == 0
                    && matches!(schedule, ReviewSchedule::FinishFirst))
                    || (depth == 1 && matches!(schedule, ReviewSchedule::Alternate))
                    || (depth == 2 && !matches!(schedule, ReviewSchedule::FinishFirst))
                {
                    &[0.01, 1.0]
                } else {
                    &[0.01]
                };
                for &delay in delays {
                    cases.push(ReviewCase {
                        depth,
                        first,
                        schedule,
                        delay,
                        extra_failures: &[],
                    });
                }
            }
        }
    }
    cases
}

fn repeated_rejection_cases() -> Vec<ReviewCase> {
    let mut cases = Vec::new();
    for first in ["user_authentication", "notification_feature"] {
        for schedule in [
            ReviewSchedule::BothRejectedThenFirst,
            ReviewSchedule::AlternateThenRepeat,
        ] {
            for extra_failures in [&[2][..], &[2, 3][..]] {
                cases.push(ReviewCase {
                    depth: 2,
                    first,
                    schedule,
                    delay: 0.01,
                    extra_failures,
                });
            }
        }
    }
    cases
}

// The input builders above remain the sole definitions of the two domains.
// Individual Rust tests give each input a stable selection and evidence name.
macro_rules! review_tests {
    ($cases:ident, $indices:ident; $($name:ident = $index:literal),+ $(,)?) => {
        const $indices: &[(&str, usize)] = &[$((stringify!($name), $index)),+];
        $(
            #[hibana_runtime::main]
            #[test]
            #[ignore = "requires the qualified WASI runtime, wash, auditor and original simulator"]
            async fn $name() -> Result<()> {
                let case = $cases().into_iter().nth($index).ok_or("missing review case")?;
                run_cases(vec![case]).await
            }
        )+
    };
}

review_tests!(ordinary_cases, ORDINARY_TESTS;
    ordinary_00 = 0, ordinary_01 = 1, ordinary_02 = 2, ordinary_03 = 3,
    ordinary_04 = 4, ordinary_05 = 5, ordinary_06 = 6, ordinary_07 = 7,
    ordinary_08 = 8, ordinary_09 = 9, ordinary_10 = 10, ordinary_11 = 11,
    ordinary_12 = 12, ordinary_13 = 13, ordinary_14 = 14, ordinary_15 = 15,
    ordinary_16 = 16, ordinary_17 = 17, ordinary_18 = 18, ordinary_19 = 19,
    ordinary_20 = 20, ordinary_21 = 21, ordinary_22 = 22, ordinary_23 = 23,
    ordinary_24 = 24, ordinary_25 = 25,
);
review_tests!(repeated_rejection_cases, REJECTION_TESTS;
    repeated_00 = 0, repeated_01 = 1, repeated_02 = 2, repeated_03 = 3,
    repeated_04 = 4, repeated_05 = 5, repeated_06 = 6, repeated_07 = 7,
);

#[test]
fn every_declared_review_input_has_exactly_one_test() {
    let mut manifest = Vec::new();
    for (tests, cases) in [
        (ORDINARY_TESTS, ordinary_cases()),
        (REJECTION_TESTS, repeated_rejection_cases()),
    ] {
        let mut indices = tests.iter().map(|(_, index)| *index).collect::<Vec<_>>();
        indices.sort_unstable();
        assert_eq!(indices, (0..cases.len()).collect::<Vec<_>>());
        for &(name, index) in tests {
            let case = &cases[index];
            manifest.push(json!({"test":format!("review_boundary::{name}"),
                "depth":case.depth,"review_first":case.first,
                "schedule":format!("{:?}",case.schedule),"delay_multiplier":case.delay,
                "failure_overrides":case.extra_failures.iter().map(|&at| ("code_review",at)).collect::<Vec<_>>()}));
        }
    }
    assert_eq!(ordinary_cases().len(), 26);
    assert_eq!(repeated_rejection_cases().len(), 8);
    println!("\n{}", json!({"review_cases":manifest}));
}

struct ReviewCase {
    depth: u8,
    first: &'static str,
    schedule: ReviewSchedule,
    delay: f64,
    extra_failures: &'static [u32],
}

#[hibana_runtime::scope]
async fn run_cases(cases: Vec<ReviewCase>) -> Result<()> {
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    )?;
    let root = std::path::PathBuf::from(
        std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("review evidence root missing")?,
    );
    fs::create_dir_all(&root)?;
    let directory = root.join(format!(
        "review-rust-boundary-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir(&directory)?;
    let runtime = std::path::PathBuf::from(
        std::env::var_os("CHOREOFIX_REVIEW_WASI_BINARY").ok_or("review binary missing")?,
    );
    let receipt_path = std::path::PathBuf::from(
        std::env::var_os("CHOREOFIX_REVIEW_BUILD_RECEIPT").ok_or("review build receipt missing")?,
    );
    let receipt = archive_build_receipt(&runtime, &receipt_path, &directory)?;
    let auditor = std::path::PathBuf::from(
        std::env::var_os("CHOREOFIX_AUDITOR").ok_or("qualified auditor missing")?,
    )
    .canonicalize()?;
    save(
        &directory.join("auditor.json"),
        &json!({"binary_sha256":digest(&auditor)?}),
    )?;
    let module = Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let mut outcomes = Vec::new();
    for ReviewCase {
        depth,
        first: review_first,
        schedule,
        delay,
        extra_failures,
    } in cases
    {
        let failures: Vec<_> = extra_failures
            .iter()
            .map(|&at| ("code_review", at))
            .collect();
        let rejected = usize::from(depth) + extra_failures.len();
        let case = directory.join(format!(
            "depth{depth}-{review_first}-{schedule:?}-delay{delay}-extra{extra_failures:?}"
        ));
        fs::create_dir(&case)?;
        let projection = crate::projection::archive(&runtime, &case, "6E")?;
        save(
            &case.join("case.json"),
            &json!({"task":"6E","depth":depth,"review_first":review_first,"schedule":format!("{schedule:?}"),"delay_multiplier":delay,
                "kind":"deterministic_rust_wasi_regression","model_calls":0,"scored_benchmark":false,"failure_overrides":failures,
                "test_barriers":"none",
                "build":receipt,
                "runtime_binary_sha256":digest(&runtime)?,"guest_wasm_sha256":digest(module)?,
                "compiled_projection_sha256":digest(&case.join("projection.json"))?,
                "harness_binary_sha256":digest(&std::env::current_exe()?)?,
                "harness_source_sha256":digest(Path::new("tools/runner/src/review_boundary.rs"))?,
                "audit_source_sha256":digest(Path::new("tools/runner/src/review_audit.rs"))?,
                "root_lock_sha256":digest(Path::new("Cargo.lock"))?,"runner_lock_sha256":digest(Path::new("tools/runner/Cargo.lock"))?,
                "simulator_source_sha256":digest(Path::new("benchmark/environments/6E/sim.py"))?}),
        )?;
        assert_eq!(
            projection["runtime_binary_sha256"],
            receipt["binary_sha256"]
        );
        let trace = Trace::create(&case.join("events.jsonl"))?;
        let (simulator, _) = oracle::Simulator::new("6E", depth, delay, trace.clone(), &failures)?;
        let (guest, runtime_loop) =
            process::guest(&runtime, module, &["6E"], &case, trace, simulator.clone())?;
        let (a, b, result) = hibana_runtime::race(
            async {
                async {
                    reviewer(&guest, &simulator, review_first, schedule).await?;
                };

                let (a, b, result) = (async {
                    async {
                        (developer(
                            &guest,
                            &simulator,
                            "DEVELOPER_A",
                            "user_authentication",
                            "dev_a_feature",
                        ))
                        .await
                    };
                    async {
                        (developer(
                            &guest,
                            &simulator,
                            "DEVELOPER_B",
                            "notification_feature",
                            "dev_b_feature",
                        ))
                        .await
                    };
                    async { (runtime_loop).await };
                })
                .await?;
                Ok((a, b, result))
            },
            hibana_runtime::host::deadline(Duration::from_secs(60)),
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
        assert_eq!(
            result["score"]["events"]
                .as_array()
                .ok_or("original events")?
                .len(),
            6 + 2 * rejected
        );
        assert_eq!(
            a.iter()
                .chain(&b)
                .filter(|reply| reply["success"] == false)
                .count(),
            rejected
        );
        for feedback in [&a, &b] {
            assert_eq!(
                feedback.last().ok_or("missing review feedback")?["success"],
                true
            );
        }
        if matches!(schedule, ReviewSchedule::FinishFirst) {
            for (feedback, expected) in [
                (
                    &a,
                    if review_first == "user_authentication" {
                        depth
                    } else {
                        0
                    },
                ),
                (
                    &b,
                    if review_first == "notification_feature" {
                        depth
                    } else {
                        0
                    },
                ),
            ] {
                let expected: Vec<_> = std::iter::repeat_n(json!(false), usize::from(expected))
                    .chain([json!(true)])
                    .collect();
                assert_eq!(
                    feedback
                        .iter()
                        .map(|r| r["success"].clone())
                        .collect::<Vec<_>>(),
                    expected
                );
            }
        }
        // The checked source search owns large temporary buffers. End that
        // ownership with the worker process instead of retaining allocator
        // pages across the entire WASI matrix. The audit and inputs are the
        // same as the normal CLI; no observation or test case is omitted.
        let status = std::process::Command::new(&auditor)
            .arg("--check-review-mutants")
            .arg(case.join("events.jsonl"))
            .arg(case.join("resource-mutants"))
            .stdout(std::process::Stdio::null())
            .stderr(fs::File::create(case.join("auditor.log"))?)
            .status()?;
        if !status.success() {
            return Err(format!("review audit failed ({status}); see {}", case.display()).into());
        }
        let mutants: Value =
            serde_json::from_slice(&fs::read(case.join("resource-mutants/summary.json"))?)?;
        assert_eq!(mutants["status"], "passed");
        assert_eq!(mutants["count"], if rejected == 0 { 27 } else { 30 });
        let audit = &mutants["audit"];
        assert_eq!(audit["status"], "passed");
        assert_eq!(audit["dynamic_protocol"]["status"], "passed");
        assert_eq!(
            audit["source_events_sha256"],
            digest(&case.join("events.jsonl"))?
        );
        if !extra_failures.is_empty() {
            let other = if review_first == "user_authentication" {
                "notification_feature"
            } else {
                "user_authentication"
            };
            let expected: Vec<_> = match schedule {
                ReviewSchedule::BothRejectedThenFirst => [review_first, other]
                    .into_iter()
                    .chain(std::iter::repeat_n(review_first, rejected - 1))
                    .chain([other])
                    .collect(),
                ReviewSchedule::AlternateThenRepeat => std::iter::once(review_first)
                    .chain(std::iter::repeat_n(other, rejected))
                    .chain([review_first])
                    .collect(),
                ReviewSchedule::FinishFirst | ReviewSchedule::Alternate => unreachable!(),
            };
            assert_eq!(audit["review_targets"], json!(expected));
        }
        if depth == 2
            && extra_failures.is_empty()
            && !matches!(schedule, ReviewSchedule::FinishFirst)
        {
            let other = if review_first == "user_authentication" {
                "notification_feature"
            } else {
                "user_authentication"
            };
            let expected = match schedule {
                ReviewSchedule::Alternate => {
                    json!([review_first, other, review_first, other])
                }
                ReviewSchedule::AlternateThenRepeat => {
                    json!([review_first, other, other, review_first])
                }
                _ => unreachable!(),
            };
            assert_eq!(audit["review_targets"], expected);
        }
        save(&case.join("resource-audit.json"), audit)?;
        if delay == 1.0 {
            for pair in ["code_review+implement_feature", "code_review+merge_code"] {
                assert!(
                    audit["overlapping_effect_pairs"][pair]
                        .as_u64()
                        .is_some_and(|n| n > 0),
                    "missing actual overlap: {pair}"
                );
            }
            if depth == 2 && matches!(schedule, ReviewSchedule::Alternate) {
                assert!(
                    audit["overlapping_effect_pairs"]["code_review+implement_feature"]
                        .as_u64()
                        .is_some_and(|n| n >= 3),
                    "both-rejected revision and re-review did not overlap"
                );
            }
        }
        let result = json!({"depth":depth,"review_first":review_first,"schedule":format!("{schedule:?}"),"delay_multiplier":delay,"result":result,"a_feedback":a,"b_feedback":b,"audit":audit,"mutant_count":mutants["count"],
                "kind":"deterministic_rust_wasi_regression","model_calls":0,"scored_benchmark":false});
        save(&case.join("result.json"), &result)?;
        outcomes.push(result);
        save(&directory.join("summary.json"), &json!(outcomes))?;
    }
    println!("{}", directory.display());
    Ok(())
}
