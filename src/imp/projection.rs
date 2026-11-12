//! Serialization of immutable observations from the program used by execution.
use hibana::runtime::{program::RoleProgram, tap};
use serde_json::{Value, json};

pub fn rows<const ROLE: u8>(program: &RoleProgram<ROLE>) -> Value {
    json!({"role":ROLE,"rows":tap::projected(program)
        .map(record)
        .collect::<Vec<_>>()})
}

pub fn record(row: tap::TapEvent) -> [u32; 5] {
    [
        row.ts(),
        u32::from(row.id()),
        u32::from(row.causal_key()),
        row.arg0(),
        row.arg1(),
    ]
}
