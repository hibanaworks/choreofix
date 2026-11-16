//! Production 13M data and two-resource effect boundary; no application Miri.
#[path = "../../../src/shell/imp/bus.rs"]
pub mod bus;
#[path = "../../../src/shell/imp/error.rs"]
pub mod error;
#[path = "../../../src/imp/projection.rs"]
pub(crate) mod projection;
#[path = "../../../src/shell/imp/review_reply.rs"]
pub mod review_reply;
#[path = "../../../src/shell/imp/service.rs"]
pub mod service;
#[path = "../../../src/shell/global.rs"]
pub mod shell_global;
#[path = "../../../src/imp/transport.rs"]
pub(crate) mod transport;
pub(crate) mod imp {
    pub(crate) use crate::{projection, transport};
}
pub mod shell {
    pub use crate::error::{ShellError, ShellResult};
    pub use crate::shell_global as global;
    pub mod imp {
        pub use crate::{bus, review_reply, service};
    }
}
#[path = "../../../src/tasks/lab_quality/mod.rs"]
pub mod lab_quality;
