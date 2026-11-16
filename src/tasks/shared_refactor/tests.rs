//! Actual production receiver rejects forged validation data before any publication.
use super::{arguments::Arguments, global::*, localside};
use crate::shell::{
    ShellResult,
    imp::{bus::Bus, service::Domain},
};
use core::cell::Cell;
use hibana::{
    g,
    runtime::{SessionKitStorage, ids::SessionId, program::project},
};
use serde_json::{Value, json};

struct Effects(Cell<Vec<Value>>);
impl Domain for Effects {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        let mut calls = self.0.take();
        calls.push(json!({"agent":agent,"tool":tool,"arguments":arguments}));
        self.0.set(calls);
        if tool != "refactor_code" {
            return Err("unexpected publication effect".into());
        }
        Ok(json!({"tool_name":tool,"success":true,"data":{
            "target":arguments["target"],"source":"actual fixture source",
            "content_sha256":"0".repeat(64),
            "working_candidates":{"module_a":"1".repeat(64),"module_b":"2".repeat(64)}}}))
    }
}

#[hibana_runtime::main]
#[test]
async fn production_receiver_rejects_stale_negative_or_wrong_kind_before_publication()
-> ShellResult<()> {
    for bad in 0..12 {
        let edits = g::par(
            g::seq(
                g::send::<A_FILE, REPO, RefactorA>(),
                g::send::<REPO, A_FILE, RefactoredA>(),
            ),
            g::par(
                g::seq(
                    g::send::<A_FILE, REPO, RefactorSharedA>(),
                    g::send::<REPO, A_FILE, SharedChangedA>(),
                ),
                g::par(
                    g::seq(
                        g::send::<B_FILE, REPO, RefactorB>(),
                        g::send::<REPO, B_FILE, RefactoredB>(),
                    ),
                    g::seq(
                        g::send::<B_FILE, REPO, RefactorSharedB>(),
                        g::send::<REPO, B_FILE, SharedChangedB>(),
                    ),
                ),
            ),
        );
        let validations = g::par(
            g::seq(
                g::par(
                    g::seq(
                        g::send::<REPO_OWNER, REVIEWER, ReadyReviewA>(),
                        g::send::<REVIEWER, REPO_OWNER, ReviewedA>(),
                    ),
                    g::seq(
                        g::send::<REPO_OWNER, TEST_ENV, ReadyTestA>(),
                        g::send::<TEST_ENV, REPO_OWNER, TestedA>(),
                    ),
                ),
                g::par(
                    g::send::<REPO_OWNER, A_FILE, ReviewFeedbackA>(),
                    g::send::<REPO_OWNER, A_FILE, TestFeedbackA>(),
                ),
            ),
            g::seq(
                g::par(
                    g::seq(
                        g::send::<REPO_OWNER, REVIEWER, ReadyReviewB>(),
                        g::send::<REVIEWER, REPO_OWNER, ReviewedB>(),
                    ),
                    g::seq(
                        g::send::<REPO_OWNER, TEST_ENV, ReadyTestB>(),
                        g::send::<TEST_ENV, REPO_OWNER, TestedB>(),
                    ),
                ),
                g::par(
                    g::send::<REPO_OWNER, B_FILE, ReviewFeedbackB>(),
                    g::send::<REPO_OWNER, B_FILE, TestFeedbackB>(),
                ),
            ),
        );
        let source = g::seq(
            g::send::<RESOURCE_INIT, REPO_OWNER, RepoLoan>(),
            g::seq(edits, validations),
        );
        let (p_owner, p_init, p_repo, p_a, p_b, p_review, p_test) = (
            project::<REPO_OWNER>(&source),
            project::<RESOURCE_INIT>(&source),
            project::<REPO>(&source),
            project::<A_FILE>(&source),
            project::<B_FILE>(&source),
            project::<REVIEWER>(&source),
            project::<TEST_ENV>(&source),
        );
        let session = SessionId::new(SESSION);
        let (bus, _, []) = Bus::open([], [(session, REPO)]);
        let mut slab = vec![0; 256 * 1024];
        let mut storage = SessionKitStorage::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        let mut owner = rv.enter(session, &p_owner)?;
        let mut init = rv.enter(session, &p_init)?;
        let repo = rv.enter(session, &p_repo)?;
        let mut a = rv.enter(session, &p_a)?;
        let mut b = rv.enter(session, &p_b)?;
        let mut reviewer = rv.enter(session, &p_review)?;
        let mut tester = rv.enter(session, &p_test)?;
        let effects = Effects(Cell::new(Vec::new()));
        let review = json!({"tool_name":"code_review","success":true,"data":{
            "target":"module_a","candidate_sha256":"1".repeat(64),
            "outputs":[{"input":-3,"actual":-5,"expected":-5},{"input":0,"actual":1,"expected":1},{"input":7,"actual":15,"expected":15}]}});
        let test = json!({"tool_name":"run_tests","success":true,"data":{
            "suite":"module_a_tests","candidate_sha256":"1".repeat(64),
            "outputs":[{"input":-3,"actual":-5,"expected":-5},{"input":0,"actual":1,"expected":1},{"input":7,"actual":15,"expected":15}]}});
        let mut wrong = if bad < 6 {
            review.clone()
        } else {
            test.clone()
        };
        match bad % 6 {
            0 => wrong["data"]["candidate_sha256"] = json!("0".repeat(64)),
            1 => wrong["success"] = json!(false),
            2 => wrong["tool_name"] = json!("unrelated"),
            3 => wrong["data"][if bad < 6 { "target" } else { "suite" }] = json!("unrelated"),
            4 => {
                wrong["data"]["outputs"][0]["actual"] = json!(999);
                wrong["data"]["outputs"][0]["expected"] = json!(999);
            }
            _ => wrong["data"]["outputs"] = json!([]),
        }
        let error = hibana_runtime::race(localside::repository::run(&mut owner, &effects), async {
            async {
                init.send::<RepoLoan>(repo).await?;
            };
            async {
                a.send::<RefactorA>(&Arguments::<2>::try_from(
                    br#"{"target":"module_a"}"#.as_slice(),
                )?)
                .await?;
                a.recv::<RefactoredA>().await?;
                b.send::<RefactorB>(&Arguments::<4>::try_from(
                    br#"{"target":"module_b"}"#.as_slice(),
                )?)
                .await?;
                b.recv::<RefactoredB>().await?;
                a.send::<RefactorSharedA>(&Arguments::<3>::try_from(
                    br#"{"target":"SHARED_LIBRARY"}"#.as_slice(),
                )?)
                .await?;
                a.recv::<SharedChangedA>().await?;
                b.send::<RefactorSharedB>(&Arguments::<5>::try_from(
                    br#"{"target":"SHARED_LIBRARY"}"#.as_slice(),
                )?)
                .await?;
                b.recv::<SharedChangedB>().await?;
            };
            async {
                reviewer.recv::<ReadyReviewA>().await?;
                let bytes = serde_json::to_vec(if bad < 6 { &wrong } else { &review }).unwrap();
                reviewer.send::<ReviewedA>(&bytes.as_slice()).await?;
            };
            async {
                tester.recv::<ReadyTestA>().await?;
                let bytes = serde_json::to_vec(if bad >= 6 { &wrong } else { &test }).unwrap();
                tester.send::<TestedA>(&bytes.as_slice()).await?;
            };
            core::future::pending::<ShellResult<()>>().await
        })
        .await
        .expect_err("production receiver accepted forged validation");
        assert!(
            error
                .to_string()
                .contains("validation is not successful evidence"),
            "{error}"
        );
        let calls = effects.0.take();
        assert_eq!(calls.len(), 4, "{calls:?}");
        assert!(calls.iter().all(|call| call["tool"] == "refactor_code"));
    }
    Ok(())
}
