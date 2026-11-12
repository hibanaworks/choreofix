//! Whole 6M Endpoint executions with controlled domain results, not Wasm or AI.
#[path = "../../../src/shell/mod.rs"]
pub mod shell;
#[path = "../../../src/imp/transport.rs"]
pub(crate) mod transport;
pub(crate) mod imp {
    pub(crate) use crate::{projection, transport};
}
#[path = "../../../src/tasks/feature_delivery/mod.rs"]
pub mod feature_delivery;
use core::cell::Cell;
use feature_delivery::{
    global::{delivery::*, *},
    imp as data, localside,
};
use hibana::{
    g::Message,
    runtime::{SessionKitStorage, ids::SessionId, program::project, resolver::ResolverRef},
};
use hibana_runtime::host::deadline;
use serde_json::{Value, json};
use shell::{
    ShellError, ShellResult,
    imp::{repository_arguments::RepositoryArguments, review_reply::ReviewReply, service::Domain},
};

// This fixture owns actual domain observations and immutable failure inputs.
// No participant consults it to authorize an Endpoint operation.
struct Effects {
    events: Cell<Vec<Value>>,
    initial_review_failures: [usize; 2],
    recovery_review_failures: [usize; 2],
    ci_failures: [usize; 2],
    corruption: Option<(usize, usize)>,
}
impl Domain for Effects {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        let mut events = self.events.take();
        let occurrence = events.len() as u64;
        events.push(json!({"kind":"start","agent":agent,"tool":tool,"arguments":arguments}));
        self.events.set(events);
        for _ in 0..3 {
            hibana_runtime::yield_now().await;
        }
        let mut events = self.events.take();
        let feature = match tool {
            "implement_feature" => arguments["feature_name"].as_str(),
            "write_tests" => arguments["target"].as_str(),
            "code_review" => arguments["pr_id"].as_str(),
            "merge_code" => arguments["branch"].as_str(),
            "run_tests" => match arguments["suite"].as_str() {
                Some(SUITE_A) => Some(FEATURE_A),
                Some(SUITE_B) => Some(FEATURE_B),
                _ => None,
            },
            _ => None,
        }
        .expect("fixture operation has a known target");
        let index = match feature {
            FEATURE_A => 0,
            FEATURE_B => 1,
            _ => panic!("unknown fixture feature"),
        };
        let mut result = match tool {
            "implement_feature" => {
                let source = arguments["source"].as_str().unwrap();
                json!({"tool_name":tool,"success":true,"data":{"agent":agent,"feature":feature,"source":source,
                    "candidate_sha256":format!("fixture-{feature}-{source}")}})
            }
            "write_tests" | "code_review" | "merge_code" => {
                let candidate = &events
                    .iter()
                    .rev()
                    .find(|row| {
                        row["kind"] == "complete"
                            && row["tool"] == "implement_feature"
                            && row["result"]["data"]["feature"] == feature
                    })
                    .expect("the source must deliver an actual completed candidate")["result"];
                let source_receipt = candidate["data"]["artifact_id"]
                    .as_str()
                    .unwrap()
                    .strip_prefix("development:")
                    .unwrap()
                    .parse::<u64>()
                    .unwrap();
                if tool == "write_tests" {
                    json!({"tool_name":tool,"success":true,"data":{"agent":agent,"component":feature,
                    "source_receipt":source_receipt,"candidate_sha256":candidate["data"]["candidate_sha256"],
                    "test_source":{"target":feature,"expected_candidate":candidate["data"]["candidate_sha256"]},
                    "test_sha256":format!("fixture-tests-{feature}-{source_receipt}")}})
                } else if tool == "code_review" {
                    let failed_ci = events.iter().position(|row| {
                        row["kind"] == "complete"
                            && row["tool"] == "run_tests"
                            && row["result"]["success"] == false
                            && row["result"]["data"]["feature"] == feature
                    });
                    let already_rejected = events
                        .iter()
                        .enumerate()
                        .filter(|(at, row)| {
                            row["kind"] == "complete"
                                && row["tool"] == "code_review"
                                && row["result"]["success"] == false
                                && row["result"]["data"]["target"] == feature
                                && failed_ci.is_none_or(|start| *at > start)
                        })
                        .count();
                    let required = if failed_ci.is_some() {
                        self.recovery_review_failures[index]
                    } else {
                        self.initial_review_failures[index]
                    };
                    json!({"tool_name":tool,"success":already_rejected>=required,"data":{"agent":agent,"target":feature,
                        "candidate_sha256":candidate["data"]["candidate_sha256"],"source_receipt":source_receipt}})
                } else {
                    let review = &events
                        .iter()
                        .rev()
                        .find(|row| {
                            row["kind"] == "complete"
                                && row["tool"] == "code_review"
                                && row["result"]["success"] == true
                                && row["result"]["data"]["target"] == feature
                                && row["result"]["data"]["source_receipt"] == source_receipt
                        })
                        .expect("merge must have this source's actual approval")["result"];
                    let previous = events
                        .iter()
                        .rev()
                        .find(|row| row["kind"] == "complete" && row["tool"] == "merge_code")
                        .map(|row| &row["result"]);
                    let mut published = previous
                        .map_or(json!({}), |row| row["data"]["published_candidates"].clone());
                    let mut publication_events =
                        previous.map_or(json!({}), |row| row["data"]["publication_events"].clone());
                    let base = previous.map(|row| row["data"]["repository_sha256"].clone());
                    let validation = events
                        .iter()
                        .rev()
                        .find(|row| {
                            row["kind"] == "complete"
                                && row["tool"] == "run_tests"
                                && base.as_ref()
                                    == Some(&row["result"]["data"]["repository_sha256"])
                                && row["result"]["data"]["publication_events"] == publication_events
                        })
                        .map(|row| &row["result"]);
                    let receipt = validation.map(|row| {
                        row["data"]["artifact_id"]
                            .as_str()
                            .unwrap()
                            .strip_prefix("development:")
                            .unwrap()
                            .parse::<u64>()
                            .unwrap()
                    });
                    let base_events = publication_events.clone();
                    let replaced = publication_events[feature].clone();
                    published[feature] = candidate["data"]["candidate_sha256"].clone();
                    publication_events[feature] = json!(occurrence);
                    json!({"tool_name":tool,"success":true,"data":{"agent":agent,"feature":feature,"branch":feature,
                        "source":candidate["data"]["source"],"candidate_sha256":candidate["data"]["candidate_sha256"],
                        "source_receipt":source_receipt,"review_receipt":review["data"]["artifact_id"].as_str().unwrap().strip_prefix("development:").unwrap().parse::<u64>().unwrap(),
                        "publication_event":occurrence,"publication_events":publication_events,"published_candidates":published,
                        "repository_sha256":format!("fixture-repository-{published}"),
                        "base_repository_sha256":base,"base_publication_events":base_events,
                        "base_test_receipt":validation.filter(|row|row["success"]==true).and(receipt),
                        "base_failure_receipt":validation.filter(|row|row["success"]==false).and(receipt),
                        "repaired_publication_event":validation.filter(|row|row["success"]==false).map(|_|replaced)}})
                }
            }
            "run_tests" => {
                let publication = &events
                    .iter()
                    .rev()
                    .find(|row| row["kind"] == "complete" && row["tool"] == "merge_code")
                    .expect("CI needs a completed publication")["result"];
                let mut receipts = json!({});
                let mut inputs = json!({});
                for (target, identity) in publication["data"]["published_candidates"]
                    .as_object()
                    .unwrap()
                {
                    let tests = &events
                        .iter()
                        .rev()
                        .find(|row| {
                            row["kind"] == "complete"
                                && row["tool"] == "write_tests"
                                && row["result"]["data"]["component"] == *target
                                && row["result"]["data"]["candidate_sha256"] == *identity
                        })
                        .expect("CI must have actual completed test data")["result"];
                    receipts[target] = json!(
                        tests["data"]["artifact_id"]
                            .as_str()
                            .unwrap()
                            .strip_prefix("development:")
                            .unwrap()
                            .parse::<u64>()
                            .unwrap()
                    );
                    inputs[target] = tests["data"].clone();
                }
                let failures = events
                    .iter()
                    .filter(|row| {
                        row["kind"] == "complete"
                            && row["tool"] == "run_tests"
                            && row["result"]["success"] == false
                            && row["result"]["data"]["feature"] == feature
                    })
                    .count();
                json!({"tool_name":tool,"success":failures>=self.ci_failures[index],"data":{"agent":agent,"feature":feature,
                    "suite":arguments["suite"],"repository_sha256":publication["data"]["repository_sha256"],
                    "publication_event":publication["data"]["publication_event"],"publication_events":publication["data"]["publication_events"],
                    "published_candidates":publication["data"]["published_candidates"],"test_receipts":receipts,"test_inputs":inputs}})
            }
            _ => unreachable!(),
        };
        result["data"]["artifact_id"] = json!(format!("development:{occurrence}"));
        if let Some((target, trial)) = self.corruption
            && target == index
        {
            match (tool, trial) {
                ("merge_code", 6) => result["data"]["review_receipt"] = Value::Null,
                ("run_tests", 3) => result["data"]["repository_sha256"] = json!("stale repository"),
                ("run_tests", 4) => result["data"]["test_receipts"][feature] = json!(0),
                ("run_tests", 7) => {
                    result["data"]["published_candidates"]["unknown-feature"] = json!("unknown")
                }
                ("run_tests", 8) => {
                    result["data"]["test_inputs"][feature]["test_sha256"] =
                        json!("forged same-id definition")
                }
                ("run_tests", 9) => result["tool_name"] = json!("write_tests"),
                ("run_tests", 10) => result["data"]["artifact_id"] = json!("development:0"),
                _ => {}
            }
        }
        events.push(json!({"kind":"complete","agent":agent,"tool":tool,"arguments":arguments,"result":result}));
        self.events.set(events);
        Ok(result)
    }
}

