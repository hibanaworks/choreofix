use super::{global::*, imp};
use serde_json::{Value, json};

// Actual attempted effect arguments only; this journal grants no operation.
struct EffectRequests(std::cell::Cell<Vec<Value>>);
impl crate::shell::imp::service::Domain for EffectRequests {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        let mut calls = self.0.take();
        calls.push(json!({"agent":agent,"tool":tool,"arguments":arguments}));
        self.0.set(calls);
        Err("a rejected document request reached the effect".into())
    }
}

fn snapshot() -> Value {
    json!({"sections":{
        CHAPTER_A:{"id":"document:1","content":"chapter one actual notes"},
        CHAPTER_B:{"id":"document:2","content":"chapter two actual notes"}},
        "references":{
        CHAPTER_A:{"id":"references:3","content":"chapter one references"},
        CHAPTER_B:{"id":"references:4","content":"chapter two references"}}})
}
fn reviews(snapshot: &Value) -> [Value; 2] {
    [CHAPTER_A, CHAPTER_B].map(|scope| {
        json!({"tool_name":"review_sections","success":true,
        "data":{"agent":"EDITOR","scope":scope,"status":"reviewed",
            "review_id":format!("review:{scope}"),"input_snapshot":snapshot}})
    })
}

#[test]
fn document_material_keeps_the_actual_input_and_allows_cross_chapter_revision() {
    for (agent, section, tool) in [
        ("WRITER_A", CHAPTER_A, "write_section"),
        ("WRITER_B", CHAPTER_B, "update_references"),
        ("WRITER_A", CHAPTER_B, "revise_section"),
        ("WRITER_B", CHAPTER_A, "revise_section"),
    ] {
        let arguments = json!({"section_name":section});
        let result = json!({"tool_name":tool,"success":true,"data":{
            "agent":agent,"section":section,"artifact":{"id":"material:7","content":"actual content"}}});
        assert_eq!(
            imp::material(&result, &arguments, (agent, tool)).unwrap(),
            result["data"]["artifact"]
        );
        for mutation in 0..5 {
            let mut changed = result.clone();
            match mutation {
                0 => changed["success"] = json!(false),
                1 => changed["data"]["section"] = json!("another chapter"),
                2 => changed["data"]["agent"] = json!("another writer"),
                3 => changed["data"]["artifact"]["content"] = Value::Null,
                _ => changed["tool_name"] = json!("another effect"),
            }
            assert!(imp::material(&changed, &arguments, (agent, tool)).is_err());
        }
    }
}

#[test]
fn both_current_complete_document_reviews_are_required_before_combination() {
    let snapshot = snapshot();
    let reviews = reviews(&snapshot);
    let request = json!({"sections":"CHAPTER_1, CHAPTER_2"});
    imp::combine_inputs(&request, &request, (&snapshot, &reviews)).unwrap();
    for mutation in 0..9 {
        let mut resource = request.clone();
        let mut current = snapshot.clone();
        let mut review = reviews.clone();
        match mutation {
            0 => resource["sections"] = json!(CHAPTER_A),
            1 => current["sections"][CHAPTER_A]["content"] = json!("later actual edit"),
            2 => current["references"][CHAPTER_B]["id"] = json!("references:later"),
            3 => review[0]["success"] = json!(false),
            4 => review[1]["data"]["scope"] = json!(CHAPTER_A),
            5 => review[1]["data"]["review_id"] = review[0]["data"]["review_id"].clone(),
            6 => {
                current["sections"] = Value::Null;
                for value in &mut review {
                    value["data"]["input_snapshot"] = current.clone();
                }
            }
            7 => review[0]["data"]["agent"] = json!("WRITER_A"),
            _ => review[1]["data"]["input_snapshot"] = Value::Null,
        }
        assert!(
            imp::combine_inputs(&request, &resource, (&current, &review)).is_err(),
            "mutation={mutation}"
        );
    }
    let mut changed = snapshot.clone();
    changed["sections"][CHAPTER_B] =
        json!({"id":"document:later","content":"cross-chapter correction"});
    let fresh = self::reviews(&changed);
    imp::combine_inputs(&request, &request, (&changed, &fresh)).unwrap();
}

