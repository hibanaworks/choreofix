//! 7H's writer files bind actual chapter operations and incoming material.
use super::{
    facts::{FACT_DATA, FACT_FILE, FACT_IO, FACT_VM},
    figures::{FIGURE_DATA, FIGURE_FILE, FIGURE_IO, FIGURE_VM},
    preparation::*,
    publication::{AuthorOutcome, RECONSIDER_1, RECONSIDER_2},
    publication::{EDITOR_DATA, EDITOR_FILE, EDITOR_IO, EDITOR_VM},
    review::{REVIEW_DATA, REVIEW_FILE, REVIEW_IO, REVIEW_VM},
};
use crate::shell::{
    GUEST_MEMORY_BYTES, ShellResult,
    global::{Exit, FileExit, Loan, file_metadata, private_io},
};
use hibana::{Endpoint, g, runtime::program::Projectable};
use hibana_wasip1_runtime::{
    ChoreoFsFile, ChoreoFsFiles, GuestMemory, HibanaWasiGuestStorage,
    choreofs::{ChoreoFsObject, FdSpec, ObjectId},
    protocol::{FdBinding, FdReadRow, FdWriteRow},
};

pub const WRITER_A_DATA: u8 = 8;
pub const WRITER_B_DATA: u8 = 9;
pub const WRITER_C_DATA: u8 = 10;
pub const WRITER_A_VM: u8 = 11;
pub const WRITER_B_VM: u8 = 12;
pub const WRITER_C_VM: u8 = 13;
pub const WRITER_A_IO: u8 = 14;
pub const WRITER_B_IO: u8 = 15;
pub const WRITER_C_IO: u8 = 16;

