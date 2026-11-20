//! The chapter carrier and the current holder use the same section continuation.
use super::{global::*, publication::SECTIONS};
use crate::{
    imp::json::{decode, encode, same},
    shell::{
        ShellError, ShellResult,
        imp::{
            review_reply::ReviewReply,
            service::{ReceivedEffects, SimulatorOperation},
        },
    },
};
use hibana::{
    Endpoint, EndpointBorrow,
    g::Message,
    runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload},
};
use serde_json::{Value, json};

pub const AUTHORS: [&str; 4] = [
    "RESEARCHER_A",
    "RESEARCHER_B",
    "RESEARCHER_C",
    "RESEARCHER_D",
];
pub const TOPICS: [&str; 4] = ["subtopic_a", "subtopic_b", "subtopic_c", "subtopic_d"];

/// A request names the exact chapter on the file's own Endpoint.
#[derive(Debug)]
pub struct ChapterArguments<const SECTION: u8, const OPERATION: u8>(Vec<u8>);

impl<const SECTION: u8, const OPERATION: u8> ChapterArguments<SECTION, OPERATION> {
    #[cfg(test)]
    pub fn from_value(value: &Value) -> ShellResult<Self> {
        let bytes = encode(value)?;
        Self::validate_payload(Payload::new(&bytes))
            .map_err(|_| ShellError::message("invalid survey chapter request"))?;
        Ok(Self(bytes))
    }
}

impl<const SECTION: u8, const OPERATION: u8> WireEncode for ChapterArguments<SECTION, OPERATION> {
    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        out.get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(&self.0);
        Ok(self.0.len())
    }
}

impl<const SECTION: u8, const OPERATION: u8> SimulatorOperation
    for ChapterArguments<SECTION, OPERATION>
{
    const RPC: (&'static str, &'static str) = match OPERATION {
        RESEARCH => (AUTHORS[SECTION as usize - 1], "research_topic"),
        WRITE => (AUTHORS[SECTION as usize - 1], "write_section"),
        REVISION => (AUTHORS[SECTION as usize - 1], "revise_section"),
        REFERENCES => (AUTHORS[SECTION as usize - 1], "update_references"),
        CREATE_FIGURE => ("DATA_ANALYST", "create_figure"),
        INSERT_FIGURE => ("DATA_ANALYST", "insert_figure"),
        PEER_REVIEW => ("REVIEWER", "peer_review"),
        _ => panic!("unknown survey chapter operation"),
    };
}

impl<const SECTION: u8, const OPERATION: u8> WirePayload for ChapterArguments<SECTION, OPERATION> {
    const SCHEMA_ID: u32 = 0xac3b_0000 | ((SECTION as u32) << 8) | OPERATION as u32;
    type Decoded<'a> = ReceivedPayload<'a, Self>;

    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        let fields = value.as_object().ok_or(CodecError::Malformed)?;
        let index = usize::from(SECTION).wrapping_sub(1);
        let chapter = SECTIONS.get(index).ok_or(CodecError::Malformed)?;
        let (key, expected, optional) = match OPERATION {
            PEER_REVIEW => ("manuscript", Some(*chapter), None),
            REVISION => ("section_name", Some(*chapter), Some("feedback")),
            RESEARCH => ("topic", Some(TOPICS[index]), None),
            WRITE => ("section_name", Some(*chapter), Some("content_notes")),
            REFERENCES => ("section_name", Some(*chapter), None),
            CREATE_FIGURE => ("description", None, None),
            INSERT_FIGURE => ("section_name", Some(*chapter), Some("figure_id")),
            _ => return Err(CodecError::Malformed),
        };
        let field = fields
            .get(key)
            .and_then(Value::as_str)
            .ok_or(CodecError::Malformed)?;
        if field.trim().is_empty()
            || expected.is_some_and(|value| field != value)
            || fields.iter().any(|(name, value)| {
                (name != key && Some(name.as_str()) != optional) || !value.is_string()
            })
            || (OPERATION == INSERT_FIGURE
                && fields
                    .get("figure_id")
                    .and_then(Value::as_str)
                    .is_none_or(str::is_empty))
        {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }

