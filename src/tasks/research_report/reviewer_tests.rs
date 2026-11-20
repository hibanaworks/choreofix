//! Actual backend replies and affine finalization through production localsides.
use super::{global::*, imp, localside};
use crate::shell::{
    ShellError, ShellResult,
    global::Loan,
    imp::{
        bus::Bus,
        report_arguments::ReportArguments,
        service::{ReceivedEffects, SimulatorOperation},
    },
};
use hibana::{
    g,
    runtime::{SessionKitStorage, ids::SessionId, program::project},
};
use serde_json::{Value, json};
use std::{cell::Cell, collections::VecDeque};

// Actual supplied effect results and arguments only, with no protocol state.
struct Replies(Cell<(VecDeque<Value>, Vec<Value>)>);
impl ReceivedEffects for Replies {
    fn apply<'io, P: SimulatorOperation>(
        &'io self,
        received: hibana::runtime::wire::ReceivedPayload<'_, P>,
    ) -> impl Future<Output = ShellResult<Value>> + use<'io, P> {
        let arguments = imp::decode(received.as_bytes());
        async move {
            let (agent, tool) = P::RPC;
            let (mut replies, mut calls) = self.0.take();
            calls.push(json!({"agent":agent,"tool":tool,"arguments":arguments?}));
            let reply = replies.pop_front();
            self.0.set((replies, calls));
            reply.ok_or_else(|| ShellError::message("unexpected effect"))
        }
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
                    g::send::<A_WASI, REVIEW_A, ApprovedA>(),
                    g::seq(
                        g::send::<REVIEW_A, REVIEW_WASI, ApprovedA>(),
                        g::seq(
                            g::send::<REVIEW_A, B_WASI, ApprovedA>(),
                            g::send::<B_WASI, REVIEW_A, ApprovedA>(),
                        ),
                    ),
                ),
            ),
        );
        let negative = g::seq(
            g::send::<REVIEW_A, DOCUMENT_SERVICE, RejectedA>(),
            g::seq(
                g::send::<REVIEW_A, A_WASI, RejectedA>(),
                g::seq(
                    g::send::<A_WASI, REVIEW_A, RejectedA>(),
                    g::seq(
                        g::send::<REVIEW_A, REVIEW_WASI, RejectedA>(),
                        g::seq(
                            g::send::<REVIEW_A, B_WASI, RejectedA>(),
                            g::send::<B_WASI, REVIEW_A, RejectedA>(),
                        ),
                    ),
                ),
            ),
        );
        let source = g::seq(
            g::send::<REVIEW_WASI, RESEARCH_A, ReviewRequestA>(),
            g::seq(
                g::route(
                    positive,
                    g::seq(
                        g::seq(
                            negative,
                            g::send::<REVIEW_WASI, RESEARCH_A, ReviewRequestA>(),
                        )
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
        );
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
        let research = rv.enter(session, &p_research)?;
        let review = rv.enter(session, &p_review)?;
        let mut editor = rv.enter(session, &p_editor)?;
        let mut a = rv.enter(session, &p_a)?;
        let mut b = rv.enter(session, &p_b)?;
        let mut doc = rv.enter(session, &p_doc)?;
        let mut editor_file = rv.enter(session, &p_editor_file)?;
        let outcomes:Vec<_>=(0..=rejections).map(|attempt|json!({"success":attempt==rejections,"data":{"review_id":format!("a:{attempt}")}})).collect();
        let domain = Replies(Cell::new((outcomes.clone().into(), Vec::new())));
        let review_arguments =
            ReportArguments::<27, true>::from_bytes(br#"{"section_name":"section_a"}"#)?;
        let combine_arguments =
            ReportArguments::<29>::from_bytes(br#"{"sections":"section_a,section_b"}"#)?;
        let result = hibana_runtime::race(
            async {
                async {
                    localside::reviewer_a::run(review, research, &domain).await?;
                };
                async {
                    for outcome in &outcomes {
                        editor.send::<ReviewRequestA>(&review_arguments).await?;
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
                        if outcome["success"] == true {
                            let bytes = a.recv::<ApprovedA>().await?.to_vec();
                            assert_eq!(serde_json::from_slice::<Value>(&bytes).unwrap(), *outcome);
                            a.send::<ApprovedA>(&crate::shell::imp::review_reply::ReviewReply::<
                                true,
                            >::from_bytes(&bytes)?)
                                .await?;
                        } else {
                            let bytes = a.recv::<RejectedA>().await?.to_vec();
                            assert_eq!(serde_json::from_slice::<Value>(&bytes).unwrap(), *outcome);
                            a.send::<RejectedA>(&crate::shell::imp::review_reply::ReviewReply::<
                                false,
                            >::from_bytes(&bytes)?)
                                .await?;
                        }
                    }
                    Ok::<(), ShellError>(())
                };
                async {
                    for outcome in &outcomes {
                        if outcome["success"] == true {
                            let bytes = b.recv::<ApprovedA>().await?.to_vec();
                            assert_eq!(serde_json::from_slice::<Value>(&bytes).unwrap(), *outcome);
                            b.send::<ApprovedA>(&crate::shell::imp::review_reply::ReviewReply::<
                                true,
                            >::from_bytes(&bytes)?)
                                .await?;
                        } else {
                            let bytes = b.recv::<RejectedA>().await?.to_vec();
                            assert_eq!(serde_json::from_slice::<Value>(&bytes).unwrap(), *outcome);
                            b.send::<RejectedA>(&crate::shell::imp::review_reply::ReviewReply::<
                                false,
                            >::from_bytes(&bytes)?)
                                .await?;
                        }
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
        assert_eq!(calls.len(), outcomes.len());
        assert!(calls.iter().all(|call| call["tool"] == "review_sections"
            && call["arguments"]["section_name"] == SECTION_A));
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
                    g::send::<B_WASI, REVIEW_B, ApprovedB>(),
                    g::seq(
                        g::send::<REVIEW_B, REVIEW_WASI, ApprovedB>(),
                        g::seq(
                            g::send::<REVIEW_B, A_WASI, ApprovedB>(),
                            g::send::<A_WASI, REVIEW_B, ApprovedB>(),
                        ),
                    ),
                ),
            ),
        );
        let negative = g::seq(
            g::send::<REVIEW_B, DOCUMENT_SERVICE, RejectedB>(),
            g::seq(
                g::send::<REVIEW_B, B_WASI, RejectedB>(),
                g::seq(
                    g::send::<B_WASI, REVIEW_B, RejectedB>(),
                    g::seq(
                        g::send::<REVIEW_B, REVIEW_WASI, RejectedB>(),
                        g::seq(
                            g::send::<REVIEW_B, A_WASI, RejectedB>(),
                            g::send::<A_WASI, REVIEW_B, RejectedB>(),
                        ),
                    ),
                ),
            ),
        );
        let source = g::seq(
            g::send::<REVIEW_WASI, RESEARCH_B, ReviewRequestB>(),
            g::seq(
                g::route(
                    positive,
                    g::seq(
                        g::seq(
                            negative,
                            g::send::<REVIEW_WASI, RESEARCH_B, ReviewRequestB>(),
                        )
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
        );
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
        let research = rv.enter(session, &p_research)?;
        let review = rv.enter(session, &p_review)?;
        let mut editor = rv.enter(session, &p_editor)?;
        let mut a = rv.enter(session, &p_a)?;
        let mut b = rv.enter(session, &p_b)?;
        let mut doc = rv.enter(session, &p_doc)?;
        let mut editor_file = rv.enter(session, &p_editor_file)?;
        let outcomes:Vec<_>=(0..=rejections).map(|attempt|json!({"success":attempt==rejections,"data":{"review_id":format!("b:{attempt}")}})).collect();
        let domain = Replies(Cell::new((outcomes.clone().into(), Vec::new())));
        let review_arguments =
            ReportArguments::<28, true>::from_bytes(br#"{"section_name":"section_b"}"#)?;
        let combine_arguments =
            ReportArguments::<29>::from_bytes(br#"{"sections":"section_a,section_b"}"#)?;
        let result = hibana_runtime::race(
            async {
                async {
                    localside::reviewer_b::run(review, research, &domain).await?;
                };
                async {
                    for outcome in &outcomes {
                        editor.send::<ReviewRequestB>(&review_arguments).await?;
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
                        if outcome["success"] == true {
                            let bytes = a.recv::<ApprovedB>().await?.to_vec();
                            assert_eq!(serde_json::from_slice::<Value>(&bytes).unwrap(), *outcome);
                            a.send::<ApprovedB>(&crate::shell::imp::review_reply::ReviewReply::<
                                true,
                            >::from_bytes(&bytes)?)
                                .await?;
                        } else {
                            let bytes = a.recv::<RejectedB>().await?.to_vec();
                            assert_eq!(serde_json::from_slice::<Value>(&bytes).unwrap(), *outcome);
                            a.send::<RejectedB>(&crate::shell::imp::review_reply::ReviewReply::<
                                false,
                            >::from_bytes(&bytes)?)
                                .await?;
                        }
                    }
                    Ok::<(), ShellError>(())
                };
                async {
                    for outcome in &outcomes {
                        if outcome["success"] == true {
                            let bytes = b.recv::<ApprovedB>().await?.to_vec();
                            assert_eq!(serde_json::from_slice::<Value>(&bytes).unwrap(), *outcome);
                            b.send::<ApprovedB>(&crate::shell::imp::review_reply::ReviewReply::<
                                true,
                            >::from_bytes(&bytes)?)
                                .await?;
                        } else {
                            let bytes = b.recv::<RejectedB>().await?.to_vec();
                            assert_eq!(serde_json::from_slice::<Value>(&bytes).unwrap(), *outcome);
                            b.send::<RejectedB>(&crate::shell::imp::review_reply::ReviewReply::<
                                false,
                            >::from_bytes(&bytes)?)
                                .await?;
                        }
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
        assert_eq!(calls.len(), outcomes.len());
        assert!(calls.iter().all(|call| call["tool"] == "review_sections"
            && call["arguments"]["section_name"] == SECTION_B));
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
                    g::send::<A_WASI, REVIEW_A, ApprovedA>(),
                    g::seq(
                        g::send::<REVIEW_A, REVIEW_WASI, ApprovedA>(),
                        g::seq(
                            g::send::<REVIEW_A, B_WASI, ApprovedA>(),
                            g::send::<B_WASI, REVIEW_A, ApprovedA>(),
                        ),
                    ),
                ),
            ),
        );
        let source = g::seq(
            g::send::<REVIEW_WASI, RESEARCH_A, ReviewRequestA>(),
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
        );
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
        let research = rv.enter(session, &p_research)?;
        let review = rv.enter(session, &p_review)?;
        let mut editor = rv.enter(session, &p_editor)?;
        let mut a = rv.enter(session, &p_a)?;
        let mut b = rv.enter(session, &p_b)?;
        let mut doc = rv.enter(session, &p_doc)?;
        let mut editor_file = rv.enter(session, &p_editor_file)?;
        let outcomes:Vec<_>=(0..=rejections).map(|attempt|json!({"success":attempt==rejections,"data":{"review_id":format!("a:{attempt}")}})).collect();
        let domain = Replies(Cell::new((outcomes.clone().into(), Vec::new())));
        let review_arguments =
            ReportArguments::<27, true>::from_bytes(br#"{"section_name":"section_a"}"#)?;
        let combine_arguments =
            ReportArguments::<29>::from_bytes(br#"{"sections":"section_a,section_b"}"#)?;
        let result = hibana_runtime::race(
            async {
                async {
                    localside::reviewer_a::run(review, research, &domain).await?;
                };
                async {
                    for outcome in &outcomes {
                        editor.send::<ReviewRequestA>(&review_arguments).await?;
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
                        if outcome["success"] == true {
                            let bytes = a.recv::<ApprovedA>().await?.to_vec();
                            assert_eq!(serde_json::from_slice::<Value>(&bytes).unwrap(), *outcome);
                            a.send::<ApprovedA>(&crate::shell::imp::review_reply::ReviewReply::<
                                true,
                            >::from_bytes(&bytes)?)
                                .await?;
                        } else {
                            let bytes = a.recv::<RejectedA>().await?.to_vec();
                            assert_eq!(serde_json::from_slice::<Value>(&bytes).unwrap(), *outcome);
                            a.send::<RejectedA>(&crate::shell::imp::review_reply::ReviewReply::<
                                false,
                            >::from_bytes(&bytes)?)
                                .await?;
                        }
                    }
                    Ok::<(), ShellError>(())
                };
                async {
                    for outcome in &outcomes {
                        if outcome["success"] == true {
                            let bytes = b.recv::<ApprovedA>().await?.to_vec();
                            assert_eq!(serde_json::from_slice::<Value>(&bytes).unwrap(), *outcome);
                            b.send::<ApprovedA>(&crate::shell::imp::review_reply::ReviewReply::<
                                true,
                            >::from_bytes(&bytes)?)
                                .await?;
                        } else {
                            let bytes = b.recv::<RejectedA>().await?.to_vec();
                            assert_eq!(serde_json::from_slice::<Value>(&bytes).unwrap(), *outcome);
                            b.send::<RejectedA>(&crate::shell::imp::review_reply::ReviewReply::<
                                false,
                            >::from_bytes(&bytes)?)
                                .await?;
                        }
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
        assert_eq!(calls.len(), outcomes.len());
        assert!(calls.iter().all(|call| call["tool"] == "review_sections"
            && call["arguments"]["section_name"] == SECTION_A));
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
            g::send::<REVIEW_WASI, RESEARCH_A, ReviewRequestA>(),
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
        );
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
        let research = rv.enter(session, &p_research)?;
        let review = rv.enter(session, &p_review)?;
        let mut editor = rv.enter(session, &p_editor)?;
        let domain = Replies(Cell::new(([invalid].into(), Vec::new())));
        let review_args =
            ReportArguments::<27, true>::from_bytes(br#"{"section_name":"section_a"}"#)?;
        let result = hibana_runtime::race(
            async {
                async {
                    localside::reviewer_a::run(review, research, &domain).await?;
                };
                async {
                    editor.send::<ReviewRequestA>(&review_args).await?;
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
        assert_eq!(calls.len(), 1);
    }
    Ok(())
}
