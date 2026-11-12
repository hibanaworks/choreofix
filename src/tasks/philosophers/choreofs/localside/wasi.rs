//! The guest owns the actual file views for its entire interpreter execution.
use super::super::global::{EAT_OBJECT, FileExit, Loan, THINK_OBJECT};
use crate::shell::{ShellResult, global::Exit};
use hibana::Endpoint;
use hibana_wasip1_runtime::{ChoreoFsFiles, GuestMemory, HibanaWasiGuestStorage};
#[hibana_runtime::scope]
pub async fn run<'run, const ROOT: u8, const THINK: u8, const EAT: u8>(
    module: &[u8],
    mut endpoint: Endpoint<'run, ROOT>,
) -> ShellResult<i32> {
    let (think, eat) = async {
        let [mut think, mut eat] = endpoint.par()?;
        async { think.recv::<Loan<THINK>>().await };
        async { eat.recv::<Loan<EAT>>().await };
    }
    .await?;
    let files = ChoreoFsFiles::new(endpoint)
        .file(THINK_OBJECT, think)?
        .file(EAT_OBJECT, eat)?;
    let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut storage = HibanaWasiGuestStorage::uninit();
    let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
    let (exit, files) = guest.run().await?;
    let status = exit.status() as i32;
    let (mut endpoint, (eat, (think, ()))) = files.into_parts();
    let mut think = think.into_endpoint();
    let mut eat = eat.into_endpoint();
    think.send::<FileExit>(&status).await?;
    eat.send::<FileExit>(&status).await?;
    // These file-operation views are consumed with this guest. The physical
    // forks have already been returned by their actual effect receiver.
    drop(think);
    drop(eat);
    endpoint.send::<Exit>(&status).await?;
    Ok(status)
}
