//! The reviewer's request is tied to the fact check for its exact scope.
use super::{
    facts::{CHECK_3, CHECK_4, CHECK_PART_I, FACT_SCOPES},
    writers::material,
};
use crate::shell::{
    GUEST_MEMORY_BYTES, ShellResult,
    global::{Exit, FileExit, Loan},
};
use hibana::{
    Endpoint, g,
    runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload},
};
use hibana_wasip1_runtime::{
    ChoreoFsFile, ChoreoFsFiles, GuestMemory, HibanaWasiGuestStorage,
    choreofs::{ChoreoFsObject, FdSpec, ObjectId},
    protocol::{FdBinding, FdWriteRow},
};
use serde_json::Value;

pub const REVIEW_FILE: u8 = 31;
pub const REVIEW_DATA: u8 = 32;
pub const REVIEW_VM: u8 = 33;
pub const REVIEW_IO: u8 = 34;

pub type ReviewInput<'a> = g::Msg<43, &'a [u8]>;
pub type ReviewRequest<const OP: u8> = g::Msg<OP, ReviewArguments<OP>>;

pub struct ReviewArguments<const OP: u8>(Value);

#[cfg(test)]
impl<const OP: u8> ReviewArguments<OP> {
    pub fn new(value: Value) -> ShellResult<Self> {
        let bytes = crate::imp::json::encode(&value)?;
        Self::validate_payload(Payload::new(&bytes))
            .map_err(|_| crate::shell::ShellError::message("invalid review arguments"))?;
        Ok(Self(value))
    }
}

impl<const OP: u8> crate::shell::imp::service::SimulatorOperation for ReviewArguments<OP> {
    const RPC: (&'static str, &'static str) = ("REVIEWER", "review_sections");
}

impl<const OP: u8> WireEncode for ReviewArguments<OP> {
    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        let mut cursor = std::io::Cursor::new(out);
        serde_json::to_writer(&mut cursor, &self.0).map_err(|_| CodecError::Truncated)?;
        Ok(cursor.position() as usize)
    }
}

impl<const OP: u8> WirePayload for ReviewArguments<OP> {
    const SCHEMA_ID: u32 = 0xac7e_0100 | OP as u32;
    type Decoded<'a> = ReceivedPayload<'a, Self>;

    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        let fields = value.as_object().ok_or(CodecError::Malformed)?;
        let (scope, chapters): (&str, &[&str]) = match OP {
            CHECK_PART_I => (FACT_SCOPES[0], &["CHAPTER_1", "CHAPTER_2"]),
            CHECK_3 => (FACT_SCOPES[1], &["CHAPTER_3"]),
            CHECK_4 => (FACT_SCOPES[2], &["CHAPTER_4"]),
            _ => return Err(CodecError::Malformed),
        };
        if value["section_name"] != scope
            || fields
                .get("feedback")
                .is_some_and(|value| !value.is_string())
            || fields.keys().any(|key| {
                !matches!(
                    key.as_str(),
                    "section_name" | "decision" | "minor_edits" | "feedback"
                )
            })
        {
            return Err(CodecError::Malformed);
        }
        match (fields.get("decision"), fields.get("minor_edits")) {
            (None, None) => Ok(()),
            (Some(decision), None) if matches!(decision.as_str(), Some("accept" | "major")) => {
                Ok(())
            }
            (Some(decision), Some(edits)) if decision == "minor" => {
                let edits = edits
                    .as_array()
                    .filter(|items| !items.is_empty())
                    .ok_or(CodecError::Malformed)?;
                if edits.iter().all(|edit| {
                    edit.as_object().is_some_and(|fields| fields.len() == 3)
                        && edit["section_name"]
                            .as_str()
                            .is_some_and(|name| chapters.contains(&name))
                        && edit["before"]
                            .as_str()
                            .is_some_and(|before| !before.is_empty())
                        && edit["after"].is_string()
                        && edit["before"] != edit["after"]
                }) {
                    Ok(())
                } else {
                    Err(CodecError::Malformed)
                }
            }
            _ => Err(CodecError::Malformed),
        }
    }

    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self::Decoded<'a> {
        input
    }
}

pub const REVIEW_FILES: [ChoreoFsObject<'static>; 4] = [
    material(b"service/inbox", 4),
    ChoreoFsObject::writable(
        b"service/review_sections/CHAPTERS_1_2",
        ObjectId(5),
        FdSpec::new(5, 64),
        FdBinding::write(FdWriteRow::Object),
    ),
    ChoreoFsObject::writable(
        b"service/review_sections/CHAPTER_3",
        ObjectId(6),
        FdSpec::new(6, 64),
        FdBinding::write(FdWriteRow::Object),
    ),
    ChoreoFsObject::writable(
        b"service/review_sections/CHAPTER_4",
        ObjectId(7),
        FdSpec::new(7, 64),
        FdBinding::write(FdWriteRow::Object),
    ),
];

pub async fn review_guest(module: &[u8], mut root: Endpoint<'_, REVIEW_VM>) -> ShellResult<()> {
    let file = root.recv::<Loan<REVIEW_FILE>>().await?;
    let mut inbox = vec![0; GUEST_MEMORY_BYTES];
    let [mut one, mut two, mut three] = [(); 3].map(|_| vec![0; GUEST_MEMORY_BYTES]);
    let file = ChoreoFsFile::new(REVIEW_FILES[0], file)
        .read_as::<ReviewInput>(&mut inbox)
        .file(REVIEW_FILES[1])?
        .write_as::<ReviewRequest<CHECK_PART_I>>(&mut one)
        .file(REVIEW_FILES[2])?
        .write_as::<ReviewRequest<CHECK_3>>(&mut two)
        .file(REVIEW_FILES[3])?
        .write_as::<ReviewRequest<CHECK_4>>(&mut three);
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
