//! 13M's scientific data and two-resource QC receiver.
//! The complete direct-file choreography and Wasm qualification are in progress.
pub mod arguments;
pub mod global;
pub mod imp;
pub mod localside {
    pub mod qc;
}
#[cfg(test)]
mod tests;
