//! Dining uses actual WASI file operations and both physical fork authorities.
pub mod global;
pub(crate) mod identity;
mod imp;
pub(crate) mod localside;
use crate::shell::{ShellError, ShellResult, imp::bus::Bus};
use hibana::runtime::ids::SessionId;

pub async fn run_stdio(module: &[u8], level: &str) -> ShellResult<()> {
    match level {
        "9E" => {
            use identity::roles::*;
            let (mut connection, mut reader, terminals) = Bus::open(
                ["PHILOSOPHER_A", "PHILOSOPHER_B", "PHILOSOPHER_C"],
                [
                    (SessionId::new(identity::SESSION), FORK_A_RESOURCE),
                    (SessionId::new(identity::SESSION), A_THINK_FILE),
                    (SessionId::new(identity::SESSION), A_EAT_FILE),
                    (SessionId::new(identity::SESSION), A_EAT_CLOSE),
                    (SessionId::new(identity::SESSION), FORK_B_RESOURCE),
                    (SessionId::new(identity::SESSION), B_THINK_FILE),
                    (SessionId::new(identity::SESSION), B_EAT_FILE),
                    (SessionId::new(identity::SESSION), B_EAT_CLOSE),
                    (SessionId::new(identity::SESSION), FORK_C_RESOURCE),
                    (SessionId::new(identity::SESSION), C_THINK_FILE),
                    (SessionId::new(identity::SESSION), C_EAT_FILE),
                    (SessionId::new(identity::SESSION), C_EAT_CLOSE),
                ],
            );
            let (bus, simulator) = connection.split();
            let result = hibana_runtime::race(
                localside::dining_9e::run(module, (bus, &simulator), terminals),
                reader.run(bus),
            )
            .await;
            reader.finish(bus, result, None).await
        }
        "9M" => {
            use identity::roles::*;
            let (mut connection, mut reader, terminals) = Bus::open(
                [
                    "PHILOSOPHER_A",
                    "PHILOSOPHER_B",
                    "PHILOSOPHER_C",
                    "PHILOSOPHER_D",
                    "PHILOSOPHER_E",
                ],
                [
                    (SessionId::new(identity::SESSION), FORK_A_RESOURCE),
                    (SessionId::new(identity::SESSION), A_THINK_FILE),
                    (SessionId::new(identity::SESSION), A_EAT_FILE),
                    (SessionId::new(identity::SESSION), A_EAT_CLOSE),
                    (SessionId::new(identity::SESSION), FORK_B_RESOURCE),
                    (SessionId::new(identity::SESSION), B_THINK_FILE),
                    (SessionId::new(identity::SESSION), B_EAT_FILE),
                    (SessionId::new(identity::SESSION), B_EAT_CLOSE),
                    (SessionId::new(identity::SESSION), FORK_C_RESOURCE),
                    (SessionId::new(identity::SESSION), C_THINK_FILE),
                    (SessionId::new(identity::SESSION), C_EAT_FILE),
                    (SessionId::new(identity::SESSION), C_EAT_CLOSE),
                    (SessionId::new(identity::SESSION), FORK_D_RESOURCE),
                    (SessionId::new(identity::SESSION), D_THINK_FILE),
                    (SessionId::new(identity::SESSION), D_EAT_FILE),
                    (SessionId::new(identity::SESSION), D_EAT_CLOSE),
                    (SessionId::new(identity::SESSION), FORK_E_RESOURCE),
                    (SessionId::new(identity::SESSION), E_THINK_FILE),
                    (SessionId::new(identity::SESSION), E_EAT_FILE),
                    (SessionId::new(identity::SESSION), E_EAT_CLOSE),
                ],
            );
            let (bus, simulator) = connection.split();
            let result = hibana_runtime::race(
                localside::dining_9m::run(module, (bus, &simulator), terminals),
                reader.run(bus),
            )
            .await;
            reader.finish(bus, result, None).await
        }
        "9H" => {
            use identity::roles::*;
            let (mut connection, mut reader, terminals) = Bus::open(
                [
                    "PHILOSOPHER_A",
                    "PHILOSOPHER_B",
                    "PHILOSOPHER_C",
                    "PHILOSOPHER_D",
                    "PHILOSOPHER_E",
                    "PHILOSOPHER_F",
                    "PHILOSOPHER_G",
                ],
                [
                    (SessionId::new(identity::SESSION), FORK_A_RESOURCE),
                    (SessionId::new(identity::SESSION), A_THINK_FILE),
                    (SessionId::new(identity::SESSION), A_EAT_FILE),
                    (SessionId::new(identity::SESSION), A_EAT_CLOSE),
                    (SessionId::new(identity::SESSION), FORK_B_RESOURCE),
                    (SessionId::new(identity::SESSION), B_THINK_FILE),
                    (SessionId::new(identity::SESSION), B_EAT_FILE),
                    (SessionId::new(identity::SESSION), B_EAT_CLOSE),
                    (SessionId::new(identity::SESSION), FORK_C_RESOURCE),
                    (SessionId::new(identity::SESSION), C_THINK_FILE),
                    (SessionId::new(identity::SESSION), C_EAT_FILE),
                    (SessionId::new(identity::SESSION), C_EAT_CLOSE),
                    (SessionId::new(identity::SESSION), FORK_D_RESOURCE),
                    (SessionId::new(identity::SESSION), D_THINK_FILE),
                    (SessionId::new(identity::SESSION), D_EAT_FILE),
                    (SessionId::new(identity::SESSION), D_EAT_CLOSE),
                    (SessionId::new(identity::SESSION), FORK_E_RESOURCE),
                    (SessionId::new(identity::SESSION), E_THINK_FILE),
                    (SessionId::new(identity::SESSION), E_EAT_FILE),
                    (SessionId::new(identity::SESSION), E_EAT_CLOSE),
                    (SessionId::new(identity::SESSION), FORK_F_RESOURCE),
                    (SessionId::new(identity::SESSION), F_THINK_FILE),
                    (SessionId::new(identity::SESSION), F_EAT_FILE),
                    (SessionId::new(identity::SESSION), F_EAT_CLOSE),
                    (SessionId::new(identity::SESSION), FORK_G_RESOURCE),
                    (SessionId::new(identity::SESSION), G_THINK_FILE),
                    (SessionId::new(identity::SESSION), G_EAT_FILE),
                    (SessionId::new(identity::SESSION), G_EAT_CLOSE),
                ],
            );
            let (bus, simulator) = connection.split();
            let result = hibana_runtime::race(
                localside::dining_9h::run(module, (bus, &simulator), terminals),
                reader.run(bus),
            )
            .await;
            reader.finish(bus, result, None).await
        }
        _ => Err(ShellError::message("expected dining task 9E, 9M or 9H")),
    }
}
pub fn projection(task: &str) -> ShellResult<serde_json::Value> {
    match task {
        "9E" => Ok(global::project_9e().observation()),
        "9M" => Ok(global::project_9m().observation()),
        "9H" => Ok(global::project_9h().observation()),
        _ => Err(ShellError::message("unsupported task projection")),
    }
}
