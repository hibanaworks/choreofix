//! Decode actual accepted file data; this grants no protocol operation.
use super::super::imp;
use crate::shell::{
    ShellError, ShellResult,
    imp::{file_wire::ClosedFile, repository_arguments::RepositoryArguments},
};

pub fn arguments<const OPERATION: u8>(
    file: &ClosedFile,
    path: &[u8],
    assigned: (&str, &str),
) -> ShellResult<RepositoryArguments<OPERATION>> {
    if file.completion.0.errno() != 0 || file.path != path {
        return Err(ShellError::message(
            "service file does not match this request",
        ));
    }
    let arguments = imp::decode(&file.bytes)?;
    imp::arguments(&arguments, assigned)?;
    RepositoryArguments::from_bytes(&file.bytes)
}
