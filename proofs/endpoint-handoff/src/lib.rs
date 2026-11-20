//! Compile the actual handoff, transport and receiver sources without unrelated tasks.
#[path = "../../../src/shell/imp/bus.rs"]
pub mod bus;
#[path = "../../../src/shell/imp/error.rs"]
pub mod error;
#[path = "../../../src/shell/imp/service.rs"]
pub mod service;
#[path = "../../../src/imp/transport.rs"]
pub(crate) mod transport;

#[path = "../../../src/imp/projection.rs"]
pub mod projection;
pub(crate) mod imp {
    pub(crate) use crate::projection;
    pub(crate) use crate::transport;
}
pub mod shell {
    pub use crate::error::{ShellError, ShellResult};
    pub mod imp {
        pub use crate::{bus, service};
    }
}
#[cfg(test)]
#[path = "../../../src/tasks/handoff/mod.rs"]
mod handoff;
