//! A figure is inserted only after the document and the figure store accept the same request.
use super::{preparation::*, writers::operation};
use crate::{
    imp::json::{decode, encode, same},
    shell::{
        GUEST_MEMORY_BYTES, ShellError, ShellResult,
        global::{Exit, FileExit, Loan},
        imp::service::ReceivedEffects,
    },
};
use hibana::{Endpoint, g, runtime::program::Projectable};
use hibana_wasip1_runtime::{
    ChoreoFsFile, ChoreoFsFiles, GuestMemory, HibanaWasiGuestStorage, choreofs::ChoreoFsObject,
};
use serde_json::Value;

pub const FIGURE_STORE: u8 = 17;
pub const FIGURE_OWNER: u8 = 18;
pub const FIGURE_CREATOR: u8 = 19;
pub const FIGURE_FILE: u8 = 20;
pub const FIGURE_DATA: u8 = 21;
pub const FIGURE_VM: u8 = 22;
pub const FIGURE_IO: u8 = 23;

pub const CREATE_1: u8 = 17;
pub const CREATE_3: u8 = 18;
pub const CREATE_4: u8 = 19;
pub const INSERT_1: u8 = 20;
pub const INSERT_3: u8 = 21;
pub const INSERT_4: u8 = 22;
pub const FIGURE_CHAPTERS: [&str; 3] = ["CHAPTER_1", "CHAPTER_3", "CHAPTER_4"];

pub const FIGURE_FILES: [ChoreoFsObject<'static>; 6] = [
    operation(b"service/create_figure/CHAPTER_1", 4),
    operation(b"service/create_figure/CHAPTER_3", 5),
    operation(b"service/create_figure/CHAPTER_4", 6),
    operation(b"service/insert_figure/CHAPTER_1", 7),
    operation(b"service/insert_figure/CHAPTER_3", 8),
    operation(b"service/insert_figure/CHAPTER_4", 9),
];

pub fn create_figure<const CREATE: u8>() -> impl Projectable {
    g::seq(
        g::send::<FIGURE_FILE, FIGURE_STORE, Request<CREATE>>(),
        g::send::<FIGURE_STORE, FIGURE_FILE, ResultFor<CREATE>>(),
    )
}

pub fn insert_figure<const INSERT: u8>() -> impl Projectable {
    g::seq(
        g::send::<FIGURE_FILE, DOCUMENT, FigureLookup<INSERT>>(),
        g::seq(
            g::send::<DOCUMENT, FIGURE_STORE, FigureLookup<INSERT>>(),
            g::seq(
                g::send::<FIGURE_STORE, DOCUMENT, Request<INSERT>>(),
                g::seq(
                    g::send::<DOCUMENT, FIGURE_STORE, ResultFor<INSERT>>(),
                    g::send::<FIGURE_STORE, FIGURE_FILE, ResultFor<INSERT>>(),
                ),
            ),
        ),
    )
}