pub(super) const fn operation(path: &'static [u8], fd: u8) -> ChoreoFsObject<'static> {
    ChoreoFsObject::read_write(
        path,
        ObjectId(fd as u32),
        FdSpec::new(fd, 66),
        b"",
        FdBinding {
            read: Some(FdReadRow::Base),
            write: Some(FdWriteRow::Object),
            readdir: None,
        },
    )
}

pub(super) const fn material(path: &'static [u8], fd: u8) -> ChoreoFsObject<'static> {
    ChoreoFsObject::readable(
        path,
        ObjectId(fd as u32),
        FdSpec::new(fd, 2),
        b"",
        FdBinding::read(FdReadRow::Base),
    )
}

pub const A_FILES: [ChoreoFsObject<'static>; 13] = [
    operation(b"service/write_section/CHAPTER_1", 4),
    operation(b"service/write_section/CHAPTER_2", 5),
    operation(b"service/update_references/CHAPTER_1", 6),
    operation(b"service/update_references/CHAPTER_2", 7),
    material(b"service/contribution", 8),
    material(b"service/evaluation", 9),
    operation(b"service/finalize_section/CHAPTER_1", 10),
    operation(b"service/finalize_section/CHAPTER_2", 11),
    material(b"service/editorial_outcome", 12),
    operation(b"service/revise_section/CHAPTER_1", 13),
    operation(b"service/revise_section/CHAPTER_2", 14),
    operation(b"service/apply_minor_edit/CHAPTER_1", 15),
    operation(b"service/apply_minor_edit/CHAPTER_2", 16),
];
pub const B_FILES: [ChoreoFsObject<'static>; 6] = [
    operation(b"service/write_section/CHAPTER_3", 4),
    operation(b"service/update_references/CHAPTER_3", 5),
    material(b"service/evaluation", 6),
    material(b"service/editorial_outcome", 7),
    operation(b"service/revise_section/CHAPTER_3", 8),
    operation(b"service/apply_minor_edit/CHAPTER_3", 9),
];
pub const C_FILES: [ChoreoFsObject<'static>; 5] = [
    operation(b"service/write_section/CHAPTER_4", 4),
    operation(b"service/update_references/CHAPTER_4", 5),
    material(b"service/editorial_outcome", 6),
    operation(b"service/revise_section/CHAPTER_4", 7),
    operation(b"service/apply_minor_edit/CHAPTER_4", 8),
];

fn guest_io<const FILE: u8, const DATA: u8, const VM: u8, const IO: u8>() -> impl Projectable {
    let closed = g::send::<FILE, DATA, FileExit>();
    let exited = g::send::<VM, IO, Exit>();
    g::par(
        g::route(closed, g::seq(file_metadata::<FILE, DATA>().roll(), closed)),
        g::route(exited, g::seq(private_io::<VM, IO>(), exited)),
    )
}

pub fn with_preparation_guests(work: impl Projectable) -> impl Projectable {
    g::seq(
        g::par(
            g::send::<WRITER_A_DATA, WRITER_A_VM, Loan<WRITER_A_FILE>>(),
            g::par(
                g::send::<WRITER_B_DATA, WRITER_B_VM, Loan<WRITER_B_FILE>>(),
                g::par(
                    g::send::<WRITER_C_DATA, WRITER_C_VM, Loan<WRITER_C_FILE>>(),
                    g::par(
                        g::send::<FIGURE_DATA, FIGURE_VM, Loan<FIGURE_FILE>>(),
                        g::par(
                            g::send::<FACT_DATA, FACT_VM, Loan<FACT_FILE>>(),
                            g::par(
                                g::send::<REVIEW_DATA, REVIEW_VM, Loan<REVIEW_FILE>>(),
                                g::send::<EDITOR_DATA, EDITOR_VM, Loan<EDITOR_FILE>>(),
                            ),
                        ),
                    ),
                ),
            ),
        ),
        g::par(
            work,
            g::par(
                guest_io::<WRITER_A_FILE, WRITER_A_DATA, WRITER_A_VM, WRITER_A_IO>(),
                g::par(
                    guest_io::<WRITER_B_FILE, WRITER_B_DATA, WRITER_B_VM, WRITER_B_IO>(),
                    g::par(
                        guest_io::<WRITER_C_FILE, WRITER_C_DATA, WRITER_C_VM, WRITER_C_IO>(),
                        g::par(
                            guest_io::<FIGURE_FILE, FIGURE_DATA, FIGURE_VM, FIGURE_IO>(),
                            g::par(
                                guest_io::<FACT_FILE, FACT_DATA, FACT_VM, FACT_IO>(),
                                g::par(
                                    guest_io::<REVIEW_FILE, REVIEW_DATA, REVIEW_VM, REVIEW_IO>(),
                                    guest_io::<EDITOR_FILE, EDITOR_DATA, EDITOR_VM, EDITOR_IO>(),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    )
}

pub async fn writer_a(module: &[u8], mut root: Endpoint<'_, WRITER_A_VM>) -> ShellResult<()> {
    let file = root.recv::<Loan<WRITER_A_FILE>>().await?;
    let [
        (mut first, mut written_first),
        (mut second, mut written_second),
        (mut refs_first, mut cited_first),
        (mut refs_second, mut cited_second),
        (mut final_first, mut finished_first),
        (mut final_second, mut finished_second),
        (mut revise_first, mut revised_first),
        (mut revise_second, mut revised_second),
    ] = [(); 8].map(|_| (vec![0; GUEST_MEMORY_BYTES], vec![0; GUEST_MEMORY_BYTES]));
    let [mut contribution, mut evaluation] = [(); 2].map(|_| vec![0; GUEST_MEMORY_BYTES]);
    let mut outcome = vec![0; GUEST_MEMORY_BYTES];
    let [
        (mut minor_first, mut amended_first),
        (mut minor_second, mut amended_second),
    ] = [(); 2].map(|_| (vec![0; GUEST_MEMORY_BYTES], vec![0; GUEST_MEMORY_BYTES]));
    let file = ChoreoFsFile::new(A_FILES[0], file)
        .write_as::<Request<WRITE_1>>(&mut first)
        .read_as::<ResultFor<WRITE_1>>(&mut written_first)
        .file(A_FILES[1])?
        .write_as::<Request<WRITE_2>>(&mut second)
        .read_as::<ResultFor<WRITE_2>>(&mut written_second)
        .file(A_FILES[2])?
        .write_as::<Request<REFERENCES_1>>(&mut refs_first)
        .read_as::<ResultFor<REFERENCES_1>>(&mut cited_first)
        .file(A_FILES[3])?
        .write_as::<Request<REFERENCES_2>>(&mut refs_second)
        .read_as::<ResultFor<REFERENCES_2>>(&mut cited_second)
        .file(A_FILES[4])?
        .read_as::<ContributionToPartI>(&mut contribution)
        .file(A_FILES[5])?
        .read_as::<EvaluationToPartI>(&mut evaluation)
        .file(A_FILES[6])?
        .write_as::<Request<FINALIZE_1>>(&mut final_first)
        .read_as::<ResultFor<FINALIZE_1>>(&mut finished_first)
        .file(A_FILES[7])?
        .write_as::<Request<FINALIZE_2>>(&mut final_second)
        .read_as::<ResultFor<FINALIZE_2>>(&mut finished_second)
        .file(A_FILES[8])?
        .read_as::<AuthorOutcome>(&mut outcome)
        .file(A_FILES[9])?
        .write_as::<Request<RECONSIDER_1>>(&mut revise_first)
        .read_as::<ResultFor<RECONSIDER_1>>(&mut revised_first)
        .file(A_FILES[10])?
        .write_as::<Request<RECONSIDER_2>>(&mut revise_second)
        .read_as::<ResultFor<RECONSIDER_2>>(&mut revised_second)
        .file(A_FILES[11])?
        .write_as::<Request<MINOR_1>>(&mut minor_first)
        .read_as::<ResultFor<MINOR_1>>(&mut amended_first)
        .file(A_FILES[12])?
        .write_as::<Request<MINOR_2>>(&mut minor_second)
        .read_as::<ResultFor<MINOR_2>>(&mut amended_second);
    let files = ChoreoFsFiles::new(root).file(file)?;
    let mut memory = vec![0; GUEST_MEMORY_BYTES];
    let mut storage = Box::new(HibanaWasiGuestStorage::uninit());
    let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
    let (exit, files) = guest.run().await?;
    let status = exit.status() as i32;
    let (mut root, (file, ())) = files.into_parts();
    root.send::<Exit>(&status).await?;
    file.into_endpoint().send::<FileExit>(&status).await?;
    Ok(())
}

pub async fn writer_b(module: &[u8], mut root: Endpoint<'_, WRITER_B_VM>) -> ShellResult<()> {
    let file = root.recv::<Loan<WRITER_B_FILE>>().await?;
    let [(mut draft, mut written), (mut refs, mut cited)] =
        [(); 2].map(|_| (vec![0; GUEST_MEMORY_BYTES], vec![0; GUEST_MEMORY_BYTES]));
    let [
        mut evaluation,
        mut outcome,
        mut revision,
        mut revised,
        mut minor,
        mut amended,
    ] = [(); 6].map(|_| vec![0; GUEST_MEMORY_BYTES]);
    let file = ChoreoFsFile::new(B_FILES[0], file)
        .write_as::<Request<WRITE_3>>(&mut draft)
        .read_as::<ResultFor<WRITE_3>>(&mut written)
        .file(B_FILES[1])?
        .write_as::<Request<REFERENCES_3>>(&mut refs)
        .read_as::<ResultFor<REFERENCES_3>>(&mut cited)
        .file(B_FILES[2])?
        .read_as::<EvaluationToChapter3>(&mut evaluation)
        .file(B_FILES[3])?
        .read_as::<AuthorOutcome>(&mut outcome)
        .file(B_FILES[4])?
        .write_as::<Request<REVISE_3>>(&mut revision)
        .read_as::<ResultFor<REVISE_3>>(&mut revised)
        .file(B_FILES[5])?
        .write_as::<Request<MINOR_3>>(&mut minor)
        .read_as::<ResultFor<MINOR_3>>(&mut amended);
    let files = ChoreoFsFiles::new(root).file(file)?;
    let mut memory = vec![0; GUEST_MEMORY_BYTES];
    let mut storage = Box::new(HibanaWasiGuestStorage::uninit());
    let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
    let (exit, files) = guest.run().await?;
    let status = exit.status() as i32;
    let (mut root, (file, ())) = files.into_parts();
    root.send::<Exit>(&status).await?;
    file.into_endpoint().send::<FileExit>(&status).await?;
    Ok(())
}

pub async fn writer_c(module: &[u8], mut root: Endpoint<'_, WRITER_C_VM>) -> ShellResult<()> {
    let file = root.recv::<Loan<WRITER_C_FILE>>().await?;
    let [(mut draft, mut written), (mut refs, mut cited)] =
        [(); 2].map(|_| (vec![0; GUEST_MEMORY_BYTES], vec![0; GUEST_MEMORY_BYTES]));
    let [
        mut outcome,
        mut revision,
        mut revised,
        mut minor,
        mut amended,
    ] = [(); 5].map(|_| vec![0; GUEST_MEMORY_BYTES]);
    let file = ChoreoFsFile::new(C_FILES[0], file)
        .write_as::<Request<WRITE_4>>(&mut draft)
        .read_as::<ResultFor<WRITE_4>>(&mut written)
        .file(C_FILES[1])?
        .write_as::<Request<REFERENCES_4>>(&mut refs)
        .read_as::<ResultFor<REFERENCES_4>>(&mut cited)
        .file(C_FILES[2])?
        .read_as::<AuthorOutcome>(&mut outcome)
        .file(C_FILES[3])?
        .write_as::<Request<REVISE_4>>(&mut revision)
        .read_as::<ResultFor<REVISE_4>>(&mut revised)
        .file(C_FILES[4])?
        .write_as::<Request<MINOR_4>>(&mut minor)
        .read_as::<ResultFor<MINOR_4>>(&mut amended);
    let files = ChoreoFsFiles::new(root).file(file)?;
    let mut memory = vec![0; GUEST_MEMORY_BYTES];
    let mut storage = Box::new(HibanaWasiGuestStorage::uninit());
    let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
    let (exit, files) = guest.run().await?;
    let status = exit.status() as i32;
    let (mut root, (file, ())) = files.into_parts();
    root.send::<Exit>(&status).await?;
    file.into_endpoint().send::<FileExit>(&status).await?;
    Ok(())
}
