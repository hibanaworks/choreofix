//! The operation and reply of each file are the messages of its own projection.
use super::global::*;
use crate::shell::{
    GUEST_MEMORY_BYTES, ShellResult,
    global::{Exit, FileExit},
};
use hibana::Endpoint;
use hibana_wasip1_runtime::{
    ChoreoFsFile, ChoreoFsFiles, GuestMemory, HibanaWasiGuestStorage,
    choreofs::{ChoreoFsObject, FdSpec, ObjectId},
    protocol::{FdBinding, FdReadRow, FdWriteRow},
};

const fn readable(path: &'static [u8], fd: u8) -> ChoreoFsObject<'static> {
    ChoreoFsObject::readable(
        path,
        ObjectId(fd as u32),
        FdSpec::new(fd, 2),
        b"",
        FdBinding::read(FdReadRow::Base),
    )
}

const fn operation(path: &'static [u8], fd: u8) -> ChoreoFsObject<'static> {
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

pub const AUTHOR_OBJECTS: [ChoreoFsObject<'static>; 5] = [
    operation(b"service/research_topic", 4),
    operation(b"service/write_section", 5),
    operation(b"service/update_references", 6),
    readable(b"service/review_result", 7),
    operation(b"service/revise_section", 8),
];
pub async fn author<'r, const SECTION: u8, const FILE: u8, const VM: u8>(
    module: &[u8],
    file: Endpoint<'r, FILE>,
    root: Endpoint<'r, VM>,
) -> ShellResult<()> {
    let [
        mut request_0,
        mut reply_0,
        mut request_1,
        mut reply_1,
        mut request_2,
        mut reply_2,
        mut reply_3,
        mut request_4,
        mut reply_4,
    ] = [(); 9].map(|_| vec![0; GUEST_MEMORY_BYTES]);
    let file = ChoreoFsFile::new(AUTHOR_OBJECTS[0], file)
        .write_as::<Research<SECTION>>(&mut request_0)
        .read_as::<Researched<SECTION>>(&mut reply_0)
        .file(AUTHOR_OBJECTS[1])?
        .write_as::<Write<SECTION>>(&mut request_1)
        .read_as::<Written<SECTION>>(&mut reply_1)
        .file(AUTHOR_OBJECTS[2])?
        .write_as::<References<SECTION>>(&mut request_2)
        .read_as::<Referenced<SECTION>>(&mut reply_2)
        .file(AUTHOR_OBJECTS[3])?
        .read_as::<AuthorNotice<SECTION>>(&mut reply_3)
        .file(AUTHOR_OBJECTS[4])?
        .write_as::<Revise<SECTION>>(&mut request_4)
        .read_as::<Revised<SECTION>>(&mut reply_4);
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

pub const ANALYST_OBJECTS: [ChoreoFsObject<'static>; 16] = [
    readable(b"service/section_a/research", 4),
    operation(b"service/section_a/create_figure", 5),
    readable(b"service/section_a/insertion_input", 6),
    operation(b"service/section_a/insert_figure", 7),
    readable(b"service/section_b/research", 8),
    operation(b"service/section_b/create_figure", 9),
    readable(b"service/section_b/insertion_input", 10),
    operation(b"service/section_b/insert_figure", 11),
    readable(b"service/section_c/research", 12),
    operation(b"service/section_c/create_figure", 13),
    readable(b"service/section_c/insertion_input", 14),
    operation(b"service/section_c/insert_figure", 15),
    readable(b"service/section_d/research", 16),
    operation(b"service/section_d/create_figure", 17),
    readable(b"service/section_d/insertion_input", 18),
    operation(b"service/section_d/insert_figure", 19),
];
pub async fn analyst<'r>(
    module: &[u8],
    file: Endpoint<'r, ANALYST_FILE>,
    root: Endpoint<'r, ANALYST_VM>,
) -> ShellResult<()> {
    let [
        mut reply_0,
        mut request_1,
        mut reply_1,
        mut reply_2,
        mut request_3,
        mut reply_3,
        mut reply_4,
        mut request_5,
        mut reply_5,
        mut reply_6,
        mut request_7,
        mut reply_7,
        mut reply_8,
        mut request_9,
        mut reply_9,
        mut reply_10,
        mut request_11,
        mut reply_11,
        mut reply_12,
        mut request_13,
        mut reply_13,
        mut reply_14,
        mut request_15,
        mut reply_15,
    ] = [(); 24].map(|_| vec![0; GUEST_MEMORY_BYTES]);
    let file = ChoreoFsFile::new(ANALYST_OBJECTS[0], file)
        .read_as::<ResearchForAnalyst<SECTION_A>>(&mut reply_0)
        .file(ANALYST_OBJECTS[1])?
        .write_as::<CreateFigure<SECTION_A>>(&mut request_1)
        .read_as::<FigureCreated<SECTION_A>>(&mut reply_1)
        .file(ANALYST_OBJECTS[2])?
        .read_as::<InsertionInput<SECTION_A>>(&mut reply_2)
        .file(ANALYST_OBJECTS[3])?
        .write_as::<InsertFigure<SECTION_A>>(&mut request_3)
        .read_as::<Inserted<SECTION_A>>(&mut reply_3)
        .file(ANALYST_OBJECTS[4])?
        .read_as::<ResearchForAnalyst<SECTION_B>>(&mut reply_4)
        .file(ANALYST_OBJECTS[5])?
        .write_as::<CreateFigure<SECTION_B>>(&mut request_5)
        .read_as::<FigureCreated<SECTION_B>>(&mut reply_5)
        .file(ANALYST_OBJECTS[6])?
        .read_as::<InsertionInput<SECTION_B>>(&mut reply_6)
        .file(ANALYST_OBJECTS[7])?
        .write_as::<InsertFigure<SECTION_B>>(&mut request_7)
        .read_as::<Inserted<SECTION_B>>(&mut reply_7)
        .file(ANALYST_OBJECTS[8])?
        .read_as::<ResearchForAnalyst<SECTION_C>>(&mut reply_8)
        .file(ANALYST_OBJECTS[9])?
        .write_as::<CreateFigure<SECTION_C>>(&mut request_9)
        .read_as::<FigureCreated<SECTION_C>>(&mut reply_9)
        .file(ANALYST_OBJECTS[10])?
        .read_as::<InsertionInput<SECTION_C>>(&mut reply_10)
        .file(ANALYST_OBJECTS[11])?
        .write_as::<InsertFigure<SECTION_C>>(&mut request_11)
        .read_as::<Inserted<SECTION_C>>(&mut reply_11)
        .file(ANALYST_OBJECTS[12])?
        .read_as::<ResearchForAnalyst<SECTION_D>>(&mut reply_12)
        .file(ANALYST_OBJECTS[13])?
        .write_as::<CreateFigure<SECTION_D>>(&mut request_13)
        .read_as::<FigureCreated<SECTION_D>>(&mut reply_13)
        .file(ANALYST_OBJECTS[14])?
        .read_as::<InsertionInput<SECTION_D>>(&mut reply_14)
        .file(ANALYST_OBJECTS[15])?
        .write_as::<InsertFigure<SECTION_D>>(&mut request_15)
        .read_as::<Inserted<SECTION_D>>(&mut reply_15);
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

pub const REVIEW_OBJECTS: [ChoreoFsObject<'static>; 8] = [
    readable(b"service/section_a/review_input", 4),
    operation(b"service/section_a/peer_review", 5),
    readable(b"service/section_b/review_input", 6),
    operation(b"service/section_b/peer_review", 7),
    readable(b"service/section_c/review_input", 8),
    operation(b"service/section_c/peer_review", 9),
    readable(b"service/section_d/review_input", 10),
    operation(b"service/section_d/peer_review", 11),
];
pub async fn review<'r>(
    module: &[u8],
    file: Endpoint<'r, REVIEW_FILE>,
    root: Endpoint<'r, REVIEW_VM>,
) -> ShellResult<()> {
    let [
        mut reply_0,
        mut request_1,
        mut reply_1,
        mut reply_2,
        mut request_3,
        mut reply_3,
        mut reply_4,
        mut request_5,
        mut reply_5,
        mut reply_6,
        mut request_7,
        mut reply_7,
    ] = [(); 12].map(|_| vec![0; GUEST_MEMORY_BYTES]);
    let file = ChoreoFsFile::new(REVIEW_OBJECTS[0], file)
        .read_as::<ReviewInput<SECTION_A>>(&mut reply_0)
        .file(REVIEW_OBJECTS[1])?
        .write_as::<Review<SECTION_A>>(&mut request_1)
        .read_as::<ReviewResult<SECTION_A>>(&mut reply_1)
        .file(REVIEW_OBJECTS[2])?
        .read_as::<ReviewInput<SECTION_B>>(&mut reply_2)
        .file(REVIEW_OBJECTS[3])?
        .write_as::<Review<SECTION_B>>(&mut request_3)
        .read_as::<ReviewResult<SECTION_B>>(&mut reply_3)
        .file(REVIEW_OBJECTS[4])?
        .read_as::<ReviewInput<SECTION_C>>(&mut reply_4)
        .file(REVIEW_OBJECTS[5])?
        .write_as::<Review<SECTION_C>>(&mut request_5)
        .read_as::<ReviewResult<SECTION_C>>(&mut reply_5)
        .file(REVIEW_OBJECTS[6])?
        .read_as::<ReviewInput<SECTION_D>>(&mut reply_6)
        .file(REVIEW_OBJECTS[7])?
        .write_as::<Review<SECTION_D>>(&mut request_7)
        .read_as::<ReviewResult<SECTION_D>>(&mut reply_7);
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
