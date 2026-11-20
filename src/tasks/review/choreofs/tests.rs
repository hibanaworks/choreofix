//! Operation data contracts; protocol and rejection schedules use actual Wasm tests.
#[test]
fn review_reply_retains_the_response_and_rejects_the_opposite_outcome() {
    use crate::shell::imp::review_reply::ReviewReply;
    use hibana::runtime::wire::{Payload, WireEncode, WirePayload};

    let approved = br#"{"success":true,"feedback":"merge this revision"}"#;
    let rejected = br#"{"success":false,"feedback":"revise this change"}"#;
    assert!(ReviewReply::<true>::from_bytes(approved).is_ok());
    assert!(ReviewReply::<false>::from_bytes(rejected).is_ok());
    assert!(ReviewReply::<true>::from_bytes(rejected).is_err());
    assert!(ReviewReply::<false>::from_bytes(approved).is_err());
    for malformed in [b"{}".as_slice(), b"null", br#"{"success":"true"}"#] {
        assert!(ReviewReply::<true>::validate_payload(Payload::new(malformed)).is_err());
        assert!(ReviewReply::<false>::validate_payload(Payload::new(malformed)).is_err());
    }
    let response = ReviewReply::<true>::from_bytes(approved).unwrap();
    let mut encoded = [0; 128];
    let written = response.encode_into(&mut encoded).unwrap();
    assert_eq!(&encoded[..written], approved);
}

#[test]
fn file_contract_keeps_target_and_source_and_rejects_foreign_operations() {
    use super::imp::Arguments;
    use hibana::runtime::wire::{Payload, WirePayload};
    let input = br#"{"feature_name":"user_authentication","source":"actual candidate"}"#;
    assert!(Arguments::<0>::validate_payload(Payload::new(input)).is_ok());
    assert!(Arguments::<2>::validate_payload(Payload::new(input)).is_err());
    for input in [
        br#"{}"#.as_slice(),
        br#"{"feature_name":"notification_feature"}"#,
        br#"{"feature_name":"user_authentication","source":[]}"#,
        br#"{"feature_name":"user_authentication","source":" "}"#,
    ] {
        assert!(Arguments::<0>::validate_payload(Payload::new(input)).is_err());
    }
    assert!(
        Arguments::<5>::validate_payload(Payload::new(br#"{"pr_id":"user_authentication"}"#))
            .is_ok()
    );
    assert!(
        Arguments::<6>::validate_payload(Payload::new(br#"{"pr_id":"user_authentication"}"#))
            .is_err()
    );
    assert!(
        Arguments::<1>::validate_payload(Payload::new(br#"{"branch":"dev_a_feature"}"#)).is_ok()
    );
    assert!(
        Arguments::<3>::validate_payload(Payload::new(br#"{"branch":"dev_a_feature"}"#)).is_err()
    );
}

#[hibana_runtime::main]
#[test]
async fn corrected_feature_cannot_be_rejected_again_after_approval() -> crate::shell::ShellResult<()>
{
    use super::super::imp::{ReviewInput, decide};
    use super::{global::*, imp::Arguments};
    use crate::shell::imp::{bus::Bus, review_reply::ReviewReply};
    use hibana::runtime::{
        SessionKitStorage, ids::SessionId, program::project, resolver::ResolverRef,
    };
    use serde_json::json;

    let source = feature_a();
    let program = project::<REVIEW_OWNER_A>(&source);
    for first_rejected in [false, true] {
        let actual = ReviewInput::default();
        let id = SessionId::new(6);
        let (bus, _, _) = Bus::open([], [(id, REVIEW_A)]);
        let mut slab = vec![0; 65536];
        let mut storage = SessionKitStorage::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        rv.set_resolver(
            &program,
            ResolverRef::<DECIDE_A>::decision_state(&actual, decide),
        )
        .expect("register actual review outcome");
        let mut owner = rv.enter(id, &program)?;
        let mut reviewer = rv.enter(id, &project::<REVIEW_A>(&source))?;
        let mut repository = rv.enter(id, &project::<REPO>(&source))?;
        let mut author = rv.enter(id, &project::<A_FILE>(&source))?;
        let mut file = rv.enter(id, &project::<A_REVIEW_FILE>(&source))?;
        let before = br#"{"success":false}"#.as_slice();
        let after = br#"{"success":true}"#.as_slice();
        let submit = Arguments::<0>::encode_request(
            br#"{"feature_name":"user_authentication","source":"candidate"}"#,
        )
        .unwrap();
        let request =
            Arguments::<5>::encode_request(br#"{"pr_id":"user_authentication"}"#).unwrap();
        let run = async {
            for attempt in 0..=usize::from(first_rejected) {
                author.send::<FileImplementA>(&submit).await?;
                repository.recv::<FileImplementA>().await?;
                repository.send::<FileImplementationA>(&after).await?;
                author.recv::<FileImplementationA>().await?;
                repository.send::<Ready<TARGET_A>>(&after).await?;
                file.send::<FileReview<TARGET_A>>(&request).await?;
                reviewer.recv::<Ready<TARGET_A>>().await?;
                reviewer.recv::<FileReview<TARGET_A>>().await?;
                if first_rejected && attempt == 0 {
                    actual.record(json!({"success": false}));
                    owner.send::<RejectedLoanA>(reviewer).await?;
                    reviewer = repository.recv::<RejectedLoanA>().await?;
                    owner
                        .send::<RejectedA>(&ReviewReply::from_bytes(before)?)
                        .await?;
                    repository.recv::<RejectedA>().await?;
                    repository.send::<FileUpdate<A_FILE>>(&before).await?;
                    author.recv::<FileUpdate<A_FILE>>().await?;
                    reviewer.send::<FileReviewResult<TARGET_A>>(&before).await?;
                    repository.send::<RejectedLoanA>(reviewer).await?;
                    reviewer = owner.recv::<RejectedLoanA>().await?;
                } else {
                    actual.record(json!({"success": true}));
                    owner.send::<ApprovedLoanA>(reviewer).await?;
                    reviewer = repository.recv::<ApprovedLoanA>().await?;
                    owner
                        .send::<ApprovedA>(&ReviewReply::from_bytes(after)?)
                        .await?;
                    repository.recv::<ApprovedA>().await?;
                    repository.send::<FileUpdate<A_FILE>>(&after).await?;
                    author.recv::<FileUpdate<A_FILE>>().await?;
                    reviewer.send::<FileReviewResult<TARGET_A>>(&after).await?;
                }
                file.recv::<FileReviewResult<TARGET_A>>().await?;
            }
            drop(bus.transport.take_observations());
            let invalid = owner
                .send::<RejectedA>(&ReviewReply::from_bytes(before)?)
                .await;
            assert!(
                invalid.is_err(),
                "late rejection accepted after first_rejected={first_rejected}"
            );
            assert_eq!(bus.transport.take_observations().count(), 0);
            let merge = Arguments::<1>::encode_request(br#"{"branch":"dev_a_feature"}"#).unwrap();
            author.send::<FileMergeA>(&merge).await?;
            repository.recv::<FileMergeA>().await?;
            repository.send::<FileMergedA>(&after).await?;
            author.recv::<FileMergedA>().await?;
            Ok::<(), crate::shell::ShellError>(())
        };
        hibana_runtime::race(
            run,
            hibana_runtime::host::deadline(std::time::Duration::from_secs(3)),
        )
        .await?;
    }
    Ok(())
}
