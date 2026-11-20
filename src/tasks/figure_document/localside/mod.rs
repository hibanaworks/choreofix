use super::{CHAPTERS, global::*};
use crate::{
    imp::json::{encode, same},
    shell::{
        GUEST_MEMORY_BYTES, ShellError, ShellResult,
        global::{Exit, FileExit, Loan},
        imp::service::Domain,
    },
};
use hibana::{Endpoint, g::Message};
use hibana_wasip1_runtime::{ChoreoFsFile, ChoreoFsFiles, GuestMemory, HibanaWasiGuestStorage};

#[cfg(test)]
pub mod drafts;
pub mod quality;
pub mod writers;

/// A writer reads from the same world that applies document effects.
pub async fn snapshots<const ROLE: u8>(
    endpoint: &mut Endpoint<'_, ROLE>,
    (writer, domain): (&'static str, &impl Domain),
) -> ShellResult<()> {
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            ReadSection::LOGICAL_LABEL => {
                let arguments = branch.recv::<ReadSection>().await?.value()?;
                let actual = domain
                    .execute(writer, "read_section", arguments.clone())
                    .await
                    .map_err(ShellError::message)?;
                let data = &actual["data"];
                let artifact = &data["artifact"];
                let valid = match actual["success"].as_bool() {
                    Some(true) => {
                        data["status"] == "read"
                            && artifact["id"].as_str().is_some_and(|id| !id.is_empty())
                            && artifact["content"].is_string()
                    }
                    Some(false) => data["status"] == "missing_material" && artifact.is_null(),
                    None => false,
                };
                if !valid
                    || actual["tool_name"] != "read_section"
                    || data["section"] != arguments["section_name"]
                {
                    return Err(ShellError::message(
                        "read_section did not return the requested snapshot",
                    ));
                }
                let bytes = encode(&actual)?;
                endpoint.send::<SectionSnapshot>(&bytes.as_slice()).await?;
            }
            ReadingFinished::LOGICAL_LABEL => {
                branch.recv::<ReadingFinished>().await?;
                return Ok(());
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected snapshot operation {label}"
                )));
            }
        }
    }
}

pub async fn document<'r, const OPERATIONS: usize>(
    endpoint: &mut Endpoint<'r, DRAFTING>,
    domain: &impl Domain,
) -> ShellResult<(serde_json::Value, serde_json::Value)> {
    let mut document = endpoint.recv::<Loan<DOCUMENT>>().await?;
    let mut drafts = serde_json::Map::new();
    let mut figures = serde_json::Map::new();
    for _ in 0..OPERATIONS {
        let branch = document.offer().await?;
        let operation = branch.label();
        let (arguments, agent, chapter, tool) = match operation {
            DRAFT_1 => (
                branch.recv::<DraftRequest<DRAFT_1>>().await?.value()?,
                "WRITER_A",
                CHAPTERS[0],
                "write_section",
            ),
            DRAFT_2 => (
                branch.recv::<DraftRequest<DRAFT_2>>().await?.value()?,
                "WRITER_A",
                CHAPTERS[1],
                "write_section",
            ),
            DRAFT_3 => (
                branch.recv::<DraftRequest<DRAFT_3>>().await?.value()?,
                "WRITER_B",
                CHAPTERS[2],
                "write_section",
            ),
            INSERT_1 => {
                let admitted = branch.recv::<Insert<INSERT_1>>().await?;
                document.send::<Insert<INSERT_1>>(&admitted).await?;
                let store = document.recv::<Insert<INSERT_1>>().await?;
                let arguments = admitted.value()?;
                same(&arguments, &store.value()?)?;
                (arguments, "FIGURE_CREATOR", CHAPTERS[0], "insert_figure")
            }
            INSERT_3 => {
                let admitted = branch.recv::<Insert<INSERT_3>>().await?;
                document.send::<Insert<INSERT_3>>(&admitted).await?;
                let store = document.recv::<Insert<INSERT_3>>().await?;
                let arguments = admitted.value()?;
                same(&arguments, &store.value()?)?;
                (arguments, "FIGURE_CREATOR", CHAPTERS[2], "insert_figure")
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected document operation {label}"
                )));
            }
        };
        let actual = domain
            .execute(agent, tool, arguments.clone())
            .await
            .map_err(ShellError::message)?;
        let data = &actual["data"];
        if actual["tool_name"] != tool
            || actual["success"] != true
            || data["section"] != chapter
            || data["artifact"]["id"].as_str().is_none_or(str::is_empty)
            || !data["artifact"]["content"].is_string()
            || (tool == "insert_figure" && data["figure_id"] != arguments["figure_id"])
        {
            return Err(ShellError::message(
                "document effect differs from its admitted request",
            ));
        }
        let bytes = encode(&actual)?;
        match operation {
            DRAFT_1 => {
                document
                    .send::<DraftWritten<DRAFT_1>>(&bytes.as_slice())
                    .await?
            }
            DRAFT_2 => {
                document
                    .send::<DraftWritten<DRAFT_2>>(&bytes.as_slice())
                    .await?
            }
            DRAFT_3 => {
                document
                    .send::<DraftWritten<DRAFT_3>>(&bytes.as_slice())
                    .await?
            }
            INSERT_1 => {
                document
                    .send::<Inserted<INSERT_1>>(&bytes.as_slice())
                    .await?
            }
            INSERT_3 => {
                document
                    .send::<Inserted<INSERT_3>>(&bytes.as_slice())
                    .await?
            }
            _ => unreachable!("the admitted operation determines its reply"),
        }
        if tool == "write_section" {
            drafts.insert(chapter.to_owned(), data["artifact"].clone());
        } else {
            figures.insert(chapter.to_owned(), data["artifact"].clone());
        }
    }
    endpoint.send::<Loan<DOCUMENT>>(document).await?;
    Ok((
        serde_json::Value::Object(drafts),
        serde_json::Value::Object(figures),
    ))
}

