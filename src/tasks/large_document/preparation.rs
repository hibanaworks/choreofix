//! Initial writing, references and completed cross-chapter inputs for 7H.
use super::{facts::*, figures::*, publication::*, review::ReviewInput};
use crate::{
    imp::json::{decode, encode, same},
    shell::{ShellError, ShellResult, global::Loan, imp::service::ReceivedEffects},
};
use hibana::{
    Endpoint, g,
    runtime::{
        program::Projectable,
        wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload},
    },
};
use serde_json::{Value, json};

pub const DOCUMENT: u8 = 1;
pub const DOCUMENT_OWNER: u8 = 2;
pub const DRAFTING: u8 = 3;
pub const REFERENCES: u8 = 4;
pub const WRITER_A_FILE: u8 = 5;
pub const WRITER_B_FILE: u8 = 6;
pub const WRITER_C_FILE: u8 = 7;

pub const CHAPTERS: [&str; 4] = ["CHAPTER_1", "CHAPTER_2", "CHAPTER_3", "CHAPTER_4"];
const WRITERS: [&str; 4] = ["WRITER_A", "WRITER_A", "WRITER_B", "WRITER_C"];

pub const WRITE_1: u8 = 1;
pub const WRITE_2: u8 = 2;
pub const WRITE_3: u8 = 3;
pub const WRITE_4: u8 = 4;
pub const REFERENCES_1: u8 = 5;
pub const REFERENCES_2: u8 = 6;
pub const REFERENCES_3: u8 = 7;
pub const REFERENCES_4: u8 = 8;
pub const FINALIZE_1: u8 = 13;
pub const FINALIZE_2: u8 = 14;
pub const REVISE_3: u8 = 44;
pub const REVISE_4: u8 = 45;
pub const MINOR_1: u8 = 42;
pub const MINOR_2: u8 = 43;
pub const MINOR_3: u8 = 46;
pub const MINOR_4: u8 = 48;

pub struct ChapterArguments<const OP: u8, const RESOURCE: bool = true>(pub(super) Value);

#[cfg(test)]
impl<const OP: u8, const RESOURCE: bool> ChapterArguments<OP, RESOURCE> {
    pub fn new(value: Value) -> ShellResult<Self> {
        Self::validate(Payload::new(&encode(&value)?))?;
        Ok(Self(value))
    }
}

impl<const OP: u8, const RESOURCE: bool> WireEncode for ChapterArguments<OP, RESOURCE> {
    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        let mut cursor = std::io::Cursor::new(out);
        serde_json::to_writer(&mut cursor, &self.0).map_err(|_| CodecError::Truncated)?;
        Ok(cursor.position() as usize)
    }
}

