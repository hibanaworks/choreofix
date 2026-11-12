//! The actual whole-task source, independently of Endpoint fixture scheduling.
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
use hibana::runtime::program::project;
use serde_json::{Value, json};

#[test]
fn feature_delivery_whole_source_static_contract() {
    let source = feature_delivery::global::delivery::protocol();
    assert_eq!(hibana::g::diagnose(&source), None);
}

#[test]
fn feature_delivery_whole_source_projects_every_view() {
    use hibana::runtime::{program::RoleProgram, tap};
    fn inspect<const ROLE: u8>(program: RoleProgram<ROLE>) -> Value {
        let records = tap::projected(&program).collect::<Vec<_>>();
        assert_eq!(records[0].id(), tap::PROJECTED_PROGRAM);
        assert_eq!(records[0].causal_key() as u8, ROLE);
        let events = records
            .iter()
            .filter(|row| row.id() == tap::PROJECTED_SOURCE)
            .count();
        let operations = records
            .iter()
            .filter(|row| {
                matches!(
                    row.id(),
                    tap::PROJECTED_LOCAL | tap::PROJECTED_SEND | tap::PROJECTED_RECV
                )
            })
            .count();
        assert!(events > 0 && operations > 0);
        assert_eq!(events, (records[0].arg0() >> 16) as usize);
        assert_eq!(operations, records[1].ts() as usize);
        json!({"view":ROLE,"actor":records[0].causal_key() >> 8,
            "source_events":events,"local_operations":operations,"records":records.len()})
    }
    let source = feature_delivery::global::delivery::protocol();
    let views = [
        inspect(project::<0>(&source)),
        inspect(project::<1>(&source)),
        inspect(project::<2>(&source)),
        inspect(project::<3>(&source)),
        inspect(project::<4>(&source)),
        inspect(project::<5>(&source)),
        inspect(project::<6>(&source)),
        inspect(project::<7>(&source)),
        inspect(project::<8>(&source)),
        inspect(project::<9>(&source)),
        inspect(project::<10>(&source)),
        inspect(project::<11>(&source)),
        inspect(project::<12>(&source)),
        inspect(project::<13>(&source)),
        inspect(project::<14>(&source)),
        inspect(project::<15>(&source)),
    ];
    assert!(
        views
            .iter()
            .all(|view| view["source_events"] == views[0]["source_events"])
    );
    println!(
        "{}",
        json!({"delivery_projection_views":views,"actual_endpoint_calls":0})
    );
}
