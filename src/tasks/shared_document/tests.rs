use super::{global::*, imp};
use crate::shell::global::Loan;
use serde_json::{Value, json};

#[test]
fn writer_selection_preserves_the_actual_wire_bytes() {
    use crate::shell::imp::report_arguments::ReportArguments;
    use hibana::runtime::wire::WireEncode;

    for name in ["WRITER_A", "WRITER_B"] {
        let bytes = format!(" {{ \"first_writer\" : \"{name}\" }} \n").into_bytes();
        let arguments = ReportArguments::<7>::from_bytes(&bytes).unwrap();
        let mut encoded = vec![0; bytes.len()];
        assert_eq!(arguments.encode_into(&mut encoded).unwrap(), bytes.len());
        assert_eq!(encoded, bytes);
        assert_eq!(arguments.into_value()["first_writer"], name);
    }
    for bytes in [
        br#"{"first_writer":"OUTSIDE"}"#.as_slice(),
        br#"{"first_writer":"WRITER_A","stage":"ready"}"#,
        br#"{"first_writer":1}"#,
    ] {
        assert!(ReportArguments::<7>::from_bytes(bytes).is_err());
    }
}

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
        tap::projected(&p.allocation_file).collect(),
        tap::projected(&p.allocation_data).collect(),
        tap::projected(&p.a_file).collect(),
        tap::projected(&p.a_data).collect(),
        tap::projected(&p.b_file).collect(),
        tap::projected(&p.b_data).collect(),
        tap::projected(&p.a_vm).collect(),
        tap::projected(&p.b_vm).collect(),
        tap::projected(&p.editor_vm).collect(),
    ];
    for rows in snapshots {
        let mut storage = vec![0; Protocol::workspace_len(&rows).unwrap()];
        Protocol::new(&rows, &mut storage).unwrap();
    }
}

// A deliberately dishonest wire payload tests the production receiver rather
// than relying on the SDK to reject the wrong chapter before sending it.
struct Untrusted<const SCHEMA: u32>(Vec<u8>);
impl<const SCHEMA: u32> hibana::runtime::wire::WireEncode for Untrusted<SCHEMA> {
    fn encode_into(&self, out: &mut [u8]) -> Result<usize, hibana::runtime::wire::CodecError> {
        out.get_mut(..self.0.len())
            .ok_or(hibana::runtime::wire::CodecError::Truncated)?
            .copy_from_slice(&self.0);
        Ok(self.0.len())
    }
}
impl<const SCHEMA: u32> hibana::runtime::wire::WirePayload for Untrusted<SCHEMA> {
    const SCHEMA_ID: u32 = SCHEMA;
    type Decoded<'a> = &'a [u8];
    fn validate_payload(
        _: hibana::runtime::wire::Payload,
    ) -> Result<(), hibana::runtime::wire::CodecError> {
        Ok(())
    }
    fn decode_validated_payload<'a>(
        payload: hibana::runtime::wire::ReceivedPayload<'a, Self>,
    ) -> &'a [u8] {
        payload.as_bytes()
    }
}

