//! The production effect receiver rejects a forged actual resource request.
use super::{global::*, imp::Arguments, localside};
use crate::shell::{
    ShellError, ShellResult,
    imp::{
        bus::Bus,
        service::{ReceivedEffects, SimulatorOperation},
    },
};
use hibana::{
    g,
    runtime::{
        SessionKitStorage,
        ids::SessionId,
        wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload},
    },
};
use hibana_wasip1_runtime::protocol;
use serde_json::{Value, json};
use std::{cell::Cell, time::Duration};

#[derive(Default)]
struct Journal(Cell<Vec<Value>>);
impl ReceivedEffects for Journal {
    async fn apply<P: SimulatorOperation>(
        &self,
        received: ReceivedPayload<'_, P>,
    ) -> ShellResult<Value> {
        let (agent, tool) = P::RPC;
        let arguments: Value = serde_json::from_slice(received.as_bytes()).unwrap();
        let mut rows = self.0.take();
        rows.push(json!({"agent":agent,"tool":tool,"arguments":arguments}));
        self.0.set(rows);
        Ok(json!({"success":true}))
    }
}
// An adversarial sender can claim the source schema. The receiving Endpoint,
// and not that claim, must validate the actual request before the effect.
struct Forged;
impl WireEncode for Forged {
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, CodecError> {
        let bytes = br#"{"component":"unreviewed_candidate"}"#;
        output
            .get_mut(..bytes.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(bytes);
        Ok(bytes.len())
    }
}
impl WirePayload for Forged {
    const SCHEMA_ID: u32 = <Arguments<1, true> as WirePayload>::SCHEMA_ID;
    type Decoded<'a> = &'a [u8];
    fn validate_payload(_: Payload<'_>) -> Result<(), CodecError> {
        Ok(())
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> &'a [u8] {
        input.as_bytes()
    }
}
#[hibana_runtime::main]
#[test]
async fn forged_compile_target_is_rejected_by_the_production_receiver_before_effect()
-> ShellResult<()> {
    let p = programs();
    let session = SessionId::new(1633);
    let (bus, _reader, _terminals) = Bus::open(
        [],
        [
            (session, BUILD_SERVER),
            (session, DEPLOY_ENV),
            (session, DEV_CLOSE),
            (session, TEST_CLOSE),
            (session, DEPLOY_CLOSE),
            (session, DEV_FILE),
            (session, TEST_FILE),
            (session, DEPLOY_FILE),
        ],
    );
    let mut slab = vec![0; 2 * 1024 * 1024];
    let mut storage = SessionKitStorage::uninit();
    let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
    let mut host = rv.enter(session, &p.build_owner)?;
    let mut deployment_host = rv.enter(session, &p.deploy_owner)?;
    let resource = rv.enter(session, &p.build_server)?;
    let developer_close = rv.enter(session, &p.dev_close)?;
    let tester_close = rv.enter(session, &p.test_close)?;
    let deploy_close = rv.enter(session, &p.deploy_close)?;
    let _environment = rv.enter(session, &p.deploy_env)?;
    let mut sender = rv.enter(session, &p.developer)?;
    let developer_file = rv.enter(session, &p.dev_file)?;
    let mut tester = rv.enter(session, &p.tester)?;
    let tester_file = rv.enter(session, &p.test_file)?;
    let mut deployer = rv.enter(session, &p.deployer)?;
    let deployer_file = rv.enter(session, &p.deploy_file)?;
    let mut guest = rv.enter(session, &p.dev_wasi)?;
    let mut test_guest = rv.enter(session, &p.test_wasi)?;
    let mut deploy_guest = rv.enter(session, &p.deploy_wasi)?;
    let journal = Journal::default();
    hibana_runtime::race(
        async {
            hibana_runtime::race(
                async {
                    let error = localside::infrastructure::build_server(
                        &mut host,
                        (resource, developer_close, tester_close),
                        &journal,
                    )
                    .await
                    .unwrap_err();
                    assert!(
                        matches!(&error,ShellError::Endpoint(error) if !error.is_unavailable()),
                        "{error}"
                    );
                    assert!(error.to_string().contains("Codec(Malformed)"), "{error}");
                    Ok::<(), ShellError>(())
                },
                async {
                    async {
                        let mut operation = sender.recv::<Loan<DEV_CLOSE>>().await?;
                        sender.send::<Loan<DEV_FILE>>(developer_file).await?;
                        operation.recv::<protocol::FdCloseObjectReqMsg>().await?;
                        let admitted =
                            Arguments::<1>::from_bytes(br#"{"component":"application"}"#)?;
                        operation.send::<CompileIntent>(&admitted).await?;
                        let mut build = operation.recv::<Loan<BUILD_SERVER>>().await?;
                        build.send::<g::Msg<30, Forged>>(&Forged).await?;
                        core::future::pending::<ShellResult<()>>().await
                    };
                    async {
                        let _operation = tester.recv::<Loan<TEST_CLOSE>>().await?;
                        tester.send::<Loan<TEST_FILE>>(tester_file).await?;
                        core::future::pending::<ShellResult<()>>().await
                    };
                    async {
                        let _operation = deployer.recv::<Loan<DEPLOY_CLOSE>>().await?;
                        deployer.send::<Loan<DEPLOY_FILE>>(deployer_file).await?;
                        core::future::pending::<ShellResult<()>>().await
                    };
                    async {
                        deployment_host
                            .send::<Loan<DEPLOY_CLOSE>>(deploy_close)
                            .await?;
                        core::future::pending::<ShellResult<()>>().await
                    };
                    async {
                        let mut file = guest.recv::<Loan<DEV_FILE>>().await?;
                        file.send::<protocol::FdCloseObjectReqMsg>(&protocol::FdCloseObjectReq(
                            protocol::FdRequest::new(4),
                        ))
                        .await?;
                        core::future::pending::<ShellResult<()>>().await
                    };
                    async {
                        let _file = test_guest.recv::<Loan<TEST_FILE>>().await?;
                        core::future::pending::<ShellResult<()>>().await
                    };
                    async {
                        let _file = deploy_guest.recv::<Loan<DEPLOY_FILE>>().await?;
                        core::future::pending::<ShellResult<()>>().await
                    };
                    Ok(())
                },
            )
            .await
        },
        hibana_runtime::host::deadline(Duration::from_secs(3)),
    )
    .await?;
    assert!(journal.0.take().is_empty());
    Ok(())
}
