//! Receiver checks use actual delegated Endpoints and current material values.
use super::{global::*, imp, localside};
use crate::shell::{
    ShellError, ShellResult,
    imp::{bus::Bus, report_arguments::ReportArguments, service::Domain},
};
use hibana::{
    g,
    runtime::{SessionKitStorage, ids::SessionId, program::project},
};
use serde_json::{Value, json};
use std::cell::Cell;

struct Calls(Cell<Vec<Value>>);
impl Domain for Calls {
    async fn execute(
        &self,
        _agent: &'static str,
        _tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        let mut calls = self.0.take();
        calls.push(arguments);
        self.0.set(calls);
        Err("the substituted request reached an effect".into())
    }
}

#[hibana_runtime::main]
#[test]
async fn substituted_document_requests_are_rejected_before_the_effect() -> ShellResult<()> {
    for supplied in [
        json!({"section_name":SECTION_A,"content_notes":"replaced notes"}),
        json!({"section_name":SECTION_B,"content_notes":"accepted notes"}),
    ] {
        let source = g::seq(
            g::send::<DOCUMENT_SERVICE, A_WASI, DocumentLoan>(),
            g::seq(
                g::send::<A_FILE, DOCUMENT_SERVICE, WriteA>(),
                g::seq(
                    g::send::<DOCUMENT_SERVICE, A_WASI, WriteArgumentsA>(),
                    g::send::<DOCUMENT, DOCUMENT_SERVICE, WriteOnDocument>(),
                ),
            ),
        );
        let p_service = project::<DOCUMENT_SERVICE>(&source);
        let p_document = project::<DOCUMENT>(&source);
        let p_author = project::<A_WASI>(&source);
        let p_file = project::<A_FILE>(&source);
        let session = SessionId::new(3110);
        let (bus, _io, _terminals) = Bus::open(
            ["RESEARCHER_A", "RESEARCHER_B", "EDITOR"],
            [(session, DOCUMENT)],
        );
        let mut slab = vec![0; 1024 * 1024];
        let mut storage = SessionKitStorage::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        let mut service = rv.enter(session, &p_service)?;
        let document = rv.enter(session, &p_document)?;
        let mut author = rv.enter(session, &p_author)?;
        let mut file = rv.enter(session, &p_file)?;
        let domain = Calls(Cell::new(Vec::new()));
        let admitted = ReportArguments::<21>::from_bytes(
            br#"{"section_name":"section_a","content_notes":"accepted notes"}"#,
        )?;
        let supplied = ReportArguments::<2>::from_bytes(&imp::encode(&supplied)?)?;
        let result = hibana_runtime::race(
            async {
                async {
                    localside::resources::document(&mut service, document, &domain).await?;
                };
                async {
                    file.send::<WriteA>(&admitted).await?;
                };
                async {
                    let mut document = author.recv::<DocumentLoan>().await?;
                    let request = author.recv::<WriteArgumentsA>().await?;
                    assert_eq!(request.as_bytes(), admitted.as_bytes());
                    document.send::<WriteOnDocument>(&supplied).await?;
                    Ok::<(), ShellError>(())
                };
                Ok(())
            },
            hibana_runtime::host::deadline(std::time::Duration::from_secs(5)),
        )
        .await;
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("differs from admitted arguments")
        );
        assert!(domain.0.take().is_empty());
    }
    Ok(())
}

#[test]
fn section_receipts_bind_current_material_without_serializing_the_other_section() {
    let artifact = |id: &str| json!({"id":id,"content":format!("actual {id}")});
    let mut snapshot = json!({"sections":{SECTION_A:artifact("a"),SECTION_B:artifact("b")},"references":{SECTION_A:artifact("ra"),SECTION_B:artifact("rb")}});
    let a = json!({"tool_name":"review_sections","success":true,"data":{"agent":"EDITOR","scope":SECTION_A,"review_id":"review:a","status":"reviewed","input_snapshot":{"sections":{SECTION_A:snapshot["sections"][SECTION_A]},"references":{SECTION_A:snapshot["references"][SECTION_A]}}}});
    imp::reviewed(&a, &snapshot, SECTION_A).unwrap();
    snapshot["sections"][SECTION_B] = artifact("revised-b");
    imp::reviewed(&a, &snapshot, SECTION_A).unwrap();
    assert!(imp::reviewed(&a, &snapshot, SECTION_B).is_err());
    snapshot["sections"][SECTION_A] = artifact("revised-a");
    assert!(imp::reviewed(&a, &snapshot, SECTION_A).is_err());
}

#[test]
fn combination_requires_both_current_positive_receipts_and_exact_arguments() {
    let artifact = |id: &str| json!({"id":id,"content":format!("actual {id}")});
    let snapshot = json!({"sections":{SECTION_A:artifact("a"),SECTION_B:artifact("b")},"references":{SECTION_A:artifact("ra"),SECTION_B:artifact("rb")}});
    let review = |section: &str| json!({"tool_name":"review_sections","success":true,"data":{"agent":"EDITOR","scope":section,"review_id":format!("review:{section}"),"status":"reviewed","input_snapshot":{"sections":{section:snapshot["sections"][section]},"references":{section:snapshot["references"][section]}}}});
    let a = review(SECTION_A);
    let b = review(SECTION_B);
    let arguments = json!({"sections":"section_b,section_a"});
    imp::combine_inputs(&arguments, &arguments, (&snapshot, &[a.clone(), b.clone()])).unwrap();
    for reviews in [
        [a.clone(), a.clone()],
        [a.clone(), Value::Null],
        [a.clone(), {
            let mut rejected = b.clone();
            rejected["success"] = json!(false);
            rejected["data"]["status"] = json!("needs_revision");
            rejected
        }],
    ] {
        assert!(imp::combine_inputs(&arguments, &arguments, (&snapshot, &reviews)).is_err());
    }
    assert!(
        imp::combine_inputs(
            &arguments,
            &json!({"sections":"section_a"}),
            (&snapshot, &[a, b])
        )
        .is_err()
    );
}