pub async fn creator<'r, const OPERATIONS: usize>(
    endpoint: &mut Endpoint<'r, FIGURE_CREATOR>,
    domain: &impl Domain,
) -> ShellResult<()> {
    let mut store = endpoint.recv::<Loan<FIGURE_STORE>>().await?;
    let mut figures = [serde_json::Value::Null, serde_json::Value::Null];
    for _ in 0..OPERATIONS {
        let branch = store.offer().await?;
        let operation = branch.label();
        let arguments = match operation {
            CREATE_1 => branch.recv::<Create<CREATE_1>>().await?.value()?,
            CREATE_3 => branch.recv::<Create<CREATE_3>>().await?.value()?,
            INSERT_1 => {
                let admitted = branch.recv::<Insert<INSERT_1>>().await?;
                same(&admitted.value()?["figure_id"], &figures[0])?;
                store.send::<Insert<INSERT_1>>(&admitted).await?;
                let actual = store.recv::<Inserted<INSERT_1>>().await?.to_vec();
                store.send::<Inserted<INSERT_1>>(&actual.as_slice()).await?;
                continue;
            }
            INSERT_3 => {
                let admitted = branch.recv::<Insert<INSERT_3>>().await?;
                same(&admitted.value()?["figure_id"], &figures[1])?;
                store.send::<Insert<INSERT_3>>(&admitted).await?;
                let actual = store.recv::<Inserted<INSERT_3>>().await?.to_vec();
                store.send::<Inserted<INSERT_3>>(&actual.as_slice()).await?;
                continue;
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected figure operation {label}"
                )));
            }
        };
        let actual = domain
            .execute("FIGURE_CREATOR", "create_figure", arguments.clone())
            .await
            .map_err(ShellError::message)?;
        if actual["tool_name"] != "create_figure"
            || actual["success"] != true
            || actual["data"]["figure_id"]
                .as_str()
                .is_none_or(str::is_empty)
            || actual["data"]["figure_id"] != actual["data"]["artifact"]["id"]
            || actual["data"]["artifact"]["content"] != arguments["description"]
        {
            return Err(ShellError::message(
                "figure store did not create the requested figure",
            ));
        }
        let bytes = encode(&actual)?;
        match operation {
            CREATE_1 => {
                figures[0] = actual["data"]["figure_id"].clone();
                store.send::<Created<CREATE_1>>(&bytes.as_slice()).await?;
            }
            CREATE_3 => {
                figures[1] = actual["data"]["figure_id"].clone();
                store.send::<Created<CREATE_3>>(&bytes.as_slice()).await?;
            }
            _ => unreachable!("the admitted creation determines its reply"),
        }
    }
    endpoint.send::<Loan<FIGURE_STORE>>(store).await?;
    Ok(())
}