impl<const OP: u8, const RESOURCE: bool> ChapterArguments<OP, RESOURCE> {
    fn validate(input: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        let fields = value.as_object().ok_or(CodecError::Malformed)?;
        if OP == EDITORIAL {
            return if fields.len() == 1
                && value["review_summary"]
                    .as_str()
                    .is_some_and(|summary| !summary.trim().is_empty())
            {
                Ok(())
            } else {
                Err(CodecError::Malformed)
            };
        }
        if OP == COMBINE {
            let mut sections: Vec<_> = value["sections"]
                .as_str()
                .ok_or(CodecError::Malformed)?
                .split(',')
                .map(str::trim)
                .collect();
            sections.sort_unstable();
            return if fields.len() == 1 && sections == CHAPTERS {
                Ok(())
            } else {
                Err(CodecError::Malformed)
            };
        }
        if (CREATE_1..=CREATE_4).contains(&OP) {
            return if fields.len() == 1
                && value["description"]
                    .as_str()
                    .is_some_and(|text| !text.trim().is_empty())
            {
                Ok(())
            } else {
                Err(CodecError::Malformed)
            };
        }
        if (INSERT_1..=INSERT_4).contains(&OP) {
            return if fields.len() == 2
                && value["section_name"] == FIGURE_CHAPTERS[usize::from(OP - INSERT_1)]
                && value["figure_id"].as_str().is_some_and(|id| !id.is_empty())
            {
                Ok(())
            } else {
                Err(CodecError::Malformed)
            };
        }
        let chapter = match OP {
            WRITE_1..=WRITE_4 => CHAPTERS[usize::from(OP - WRITE_1)],
            REFERENCES_1..=REFERENCES_4 => CHAPTERS[usize::from(OP - REFERENCES_1)],
            FINALIZE_1..=FINALIZE_2 => CHAPTERS[usize::from(OP - FINALIZE_1)],
            RECONSIDER_1..=RECONSIDER_2 => CHAPTERS[usize::from(OP - RECONSIDER_1)],
            REVISE_3..=REVISE_4 => CHAPTERS[2 + usize::from(OP - REVISE_3)],
            MINOR_1..=MINOR_2 => CHAPTERS[usize::from(OP - MINOR_1)],
            MINOR_3 => CHAPTERS[2],
            MINOR_4 => CHAPTERS[3],
            CHECK_PART_I..=CHECK_4 => FACT_SCOPES[usize::from(OP - CHECK_PART_I)],
            _ => return Err(CodecError::Malformed),
        };
        if matches!(OP, MINOR_1 | MINOR_2 | MINOR_3 | MINOR_4) {
            return if fields.len() == 2
                && value["section_name"] == chapter
                && value["review_id"].as_str().is_some_and(|id| !id.is_empty())
            {
                Ok(())
            } else {
                Err(CodecError::Malformed)
            };
        }
        let content_field = match OP {
            WRITE_1..=WRITE_4 => Some("content_notes"),
            FINALIZE_1..=FINALIZE_2 | RECONSIDER_1..=RECONSIDER_2 | REVISE_3..=REVISE_4 => {
                Some("feedback")
            }
            _ => None,
        };
        if value["section_name"] != chapter
            || fields.iter().any(|(key, value)| {
                (key != "section_name" && Some(key.as_str()) != content_field) || !value.is_string()
            })
        {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }
}

// A figure lookup conveys data to the two resource owners. Only the
// message received back from the figure store can submit the insertion.
impl<const OP: u8> WirePayload for ChapterArguments<OP, false> {
    const SCHEMA_ID: u32 = 0xac7e_0200 | OP as u32;
    type Decoded<'a> = Self;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        Self::validate(input)
    }
    fn decode_validated_payload(input: ReceivedPayload<'_, Self>) -> Self {
        Self(serde_json::from_slice(input.as_bytes()).expect("validated figure lookup"))
    }
}

impl<const OP: u8> WirePayload for ChapterArguments<OP> {
    const SCHEMA_ID: u32 = 0xac7e_0000 | OP as u32;
    type Decoded<'a> = ReceivedPayload<'a, Self>;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        Self::validate(input)
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self::Decoded<'a> {
        input
    }
}

impl<const OP: u8> crate::shell::imp::service::SimulatorOperation for ChapterArguments<OP> {
    const RPC: (&'static str, &'static str) = match OP {
        WRITE_1..=WRITE_4 => (WRITERS[(OP - WRITE_1) as usize], "write_section"),
        REFERENCES_1..=REFERENCES_4 => (WRITERS[(OP - REFERENCES_1) as usize], "update_references"),
        FINALIZE_1 | FINALIZE_2 | RECONSIDER_1 | RECONSIDER_2 | MINOR_1 | MINOR_2 => {
            ("WRITER_A", "revise_section")
        }
        REVISE_3 | MINOR_3 => ("WRITER_B", "revise_section"),
        REVISE_4 | MINOR_4 => ("WRITER_C", "revise_section"),
        CREATE_1..=CREATE_4 => ("FIGURE_CREATOR", "create_figure"),
        INSERT_1..=INSERT_4 => ("FIGURE_CREATOR", "insert_figure"),
        CHECK_PART_I..=CHECK_4 => ("FACT_CHECKER", "fact_check"),
        EDITORIAL => ("EDITOR_IN_CHIEF", "make_editorial_decision"),
        COMBINE => ("EDITOR_IN_CHIEF", "combine_sections"),
        _ => panic!("unknown manuscript operation"),
    };
}

pub type FigureLookup<const OP: u8> = g::Msg<OP, ChapterArguments<OP, false>>;
pub type Request<const OP: u8> = g::Msg<OP, ChapterArguments<OP>>;
pub type ResultFor<'a, const OP: u8> = g::Msg<OP, &'a [u8]>;
pub type ContributionToPartI<'a> = g::Msg<9, &'a [u8]>;
pub type EvaluationToPartI<'a> = g::Msg<10, &'a [u8]>;
pub type EvaluationToChapter3<'a> = g::Msg<11, &'a [u8]>;
type StableEvaluation<'a> = g::Msg<12, &'a [u8]>;

