//! 13M's direct-file choreography, scientific localsides and QC effect boundary.
//! Device/QC control localsides and whole-Wasm qualification are in progress.
pub mod arguments;
pub mod global;
pub mod imp;
pub mod localside {
    pub mod analyst;
    pub mod biologist;
    pub mod chemist;
    pub mod qc;
}
#[cfg(test)]
mod tests;
