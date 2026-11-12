//! Current 6M source and actual receiver boundaries, without other tasks.
#[path = "../../../src/imp/projection.rs"]
pub(crate) mod projection;
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
use feature_delivery::{global::*, localside};
use hibana::runtime::{SessionKitStorage, ids::SessionId, program::project};
use serde_json::{Value, json};
use shell::{
    ShellResult,
    imp::{repository_arguments::RepositoryArguments, service::Domain},
};
#[cfg(miri)]
#[path = "../../../src/tasks/handoff/tests/clock.rs"]
mod clock;
#[cfg(miri)]
use clock::deadline;
#[cfg(not(miri))]
use hibana_runtime::host::deadline;

use feature_delivery::global::delivery::{self, *};
use shell::imp::review_reply::ReviewReply;
// The fixture owns an append-only effect journal, not participant progress.
#[derive(Default)]
struct Effects(Cell<Vec<Value>>);

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

impl Domain for Effects {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        let mut journal = self.0.take();
        journal
            .push(json!({"observation":"start","agent":agent,"tool":tool,"arguments":arguments}));
        self.0.set(journal);
        for _ in 0..3 {
            hibana_runtime::yield_now().await;
        }
        let mut journal = self.0.take();
        let result = match tool {
            "implement_feature" => json!({"tool_name":tool,"success":true,
                "data":{"feature":arguments["feature_name"],
                "source":arguments["source"],"candidate_sha256":arguments["source"]}}),
            "write_tests" | "code_review" => {
                let target = if tool == "write_tests" {
                    &arguments["target"]
                } else {
                    &arguments["pr_id"]
                };
                let candidate = journal
                    .iter()
                    .rev()
                    .find(|row| {
                        row["observation"] == "complete"
                            && row["tool"] == "implement_feature"
                            && row["result"]["data"]["feature"] == *target
                    })
                    .expect("validation must have completed source");
                json!({"tool_name":tool,"success":true,"data":{"target":target,"component":target,
                    "candidate_sha256":candidate["result"]["data"]["candidate_sha256"],
                    "test_source":{"target":target},"test_sha256":"fixture test data"}})
            }
            _ => panic!("unexpected preparation effect {tool}"),
        };
        journal.push(json!({"observation":"complete","agent":agent,"tool":tool,"arguments":arguments,"result":result}));
        self.0.set(journal);
        Ok(result)
    }
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn feature_delivery_preparation_preserves_both_review_and_test_orders() -> ShellResult<()> {
    use hibana::{
        g::Message,
        runtime::{resolver::ResolverRef, transport::EndpointBinding},
    };
    for review_b_first in [false, true] {
        for tests_b_first in [false, true] {
            let global = delivery::protocol();
            let a_program = project::<DEVELOPER_A>(&global);
            let b_program = project::<DEVELOPER_B>(&global);
            let writer_program = project::<TEST_WRITER>(&global);
            let reviewer_program = project::<REVIEWER>(&global);
            let runner_program = project::<CI_RUNNER>(&global);
            let repository_program = project::<REPOSITORY>(&global);
            let review_a_program = project::<REVIEW_A>(&global);
            let review_b_program = project::<REVIEW_B>(&global);
            let data_a_program = project::<TEST_DATA_A>(&global);
            let data_b_program = project::<TEST_DATA_B>(&global);
            let ci_program = project::<CI_SERVICE>(&global);
            let repo_program = project::<REPO>(&global);
            let environment_program = project::<TEST_ENV>(&global);
            let build_program = project::<BUILD_SERVER>(&global);
            let feature_a_program = project::<AUTH_A>(&global);
            let feature_b_program = project::<AUTH_B>(&global);
            let sid = SessionId::new(SESSION);
            // SAFETY: this fixture is the sole issuer of its five distinct bindings.
            #[allow(unsafe_code)]
            let transport = transport::InMemoryTransport::new(unsafe {
                [
                    (sid, REPO, EndpointBinding::new()),
                    (sid, TEST_ENV, EndpointBinding::new()),
                    (sid, BUILD_SERVER, EndpointBinding::new()),
                    (sid, AUTH_A, EndpointBinding::new()),
                    (sid, AUTH_B, EndpointBinding::new()),
                ]
            });
            let mut slab = vec![0; 1024 * 1024];
            let actual_a = Cell::new(Value::Null);
            let actual_b = Cell::new(Value::Null);
            let mut storage = SessionKitStorage::uninit();
            let rv = storage.init().rendezvous(&mut slab, &transport)?;
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
            let effects = &Effects::default();
            rv.set_resolver(
                &reviewer_program,
                ResolverRef::<REVIEW_DECISION_A>::decision_state(&actual_a, decide_actual),
            )
            .unwrap();
            rv.set_resolver(
                &reviewer_program,
                ResolverRef::<REVIEW_DECISION_B>::decision_state(&actual_b, decide_actual),
            )
            .unwrap();
            let result = hibana_runtime::race(async {
async { let _resources = localside::repository::run(&mut repository, (repo, environment, feature_a, feature_b), effects).await?; Ok(()) };
async { localside::ci::run(&mut ci, build, effects).await };
async { let _build = runner.recv::<BuildLoan>().await?; core::future::pending::<ShellResult<()>>().await };
async { localside::reviews::feature_a(&mut review_a, effects).await };
async { localside::reviews::feature_b(&mut review_b, effects).await };
async { localside::tests::feature_a(&mut data_a, effects).await };
async { localside::tests::feature_b(&mut data_b, effects).await };
async { let mut feature = a.recv::<Feature<AUTH_A>>().await?;
let branch = a.offer().await?;
let first_review = match branch.label() {
ReviewFirstA::LOGICAL_LABEL => branch.recv::<ReviewFirstA>().await?.into_value(),
ReviewFirstB::LOGICAL_LABEL => branch.recv::<ReviewFirstB>().await?.into_value(),
_ => panic!("expected the selected initial review"),
};
if first_review["pr_id"] == FEATURE_B {
    let changed: Value = serde_json::from_slice(a.recv::<InterfaceChangedB>().await?).unwrap();
    assert_eq!(changed["data"]["feature"], FEATURE_B);
}
let admitted = RepositoryArguments::from_bytes(br#"{"feature_name":"user_authentication","source":"auth candidate bytes"}"#)?;
a.send::<ImplementA>(&admitted).await?;
let mut repo = a.recv::<RepoLoan>().await?;
let authority = &admitted; let arguments = &admitted;
async {
async { feature.send::<Implement>(authority).await?; };
async { repo.send::<ImplementOnRepo>(arguments).await?; };
Ok::<_, shell::ShellError>(())
}.await?;
let candidate: Value = serde_json::from_slice(repo.recv::<RepoResult>().await?).unwrap();
a.send::<RepoReturn>(repo).await?;
let changed = serde_json::to_vec(&candidate).unwrap();
feature.send::<InterfaceChangedA>(&changed.as_slice()).await?;
a.send::<Feature<AUTH_A>>(feature).await?;
if first_review["pr_id"] == FEATURE_A {
    a.recv::<InterfaceChangedB>().await?;
} let tests: Value = serde_json::from_slice(a.recv::<TestedA>().await?).unwrap();
let review: Value = loop {
    let branch = a.offer().await?;
    match branch.label() {
        ReviewApproved::LOGICAL_LABEL => { branch.recv::<ReviewApproved>().await?; }
        Approved::LOGICAL_LABEL => break serde_json::from_slice(branch.recv::<Approved>().await?).unwrap(),
        _ => panic!("expected the current or peer's actual review result"),
    }
};
assert_eq!(tests["data"]["candidate_sha256"], candidate["data"]["candidate_sha256"]);
assert_eq!(review["data"]["candidate_sha256"], candidate["data"]["candidate_sha256"]); core::future::pending::<ShellResult<()>>().await };
async { let mut feature = b.recv::<Feature<AUTH_B>>().await?;
let branch = b.offer().await?;
let first_review = match branch.label() {
ReviewFirstA::LOGICAL_LABEL => branch.recv::<ReviewFirstA>().await?.into_value(),
ReviewFirstB::LOGICAL_LABEL => branch.recv::<ReviewFirstB>().await?.into_value(),
_ => panic!("expected the selected initial review"),
};
if first_review["pr_id"] == FEATURE_A {
    let changed: Value = serde_json::from_slice(b.recv::<InterfaceChangedA>().await?).unwrap();
    assert_eq!(changed["data"]["feature"], FEATURE_A);
}
let admitted = RepositoryArguments::from_bytes(br#"{"feature_name":"payment_processing","source":"payment candidate bytes"}"#)?;
b.send::<ImplementB>(&admitted).await?;
let mut repo = b.recv::<RepoLoan>().await?;
let authority = &admitted; let arguments = &admitted;
async {
async { feature.send::<Implement>(authority).await?; };
async { repo.send::<ImplementOnRepo>(arguments).await?; };
Ok::<_, shell::ShellError>(())
}.await?;
let candidate: Value = serde_json::from_slice(repo.recv::<RepoResult>().await?).unwrap();
b.send::<RepoReturn>(repo).await?;
let changed = serde_json::to_vec(&candidate).unwrap();
feature.send::<InterfaceChangedB>(&changed.as_slice()).await?;
b.send::<Feature<AUTH_B>>(feature).await?;
if first_review["pr_id"] == FEATURE_B {
    b.recv::<InterfaceChangedA>().await?;
} let tests: Value = serde_json::from_slice(b.recv::<TestedB>().await?).unwrap();
let review: Value = loop {
    let branch = b.offer().await?;
    match branch.label() {
        ReviewApproved::LOGICAL_LABEL => { branch.recv::<ReviewApproved>().await?; }
        Approved::LOGICAL_LABEL => break serde_json::from_slice(branch.recv::<Approved>().await?).unwrap(),
        _ => panic!("expected the current or peer's actual review result"),
    }
};
assert_eq!(tests["data"]["candidate_sha256"], candidate["data"]["candidate_sha256"]);
assert_eq!(review["data"]["candidate_sha256"], candidate["data"]["candidate_sha256"]); core::future::pending::<ShellResult<()>>().await };
async {
async {
async {
if review_b_first {
let arguments = RepositoryArguments::from_bytes(br#"{"pr_id":"payment_processing"}"#)?;
reviewer.send::<ReviewFirstB>(&arguments).await?;
reviewer.send::<ReviewFirstB>(&arguments).await?;
reviewer.send::<ReviewFirstB>(&arguments).await?;
reviewer.send::<ReviewFirstB>(&arguments).await?;
let mut b = reviewer.recv::<Feature<AUTH_B>>().await?;
let arguments = RepositoryArguments::from_bytes(br#"{"pr_id":"payment_processing"}"#)?;
b.send::<Review>(&arguments).await?;
let b_result = b.recv::<ReviewResultB>().await?;
let b_bytes = b_result.to_vec();
actual_b.set(serde_json::from_slice(&b_bytes).unwrap());
let mut a = reviewer.recv::<Feature<AUTH_A>>().await?;
let arguments = RepositoryArguments::from_bytes(br#"{"pr_id":"user_authentication"}"#)?;
a.send::<Review>(&arguments).await?;
let a_result = a.recv::<ReviewResultA>().await?;
let a_bytes = a_result.to_vec();
actual_a.set(serde_json::from_slice(&a_bytes).unwrap());
let reply = ReviewReply::<true>::from_bytes(&b_bytes)?;
b.send::<Approved>(&reply).await?;
b.send::<ReviewApproved>(&reply).await?;
b.send::<ReviewApproved>(&reply).await?;
b.send::<ReviewApproved>(&reply).await?;
b.send::<ReviewApproved>(&reply).await?;
b.send::<ReviewApproved>(&reply).await?;
b.send::<ReviewApproved>(&reply).await?;
reviewer.send::<Feature<AUTH_B>>(b).await?;
let reply = ReviewReply::<true>::from_bytes(&a_bytes)?;
a.send::<Approved>(&reply).await?;
a.send::<ReviewApproved>(&reply).await?;
a.send::<ReviewApproved>(&reply).await?;
a.send::<ReviewApproved>(&reply).await?;
a.send::<ReviewApproved>(&reply).await?;
a.send::<ReviewApproved>(&reply).await?;
a.send::<ReviewApproved>(&reply).await?;
reviewer.send::<Feature<AUTH_A>>(a).await?;
} else {
let arguments = RepositoryArguments::from_bytes(br#"{"pr_id":"user_authentication"}"#)?;
reviewer.send::<ReviewFirstA>(&arguments).await?;
reviewer.send::<ReviewFirstA>(&arguments).await?;
reviewer.send::<ReviewFirstA>(&arguments).await?;
reviewer.send::<ReviewFirstA>(&arguments).await?;
let mut a = reviewer.recv::<Feature<AUTH_A>>().await?;
let arguments = RepositoryArguments::from_bytes(br#"{"pr_id":"user_authentication"}"#)?;
a.send::<Review>(&arguments).await?;
let a_result = a.recv::<ReviewResultA>().await?;
let a_bytes = a_result.to_vec();
actual_a.set(serde_json::from_slice(&a_bytes).unwrap());
let mut b = reviewer.recv::<Feature<AUTH_B>>().await?;
let arguments = RepositoryArguments::from_bytes(br#"{"pr_id":"payment_processing"}"#)?;
b.send::<Review>(&arguments).await?;
let b_result = b.recv::<ReviewResultB>().await?;
let b_bytes = b_result.to_vec();
actual_b.set(serde_json::from_slice(&b_bytes).unwrap());
let reply = ReviewReply::<true>::from_bytes(&a_bytes)?;
a.send::<Approved>(&reply).await?;
a.send::<ReviewApproved>(&reply).await?;
a.send::<ReviewApproved>(&reply).await?;
a.send::<ReviewApproved>(&reply).await?;
a.send::<ReviewApproved>(&reply).await?;
a.send::<ReviewApproved>(&reply).await?;
a.send::<ReviewApproved>(&reply).await?;
reviewer.send::<Feature<AUTH_A>>(a).await?;
let reply = ReviewReply::<true>::from_bytes(&b_bytes)?;
b.send::<Approved>(&reply).await?;
b.send::<ReviewApproved>(&reply).await?;
b.send::<ReviewApproved>(&reply).await?;
b.send::<ReviewApproved>(&reply).await?;
b.send::<ReviewApproved>(&reply).await?;
b.send::<ReviewApproved>(&reply).await?;
b.send::<ReviewApproved>(&reply).await?;
reviewer.send::<Feature<AUTH_B>>(b).await?;
}
// Deliberately stop at this prefix of the actual whole-task source.
Ok::<_,shell::ShellError>(())
 };
async {
let mut environment = writer.recv::<TestLoan>().await?;
let branch = writer.offer().await?;
match branch.label() {
ReviewFirstA::LOGICAL_LABEL => { branch.recv::<ReviewFirstA>().await?; }
ReviewFirstB::LOGICAL_LABEL => { branch.recv::<ReviewFirstB>().await?; }
_ => panic!("expected the selected initial review"),
}
let args_a = RepositoryArguments::from_bytes(br#"{"target":"user_authentication"}"#)?;
let args_b = RepositoryArguments::from_bytes(br#"{"target":"payment_processing"}"#)?;
for target in if tests_b_first { [FEATURE_B, FEATURE_A] } else { [FEATURE_A, FEATURE_B] } {
if target == FEATURE_A {
writer.send::<TestIntentA>(&args_a).await?;
writer.recv::<CandidateForTestsA>().await?;
environment.send::<TestRequestA>(&args_a).await?;
let tests = environment.recv::<TestResultA>().await?;
writer.send::<TestsForCiA>(&tests).await?;
} else {
writer.send::<TestIntentB>(&args_b).await?;
writer.recv::<CandidateForTestsB>().await?;
environment.send::<TestRequestB>(&args_b).await?;
let tests = environment.recv::<TestResultB>().await?;
writer.send::<TestsForCiB>(&tests).await?;
}
}
writer.recv::<ReviewApproved>().await?; writer.recv::<ReviewApproved>().await?;
Ok::<_,shell::ShellError>(())
 };
Ok::<_,shell::ShellError>(())
}.await?;
Err::<(),_>(shell::ShellError::message("preparation prefix checked"))
};
Ok::<_,shell::ShellError>(())
}, deadline(core::time::Duration::from_secs(if cfg!(miri) {120} else {5}))).await;
            assert_eq!(
                result.unwrap_err().to_string(),
                "preparation prefix checked"
            );
            let journal = effects.0.take();
            assert_eq!(
                journal
                    .iter()
                    .filter(|row| row["observation"] == "complete")
                    .count(),
                6
            );
            let first = if review_b_first { FEATURE_B } else { FEATURE_A };
            let second = if review_b_first { FEATURE_A } else { FEATURE_B };
            let review_start = journal
                .iter()
                .position(|row| {
                    row["observation"] == "start"
                        && row["tool"] == "code_review"
                        && row["arguments"]["pr_id"] == first
                })
                .unwrap();
            let implementation_end = journal
                .iter()
                .position(|row| {
                    row["observation"] == "complete"
                        && row["tool"] == "implement_feature"
                        && row["result"]["data"]["feature"] == second
                })
                .unwrap();
            assert!(
                review_start < implementation_end,
                "review must overlap the other implementation: {journal:?}"
            );
        }
    }
    Ok(())
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn feature_delivery_reject_repository_argument_substitution() -> ShellResult<()> {
    use hibana::{g::Message, runtime::transport::EndpointBinding};
    for (substituted, substitute_feature) in [
        (
            br#"{"feature_name":"user_authentication","source":"changed after admission"}"#
                .as_slice(),
            false,
        ),
        (
            br#"{"feature_name":"payment_processing","source":"auth candidate bytes"}"#.as_slice(),
            false,
        ),
        (
            br#"{"feature_name":"user_authentication","source":"changed after admission"}"#
                .as_slice(),
            true,
        ),
    ] {
        let global = delivery::protocol();
        let a_program = project::<DEVELOPER_A>(&global);
        let b_program = project::<DEVELOPER_B>(&global);
        let writer_program = project::<TEST_WRITER>(&global);
        let reviewer_program = project::<REVIEWER>(&global);
        let runner_program = project::<CI_RUNNER>(&global);
        let repository_program = project::<REPOSITORY>(&global);

        let ci_program = project::<CI_SERVICE>(&global);
        let repo_program = project::<REPO>(&global);
        let environment_program = project::<TEST_ENV>(&global);
        let build_program = project::<BUILD_SERVER>(&global);
        let feature_a_program = project::<AUTH_A>(&global);
        let feature_b_program = project::<AUTH_B>(&global);
        let sid = SessionId::new(SESSION);
        // SAFETY: this fixture is the sole issuer of its five distinct bindings.
        #[allow(unsafe_code)]
        let transport = transport::InMemoryTransport::new(unsafe {
            [
                (sid, REPO, EndpointBinding::new()),
                (sid, TEST_ENV, EndpointBinding::new()),
                (sid, BUILD_SERVER, EndpointBinding::new()),
                (sid, AUTH_A, EndpointBinding::new()),
                (sid, AUTH_B, EndpointBinding::new()),
            ]
        });
        let mut slab = vec![0; 1024 * 1024];
        let mut storage = SessionKitStorage::uninit();
        let rv = storage.init().rendezvous(&mut slab, &transport)?;
        let mut a = rv.enter(sid, &a_program)?;
        let mut b = rv.enter(sid, &b_program)?;
        let mut writer = rv.enter(sid, &writer_program)?;
        let mut reviewer = rv.enter(sid, &reviewer_program)?;
        let mut runner = rv.enter(sid, &runner_program)?;
        let mut repository = rv.enter(sid, &repository_program)?;

        let mut ci = rv.enter(sid, &ci_program)?;
        let repo = rv.enter(sid, &repo_program)?;
        let environment = rv.enter(sid, &environment_program)?;
        let build = rv.enter(sid, &build_program)?;
        let feature_a = rv.enter(sid, &feature_a_program)?;
        let feature_b = rv.enter(sid, &feature_b_program)?;
        let effects = &Effects::default();
        let result = hibana_runtime::race(async {
async { let _resources = localside::repository::run(&mut repository, (repo, environment, feature_a, feature_b), effects).await?; Ok(()) };
async { localside::ci::run(&mut ci, build, effects).await };
async { let _build = runner.recv::<BuildLoan>().await?; core::future::pending::<ShellResult<()>>().await };
async { let _feature = b.recv::<Feature<AUTH_B>>().await?; core::future::pending::<ShellResult<()>>().await };
async { let _environment = writer.recv::<TestLoan>().await?; core::future::pending::<ShellResult<()>>().await };
async { let arguments = RepositoryArguments::from_bytes(br#"{"pr_id":"user_authentication"}"#)?;
reviewer.send::<ReviewFirstA>(&arguments).await?;
reviewer.send::<ReviewFirstA>(&arguments).await?;
reviewer.send::<ReviewFirstA>(&arguments).await?;
reviewer.send::<ReviewFirstA>(&arguments).await?; core::future::pending::<ShellResult<()>>().await };
async { let mut feature = a.recv::<Feature<AUTH_A>>().await?;
let branch = a.offer().await?;
let first_review = match branch.label() {
ReviewFirstA::LOGICAL_LABEL => branch.recv::<ReviewFirstA>().await?.into_value(),
ReviewFirstB::LOGICAL_LABEL => branch.recv::<ReviewFirstB>().await?.into_value(),
_ => panic!("expected the selected initial review"),
};
if first_review["pr_id"] == FEATURE_B {
    let changed: Value = serde_json::from_slice(a.recv::<InterfaceChangedB>().await?).unwrap();
    assert_eq!(changed["data"]["feature"], FEATURE_B);
}
let admitted = RepositoryArguments::from_bytes(br#"{"feature_name":"user_authentication","source":"auth candidate bytes"}"#)?;
a.send::<ImplementA>(&admitted).await?;
let mut repo = a.recv::<RepoLoan>().await?;
let changed = RepositoryArguments::from_bytes(substituted)?;
let authority = if substitute_feature { &changed } else { &admitted };
let arguments = if substitute_feature { &admitted } else { &changed };
async {
async { feature.send::<Implement>(authority).await?; };
async { repo.send::<ImplementOnRepo>(arguments).await?; };
Ok::<_, shell::ShellError>(())
}.await?;
let candidate: Value = serde_json::from_slice(repo.recv::<RepoResult>().await?).unwrap();
a.send::<RepoReturn>(repo).await?;
let changed = serde_json::to_vec(&candidate).unwrap();
feature.send::<InterfaceChangedA>(&changed.as_slice()).await?;
a.send::<Feature<AUTH_A>>(feature).await?;
if first_review["pr_id"] == FEATURE_A {
    a.recv::<InterfaceChangedB>().await?;
} core::future::pending::<ShellResult<()>>().await };
Ok::<_,shell::ShellError>(())
}, deadline(core::time::Duration::from_secs(if cfg!(miri) {120} else {5}))).await;
        assert_eq!(
            result.unwrap_err().to_string(),
            "resource request differs from admitted arguments"
        );
        assert!(effects.0.take().is_empty());
    }
    Ok(())
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn feature_delivery_reject_test_environment_target_substitution() -> ShellResult<()> {
    use hibana::{g::Message, runtime::transport::EndpointBinding};
    let global = delivery::protocol();
    let a_program = project::<DEVELOPER_A>(&global);
    let b_program = project::<DEVELOPER_B>(&global);
    let writer_program = project::<TEST_WRITER>(&global);
    let reviewer_program = project::<REVIEWER>(&global);
    let runner_program = project::<CI_RUNNER>(&global);
    let repository_program = project::<REPOSITORY>(&global);
    let review_a_program = project::<REVIEW_A>(&global);

    let data_a_program = project::<TEST_DATA_A>(&global);

    let ci_program = project::<CI_SERVICE>(&global);
    let repo_program = project::<REPO>(&global);
    let environment_program = project::<TEST_ENV>(&global);
    let build_program = project::<BUILD_SERVER>(&global);
    let feature_a_program = project::<AUTH_A>(&global);
    let feature_b_program = project::<AUTH_B>(&global);
    let sid = SessionId::new(SESSION);
    // SAFETY: this fixture is the sole issuer of its five distinct bindings.
    #[allow(unsafe_code)]
    let transport = transport::InMemoryTransport::new(unsafe {
        [
            (sid, REPO, EndpointBinding::new()),
            (sid, TEST_ENV, EndpointBinding::new()),
            (sid, BUILD_SERVER, EndpointBinding::new()),
            (sid, AUTH_A, EndpointBinding::new()),
            (sid, AUTH_B, EndpointBinding::new()),
        ]
    });
    let mut slab = vec![0; 1024 * 1024];
    let mut storage = SessionKitStorage::uninit();
    let rv = storage.init().rendezvous(&mut slab, &transport)?;
    let mut a = rv.enter(sid, &a_program)?;
    let mut b = rv.enter(sid, &b_program)?;
    let mut writer = rv.enter(sid, &writer_program)?;
    let mut reviewer = rv.enter(sid, &reviewer_program)?;
    let mut runner = rv.enter(sid, &runner_program)?;
    let mut repository = rv.enter(sid, &repository_program)?;
    let mut review_a = rv.enter(sid, &review_a_program)?;

    let mut data_a = rv.enter(sid, &data_a_program)?;

    let mut ci = rv.enter(sid, &ci_program)?;
    let repo = rv.enter(sid, &repo_program)?;
    let environment = rv.enter(sid, &environment_program)?;
    let build = rv.enter(sid, &build_program)?;
    let feature_a = rv.enter(sid, &feature_a_program)?;
    let feature_b = rv.enter(sid, &feature_b_program)?;
    let effects = &Effects::default();
    let result = hibana_runtime::race(
        async {
            async {
                let _resources = localside::repository::run(
                    &mut repository,
                    (repo, environment, feature_a, feature_b),
                    effects,
                )
                .await?;
                Ok(())
            };
            async { localside::ci::run(&mut ci, build, effects).await };
            async {
                let _build = runner.recv::<BuildLoan>().await?;
                core::future::pending::<ShellResult<()>>().await
            };
            async { localside::reviews::feature_a(&mut review_a, effects).await };
            async { localside::tests::feature_a(&mut data_a, effects).await };
            async {
                let _feature = b.recv::<Feature<AUTH_B>>().await?;
                core::future::pending::<ShellResult<()>>().await
            };
            async {
                let arguments =
                    RepositoryArguments::from_bytes(br#"{"pr_id":"user_authentication"}"#)?;
                reviewer.send::<ReviewFirstA>(&arguments).await?;
                reviewer.send::<ReviewFirstA>(&arguments).await?;
                reviewer.send::<ReviewFirstA>(&arguments).await?;
                reviewer.send::<ReviewFirstA>(&arguments).await?;
                let mut feature = reviewer.recv::<Feature<AUTH_A>>().await?;
                let arguments =
                    RepositoryArguments::from_bytes(br#"{"pr_id":"user_authentication"}"#)?;
                feature.send::<Review>(&arguments).await?;
                feature.recv::<ReviewResultA>().await?;
                core::future::pending::<ShellResult<()>>().await
            };
            async {
                let mut feature = a.recv::<Feature<AUTH_A>>().await?;
                let branch = a.offer().await?;
                match branch.label() {
                    ReviewFirstA::LOGICAL_LABEL => {
                        branch.recv::<ReviewFirstA>().await?;
                    }
                    ReviewFirstB::LOGICAL_LABEL => {
                        branch.recv::<ReviewFirstB>().await?;
                    }
                    _ => panic!("expected the selected initial review"),
                }
                let admitted = RepositoryArguments::from_bytes(
                    br#"{"feature_name":"user_authentication","source":"auth candidate bytes"}"#,
                )?;
                a.send::<ImplementA>(&admitted).await?;
                let mut repo = a.recv::<RepoLoan>().await?;
                let authority = &admitted;
                let arguments = &admitted;
                async {
                    async {
                        feature.send::<Implement>(authority).await?;
                    };
                    async {
                        repo.send::<ImplementOnRepo>(arguments).await?;
                    };
                    Ok::<_, shell::ShellError>(())
                }
                .await?;
                let candidate: Value =
                    serde_json::from_slice(repo.recv::<RepoResult>().await?).unwrap();
                a.send::<RepoReturn>(repo).await?;
                let changed = serde_json::to_vec(&candidate).unwrap();
                feature
                    .send::<InterfaceChangedA>(&changed.as_slice())
                    .await?;
                a.send::<Feature<AUTH_A>>(feature).await?;
                let _candidate = candidate;
                core::future::pending::<ShellResult<()>>().await
            };
            async {
                let mut environment = writer.recv::<TestLoan>().await?;
                writer.recv::<ReviewFirstA>().await?;
                let admitted =
                    RepositoryArguments::from_bytes(br#"{"target":"user_authentication"}"#)?;
                writer.send::<TestIntentA>(&admitted).await?;
                writer.recv::<CandidateForTestsA>().await?;
                let changed =
                    RepositoryArguments::from_bytes(br#"{"target":"payment_processing"}"#)?;
                environment.send::<TestRequestA>(&changed).await?;
                environment.recv::<TestResultA>().await?;
                Ok::<_, shell::ShellError>(())
            };
            Ok::<_, shell::ShellError>(())
        },
        deadline(core::time::Duration::from_secs(if cfg!(miri) {
            120
        } else {
            5
        })),
    )
    .await;
    assert_eq!(
        result.unwrap_err().to_string(),
        "resource request differs from admitted arguments"
    );
    assert!(
        !effects
            .0
            .take()
            .iter()
            .any(|row| row["tool"] == "write_tests")
    );
    Ok(())
}
