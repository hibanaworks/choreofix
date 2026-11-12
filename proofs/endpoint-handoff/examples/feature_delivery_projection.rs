//! Native compiler diagnosis only. This does not qualify const projection.
//! RUSTFLAGS='--cfg hibana_profile' cargo +stable run --locked
//!   --manifest-path proofs/endpoint-handoff/Cargo.toml
//!   --features projection-profile --example feature_delivery_projection
#[path = "../../../src/imp/projection.rs"]
pub(crate) mod projection;
#[path = "../../../src/shell/mod.rs"]
pub mod shell;
#[path = "../../../src/imp/transport.rs"]
pub(crate) mod transport;
pub(crate) mod imp {
    pub(crate) use crate::{projection, transport};
}
#[path = "../../../src/tasks/feature_delivery/mod.rs"]
pub mod feature_delivery;

fn main() {
    let source = feature_delivery::global::delivery::protocol();
    let repetitions: usize = std::env::args()
        .nth(1)
        .map_or(Ok(1), |value| value.parse())
        .unwrap();
    assert!(repetitions > 0);
    let start = std::time::Instant::now();
    for _ in 0..repetitions {
        assert!(hibana::g::profiling::reentry_knowledge(
            std::hint::black_box(&source)
        ));
    }
    let seconds = start.elapsed().as_secs_f64();
    let profile = hibana::g::profiling::source(&source);
    println!(
        "{}",
        serde_json::json!({"reentry_knowledge":true,"native_seconds":seconds,
        "repetitions":repetitions,"events":profile.events,"source_rows":profile.source_rows,
        "actor_decisions":profile.actor_decisions,"receive_causal":profile.receive_causal,
        "const_projection_qualified":false})
    );
}