#[hibana_runtime::main]
#[test]
async fn the_real_document_receiver_rejects_substituted_arguments_and_wrong_initial_chapters()
-> crate::shell::ShellResult<()> {
    use super::localside;
    use crate::shell::{
        ShellError,
        imp::{bus::Bus, report_arguments::ReportArguments},
    };
    use hibana::runtime::{SessionKitStorage, ids::SessionId};
    let p = programs();
    for assigned in [CHAPTER_A, CHAPTER_B] {
        for wrong_chapter in [false, true] {
            let session = SessionId::new(SESSION + 1);
            let (bus, _, _) = Bus::open(
                [],
                [
                    (session, DOCUMENT),
                    (session, WRITER_A),
                    (session, WRITER_B),
                    (session, EDITOR),
                    (session, A_FILE),
                    (session, B_FILE),
                    (session, ALLOCATION_FILE),
                ],
            );
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
            let mut a_vm = caller.enter(session, &p.a_vm)?;
            let mut b_vm = caller.enter(session, &p.b_vm)?;
            let mut editor_vm = caller.enter(session, &p.editor_vm)?;
            service
                .send::<Loan<WRITER_A>>(owner.enter(session, &p.a)?)
                .await?;
            let mut a = a_vm.recv::<Loan<WRITER_A>>().await?;
            service
                .send::<Loan<WRITER_B>>(owner.enter(session, &p.b)?)
                .await?;
            let mut b = b_vm.recv::<Loan<WRITER_B>>().await?;
            service
                .send::<Loan<EDITOR>>(owner.enter(session, &p.editor)?)
                .await?;
            let _editor = editor_vm.recv::<Loan<EDITOR>>().await?;
            let mut a_data = owner.enter(session, &p.a_data)?;
            let mut b_data = owner.enter(session, &p.b_data)?;
            let mut data = owner.enter(session, &p.allocation_data)?;
            a_data
                .send::<Loan<A_FILE>>(owner.enter(session, &p.a_file)?)
                .await?;
            let mut a_file = a_vm.recv::<Loan<A_FILE>>().await?;
            b_data
                .send::<Loan<B_FILE>>(owner.enter(session, &p.b_file)?)
                .await?;
            let mut b_file = b_vm.recv::<Loan<B_FILE>>().await?;
            data.send::<Loan<ALLOCATION_FILE>>(owner.enter(session, &p.allocation_file)?)
                .await?;
            let mut file = editor_vm.recv::<Loan<ALLOCATION_FILE>>().await?;
            let other = if assigned == CHAPTER_A {
                CHAPTER_B
            } else {
                CHAPTER_A
            };
            let admitted = json!({"section_name":if wrong_chapter {other} else {assigned},"content_notes":"actual admitted notes"});
            let substituted = ReportArguments::<2>::from_bytes(&imp::encode(
                &json!({"section_name":assigned,"content_notes":"substituted notes"}),
            )?)?;
            let effects = EffectRequests(std::cell::Cell::new(Vec::new()));
            let result = hibana_runtime::race(async {
                let _document = localside::document::run(&mut service,document,&effects).await?;
                Ok::<(),ShellError>(())
            }, async {
                let input = ReportArguments::<7>::from_bytes(&imp::encode(&json!({"first_writer":if assigned==CHAPTER_A {"WRITER_A"} else {"WRITER_B"}}))?)?;
                file.send::<Allocate>(&input).await?;
                let mut document = if assigned==CHAPTER_A { a.recv::<InitialDocument>().await? } else { b.recv::<InitialDocumentB>().await? };
                if wrong_chapter {
                    let bytes = imp::encode(&admitted)?;
                    if assigned==CHAPTER_A {
                        a_file.send::<hibana::g::Msg<4,Untrusted<0xac03_0008>>>(&Untrusted(bytes)).await?;
                    } else {
                        b_file.send::<hibana::g::Msg<5,Untrusted<0xac03_0009>>>(&Untrusted(bytes)).await?;
                    }
                    core::future::pending::<()>().await;
                } else {
                    let bytes = imp::encode(&admitted)?;
                    if assigned==CHAPTER_A {
                        a_file.send::<WriteA>(&ReportArguments::<8>::from_bytes(&bytes)?).await?;
                        a.recv::<WriteArgumentsA>().await?;
                    } else {
                        b_file.send::<WriteB>(&ReportArguments::<9>::from_bytes(&bytes)?).await?;
                        b.recv::<WriteArgumentsB>().await?;
                    }
                    document.send::<WriteOnDocument>(&substituted).await?;
                    document.recv::<Written>().await?;
                }
                Err::<(),_>(ShellError::message("forged request unexpectedly returned"))
            }).await;
            assert!(result.is_err());
            assert!(
                effects.0.take().is_empty(),
                "rejection must precede the actual effect"
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
    use crate::shell::imp::{
        bus::Bus, report_arguments::ReportArguments, review_reply::ReviewReply,
    };
    use hibana::runtime::{SessionKitStorage, ids::SessionId};
    use std::time::Duration;

    let p = programs();
    let chapter_a = ReportArguments::<2>::from_bytes(br#"{"section_name":"CHAPTER_1"}"#)?;
    let chapter_b = ReportArguments::<2>::from_bytes(br#"{"section_name":"CHAPTER_2"}"#)?;
    let references_a = ReportArguments::<10>::from_bytes(br#"{"section_name":"CHAPTER_1"}"#)?;
    let references_b = ReportArguments::<11>::from_bytes(br#"{"section_name":"CHAPTER_2"}"#)?;
    let review_a = ReportArguments::<12>::from_bytes(br#"{"section_name":"CHAPTER_1"}"#)?;
    let review_b = ReportArguments::<13>::from_bytes(br#"{"section_name":"CHAPTER_2"}"#)?;
    let revision = ReportArguments::<3>::from_bytes(
        br#"{"section_name":"CHAPTER_2","feedback":"actual fixture rejection"}"#,
    )?;
    let combine = ReportArguments::<6>::from_bytes(br#"{"sections":"CHAPTER_1,CHAPTER_2"}"#)?;
    let initial_a = ReportArguments::<8>::from_bytes(br#"{"section_name":"CHAPTER_1"}"#)?;
    let initial_b = ReportArguments::<9>::from_bytes(br#"{"section_name":"CHAPTER_2"}"#)?;
    let revision_input = ReportArguments::<14>::from_bytes(
        br#"{"section_name":"CHAPTER_2","feedback":"actual fixture rejection"}"#,
    )?;
    let combine_input =
        ReportArguments::<16>::from_bytes(br#"{"sections":"CHAPTER_1,CHAPTER_2"}"#)?;
    let positive_bytes = br#"{"success":true}"#.as_slice();
    let negative_bytes =
        br#"{"tool_name":"review_sections","success":false,"data":{"scope":"CHAPTER_1"}}"#
            .as_slice();
    let positive = ReviewReply::<true>::from_bytes(positive_bytes)?;
    let negative = ReportArguments::<17>::from_bytes(negative_bytes)?;
    let reconsider = ReportArguments::<18>::from_bytes(br#"{"review_id":"review:retry"}"#)?;
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
            let (bus, _, _) = Bus::open(
                ["WRITER_A", "WRITER_B", "EDITOR"],
                [
                    (session, DOCUMENT),
                    (session, WRITER_A),
                    (session, WRITER_B),
                    (session, EDITOR),
                    (session, A_FILE),
                    (session, B_FILE),
                    (session, ALLOCATION_FILE),
                ],
            );
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
            let mut a_vm = caller.enter(session, &p.a_vm)?;
            let mut b_vm = caller.enter(session, &p.b_vm)?;
            let mut editor_vm = caller.enter(session, &p.editor_vm)?;
            service
                .send::<Loan<WRITER_A>>(owner.enter(session, &p.a)?)
                .await?;
            let mut a = a_vm.recv::<Loan<WRITER_A>>().await?;
            service
                .send::<Loan<WRITER_B>>(owner.enter(session, &p.b)?)
                .await?;
            let mut b = b_vm.recv::<Loan<WRITER_B>>().await?;
            service
                .send::<Loan<EDITOR>>(owner.enter(session, &p.editor)?)
                .await?;
            let mut editor = editor_vm.recv::<Loan<EDITOR>>().await?;
            let mut a_data = owner.enter(session, &p.a_data)?;
            let mut b_data = owner.enter(session, &p.b_data)?;
            let mut data = owner.enter(session, &p.allocation_data)?;
            a_data
                .send::<Loan<A_FILE>>(owner.enter(session, &p.a_file)?)
                .await?;
            let mut a_file = a_vm.recv::<Loan<A_FILE>>().await?;
            b_data
                .send::<Loan<B_FILE>>(owner.enter(session, &p.b_file)?)
                .await?;
            let mut b_file = b_vm.recv::<Loan<B_FILE>>().await?;
            data.send::<Loan<ALLOCATION_FILE>>(owner.enter(session, &p.allocation_file)?)
                .await?;
            let mut file = editor_vm.recv::<Loan<ALLOCATION_FILE>>().await?;
            let selection = ReportArguments::<7>::from_bytes(br#"{"first_writer":"WRITER_A"}"#)?;
            file.send::<Allocate>(&selection).await?;
            let selected = service.recv::<Allocate>().await?.into_value();
            let selected = imp::encode(&selected)?;
            service
                .send::<AllocationAccepted>(&selected.as_slice())
                .await?;
            assert_eq!(
                imp::decode(file.recv::<AllocationAccepted>().await?)?["first_writer"],
                "WRITER_A"
            );

            service.send::<InitialDocument>(document).await?;
            let mut document = a.recv::<InitialDocument>().await?;
            a_file.send::<WriteA>(&initial_a).await?;
            let admitted = service.recv::<WriteA>().await?;
            service.send::<WriteArgumentsA>(&admitted).await?;
            a.recv::<WriteArgumentsA>().await?;
            document.send::<WriteOnDocument>(&chapter_a).await?;
            service.recv::<WriteOnDocument>().await?;
            service.send::<Written>(&material).await?;
            document.recv::<Written>().await?;
            a.send::<WriteResultA>(&material).await?;
            a_file.recv::<WriteResultA>().await?;
            service.send::<MaterialA>(&material).await?;
            editor.recv::<MaterialA>().await?;
            a.send::<InitialDocument>(document).await?;
            let mut document = b.recv::<InitialDocument>().await?;
            b_file.send::<WriteB>(&initial_b).await?;
            let admitted = service.recv::<WriteB>().await?;
            service.send::<WriteArgumentsB>(&admitted).await?;
            b.recv::<WriteArgumentsB>().await?;
            document.send::<WriteOnDocument>(&chapter_b).await?;
            service.recv::<WriteOnDocument>().await?;
            service.send::<Written>(&material).await?;
            document.recv::<Written>().await?;
            b.send::<WriteResultB>(&material).await?;
            b_file.recv::<WriteResultB>().await?;
            service.send::<MaterialB>(&material).await?;
            editor.recv::<MaterialB>().await?;
            b.send::<InitialDocument>(document).await?;
            let mut document = editor.recv::<InitialDocument>().await?;
            a_file.send::<ReferencesA>(&references_a).await?;
            service.recv::<ReferencesA>().await?;
            service.send::<ReferencedA>(&material).await?;
            a_file.recv::<ReferencedA>().await?;
            service.send::<ReferenceMaterialA>(&material).await?;
            editor.recv::<ReferenceMaterialA>().await?;
            b_file.send::<ReferencesB>(&references_b).await?;
            service.recv::<ReferencesB>().await?;
            service.send::<ReferencedB>(&material).await?;
            b_file.recv::<ReferencedB>().await?;
            service.send::<ReferenceMaterialB>(&material).await?;
            editor.recv::<ReferenceMaterialB>().await?;

            for visit in 0..=failed_visits {
                let outcome = if visit == failed_visits {
                    positive_bytes
                } else {
                    negative_bytes
                };
                file.send::<ReviewA>(&review_a).await?;
                service.recv::<ReviewA>().await?;
                service.send::<ReviewedA>(&outcome).await?;
                file.recv::<ReviewedA>().await?;
                service.send::<ReviewEvidenceA>(&outcome).await?;
                editor.recv::<ReviewEvidenceA>().await?;
                file.send::<ReviewB>(&review_b).await?;
                service.recv::<ReviewB>().await?;
                service.send::<ReviewedB>(&outcome).await?;
                file.recv::<ReviewedB>().await?;
                service.send::<ReviewEvidenceB>(&outcome).await?;
                editor.recv::<ReviewEvidenceB>().await?;
                if visit == failed_visits {
                    break;
                }
                document = if visit % 2 == 0 {
                    file.send::<RequestReconsiderA>(&reconsider).await?;
                    editor.recv::<RequestReconsiderA>().await?;
                    editor.send::<ReconsiderA>(&negative).await?;
                    service.recv::<ReconsiderA>().await?;
                    editor.send::<RevisionReviewA>(&negative).await?;
                    a.recv::<RevisionReviewA>().await?;
                    a.send::<FeedbackA>(&negative.as_bytes()).await?;
                    a_file.recv::<FeedbackA>().await?;
                    editor.send::<RevisionReviewAForB>(&negative).await?;
                    b.recv::<RevisionReviewAForB>().await?;
                    b.send::<FeedbackB>(&negative.as_bytes()).await?;
                    b_file.recv::<FeedbackB>().await?;
                    editor.send::<RevisionDocument>(document).await?;
                    let mut document = a.recv::<RevisionDocument>().await?;
                    a_file.send::<ReviseA>(&revision_input).await?;
                    let admitted = service.recv::<ReviseA>().await?;
                    service.send::<RevisionArgumentsA>(&admitted).await?;
                    a.recv::<RevisionArgumentsA>().await?;
                    document.send::<ReviseOnDocument>(&revision).await?;
                    service.recv::<ReviseOnDocument>().await?;
                    service.send::<Revised>(&material).await?;
                    document.recv::<Revised>().await?;
                    a.send::<RevisionResultA>(&material).await?;
                    a_file.recv::<RevisionResultA>().await?;
                    service.send::<UpdatedDocument>(&material).await?;
                    editor.recv::<UpdatedDocument>().await?;
                    a.send::<RevisionDocument>(document).await?;
                    let document = editor.recv::<RevisionDocument>().await?;
                    editor.send::<RevisionAckA>(&material).await?;
                    file.recv::<RevisionAckA>().await?;
                    document
                } else {
                    file.send::<RequestReconsiderB>(&reconsider).await?;
                    editor.recv::<RequestReconsiderB>().await?;
                    editor.send::<ReconsiderB>(&negative).await?;
                    service.recv::<ReconsiderB>().await?;
                    editor.send::<RevisionReviewB>(&negative).await?;
                    b.recv::<RevisionReviewB>().await?;
                    b.send::<FeedbackB>(&negative.as_bytes()).await?;
                    b_file.recv::<FeedbackB>().await?;
                    editor.send::<RevisionReviewBForA>(&negative).await?;
                    a.recv::<RevisionReviewBForA>().await?;
                    a.send::<FeedbackA>(&negative.as_bytes()).await?;
                    a_file.recv::<FeedbackA>().await?;
                    editor.send::<RevisionDocument>(document).await?;
                    let mut document = b.recv::<RevisionDocument>().await?;
                    b_file.send::<ReviseB>(&revision_input).await?;
                    let admitted = service.recv::<ReviseB>().await?;
                    service.send::<RevisionArgumentsB>(&admitted).await?;
                    b.recv::<RevisionArgumentsB>().await?;
                    document.send::<ReviseOnDocument>(&revision).await?;
                    service.recv::<ReviseOnDocument>().await?;
                    service.send::<Revised>(&material).await?;
                    document.recv::<Revised>().await?;
                    b.send::<RevisionResultB>(&material).await?;
                    b_file.recv::<RevisionResultB>().await?;
                    service.send::<UpdatedDocument>(&material).await?;
                    editor.recv::<UpdatedDocument>().await?;
                    b.send::<RevisionDocument>(document).await?;
                    let document = editor.recv::<RevisionDocument>().await?;
                    editor.send::<RevisionAckB>(&material).await?;
                    file.recv::<RevisionAckB>().await?;
                    document
                };
            }

            file.send::<CombineInput>(&combine_input).await?;
            editor.recv::<CombineInput>().await?;
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
                        "writer" => a_file.send::<ReviseA>(&revision_input).await,
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
