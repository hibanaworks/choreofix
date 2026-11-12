//! Exercise the production resource receiver with independent Endpoint callers.
use super::{global::*, localside};
use crate::shell::imp::report_arguments::ReportArguments;
use crate::shell::{
    ShellError,
    imp::{bus::Bus, service::Domain},
};
use hibana::runtime::{SessionKitStorage, ids::SessionId};
use serde_json::{Value, json};

struct NoEffects;
impl Domain for NoEffects {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        panic!("rejected report request reached the effect: {agent} {tool} {arguments}")
    }
}

#[hibana_runtime::main]
#[test]
async fn document_rejects_replaced_notes_and_the_other_researchers_section()
-> Result<(), ShellError> {
    for (admitted, actual) in [
        (
            json!({"section_name":SECTION_A,"content_notes":"reviewed"}),
            json!({"section_name":SECTION_A,"content_notes":"replaced"}),
        ),
        (
            json!({"section_name":SECTION_B}),
            json!({"section_name":SECTION_B}),
        ),
    ] {
        let p = programs();
        let session = SessionId::new(3001);
        let (bus, _reader, _terminals) = Bus::open(
            ["RESEARCHER_A", "RESEARCHER_B", "EDITOR"],
            [(session, DOCUMENT), (session, DATABASE)],
        );
        let mut owner_slab = vec![0; 1024 * 1024];
        let mut owner_storage = SessionKitStorage::uninit();
        let owner = owner_storage
            .init()
            .rendezvous(&mut owner_slab, &bus.transport)?;
        let mut caller_slab = vec![0; 1024 * 1024];
        let mut caller_storage = SessionKitStorage::uninit();
        let caller = caller_storage
            .init()
            .rendezvous(&mut caller_slab, &bus.transport)?;
        let mut receiver = owner.enter(session, &p.document_service)?;
        let document = owner.enter(session, &p.document)?;
        let mut a = caller.enter(session, &p.a_wasi)?;
        let mut b = caller.enter(session, &p.b_wasi)?;
        let mut research_a = caller.enter(session, &p.research_a)?;
        let mut research_b = caller.enter(session, &p.research_b)?;
        let [mut a, _a_private] = a.par()?;
        let [mut b, _b_private] = b.par()?;
        let admitted = ReportArguments::<2>::from_bytes(&serde_json::to_vec(&admitted).unwrap())?;
        let actual = ReportArguments::<2>::from_bytes(&serde_json::to_vec(&actual).unwrap())?;
        let research_a_arguments = ReportArguments::<1>::from_bytes(br#"{"topic":"subtopic_a"}"#)?;
        let research_b_arguments = ReportArguments::<1>::from_bytes(br#"{"topic":"subtopic_b"}"#)?;
        let result = hibana_runtime::race(
            async {
                let (result, held) = async {
                    async {
                        Ok::<_, ShellError>(
                            localside::resources::document(&mut receiver, document, &NoEffects)
                                .await,
                        )
                    };
                    async {
                        a.send::<Research>(&research_a_arguments).await?;
                        b.send::<Research>(&research_b_arguments).await?;
                        research_a.recv::<Research>().await?;
                        research_b.recv::<Research>().await?;
                        research_a
                            .send::<ResearchResult>(&br#"{"success":true}"#.as_slice())
                            .await?;
                        research_b
                            .send::<ResearchResult>(&br#"{"success":true}"#.as_slice())
                            .await?;
                        a.recv::<ResearchResult>().await?;
                        b.recv::<ResearchResult>().await?;
                        a.send::<WriteA>(&admitted).await?;
                        let mut document = a.recv::<DocumentLoan>().await?;
                        assert!(owner.enter(session, &p.document).is_err());
                        assert!(caller.enter(session, &p.document).is_err());
                        document.send::<WriteOnDocument>(&actual).await?;
                        Ok::<_, ShellError>(document)
                    };
                }
                .await?;
                assert_eq!(
                    result.unwrap_err().to_string(),
                    "document write arguments differ"
                );
                drop(held);
                Ok::<_, ShellError>(())
            },
            hibana_runtime::host::deadline(std::time::Duration::from_secs(5)),
        )
        .await;
        if result.is_err() {
            eprintln!(
                "document receiver prefix: {:?}",
                bus.transport.take_observations()
            );
        }
        result?;
    }
    Ok(())
}

struct EchoEffect;
impl Domain for EchoEffect {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        Ok(json!({"success":true,"agent":agent,"tool":tool,"arguments":arguments}))
    }
}

#[hibana_runtime::main]
#[test]
async fn complete_report_uses_both_real_resource_owners_and_both_review_results()
-> Result<(), ShellError> {
    hibana_runtime::race(
        async {
            use super::imp::{ReviewInput, ReviewTarget, decide};
            use crate::shell::imp::review_reply::ReviewReply;
            use hibana::runtime::resolver::ResolverRef;
            let p = programs();
            let session = SessionId::new(3003);
            let (bus, _reader, _terminals) = Bus::open(
                ["RESEARCHER_A", "RESEARCHER_B", "EDITOR"],
                [(session, DOCUMENT), (session, DATABASE)],
            );
            let review_a_input = ReviewInput::default();
            let review_b_input = ReviewInput::default();
            let selection = ReviewTarget::default();
            let mut slab = vec![0; 1024 * 1024];
            let mut storage = SessionKitStorage::uninit();
            let runtime = storage.init().rendezvous(&mut slab, &bus.transport)?;
            selection.record(SECTION_A);
            runtime
                .set_resolver(
                    &p.review_a,
                    ResolverRef::<DECIDE_A>::decision_state(&review_a_input, decide),
                )
                .unwrap();
            runtime
                .set_resolver(
                    &p.review_b,
                    ResolverRef::<DECIDE_B>::decision_state(&review_b_input, decide),
                )
                .unwrap();
            runtime
                .set_resolver(
                    &p.review_wasi,
                    ResolverRef::<SELECT_REVIEW>::decision_state(&selection, ReviewTarget::decide),
                )
                .unwrap();
            let mut document_service = runtime.enter(session, &p.document_service)?;
            let document = runtime.enter(session, &p.document)?;
            let mut database_service = runtime.enter(session, &p.database_service)?;
            let database = runtime.enter(session, &p.database)?;
            let mut research_a = runtime.enter(session, &p.research_a)?;
            let mut research_b = runtime.enter(session, &p.research_b)?;
            let mut review_a = runtime.enter(session, &p.review_a)?;
            let mut review_b = runtime.enter(session, &p.review_b)?;
            let mut a = runtime.enter(session, &p.a_wasi)?;
            let mut b = runtime.enter(session, &p.b_wasi)?;
            let mut editor = runtime.enter(session, &p.review_wasi)?;
            let [mut a, _a_private] = a.par()?;
            let [mut b, _b_private] = b.par()?;
            let [mut editor, _editor_private] = editor.par()?;
            async {
                localside::resources::document(&mut document_service, document, &EchoEffect)
                    .await
                    .map_err(|error| ShellError::message(format!("document receiver: {error}")))?;
            };
            async {
                localside::resources::database(&mut database_service, database, &EchoEffect)
                    .await
                    .map_err(|error| ShellError::message(format!("database receiver: {error}")))?;
            };
            async {
                localside::resources::research::<RESEARCH_A>(&mut research_a, &EchoEffect).await?;
            };
            async {
                localside::resources::research::<RESEARCH_B>(&mut research_b, &EchoEffect).await?;
            };
            async {
                localside::reviewer_a::run(&mut review_a, &EchoEffect, &review_a_input)
                    .await
                    .map_err(|error| ShellError::message(format!("review A: {error}")))?;
            };
            async {
                localside::reviewer_b::run(&mut review_b, &EchoEffect, &review_b_input)
                    .await
                    .map_err(|error| ShellError::message(format!("review B: {error}")))?;
            };
            let (.., mut editor) = async {
        async {
            let research = ReportArguments::<1>::from_bytes(br#"{"topic":"subtopic_a"}"#)?;
            a.send::<Research>(&research).await?;
            a.recv::<ResearchResult>().await?;
            let write = ReportArguments::<2>::from_bytes(
                br#"{"section_name":"section_a","content_notes":"A data"}"#,
            )?;
            a.send::<WriteA>(&write).await?;
            let mut document = a.recv::<DocumentLoan>().await?;
            document.send::<WriteOnDocument>(&write).await?;
            let result = document.recv::<DocumentResult>().await?.to_vec();
            assert_eq!(
                serde_json::from_slice::<Value>(&result).unwrap()["arguments"]["content_notes"],
                "A data"
            );
            a.send::<DocumentLoan>(document).await?;
            let references = ReportArguments::<4>::from_bytes(br#"{"section_name":"section_a"}"#)?;
            a.send::<ReferencesA>(&references).await?;
            let mut database = a.recv::<DatabaseLoan>().await?;
            database.send::<ReferencesOnDatabase>(&references).await?;
            let result = database.recv::<DatabaseResult>().await?.to_vec();
            a.send::<DatabaseLoan>(database).await?;
            a.send::<ReadyA>(&result.as_slice()).await?;
            a.send::<ReadyA>(&result.as_slice()).await?;
            a.recv::<FirstA>().await?;
            a.recv::<ApprovedA>().await?;
            Ok::<_, ShellError>(())
        };
        async {
            let research = ReportArguments::<1>::from_bytes(br#"{"topic":"subtopic_b"}"#)?;
            b.send::<Research>(&research).await?;
            b.recv::<ResearchResult>().await?;
            let write = ReportArguments::<2>::from_bytes(br#"{"section_name":"section_b"}"#)?;
            b.send::<WriteB>(&write).await?;
            let mut document = b.recv::<DocumentLoan>().await?;
            document.send::<WriteOnDocument>(&write).await?;
            document.recv::<DocumentResult>().await?;
            b.send::<DocumentLoan>(document).await?;
            let references = ReportArguments::<4>::from_bytes(br#"{"section_name":"section_b"}"#)?;
            b.send::<ReferencesB>(&references).await?;
            let mut database = b.recv::<DatabaseLoan>().await?;
            database.send::<ReferencesOnDatabase>(&references).await?;
            let result = database.recv::<DatabaseResult>().await?.to_vec();
            b.send::<DatabaseLoan>(database).await?;
            b.send::<ReadyB>(&result.as_slice()).await?;
            b.send::<ReadyB>(&result.as_slice()).await?;
            b.recv::<ApprovedA>().await?;
            b.recv::<BOnly>().await?;
            b.recv::<ApprovedB>().await?;
            Ok::<_, ShellError>(())
        };
        async {
            let a = ReportArguments::<5>::from_bytes(br#"{"section_name":"section_a"}"#)?;
            editor.send::<FirstA>(&a).await?;
            editor.send::<FirstA>(&a).await?;
            editor.send::<FirstA>(&a).await?;
            editor.send::<ReviewA>(&a).await?;
            let result = editor.recv::<ApprovedA>().await?.to_vec();
            selection.received(SECTION_A, &result)?;
            let b = ReportArguments::<5>::from_bytes(br#"{"section_name":"section_b"}"#)?;
            selection.record(SECTION_B);
            editor.send::<BOnly>(&b).await?;
            editor.send::<BOnly>(&b).await?;
            editor.send::<BOnly>(&b).await?;
            editor.send::<ReviewB>(&b).await?;
            let result = editor.recv::<ApprovedB>().await?.to_vec();
            selection.received(SECTION_B, &result)?;
            let (a, b) = selection.approvals()?;
            let a = ReviewReply::<true>::from_bytes(&a)?;
            let b = ReviewReply::<true>::from_bytes(&b)?;
            editor.send::<ConfirmApprovedA>(&a).await?;
            editor.send::<ConfirmApprovedB>(&b).await?;
            let combine =
                ReportArguments::<6>::from_bytes(br#"{"sections":" section_b, section_a "}"#)?;
            editor.send::<Combine>(&combine).await?;
            let mut document = editor.recv::<DocumentLoan>().await?;
            document.send::<CombineOnDocument>(&combine).await?;
            let result = document.recv::<DocumentResult>().await?.to_vec();
            let result: Value = serde_json::from_slice(&result).unwrap();
            assert_eq!(result["tool"], "combine_sections");
            assert_eq!(result["agent"], "EDITOR");
            editor.send::<DocumentLoan>(document).await?;
            Ok::<_, ShellError>(editor)
        };
    }
    .await?;
            editor.send::<DocumentExit>(&0).await?;
            editor.send::<DatabaseExit>(&0).await?;
            {
                let [mut a, mut b] = editor.par()?;
                async {
                    async {
                        a.send::<ResearchExitA>(&0).await?;
                    };
                    async {
                        b.send::<ResearchExitB>(&0).await?;
                    };
                    Ok(())
                }
                .await?;
            }
            Ok(())
        },
        hibana_runtime::host::deadline(std::time::Duration::from_secs(10)),
    )
    .await
}

#[hibana_runtime::main]
#[test]
async fn document_returns_the_actual_effect_result_on_the_delegated_endpoint()
-> Result<(), ShellError> {
    let p = programs();
    let session = SessionId::new(3002);
    let (bus, _reader, _terminals) = Bus::open(
        ["RESEARCHER_A", "RESEARCHER_B", "EDITOR"],
        [(session, DOCUMENT), (session, DATABASE)],
    );
    let mut slab = vec![0; 1024 * 1024];
    let mut storage = SessionKitStorage::uninit();
    let runtime = storage.init().rendezvous(&mut slab, &bus.transport)?;
    let mut receiver = runtime.enter(session, &p.document_service)?;
    let document = runtime.enter(session, &p.document)?;
    let mut a = runtime.enter(session, &p.a_wasi)?;
    let mut b = runtime.enter(session, &p.b_wasi)?;
    let [mut a, _a_private] = a.par()?;
    let [mut b, _b_private] = b.par()?;
    let mut research_a = runtime.enter(session, &p.research_a)?;
    let mut research_b = runtime.enter(session, &p.research_b)?;
    let a_research = ReportArguments::<1>::from_bytes(br#"{"topic":"subtopic_a"}"#)?;
    let b_research = ReportArguments::<1>::from_bytes(br#"{"topic":"subtopic_b"}"#)?;
    let arguments = ReportArguments::<2>::from_bytes(
        br#"{"section_name":"section_a","content_notes":"actual notes"}"#,
    )?;
    let result = hibana_runtime::race(
        async {
            // This test certifies a receiver prefix, not whole-session completion.
            hibana_runtime::race(
                localside::resources::document(&mut receiver, document, &EchoEffect),
                async {
                    a.send::<Research>(&a_research).await?;
                    b.send::<Research>(&b_research).await?;
                    research_a.recv::<Research>().await?;
                    research_b.recv::<Research>().await?;
                    research_a.send::<ResearchResult>(&br#"{"success":true}"#.as_slice()).await?;
                    research_b.send::<ResearchResult>(&br#"{"success":true}"#.as_slice()).await?;
                    a.recv::<ResearchResult>().await?;
                    b.recv::<ResearchResult>().await?;
                    a.send::<WriteA>(&arguments).await?;
                    let mut document = a.recv::<DocumentLoan>().await?;
                    document.send::<WriteOnDocument>(&arguments).await?;
                    let reply = document.recv::<DocumentResult>().await?.to_vec();
                    let reply: Value = serde_json::from_slice(&reply).unwrap();
                    assert_eq!(reply, json!({"success":true,"agent":"RESEARCHER_A","tool":"write_section","arguments":{"section_name":SECTION_A,"content_notes":"actual notes"}}));
                    a.send::<DocumentLoan>(document).await?;
                    Ok::<_, ShellError>(())
                },
            ).await
        },
        hibana_runtime::host::deadline(std::time::Duration::from_secs(5)),
    ).await;
    if result.is_err() {
        eprintln!(
            "document response prefix: {:?}",
            bus.transport.take_observations()
        );
    }
    result?;
    Ok(())
}
