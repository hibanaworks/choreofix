#![forbid(long_running_const_eval)]

pub mod imp;

pub use imp::domain::RunError;

pub mod shell;

pub mod tasks {
    pub mod wafer_fabrication {
        pub mod spot_checks;
    }
}