pub(super) fn operation<const FILE: u8, const OWNER: u8, const OP: u8>() -> impl Projectable {
    g::seq(
        g::send::<FILE, OWNER, Request<OP>>(),
        g::send::<OWNER, FILE, ResultFor<OP>>(),
    )
}

pub fn preparation() -> impl Projectable {
    let contribution = g::seq(
        g::par(
            create_figure::<CREATE_3>(),
            operation::<WRITER_B_FILE, DOCUMENT, WRITE_3>(),
        ),
        g::seq(
            insert_figure::<INSERT_3>(),
            g::send::<DOCUMENT, FACT_3, FactMaterial<CHECK_3>>(),
        ),
    );
    let evaluation = g::seq(
        g::par(
            create_figure::<CREATE_4>(),
            operation::<WRITER_C_FILE, DOCUMENT, WRITE_4>(),
        ),
        g::seq(
            insert_figure::<INSERT_4>(),
            g::send::<DOCUMENT, FACT_4, FactMaterial<CHECK_4>>(),
        ),
    );
    let stable_part_ii = g::seq(
        chapter_flow::<FACT_4, WRITER_C_FILE, CHECK_4, REVISE_4, WRITE_4, MINOR_4, CHAPTER_4_OUTCOME>(
        ),
        g::seq(
            g::send::<DOCUMENT, REFERENCES, StableEvaluation>(),
            g::par(
                g::send::<REFERENCES, WRITER_A_FILE, EvaluationToPartI>(),
                g::seq(
                    g::send::<REFERENCES, WRITER_B_FILE, EvaluationToChapter3>(),
                    g::seq(
                        operation::<WRITER_B_FILE, REFERENCES, REFERENCES_3>(),
                        g::seq(
                            g::send::<REFERENCES, FACT_3, FactMaterial<MATERIAL_3>>(),
                            g::seq(
                                chapter_flow::<
                                    FACT_3,
                                    WRITER_B_FILE,
                                    CHECK_3,
                                    REVISE_3,
                                    WRITE_3,
                                    MINOR_3,
                                    CHAPTER_3_OUTCOME,
                                >(),
                                g::send::<DOCUMENT, WRITER_A_FILE, ContributionToPartI>(),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );
    let drafts = g::seq(
        g::par(
            g::send::<DOCUMENT_OWNER, DRAFTING, Loan<DOCUMENT>>(),
            g::send::<FIGURE_OWNER, FIGURE_CREATOR, Loan<FIGURE_STORE>>(),
        ),
        g::seq(
            g::par(
                g::seq(
                    g::par(
                        create_figure::<CREATE_1>(),
                        operation::<WRITER_A_FILE, DOCUMENT, WRITE_1>(),
                    ),
                    insert_figure::<INSERT_1>(),
                ),
                g::par(
                    operation::<WRITER_A_FILE, DOCUMENT, WRITE_2>(),
                    g::par(contribution, evaluation),
                ),
            ),
            g::send::<FIGURE_CREATOR, FIGURE_OWNER, Loan<FIGURE_STORE>>(),
        ),
    );
    let preparation = g::par(
        g::seq(
            g::par(
                operation::<WRITER_A_FILE, REFERENCES, REFERENCES_1>(),
                operation::<WRITER_A_FILE, REFERENCES, REFERENCES_2>(),
            ),
            g::send::<REFERENCES, FACT_PART_I, FactMaterial<MATERIAL_PART_I>>(),
        ),
        g::seq(
            g::par(
                drafts,
                g::seq(
                    operation::<WRITER_C_FILE, REFERENCES, REFERENCES_4>(),
                    g::send::<REFERENCES, FACT_4, FactMaterial<MATERIAL_4>>(),
                ),
            ),
            g::seq(
                stable_part_ii,
                g::seq(
                    g::par(
                        operation::<WRITER_A_FILE, DOCUMENT, FINALIZE_1>(),
                        operation::<WRITER_A_FILE, DOCUMENT, FINALIZE_2>(),
                    ),
                    g::seq(
                        g::send::<DOCUMENT, FACT_PART_I, FactMaterial<CHECK_PART_I>>(),
                        g::send::<DRAFTING, DOCUMENT_OWNER, Loan<DOCUMENT>>(),
                    ),
                ),
            ),
        ),
    );
    g::seq(
        preparation,
        g::seq(part_i_fact_flow::<FACT_PART_I>(), publication()),
    )
}

pub(super) fn artifact(result: &Value, tool: &str, index: usize) -> ShellResult<Value> {
    let data = &result["data"];
    if result["success"] != true
        || result["tool_name"] != tool
        || data["agent"] != WRITERS[index]
        || data["section"] != CHAPTERS[index]
        || data["artifact"]["id"].as_str().is_none_or(str::is_empty)
        || !data["artifact"]["content"].is_string()
    {
        return Err(ShellError::message(
            "material does not match the admitted chapter",
        ));
    }
    Ok(data["artifact"].clone())
}

/// One borrower performs document writes in the order admitted by its projection.
#[hibana_runtime::scope]
pub async fn write_drafts(
    drafting: &mut Endpoint<DRAFTING>,
    effects: &impl ReceivedEffects,
) -> ShellResult<[Value; 4]> {
    let mut document = drafting.recv::<Loan<DOCUMENT>>().await?;
    let mut chapters = [const { Value::Null }; 4];
    let mut part_i_figure = Value::Null;
    for _ in 0..CHAPTERS.len() + FIGURE_CHAPTERS.len() {
        let branch = document.offer().await?;
        let label = branch.label();
        if (INSERT_1..=INSERT_4).contains(&label) {
            let (args, actual) = match label {
                INSERT_1 => {
                    let lookup = branch.recv::<FigureLookup<INSERT_1>>().await?;
                    document.send::<FigureLookup<INSERT_1>>(&lookup).await?;
                    let confirmed = document.recv::<Request<INSERT_1>>().await?;
                    same(&lookup.0, &decode(confirmed.as_bytes())?)?;
                    (lookup.0, effects.apply(confirmed).await?)
                }
                INSERT_3 => {
                    let lookup = branch.recv::<FigureLookup<INSERT_3>>().await?;
                    document.send::<FigureLookup<INSERT_3>>(&lookup).await?;
                    let confirmed = document.recv::<Request<INSERT_3>>().await?;
                    same(&lookup.0, &decode(confirmed.as_bytes())?)?;
                    (lookup.0, effects.apply(confirmed).await?)
                }
                INSERT_4 => {
                    let lookup = branch.recv::<FigureLookup<INSERT_4>>().await?;
                    document.send::<FigureLookup<INSERT_4>>(&lookup).await?;
                    let confirmed = document.recv::<Request<INSERT_4>>().await?;
                    same(&lookup.0, &decode(confirmed.as_bytes())?)?;
                    (lookup.0, effects.apply(confirmed).await?)
                }
                _ => unreachable!("figure insertion label"),
            };
            let data = &actual["data"];
            if actual["tool_name"] != "insert_figure"
                || actual["success"] != true
                || data["agent"] != "FIGURE_CREATOR"
                || data["section"] != args["section_name"]
                || data["figure_id"] != args["figure_id"]
                || data["artifact"]["id"] != args["figure_id"]
            {
                return Err(ShellError::message(
                    "insertion does not match the admitted figure and chapter",
                ));
            }
            let bytes = encode(&actual)?;
            match label {
                INSERT_1 => {
                    document
                        .send::<ResultFor<INSERT_1>>(&bytes.as_slice())
                        .await?;
                    part_i_figure = actual["data"]["artifact"].clone();
                }
                INSERT_3 => {
                    document
                        .send::<ResultFor<INSERT_3>>(&bytes.as_slice())
                        .await?;
                    let material = json!({"sections":{(CHAPTERS[2]):chapters[2]},
                        "figures":{(CHAPTERS[2]):actual["data"]["artifact"]}});
                    document
                        .send::<FactMaterial<CHECK_3>>(&encode(&material)?.as_slice())
                        .await?;
                }
                INSERT_4 => {
                    document
                        .send::<ResultFor<INSERT_4>>(&bytes.as_slice())
                        .await?;
                    let material = json!({"sections":{(CHAPTERS[3]):chapters[3]},
                        "figures":{(CHAPTERS[3]):actual["data"]["artifact"]}});
                    document
                        .send::<FactMaterial<CHECK_4>>(&encode(&material)?.as_slice())
                        .await?;
                }
                _ => unreachable!("insertion label"),
            }
            continue;
        }
        let result = match label {
            WRITE_1 => {
                effects
                    .apply(branch.recv::<Request<WRITE_1>>().await?)
                    .await?
            }
            WRITE_2 => {
                effects
                    .apply(branch.recv::<Request<WRITE_2>>().await?)
                    .await?
            }
            WRITE_3 => {
                effects
                    .apply(branch.recv::<Request<WRITE_3>>().await?)
                    .await?
            }
            WRITE_4 => {
                effects
                    .apply(branch.recv::<Request<WRITE_4>>().await?)
                    .await?
            }
            _ => {
                return Err(ShellError::message(format!(
                    "unexpected initial chapter {label}"
                )));
            }
        };
        let index = usize::from(label - WRITE_1);
        chapters[index] = artifact(&result, "write_section", index)?;
        let bytes = encode(&result)?;
        match label {
            WRITE_1 => {
                document
                    .send::<ResultFor<WRITE_1>>(&bytes.as_slice())
                    .await?
            }
            WRITE_2 => {
                document
                    .send::<ResultFor<WRITE_2>>(&bytes.as_slice())
                    .await?
            }
            WRITE_3 => {
                document
                    .send::<ResultFor<WRITE_3>>(&bytes.as_slice())
                    .await?
            }
            WRITE_4 => {
                document
                    .send::<ResultFor<WRITE_4>>(&bytes.as_slice())
                    .await?
            }
            _ => unreachable!("draft label was matched before the physical effect"),
        }
    }
    drafting.send::<Loan<DOCUMENT>>(document).await?;
    let fourth = drafting.recv::<ApprovedChapter<CHECK_4>>().await?.to_vec();
    let (_, returned) = async {
        let [mut author, mut custody] = drafting.par()?;
        async {
            author
                .send::<AuthorOutcome<CHAPTER_4_OUTCOME>>(&fourth.as_slice())
                .await
        };
        async { custody.recv::<Loan<DOCUMENT>>().await };
    }
    .await?;
    document = returned;
    let approval = crate::imp::json::decode(&fourth)?;
    chapters[3] = approval["data"]["input_snapshot"]["sections"]["CHAPTER_4"].clone();
    async {
        let [mut checker, mut reviewer] = document.par()?;
        async { checker.send::<FactInput>(&fourth.as_slice()).await };
        async { reviewer.send::<ReviewInput>(&fourth.as_slice()).await };
        Ok(())
    }
    .await?;
    document
        .send::<StableEvaluation>(&crate::imp::json::encode(&chapters[3])?.as_slice())
        .await?;
    drafting.send::<Loan<DOCUMENT>>(document).await?;
    let third = drafting.recv::<ApprovedChapter<CHECK_3>>().await?.to_vec();
    let (_, returned) = async {
        let [mut author, mut custody] = drafting.par()?;
        async {
            author
                .send::<AuthorOutcome<CHAPTER_3_OUTCOME>>(&third.as_slice())
                .await
        };
        async { custody.recv::<Loan<DOCUMENT>>().await };
    }
    .await?;
    document = returned;
    let approval = crate::imp::json::decode(&third)?;
    chapters[2] = approval["data"]["input_snapshot"]["sections"]["CHAPTER_3"].clone();
    async {
        let [mut checker, mut reviewer] = document.par()?;
        async { checker.send::<FactInput>(&third.as_slice()).await };
        async { reviewer.send::<ReviewInput>(&third.as_slice()).await };
        Ok(())
    }
    .await?;
    document
        .send::<ContributionToPartI>(&crate::imp::json::encode(&chapters[2])?.as_slice())
        .await?;
    let inputs = json!({(CHAPTERS[2]):chapters[2], (CHAPTERS[3]):chapters[3]});
    for _ in 0..2 {
        let branch = document.offer().await?;
        let label = branch.label();
        let revised = match label {
            FINALIZE_1 => {
                let request = branch.recv::<Request<FINALIZE_1>>().await?;
                if chapters[0]["input_sections"] != inputs
                    || decode(request.as_bytes())?.get("feedback").is_some()
                {
                    Some(effects.apply(request).await?)
                } else {
                    None
                }
            }
            FINALIZE_2 => {
                let request = branch.recv::<Request<FINALIZE_2>>().await?;
                if chapters[1]["input_sections"] != inputs
                    || decode(request.as_bytes())?.get("feedback").is_some()
                {
                    Some(effects.apply(request).await?)
                } else {
                    None
                }
            }
            _ => {
                return Err(ShellError::message(format!(
                    "unexpected Part I submission {label}"
                )));
            }
        };
        let index = usize::from(label - FINALIZE_1);
        if let Some(revised) = revised {
            let current = artifact(&revised, "revise_section", index)?;
            same(&current["input_sections"], &inputs)?;
            chapters[index] = current;
        }
        let bytes = encode(&chapters[index])?;
        match label {
            FINALIZE_1 => {
                document
                    .send::<ResultFor<FINALIZE_1>>(&bytes.as_slice())
                    .await?
            }
            FINALIZE_2 => {
                document
                    .send::<ResultFor<FINALIZE_2>>(&bytes.as_slice())
                    .await?
            }
            _ => unreachable!("Part I submission label"),
        }
    }
    let material = json!({"sections":{(CHAPTERS[0]):chapters[0], (CHAPTERS[1]):chapters[1]},
        "figures":{(CHAPTERS[0]):part_i_figure}});
    document
        .send::<FactMaterial<CHECK_PART_I>>(&encode(&material)?.as_slice())
        .await?;
    drafting.send::<Loan<DOCUMENT>>(document).await?;
    Ok(chapters)
}

/// Reference effects do not consume the physical document's write authority.
#[hibana_runtime::scope]
pub async fn update_references(
    endpoint: &mut Endpoint<REFERENCES>,
    effects: &impl ReceivedEffects,
) -> ShellResult<[Value; 4]> {
    let ((one, two), (three, four)) = async {
        let [mut part_i, mut part_ii] = endpoint.par()?;
        async {
            let (one, two) = async {
                let [mut one, mut two] = part_i.par()?;
                async {
                    let result = effects
                        .apply(one.recv::<Request<REFERENCES_1>>().await?)
                        .await?;
                    let material = artifact(&result, "update_references", 0)?;
                    one.send::<ResultFor<REFERENCES_1>>(&encode(&result)?.as_slice())
                        .await?;
                    Ok(material)
                };
                async {
                    let result = effects
                        .apply(two.recv::<Request<REFERENCES_2>>().await?)
                        .await?;
                    let material = artifact(&result, "update_references", 1)?;
                    two.send::<ResultFor<REFERENCES_2>>(&encode(&result)?.as_slice())
                        .await?;
                    Ok(material)
                };
            }
            .await?;
            let references = json!({(CHAPTERS[0]):one, (CHAPTERS[1]):two});
            part_i
                .send::<FactMaterial<MATERIAL_PART_I>>(&encode(&references)?.as_slice())
                .await?;
            Ok((one, two))
        };
        async {
            let result = effects
                .apply(part_ii.recv::<Request<REFERENCES_4>>().await?)
                .await?;
            let material = artifact(&result, "update_references", 3)?;
            part_ii
                .send::<ResultFor<REFERENCES_4>>(&encode(&result)?.as_slice())
                .await?;
            let references = json!({(CHAPTERS[3]):material});
            part_ii
                .send::<FactMaterial<MATERIAL_4>>(&encode(&references)?.as_slice())
                .await?;
            let fourth = material;
            let evaluation = part_ii.recv::<StableEvaluation>().await?.to_vec();
            let evaluation_artifact = crate::imp::json::decode(&evaluation)?;
            let ((), material) = async {
                let [mut part, mut contribution] = part_ii.par()?;
                async { part.send::<EvaluationToPartI>(&evaluation.as_slice()).await };
                async {
                    contribution
                        .send::<EvaluationToChapter3>(&evaluation.as_slice())
                        .await?;
                    let result = effects
                        .apply(contribution.recv::<Request<REFERENCES_3>>().await?)
                        .await?;
                    let material = artifact(&result, "update_references", 2)?;
                    if material["input_sections"]
                        != serde_json::json!({(CHAPTERS[3]): evaluation_artifact})
                    {
                        return Err(ShellError::message(
                            "chapter 3 references a different evaluation",
                        ));
                    }
                    contribution
                        .send::<ResultFor<REFERENCES_3>>(&encode(&result)?.as_slice())
                        .await?;
                    let references = json!({(CHAPTERS[2]):material});
                    contribution
                        .send::<FactMaterial<MATERIAL_3>>(&encode(&references)?.as_slice())
                        .await?;
                    Ok(material)
                };
            }
            .await?;
            Ok((material, fourth))
        };
    }
    .await?;
    Ok([one, two, three, four])
}