#[test]
fn combined_receipts_bind_both_reviews_and_the_current_material() {
    let snapshot = snapshot();
    let reviews = reviews(&snapshot);
    let result = json!({"tool_name":"combine_sections","success":true,"data":{
        "agent":"EDITOR","sections":[CHAPTER_A,CHAPTER_B],"artifact_id":"combined:8",
        "input_snapshot":snapshot,"review_receipts":{
            CHAPTER_A:reviews[0]["data"]["review_id"], CHAPTER_B:reviews[1]["data"]["review_id"]}}});
    imp::combined(&result, &snapshot, &reviews).unwrap();
    for mutation in 0..6 {
        let mut changed = result.clone();
        match mutation {
            0 => changed["data"]["sections"] = json!([CHAPTER_A]),
            1 => {
                changed["data"]["input_snapshot"]["sections"][CHAPTER_A]["content"] =
                    json!("substitution")
            }
            2 => changed["data"]["review_receipts"][CHAPTER_B] = json!("old review"),
            3 => changed["success"] = json!(false),
            4 => changed["data"]["agent"] = json!("WRITER_A"),
            _ => changed["data"]["artifact_id"] = Value::Null,
        }
        assert!(imp::combined(&changed, &snapshot, &reviews).is_err());
    }
}

#[test]
fn complete_source_projects_and_exports_checked_actor_bindings() {
    use hibana::runtime::tap;
    use hibana_epf::Protocol;
    let p = programs();
    let snapshots = [
        tap::projected(&p.a).collect::<Vec<_>>(),
        tap::projected(&p.b).collect(),
        tap::projected(&p.editor).collect(),
        tap::projected(&p.document).collect(),
        tap::projected(&p.service).collect(),
        tap::projected(&p.a_io).collect(),
        tap::projected(&p.b_io).collect(),
        tap::projected(&p.editor_io).collect(),
    ];
    for rows in snapshots {
        let mut storage = vec![0; Protocol::workspace_len(&rows).unwrap()];
        Protocol::new(&rows, &mut storage).unwrap();
    }
}

#[hibana_runtime::main]
#[test]
async fn the_real_document_receiver_rejects_substituted_arguments_and_wrong_initial_chapters()
-> crate::shell::ShellResult<()> {
    use super::localside;
    use crate::{
        shell::imp::report_arguments::ReportArguments,
        shell::{ShellError, imp::bus::Bus},
    };
    use hibana::runtime::{
        SessionKitStorage,
        ids::SessionId,
        resolver::{DecisionArm, ResolverRef},
    };
    let p = programs();
    for first in [DecisionArm::Left, DecisionArm::Right] {
        for wrong_chapter in [false, true] {
            let session = SessionId::new(SESSION + 1);
            let (bus, _io, _terminals) =
                Bus::open(["WRITER_A", "WRITER_B", "EDITOR"], [(session, DOCUMENT)]);
            let mut owner_slab = vec![0; 1024 * 1024];
            let mut caller_slab = vec![0; 1024 * 1024];
            let mut owner_store = SessionKitStorage::uninit();
            let mut caller_store = SessionKitStorage::uninit();
            let owner = owner_store
                .init()
                .rendezvous(&mut owner_slab, &bus.transport)?;
            let caller = caller_store
                .init()
                .rendezvous(&mut caller_slab, &bus.transport)?;
            let mut service = owner.enter(session, &p.service)?;
            let document = owner.enter(session, &p.document)?;
            let mut a = caller.enter(session, &p.a)?;
            let mut b = caller.enter(session, &p.b)?;
            let [mut a, _a_private] = a.par()?;
            let [mut b, _b_private] = b.par()?;
            owner
                .set_resolver(
                    &p.service,
                    ResolverRef::<FIRST_WRITER>::decision_state(&first, |arm| Ok(*arm)),
                )
                .map_err(|_| ShellError::message("initial document resolver"))?;
            let assigned = match first {
                DecisionArm::Left => CHAPTER_A,
                DecisionArm::Right => CHAPTER_B,
            };
            let other = if assigned == CHAPTER_A {
                CHAPTER_B
            } else {
                CHAPTER_A
            };
            let admitted = json!({"section_name":if wrong_chapter { other } else { assigned },"content_notes":"admitted actual notes"});
            let substituted = json!({"section_name":assigned,"content_notes":"substituted notes"});
            let admitted =
                ReportArguments::<2>::from_bytes(&serde_json::to_vec(&admitted).unwrap())?;
            let substituted =
                ReportArguments::<2>::from_bytes(&serde_json::to_vec(&substituted).unwrap())?;
            let effects = EffectRequests(std::cell::Cell::new(Vec::new()));
            let result = hibana_runtime::race(
                hibana_runtime::race(
                    async {
                        let _document =
                            localside::document::run(&mut service, (document, first), &effects)
                                .await?;
                        Ok::<(), ShellError>(())
                    },
                    async {
                        let mut document = match first {
                            DecisionArm::Left => {
                                let document = a.recv::<InitialDocument>().await?;
                                a.send::<WriteA>(&admitted).await?;
                                document
                            }
                            DecisionArm::Right => {
                                let document = b.recv::<InitialDocument>().await?;
                                b.send::<WriteB>(&admitted).await?;
                                document
                            }
                        };
                        assert!(owner.enter(session, &p.document).is_err());
                        assert!(caller.enter(session, &p.document).is_err());
                        document.send::<WriteOnDocument>(&substituted).await?;
                        document.recv::<Written>().await?;
                        Err::<(), _>(ShellError::message("the changed request was accepted"))
                    },
                ),
                hibana_runtime::host::deadline(std::time::Duration::from_secs(5)),
            )
            .await;
            let expected = if wrong_chapter {
                if assigned == CHAPTER_A {
                    "initial chapter A differs"
                } else {
                    "initial chapter B differs"
                }
            } else {
                "document request differs from admitted arguments"
            };
            assert!(
                result
                    .as_ref()
                    .is_err_and(|error| error.to_string() == expected),
                "{result:?}"
            );
            assert!(
                effects.0.take().is_empty(),
                "rejection must precede the real effect entry"
            );
        }
    }
    Ok(())
}