pub async fn creator_guest<'r>(
    module: &[u8],
    mut root: Endpoint<'r, FIGURE_VM>,
) -> ShellResult<()> {
    let file = root.recv::<Loan<FIGURE_FILE>>().await?;
    let [
        mut create_1,
        mut created_1,
        mut create_3,
        mut created_3,
        mut insert_1,
        mut inserted_1,
        mut insert_3,
        mut inserted_3,
    ] = [(); 8].map(|_| vec![0; GUEST_MEMORY_BYTES]);
    let file = ChoreoFsFile::new(FIGURE_OBJECTS[0], file)
        .write_as::<Create<CREATE_1>>(&mut create_1)
        .read_as::<Created<CREATE_1>>(&mut created_1)
        .file(FIGURE_OBJECTS[1])?
        .write_as::<Create<CREATE_3>>(&mut create_3)
        .read_as::<Created<CREATE_3>>(&mut created_3)
        .file(FIGURE_OBJECTS[2])?
        .write_as::<Insert<INSERT_1>>(&mut insert_1)
        .read_as::<Inserted<INSERT_1>>(&mut inserted_1)
        .file(FIGURE_OBJECTS[3])?
        .write_as::<Insert<INSERT_3>>(&mut insert_3)
        .read_as::<Inserted<INSERT_3>>(&mut inserted_3);
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

pub async fn fact_checker_guest<'r>(
    module: &[u8],
    mut root: Endpoint<'r, FACT_VM>,
) -> ShellResult<()> {
    let file = root.recv::<Loan<FACT_FILE>>().await?;
    let [
        mut outcome,
        mut check_a,
        mut check_b,
        mut result_a,
        mut result_b,
    ] = [(); 5].map(|_| vec![0; GUEST_MEMORY_BYTES]);
    let file = ChoreoFsFile::new(FACT_OBJECTS[0], file)
        .read_as::<FactOutcome>(&mut outcome)
        .file(FACT_OBJECTS[1])?
        .write_as::<QualityRequest<CHECK_12>>(&mut check_a)
        .read_as::<QualityResult<CHECK_12>>(&mut result_a)
        .file(FACT_OBJECTS[2])?
        .write_as::<QualityRequest<CHECK_3>>(&mut check_b)
        .read_as::<QualityResult<CHECK_3>>(&mut result_b);
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

pub async fn editor_guest<'r>(
    module: &[u8],
    (root, file): (Endpoint<'r, EDITOR_VM>, Endpoint<'r, EDITOR_FILE>),
) -> ShellResult<()> {
    let [
        mut a,
        mut b,
        mut c,
        mut combine,
        mut ra,
        mut rb,
        mut rc,
        mut combined,
        mut material,
    ] = [(); 9].map(|_| vec![0; GUEST_MEMORY_BYTES]);
    let file = ChoreoFsFile::new(EDITOR_OBJECTS[0], file)
        .write_as::<QualityRequest<REVIEW_1>>(&mut a)
        .read_as::<QualityResult<REVIEW_1>>(&mut ra)
        .file(EDITOR_OBJECTS[1])?
        .write_as::<QualityRequest<REVIEW_2>>(&mut b)
        .read_as::<QualityResult<REVIEW_2>>(&mut rb)
        .file(EDITOR_OBJECTS[2])?
        .write_as::<QualityRequest<REVIEW_3>>(&mut c)
        .read_as::<QualityResult<REVIEW_3>>(&mut rc)
        .file(EDITOR_OBJECTS[3])?
        .write_as::<Combine>(&mut combine)
        .read_as::<Combined>(&mut combined)
        .file(EDITOR_OBJECTS[4])?
        .read_as::<ReviewMaterial>(&mut material);
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
