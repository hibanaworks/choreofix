//! The writers have separate Wasm instances and files on their own Endpoint views.
use super::super::{CHAPTERS, global::*};
use crate::{
    imp::json::encode,
    shell::{
        GUEST_MEMORY_BYTES, ShellError, ShellResult,
        global::{Exit, FileExit},
        imp::service::ReceivedEffects,
    },
};
use hibana::Endpoint;
use hibana_wasip1_runtime::{ChoreoFsFile, ChoreoFsFiles, GuestMemory, HibanaWasiGuestStorage};
use serde_json::{Value, json};

pub async fn writer_a<'r>(
    module: &[u8],
    file: Endpoint<'r, WRITER_FILE>,
    root: Endpoint<'r, WRITER_A_VM>,
) -> ShellResult<()> {
    let [
        (mut read, mut material),
        (mut a, mut written_a),
        (mut b, mut written_b),
        (mut citation_a, mut references_a),
        (mut citation_b, mut references_b),
        (mut revised_request, mut revised),
    ] = [(); 6].map(|_| (vec![0; GUEST_MEMORY_BYTES], vec![0; GUEST_MEMORY_BYTES]));
    let mut outcome = vec![0; GUEST_MEMORY_BYTES];
    let file = ChoreoFsFile::new(WRITER_A_OBJECTS[0], file)
        .write_as::<ReadSection<WRITER_A_SNAPSHOTS>>(&mut read)
        .read_as::<SectionSnapshot>(&mut material)
        .file(WRITER_A_OBJECTS[1])?
        .write_as::<DraftRequest<DRAFT_1>>(&mut a)
        .read_as::<DraftWritten<DRAFT_1>>(&mut written_a)
        .file(WRITER_A_OBJECTS[2])?
        .write_as::<DraftRequest<DRAFT_2>>(&mut b)
        .read_as::<DraftWritten<DRAFT_2>>(&mut written_b)
        .file(WRITER_A_OBJECTS[3])?
        .write_as::<ReferenceRequest<REFERENCES_1>>(&mut citation_a)
        .read_as::<ReferenceResult<REFERENCES_1>>(&mut references_a)
        .file(WRITER_A_OBJECTS[4])?
        .write_as::<ReferenceRequest<REFERENCES_2>>(&mut citation_b)
        .read_as::<ReferenceResult<REFERENCES_2>>(&mut references_b)
        .file(WRITER_A_OBJECTS[5])?
        .read_as::<WriterOutcome<WRITER_FILE>>(&mut outcome)
        .file(WRITER_A_OBJECTS[6])?
        .write_as::<Revise<REVISE_A>>(&mut revised_request)
        .read_as::<Revised>(&mut revised);
    let files = ChoreoFsFiles::new(root).file(file)?;
    let mut memory = vec![0; GUEST_MEMORY_BYTES];
    let mut storage = Box::new(HibanaWasiGuestStorage::uninit());
    let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
    let (exit, files) = guest.run().await?;
    let status = exit.status() as i32;
    let (mut root, (file, ())) = files.into_parts();
    root.send::<Exit>(&status).await?;
    let mut file = file.into_endpoint();
    file.send::<ReadingFinished>(&status).await?;
    file.send::<FileExit>(&status).await?;
    Ok(())
}

pub async fn writer_b<'r>(
    module: &[u8],
    file: Endpoint<'r, WRITER_B_FILE>,
    root: Endpoint<'r, WRITER_B_VM>,
) -> ShellResult<()> {
    let [
        (mut read, mut material),
        (mut c, mut written),
        (mut citation, mut references),
        (mut revised_request, mut revised),
    ] = [(); 4].map(|_| (vec![0; GUEST_MEMORY_BYTES], vec![0; GUEST_MEMORY_BYTES]));
    let mut outcome = vec![0; GUEST_MEMORY_BYTES];
    let file = ChoreoFsFile::new(WRITER_B_OBJECTS[0], file)
        .write_as::<ReadSection<WRITER_B_SNAPSHOTS>>(&mut read)
        .read_as::<SectionSnapshot>(&mut material)
        .file(WRITER_B_OBJECTS[1])?
        .write_as::<DraftRequest<DRAFT_3>>(&mut c)
        .read_as::<DraftWritten<DRAFT_3>>(&mut written)
        .file(WRITER_B_OBJECTS[2])?
        .write_as::<ReferenceRequest<REFERENCES_3>>(&mut citation)
        .read_as::<ReferenceResult<REFERENCES_3>>(&mut references)
        .file(WRITER_B_OBJECTS[3])?
        .read_as::<WriterOutcome<WRITER_B_FILE>>(&mut outcome)
        .file(WRITER_B_OBJECTS[4])?
        .write_as::<Revise<REVISE_B>>(&mut revised_request)
        .read_as::<Revised>(&mut revised);
    let files = ChoreoFsFiles::new(root).file(file)?;
    let mut memory = vec![0; GUEST_MEMORY_BYTES];
    let mut storage = Box::new(HibanaWasiGuestStorage::uninit());
    let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
    let (exit, files) = guest.run().await?;
    let status = exit.status() as i32;
    let (mut root, (file, ())) = files.into_parts();
    root.send::<Exit>(&status).await?;
    let mut file = file.into_endpoint();
    file.send::<ReadingFinished>(&status).await?;
    file.send::<FileExit>(&status).await?;
    Ok(())
}

fn check_references(result: &Value, writer: &str, section: &str) -> ShellResult<()> {
    let data = &result["data"];
    if result["tool_name"] != "update_references"
        || result["success"] != true
        || data["section"] != section
        || data["scope"] != section
        || data["agent"] != writer
        || data["artifact"]["id"].as_str().is_none_or(str::is_empty)
        || !data["artifact"]["content"].is_string()
    {
        return Err(ShellError::message(
            "reference result differs from its admitted chapter",
        ));
    }
    Ok(())
}

#[hibana_runtime::scope]
pub async fn references(
    endpoint: &mut Endpoint<REFERENCES>,
    effects: &impl ReceivedEffects,
) -> ShellResult<Value> {
    let (one, two, three) = async {
        let [mut one, mut two, mut three] = endpoint.par()?;
        async {
            let received = one.recv::<ReferenceRequest<REFERENCES_1>>().await?;
            let actual = effects.apply(received).await?;
            check_references(&actual, "WRITER_A", CHAPTERS[0])?;
            one.send::<ReferenceResult<REFERENCES_1>>(&encode(&actual)?.as_slice())
                .await?;
            Ok(actual["data"]["artifact"].clone())
        };
        async {
            let received = two.recv::<ReferenceRequest<REFERENCES_2>>().await?;
            let actual = effects.apply(received).await?;
            check_references(&actual, "WRITER_A", CHAPTERS[1])?;
            two.send::<ReferenceResult<REFERENCES_2>>(&encode(&actual)?.as_slice())
                .await?;
            Ok(actual["data"]["artifact"].clone())
        };
        async {
            let received = three.recv::<ReferenceRequest<REFERENCES_3>>().await?;
            let actual = effects.apply(received).await?;
            check_references(&actual, "WRITER_B", CHAPTERS[2])?;
            three
                .send::<ReferenceResult<REFERENCES_3>>(&encode(&actual)?.as_slice())
                .await?;
            Ok(actual["data"]["artifact"].clone())
        };
    }
    .await?;
    Ok(json!({CHAPTERS[0]: one, CHAPTERS[1]: two, CHAPTERS[2]: three}))
}