// Drive the canonical source through alternate Endpoint calls, without the
// production writers' decision to stop editing. Payloads here are wire fixtures;
// the separate real-Wasm tests certify actual effects and content-bound results.
#[hibana_runtime::main]
#[test]
async fn the_frozen_document_cannot_reopen_editing_before_or_after_publication_reply()
-> crate::shell::ShellResult<()> {
    use crate::shell::{
        ShellError,
        imp::{bus::Bus, report_arguments::ReportArguments, review_reply::ReviewReply},
    };
    use hibana::runtime::{
        SessionKitStorage,
        ids::SessionId,
        resolver::{DecisionArm, ResolverRef},
    };
    use std::time::Duration;

    let p = programs();
    let first = DecisionArm::Left;
    let chapter_a = ReportArguments::<2>::from_bytes(br#"{"section_name":"CHAPTER_1"}"#)?;
    let chapter_b = ReportArguments::<2>::from_bytes(br#"{"section_name":"CHAPTER_2"}"#)?;
    let references_a = ReportArguments::<4>::from_bytes(br#"{"section_name":"CHAPTER_1"}"#)?;
    let references_b = ReportArguments::<4>::from_bytes(br#"{"section_name":"CHAPTER_2"}"#)?;
    let review_a = ReportArguments::<5>::from_bytes(br#"{"section_name":"CHAPTER_1"}"#)?;
    let review_b = ReportArguments::<5>::from_bytes(br#"{"section_name":"CHAPTER_2"}"#)?;
    let revision = ReportArguments::<3>::from_bytes(
        br#"{"section_name":"CHAPTER_2","feedback":"actual fixture rejection"}"#,
    )?;
    let combine = ReportArguments::<6>::from_bytes(br#"{"sections":"CHAPTER_1,CHAPTER_2"}"#)?;
    let positive_bytes = br#"{"success":true}"#.as_slice();
    let negative_bytes = br#"{"success":false}"#.as_slice();
    let positive = ReviewReply::<true>::from_bytes(positive_bytes)?;
    let negative = ReviewReply::<false>::from_bytes(negative_bytes)?;
    let material = b"fixture material".as_slice();

    for failed_visits in [0, 1, 3] {
        for (after_reply, operation) in [
            (false, "resource"),
            (false, "writer"),
            (false, "return"),
            (true, "resource"),
            (true, "writer"),
            (true, "return"),
        ] {
            let session = SessionId::new(SESSION + 2);
            let (bus, _, _) = Bus::open(["WRITER_A", "WRITER_B", "EDITOR"], [(session, DOCUMENT)]);
            let mut owner_slab = vec![0; 1024 * 1024];
            let mut caller_slab = vec![0; 1024 * 1024];
            let mut owner_store = SessionKitStorage::uninit();
            let mut caller_store = SessionKitStorage::uninit();
            let owner = owner_store
                .init()
                .rendezvous(&mut owner_slab, &bus.transport)?;
            let caller = caller_store
                .init()
                .rendezvous(&mut caller_slab, &bus.transport)?;
            let mut service = owner.enter(session, &p.service)?;
            let document = owner.enter(session, &p.document)?;
            let mut a = caller.enter(session, &p.a)?;
            let mut b = caller.enter(session, &p.b)?;
            let mut editor = caller.enter(session, &p.editor)?;
            let [mut a, _a_private] = a.par()?;
            let [mut b, _b_private] = b.par()?;
            let [mut editor, _editor_private] = editor.par()?;
            owner
                .set_resolver(
                    &p.service,
                    ResolverRef::<FIRST_WRITER>::decision_state(&first, |arm| Ok(*arm)),
                )
                .map_err(|_| ShellError::message("initial document resolver"))?;

            service.send::<InitialDocument>(document).await?;
            let mut document = a.recv::<InitialDocument>().await?;
            a.send::<WriteA>(&chapter_a).await?;
            service.recv::<WriteA>().await?;
            document.send::<WriteOnDocument>(&chapter_a).await?;
            service.recv::<WriteOnDocument>().await?;
            service.send::<Written>(&material).await?;
            document.recv::<Written>().await?;
            service.send::<MaterialA>(&material).await?;
            editor.recv::<MaterialA>().await?;
            a.send::<InitialDocument>(document).await?;
            let mut document = b.recv::<InitialDocument>().await?;
            b.send::<WriteB>(&chapter_b).await?;
            service.recv::<WriteB>().await?;
            document.send::<WriteOnDocument>(&chapter_b).await?;
            service.recv::<WriteOnDocument>().await?;
            service.send::<Written>(&material).await?;
            document.recv::<Written>().await?;
            service.send::<MaterialB>(&material).await?;
            editor.recv::<MaterialB>().await?;
            b.send::<InitialDocument>(document).await?;
            let mut document = editor.recv::<InitialDocument>().await?;
            a.send::<ReferencesA>(&references_a).await?;
            service.recv::<ReferencesA>().await?;
            service.send::<ReferencedA>(&material).await?;
            a.recv::<ReferencedA>().await?;
            service.send::<ReferenceMaterialA>(&material).await?;
            editor.recv::<ReferenceMaterialA>().await?;
            b.send::<ReferencesB>(&references_b).await?;
            service.recv::<ReferencesB>().await?;
            service.send::<ReferencedB>(&material).await?;
            b.recv::<ReferencedB>().await?;
            service.send::<ReferenceMaterialB>(&material).await?;
            editor.recv::<ReferenceMaterialB>().await?;

            for visit in 0..=failed_visits {
                let outcome = if visit == failed_visits {
                    positive_bytes
                } else {
                    negative_bytes
                };
                editor.send::<ReviewA>(&review_a).await?;
                service.recv::<ReviewA>().await?;
                service.send::<ReviewedA>(&outcome).await?;
                editor.recv::<ReviewedA>().await?;
                editor.send::<ReviewB>(&review_b).await?;
                service.recv::<ReviewB>().await?;
                service.send::<ReviewedB>(&outcome).await?;
                editor.recv::<ReviewedB>().await?;
                if visit == failed_visits {
                    break;
                }
                document = if visit % 2 == 0 {
                    editor.send::<ReconsiderA>(&negative).await?;
                    service.recv::<ReconsiderA>().await?;
                    editor.send::<RejectedA>(&negative).await?;
                    a.recv::<RejectedA>().await?;
                    editor.send::<RejectedAForB>(&negative).await?;
                    b.recv::<RejectedAForB>().await?;
                    editor.send::<RevisionDocument>(document).await?;
                    let mut document = a.recv::<RevisionDocument>().await?;
                    a.send::<ReviseA>(&revision).await?;
                    service.recv::<ReviseA>().await?;
                    document.send::<ReviseOnDocument>(&revision).await?;
                    service.recv::<ReviseOnDocument>().await?;
                    service.send::<Revised>(&material).await?;
                    document.recv::<Revised>().await?;
                    service.send::<UpdatedDocument>(&material).await?;
                    editor.recv::<UpdatedDocument>().await?;
                    a.send::<RevisionDocument>(document).await?;
                    editor.recv::<RevisionDocument>().await?
                } else {
                    editor.send::<ReconsiderB>(&negative).await?;
                    service.recv::<ReconsiderB>().await?;
                    editor.send::<RejectedB>(&negative).await?;
                    b.recv::<RejectedB>().await?;
                    editor.send::<RejectedBForA>(&negative).await?;
                    a.recv::<RejectedBForA>().await?;
                    editor.send::<RevisionDocument>(document).await?;
                    let mut document = b.recv::<RevisionDocument>().await?;
                    b.send::<ReviseB>(&revision).await?;
                    service.recv::<ReviseB>().await?;
                    document.send::<ReviseOnDocument>(&revision).await?;
                    service.recv::<ReviseOnDocument>().await?;
                    service.send::<Revised>(&material).await?;
                    document.recv::<Revised>().await?;
                    service.send::<UpdatedDocument>(&material).await?;
                    editor.recv::<UpdatedDocument>().await?;
                    b.send::<RevisionDocument>(document).await?;
                    editor.recv::<RevisionDocument>().await?
                };
            }

            editor.send::<FrozenDocument>(document).await?;
            let mut document = a.recv::<FrozenDocument>().await?;
            if after_reply {
                editor.send::<FinalApprovedA>(&positive).await?;
                service.recv::<FinalApprovedA>().await?;
                editor.send::<FinalApprovedB>(&positive).await?;
                service.recv::<FinalApprovedB>().await?;
                editor.send::<Combine>(&combine).await?;
                service.recv::<Combine>().await?;
                editor.send::<CombineForward>(&combine).await?;
                a.recv::<CombineForward>().await?;
                document.send::<CombineOnDocument>(&combine).await?;
                service.recv::<CombineOnDocument>().await?;
                service.send::<Combined>(&material).await?;
                document.recv::<Combined>().await?;
            }
            bus.transport.take_observations();
            eprintln!(
                "editing probe: failed_visits={failed_visits} after_reply={after_reply} operation={operation}"
            );
            let completion = hibana_runtime::race(
                async {
                    Ok::<_, std::io::Error>(match operation {
                        "resource" => document.send::<ReviseOnDocument>(&revision).await,
                        "writer" => a.send::<ReviseA>(&revision).await,
                        "return" => a.send::<RevisionDocument>(document).await,
                        _ => unreachable!("undeclared adversarial test operation"),
                    })
                },
                hibana_runtime::host::deadline(Duration::from_secs(2)),
            )
            .await;
            // Other-actor predecessors may wait. Neither waiting nor rejection
            // may issue an edit or return the frozen authority via an old tail.
            match completion {
                Ok(Err(error)) => eprintln!("rejected: {error:?}"),
                Err(error) if error.kind() == std::io::ErrorKind::TimedOut => {
                    eprintln!("waiting for a predecessor; no accepted operation")
                }
                Ok(Ok(())) => panic!(
                    "frozen authority reopened: visits={failed_visits} after_reply={after_reply} operation={operation}"
                ),
                Err(error) => panic!("probe infrastructure failed: {error}"),
            }
            assert!(bus.transport.take_observations().is_empty());
            // An illegal protocol call may fault its run. Every attempted call
            // gets a fresh trial; do not resume one to perform another probe.
        }
    }
    Ok(())
}
