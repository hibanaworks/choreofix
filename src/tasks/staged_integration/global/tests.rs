use super::*;
use crate::shell::{
    ShellResult,
    imp::{
        bus::Bus,
        service::{ReceivedEffects, SimulatorOperation},
    },
};
use core::cell::Cell;
use hibana::runtime::{SessionKitStorage, ids::SessionId, resolver::ResolverRef};
use serde_json::{Value, json};
use std::{collections::VecDeque, time::Duration};

// A finite sequence of actual effect results, not a participant progress cursor.
struct ReviewEffects(Cell<VecDeque<Value>>);
impl ReceivedEffects for ReviewEffects {
    fn apply<'io, P: SimulatorOperation>(
        &'io self,
        received: hibana::runtime::wire::ReceivedPayload<'_, P>,
    ) -> impl Future<Output = ShellResult<Value>> + use<'io, P> {
        let arguments = crate::imp::json::decode(received.as_bytes());
        async move {
            let arguments = arguments?;
            let (agent, tool) = P::RPC;
            assert_eq!((agent, tool), ("REVIEWER", "code_review"));
            assert_eq!(arguments, json!({"pr_id":"user_authentication"}));
            let mut replies = self.0.take();
            let reply = replies.pop_front().expect("unrequested extra review");
            self.0.set(replies);
            Ok(reply)
        }
    }
}

fn material(attempt: u64, success: bool) -> (Value, Value, Value) {
    let id = attempt * 10 + 1;
    let candidate = json!({"tool_name":"implement_feature","success":true,"data":{
        "agent":"DEVELOPER_A","artifact_id":format!("development:{id}"),
        "feature":"user_authentication","source":format!("source {attempt}"),
        "candidate_sha256":format!("candidate {attempt}")}});
    let tests = json!({"tool_name":"write_tests","success":true,"data":{
        "agent":"TEST_WRITER","artifact_id":format!("development:{}", id+1),
        "component":"user_authentication","source_receipt":id,
        "candidate_sha256":candidate["data"]["candidate_sha256"],
        "test_source":{"case":"current source"},"test_sha256":format!("test {attempt}")}});
    let review = json!({"tool_name":"code_review","success":success,"data":{
        "agent":"REVIEWER","artifact_id":format!("development:{}", id+2),
        "target":"user_authentication","source_receipt":id,
        "candidate_sha256":candidate["data"]["candidate_sha256"]}});
    (candidate, tests, review)
}