    fn decode_validated_payload(input: ReceivedPayload<'_, Self>) -> Self::Decoded<'_> {
        input
    }
}

/// Ordinary response bytes with a distinct contract for each concurrent chapter.
pub struct ChapterBytes<'a, const SECTION: u8>(pub &'a [u8]);

impl<const SECTION: u8> WireEncode for ChapterBytes<'_, SECTION> {
    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        out.get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(self.0);
        Ok(self.0.len())
    }
}

impl<const SECTION: u8> WirePayload for ChapterBytes<'_, SECTION> {
    const SCHEMA_ID: u32 = 0xac3c_0000 | SECTION as u32;
    type Decoded<'a> = &'a [u8];

    fn validate_payload(_input: Payload<'_>) -> Result<(), CodecError> {
        Ok(())
    }

    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> &'a [u8] {
        input.as_bytes()
    }
}

pub async fn review<const SECTION: u8>(
    reviewer: &mut EndpointBorrow<'_, '_, REVIEWER>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    let chapter = SECTIONS[usize::from(SECTION) - 1];
    let mut draft = decode(reviewer.recv::<Draft<SECTION>>().await?)?;
    let mut section = reviewer.recv::<Accepted<SECTION>>().await?;
    loop {
        let notice = encode(&draft)?;
        section
            .send::<ReviewInput<SECTION>>(&ChapterBytes(&notice))
            .await?;
        let received = section.recv::<Review<SECTION>>().await?;
        let result = effects.apply(received).await?;
        let data = &result["data"];
        if result["tool_name"] != "peer_review"
            || !result["success"].is_boolean()
            || data["agent"] != "REVIEWER"
            || data["scope"] != chapter
            || data["review_id"].as_str().is_none_or(str::is_empty)
        {
            return Err(ShellError::message(
                "review result differs from its actual chapter",
            ));
        }
        same(&data["input_snapshot"], &draft)?;
        let bytes = encode(&result)?;
        if result["success"] == true {
            let proof = ReviewReply::from_bytes(&bytes)?;
            section.send::<PeerAccepted>(&proof).await?;
            section
                .send::<ReviewResult<SECTION>>(&ChapterBytes(&bytes))
                .await?;
            section.send::<Reviewed<SECTION>>(&proof).await?;
            reviewer.send::<Accepted<SECTION>>(section).await?;
            return Ok(());
        }

        let proof = ReviewReply::from_bytes(&bytes)?;
        section.send::<PeerRejected>(&proof).await?;
        section
            .send::<ReviewResult<SECTION>>(&ChapterBytes(&bytes))
            .await?;
        reviewer.send::<Accepted<SECTION>>(section).await?;
        draft = decode(reviewer.recv::<Draft<SECTION>>().await?)?;
        section = reviewer.recv::<Accepted<SECTION>>().await?;
    }
}

