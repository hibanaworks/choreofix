//! Actual production receiver rejects forged validation data before any publication.
use super::{arguments::Arguments, global::*, localside};
use crate::shell::{
    ShellError, ShellResult,
    imp::{
        bus::Bus,
        service::{ReceivedEffects, SimulatorOperation},
    },
};
use core::cell::Cell;
use hibana::runtime::{SessionKitStorage, ids::SessionId};
use serde_json::{Value, json};

struct Effects(Cell<Vec<Value>>);
impl ReceivedEffects for Effects {
    fn apply<'io, P: SimulatorOperation>(
        &'io self,
        received: hibana::runtime::wire::ReceivedPayload<'_, P>,
    ) -> impl Future<Output = ShellResult<Value>> + use<'io, P> {
        let input = crate::imp::json::decode(received.as_bytes());
        async move {
            let arguments = input?;
            let (agent, tool) = P::RPC;
            let mut calls = self.0.take();
            calls.push(json!({"agent":agent,"tool":tool,"arguments":arguments}));
            self.0.set(calls);
            if tool != "refactor_code" {
                return Err(ShellError::message("unexpected publication effect"));
            }
            Ok(json!({"tool_name":tool,"success":true,"data":{
            "target":arguments["target"],"source":"actual fixture source",
            "content_sha256":"0".repeat(64),
            "working_candidates":{"module_a":"1".repeat(64),"module_b":"2".repeat(64)}}}))
        }
    }
}

#[hibana_runtime::main]
#[test]
async fn production_receiver_rejects_stale_negative_or_wrong_kind_before_publication()
-> ShellResult<()> {
    for bad in 0..12 {
        let p = programs();
        let session = SessionId::new(SESSION);
        let (bus, _, []) = Bus::open(
            [],
            [
                REPO,
                TEST_ENV,
                ARCH_FILE,
                A_FILE,
                B_FILE,
                REVIEW_FILE,
                TEST_FILE,
            ]
            .map(|role| (session, role)),
        );
        let mut slab = vec![0; 256 * 1024];
        let mut storage = SessionKitStorage::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        let mut owner = rv.enter(session, &p.repo_owner)?;
        let mut init = rv.enter(session, &p.resource_init)?;
        let mut test_owner = rv.enter(session, &p.test_owner)?;
        let mut reviewer = rv.enter(session, &p.reviewer)?;
        init.send::<RepoLoan>(rv.enter(session, &p.repo)?).await?;
        init.send::<TestLoan>(rv.enter(session, &p.environment)?)
            .await?;
        let mut tester = test_owner.recv::<TestLoan>().await?;
        let mut arch_data = rv.enter(session, &p.arch_data)?;
        let mut arch_vm = rv.enter(session, &p.arch_vm)?;
        arch_data
            .send::<Loan<ARCH_FILE>>(rv.enter(session, &p.arch_file)?)
            .await?;
        let _arch = arch_vm.recv::<Loan<ARCH_FILE>>().await?;
        let mut a_data = rv.enter(session, &p.a_data)?;
        let mut a_vm = rv.enter(session, &p.a_vm)?;
        a_data
            .send::<Loan<A_FILE>>(rv.enter(session, &p.a_file)?)
            .await?;
        let mut a = a_vm.recv::<Loan<A_FILE>>().await?;
        let mut b_data = rv.enter(session, &p.b_data)?;
        let mut b_vm = rv.enter(session, &p.b_vm)?;
        b_data
            .send::<Loan<B_FILE>>(rv.enter(session, &p.b_file)?)
            .await?;
        let mut b = b_vm.recv::<Loan<B_FILE>>().await?;
        let mut review_data = rv.enter(session, &p.review_data)?;
        let mut review_vm = rv.enter(session, &p.review_vm)?;
        review_data
            .send::<Loan<REVIEW_FILE>>(rv.enter(session, &p.review_file)?)
            .await?;
        let mut review_file = review_vm.recv::<Loan<REVIEW_FILE>>().await?;
        let mut test_data = rv.enter(session, &p.test_data)?;
        let mut test_vm = rv.enter(session, &p.test_vm)?;
        test_data
            .send::<Loan<TEST_FILE>>(rv.enter(session, &p.test_file)?)
            .await?;
        let mut test_file = test_vm.recv::<Loan<TEST_FILE>>().await?;
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
            async {
                review_file
                    .send::<ReviewRequestA>(&Arguments::<6>::try_from(
                        br#"{"pr_id":"module_a"}"#.as_slice(),
                    )?)
                    .await?;
                reviewer.recv::<ReadyReviewA>().await?;
                reviewer.recv::<ReviewRequestA>().await?;
                let bytes = serde_json::to_vec(if bad < 6 { &wrong } else { &review }).unwrap();
                reviewer.send::<ReviewedA>(&bytes.as_slice()).await?;
            };
            async {
                test_file
                    .send::<TestRequestA>(&Arguments::<8>::try_from(
                        br#"{"suite":"module_a_tests"}"#.as_slice(),
                    )?)
                    .await?;
                tester.recv::<ReadyTestA>().await?;
                tester.recv::<TestRequestA>().await?;
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
