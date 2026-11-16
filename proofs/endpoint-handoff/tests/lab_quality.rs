//! Production 13M data and two-resource effect boundary; no application Miri.
#[path = "../../../src/shell/imp/bus.rs"]
pub mod bus;
#[path = "../../../src/shell/imp/error.rs"]
pub mod error;
#[path = "../../../src/shell/imp/service.rs"]
pub mod service;
#[path = "../../../src/imp/transport.rs"]
pub(crate) mod transport;
pub(crate) mod imp {
    pub(crate) use crate::transport;
}
pub mod shell {
    pub use crate::error::{ShellError, ShellResult};
    pub mod imp {
        pub use crate::{bus, service};
    }
}
#[path = "../../../src/tasks/lab_quality/mod.rs"]
pub mod lab_quality;
