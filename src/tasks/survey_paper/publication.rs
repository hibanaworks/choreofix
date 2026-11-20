//! The editor consumes the actual approved chapters and file requests.
use super::global::*;
use crate::{
    imp::json::{decode, encode, same},
    shell::{ShellError, ShellResult, imp::service::Domain},
};
use hibana::{
    Endpoint,
    runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload},
};
use serde_json::{Value, json};

pub const SECTIONS: [&str; 4] = ["section_a", "section_b", "section_c", "section_d"];
const EDITOR_NAME: &str = "EDITOR_IN_CHIEF";
const CATEGORIES: [&str; 3] = ["sections", "references", "figures"];

/// The exact JSON of one admitted editor file operation.
#[derive(Debug)]
pub struct Arguments<const COMBINE: bool>(Vec<u8>);

impl<const COMBINE: bool> Arguments<COMBINE> {
    fn into_value(self) -> Value {
        serde_json::from_slice(&self.0).expect("validated survey editor request")
    }
}

impl<const COMBINE: bool> WireEncode for Arguments<COMBINE> {
    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        out.get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(&self.0);
        Ok(self.0.len())
    }
}

impl<const COMBINE: bool> WirePayload for Arguments<COMBINE> {
    const SCHEMA_ID: u32 = 0xac3a_0000 | COMBINE as u32;
    type Decoded<'a> = Self;

    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        let fields = value.as_object().ok_or(CodecError::Malformed)?;
        let key = if COMBINE {
            "sections"
        } else {
            "review_summary"
        };
        let text = fields
            .get(key)
            .and_then(Value::as_str)
            .ok_or(CodecError::Malformed)?;
        if fields.len() != 1 || text.trim().is_empty() {
            return Err(CodecError::Malformed);
        }
        if COMBINE {
            let mut sections: Vec<_> = chapters(text).collect();
            sections.sort_unstable();
            if sections != SECTIONS {
                return Err(CodecError::Malformed);
            }
        }
        Ok(())
    }

    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self {
        Self(input.as_bytes().to_vec())
    }
}

fn chapters(text: &str) -> impl Iterator<Item = &str> {
    text.split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
}

#[hibana_runtime::scope]
pub async fn run(editor: &mut Endpoint<EDITOR>, domain: &impl Domain) -> ShellResult<Value> {
    let ((a, section_a), (b, section_b), (c, section_c), (d, section_d)) = async {
        let [mut a, mut b, mut c, mut d] = editor.par()?;
        async {
            let proof = decode(a.recv::<Reviewed<SECTION_A>>().await?)?;
            let section = a.recv::<Accepted<SECTION_A>>().await?;
            Ok::<_, ShellError>((proof, section))
        };
        async {
            let proof = decode(b.recv::<Reviewed<SECTION_B>>().await?)?;
            let section = b.recv::<Accepted<SECTION_B>>().await?;
            Ok::<_, ShellError>((proof, section))
        };
        async {
            let proof = decode(c.recv::<Reviewed<SECTION_C>>().await?)?;
            let section = c.recv::<Accepted<SECTION_C>>().await?;
            Ok::<_, ShellError>((proof, section))
        };
        async {
            let proof = decode(d.recv::<Reviewed<SECTION_D>>().await?)?;
            let section = d.recv::<Accepted<SECTION_D>>().await?;
            Ok::<_, ShellError>((proof, section))
        };
    }
    .await?;
    let mut snapshot = json!({"sections":{}, "references":{}, "figures":{}});
    let mut receipts = json!({});
    for (name, review) in SECTIONS.into_iter().zip([a, b, c, d]) {
        let data = &review["data"];
        let source = &data["input_snapshot"];
        if review["tool_name"] != "peer_review"
            || review["success"] != true
            || data["agent"] != "REVIEWER"
            || data["scope"] != name
            || data["review_id"].as_str().is_none_or(str::is_empty)
            || source
                .as_object()
                .is_none_or(|fields| fields.len() != CATEGORIES.len())
        {
            return Err(ShellError::message(
                "invalid peer review for this survey chapter",
            ));
        }
        receipts[name] = data["review_id"].clone();
        for category in CATEGORIES {
            let artifact = &source[category][name];
            if source[category]
                .as_object()
                .is_none_or(|items| items.len() != 1)
                || artifact["id"].as_str().is_none_or(str::is_empty)
                || !artifact["content"].is_string()
            {
                return Err(ShellError::message(
                    "peer review lacks its actual chapter material",
                ));
            }
            snapshot[category][name] = artifact.clone();
        }
    }
    for (index, name) in SECTIONS.iter().enumerate() {
        for other in &SECTIONS[..index] {
            if receipts[*name] == receipts[*other]
                || snapshot["figures"][*name]["id"] == snapshot["figures"][*other]["id"]
            {
                return Err(ShellError::message("two chapters reuse a review or figure"));
            }
        }
    }

    let input = encode(&json!({"input_snapshot":snapshot,"review_receipts":receipts}))?;
    editor.send::<EditorialInput>(&input.as_slice()).await?;
    let arguments = editor.recv::<Editorial>().await?.into_value();
    let decision = domain
        .execute(EDITOR_NAME, "make_editorial_decision", arguments.clone())
        .await
        .map_err(ShellError::message)?;
    let evidence = &decision["data"];
    if decision["tool_name"] != "make_editorial_decision"
        || decision["success"] != true
        || evidence["agent"] != EDITOR_NAME
        || evidence["decision_type"] != "finalize"
        || evidence["decision_id"].as_str().is_none_or(str::is_empty)
    {
        return Err(ShellError::message(
            "missing successful survey editorial decision",
        ));
    }
    same(&evidence["review_summary"], &arguments["review_summary"])?;
    same(&evidence["input_snapshot"], &snapshot)?;
    same(&evidence["review_receipts"], &receipts)?;
    let reply = encode(&decision)?;
    editor.send::<EditorialResult>(&reply.as_slice()).await?;

    let arguments = editor.recv::<Combine>().await?.into_value();
    let combined = domain
        .execute(EDITOR_NAME, "combine_sections", arguments.clone())
        .await
        .map_err(ShellError::message)?;
    let final_data = &combined["data"];
    let sections: Vec<_> =
        chapters(arguments["sections"].as_str().expect("typed sections")).collect();
    if combined["tool_name"] != "combine_sections"
        || combined["success"] != true
        || final_data["agent"] != EDITOR_NAME
        || final_data["artifact_id"].as_str().is_none_or(str::is_empty)
    {
        return Err(ShellError::message("missing actual combined survey"));
    }
    same(&final_data["sections"], &json!(sections))?;
    same(&final_data["input_snapshot"], &snapshot)?;
    same(&final_data["review_receipts"], &receipts)?;
    same(
        &final_data["editorial_receipts"],
        &json!({"finalize": evidence["decision_id"]}),
    )?;
    let reply = encode(&combined)?;
    editor.send::<Combined>(&reply.as_slice()).await?;

    drop((section_a, section_b, section_c, section_d));
    Ok(combined)
}
