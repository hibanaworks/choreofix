//! Alternate localsides drive the actual request/effect dependency.
use super::super::{
    global::{self, *},
    wire::{COMMIT, Close, DESIGN, IMPLEMENT},
};
use hibana::{
    g,
    runtime::{
        SessionKitStorage,
        ids::SessionId,
        wire::{CodecError, Payload, WireEncode, WirePayload},
    },
};
use hibana_wasip1_runtime::protocol;
use serde_json::json;

use crate::imp::transport;

#[hibana_runtime::main]
#[test]
async fn request_codec_rejects_fd_target_and_noncanonical_suffixes() {
    type Request = Close<EASY, 0, DESIGN>;
    let valid = b"\x04{\"feature_name\":\"user_authentication\"}";
    Request::validate_payload(Payload::new(valid)).unwrap();
    type Message = g::Msg<1, Request>;
    let global = g::send::<0, 1, Message>();
    let source = hibana::runtime::program::project::<0>(&global);
    let target = hibana::runtime::program::project::<1>(&global);
    let mut slab = vec![0; 4096];
    let transport_storage = transport::InMemoryTransport::new([]);
    let mut storage = SessionKitStorage::uninit();
    let runtime = storage
        .init()
        .rendezvous(&mut slab, &transport_storage)
        .unwrap();
    let mut source = runtime.enter(SessionId::new(85), &source).unwrap();
    let mut target = runtime.enter(SessionId::new(85), &target).unwrap();
    let request = Request::new(
        protocol::FdCloseReq(protocol::FdRequest::new(SERVICE_FD)),
        json!({"feature_name":"user_authentication"}),
    )
    .unwrap();
    source.send::<Message>(&request).await.unwrap();
    let decoded = target.recv::<Message>().await.unwrap();
    assert_eq!(decoded.as_bytes(), valid);
    let objects = Request::objects();
    let closed = objects.choreofs().fd_close(decoded);
    assert_eq!(closed.0.fd(), SERVICE_FD);
    assert_eq!(closed.0.errno(), 0);
    for bytes in [
        &b""[..],
        b"\x05{\"feature_name\":\"user_authentication\"}",
        b"\x04{\"feature_name\":\"rest_api_endpoints\"}",
        b"\x04{\"feature_name\":\"user_authentication\"}x",
        b"\x04{\"feature_name\":\"user_authentication\",\"extra\":true}",
    ] {
        assert!(Request::validate_payload(Payload::new(bytes)).is_err());
    }
}

#[test]
fn module_data_is_normalized_without_changing_the_pair_or_its_order() {
    type Request = Close<HARD, 6, COMMIT>;
    let close = protocol::FdCloseReq(protocol::FdRequest::new(SERVICE_FD));
    let request = Request::new(close, json!({"modules":" SEARCH_MODULE , AUTH_MODULE "})).unwrap();
    let mut buffer = [0u8; 128];
    let len = request.encode_into(&mut buffer).unwrap();
    Request::validate_payload(Payload::new(&buffer[..len])).unwrap();
    assert_eq!(
        &buffer[1..len],
        br#"{"modules":"SEARCH_MODULE,AUTH_MODULE"}"#
    );
    for modules in [
        "AUTH_MODULE,AUTH_MODULE",
        "AUTH_MODULE,API_MODULE",
        "AUTH_MODULE,SEARCH_MODULE,API_MODULE",
    ] {
        assert!(Request::new(close, json!({"modules":modules})).is_err());
    }
    assert!(
        Request::validate_payload(Payload::new(
            b"\x04{\"modules\":\"AUTH_MODULE,AUTH_MODULE\"}"
        ))
        .is_err()
    );
    assert!(
        Request::validate_payload(Payload::new(
            b"\x04{\"modules\":\"AUTH_MODULE,SEARCH_MODULE\"}x"
        ))
        .is_err()
    );
}

