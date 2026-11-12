//! WASI shell integration with task-specific projected protocols.
pub mod global;
pub mod imp {
    pub mod bus;
    pub mod error;
    pub mod file_data;
    pub mod private_fs;
    pub mod report_arguments;
    pub mod repository_arguments;
    pub mod review_reply;
    pub mod service;
}
pub mod localside {
    pub mod private_io;
}
pub use imp::error::{ShellError, ShellResult};

// The std shell needs 17 initial pages. Allow memory.grow within a two-MiB
// caller-supplied region per guest; seven guests use fourteen MiB in total.
pub const GUEST_MEMORY_BYTES: usize = 32 * hibana_wasip1_runtime::GUEST_MEMORY_PAGE_SIZE;
