//! The sole DOCUMENT owner works on the next chapter admitted by its projection.
use super::super::{CHAPTERS, global::*};
use crate::{
    imp::json::encode,
    shell::{ShellError, ShellResult, imp::service::ReceivedEffects},
};
use hibana::Endpoint;
use serde_json::{Map, Value};

pub async fn initial(
    document: &mut Endpoint<'_, DOCUMENT>,
    effects: &impl ReceivedEffects,
) -> ShellResult<Value> {
    let mut sections = Map::new();
    for _ in CHAPTERS {
        let branch = document.offer().await?;
        let chapter = branch.label();
        let (actual, writer) = match chapter {
            DRAFT_1 => (
                effects
                    .apply(branch.recv::<DraftRequest<DRAFT_1>>().await?)
                    .await?,
                "WRITER_A",
            ),
            DRAFT_2 => (
                effects
                    .apply(branch.recv::<DraftRequest<DRAFT_2>>().await?)
                    .await?,
                "WRITER_A",
            ),
            DRAFT_3 => (
                effects
                    .apply(branch.recv::<DraftRequest<DRAFT_3>>().await?)
                    .await?,
                "WRITER_B",
            ),
            label => {
                return Err(ShellError::message(format!(
                    "unexpected first draft {label}"
                )));
            }
        };
        let section = CHAPTERS[usize::from(chapter - DRAFT_1)];
        let data = &actual["data"];
        if actual["tool_name"] != "write_section"
            || actual["success"] != true
            || data["section"] != section
            || data["agent"] != writer
            || data["artifact"]["id"].as_str().is_none_or(str::is_empty)
            || !data["artifact"]["content"].is_string()
        {
            return Err(ShellError::message(
                "write_section did not supply the admitted chapter's artifact",
            ));
        }
        let bytes = encode(&actual)?;
        match chapter {
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
            _ => unreachable!("the admitted draft determines the response"),
        }
        sections.insert(section.to_owned(), data["artifact"].clone());
    }
    Ok(Value::Object(sections))
}