#[hibana_runtime::main]
#[test]
async fn a_design_result_requires_its_typed_request() {
    let programs = global::project_1e();
    let mut slab = vec![0; 2 * 1024 * 1024];
    let transport_storage = transport::InMemoryTransport::new([]);
    let mut storage = SessionKitStorage::uninit();
    let rv = storage
        .init()
        .rendezvous(&mut slab, &transport_storage)
        .unwrap();
    let mut owner = rv.enter(SessionId::new(87), &programs.a).unwrap();
    assert!(
        owner
            .send::<DesignCompleted>(&b"{}".as_slice())
            .await
            .is_err()
    );
}

#[hibana_runtime::main]
#[test]
async fn invalid_abi_data_then_valid_work_keeps_one_affine_effect() {
    let programs = global::project_1e();
    let mut slab = vec![0; 2 * 1024 * 1024];
    let wire = transport::InMemoryTransport::new([]);
    let mut storage = SessionKitStorage::uninit();
    let rv = storage.init().rendezvous(&mut slab, &wire).unwrap();
    let mut owner = rv.enter(SessionId::new(88), &programs.a).unwrap();
    let mut wasi = rv.enter(SessionId::new(88), &programs.a_wasi).unwrap();
    let close = protocol::FdCloseReq(protocol::FdRequest::new(SERVICE_FD));
    assert!(Close::<EASY, 0, DESIGN>::new(close, json!({"feature_name":"wrong_object"})).is_err());
    assert!(wire.take_observations().is_empty());
    let request =
        Close::<EASY, 0, DESIGN>::new(close, json!({"feature_name":"user_authentication"}))
            .unwrap();
    wasi.send::<DesignRequest<EASY, 0>>(&request).await.unwrap();
    let received = owner.recv::<DesignRequest<EASY, 0>>().await.unwrap();
    assert_eq!(
        &received.as_bytes()[1..],
        br#"{"feature_name":"user_authentication"}"#
    );
    owner
        .send::<DesignCompleted>(&b"{}".as_slice())
        .await
        .unwrap();
    assert!(
        owner
            .send::<DesignCompleted>(&b"{}".as_slice())
            .await
            .is_err(),
        "request completed twice"
    );
}

#[hibana_runtime::main]
#[test]
async fn a_valid_implementation_payload_cannot_skip_design() {
    let programs = global::project_1e();
    let mut slab = vec![0; 2 * 1024 * 1024];
    let transport_storage = transport::InMemoryTransport::new([]);
    let mut storage = SessionKitStorage::uninit();
    let rv = storage
        .init()
        .rendezvous(&mut slab, &transport_storage)
        .unwrap();
    let mut owner = rv.enter(SessionId::new(89), &programs.a).unwrap();
    let mut wasi = rv.enter(SessionId::new(89), &programs.a_wasi).unwrap();
    assert_eq!(
        Close::<EASY, 0, DESIGN>::SCHEMA_ID,
        Close::<EASY, 0, IMPLEMENT>::SCHEMA_ID
    );
    let request = Close::<EASY, 0, IMPLEMENT>::new(
        protocol::FdCloseReq(protocol::FdRequest::new(SERVICE_FD)),
        json!({"feature_name":"user_authentication"}),
    )
    .unwrap();
    assert!(
        wasi.send::<ImplementRequest<EASY, 0>>(&request)
            .await
            .is_err()
    );
    assert!(
        owner
            .send::<DesignCompleted>(&b"{}".as_slice())
            .await
            .is_err()
    );
}

