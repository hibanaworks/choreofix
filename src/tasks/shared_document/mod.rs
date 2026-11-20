//! Direct file projections, document custody and actual effects for 7E.
use crate::shell::global::{FileExit, Loan};
#[cfg(test)]
mod allocation_tests;
pub mod global;
pub(crate) mod imp;
pub mod localside;
#[cfg(test)]
mod tests;

use crate::shell::{ShellError, ShellResult};
use hibana::runtime::{SessionKitStorage, ids::SessionId};

#[hibana_runtime::scope]
pub async fn run_stdio(module: &[u8]) -> ShellResult<()> {
    use crate::shell::{
        imp::{bus::Bus, private_fs::PrivateFs},
        localside::private_io,
    };
    use global::*;
    use hibana_wasip1_runtime::{ChoreoFsFile, ChoreoFsFiles, GuestMemory, HibanaWasiGuestStorage};
    let (mut connection, mut io, [a_terminal, b_terminal, editor_terminal]) = Bus::open(
        ["WRITER_A", "WRITER_B", "EDITOR"],
        [
            (SessionId::new(SESSION), WRITER_A),
            (SessionId::new(SESSION), WRITER_B),
            (SessionId::new(SESSION), EDITOR),
            (SessionId::new(SESSION), DOCUMENT),
            (SessionId::new(SESSION), A_FILE),
            (SessionId::new(SESSION), B_FILE),
            (SessionId::new(SESSION), ALLOCATION_FILE),
        ],
    );
    let (bus, simulator) = connection.split();
    let result = hibana_runtime::race(
        async {
            let p = programs();
            bus.projected(p.observation());
            let capture = |event| bus.capture(event);
            let mut slab = vec![0; 1024 * 1024];
            let incoming_capacity = slab.len();
            let mut storage = SessionKitStorage::uninit();
            let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
            rv.observe(&capture);
            let observations = rv.tap();
            let session = SessionId::new(SESSION);
            let result = async {
                let mut service = rv.enter(session, &p.service)?;
                let a = rv.enter(session, &p.a)?;
                let b = rv.enter(session, &p.b)?;
                let editor = rv.enter(session, &p.editor)?;
                let document = rv.enter(session, &p.document)?;
                let mut a_vm = rv.enter(session, &p.a_vm)?;
                let mut b_vm = rv.enter(session, &p.b_vm)?;
                let mut editor_vm = rv.enter(session, &p.editor_vm)?;
                // Initial view transfer is unconditional; it does not select an AI operation.
                let (mut a, mut b, mut editor) = async {
                    let [mut a_owner, mut b_owner, mut editor_owner] = service.par()?;
                    async {
                        a_owner.send::<Loan<WRITER_A>>(a).await?;
                        a_vm.recv::<Loan<WRITER_A>>().await
                    };
                    async {
                        b_owner.send::<Loan<WRITER_B>>(b).await?;
                        b_vm.recv::<Loan<WRITER_B>>().await
                    };
                    async {
                        editor_owner.send::<Loan<EDITOR>>(editor).await?;
                        editor_vm.recv::<Loan<EDITOR>>().await
                    };
                }
                .await?;
                let mut a_data = rv.enter(session, &p.a_data)?;
                let a_file = rv.enter(session, &p.a_file)?;
                let mut b_data = rv.enter(session, &p.b_data)?;
                let b_file = rv.enter(session, &p.b_file)?;
                let mut allocation_data = rv.enter(session, &p.allocation_data)?;
                let allocation_file = rv.enter(session, &p.allocation_file)?;
                let mut a_io = rv.enter(session, &p.a_io)?;
                let mut b_io = rv.enter(session, &p.b_io)?;
                let mut editor_io = rv.enter(session, &p.editor_io)?;
                async {
                    crate::shell::localside::file_metadata::run(&mut a_data, a_file, WRITER_FILES)
                        .await?;
                };
                async {
                    crate::shell::localside::file_metadata::run(&mut b_data, b_file, WRITER_FILES)
                        .await?;
                };
                async {
                    crate::shell::localside::file_metadata::run(
                        &mut allocation_data,
                        allocation_file,
                        EDITOR_FILES,
                    )
                    .await?;
                };
                async {
                    let _document =
                        localside::document::run(&mut service, document, &simulator).await?;
                };
                async {
                    localside::writer::run(&mut a).await?;
                };
                async {
                    localside::writer::run(&mut b).await?;
                };
                async {
                    localside::editor::run(&mut editor).await?;
                };
                async {
                    let file = a_vm.recv::<Loan<A_FILE>>().await?;
                    let (mut write, mut references, mut revision) = (
                        vec![0; crate::shell::GUEST_MEMORY_BYTES],
                        vec![0; crate::shell::GUEST_MEMORY_BYTES],
                        vec![0; crate::shell::GUEST_MEMORY_BYTES],
                    );
                    let (mut written, mut referenced, mut revised, mut feedback) = (
                        vec![0; incoming_capacity],
                        vec![0; incoming_capacity],
                        vec![0; incoming_capacity],
                        vec![0; incoming_capacity],
                    );
                    let files = ChoreoFsFiles::new(a_vm).file(
                        ChoreoFsFile::new(WRITER_FILES[0], file)
                            .write_as::<WriteA>(&mut write)
                            .read_as::<WriteResultA>(&mut written)
                            .file(WRITER_FILES[1])?
                            .write_as::<ReferencesA>(&mut references)
                            .read_as::<ReferencedA>(&mut referenced)
                            .file(WRITER_FILES[2])?
                            .write_as::<ReviseA>(&mut revision)
                            .read_as::<RevisionResultA>(&mut revised)
                            .file(WRITER_FILES[3])?
                            .read_as::<FeedbackA>(&mut feedback),
                    )?;
                    let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut storage = HibanaWasiGuestStorage::uninit();
                    let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                    let (exit, files) = guest.run().await?;
                    let status = exit.status() as i32;
                    let (mut root, (file, ())) = files.into_parts();
                    let mut file = file.into_endpoint();
                    file.send::<FileExit>(&status).await?;
                    root.send::<crate::shell::global::Exit>(&status).await?;
                };
                async {
                    let file = b_vm.recv::<Loan<B_FILE>>().await?;
                    let (mut write, mut references, mut revision) = (
                        vec![0; crate::shell::GUEST_MEMORY_BYTES],
                        vec![0; crate::shell::GUEST_MEMORY_BYTES],
                        vec![0; crate::shell::GUEST_MEMORY_BYTES],
                    );
                    let (mut written, mut referenced, mut revised, mut feedback) = (
                        vec![0; incoming_capacity],
                        vec![0; incoming_capacity],
                        vec![0; incoming_capacity],
                        vec![0; incoming_capacity],
                    );
                    let files = ChoreoFsFiles::new(b_vm).file(
                        ChoreoFsFile::new(WRITER_FILES[0], file)
                            .write_as::<WriteB>(&mut write)
                            .read_as::<WriteResultB>(&mut written)
                            .file(WRITER_FILES[1])?
                            .write_as::<ReferencesB>(&mut references)
                            .read_as::<ReferencedB>(&mut referenced)
                            .file(WRITER_FILES[2])?
                            .write_as::<ReviseB>(&mut revision)
                            .read_as::<RevisionResultB>(&mut revised)
                            .file(WRITER_FILES[3])?
                            .read_as::<FeedbackB>(&mut feedback),
                    )?;
                    let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut storage = HibanaWasiGuestStorage::uninit();
                    let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                    let (exit, files) = guest.run().await?;
                    let status = exit.status() as i32;
                    let (mut root, (file, ())) = files.into_parts();
                    let mut file = file.into_endpoint();
                    file.send::<FileExit>(&status).await?;
                    root.send::<crate::shell::global::Exit>(&status).await?;
                };
                async {
                    let file = editor_vm.recv::<Loan<ALLOCATION_FILE>>().await?;
                    let (
                        mut allocate,
                        mut review_a,
                        mut review_b,
                        mut reconsider_a,
                        mut reconsider_b,
                        mut combine,
                    ) = (
                        vec![0; crate::shell::GUEST_MEMORY_BYTES],
                        vec![0; crate::shell::GUEST_MEMORY_BYTES],
                        vec![0; crate::shell::GUEST_MEMORY_BYTES],
                        vec![0; crate::shell::GUEST_MEMORY_BYTES],
                        vec![0; crate::shell::GUEST_MEMORY_BYTES],
                        vec![0; crate::shell::GUEST_MEMORY_BYTES],
                    );
                    let (
                        mut allocated,
                        mut reviewed_a,
                        mut reviewed_b,
                        mut revised_a,
                        mut revised_b,
                        mut combined,
                    ) = (
                        vec![0; incoming_capacity],
                        vec![0; incoming_capacity],
                        vec![0; incoming_capacity],
                        vec![0; incoming_capacity],
                        vec![0; incoming_capacity],
                        vec![0; incoming_capacity],
                    );
                    let files = ChoreoFsFiles::new(editor_vm).file(
                        ChoreoFsFile::new(EDITOR_FILES[0], file)
                            .write_as::<Allocate>(&mut allocate)
                            .read_as::<AllocationAccepted>(&mut allocated)
                            .file(EDITOR_FILES[1])?
                            .write_as::<ReviewA>(&mut review_a)
                            .read_as::<ReviewedA>(&mut reviewed_a)
                            .file(EDITOR_FILES[2])?
                            .write_as::<ReviewB>(&mut review_b)
                            .read_as::<ReviewedB>(&mut reviewed_b)
                            .file(EDITOR_FILES[3])?
                            .write_as::<RequestReconsiderA>(&mut reconsider_a)
                            .read_as::<RevisionAckA>(&mut revised_a)
                            .file(EDITOR_FILES[4])?
                            .write_as::<RequestReconsiderB>(&mut reconsider_b)
                            .read_as::<RevisionAckB>(&mut revised_b)
                            .file(EDITOR_FILES[5])?
                            .write_as::<CombineInput>(&mut combine)
                            .read_as::<CombinedReply>(&mut combined),
                    )?;
                    let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut storage = HibanaWasiGuestStorage::uninit();
                    let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                    let (exit, files) = guest.run().await?;
                    let status = exit.status() as i32;
                    let (mut root, (file, ())) = files.into_parts();
                    let mut file = file.into_endpoint();
                    file.send::<FileExit>(&status).await?;
                    root.send::<crate::shell::global::Exit>(&status).await?;
                };
                async {
                    private_io::run(&mut a_io, PrivateFs::new(a_terminal, bus, &WRITER_FILES))
                        .await?;
                };
                async {
                    private_io::run(&mut b_io, PrivateFs::new(b_terminal, bus, &WRITER_FILES))
                        .await?;
                };
                async {
                    private_io::run(
                        &mut editor_io,
                        PrivateFs::new(editor_terminal, bus, &EDITOR_FILES),
                    )
                    .await?;
                };
                Ok::<(), ShellError>(())
            }
            .await;
            bus.checkpoint(SESSION, observations.checkpoint());
            result
        },
        io.run(bus),
    )
    .await;
    io.finish(bus, result, None).await
}

pub fn projection() -> serde_json::Value {
    global::programs().observation()
}