/// Owns the three actual created figure artifacts until their insertion.
pub async fn figure_store<'run>(
    creator: &mut Endpoint<'run, FIGURE_CREATOR>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    let mut store = creator.recv::<Loan<FIGURE_STORE>>().await?;
    let mut created = [const { Value::Null }; 3];
    for _ in 0..2 * FIGURE_CHAPTERS.len() {
        let branch = store.offer().await?;
        let label = branch.label();
        let (description, result) = match label {
            CREATE_1 => {
                let request = branch.recv::<Request<CREATE_1>>().await?;
                let description = decode(request.as_bytes())?["description"].take();
                (description, effects.apply(request).await?)
            }
            CREATE_3 => {
                let request = branch.recv::<Request<CREATE_3>>().await?;
                let description = decode(request.as_bytes())?["description"].take();
                (description, effects.apply(request).await?)
            }
            CREATE_4 => {
                let request = branch.recv::<Request<CREATE_4>>().await?;
                let description = decode(request.as_bytes())?["description"].take();
                (description, effects.apply(request).await?)
            }
            INSERT_1 => {
                let args = branch.recv::<FigureLookup<INSERT_1>>().await?;
                same(&args.0["figure_id"], &created[0]["id"])?;
                store
                    .send::<Request<INSERT_1>>(&ChapterArguments(args.0))
                    .await?;
                let reply = store.recv::<ResultFor<INSERT_1>>().await?.to_vec();
                same(&decode(&reply)?["data"]["artifact"], &created[0])?;
                store.send::<ResultFor<INSERT_1>>(&reply.as_slice()).await?;
                continue;
            }
            INSERT_3 => {
                let args = branch.recv::<FigureLookup<INSERT_3>>().await?;
                same(&args.0["figure_id"], &created[1]["id"])?;
                store
                    .send::<Request<INSERT_3>>(&ChapterArguments(args.0))
                    .await?;
                let reply = store.recv::<ResultFor<INSERT_3>>().await?.to_vec();
                same(&decode(&reply)?["data"]["artifact"], &created[1])?;
                store.send::<ResultFor<INSERT_3>>(&reply.as_slice()).await?;
                continue;
            }
            INSERT_4 => {
                let args = branch.recv::<FigureLookup<INSERT_4>>().await?;
                same(&args.0["figure_id"], &created[2]["id"])?;
                store
                    .send::<Request<INSERT_4>>(&ChapterArguments(args.0))
                    .await?;
                let reply = store.recv::<ResultFor<INSERT_4>>().await?.to_vec();
                same(&decode(&reply)?["data"]["artifact"], &created[2])?;
                store.send::<ResultFor<INSERT_4>>(&reply.as_slice()).await?;
                continue;
            }
            _ => {
                return Err(ShellError::message(format!(
                    "unexpected figure operation {label}"
                )));
            }
        };
        let data = &result["data"];
        if result["success"] != true
            || result["tool_name"] != "create_figure"
            || data["agent"] != "FIGURE_CREATOR"
            || data["figure_id"].as_str().is_none_or(str::is_empty)
            || data["artifact"]["id"] != data["figure_id"]
            || data["artifact"]["content"] != description
        {
            return Err(ShellError::message(
                "creation does not match the admitted figure",
            ));
        }
        created[usize::from(label - CREATE_1)] = data["artifact"].clone();
        let bytes = encode(&result)?;
        match label {
            CREATE_1 => store.send::<ResultFor<CREATE_1>>(&bytes.as_slice()).await?,
            CREATE_3 => store.send::<ResultFor<CREATE_3>>(&bytes.as_slice()).await?,
            CREATE_4 => store.send::<ResultFor<CREATE_4>>(&bytes.as_slice()).await?,
            _ => unreachable!("creation label"),
        }
    }
    creator.send::<Loan<FIGURE_STORE>>(store).await?;
    Ok(())
}

pub async fn figure_guest(module: &[u8], mut root: Endpoint<'_, FIGURE_VM>) -> ShellResult<()> {
    let file = root.recv::<Loan<FIGURE_FILE>>().await?;
    let [
        (mut a, mut created_a),
        (mut b, mut created_b),
        (mut c, mut created_c),
        (mut insert_a, mut inserted_a),
        (mut insert_b, mut inserted_b),
        (mut insert_c, mut inserted_c),
    ] = [(); 6].map(|_| (vec![0; GUEST_MEMORY_BYTES], vec![0; GUEST_MEMORY_BYTES]));
    let file = ChoreoFsFile::new(FIGURE_FILES[0], file)
        .write_as::<Request<CREATE_1>>(&mut a)
        .read_as::<ResultFor<CREATE_1>>(&mut created_a)
        .file(FIGURE_FILES[1])?
        .write_as::<Request<CREATE_3>>(&mut b)
        .read_as::<ResultFor<CREATE_3>>(&mut created_b)
        .file(FIGURE_FILES[2])?
        .write_as::<Request<CREATE_4>>(&mut c)
        .read_as::<ResultFor<CREATE_4>>(&mut created_c)
        .file(FIGURE_FILES[3])?
        .write_as::<FigureLookup<INSERT_1>>(&mut insert_a)
        .read_as::<ResultFor<INSERT_1>>(&mut inserted_a)
        .file(FIGURE_FILES[4])?
        .write_as::<FigureLookup<INSERT_3>>(&mut insert_b)
        .read_as::<ResultFor<INSERT_3>>(&mut inserted_b)
        .file(FIGURE_FILES[5])?
        .write_as::<FigureLookup<INSERT_4>>(&mut insert_c)
        .read_as::<ResultFor<INSERT_4>>(&mut inserted_c);
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