#[hibana_runtime::main]
#[test]
async fn a_valid_commit_request_cannot_use_the_guest_endpoint_as_a_module() {
    let programs = global::project_1e();
    let mut slab = vec![0; 2 * 1024 * 1024];
    let transport_storage = transport::InMemoryTransport::new([]);
    let mut storage = SessionKitStorage::uninit();
    let rv = storage
        .init()
        .rendezvous(&mut slab, &transport_storage)
        .unwrap();
    let session = SessionId::new(91);
    let mut owner = rv.enter(session, &programs.a).unwrap();
    let mut wasi = rv.enter(session, &programs.a_wasi).unwrap();
    let close = protocol::FdCloseReq(protocol::FdRequest::new(SERVICE_FD));
    let request =
        Close::<EASY, 0, DESIGN>::new(close, json!({"feature_name":"user_authentication"}))
            .unwrap();
    wasi.send::<DesignRequest<EASY, 0>>(&request).await.unwrap();
    owner.recv::<DesignRequest<EASY, 0>>().await.unwrap();
    owner
        .send::<DesignCompleted>(&b"{}".as_slice())
        .await
        .unwrap();
    wasi.recv::<DesignCompleted>().await.unwrap();
    let request =
        Close::<EASY, 0, IMPLEMENT>::new(close, json!({"feature_name":"user_authentication"}))
            .unwrap();
    wasi.send::<ImplementRequest<EASY, 0>>(&request)
        .await
        .unwrap();
    owner.recv::<ImplementRequest<EASY, 0>>().await.unwrap();
    owner
        .send::<ImplementCompleted>(&b"{}".as_slice())
        .await
        .unwrap();
    wasi.recv::<ImplementCompleted>().await.unwrap();
    let request =
        Close::<EASY, 0, COMMIT>::new(close, json!({"modules":"AUTH_MODULE,DATABASE_MODULE"}))
            .unwrap();
    assert!(
        wasi.send::<CommitRequest<EASY, 0, { global::roles1e::AUTH_RESOURCE }>>(&request)
            .await
            .is_err(),
        "the guest forged a module's operation"
    );
}

// A peer encoder violates the shared wire contract. The receiver still uses
// the legitimate codec before committing its request or permitting an effect.
struct UncheckedBytes(&'static [u8]);
impl WireEncode for UncheckedBytes {
    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        let dest = out.get_mut(..self.0.len()).ok_or(CodecError::Truncated)?;
        dest.copy_from_slice(self.0);
        Ok(self.0.len())
    }
}
impl WirePayload for UncheckedBytes {
    const SCHEMA_ID: u32 = Close::<EASY, 0, DESIGN>::SCHEMA_ID;
    type Decoded<'a> = &'a [u8];
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        Close::<EASY, 0, DESIGN>::validate_payload(input)
    }
    fn decode_validated_payload<'a>(
        input: hibana::runtime::wire::ReceivedPayload<'a, Self>,
    ) -> Self::Decoded<'a> {
        input.as_bytes()
    }
}

#[hibana_runtime::main]
#[test]
async fn untrusted_request_bytes_do_not_commit_an_effect() {
    let programs = global::project_1e();
    let mut slab = vec![0; 2 * 1024 * 1024];
    let wire = transport::InMemoryTransport::new([]);
    let mut storage = SessionKitStorage::uninit();
    let rv = storage.init().rendezvous(&mut slab, &wire).unwrap();
    let mut owner = rv.enter(SessionId::new(90), &programs.a).unwrap();
    let mut wasi = rv.enter(SessionId::new(90), &programs.a_wasi).unwrap();
    wasi.send::<g::Msg<104, UncheckedBytes>>(&UncheckedBytes(
        b"\x04{\"feature_name\":\"wrong_object\"}",
    ))
    .await
    .unwrap();
    let request = owner.recv::<DesignRequest<EASY, 0>>().await;
    assert!(request.is_err(), "invalid request committed");
    let effect = owner.send::<DesignCompleted>(&b"{}".as_slice()).await;
    assert!(effect.is_err());
    eprintln!(
        "{}",
        json!({"kind":"invalid_request_effect_admission", "effect":format!("{effect:?}"),
        "frames":wire.take_observations(), "model_calls":0,"scored_benchmark":false})
    );
}