#[hibana_runtime::scope]
pub async fn author<'r, const SECTION: u8, const AUTHOR: u8>(
    author: &mut Endpoint<'r, AUTHOR>,
    mut section: Endpoint<'r, SECTION>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    let index = usize::from(SECTION) - 1;
    let chapter = SECTIONS[index];
    let name = AUTHORS[index];
    let received = section.recv::<Research<SECTION>>().await?;
    let args = decode(received.as_bytes())?;
    let research = effects.apply(received).await?;
    if research["tool_name"] != "research_topic"
        || research["success"] != true
        || research["data"]["agent"] != name
        || research["data"]["topic"] != args["topic"]
        || research["data"]["artifact"]["id"]
            .as_str()
            .is_none_or(str::is_empty)
        || !research["data"]["artifact"]["content"].is_string()
    {
        return Err(ShellError::message(
            "research result differs from its actual request",
        ));
    }
    let bytes = encode(&research)?;
    async {
        let [mut file, mut analyst] = section.par()?;
        async {
            file.send::<Researched<SECTION>>(&ChapterBytes(&bytes))
                .await?;
        };
        async {
            analyst
                .send::<ResearchForAnalyst<SECTION>>(&ChapterBytes(&bytes))
                .await?;
        };
        Ok::<(), ShellError>(())
    }
    .await?;
    let (draft, references, figure) = async {
        let [mut draft, mut references, mut figures] = section.par()?;
        async {
            let received = draft.recv::<Write<SECTION>>().await?;
            let args = decode(received.as_bytes())?;
            let actual = effects.apply(received).await?;
            let material = &actual["data"]["artifact"];
            if actual["tool_name"] != "write_section"
                || actual["success"] != true
                || actual["data"]["agent"] != name
                || actual["data"]["section"] != chapter
                || material["id"].as_str().is_none_or(str::is_empty)
                || !material["content"].is_string()
            {
                return Err(ShellError::message("write produced a different chapter"));
            }
            if let Some(notes) = args.get("content_notes") {
                same(&material["content"], notes)?;
            }
            same(
                &material["research_inputs"],
                &json!({TOPICS[index]:research["data"]["artifact"]}),
            )?;
            let bytes = encode(&actual)?;
            draft
                .send::<Written<SECTION>>(&ChapterBytes(&bytes))
                .await?;
            Ok::<_, ShellError>(actual)
        };
        async {
            references
                .send::<ReferenceSource<SECTION>>(&ChapterBytes(&bytes))
                .await?;
            decode(references.recv::<ReferencesForChapter<SECTION>>().await?)
        };
        async {
            figures
                .send::<FigureSource<SECTION>>(&ChapterBytes(&bytes))
                .await?;
            decode(figures.recv::<FigureForChapter<SECTION>>().await?)
        };
    }
    .await?;
    let prepared = encode(&json!({
        "sections":{chapter:draft["data"]["artifact"]},
        "references":{chapter:references["data"]["artifact"]},
        "figures":{chapter:figure["data"]["artifact"]},
    }))?;
    section
        .send::<Prepared<SECTION>>(&ChapterBytes(&prepared))
        .await?;
    author.send::<Accepted<SECTION>>(section).await?;

    loop {
        let branch = author.offer().await?;
        let rejected = match branch.label() {
            PeerAccepted::LOGICAL_LABEL => {
                let review = branch.recv::<PeerAccepted>().await?.to_vec();
                author
                    .send::<AuthorNotice<SECTION>>(&ChapterBytes(&review))
                    .await?;
                return Ok(());
            }
            PeerRejected::LOGICAL_LABEL => decode(branch.recv::<PeerRejected>().await?)?,
            _ => return Err(ShellError::message("unexpected survey author continuation")),
        };
        let bytes = encode(&rejected)?;
        author
            .send::<AuthorNotice<SECTION>>(&ChapterBytes(&bytes))
            .await?;
        let mut section = author.recv::<Accepted<SECTION>>().await?;
        let received = section.recv::<Revise<SECTION>>().await?;
        let arguments = decode(received.as_bytes())?;
        let result = effects.apply(received).await?;
        let mut draft = rejected["data"]["input_snapshot"].clone();
        let old = &draft["sections"][chapter];
        let actual = &result["data"]["artifact"];
        if result["success"] != true
            || result["tool_name"] != "revise_section"
            || result["data"]["agent"] != name
            || result["data"]["section"] != chapter
            || actual["id"].as_str().is_none_or(str::is_empty)
            || actual["id"] == old["id"]
            || !actual["content"].is_string()
            || actual["content"] == old["content"]
        {
            return Err(ShellError::message(
                "revision did not produce a changed chapter",
            ));
        }
        if let Some(feedback) = arguments.get("feedback") {
            same(&actual["feedback"], feedback)?;
        }
        same(&actual["research_inputs"], &old["research_inputs"])?;
        draft["sections"][chapter] = actual.clone();
        let reply = encode(&result)?;
        section
            .send::<Revised<SECTION>>(&ChapterBytes(&reply))
            .await?;
        let updated = encode(&draft)?;
        section
            .send::<Draft<SECTION>>(&ChapterBytes(&updated))
            .await?;
        author.send::<Accepted<SECTION>>(section).await?;
    }
}
