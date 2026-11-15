//! Actual backend replies and affine finalization through production localsides.
use super::{global::*, localside};
use crate::shell::{
    ShellError, ShellResult,
    global::Loan,
    imp::{bus::Bus, report_arguments::ReportArguments, service::Domain},
};
use hibana::{
    g,
    runtime::{SessionKitStorage, ids::SessionId, program::project},
};
use serde_json::{Value, json};
use std::{cell::Cell, collections::VecDeque};

// Actual supplied effect results and arguments only, with no protocol state.
struct Replies(Cell<(VecDeque<Value>, Vec<Value>)>);
impl Domain for Replies {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        let (mut replies, mut calls) = self.0.take();
        calls.push(json!({"agent":agent,"tool":tool,"arguments":arguments}));
        let result = if tool == "research_topic" {
            json!({"success":true,"data":{"topic":arguments["topic"]}})
        } else {
            replies.pop_front().ok_or("unexpected effect")?
        };
        self.0.set((replies, calls));
        Ok(result)
    }
}
#[hibana_runtime::main]
#[test]
async fn section_a_uses_actual_results_and_retires_its_review_authority() -> ShellResult<()> {
    for rejections in [0, 1, 3, 4] {
        let positive = g::seq(
            g::send::<REVIEW_A, DOCUMENT_SERVICE, ApprovedA>(),
            g::seq(
                g::send::<REVIEW_A, A_WASI, ApprovedA>(),
                g::seq(
                    g::send::<REVIEW_A, B_WASI, ApprovedA>(),
                    g::send::<REVIEW_A, REVIEW_WASI, ApprovedA>(),
                ),
            ),
        );
        let negative = g::seq(
            g::send::<REVIEW_A, DOCUMENT_SERVICE, RejectedA>(),
            g::seq(
                g::send::<REVIEW_A, A_WASI, RejectedA>(),
                g::seq(
                    g::send::<REVIEW_A, B_WASI, RejectedA>(),
                    g::send::<REVIEW_A, REVIEW_WASI, RejectedA>(),
                ),
            ),
        );
        let source = g::seq(
            g::send::<A_FILE, RESEARCH_A, ResearchA>(),
            g::seq(
                g::send::<RESEARCH_A, A_FILE, ResearchResult>(),
                g::seq(
                    g::send::<REVIEW_WASI, RESEARCH_A, ReviewA>(),
                    g::seq(
                        g::route(
                            positive,
                            g::seq(
                                g::seq(negative, g::send::<REVIEW_WASI, RESEARCH_A, ReviewA>())
                                    .roll(),
                                positive,
                            ),
                        ),
                        g::seq(
                            g::send::<EDITOR_FILE, REVIEW_WASI, Combine>(),
                            g::seq(
                                g::send::<REVIEW_WASI, RESEARCH_A, ConfirmApprovedA>(),
                                g::seq(
                                    g::send::<RESEARCH_A, DOCUMENT_SERVICE, ConfirmApprovedA>(),
                                    g::send::<RESEARCH_A, REVIEW_WASI, Loan<REVIEW_A>>(),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        );
        let p_file = project::<A_FILE>(&source);
        let p_research = project::<RESEARCH_A>(&source);
        let p_review = project::<REVIEW_A>(&source);
        let p_editor = project::<REVIEW_WASI>(&source);
        let p_a = project::<A_WASI>(&source);
        let p_b = project::<B_WASI>(&source);
        let p_doc = project::<DOCUMENT_SERVICE>(&source);
        let p_editor_file = project::<EDITOR_FILE>(&source);
        let (bus, _io, _terminals) = Bus::open(
            ["RESEARCHER_A", "RESEARCHER_B", "EDITOR"],
            [(SessionId::new(3101), REVIEW_A)],
        );
        let captured = Cell::new(Vec::new());
        let capture = |event| {
            let mut rows = captured.take();
            rows.push(event);
            captured.set(rows);
        };
        let mut slab = vec![0; 1024 * 1024];
        let mut storage = SessionKitStorage::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        rv.observe(&capture);
        let session = SessionId::new(3101);
        let mut file = rv.enter(session, &p_file)?;
        let research = rv.enter(session, &p_research)?;
        let review = rv.enter(session, &p_review)?;
        let mut editor = rv.enter(session, &p_editor)?;
        let mut a = rv.enter(session, &p_a)?;
        let mut b = rv.enter(session, &p_b)?;
        let mut doc = rv.enter(session, &p_doc)?;
        let mut editor_file = rv.enter(session, &p_editor_file)?;
        let outcomes:Vec<_>=(0..=rejections).map(|attempt|json!({"success":attempt==rejections,"data":{"review_id":format!("a:{attempt}")}})).collect();
        let domain = Replies(Cell::new((outcomes.clone().into(), Vec::new())));
        let research_arguments = ReportArguments::<19>::from_bytes(br#"{"topic":"subtopic_a"}"#)?;
        let review_arguments =
            ReportArguments::<27>::from_bytes(br#"{"section_name":"section_a"}"#)?;
        let combine_arguments =
            ReportArguments::<29>::from_bytes(br#"{"sections":"section_a,section_b"}"#)?;
        let result = hibana_runtime::race(
            async {
                async {
                    localside::reviewer_a::run(review, research, &domain).await?;
                };
                async {
                    file.send::<ResearchA>(&research_arguments).await?;
                    file.recv::<ResearchResult>().await?;
                    Ok::<(), ShellError>(())
                };
                async {
                    for outcome in &outcomes {
                        editor.send::<ReviewA>(&review_arguments).await?;
                        let received = if outcome["success"] == true {
                            editor.recv::<ApprovedA>().await?
                        } else {
                            editor.recv::<RejectedA>().await?
                        };
                        assert_eq!(serde_json::from_slice::<Value>(received).unwrap(), *outcome);
                    }
                    editor.recv::<Combine>().await?;
                    let bytes = serde_json::to_vec(outcomes.last().unwrap()).unwrap();
                    let receipt =
                        crate::shell::imp::review_reply::ReviewReply::<true>::from_bytes(&bytes)?;
                    editor.send::<ConfirmApprovedA>(&receipt).await?;
                    let _retired = editor.recv::<Loan<REVIEW_A>>().await?;
                    Ok::<(), ShellError>(())
                };
                async {
                    editor_file.send::<Combine>(&combine_arguments).await?;
                };
                async {
                    for outcome in &outcomes {
                        let bytes = if outcome["success"] == true {
                            a.recv::<ApprovedA>().await?
                        } else {
                            a.recv::<RejectedA>().await?
                        };
                        assert_eq!(serde_json::from_slice::<Value>(bytes).unwrap(), *outcome);
                    }
                    Ok::<(), ShellError>(())
                };
                async {
                    for outcome in &outcomes {
                        let bytes = if outcome["success"] == true {
                            b.recv::<ApprovedA>().await?
                        } else {
                            b.recv::<RejectedA>().await?
                        };
                        assert_eq!(serde_json::from_slice::<Value>(bytes).unwrap(), *outcome);
                    }
                    Ok::<(), ShellError>(())
                };
                async {
                    for outcome in &outcomes {
                        let bytes = if outcome["success"] == true {
                            doc.recv::<ApprovedA>().await?
                        } else {
                            doc.recv::<RejectedA>().await?
                        };
                        assert_eq!(serde_json::from_slice::<Value>(bytes).unwrap(), *outcome);
                    }
                    let receipt = doc.recv::<ConfirmApprovedA>().await?;
                    assert_eq!(
                        serde_json::from_slice::<Value>(receipt).unwrap(),
                        *outcomes.last().unwrap()
                    );
                    Ok::<(), ShellError>(())
                };
                Ok(())
            },
            hibana_runtime::host::deadline(std::time::Duration::from_secs(5)),
        )
        .await;
        if result.is_err() {
            eprintln!(
                "rejections={rejections} records={:?} calls={:?}",
                captured.take(),
                domain.0.take()
            );
        }
        result?;
        let (remaining, calls) = domain.0.take();
        assert!(remaining.is_empty());
        assert_eq!(calls.len(), outcomes.len() + 1);
        assert_eq!(calls[0]["arguments"]["topic"], "subtopic_a");
        assert!(
            calls[1..]
                .iter()
                .all(|call| call["tool"] == "review_sections"
                    && call["arguments"]["section_name"] == SECTION_A)
        );
    }
    Ok(())
}
#[hibana_runtime::main]
#[test]
async fn section_b_uses_actual_results_and_retires_its_review_authority() -> ShellResult<()> {
    for rejections in [0, 1, 3, 4] {
        let positive = g::seq(
            g::send::<REVIEW_B, DOCUMENT_SERVICE, ApprovedB>(),
            g::seq(
                g::send::<REVIEW_B, B_WASI, ApprovedB>(),
                g::seq(
                    g::send::<REVIEW_B, A_WASI, ApprovedB>(),
                    g::send::<REVIEW_B, REVIEW_WASI, ApprovedB>(),
                ),
            ),
        );
        let negative = g::seq(
            g::send::<REVIEW_B, DOCUMENT_SERVICE, RejectedB>(),
            g::seq(
                g::send::<REVIEW_B, B_WASI, RejectedB>(),
                g::seq(
                    g::send::<REVIEW_B, A_WASI, RejectedB>(),
                    g::send::<REVIEW_B, REVIEW_WASI, RejectedB>(),
                ),
            ),
        );
        let source = g::seq(
            g::send::<B_FILE, RESEARCH_B, ResearchB>(),
            g::seq(
                g::send::<RESEARCH_B, B_FILE, ResearchResult>(),
                g::seq(
                    g::send::<REVIEW_WASI, RESEARCH_B, ReviewB>(),
                    g::seq(
                        g::route(
                            positive,
                            g::seq(
                                g::seq(negative, g::send::<REVIEW_WASI, RESEARCH_B, ReviewB>())
                                    .roll(),
                                positive,
                            ),
                        ),
                        g::seq(
                            g::send::<EDITOR_FILE, REVIEW_WASI, Combine>(),
                            g::seq(
                                g::send::<REVIEW_WASI, RESEARCH_B, ConfirmApprovedB>(),
                                g::seq(
                                    g::send::<RESEARCH_B, DOCUMENT_SERVICE, ConfirmApprovedB>(),
                                    g::send::<RESEARCH_B, REVIEW_WASI, Loan<REVIEW_B>>(),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        );
        let p_file = project::<B_FILE>(&source);
        let p_research = project::<RESEARCH_B>(&source);
        let p_review = project::<REVIEW_B>(&source);
        let p_editor = project::<REVIEW_WASI>(&source);
        let p_a = project::<A_WASI>(&source);
        let p_b = project::<B_WASI>(&source);
        let p_doc = project::<DOCUMENT_SERVICE>(&source);
        let p_editor_file = project::<EDITOR_FILE>(&source);
        let (bus, _io, _terminals) = Bus::open(
            ["RESEARCHER_A", "RESEARCHER_B", "EDITOR"],
            [(SessionId::new(3102), REVIEW_B)],
        );
        let captured = Cell::new(Vec::new());
        let capture = |event| {
            let mut rows = captured.take();
            rows.push(event);
            captured.set(rows);
        };
        let mut slab = vec![0; 1024 * 1024];
        let mut storage = SessionKitStorage::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        rv.observe(&capture);
        let session = SessionId::new(3102);
        let mut file = rv.enter(session, &p_file)?;
        let research = rv.enter(session, &p_research)?;
        let review = rv.enter(session, &p_review)?;
        let mut editor = rv.enter(session, &p_editor)?;
        let mut a = rv.enter(session, &p_a)?;
        let mut b = rv.enter(session, &p_b)?;
        let mut doc = rv.enter(session, &p_doc)?;
        let mut editor_file = rv.enter(session, &p_editor_file)?;
        let outcomes:Vec<_>=(0..=rejections).map(|attempt|json!({"success":attempt==rejections,"data":{"review_id":format!("b:{attempt}")}})).collect();
        let domain = Replies(Cell::new((outcomes.clone().into(), Vec::new())));
        let research_arguments = ReportArguments::<20>::from_bytes(br#"{"topic":"subtopic_b"}"#)?;
        let review_arguments =
            ReportArguments::<28>::from_bytes(br#"{"section_name":"section_b"}"#)?;
        let combine_arguments =
            ReportArguments::<29>::from_bytes(br#"{"sections":"section_a,section_b"}"#)?;
        let result = hibana_runtime::race(
            async {
                async {
                    localside::reviewer_b::run(review, research, &domain).await?;
                };
                async {
                    file.send::<ResearchB>(&research_arguments).await?;
                    file.recv::<ResearchResult>().await?;
                    Ok::<(), ShellError>(())
                };
                async {
                    for outcome in &outcomes {
                        editor.send::<ReviewB>(&review_arguments).await?;
                        let received = if outcome["success"] == true {
                            editor.recv::<ApprovedB>().await?
                        } else {
                            editor.recv::<RejectedB>().await?
                        };
                        assert_eq!(serde_json::from_slice::<Value>(received).unwrap(), *outcome);
                    }
                    editor.recv::<Combine>().await?;
                    let bytes = serde_json::to_vec(outcomes.last().unwrap()).unwrap();
                    let receipt =
                        crate::shell::imp::review_reply::ReviewReply::<true>::from_bytes(&bytes)?;
                    editor.send::<ConfirmApprovedB>(&receipt).await?;
                    let _retired = editor.recv::<Loan<REVIEW_B>>().await?;
                    Ok::<(), ShellError>(())
                };
                async {
                    editor_file.send::<Combine>(&combine_arguments).await?;
                };
                async {
                    for outcome in &outcomes {
                        let bytes = if outcome["success"] == true {
                            a.recv::<ApprovedB>().await?
                        } else {
                            a.recv::<RejectedB>().await?
                        };
                        assert_eq!(serde_json::from_slice::<Value>(bytes).unwrap(), *outcome);
                    }
                    Ok::<(), ShellError>(())
                };
                async {
                    for outcome in &outcomes {
                        let bytes = if outcome["success"] == true {
                            b.recv::<ApprovedB>().await?
                        } else {
                            b.recv::<RejectedB>().await?
                        };
                        assert_eq!(serde_json::from_slice::<Value>(bytes).unwrap(), *outcome);
                    }
                    Ok::<(), ShellError>(())
                };
                async {
                    for outcome in &outcomes {
                        let bytes = if outcome["success"] == true {
                            doc.recv::<ApprovedB>().await?
                        } else {
                            doc.recv::<RejectedB>().await?
                        };
                        assert_eq!(serde_json::from_slice::<Value>(bytes).unwrap(), *outcome);
                    }
                    let receipt = doc.recv::<ConfirmApprovedB>().await?;
                    assert_eq!(
                        serde_json::from_slice::<Value>(receipt).unwrap(),
                        *outcomes.last().unwrap()
                    );
                    Ok::<(), ShellError>(())
                };
                Ok(())
            },
            hibana_runtime::host::deadline(std::time::Duration::from_secs(5)),
        )
        .await;
        if result.is_err() {
            eprintln!(
                "rejections={rejections} records={:?} calls={:?}",
                captured.take(),
                domain.0.take()
            );
        }
        result?;
        let (remaining, calls) = domain.0.take();
        assert!(remaining.is_empty());
        assert_eq!(calls.len(), outcomes.len() + 1);
        assert_eq!(calls[0]["arguments"]["topic"], "subtopic_b");
        assert!(
            calls[1..]
                .iter()
                .all(|call| call["tool"] == "review_sections"
                    && call["arguments"]["section_name"] == SECTION_B)
        );
    }
    Ok(())
}
#[hibana_runtime::main]
#[test]
async fn section_a_single_review_retires_without_a_repeatable_fragment() -> ShellResult<()> {
    for rejections in [0] {
        let positive = g::seq(
            g::send::<REVIEW_A, DOCUMENT_SERVICE, ApprovedA>(),
            g::seq(
                g::send::<REVIEW_A, A_WASI, ApprovedA>(),
                g::seq(
                    g::send::<REVIEW_A, B_WASI, ApprovedA>(),
                    g::send::<REVIEW_A, REVIEW_WASI, ApprovedA>(),
                ),
            ),
        );
        let source = g::seq(
            g::send::<A_FILE, RESEARCH_A, ResearchA>(),
            g::seq(
                g::send::<RESEARCH_A, A_FILE, ResearchResult>(),
                g::seq(
                    g::send::<REVIEW_WASI, RESEARCH_A, ReviewA>(),
                    g::seq(
                        positive,
                        g::seq(
                            g::send::<EDITOR_FILE, REVIEW_WASI, Combine>(),
                            g::seq(
                                g::send::<REVIEW_WASI, RESEARCH_A, ConfirmApprovedA>(),
                                g::seq(
                                    g::send::<RESEARCH_A, DOCUMENT_SERVICE, ConfirmApprovedA>(),
                                    g::send::<RESEARCH_A, REVIEW_WASI, Loan<REVIEW_A>>(),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        );
        let p_file = project::<A_FILE>(&source);
        let p_research = project::<RESEARCH_A>(&source);
        let p_review = project::<REVIEW_A>(&source);
        let p_editor = project::<REVIEW_WASI>(&source);
        let p_a = project::<A_WASI>(&source);
        let p_b = project::<B_WASI>(&source);
        let p_doc = project::<DOCUMENT_SERVICE>(&source);
        let p_editor_file = project::<EDITOR_FILE>(&source);
        let (bus, _io, _terminals) = Bus::open(
            ["RESEARCHER_A", "RESEARCHER_B", "EDITOR"],
            [(SessionId::new(3101), REVIEW_A)],
        );
        let captured = Cell::new(Vec::new());
        let capture = |event| {
            let mut rows = captured.take();
            rows.push(event);
            captured.set(rows);
        };
        let mut slab = vec![0; 1024 * 1024];
        let mut storage = SessionKitStorage::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        rv.observe(&capture);
        let session = SessionId::new(3101);
        let mut file = rv.enter(session, &p_file)?;
        let research = rv.enter(session, &p_research)?;
        let review = rv.enter(session, &p_review)?;
        let mut editor = rv.enter(session, &p_editor)?;
        let mut a = rv.enter(session, &p_a)?;
        let mut b = rv.enter(session, &p_b)?;
        let mut doc = rv.enter(session, &p_doc)?;
        let mut editor_file = rv.enter(session, &p_editor_file)?;
        let outcomes:Vec<_>=(0..=rejections).map(|attempt|json!({"success":attempt==rejections,"data":{"review_id":format!("a:{attempt}")}})).collect();
        let domain = Replies(Cell::new((outcomes.clone().into(), Vec::new())));
        let research_arguments = ReportArguments::<19>::from_bytes(br#"{"topic":"subtopic_a"}"#)?;
        let review_arguments =
            ReportArguments::<27>::from_bytes(br#"{"section_name":"section_a"}"#)?;
        let combine_arguments =
            ReportArguments::<29>::from_bytes(br#"{"sections":"section_a,section_b"}"#)?;
        let result = hibana_runtime::race(
            async {
                async {
                    localside::reviewer_a::run(review, research, &domain).await?;
                };
                async {
                    file.send::<ResearchA>(&research_arguments).await?;
                    file.recv::<ResearchResult>().await?;
                    Ok::<(), ShellError>(())
                };
                async {
                    for outcome in &outcomes {
                        editor.send::<ReviewA>(&review_arguments).await?;
                        let received = if outcome["success"] == true {
                            editor.recv::<ApprovedA>().await?
                        } else {
                            editor.recv::<RejectedA>().await?
                        };
                        assert_eq!(serde_json::from_slice::<Value>(received).unwrap(), *outcome);
                    }
                    editor.recv::<Combine>().await?;
                    let bytes = serde_json::to_vec(outcomes.last().unwrap()).unwrap();
                    let receipt =
                        crate::shell::imp::review_reply::ReviewReply::<true>::from_bytes(&bytes)?;
                    editor.send::<ConfirmApprovedA>(&receipt).await?;
                    let _retired = editor.recv::<Loan<REVIEW_A>>().await?;
                    Ok::<(), ShellError>(())
                };
                async {
                    editor_file.send::<Combine>(&combine_arguments).await?;
                };
                async {
                    for outcome in &outcomes {
                        let bytes = if outcome["success"] == true {
                            a.recv::<ApprovedA>().await?
                        } else {
                            a.recv::<RejectedA>().await?
                        };
                        assert_eq!(serde_json::from_slice::<Value>(bytes).unwrap(), *outcome);
                    }
                    Ok::<(), ShellError>(())
                };
                async {
                    for outcome in &outcomes {
                        let bytes = if outcome["success"] == true {
                            b.recv::<ApprovedA>().await?
                        } else {
                            b.recv::<RejectedA>().await?
                        };
                        assert_eq!(serde_json::from_slice::<Value>(bytes).unwrap(), *outcome);
                    }
                    Ok::<(), ShellError>(())
                };
                async {
                    for outcome in &outcomes {
                        let bytes = if outcome["success"] == true {
                            doc.recv::<ApprovedA>().await?
                        } else {
                            doc.recv::<RejectedA>().await?
                        };
                        assert_eq!(serde_json::from_slice::<Value>(bytes).unwrap(), *outcome);
                    }
                    let receipt = doc.recv::<ConfirmApprovedA>().await?;
                    assert_eq!(
                        serde_json::from_slice::<Value>(receipt).unwrap(),
                        *outcomes.last().unwrap()
                    );
                    Ok::<(), ShellError>(())
                };
                Ok(())
            },
            hibana_runtime::host::deadline(std::time::Duration::from_secs(5)),
        )
        .await;
        if result.is_err() {
            eprintln!(
                "rejections={rejections} records={:?} calls={:?}",
                captured.take(),
                domain.0.take()
            );
        }
        result?;
        let (remaining, calls) = domain.0.take();
        assert!(remaining.is_empty());
        assert_eq!(calls.len(), outcomes.len() + 1);
        assert_eq!(calls[0]["arguments"]["topic"], "subtopic_a");
        assert!(
            calls[1..]
                .iter()
                .all(|call| call["tool"] == "review_sections"
                    && call["arguments"]["section_name"] == SECTION_A)
        );
    }
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn malformed_review_verdict_never_sends_an_approval() -> ShellResult<()> {
    for invalid in [
        json!({}),
        json!({"success":"true"}),
        json!({"success":null}),
    ] {
        let source = g::seq(
            g::send::<A_FILE, RESEARCH_A, ResearchA>(),
            g::seq(
                g::send::<RESEARCH_A, A_FILE, ResearchResult>(),
                g::seq(
                    g::send::<REVIEW_WASI, RESEARCH_A, ReviewA>(),
                    g::seq(
                        g::send::<REVIEW_A, DOCUMENT_SERVICE, ApprovedA>(),
                        g::seq(
                            g::send::<REVIEW_A, A_WASI, ApprovedA>(),
                            g::seq(
                                g::send::<REVIEW_A, B_WASI, ApprovedA>(),
                                g::send::<REVIEW_A, REVIEW_WASI, ApprovedA>(),
                            ),
                        ),
                    ),
                ),
            ),
        );
        let p_file = project::<A_FILE>(&source);
        let p_research = project::<RESEARCH_A>(&source);
        let p_review = project::<REVIEW_A>(&source);
        let p_editor = project::<REVIEW_WASI>(&source);
        let (bus, _io, _terminals) = Bus::open(["RESEARCHER_A", "RESEARCHER_B", "EDITOR"], []);
        let captured = Cell::new(Vec::new());
        let capture = |event| {
            let mut rows = captured.take();
            rows.push(event);
            captured.set(rows);
        };
        let mut slab = vec![0; 1024 * 1024];
        let mut storage = SessionKitStorage::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        rv.observe(&capture);
        let session = SessionId::new(3111);
        let mut file = rv.enter(session, &p_file)?;
        let research = rv.enter(session, &p_research)?;
        let review = rv.enter(session, &p_review)?;
        let mut editor = rv.enter(session, &p_editor)?;
        let domain = Replies(Cell::new(([invalid].into(), Vec::new())));
        let research_args = ReportArguments::<19>::from_bytes(br#"{"topic":"subtopic_a"}"#)?;
        let review_args = ReportArguments::<27>::from_bytes(br#"{"section_name":"section_a"}"#)?;
        let result = hibana_runtime::race(
            async {
                async {
                    localside::reviewer_a::run(review, research, &domain).await?;
                };
                async {
                    file.send::<ResearchA>(&research_args).await?;
                    file.recv::<ResearchResult>().await?;
                    Ok::<(), ShellError>(())
                };
                async {
                    editor.send::<ReviewA>(&review_args).await?;
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
                .contains("review result has no success field")
        );
        assert!(
            captured
                .take()
                .iter()
                .all(|event| event.id() != hibana::runtime::tap::ENDPOINT_SEND
                    || event.arg1() >> 24 != u32::from(REVIEW_A))
        );
        let (remaining, calls) = domain.0.take();
        assert!(remaining.is_empty());
        assert_eq!(calls.len(), 2);
    }
    Ok(())
}