#[hibana_runtime::main]
#[test]
async fn actual_review_results_leave_rejection_roll_for_approval() -> ShellResult<()> {
    use super::super::{actual_outcome, imp, localside};

    // Use the production preparation source, including its elastic rejection roll.
    // The suffix only closes the review participant's unrelated CI/staging inputs.
    let source = g::seq(
        prepare_a::<ApprovedA, RejectedA>(),
        g::seq(
            g::send::<REPO, REVIEW_A, CiUpdateA>(),
            g::seq(
                g::send::<REPO, REVIEW_A, StageUpdateA>(),
                g::send::<REPO, REVIEW_A, TaskExit>(),
            ),
        ),
    );
    let repo_program: RoleProgram<REPO> = project(&source);
    let review_program: RoleProgram<REVIEW_A> = project(&source);
    let test_program: RoleProgram<TEST_ENV> = project(&source);
    let a_program: RoleProgram<A_FILE> = project(&source);
    let b_program: RoleProgram<B_FILE> = project(&source);
    let c_program: RoleProgram<C_FILE> = project(&source);
    let test_file_program: RoleProgram<TEST_FILE> = project(&source);
    let review_file_program: RoleProgram<A_REVIEW_FILE> = project(&source);

    for outcomes in [&[true][..], &[false, false, true][..]] {
        let material: Vec<_> = outcomes
            .iter()
            .enumerate()
            .map(|(attempt, &success)| material(attempt as u64, success))
            .collect();
        let domain = ReviewEffects(Cell::new(
            material
                .iter()
                .map(|(_, _, review)| review.clone())
                .collect(),
        ));
        let actual = Cell::new(None);
        let (bus, _, _) = Bus::open([], []);
        let mut slab = vec![0; 1024 * 1024];
        let mut storage = SessionKitStorage::<&crate::imp::transport::InMemoryTransport>::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        rv.set_resolver(
            &review_program,
            ResolverRef::<REVIEW_DECISION_A>::decision_state(&actual, actual_outcome),
        )
        .expect("register the actual review result");
        let session = SessionId::new(63);
        let mut repo = rv.enter(session, &repo_program)?;
        let mut review = rv.enter(session, &review_program)?;
        let mut test = rv.enter(session, &test_program)?;
        let mut a = rv.enter(session, &a_program)?;
        let mut b = rv.enter(session, &b_program)?;
        let mut c = rv.enter(session, &c_program)?;
        let mut test_file = rv.enter(session, &test_file_program)?;
        let mut review_file = rv.enter(session, &review_file_program)?;
        let operation = async {
            async {
                localside::reviews::feature_a(&mut review, (&domain, &actual)).await?;
            };
            async {
                for (candidate, _, expected) in &material {
                    let _ = repo.recv::<ImplementA>().await?;
                    let bytes = imp::encode(candidate)?;
                    repo.send::<ImplementationA>(&bytes.as_slice()).await?;
                    repo.send::<TestCandidateA>(&bytes.as_slice()).await?;
                    repo.send::<FileTestCandidateA>(&bytes.as_slice()).await?;
                    repo.send::<InterfaceAToB>(&bytes.as_slice()).await?;
                    repo.send::<InterfaceAToC>(&bytes.as_slice()).await?;
                    let tests = imp::decode(repo.recv::<TestedA>().await?)?;
                    let bytes = imp::encode(&json!({"candidate":candidate,"tests":tests}))?;
                    repo.send::<ReviewCandidateA>(&bytes.as_slice()).await?;
                    repo.send::<FileReviewCandidateA>(&bytes.as_slice()).await?;
                    let received = if expected["success"] == true {
                        imp::decode(repo.recv::<ApprovedA>().await?)?
                    } else {
                        imp::decode(repo.recv::<RejectedA>().await?)?
                    };
                    assert_eq!(&received, expected);
                    let bytes = imp::encode(&received)?;
                    // All five feedback deliveries belong to the production source.
                    for _ in 0..5 {
                        repo.send::<ReviewFeedbackA>(&bytes.as_slice()).await?;
                    }
                }
                let ci = imp::encode(&json!({"tool_name":"run_tests","success":true,"data":{
                    "agent":"CI_RUNNER","artifact_id":"development:100",
                    "feature":"user_authentication","suite":"full_suite_after_auth"}}))?;
                repo.send::<CiUpdateA>(&ci.as_slice()).await?;
                let stage = imp::encode(&json!({"tool_name":"validate_staging","success":true,
                    "data":{"agent":"RELEASE_MANAGER","artifact_id":"development:101",
                    "service":"user_authentication","candidate_sha256":"current",
                    "deployment_event":99}}))?;
                repo.send::<StageUpdateA>(&stage.as_slice()).await?;
                repo.send::<TaskExit>(&0).await?;
            };
            async {
                for (_, tests, expected) in &material {
                    let _ = test.recv::<TestCandidateA>().await?;
                    let _ = test.recv::<TestRequestA>().await?;
                    let bytes = imp::encode(tests)?;
                    test.send::<TestedA>(&bytes.as_slice()).await?;
                    test.send::<FileTestedA>(&bytes.as_slice()).await?;
                    assert_eq!(
                        imp::decode(test.recv::<ReviewFeedbackA>().await?)?,
                        *expected
                    );
                }
            };
            async {
                let request = EffectRequest(Arguments::<0>::try_from(
                    b"{\"feature_name\":\"user_authentication\"}".as_slice(),
                )?);
                for (candidate, _, expected) in &material {
                    a.send::<ImplementA>(&request).await?;
                    assert_eq!(imp::decode(a.recv::<ImplementationA>().await?)?, *candidate);
                    assert_eq!(imp::decode(a.recv::<ReviewFeedbackA>().await?)?, *expected);
                }
            };
            async {
                for (candidate, _, expected) in &material {
                    assert_eq!(imp::decode(b.recv::<InterfaceAToB>().await?)?, *candidate);
                    assert_eq!(imp::decode(b.recv::<ReviewFeedbackA>().await?)?, *expected);
                }
            };
            async {
                for (candidate, _, expected) in &material {
                    assert_eq!(imp::decode(c.recv::<InterfaceAToC>().await?)?, *candidate);
                    assert_eq!(imp::decode(c.recv::<ReviewFeedbackA>().await?)?, *expected);
                }
            };
            async {
                let request = EffectRequest(Arguments::<1>::try_from(
                    b"{\"target\":\"user_authentication\"}".as_slice(),
                )?);
                for (candidate, tests, expected) in &material {
                    assert_eq!(
                        imp::decode(test_file.recv::<FileTestCandidateA>().await?)?,
                        *candidate
                    );
                    test_file.send::<TestRequestA>(&request).await?;
                    assert_eq!(imp::decode(test_file.recv::<FileTestedA>().await?)?, *tests);
                    assert_eq!(
                        imp::decode(test_file.recv::<ReviewFeedbackA>().await?)?,
                        *expected
                    );
                }
            };
            async {
                let request = EffectRequest(Arguments::<2>::try_from(
                    b"{\"pr_id\":\"user_authentication\"}".as_slice(),
                )?);
                for (candidate, tests, expected) in &material {
                    assert_eq!(
                        imp::decode(review_file.recv::<FileReviewCandidateA>().await?)?,
                        json!({"candidate":candidate,"tests":tests})
                    );
                    review_file.send::<ReviewRequestA>(&request).await?;
                    assert_eq!(
                        imp::decode(review_file.recv::<FileReviewResultA>().await?)?,
                        *expected
                    );
                }
            };
            Ok::<_, crate::shell::ShellError>(())
        };
        hibana_runtime::race(
            async { Ok::<_, std::io::Error>(operation.await) },
            hibana_runtime::host::deadline(Duration::from_secs(2)),
        )
        .await
        .expect("the review source stalled")?;
        assert!(domain.0.take().is_empty());
        assert_eq!(actual.take().expect("actual final review")["success"], true);
    }
    Ok(())
}
