pub mod files;
pub mod global;
pub mod preparation;
pub mod publication;
pub mod quality;
pub mod runtime;

use crate::shell::{
    GUEST_MEMORY_BYTES, ShellResult,
    global::{Exit, FileExit},
};
use global::*;
use hibana::Endpoint;
use hibana_wasip1_runtime::{ChoreoFsFile, ChoreoFsFiles, GuestMemory, HibanaWasiGuestStorage};

pub async fn editor_guest<'r>(
    module: &[u8],
    file: Endpoint<'r, EDITOR_FILE>,
    root: Endpoint<'r, EDITOR_VM>,
) -> ShellResult<()> {
    let [
        mut notice,
        mut decision,
        mut result,
        mut combination,
        mut combined,
    ] = [(); 5].map(|_| vec![0; GUEST_MEMORY_BYTES]);
    let file = ChoreoFsFile::new(EDITOR_OBJECTS[0], file)
        .read_as::<EditorialInput>(&mut notice)
        .file(EDITOR_OBJECTS[1])?
        .write_as::<Editorial>(&mut decision)
        .read_as::<EditorialResult>(&mut result)
        .file(EDITOR_OBJECTS[2])?
        .write_as::<Combine>(&mut combination)
        .read_as::<Combined>(&mut combined);
    let mut memory = vec![0; GUEST_MEMORY_BYTES];
    let mut storage = HibanaWasiGuestStorage::uninit();
    let files = ChoreoFsFiles::new(root).file(file)?;
    let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
    let (exit, files) = guest.run().await?;
    let status = exit.status() as i32;
    let (mut root, (file, ())) = files.into_parts();
    root.send::<Exit>(&status).await?;
    file.into_endpoint().send::<FileExit>(&status).await?;
    Ok(())
}

#[cfg(test)]
mod quality_tests;
#[cfg(test)]
mod tests;