fn decide_actual(
    actual: &Cell<Value>,
) -> Result<hibana::runtime::resolver::DecisionArm, hibana::runtime::resolver::ResolverError> {
    use hibana::runtime::resolver::{DecisionArm, ResolverError};
    let value = actual.take();
    let choice = match value["success"].as_bool() {
        Some(true) => Ok(DecisionArm::Left),
        Some(false) => Ok(DecisionArm::Right),
        None => Err(ResolverError::reject()),
    };
    actual.set(value);
    choice
}

#[path = "../../../src/imp/projection.rs"]
pub(crate) mod projection;

#[hibana_runtime::main]
#[test]
async fn feature_delivery_whole_execution_retries_and_returns_every_resource() -> ShellResult<()> {
    let (
        a_program,
        b_program,
        writer_program,
        reviewer_program,
        runner_program,
        repository_program,
        review_a_program,
        review_b_program,
        data_a_program,
        data_b_program,
        ci_program,
        repo_program,
        environment_program,
        build_program,
        feature_a_program,
        feature_b_program,
    ) = {
        let source = delivery::protocol();
        (
            project::<DEVELOPER_A>(&source),
            project::<DEVELOPER_B>(&source),
            project::<TEST_WRITER>(&source),
            project::<REVIEWER>(&source),
            project::<CI_RUNNER>(&source),
            project::<REPOSITORY>(&source),
            project::<REVIEW_A>(&source),
            project::<REVIEW_B>(&source),
            project::<TEST_DATA_A>(&source),
            project::<TEST_DATA_B>(&source),
            project::<CI_SERVICE>(&source),
            project::<REPO>(&source),
            project::<TEST_ENV>(&source),
            project::<BUILD_SERVER>(&source),
            project::<AUTH_A>(&source),
            project::<AUTH_B>(&source),
        )
    };
    let evidence = std::env::var_os("CHOREOFIX_NATIVE_FEATURE_DELIVERY_EVIDENCE")
        .map(std::path::PathBuf::from);
    if let Some(directory) = &evidence {
        let artifact = json!({"compiled":{"programs":[projection::rows(&a_program),projection::rows(&b_program),projection::rows(&writer_program),projection::rows(&reviewer_program),projection::rows(&runner_program),projection::rows(&repository_program),projection::rows(&review_a_program),projection::rows(&review_b_program),projection::rows(&data_a_program),projection::rows(&data_b_program),projection::rows(&ci_program),projection::rows(&repo_program),projection::rows(&environment_program),projection::rows(&build_program),projection::rows(&feature_a_program),projection::rows(&feature_b_program)]}});

        std::fs::create_dir_all(directory).map_err(|e| ShellError::message(e.to_string()))?;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(directory.join("projection.json"))
            .map_err(|e| ShellError::message(e.to_string()))?;
        serde_json::to_writer(&mut file, &artifact)
            .map_err(|e| ShellError::message(e.to_string()))?;
    }
    let mut cases = Vec::new();
    for review_b_first in [false, true] {
        for tests_b_first in [false, true] {
            for (initial, recovery, ci) in [
                ([0, 0], [0, 0], [0, 0]),
                ([1, 1], [0, 0], [0, 0]),
                ([0, 0], [1, 1], [1, 1]),
                ([2, 2], [2, 2], [2, 2]),
                ([3, 1], [2, 3], [4, 2]),
            ] {
                cases.push((review_b_first, tests_b_first, initial, recovery, ci, None));
            }
        }
        for trial in 2..=10 {
            cases.push((review_b_first, false, [0, 0], [0, 0], [0, 0], Some(trial)));
        }
    }
    assert_eq!(cases.len(), 38);
    for (
        case,
        (
            review_b_first,
            tests_b_first,
            initial_review_failures,
            recovery_review_failures,
            ci_failures,
            corruption,
        ),
    ) in cases.into_iter().enumerate()
    {
        let sid = SessionId::new(SESSION);
        // SAFETY: this test owns one fresh physical fixture and five distinct resources.
        #[allow(unsafe_code)]
        let transport = transport::InMemoryTransport::new(unsafe {
            [
                (
                    sid,
                    REPO,
                    hibana::runtime::transport::EndpointBinding::new(),
                ),
                (
                    sid,
                    TEST_ENV,
                    hibana::runtime::transport::EndpointBinding::new(),
                ),
                (
                    sid,
                    BUILD_SERVER,
                    hibana::runtime::transport::EndpointBinding::new(),
                ),
                (
                    sid,
                    AUTH_A,
                    hibana::runtime::transport::EndpointBinding::new(),
                ),
                (
                    sid,
                    AUTH_B,
                    hibana::runtime::transport::EndpointBinding::new(),
                ),
            ]
        });
        let effects = &Effects {
            events: Cell::new(Vec::new()),
            initial_review_failures,
            recovery_review_failures,
            ci_failures,
            corruption: corruption.map(|trial| (usize::from(review_b_first), trial)),
        };
        let initial_a = Cell::new(Value::Null);
        let initial_b = Cell::new(Value::Null);
        let recovery_a = Cell::new(Value::Null);
        let recovery_b = Cell::new(Value::Null);
        let actual_ci_a = Cell::new(Value::Null);
        let actual_ci_b = Cell::new(Value::Null);
        let records = Cell::new(Vec::new());
        let observer = |row| {
            let mut output = records.take();
            output.push(projection::record(row));
            records.set(output);
        };
        let mut slab = vec![0; 1024 * 1024];
        let mut storage = SessionKitStorage::uninit();
        let rv = storage.init().rendezvous(&mut slab, &transport)?;
        rv.observe(&observer);
        let mut a = rv.enter(sid, &a_program)?;
        let mut b = rv.enter(sid, &b_program)?;
        let mut writer = rv.enter(sid, &writer_program)?;
        let mut reviewer = rv.enter(sid, &reviewer_program)?;
        let mut runner = rv.enter(sid, &runner_program)?;
        let mut repository = rv.enter(sid, &repository_program)?;
        let mut review_a = rv.enter(sid, &review_a_program)?;
        let mut review_b = rv.enter(sid, &review_b_program)?;
        let mut data_a = rv.enter(sid, &data_a_program)?;
        let mut data_b = rv.enter(sid, &data_b_program)?;
        let mut ci = rv.enter(sid, &ci_program)?;
        let repo = rv.enter(sid, &repo_program)?;
        let environment = rv.enter(sid, &environment_program)?;
        let build = rv.enter(sid, &build_program)?;
        let feature_a = rv.enter(sid, &feature_a_program)?;
        let feature_b = rv.enter(sid, &feature_b_program)?;
        rv.set_resolver(
            &reviewer_program,
            ResolverRef::<REVIEW_DECISION_A>::decision_state(&initial_a, decide_actual),
        )
        .unwrap();
        rv.set_resolver(
            &reviewer_program,
            ResolverRef::<REVIEW_DECISION_B>::decision_state(&initial_b, decide_actual),
        )
        .unwrap();
        rv.set_resolver(
            &a_program,
            ResolverRef::<RECOVERY_REVIEW_A>::decision_state(&recovery_a, decide_actual),
        )
        .unwrap();
        rv.set_resolver(
            &b_program,
            ResolverRef::<RECOVERY_REVIEW_B>::decision_state(&recovery_b, decide_actual),
        )
        .unwrap();
        rv.set_resolver(
            &runner_program,
            ResolverRef::<CI_DECISION_A>::decision_state(&actual_ci_a, decide_actual),
        )
        .unwrap();
        rv.set_resolver(
            &runner_program,
            ResolverRef::<CI_DECISION_B>::decision_state(&actual_ci_b, decide_actual),
        )
        .unwrap();
        let result=hibana_runtime::race(async{
 async{let _resources=localside::repository::run(&mut repository,(repo,environment,feature_a,feature_b),effects).await?;Ok::<_,ShellError>(())};
 async{localside::reviews::feature_a(&mut review_a,effects).await};
 async{localside::reviews::feature_b(&mut review_b,effects).await};
 async{localside::tests::feature_a(&mut data_a,effects).await};
 async{localside::tests::feature_b(&mut data_b,effects).await};
 async{localside::ci::run(&mut ci,build,effects).await};
 async {
let mut feature = a.recv::<Feature<AUTH_A>>().await?;
let branch = a.offer().await?;
let selected = match branch.label() {
    ReviewFirstA::LOGICAL_LABEL => branch.recv::<ReviewFirstA>().await?.into_value(),
    ReviewFirstB::LOGICAL_LABEL => branch.recv::<ReviewFirstB>().await?.into_value(),
    _ => return Err(ShellError::message("missing actual first review")),
};
let mut peer;
if selected["pr_id"]==FEATURE_B {
    peer = data::decode(a.recv::<InterfaceChangedB>().await?)?;
    data::candidate(&peer, FEATURE_B)?;
} else { peer = Value::Null; }
let arguments = RepositoryArguments::from_bytes(br#"{"feature_name":"user_authentication","source":"a fixture source"}"#)?;
a.send::<ImplementA>(&arguments).await?;
let mut repo = a.recv::<RepoLoan>().await?;
async {
    async { feature.send::<Implement>(&arguments).await?; };
    async { repo.send::<ImplementOnRepo>(&arguments).await?; };
    Ok::<_,ShellError>(())
}.await?;
let candidate = data::decode(repo.recv::<RepoResult>().await?)?;
data::candidate(&candidate, FEATURE_A)?;
a.send::<RepoReturn>(repo).await?;
let changed = data::encode(&candidate)?;
feature.send::<InterfaceChangedA>(&changed.as_slice()).await?;
a.send::<Feature<AUTH_A>>(feature).await?;
if selected["pr_id"]==FEATURE_A {
    peer = data::decode(a.recv::<InterfaceChangedB>().await?)?;
    data::candidate(&peer, FEATURE_B)?;
}
'initial_review: loop {
    let _tests = data::decode(a.recv::<TestedA>().await?)?;
    loop {
        let branch = a.offer().await?;
        match branch.label() {
            InterfaceChangedB::LOGICAL_LABEL => {
    peer = data::decode(branch.recv::<InterfaceChangedB>().await?)?;
    data::candidate(&peer, FEATURE_B)?;
}
ReviewApproved::LOGICAL_LABEL => { let result = data::decode(branch.recv::<ReviewApproved>().await?)?; data::reviewed(&result, &peer, FEATURE_B)?; }
ReviewRejected::LOGICAL_LABEL => { let result = data::decode(branch.recv::<ReviewRejected>().await?)?; data::reviewed(&result, &peer, FEATURE_B)?; }
RecoveryReviewApproved::LOGICAL_LABEL => { let result = data::decode(branch.recv::<RecoveryReviewApproved>().await?)?; data::reviewed(&result, &peer, FEATURE_B)?; }
RecoveryReviewRejected::LOGICAL_LABEL => { let result = data::decode(branch.recv::<RecoveryReviewRejected>().await?)?; data::reviewed(&result, &peer, FEATURE_B)?; }
            Approved::LOGICAL_LABEL => { branch.recv::<Approved>().await?; break 'initial_review; }
            Rejected::LOGICAL_LABEL => { branch.recv::<Rejected>().await?; break; }
            _ => return Err(ShellError::message("unexpected initial review continuation")),
        }
    }
    feature = a.recv::<Feature<AUTH_A>>().await?;
    let arguments = RepositoryArguments::from_bytes(br#"{"feature_name":"user_authentication","source":"a fixture source"}"#)?;
a.send::<ImplementA>(&arguments).await?;
let mut repo = a.recv::<RepoLoan>().await?;
async {
    async { feature.send::<Implement>(&arguments).await?; };
    async { repo.send::<ImplementOnRepo>(&arguments).await?; };
    Ok::<_,ShellError>(())
}.await?;
let candidate = data::decode(repo.recv::<RepoResult>().await?)?;
data::candidate(&candidate, FEATURE_A)?;
a.send::<RepoReturn>(repo).await?;
let changed = data::encode(&candidate)?;
feature.send::<InterfaceChangedA>(&changed.as_slice()).await?;
    a.send::<Feature<AUTH_A>>(feature).await?;
}
feature = loop {
    let branch = a.offer().await?;
    match branch.label() {
        InterfaceChangedB::LOGICAL_LABEL => {
    peer = data::decode(branch.recv::<InterfaceChangedB>().await?)?;
    data::candidate(&peer, FEATURE_B)?;
}
ReviewApproved::LOGICAL_LABEL => { let result = data::decode(branch.recv::<ReviewApproved>().await?)?; data::reviewed(&result, &peer, FEATURE_B)?; }
ReviewRejected::LOGICAL_LABEL => { let result = data::decode(branch.recv::<ReviewRejected>().await?)?; data::reviewed(&result, &peer, FEATURE_B)?; }
RecoveryReviewApproved::LOGICAL_LABEL => { let result = data::decode(branch.recv::<RecoveryReviewApproved>().await?)?; data::reviewed(&result, &peer, FEATURE_B)?; }
RecoveryReviewRejected::LOGICAL_LABEL => { let result = data::decode(branch.recv::<RecoveryReviewRejected>().await?)?; data::reviewed(&result, &peer, FEATURE_B)?; }
        CiPassedB::LOGICAL_LABEL => { branch.recv::<CiPassedB>().await?; }
        CiFailedB::LOGICAL_LABEL => { branch.recv::<CiFailedB>().await?; }
        MergeLoanA::LOGICAL_LABEL => break branch.recv::<MergeLoanA>().await?,
        _ => return Err(ShellError::message("unexpected merge continuation")),
    }
};
'merged: loop {
    let args = RepositoryArguments::from_bytes(br#"{"branch":"user_authentication"}"#)?;
    a.send::<MergeAdmissionA>(&args).await?;
    let mut repo = a.recv::<RepoLoan>().await?;
    async {
        async { feature.send::<Merge>(&args).await?; };
        async { repo.send::<MergeOnRepo>(&args).await?; };
        Ok::<_,ShellError>(())
    }.await?;
    let publication = feature.recv::<Merged>().await?.to_vec();
    a.send::<RepoReturn>(repo).await?;
    a.send::<Merged>(&publication.as_slice()).await?;
    a.send::<Feature<AUTH_A>>(feature).await?;
    loop {
        let branch = a.offer().await?;
        match branch.label() {
            InterfaceChangedB::LOGICAL_LABEL => {
    peer = data::decode(branch.recv::<InterfaceChangedB>().await?)?;
    data::candidate(&peer, FEATURE_B)?;
}
ReviewApproved::LOGICAL_LABEL => { let result = data::decode(branch.recv::<ReviewApproved>().await?)?; data::reviewed(&result, &peer, FEATURE_B)?; }
ReviewRejected::LOGICAL_LABEL => { let result = data::decode(branch.recv::<ReviewRejected>().await?)?; data::reviewed(&result, &peer, FEATURE_B)?; }
RecoveryReviewApproved::LOGICAL_LABEL => { let result = data::decode(branch.recv::<RecoveryReviewApproved>().await?)?; data::reviewed(&result, &peer, FEATURE_B)?; }
RecoveryReviewRejected::LOGICAL_LABEL => { let result = data::decode(branch.recv::<RecoveryReviewRejected>().await?)?; data::reviewed(&result, &peer, FEATURE_B)?; }
            CiPassedA::LOGICAL_LABEL => { branch.recv::<CiPassedA>().await?; break 'merged; }
            LastCiPassedA::LOGICAL_LABEL => { branch.recv::<LastCiPassedA>().await?; return Ok::<_,ShellError>(()); }
            CiFailedA::LOGICAL_LABEL => { branch.recv::<CiFailedA>().await?; break; }
            _ => return Err(ShellError::message("unexpected own CI outcome")),
        }
    }
    feature = a.recv::<Feature<AUTH_A>>().await?;
    loop {
        let arguments = RepositoryArguments::from_bytes(br#"{"feature_name":"user_authentication","source":"a fixture source"}"#)?;
a.send::<ImplementA>(&arguments).await?;
let mut repo = a.recv::<RepoLoan>().await?;
async {
    async { feature.send::<Implement>(&arguments).await?; };
    async { repo.send::<ImplementOnRepo>(&arguments).await?; };
    Ok::<_,ShellError>(())
}.await?;
let candidate = data::decode(repo.recv::<RepoResult>().await?)?;
data::candidate(&candidate, FEATURE_A)?;
a.send::<RepoReturn>(repo).await?;
let changed = data::encode(&candidate)?;
feature.send::<InterfaceChangedA>(&changed.as_slice()).await?;
        a.recv::<TestedA>().await?;
        let args = RepositoryArguments::from_bytes(br#"{"pr_id":"user_authentication"}"#)?;
        feature.send::<Review>(&args).await?;
        let result = data::decode(feature.recv::<ReviewResultA>().await?)?;
        recovery_a.set(result.clone());
        let bytes = data::encode(&result)?;
        if result["success"]==true { let reply = ReviewReply::<true>::from_bytes(&bytes)?;
feature.send::<RecoveryApproved>(&reply).await?;
feature.send::<RecoveryReviewApproved>(&reply).await?;
feature.send::<RecoveryReviewApproved>(&reply).await?;
feature.send::<RecoveryReviewApproved>(&reply).await?;
feature.send::<RecoveryReviewApproved>(&reply).await?;
feature.send::<RecoveryReviewApproved>(&reply).await?;
feature.send::<RecoveryReviewApproved>(&reply).await?; break; }
        else { let reply = ReviewReply::<false>::from_bytes(&bytes)?;
feature.send::<RecoveryRejected>(&reply).await?;
feature.send::<RecoveryReviewRejected>(&reply).await?;
feature.send::<RecoveryReviewRejected>(&reply).await?;
feature.send::<RecoveryReviewRejected>(&reply).await?;
feature.send::<RecoveryReviewRejected>(&reply).await?;
feature.send::<RecoveryReviewRejected>(&reply).await?;
feature.send::<RecoveryReviewRejected>(&reply).await?; }
    }
}
loop {
    let branch = a.offer().await?;
    match branch.label() {
        InterfaceChangedB::LOGICAL_LABEL => {
    peer = data::decode(branch.recv::<InterfaceChangedB>().await?)?;
    data::candidate(&peer, FEATURE_B)?;
}
ReviewApproved::LOGICAL_LABEL => { let result = data::decode(branch.recv::<ReviewApproved>().await?)?; data::reviewed(&result, &peer, FEATURE_B)?; }
ReviewRejected::LOGICAL_LABEL => { let result = data::decode(branch.recv::<ReviewRejected>().await?)?; data::reviewed(&result, &peer, FEATURE_B)?; }
RecoveryReviewApproved::LOGICAL_LABEL => { let result = data::decode(branch.recv::<RecoveryReviewApproved>().await?)?; data::reviewed(&result, &peer, FEATURE_B)?; }
RecoveryReviewRejected::LOGICAL_LABEL => { let result = data::decode(branch.recv::<RecoveryReviewRejected>().await?)?; data::reviewed(&result, &peer, FEATURE_B)?; }
        CiFailedB::LOGICAL_LABEL => { branch.recv::<CiFailedB>().await?; }
        LastCiPassedB::LOGICAL_LABEL => { branch.recv::<LastCiPassedB>().await?; return Ok::<_,ShellError>(()); }
        _ => return Err(ShellError::message("unexpected final delivery continuation")),
    }
}
};
 async {
let mut feature = b.recv::<Feature<AUTH_B>>().await?;
let branch = b.offer().await?;
let selected = match branch.label() {
    ReviewFirstA::LOGICAL_LABEL => branch.recv::<ReviewFirstA>().await?.into_value(),
    ReviewFirstB::LOGICAL_LABEL => branch.recv::<ReviewFirstB>().await?.into_value(),
    _ => return Err(ShellError::message("missing actual first review")),
};
let mut peer;
if selected["pr_id"]==FEATURE_A {
    peer = data::decode(b.recv::<InterfaceChangedA>().await?)?;
    data::candidate(&peer, FEATURE_A)?;
} else { peer = Value::Null; }
let arguments = RepositoryArguments::from_bytes(br#"{"feature_name":"payment_processing","source":"b fixture source"}"#)?;
b.send::<ImplementB>(&arguments).await?;
let mut repo = b.recv::<RepoLoan>().await?;
async {
    async { feature.send::<Implement>(&arguments).await?; };
    async { repo.send::<ImplementOnRepo>(&arguments).await?; };
    Ok::<_,ShellError>(())
}.await?;
let candidate = data::decode(repo.recv::<RepoResult>().await?)?;
data::candidate(&candidate, FEATURE_B)?;
b.send::<RepoReturn>(repo).await?;
let changed = data::encode(&candidate)?;
feature.send::<InterfaceChangedB>(&changed.as_slice()).await?;
b.send::<Feature<AUTH_B>>(feature).await?;
if selected["pr_id"]==FEATURE_B {
    peer = data::decode(b.recv::<InterfaceChangedA>().await?)?;
    data::candidate(&peer, FEATURE_A)?;
}
'initial_review: loop {
    let _tests = data::decode(b.recv::<TestedB>().await?)?;
    loop {
        let branch = b.offer().await?;
        match branch.label() {
            InterfaceChangedA::LOGICAL_LABEL => {
    peer = data::decode(branch.recv::<InterfaceChangedA>().await?)?;
    data::candidate(&peer, FEATURE_A)?;
}
ReviewApproved::LOGICAL_LABEL => { let result = data::decode(branch.recv::<ReviewApproved>().await?)?; data::reviewed(&result, &peer, FEATURE_A)?; }
ReviewRejected::LOGICAL_LABEL => { let result = data::decode(branch.recv::<ReviewRejected>().await?)?; data::reviewed(&result, &peer, FEATURE_A)?; }
RecoveryReviewApproved::LOGICAL_LABEL => { let result = data::decode(branch.recv::<RecoveryReviewApproved>().await?)?; data::reviewed(&result, &peer, FEATURE_A)?; }
RecoveryReviewRejected::LOGICAL_LABEL => { let result = data::decode(branch.recv::<RecoveryReviewRejected>().await?)?; data::reviewed(&result, &peer, FEATURE_A)?; }
            Approved::LOGICAL_LABEL => { branch.recv::<Approved>().await?; break 'initial_review; }
            Rejected::LOGICAL_LABEL => { branch.recv::<Rejected>().await?; break; }
            _ => return Err(ShellError::message("unexpected initial review continuation")),
        }
    }
    feature = b.recv::<Feature<AUTH_B>>().await?;
    let arguments = RepositoryArguments::from_bytes(br#"{"feature_name":"payment_processing","source":"b fixture source"}"#)?;
b.send::<ImplementB>(&arguments).await?;
let mut repo = b.recv::<RepoLoan>().await?;
async {
    async { feature.send::<Implement>(&arguments).await?; };
    async { repo.send::<ImplementOnRepo>(&arguments).await?; };
    Ok::<_,ShellError>(())
}.await?;
let candidate = data::decode(repo.recv::<RepoResult>().await?)?;
data::candidate(&candidate, FEATURE_B)?;
b.send::<RepoReturn>(repo).await?;
let changed = data::encode(&candidate)?;
feature.send::<InterfaceChangedB>(&changed.as_slice()).await?;
    b.send::<Feature<AUTH_B>>(feature).await?;
}
feature = loop {
    let branch = b.offer().await?;
    match branch.label() {
        InterfaceChangedA::LOGICAL_LABEL => {
    peer = data::decode(branch.recv::<InterfaceChangedA>().await?)?;
    data::candidate(&peer, FEATURE_A)?;
}
ReviewApproved::LOGICAL_LABEL => { let result = data::decode(branch.recv::<ReviewApproved>().await?)?; data::reviewed(&result, &peer, FEATURE_A)?; }
ReviewRejected::LOGICAL_LABEL => { let result = data::decode(branch.recv::<ReviewRejected>().await?)?; data::reviewed(&result, &peer, FEATURE_A)?; }
RecoveryReviewApproved::LOGICAL_LABEL => { let result = data::decode(branch.recv::<RecoveryReviewApproved>().await?)?; data::reviewed(&result, &peer, FEATURE_A)?; }
RecoveryReviewRejected::LOGICAL_LABEL => { let result = data::decode(branch.recv::<RecoveryReviewRejected>().await?)?; data::reviewed(&result, &peer, FEATURE_A)?; }
        CiPassedA::LOGICAL_LABEL => { branch.recv::<CiPassedA>().await?; }
        CiFailedA::LOGICAL_LABEL => { branch.recv::<CiFailedA>().await?; }
        MergeLoanB::LOGICAL_LABEL => break branch.recv::<MergeLoanB>().await?,
        _ => return Err(ShellError::message("unexpected merge continuation")),
    }
};
'merged: loop {
    let args = RepositoryArguments::from_bytes(br#"{"branch":"payment_processing"}"#)?;
    b.send::<MergeAdmissionB>(&args).await?;
    let mut repo = b.recv::<RepoLoan>().await?;
    async {
        async { feature.send::<Merge>(&args).await?; };
        async { repo.send::<MergeOnRepo>(&args).await?; };
        Ok::<_,ShellError>(())
    }.await?;
    let publication = feature.recv::<Merged>().await?.to_vec();
    b.send::<RepoReturn>(repo).await?;
    b.send::<Merged>(&publication.as_slice()).await?;
    b.send::<Feature<AUTH_B>>(feature).await?;
    loop {
        let branch = b.offer().await?;
        match branch.label() {
            InterfaceChangedA::LOGICAL_LABEL => {
    peer = data::decode(branch.recv::<InterfaceChangedA>().await?)?;
    data::candidate(&peer, FEATURE_A)?;
}
ReviewApproved::LOGICAL_LABEL => { let result = data::decode(branch.recv::<ReviewApproved>().await?)?; data::reviewed(&result, &peer, FEATURE_A)?; }
ReviewRejected::LOGICAL_LABEL => { let result = data::decode(branch.recv::<ReviewRejected>().await?)?; data::reviewed(&result, &peer, FEATURE_A)?; }
RecoveryReviewApproved::LOGICAL_LABEL => { let result = data::decode(branch.recv::<RecoveryReviewApproved>().await?)?; data::reviewed(&result, &peer, FEATURE_A)?; }
RecoveryReviewRejected::LOGICAL_LABEL => { let result = data::decode(branch.recv::<RecoveryReviewRejected>().await?)?; data::reviewed(&result, &peer, FEATURE_A)?; }
            CiPassedB::LOGICAL_LABEL => { branch.recv::<CiPassedB>().await?; break 'merged; }
            LastCiPassedB::LOGICAL_LABEL => { branch.recv::<LastCiPassedB>().await?; return Ok::<_,ShellError>(()); }
            CiFailedB::LOGICAL_LABEL => { branch.recv::<CiFailedB>().await?; break; }
            _ => return Err(ShellError::message("unexpected own CI outcome")),
        }
    }
    feature = b.recv::<Feature<AUTH_B>>().await?;
    loop {
        let arguments = RepositoryArguments::from_bytes(br#"{"feature_name":"payment_processing","source":"b fixture source"}"#)?;
b.send::<ImplementB>(&arguments).await?;
let mut repo = b.recv::<RepoLoan>().await?;
async {
    async { feature.send::<Implement>(&arguments).await?; };
    async { repo.send::<ImplementOnRepo>(&arguments).await?; };
    Ok::<_,ShellError>(())
}.await?;
let candidate = data::decode(repo.recv::<RepoResult>().await?)?;
data::candidate(&candidate, FEATURE_B)?;
b.send::<RepoReturn>(repo).await?;
let changed = data::encode(&candidate)?;
feature.send::<InterfaceChangedB>(&changed.as_slice()).await?;
        b.recv::<TestedB>().await?;
        let args = RepositoryArguments::from_bytes(br#"{"pr_id":"payment_processing"}"#)?;
        feature.send::<Review>(&args).await?;
        let result = data::decode(feature.recv::<ReviewResultB>().await?)?;
        recovery_b.set(result.clone());
        let bytes = data::encode(&result)?;
        if result["success"]==true { let reply = ReviewReply::<true>::from_bytes(&bytes)?;
feature.send::<RecoveryApproved>(&reply).await?;
feature.send::<RecoveryReviewApproved>(&reply).await?;
feature.send::<RecoveryReviewApproved>(&reply).await?;
feature.send::<RecoveryReviewApproved>(&reply).await?;
feature.send::<RecoveryReviewApproved>(&reply).await?;
feature.send::<RecoveryReviewApproved>(&reply).await?;
feature.send::<RecoveryReviewApproved>(&reply).await?; break; }
        else { let reply = ReviewReply::<false>::from_bytes(&bytes)?;
feature.send::<RecoveryRejected>(&reply).await?;
feature.send::<RecoveryReviewRejected>(&reply).await?;
feature.send::<RecoveryReviewRejected>(&reply).await?;
feature.send::<RecoveryReviewRejected>(&reply).await?;
feature.send::<RecoveryReviewRejected>(&reply).await?;
feature.send::<RecoveryReviewRejected>(&reply).await?;
feature.send::<RecoveryReviewRejected>(&reply).await?; }
    }
}
loop {
    let branch = b.offer().await?;
    match branch.label() {
        InterfaceChangedA::LOGICAL_LABEL => {
    peer = data::decode(branch.recv::<InterfaceChangedA>().await?)?;
    data::candidate(&peer, FEATURE_A)?;
}
ReviewApproved::LOGICAL_LABEL => { let result = data::decode(branch.recv::<ReviewApproved>().await?)?; data::reviewed(&result, &peer, FEATURE_A)?; }
ReviewRejected::LOGICAL_LABEL => { let result = data::decode(branch.recv::<ReviewRejected>().await?)?; data::reviewed(&result, &peer, FEATURE_A)?; }
RecoveryReviewApproved::LOGICAL_LABEL => { let result = data::decode(branch.recv::<RecoveryReviewApproved>().await?)?; data::reviewed(&result, &peer, FEATURE_A)?; }
RecoveryReviewRejected::LOGICAL_LABEL => { let result = data::decode(branch.recv::<RecoveryReviewRejected>().await?)?; data::reviewed(&result, &peer, FEATURE_A)?; }
        CiFailedA::LOGICAL_LABEL => { branch.recv::<CiFailedA>().await?; }
        LastCiPassedA::LOGICAL_LABEL => { branch.recv::<LastCiPassedA>().await?; return Ok::<_,ShellError>(()); }
        _ => return Err(ShellError::message("unexpected final delivery continuation")),
    }
}
};
 async{
 let(mut feature_a,mut feature_b,mut result_a,mut result_b)=if review_b_first{
 let args=RepositoryArguments::from_bytes(br#"{"pr_id":"payment_processing"}"#)?;
 reviewer.send::<ReviewFirstB>(&args).await?;
reviewer.send::<ReviewFirstB>(&args).await?;
reviewer.send::<ReviewFirstB>(&args).await?;
reviewer.send::<ReviewFirstB>(&args).await?; let mut feature_b=reviewer.recv::<Feature<AUTH_B>>().await?;
 let args=RepositoryArguments::from_bytes(br#"{"pr_id":"payment_processing"}"#)?;
 feature_b.send::<Review>(&args).await?;
 let result_b=data::decode(feature_b.recv::<ReviewResultB>().await?)?; let mut feature_a=reviewer.recv::<Feature<AUTH_A>>().await?;
 let args=RepositoryArguments::from_bytes(br#"{"pr_id":"user_authentication"}"#)?;
 feature_a.send::<Review>(&args).await?;
 let result_a=data::decode(feature_a.recv::<ReviewResultA>().await?)?;
 (feature_a,feature_b,result_a,result_b)
 }else{
 let args=RepositoryArguments::from_bytes(br#"{"pr_id":"user_authentication"}"#)?;
 reviewer.send::<ReviewFirstA>(&args).await?;
reviewer.send::<ReviewFirstA>(&args).await?;
reviewer.send::<ReviewFirstA>(&args).await?;
reviewer.send::<ReviewFirstA>(&args).await?; let mut feature_a=reviewer.recv::<Feature<AUTH_A>>().await?;
 let args=RepositoryArguments::from_bytes(br#"{"pr_id":"user_authentication"}"#)?;
 feature_a.send::<Review>(&args).await?;
 let result_a=data::decode(feature_a.recv::<ReviewResultA>().await?)?; let mut feature_b=reviewer.recv::<Feature<AUTH_B>>().await?;
 let args=RepositoryArguments::from_bytes(br#"{"pr_id":"payment_processing"}"#)?;
 feature_b.send::<Review>(&args).await?;
 let result_b=data::decode(feature_b.recv::<ReviewResultB>().await?)?;
 (feature_a,feature_b,result_a,result_b)
 };
 if review_b_first{ loop{
 initial_b.set(result_b.clone());
 let bytes=data::encode(&result_b)?;
 if result_b["success"]==true {
 let reply=ReviewReply::<true>::from_bytes(&bytes)?;
 feature_b.send::<Approved>(&reply).await?;
 feature_b.send::<ReviewApproved>(&reply).await?;
feature_b.send::<ReviewApproved>(&reply).await?;
feature_b.send::<ReviewApproved>(&reply).await?;
feature_b.send::<ReviewApproved>(&reply).await?;
feature_b.send::<ReviewApproved>(&reply).await?;
feature_b.send::<ReviewApproved>(&reply).await?;
 reviewer.send::<Feature<AUTH_B>>(feature_b).await?; break;
 } else{
 let reply=ReviewReply::<false>::from_bytes(&bytes)?;
 feature_b.send::<Rejected>(&reply).await?;
 feature_b.send::<ReviewRejected>(&reply).await?;
feature_b.send::<ReviewRejected>(&reply).await?;
feature_b.send::<ReviewRejected>(&reply).await?;
feature_b.send::<ReviewRejected>(&reply).await?;
feature_b.send::<ReviewRejected>(&reply).await?;
feature_b.send::<ReviewRejected>(&reply).await?;
 reviewer.send::<Feature<AUTH_B>>(feature_b).await?;
 feature_b=reviewer.recv::<Feature<AUTH_B>>().await?;
 let args=RepositoryArguments::from_bytes(br#"{"pr_id":"payment_processing"}"#)?;
 feature_b.send::<Review>(&args).await?;
 result_b=data::decode(feature_b.recv::<ReviewResultB>().await?)?;
 }
 } loop{
 initial_a.set(result_a.clone());
 let bytes=data::encode(&result_a)?;
 if result_a["success"]==true {
 let reply=ReviewReply::<true>::from_bytes(&bytes)?;
 feature_a.send::<Approved>(&reply).await?;
 feature_a.send::<ReviewApproved>(&reply).await?;
feature_a.send::<ReviewApproved>(&reply).await?;
feature_a.send::<ReviewApproved>(&reply).await?;
feature_a.send::<ReviewApproved>(&reply).await?;
feature_a.send::<ReviewApproved>(&reply).await?;
feature_a.send::<ReviewApproved>(&reply).await?;
 reviewer.send::<Feature<AUTH_A>>(feature_a).await?; break;
 } else{
 let reply=ReviewReply::<false>::from_bytes(&bytes)?;
 feature_a.send::<Rejected>(&reply).await?;
 feature_a.send::<ReviewRejected>(&reply).await?;
feature_a.send::<ReviewRejected>(&reply).await?;
feature_a.send::<ReviewRejected>(&reply).await?;
feature_a.send::<ReviewRejected>(&reply).await?;
feature_a.send::<ReviewRejected>(&reply).await?;
feature_a.send::<ReviewRejected>(&reply).await?;
 reviewer.send::<Feature<AUTH_A>>(feature_a).await?;
 feature_a=reviewer.recv::<Feature<AUTH_A>>().await?;
 let args=RepositoryArguments::from_bytes(br#"{"pr_id":"user_authentication"}"#)?;
 feature_a.send::<Review>(&args).await?;
 result_a=data::decode(feature_a.recv::<ReviewResultA>().await?)?;
 }
 } }else{ loop{
 initial_a.set(result_a.clone());
 let bytes=data::encode(&result_a)?;
 if result_a["success"]==true {
 let reply=ReviewReply::<true>::from_bytes(&bytes)?;
 feature_a.send::<Approved>(&reply).await?;
 feature_a.send::<ReviewApproved>(&reply).await?;
feature_a.send::<ReviewApproved>(&reply).await?;
feature_a.send::<ReviewApproved>(&reply).await?;
feature_a.send::<ReviewApproved>(&reply).await?;
feature_a.send::<ReviewApproved>(&reply).await?;
feature_a.send::<ReviewApproved>(&reply).await?;
 reviewer.send::<Feature<AUTH_A>>(feature_a).await?; break;
 } else{
 let reply=ReviewReply::<false>::from_bytes(&bytes)?;
 feature_a.send::<Rejected>(&reply).await?;
 feature_a.send::<ReviewRejected>(&reply).await?;
feature_a.send::<ReviewRejected>(&reply).await?;
feature_a.send::<ReviewRejected>(&reply).await?;
feature_a.send::<ReviewRejected>(&reply).await?;
feature_a.send::<ReviewRejected>(&reply).await?;
feature_a.send::<ReviewRejected>(&reply).await?;
 reviewer.send::<Feature<AUTH_A>>(feature_a).await?;
 feature_a=reviewer.recv::<Feature<AUTH_A>>().await?;
 let args=RepositoryArguments::from_bytes(br#"{"pr_id":"user_authentication"}"#)?;
 feature_a.send::<Review>(&args).await?;
 result_a=data::decode(feature_a.recv::<ReviewResultA>().await?)?;
 }
 } loop{
 initial_b.set(result_b.clone());
 let bytes=data::encode(&result_b)?;
 if result_b["success"]==true {
 let reply=ReviewReply::<true>::from_bytes(&bytes)?;
 feature_b.send::<Approved>(&reply).await?;
 feature_b.send::<ReviewApproved>(&reply).await?;
feature_b.send::<ReviewApproved>(&reply).await?;
feature_b.send::<ReviewApproved>(&reply).await?;
feature_b.send::<ReviewApproved>(&reply).await?;
feature_b.send::<ReviewApproved>(&reply).await?;
feature_b.send::<ReviewApproved>(&reply).await?;
 reviewer.send::<Feature<AUTH_B>>(feature_b).await?; break;
 } else{
 let reply=ReviewReply::<false>::from_bytes(&bytes)?;
 feature_b.send::<Rejected>(&reply).await?;
 feature_b.send::<ReviewRejected>(&reply).await?;
feature_b.send::<ReviewRejected>(&reply).await?;
feature_b.send::<ReviewRejected>(&reply).await?;
feature_b.send::<ReviewRejected>(&reply).await?;
feature_b.send::<ReviewRejected>(&reply).await?;
feature_b.send::<ReviewRejected>(&reply).await?;
 reviewer.send::<Feature<AUTH_B>>(feature_b).await?;
 feature_b=reviewer.recv::<Feature<AUTH_B>>().await?;
 let args=RepositoryArguments::from_bytes(br#"{"pr_id":"payment_processing"}"#)?;
 feature_b.send::<Review>(&args).await?;
 result_b=data::decode(feature_b.recv::<ReviewResultB>().await?)?;
 }
 } }
 loop{let branch=reviewer.offer().await?;
 match branch.label(){
 CiPassedA::LOGICAL_LABEL=>{branch.recv::<CiPassedA>().await?;}
 CiPassedB::LOGICAL_LABEL=>{branch.recv::<CiPassedB>().await?;}
 LastCiPassedA::LOGICAL_LABEL=>{branch.recv::<LastCiPassedA>().await?;return Ok::<_,ShellError>(());}
 LastCiPassedB::LOGICAL_LABEL=>{branch.recv::<LastCiPassedB>().await?;return Ok::<_,ShellError>(());}
 CiFailedA::LOGICAL_LABEL=>{branch.recv::<CiFailedA>().await?;
 loop{let args=RepositoryArguments::from_bytes(br#"{"pr_id":"user_authentication"}"#)?;
 reviewer.send::<ReviewRequestA>(&args).await?;
 let branch=reviewer.offer().await?;match branch.label(){
 RecoveryApproved::LOGICAL_LABEL=>{branch.recv::<RecoveryApproved>().await?;break;}
 RecoveryRejected::LOGICAL_LABEL=>{branch.recv::<RecoveryRejected>().await?;}
 _=>return Err(ShellError::message("expected actual recovery review outcome"))}}}
CiFailedB::LOGICAL_LABEL=>{branch.recv::<CiFailedB>().await?;
 loop{let args=RepositoryArguments::from_bytes(br#"{"pr_id":"payment_processing"}"#)?;
 reviewer.send::<ReviewRequestB>(&args).await?;
 let branch=reviewer.offer().await?;match branch.label(){
 RecoveryApproved::LOGICAL_LABEL=>{branch.recv::<RecoveryApproved>().await?;break;}
 RecoveryRejected::LOGICAL_LABEL=>{branch.recv::<RecoveryRejected>().await?;}
 _=>return Err(ShellError::message("expected actual recovery review outcome"))}}}
 _=>return Err(ShellError::message("unexpected reviewer continuation"))
 }} };
 async{
 let mut environment=writer.recv::<TestLoan>().await?;
 let branch=writer.offer().await?;match branch.label(){
 ReviewFirstA::LOGICAL_LABEL=>{branch.recv::<ReviewFirstA>().await?;}
 ReviewFirstB::LOGICAL_LABEL=>{branch.recv::<ReviewFirstB>().await?;}
 _=>return Err(ShellError::message("expected actual initial review choice"))}
 if tests_b_first{{let args=RepositoryArguments::from_bytes(br#"{"target":"payment_processing"}"#)?;
 writer.send::<TestIntentB>(&args).await?;
 let candidate=data::decode(writer.recv::<CandidateForTestsB>().await?)?;
 data::candidate(&candidate,FEATURE_B)?;
 environment.send::<TestRequestB>(&args).await?;
 let result=environment.recv::<TestResultB>().await?.to_vec();
 let decoded=data::decode(&result)?; data::tested(&decoded,&candidate,FEATURE_B)?;
 let result=if corruption==Some(5)&&review_b_first==true{
 let mut altered=data::decode(&result)?;altered["data"]["candidate_sha256"]=json!("old source");data::encode(&altered)?
 }else{result};
 writer.send::<TestsForCiB>(&result.as_slice()).await?;
 } {let args=RepositoryArguments::from_bytes(br#"{"target":"user_authentication"}"#)?;
 writer.send::<TestIntentA>(&args).await?;
 let candidate=data::decode(writer.recv::<CandidateForTestsA>().await?)?;
 data::candidate(&candidate,FEATURE_A)?;
 environment.send::<TestRequestA>(&args).await?;
 let result=environment.recv::<TestResultA>().await?.to_vec();
 let decoded=data::decode(&result)?; data::tested(&decoded,&candidate,FEATURE_A)?;
 let result=if corruption==Some(5)&&review_b_first==false{
 let mut altered=data::decode(&result)?;altered["data"]["candidate_sha256"]=json!("old source");data::encode(&altered)?
 }else{result};
 writer.send::<TestsForCiA>(&result.as_slice()).await?;
 }}else{{let args=RepositoryArguments::from_bytes(br#"{"target":"user_authentication"}"#)?;
 writer.send::<TestIntentA>(&args).await?;
 let candidate=data::decode(writer.recv::<CandidateForTestsA>().await?)?;
 data::candidate(&candidate,FEATURE_A)?;
 environment.send::<TestRequestA>(&args).await?;
 let result=environment.recv::<TestResultA>().await?.to_vec();
 let decoded=data::decode(&result)?; data::tested(&decoded,&candidate,FEATURE_A)?;
 let result=if corruption==Some(5)&&review_b_first==false{
 let mut altered=data::decode(&result)?;altered["data"]["candidate_sha256"]=json!("old source");data::encode(&altered)?
 }else{result};
 writer.send::<TestsForCiA>(&result.as_slice()).await?;
 } {let args=RepositoryArguments::from_bytes(br#"{"target":"payment_processing"}"#)?;
 writer.send::<TestIntentB>(&args).await?;
 let candidate=data::decode(writer.recv::<CandidateForTestsB>().await?)?;
 data::candidate(&candidate,FEATURE_B)?;
 environment.send::<TestRequestB>(&args).await?;
 let result=environment.recv::<TestResultB>().await?.to_vec();
 let decoded=data::decode(&result)?; data::tested(&decoded,&candidate,FEATURE_B)?;
 let result=if corruption==Some(5)&&review_b_first==true{
 let mut altered=data::decode(&result)?;altered["data"]["candidate_sha256"]=json!("old source");data::encode(&altered)?
 }else{result};
 writer.send::<TestsForCiB>(&result.as_slice()).await?;
 }}
 loop{let branch=writer.offer().await?;match branch.label(){
 ReviewApproved::LOGICAL_LABEL=>{branch.recv::<ReviewApproved>().await?;}
 RecoveryReviewApproved::LOGICAL_LABEL=>{branch.recv::<RecoveryReviewApproved>().await?;}
 ReviewRejected::LOGICAL_LABEL=>{
 let result=data::decode(branch.recv::<ReviewRejected>().await?)?;
 if result["data"]["target"]==FEATURE_A{{let args=RepositoryArguments::from_bytes(br#"{"target":"user_authentication"}"#)?;
 writer.send::<TestIntentA>(&args).await?;
 let candidate=data::decode(writer.recv::<CandidateForTestsA>().await?)?;
 data::candidate(&candidate,FEATURE_A)?;
 environment.send::<TestRequestA>(&args).await?;
 let result=environment.recv::<TestResultA>().await?.to_vec();
 let decoded=data::decode(&result)?; data::tested(&decoded,&candidate,FEATURE_A)?;
 let result=if corruption==Some(5)&&review_b_first==false{
 let mut altered=data::decode(&result)?;altered["data"]["candidate_sha256"]=json!("old source");data::encode(&altered)?
 }else{result};
 writer.send::<TestsForCiA>(&result.as_slice()).await?;
 }}else{{let args=RepositoryArguments::from_bytes(br#"{"target":"payment_processing"}"#)?;
 writer.send::<TestIntentB>(&args).await?;
 let candidate=data::decode(writer.recv::<CandidateForTestsB>().await?)?;
 data::candidate(&candidate,FEATURE_B)?;
 environment.send::<TestRequestB>(&args).await?;
 let result=environment.recv::<TestResultB>().await?.to_vec();
 let decoded=data::decode(&result)?; data::tested(&decoded,&candidate,FEATURE_B)?;
 let result=if corruption==Some(5)&&review_b_first==true{
 let mut altered=data::decode(&result)?;altered["data"]["candidate_sha256"]=json!("old source");data::encode(&altered)?
 }else{result};
 writer.send::<TestsForCiB>(&result.as_slice()).await?;
 }}
 }
 RecoveryReviewRejected::LOGICAL_LABEL=>{
 let result=data::decode(branch.recv::<RecoveryReviewRejected>().await?)?;
 if result["data"]["target"]==FEATURE_A{{let args=RepositoryArguments::from_bytes(br#"{"target":"user_authentication"}"#)?;
 writer.send::<TestIntentA>(&args).await?;
 let candidate=data::decode(writer.recv::<CandidateForTestsA>().await?)?;
 data::candidate(&candidate,FEATURE_A)?;
 environment.send::<TestRequestA>(&args).await?;
 let result=environment.recv::<TestResultA>().await?.to_vec();
 let decoded=data::decode(&result)?; data::tested(&decoded,&candidate,FEATURE_A)?;
 let result=if corruption==Some(5)&&review_b_first==false{
 let mut altered=data::decode(&result)?;altered["data"]["candidate_sha256"]=json!("old source");data::encode(&altered)?
 }else{result};
 writer.send::<TestsForCiA>(&result.as_slice()).await?;
 }}else{{let args=RepositoryArguments::from_bytes(br#"{"target":"payment_processing"}"#)?;
 writer.send::<TestIntentB>(&args).await?;
 let candidate=data::decode(writer.recv::<CandidateForTestsB>().await?)?;
 data::candidate(&candidate,FEATURE_B)?;
 environment.send::<TestRequestB>(&args).await?;
 let result=environment.recv::<TestResultB>().await?.to_vec();
 let decoded=data::decode(&result)?; data::tested(&decoded,&candidate,FEATURE_B)?;
 let result=if corruption==Some(5)&&review_b_first==true{
 let mut altered=data::decode(&result)?;altered["data"]["candidate_sha256"]=json!("old source");data::encode(&altered)?
 }else{result};
 writer.send::<TestsForCiB>(&result.as_slice()).await?;
 }}
 }
 CiFailedA::LOGICAL_LABEL=>{branch.recv::<CiFailedA>().await?;{let args=RepositoryArguments::from_bytes(br#"{"target":"user_authentication"}"#)?;
 writer.send::<TestIntentA>(&args).await?;
 let candidate=data::decode(writer.recv::<CandidateForTestsA>().await?)?;
 data::candidate(&candidate,FEATURE_A)?;
 environment.send::<TestRequestA>(&args).await?;
 let result=environment.recv::<TestResultA>().await?.to_vec();
 let decoded=data::decode(&result)?; data::tested(&decoded,&candidate,FEATURE_A)?;
 let result=if corruption==Some(5)&&review_b_first==false{
 let mut altered=data::decode(&result)?;altered["data"]["candidate_sha256"]=json!("old source");data::encode(&altered)?
 }else{result};
 writer.send::<TestsForCiA>(&result.as_slice()).await?;
 }}
 CiFailedB::LOGICAL_LABEL=>{branch.recv::<CiFailedB>().await?;{let args=RepositoryArguments::from_bytes(br#"{"target":"payment_processing"}"#)?;
 writer.send::<TestIntentB>(&args).await?;
 let candidate=data::decode(writer.recv::<CandidateForTestsB>().await?)?;
 data::candidate(&candidate,FEATURE_B)?;
 environment.send::<TestRequestB>(&args).await?;
 let result=environment.recv::<TestResultB>().await?.to_vec();
 let decoded=data::decode(&result)?; data::tested(&decoded,&candidate,FEATURE_B)?;
 let result=if corruption==Some(5)&&review_b_first==true{
 let mut altered=data::decode(&result)?;altered["data"]["candidate_sha256"]=json!("old source");data::encode(&altered)?
 }else{result};
 writer.send::<TestsForCiB>(&result.as_slice()).await?;
 }}
 CiPassedA::LOGICAL_LABEL=>{branch.recv::<CiPassedA>().await?;}
 CiPassedB::LOGICAL_LABEL=>{branch.recv::<CiPassedB>().await?;}
 LastCiPassedA::LOGICAL_LABEL=>{branch.recv::<LastCiPassedA>().await?;break;}
 LastCiPassedB::LOGICAL_LABEL=>{branch.recv::<LastCiPassedB>().await?;break;}
 _=>return Err(ShellError::message("unexpected tester continuation"))}}
 writer.send::<TestReturn>(environment).await?;
 Ok::<_,ShellError>(())
 };
 async{
 let mut build=runner.recv::<BuildLoan>().await?;
 loop{let publication=data::decode(runner.recv::<Merged>().await?)?;
 if publication["data"]["feature"]==FEATURE_A{let mut feature=runner.recv::<Feature<AUTH_A>>().await?;
 let args=RepositoryArguments::from_bytes(br#"{"suite":"full_test_suite_after_auth"}"#)?;
 runner.send::<CiIntentA>(&args).await?;
 async{
 async{feature.send::<RunCiOnFeatureA>(&args).await?;};
 async{let sent=if corruption==Some(2)&&review_b_first==false{
 RepositoryArguments::from_bytes(br#"{"suite":"full_test_suite_after_payment"}"#)?
 }else{RepositoryArguments::from_bytes(br#"{"suite":"full_test_suite_after_auth"}"#)?};
 build.send::<RunCiA>(&sent).await?;};
 Ok::<_,ShellError>(())
 }.await?;
 let result=data::decode(feature.recv::<CiResult>().await?)?;
 let second=data::decode(build.recv::<CiResult>().await?)?;
 data::same(&result,&second)?; data::ci_publication(&result,&publication)?;
 actual_ci_a.set(result.clone());
 let bytes=data::encode(&result)?;
 if result["success"]==true{
 let reply=ReviewReply::<true>::from_bytes(&bytes)?;
 feature.send::<IntegratedA>(&reply).await?;
 runner.send::<Feature<AUTH_A>>(feature).await?;
 let branch=runner.offer().await?;match branch.label(){
 CiPassedA::LOGICAL_LABEL=>{branch.recv::<CiPassedA>().await?;}
 LastCiPassedA::LOGICAL_LABEL=>{branch.recv::<LastCiPassedA>().await?;break;}
 _=>return Err(ShellError::message("expected exact successful CI feedback"))}
 }else{
 let reply=ReviewReply::<false>::from_bytes(&bytes)?;
 feature.send::<IntegrationFailedA>(&reply).await?;
 feature.send::<CiFailedA>(&reply).await?;
feature.send::<CiFailedA>(&reply).await?;
feature.send::<CiFailedA>(&reply).await?;
feature.send::<CiFailedA>(&reply).await?;
feature.send::<CiFailedA>(&reply).await?;
feature.send::<CiFailedA>(&reply).await?;
feature.send::<CiFailedA>(&reply).await?;
feature.send::<CiFailedA>(&reply).await?;
 runner.recv::<CiFailedA>().await?;
 runner.send::<Feature<AUTH_A>>(feature).await?;
 }}
 else if publication["data"]["feature"]==FEATURE_B{let mut feature=runner.recv::<Feature<AUTH_B>>().await?;
 let args=RepositoryArguments::from_bytes(br#"{"suite":"full_test_suite_after_payment"}"#)?;
 runner.send::<CiIntentB>(&args).await?;
 async{
 async{feature.send::<RunCiOnFeatureB>(&args).await?;};
 async{let sent=if corruption==Some(2)&&review_b_first==true{
 RepositoryArguments::from_bytes(br#"{"suite":"full_test_suite_after_auth"}"#)?
 }else{RepositoryArguments::from_bytes(br#"{"suite":"full_test_suite_after_payment"}"#)?};
 build.send::<RunCiB>(&sent).await?;};
 Ok::<_,ShellError>(())
 }.await?;
 let result=data::decode(feature.recv::<CiResult>().await?)?;
 let second=data::decode(build.recv::<CiResult>().await?)?;
 data::same(&result,&second)?; data::ci_publication(&result,&publication)?;
 actual_ci_b.set(result.clone());
 let bytes=data::encode(&result)?;
 if result["success"]==true{
 let reply=ReviewReply::<true>::from_bytes(&bytes)?;
 feature.send::<IntegratedB>(&reply).await?;
 runner.send::<Feature<AUTH_B>>(feature).await?;
 let branch=runner.offer().await?;match branch.label(){
 CiPassedB::LOGICAL_LABEL=>{branch.recv::<CiPassedB>().await?;}
 LastCiPassedB::LOGICAL_LABEL=>{branch.recv::<LastCiPassedB>().await?;break;}
 _=>return Err(ShellError::message("expected exact successful CI feedback"))}
 }else{
 let reply=ReviewReply::<false>::from_bytes(&bytes)?;
 feature.send::<IntegrationFailedB>(&reply).await?;
 feature.send::<CiFailedB>(&reply).await?;
feature.send::<CiFailedB>(&reply).await?;
feature.send::<CiFailedB>(&reply).await?;
feature.send::<CiFailedB>(&reply).await?;
feature.send::<CiFailedB>(&reply).await?;
feature.send::<CiFailedB>(&reply).await?;
feature.send::<CiFailedB>(&reply).await?;
feature.send::<CiFailedB>(&reply).await?;
 runner.recv::<CiFailedB>().await?;
 runner.send::<Feature<AUTH_B>>(feature).await?;
 }}
 else{return Err(ShellError::message("unknown merged candidate"));}}
 runner.send::<BuildReturn>(build).await?;
 Ok::<_,ShellError>(())
 };
 Ok::<_,ShellError>(())
 },deadline(core::time::Duration::from_secs(8))).await;
        let checkpoint = rv.tap().checkpoint().expect("capture counter exhausted");
        let records = records.take();
        let frames = transport.take_observations();
        let effects = effects.events.take();
        if let Some(directory) = &evidence {
            let name = format!(
                "review-{}-tests-{}-case-{case}.json",
                u8::from(review_b_first),
                u8::from(tests_b_first)
            );
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(directory.join(name))
                .map_err(|e| ShellError::message(e.to_string()))?;
            serde_json::to_writer(&mut file,&json!({"native_fixture":true,"model_calls":0,"runtime_error":result.as_ref().err().map(ToString::to_string),
 "review_b_first":review_b_first,"tests_b_first":tests_b_first,"failure_plan":{"initial_review":initial_review_failures,"recovery_review":recovery_review_failures,"ci":ci_failures},
 "session":SESSION,"checkpoint":checkpoint,"records":records,"frames":frames,"effects":effects})).map_err(|e|ShellError::message(e.to_string()))?;
        }
        if let Some(trial) = corruption {
            let expected = match trial {
                2 => "resource request differs from admitted arguments",
                3 | 7 | 9 | 10 => "CI result differs from the admitted repository",
                4 => "successful CI did not use the received test occurrence",
                5 => "CI test data differ from the published candidate",
                6 => "CI needs the actual reviewed publication",
                8 => "successful CI used different test definition data",
                _ => unreachable!(),
            };
            assert_eq!(
                result.unwrap_err().to_string(),
                expected,
                "whole corruption case={case} trial={trial}"
            );
            let ci_calls = effects
                .iter()
                .filter(|row| row["kind"] == "complete" && row["tool"] == "run_tests")
                .count();
            assert_eq!(ci_calls, usize::from(!matches!(trial, 2 | 5 | 6)));
            eprintln!(
                "{}",
                json!({"native_fixture":true,"case":case,"corruption":trial,"review_b_first":review_b_first,"rejected":true,"ci_calls":ci_calls,"model_calls":0})
            );
            continue;
        }
        result.map_err(|e|ShellError::message(format!("whole execution review_b_first={review_b_first} tests_b_first={tests_b_first} case={case}: {e}")))?;
        assert_eq!(
            records.len() as u64,
            u64::from(checkpoint),
            "producer capture must be complete"
        );
        let completed = effects
            .iter()
            .filter(|row| row["kind"] == "complete")
            .count();
        let expected = 10
            + 3 * (initial_review_failures.iter().sum::<usize>()
                + recovery_review_failures.iter().sum::<usize>())
            + 5 * ci_failures.iter().sum::<usize>();
        assert_eq!(completed, expected);
        let mut last_frames = std::collections::BTreeMap::new();
        for row in &frames {
            last_frames.insert(
                row["frame_id"].as_u64().unwrap(),
                row["action"].as_str().unwrap(),
            );
        }
        assert!(
            last_frames.values().all(|action| *action == "receive"),
            "a resource or result frame was left unconsumed"
        );
        eprintln!(
            "{}",
            json!({"native_fixture":true,"review_b_first":review_b_first,"tests_b_first":tests_b_first,"case":case,"completed_effects":completed,"captured_records":records.len(),"frames":last_frames.len(),"model_calls":0})
        );
    }
    Ok(())
}
