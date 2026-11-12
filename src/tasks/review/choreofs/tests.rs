//! Drive the production effect receiver with independent Endpoint callers.
mod repeated_rejection;
use super::{global::*, localside};
use crate::shell::imp::repository_arguments::RepositoryArguments;
use crate::shell::imp::{bus::Bus, service::Domain};
use hibana::runtime::{SessionKitStorage, ids::SessionId, resolver::ResolverRef};
use serde_json::{Value, json};

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

struct NoEffects;
impl Domain for NoEffects {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        panic!("rejected request reached the effect: {agent} {tool} {arguments}")
    }
}

#[hibana_runtime::main]
#[test]
async fn review_control_remains_unique_while_private_io_completes()
-> Result<(), crate::shell::ShellError> {
    hibana_runtime::race(
        async {
            let p = programs();
            let session = SessionId::new(6205);
            let (bus, _, _) = Bus::open([], [(session, REVIEW), (session, REPO_ENDPOINT)]);
            let mut slab = vec![0; 1024 * 1024];
            let mut storage = SessionKitStorage::uninit();
            let rv = storage
                .init()
                .rendezvous(&mut slab, &bus.transport)
                .unwrap();
            let mut repo = rv.enter(session, &p.repo).unwrap();
            let control = rv.enter(session, &p.review).unwrap();
            let mut reviewer = rv.enter(session, &p.review_wasi).unwrap();
            let mut io = rv.enter(session, &p.review_io).unwrap();
            let [mut carrier, mut private] = reviewer.par().unwrap();
            let ((), control, ()) = async {
                async {
                    repo.send::<ReviewControl>(control).await?;
                };
                async {
                    let control = carrier.recv::<ReviewControl>().await?;
                    assert!(rv.enter(session, &p.review).is_err());
                    assert!(rv.enter(SessionId::new(6206), &p.review).is_err());
                    private.send::<crate::shell::global::Exit>(&0).await?;
                    Ok::<_, crate::shell::ShellError>(control)
                };
                async {
                    assert_eq!(io.recv::<crate::shell::global::Exit>().await?, 0);
                };
            }
            .await
            .unwrap();
            drop(control);
            assert!(rv.enter(session, &p.review).is_err());
            assert!(rv.enter(SessionId::new(6206), &p.review).is_err());
            Ok::<_, std::io::Error>(())
        },
        hibana_runtime::host::deadline(std::time::Duration::from_secs(5)),
    )
    .await
    .expect("the actual control view must stay unique while independent WASI I/O completes");
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn repository_receiver_rejects_substitution_and_wrong_target_before_effect()
-> Result<(), crate::shell::ShellError> {
    for (admitted, actual) in [
        (FEATURE_A, FEATURE_B),
        (FEATURE_B, FEATURE_B),
        (FEATURE_A, "user_authentication "),
    ] {
        hibana_runtime::race(
            async {
                {
                    async {
                        let p = programs();
                        let session = SessionId::new(6201);
                        let (bus, _reader, _terminals) = Bus::open(
                            ["DEVELOPER_A", "DEVELOPER_B", "REVIEWER"],
                            [(session, REPO_ENDPOINT), (session, REVIEW)],
                        );
                        let target = super::imp::ReviewTarget::default();
                        target.record(FEATURE_A);
                        let mut owner_slab = vec![0; 1024 * 1024];
                        let mut owner_storage = SessionKitStorage::uninit();
                        let owner = owner_storage
                            .init()
                            .rendezvous(&mut owner_slab, &bus.transport)
                            .unwrap();
                        let mut guest_slab = vec![0; 1024 * 1024];
                        let mut guest_storage = SessionKitStorage::uninit();
                        let guest = guest_storage
                            .init()
                            .rendezvous(&mut guest_slab, &bus.transport)
                            .unwrap();
                        guest
                            .set_resolver(
                                &p.review,
                                ResolverRef::<SELECT_REVIEW>::decision_state(
                                    &target,
                                    super::imp::ReviewTarget::decide,
                                ),
                            )
                            .unwrap();
                        let mut service = owner.enter(session, &p.repo).unwrap();
                        let repository = owner.enter(session, &p.repo_endpoint).unwrap();
                        let review_control = owner.enter(session, &p.review).unwrap();
                        let mut a = guest.enter(session, &p.a_wasi).unwrap();
                        let mut reviewer = guest.enter(session, &p.review_wasi).unwrap();
                        let [mut review, _review_private] = reviewer.par().unwrap();
                        let [mut task, _private] = a.par().unwrap();
                        let selected = RepositoryArguments::<3>::from_bytes(
                            &serde_json::to_vec(&json!({"pr_id":FEATURE_A})).unwrap(),
                        )
                        .unwrap();
                        let admitted = RepositoryArguments::<1>::from_bytes(
                            &serde_json::to_vec(&json!({"feature_name":admitted})).unwrap(),
                        )
                        .unwrap();
                        let substituted = RepositoryArguments::<1>::from_bytes(
                            &serde_json::to_vec(&json!({"feature_name":actual})).unwrap(),
                        )
                        .unwrap();

                        let (result, held) = async {
                            async {
                                Ok(localside::repo::run(
                                    &mut service,
                                    (repository, review_control),
                                    &NoEffects,
                                )
                                .await)
                            };
                            async {
                                let mut control = review.recv::<ReviewControl>().await.unwrap();
                                control.send::<FirstA>(&selected).await.unwrap();
                                control.send::<FirstA>(&selected).await.unwrap();
                                control.send::<FirstA>(&selected).await.unwrap();
                                control.send::<FirstA>(&selected).await.unwrap();
                                task.recv::<FirstA>().await.unwrap();
                                task.send::<ImplementA>(&admitted).await.unwrap();
                                let mut repository = task.recv::<RepoLoan>().await.unwrap();
                                assert!(owner.enter(session, &p.repo_endpoint).is_err());
                                assert!(guest.enter(session, &p.repo_endpoint).is_err());
                                repository
                                    .send::<ImplementOnRepo>(&substituted)
                                    .await
                                    .unwrap();
                                Ok(repository)
                            };
                        }
                        .await
                        .unwrap();
                        assert_eq!(
                            result.unwrap_err().to_string(),
                            "repository implementation arguments differ"
                        );
                        assert!(owner.enter(session, &p.repo_endpoint).is_err());
                        assert!(guest.enter(session, &p.repo_endpoint).is_err());
                        drop(held);
                    }
                    .await;
                    Ok::<_, std::io::Error>(())
                }
            },
            hibana_runtime::host::deadline(std::time::Duration::from_secs(5)),
        )
        .await
        .expect("repository rejection stalled");
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
        assert_eq!((agent, tool), ("DEVELOPER_A", "implement_feature"));
        assert_eq!(arguments, json!({"feature_name":FEATURE_A}));
        Ok(json!({"success":true,"agent":agent,"tool":tool,"arguments":arguments}))
    }
}

#[hibana_runtime::main]
#[test]
async fn repository_receiver_executes_the_exact_received_request_and_returns_its_result()
-> Result<(), crate::shell::ShellError> {
    hibana_runtime::race(
        async {
            {
                async {
                    let p = programs();
                    let session = SessionId::new(6202);
                    let (bus, _reader, _terminals) = Bus::open(
                        ["DEVELOPER_A", "DEVELOPER_B", "REVIEWER"],
                        [(session, REPO_ENDPOINT), (session, REVIEW)],
                    );
                    let target = super::imp::ReviewTarget::default();
                    target.record(FEATURE_A);
                    let mut owner_slab = vec![0; 1024 * 1024];
                    let mut owner_storage = SessionKitStorage::uninit();
                    let owner = owner_storage
                        .init()
                        .rendezvous(&mut owner_slab, &bus.transport)
                        .unwrap();
                    let mut guest_slab = vec![0; 1024 * 1024];
                    let mut guest_storage = SessionKitStorage::uninit();
                    let guest = guest_storage
                        .init()
                        .rendezvous(&mut guest_slab, &bus.transport)
                        .unwrap();
                    guest
                        .set_resolver(
                            &p.review,
                            ResolverRef::<SELECT_REVIEW>::decision_state(
                                &target,
                                super::imp::ReviewTarget::decide,
                            ),
                        )
                        .unwrap();
                    let mut service = owner.enter(session, &p.repo).unwrap();
                    let repository = owner.enter(session, &p.repo_endpoint).unwrap();
                        let review_control = owner.enter(session, &p.review).unwrap();
                    let mut a = guest.enter(session, &p.a_wasi).unwrap();
                    let mut reviewer = guest.enter(session, &p.review_wasi).unwrap();
                    let [mut review, _review_private] = reviewer.par().unwrap();
                    let [mut task, _private] = a.par().unwrap();
                    let selected = RepositoryArguments::<3>::from_bytes(
                        &serde_json::to_vec(&json!({"pr_id":FEATURE_A})).unwrap(),
                    )
                    .unwrap();
                    let arguments = RepositoryArguments::<1>::from_bytes(
                        &serde_json::to_vec(&json!({"feature_name":FEATURE_A})).unwrap(),
                    )
                    .unwrap();

                    hibana_runtime::race(
                        async {
                            let result =
                                localside::repo::run(&mut service, (repository, review_control), &EchoEffect).await;
                            Err::<(), crate::shell::ShellError>(crate::shell::ShellError::message(
                                format!(
                                    "repository owner stopped before the checked prefix: {result:?}"
                                ),
                            ))
                        },
                        async {
                            let result = async {
                                let mut control = review.recv::<ReviewControl>().await.unwrap();
                                control.send::<FirstA>(&selected).await.unwrap();
                                control.send::<FirstA>(&selected).await.unwrap();
                                control.send::<FirstA>(&selected).await.unwrap();
                                control.send::<FirstA>(&selected).await.unwrap();
                                task.recv::<FirstA>().await.unwrap();
                                task.send::<ImplementA>(&arguments).await?;
                                let mut repository = task.recv::<RepoLoan>().await?;
                                repository.send::<ImplementOnRepo>(&arguments).await?;
                                let response = repository.recv::<RepoResult>().await?.to_vec();
                                assert_eq!(serde_json::from_slice::<Value>(&response).unwrap(), json!({
                                    "success":true,"agent":"DEVELOPER_A","tool":"implement_feature",
                                    "arguments":{"feature_name":FEATURE_A}
                                }));
                                task.send::<RepoLoan>(repository).await?;
                                task.send::<ReadyA>(&response.as_slice()).await?;
                                Ok::<_, crate::shell::ShellError>(())
                            }.await;
                            result.unwrap();
                            Ok::<(), crate::shell::ShellError>(())
                        },
                    )
                    .await
                    .unwrap();
                }
                .await;
                Ok::<_, std::io::Error>(())
            }
        },
        hibana_runtime::host::deadline(std::time::Duration::from_secs(5)),
    )
    .await
    .expect("repository effect/result prefix stalled");

    Ok(())
}
