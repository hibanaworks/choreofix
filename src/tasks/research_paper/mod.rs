//! 3M: independent affine chapters, two physical I/O owners and checked publication.
//! Native integration is under validation; Wasm and model qualification remain.
pub mod global;
pub(crate) mod imp;
pub mod localside;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod publication_tests;

#[cfg(test)]
mod whole_tests;
