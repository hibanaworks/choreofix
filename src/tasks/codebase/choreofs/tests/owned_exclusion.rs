//! Drive the current 1H projection without its production module localside.
use super::super::{
    global::{roles1h::*, *},
    wire::{COMMIT, Close, DESIGN, IMPLEMENT},
};
use hibana::runtime::{SessionKitStorage, ids::SessionId, transport::EndpointBinding};
use hibana_wasip1_runtime::protocol;
use std::{future::Future, task::Poll};

use crate::imp::transport;

#[hibana_runtime::main]
#[test]
async fn the_actual_1h_database_endpoint_reaches_b_only_after_a_returns_it()
-> Result<(), choreofix::shell::ShellError> {
    hibana_runtime::race(
        async {
            async {
                let programs = project_1h();
                let session = SessionId::new(9020);
                let bindings = [AUTH_RESOURCE, DATABASE_RESOURCE].map(|role| {
                    // SAFETY: these are the sole issuers for this test's two physical modules.
                    #[allow(unsafe_code)]
                    let binding = unsafe { EndpointBinding::new() };
                    (session, role, binding)
                });
                let wire = transport::InMemoryTransport::new(bindings);
                let mut slab = vec![0; 2 * 1024 * 1024];
                let mut storage = SessionKitStorage::uninit();
                let runtime = storage.init().rendezvous(&mut slab, &wire).unwrap();
                let mut a = runtime.enter(session, &programs.a).unwrap();
                let mut a_wasi = runtime.enter(session, &programs.a_wasi).unwrap();
                let mut b = runtime.enter(session, &programs.b).unwrap();
                let mut b_wasi = runtime.enter(session, &programs.b_wasi).unwrap();
                let mut auth = runtime.enter(session, &programs.auth).unwrap();
                let mut database = runtime.enter(session, &programs.database).unwrap();
                let auth_resource = runtime.enter(session, &programs.auth_resource).unwrap();
                let database_resource =
                    runtime.enter(session, &programs.database_resource).unwrap();
                let [mut a_task, _a_private] = a_wasi.par().unwrap();
                let [mut b_task, _b_private] = b_wasi.par().unwrap();
                let close = protocol::FdCloseReq(protocol::FdRequest::new(SERVICE_FD));
                let result: &[u8] = b"{}";

                let design_a = Close::<HARD, A, DESIGN>::new(
                    close,
                    serde_json::json!({"feature_name":"user_authentication"}),
                )
                .unwrap();
                a_task
                    .send::<DesignRequest<HARD, A>>(&design_a)
                    .await
                    .unwrap();
                a.recv::<DesignRequest<HARD, A>>().await.unwrap();
                a.send::<DesignCompleted>(&result).await.unwrap();
                a_task.recv::<DesignCompleted>().await.unwrap();
                let implement_a = Close::<HARD, A, IMPLEMENT>::new(
                    close,
                    serde_json::json!({"feature_name":"user_authentication"}),
                )
                .unwrap();
                a_task
                    .send::<ImplementRequest<HARD, A>>(&implement_a)
                    .await
                    .unwrap();
                a.recv::<ImplementRequest<HARD, A>>().await.unwrap();
                a.send::<ImplementCompleted>(&result).await.unwrap();
                a_task.recv::<ImplementCompleted>().await.unwrap();

                let design_b = Close::<HARD, B, DESIGN>::new(
                    close,
                    serde_json::json!({"feature_name":"rest_api_endpoints"}),
                )
                .unwrap();
                b_task
                    .send::<DesignRequest<HARD, B>>(&design_b)
                    .await
                    .unwrap();
                b.recv::<DesignRequest<HARD, B>>().await.unwrap();
                b.send::<DesignCompleted>(&result).await.unwrap();
                b_task.recv::<DesignCompleted>().await.unwrap();
                let implement_b = Close::<HARD, B, IMPLEMENT>::new(
                    close,
                    serde_json::json!({"feature_name":"rest_api_endpoints"}),
                )
                .unwrap();
                b_task
                    .send::<ImplementRequest<HARD, B>>(&implement_b)
                    .await
                    .unwrap();
                b.recv::<ImplementRequest<HARD, B>>().await.unwrap();
                b.send::<ImplementCompleted>(&result).await.unwrap();
                b_task.recv::<ImplementCompleted>().await.unwrap();

                auth.send::<Loan<AUTH_RESOURCE>>(auth_resource)
                    .await
                    .unwrap();
                database
                    .send::<Loan<DATABASE_RESOURCE>>(database_resource)
                    .await
                    .unwrap();
                let [mut a_auth, mut a_database] = a_task.par().unwrap();
                let (mut auth_resource, mut database_resource) = async {
                    async { (a_auth.recv::<Loan<AUTH_RESOURCE>>()).await };
                    async { (a_database.recv::<Loan<DATABASE_RESOURCE>>()).await };
                }
                .await
                .unwrap();
                assert!(runtime.enter(session, &programs.database_resource).is_err());

                // B has already completed its real prerequisites. No module localside or
                // owner table decides whether this receive is available.
                let [_b_api, mut b_database] = b_task.par().unwrap();
                let mut waiting = std::pin::pin!(b_database.recv::<Loan<DATABASE_RESOURCE>>());
                assert!(
                    std::future::poll_fn(|cx| Poll::Ready(waiting.as_mut().poll(cx)))
                        .await
                        .is_pending()
                );

                let commit = Close::<HARD, A, COMMIT>::new(
                    close,
                    serde_json::json!({"modules":"AUTH_MODULE,DATABASE_MODULE"}),
                )
                .unwrap();
                async {
                    async {
                        auth_resource
                            .send::<CommitRequest<HARD, A, AUTH_RESOURCE>>(&commit)
                            .await?;
                    };
                    async {
                        database_resource
                            .send::<CommitRequest<HARD, A, DATABASE_RESOURCE>>(&commit)
                            .await?;
                    };
                    Ok(())
                }
                .await
                .unwrap();
                {
                    let [mut first, mut second] = a.par().unwrap();
                    async {
                        async { (first.recv::<CommitRequest<HARD, A, AUTH_RESOURCE>>()).await };
                        async {
                            (second.recv::<CommitRequest<HARD, A, DATABASE_RESOURCE>>()).await
                        };
                    }
                    .await
                    .unwrap();
                }
                {
                    let [mut first, mut second] = a.par().unwrap();
                    async {
                        async {
                            first
                                .send::<CommitCompleted<AUTH_RESOURCE>>(&result)
                                .await?;
                        };
                        async {
                            second
                                .send::<CommitCompleted<DATABASE_RESOURCE>>(&result)
                                .await?;
                        };
                        Ok(())
                    }
                    .await
                    .unwrap();
                }
                async {
                    async { (auth_resource.recv::<CommitCompleted<AUTH_RESOURCE>>()).await };
                    async {
                        (database_resource.recv::<CommitCompleted<DATABASE_RESOURCE>>()).await
                    };
                }
                .await
                .unwrap();
                drop((a_auth, a_database));
                {
                    let [mut first, mut second] = a_task.par().unwrap();
                    async {
                        async {
                            first.send::<Loan<AUTH_RESOURCE>>(auth_resource).await?;
                        };
                        async {
                            second
                                .send::<Loan<DATABASE_RESOURCE>>(database_resource)
                                .await?;
                        };
                        Ok(())
                    }
                    .await
                    .unwrap();
                }
                let _auth_resource = auth.recv::<Loan<AUTH_RESOURCE>>().await.unwrap();
                let database_resource = database.recv::<Loan<DATABASE_RESOURCE>>().await.unwrap();
                database
                    .send::<Loan<DATABASE_RESOURCE>>(database_resource)
                    .await
                    .unwrap();
                let _b_resource = waiting.await.unwrap();
                assert!(runtime.enter(session, &programs.database_resource).is_err());
                let frames = wire.take_observations();
                let sends = |from: u8, to: u8| {
                    frames
                        .iter()
                        .filter(|frame| {
                            frame["action"] == "send"
                                && frame["source"] == from
                                && frame["target"] == to
                        })
                        .collect::<Vec<_>>()
                };
                let first_loan = sends(DATABASE, A_WASI);
                let returned = sends(A_WASI, DATABASE);
                let next_loan = sends(DATABASE, B_WASI);
                assert_eq!(
                    (first_loan.len(), returned.len(), next_loan.len()),
                    (1, 1, 1)
                );
                assert!(
                    first_loan[0]["frame_id"].as_u64().unwrap()
                        < returned[0]["frame_id"].as_u64().unwrap()
                );
                assert!(
                    returned[0]["frame_id"].as_u64().unwrap()
                        < next_loan[0]["frame_id"].as_u64().unwrap()
                );
                eprintln!(
                    "{}",
                    serde_json::json!({
                        "kind":"owned_1h_global_exclusion_prefix",
                        "task":"1H",
                        "source":"current choreofs global",
                        "production_module_localside":false,
                        "scored_benchmark":false,
                        "frames":frames
                    })
                );
            }
            .await;
            Ok::<(), std::io::Error>(())
        },
        hibana_runtime::host::deadline(std::time::Duration::from_secs(10)),
    )
    .await
    .expect("the actual 1H loan/commit/return prefix stalled");

    Ok(())
}
