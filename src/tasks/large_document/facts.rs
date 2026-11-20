//! The fact checker sees the exact sections, figures and references it checks.
use super::{
    preparation::{
        CHAPTERS, DOCUMENT, DOCUMENT_OWNER, DRAFTING, MINOR_1, MINOR_2, Request, ResultFor,
        WRITER_A_FILE, artifact,
    },
    publication::{
        AuthorOutcome, PartIClosed, RECONSIDER_1, RECONSIDER_2, UpdatedPartI, part_i_revision,
    },
    review::{REVIEW_FILE, ReviewInput, ReviewRequest},
    writers::{material, operation},
};
use crate::{
    imp::json::{decode, encode, same},
    shell::{
        GUEST_MEMORY_BYTES, ShellError, ShellResult,
        global::{Exit, FileExit, Loan},
        imp::service::ReceivedEffects,
    },
};
use hibana::{
    Endpoint,
    g::{self, Message},
    runtime::program::Projectable,
};
use hibana_wasip1_runtime::{
    ChoreoFsFile, ChoreoFsFiles, GuestMemory, HibanaWasiGuestStorage, choreofs::ChoreoFsObject,
};

pub const FACT_PART_I: u8 = 24;
pub const FACT_3: u8 = 25;
pub const FACT_4: u8 = 26;
pub const FACT_EDITORIAL: u8 = 40;
pub const FACT_FILE: u8 = 27;
pub const FACT_DATA: u8 = 28;
pub const FACT_VM: u8 = 29;
pub const FACT_IO: u8 = 30;

pub const CHECK_PART_I: u8 = 23;
pub const CHECK_3: u8 = 24;
pub const CHECK_4: u8 = 25;
pub const CHAPTER_3_OUTCOME: u8 = 49;
pub const CHAPTER_4_OUTCOME: u8 = 50;
pub const MATERIAL_PART_I: u8 = 26;
pub const MATERIAL_3: u8 = 27;
pub const MATERIAL_4: u8 = 28;
pub const FACT_SCOPES: [&str; 3] = ["CHAPTERS_1_2", "CHAPTER_3", "CHAPTER_4"];

pub type FactMaterial<'a, const LABEL: u8> = g::Msg<LABEL, &'a [u8]>;
pub type FactInput<'a> = g::Msg<42, &'a [u8]>;
pub type ApprovedChapter<'a, const OP: u8> = g::Msg<OP, &'a [u8]>;

pub const FACT_FILES: [ChoreoFsObject<'static>; 4] = [
    material(b"service/inbox", 4),
    operation(b"service/fact_check/CHAPTERS_1_2", 5),
    operation(b"service/fact_check/CHAPTER_3", 6),
    operation(b"service/fact_check/CHAPTER_4", 7),
];

fn fact_request<const FACT: u8, const OP: u8>() -> impl Projectable {
    g::seq(
        g::send::<FACT, FACT_FILE, FactInput>(),
        g::send::<FACT_FILE, FACT, Request<OP>>(),
    )
}

pub(super) fn chapter_flow<
    const FACT: u8,
    const FILE: u8,
    const OP: u8,
    const REVISION: u8,
    const MINOR: u8,
    const OUTCOME: u8,
>() -> impl Projectable {
    let assessment = || {
        g::seq(
            fact_request::<DOCUMENT, OP>(),
            g::route(
                g::seq(
                    g::send::<FACT, FACT_FILE, ResultFor<OP>>(),
                    g::seq(
                        g::send::<DOCUMENT, REVIEW_FILE, ReviewInput>(),
                        g::send::<REVIEW_FILE, DOCUMENT, ReviewRequest<OP>>(),
                    ),
                ),
                g::seq(
                    g::send::<DOCUMENT, FACT_FILE, ResultFor<OP>>(),
                    g::send::<DOCUMENT, REVIEW_FILE, ReviewInput>(),
                ),
            ),
        )
    };
    let revision = g::seq(
        g::seq(
            g::send::<FACT, FILE, AuthorOutcome<OUTCOME>>(),
            g::par(
                g::send::<DOCUMENT, FACT_FILE, FactInput>(),
                g::send::<DOCUMENT, REVIEW_FILE, ReviewInput>(),
            ),
        ),
        g::seq(
            g::send::<FILE, DOCUMENT, Request<REVISION>>(),
            g::send::<DOCUMENT, FILE, ResultFor<REVISION>>(),
        ),
    );
    let amendment = g::seq(
        g::send::<DOCUMENT, FILE, AuthorOutcome<OUTCOME>>(),
        g::seq(
            g::par(
                g::send::<DOCUMENT, FACT_FILE, FactInput>(),
                g::send::<DOCUMENT, REVIEW_FILE, ReviewInput>(),
            ),
            g::seq(
                g::send::<FILE, DOCUMENT, Request<MINOR>>(),
                g::seq(
                    g::send::<DOCUMENT, FILE, ResultFor<MINOR>>(),
                    g::seq(
                        fact_request::<DOCUMENT, OP>(),
                        g::send::<FACT, FACT_FILE, ResultFor<OP>>(),
                    ),
                ),
            ),
        ),
    );
    let approved = || {
        g::seq(
            g::send::<DOCUMENT, DRAFTING, ApprovedChapter<OP>>(),
            g::seq(
                g::par(
                    g::send::<DRAFTING, FILE, AuthorOutcome<OUTCOME>>(),
                    g::send::<FACT, DRAFTING, Loan<DOCUMENT>>(),
                ),
                g::par(
                    g::par(
                        g::send::<DOCUMENT, FACT_FILE, FactInput>(),
                        g::send::<DOCUMENT, REVIEW_FILE, ReviewInput>(),
                    ),
                    g::send::<FACT, DOCUMENT_OWNER, ResultFor<REVISION>>(),
                ),
            ),
        )
    };
    g::seq(
        g::send::<DRAFTING, FACT, Loan<DOCUMENT>>(),
        g::seq(
            assessment(),
            g::route(
                approved(),
                g::seq(
                    g::route(g::seq(revision, assessment()), amendment).roll(),
                    approved(),
                ),
            ),
        ),
    )
}

#[hibana_runtime::scope]
pub(super) async fn check_chapter<
    const FACT: u8,
    const OP: u8,
    const MATERIAL: u8,
    const REVISION: u8,
    const MINOR: u8,
    const OUTCOME: u8,
>(
    facts: &mut Endpoint<FACT>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    let (mut snapshot, references) = if OP == CHECK_4 {
        async {
            let [mut draft, mut references] = facts.par()?;
            async { decode(draft.recv::<FactMaterial<OP>>().await?) };
            async { decode(references.recv::<FactMaterial<MATERIAL>>().await?) };
        }
        .await?
    } else {
        let draft = decode(facts.recv::<FactMaterial<OP>>().await?)?;
        let references = decode(facts.recv::<FactMaterial<MATERIAL>>().await?)?;
        (draft, references)
    };
    snapshot["references"] = references;
    let mut document = facts.recv::<Loan<DOCUMENT>>().await?;
    let index = 2 + usize::from(OP - CHECK_3);
    let chapter = CHAPTERS[index];
    loop {
        document
            .send::<FactInput>(&encode(&snapshot)?.as_slice())
            .await?;
        let request = document.recv::<Request<OP>>().await?;
        let checked = effects.apply(request).await?;
        let data = &checked["data"];
        if checked["tool_name"] != "fact_check"
            || data["agent"] != "FACT_CHECKER"
            || data["section"] != chapter
            || data["check_id"].as_str().is_none_or(str::is_empty)
            || !checked["success"].is_boolean()
        {
            return Err(ShellError::message(
                "fact check returned a different chapter",
            ));
        }
        same(&data["input_snapshot"], &snapshot)?;
        let checked_bytes = encode(&checked)?;
        let result = if checked["success"] == true {
            facts
                .send::<ResultFor<OP>>(&checked_bytes.as_slice())
                .await?;
            document
                .send::<ReviewInput>(&checked_bytes.as_slice())
                .await?;
            let request = document.recv::<ReviewRequest<OP>>().await?;
            let feedback = decode(request.as_bytes())?.get("feedback").cloned();
            let result = effects.apply(request).await?;
            let data = &result["data"];
            if result["tool_name"] != "review_sections"
                || data["agent"] != "REVIEWER"
                || data["scope"] != chapter
                || data.get("feedback") != feedback.as_ref()
                || data["review_id"].as_str().is_none_or(str::is_empty)
                || !matches!(
                    (data["status"].as_str(), result["success"].as_bool()),
                    (Some("reviewed"), Some(true))
                        | (Some("needs_revision" | "minor_revision"), Some(false))
                )
            {
                return Err(ShellError::message(
                    "review returned an unsupported chapter decision",
                ));
            }
            same(&data["input_snapshot"], &snapshot)?;
            same(
                &data["fact_receipts"],
                &serde_json::json!({(chapter):checked["data"]["check_id"]}),
            )?;
            result
        } else {
            document
                .send::<ResultFor<OP>>(&checked_bytes.as_slice())
                .await?;
            document
                .send::<ReviewInput>(&checked_bytes.as_slice())
                .await?;
            checked
        };
        let (result, amended) = if result["data"]["status"] == "minor_revision" {
            let outcome = encode(&result)?;
            document
                .send::<AuthorOutcome<OUTCOME>>(&outcome.as_slice())
                .await?;
            async {
                let [mut checker, mut reviewer] = document.par()?;
                async { checker.send::<FactInput>(&outcome.as_slice()).await };
                async { reviewer.send::<ReviewInput>(&outcome.as_slice()).await };
                Ok(())
            }
            .await?;
            let review = &result["data"];
            let replacements = review["minor_replacements"]
                .as_object()
                .ok_or_else(|| ShellError::message("missing minor amendment"))?;
            let replacement = replacements
                .get(chapter)
                .filter(|text| replacements.len() == 1 && text.is_string())
                .ok_or_else(|| {
                    ShellError::message("minor amendment exceeds the reviewed chapter")
                })?;
            let args = document.recv::<Request<MINOR>>().await?;
            let prior = &snapshot["sections"][chapter];
            same(&decode(args.as_bytes())?["review_id"], &review["review_id"])?;
            let revised = effects.apply(args).await?;
            let current = artifact(&revised, "revise_section", index)?;
            let mut expected = prior.clone();
            expected["id"] = current["id"].clone();
            expected["content"] = replacement.clone();
            expected["previous_id"] = prior["id"].clone();
            expected["minor_review_id"] = review["review_id"].clone();
            same(&current, &expected)?;
            if current["id"] == prior["id"] {
                return Err(ShellError::message(
                    "minor amendment did not create a new version",
                ));
            }
            snapshot["sections"][chapter] = current;
            document
                .send::<ResultFor<MINOR>>(&encode(&revised)?.as_slice())
                .await?;
            document
                .send::<FactInput>(&encode(&snapshot)?.as_slice())
                .await?;
            let args = document.recv::<Request<OP>>().await?;
            let checked = effects.apply(args).await?;
            let data = &checked["data"];
            if checked["tool_name"] != "fact_check"
                || data["agent"] != "FACT_CHECKER"
                || data["section"] != chapter
                || data["check_id"].as_str().is_none_or(str::is_empty)
                || !checked["success"].is_boolean()
            {
                return Err(ShellError::message(
                    "minor amendment check differs from its chapter",
                ));
            }
            same(&data["input_snapshot"], &snapshot)?;
            facts
                .send::<ResultFor<OP>>(&encode(&checked)?.as_slice())
                .await?;
            (checked, Some(result))
        } else {
            (result, None)
        };
        let outcome = encode(&result)?;
        if result["success"] == true {
            document
                .send::<ApprovedChapter<OP>>(&outcome.as_slice())
                .await?;
            let receipt = match amended {
                Some(review) => {
                    encode(&serde_json::json!({"review":review,"checked_minor":result}))?
                }
                None => outcome,
            };
            facts.send::<Loan<DOCUMENT>>(document).await?;
            facts
                .send::<ResultFor<REVISION>>(&receipt.as_slice())
                .await?;
            return Ok(());
        }
        facts
            .send::<AuthorOutcome<OUTCOME>>(&outcome.as_slice())
            .await?;
        async {
            let [mut checker, mut reviewer] = document.par()?;
            async { checker.send::<FactInput>(&outcome.as_slice()).await };
            async { reviewer.send::<ReviewInput>(&outcome.as_slice()).await };
            Ok(())
        }
        .await?;
        let args = document.recv::<Request<REVISION>>().await?;
        let revised = effects.apply(args).await?;
        let current = artifact(&revised, "revise_section", index)?;
        if current["content"] == snapshot["sections"][chapter]["content"] {
            return Err(ShellError::message(
                "chapter revision did not change the manuscript",
            ));
        }
        snapshot["sections"][chapter] = current;
        document
            .send::<ResultFor<REVISION>>(&encode(&revised)?.as_slice())
            .await?;
    }
}

pub fn part_i_fact_flow<const FACT: u8>() -> impl Projectable {
    let assessment = || {
        g::seq(
            fact_request::<FACT, CHECK_PART_I>(),
            g::route(
                g::seq(
                    g::send::<FACT, FACT_FILE, ResultFor<CHECK_PART_I>>(),
                    g::seq(
                        g::send::<FACT, REVIEW_FILE, ReviewInput>(),
                        g::send::<REVIEW_FILE, FACT, ReviewRequest<CHECK_PART_I>>(),
                    ),
                ),
                g::seq(
                    g::send::<DOCUMENT, FACT_FILE, ResultFor<CHECK_PART_I>>(),
                    g::send::<FACT, REVIEW_FILE, ReviewInput>(),
                ),
            ),
        )
    };
    let first_edit = g::send::<WRITER_A_FILE, DOCUMENT, Request<MINOR_1>>();
    let first_applied = g::send::<DOCUMENT, WRITER_A_FILE, ResultFor<MINOR_1>>();
    let second_edit = g::send::<WRITER_A_FILE, DOCUMENT, Request<MINOR_2>>();
    let second_applied = g::send::<DOCUMENT, WRITER_A_FILE, ResultFor<MINOR_2>>();
    let minor_edits = g::route(
        g::seq(
            first_edit,
            g::route(
                g::send::<FACT, WRITER_A_FILE, ResultFor<MINOR_1>>(),
                g::seq(first_applied, g::seq(second_edit, second_applied)),
            ),
        ),
        g::seq(
            second_edit,
            g::route(
                g::send::<FACT, WRITER_A_FILE, ResultFor<MINOR_2>>(),
                g::seq(second_applied, g::seq(first_edit, first_applied)),
            ),
        ),
    );
    let amendment = g::seq(
        g::send::<FACT, FACT_FILE, FactInput>(),
        g::seq(
            g::par(
                g::send::<DOCUMENT, WRITER_A_FILE, AuthorOutcome>(),
                g::send::<FACT, REVIEW_FILE, ReviewInput>(),
            ),
            g::seq(
                minor_edits,
                g::seq(
                    fact_request::<FACT, CHECK_PART_I>(),
                    g::send::<FACT, FACT_FILE, ResultFor<CHECK_PART_I>>(),
                ),
            ),
        ),
    );
    let approved = || {
        g::seq(
            g::send::<DOCUMENT, FACT_FILE, FactInput>(),
            g::seq(
                g::par(
                    g::send::<DOCUMENT, WRITER_A_FILE, AuthorOutcome>(),
                    g::send::<FACT, REVIEW_FILE, ReviewInput>(),
                ),
                g::seq(
                    g::send::<FACT, DOCUMENT_OWNER, Loan<DOCUMENT>>(),
                    g::send::<FACT, DOCUMENT_OWNER, ResultFor<MATERIAL_PART_I>>(),
                ),
            ),
        )
    };
    let rejected = g::seq(
        g::seq(
            g::send::<FACT, WRITER_A_FILE, AuthorOutcome>(),
            g::par(
                g::send::<FACT, FACT_FILE, FactInput>(),
                g::send::<FACT, REVIEW_FILE, ReviewInput>(),
            ),
        ),
        part_i_revision(),
    );
    g::seq(
        g::send::<DOCUMENT_OWNER, FACT, Loan<DOCUMENT>>(),
        g::seq(
            assessment(),
            g::route(
                approved(),
                g::seq(
                    g::route(g::seq(rejected, assessment()), amendment).roll(),
                    approved(),
                ),
            ),
        ),
    )
}

#[hibana_runtime::scope]
pub async fn check_and_review<const FACT: u8>(
    facts: &mut Endpoint<FACT>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    let scope = FACT_SCOPES[0];
    loop {
        let mut snapshot = if FACT == FACT_PART_I {
            let (mut draft, references) = async {
                let [mut references, mut draft] = facts.par()?;
                async { decode(draft.recv::<FactMaterial<CHECK_PART_I>>().await?) };
                async { decode(references.recv::<FactMaterial<MATERIAL_PART_I>>().await?) };
            }
            .await?;
            draft["references"] = references;
            draft
        } else {
            let branch = facts.offer().await?;
            match branch.label() {
                UpdatedPartI::LOGICAL_LABEL => {
                    let mut update = decode(branch.recv::<UpdatedPartI>().await?)?;
                    let editorial = encode(&update["editorial"])?;
                    async {
                        let [mut checker, mut reviewer] = facts.par()?;
                        async { checker.send::<FactInput>(&editorial.as_slice()).await };
                        async { reviewer.send::<ReviewInput>(&editorial.as_slice()).await };
                        Ok(())
                    }
                    .await?;
                    update["material"].take()
                }
                PartIClosed::LOGICAL_LABEL => {
                    branch.recv::<PartIClosed>().await?;
                    return Ok(());
                }
                _ => return Err(ShellError::message("unexpected Part I outcome")),
            }
        };
        let mut held = facts.recv::<Loan<DOCUMENT>>().await?;
        let scope_receipt = loop {
            let bytes = encode(&snapshot)?;
            facts.send::<FactInput>(&bytes.as_slice()).await?;
            let arguments = facts.recv::<Request<CHECK_PART_I>>().await?;
            let checked = effects.apply(arguments).await?;
            let data = &checked["data"];
            if checked["tool_name"] != "fact_check"
                || data["agent"] != "FACT_CHECKER"
                || data["section"] != scope
                || data["check_id"].as_str().is_none_or(str::is_empty)
                || !checked["success"].is_boolean()
            {
                return Err(ShellError::message("fact check returned a different scope"));
            }
            same(&data["input_snapshot"], &snapshot)?;
            let receipt = encode(&checked)?;
            let verdict = if checked["success"] == true {
                facts
                    .send::<ResultFor<CHECK_PART_I>>(&receipt.as_slice())
                    .await?;
                facts.send::<ReviewInput>(&receipt.as_slice()).await?;
                let arguments = facts.recv::<ReviewRequest<CHECK_PART_I>>().await?;
                let values = decode(arguments.as_bytes())?;
                let decision = &values["decision"];
                let feedback = values.get("feedback");
                let review = effects.apply(arguments).await?;
                let result = &review["data"];
                let status = result["status"]
                    .as_str()
                    .ok_or_else(|| ShellError::message("missing review status"))?;
                let recorded = result["review_id"]
                    .as_str()
                    .is_some_and(|id| !id.is_empty());
                let correct_outcome = match status {
                    "reviewed" => {
                        recorded
                            && (decision.is_null() || decision == "accept")
                            && review["success"] == true
                    }
                    "needs_revision" => recorded && review["success"] == false,
                    "minor_revision" => {
                        recorded
                            && decision == "minor"
                            && review["success"] == false
                            && result["minor_replacements"]
                                .as_object()
                                .is_some_and(|edits| !edits.is_empty())
                    }
                    "invalid_review" => !recorded && review["success"] == false,
                    _ => false,
                };
                if review["tool_name"] != "review_sections"
                    || result["agent"] != "REVIEWER"
                    || result["scope"] != scope
                    || (recorded && result.get("feedback") != feedback)
                    || !correct_outcome
                {
                    return Err(ShellError::message(
                        "review returned a different scope or decision",
                    ));
                }
                same(&result["input_snapshot"], &snapshot)?;
                same(
                    &result["fact_receipts"],
                    &serde_json::json!({(scope):data["check_id"]}),
                )?;
                review
            } else {
                held.send::<ResultFor<CHECK_PART_I>>(&receipt.as_slice())
                    .await?;
                facts.send::<ReviewInput>(&receipt.as_slice()).await?;
                checked
            };
            let (verdict, reviewed_minor) = if verdict["data"]["status"] == "minor_revision" {
                let outcome = encode(&verdict)?;
                facts.send::<FactInput>(&outcome.as_slice()).await?;
                async {
                    async { held.send::<AuthorOutcome>(&outcome.as_slice()).await };
                    async { facts.send::<ReviewInput>(&outcome.as_slice()).await };
                    Ok(())
                }
                .await?;
                let review_data = &verdict["data"];
                let edits = review_data["minor_replacements"]
                    .as_object()
                    .ok_or_else(|| ShellError::message("missing minor amendments"))?;
                if edits.is_empty()
                    || edits.iter().any(|(chapter, text)| {
                        !CHAPTERS[..2].contains(&chapter.as_str()) || !text.is_string()
                    })
                {
                    return Err(ShellError::message(
                        "minor amendments exceed the reviewed scope",
                    ));
                }
                for _ in edits {
                    let branch = held.offer().await?;
                    let label = branch.label();
                    let index = match label {
                        MINOR_1 => 0,
                        MINOR_2 => 1,
                        _ => return Err(ShellError::message("unexpected minor amendment")),
                    };
                    let chapter = CHAPTERS[index];
                    let prior = &review_data["input_snapshot"]["sections"][chapter];
                    let replacement = edits.get(chapter).ok_or_else(|| {
                        ShellError::message("section has no authorized minor edit")
                    })?;
                    same(&snapshot["sections"][chapter], prior)?;
                    let revised = match label {
                        MINOR_1 => {
                            let request = branch.recv::<Request<MINOR_1>>().await?;
                            same(
                                &decode(request.as_bytes())?["review_id"],
                                &review_data["review_id"],
                            )?;
                            effects.apply(request).await?
                        }
                        MINOR_2 => {
                            let request = branch.recv::<Request<MINOR_2>>().await?;
                            same(
                                &decode(request.as_bytes())?["review_id"],
                                &review_data["review_id"],
                            )?;
                            effects.apply(request).await?
                        }
                        _ => unreachable!("matched minor amendment"),
                    };
                    let artifact = artifact(&revised, "revise_section", index)?;
                    let mut expected = prior.clone();
                    expected["id"] = artifact["id"].clone();
                    expected["content"] = replacement.clone();
                    expected["previous_id"] = prior["id"].clone();
                    expected["minor_review_id"] = review_data["review_id"].clone();
                    same(&artifact, &expected)?;
                    if artifact["id"] == prior["id"] {
                        return Err(ShellError::message(
                            "minor amendment did not create a new version",
                        ));
                    }
                    snapshot["sections"][chapter] = artifact;
                    let response = encode(&revised)?;
                    match (label, edits.len()) {
                        (MINOR_1, 1) => {
                            facts
                                .send::<ResultFor<MINOR_1>>(&response.as_slice())
                                .await?
                        }
                        (MINOR_2, 1) => {
                            facts
                                .send::<ResultFor<MINOR_2>>(&response.as_slice())
                                .await?
                        }
                        (MINOR_1, _) => {
                            held.send::<ResultFor<MINOR_1>>(&response.as_slice())
                                .await?
                        }
                        (MINOR_2, _) => {
                            held.send::<ResultFor<MINOR_2>>(&response.as_slice())
                                .await?
                        }
                        _ => unreachable!("minor amendment label"),
                    }
                }
                let material = encode(&snapshot)?;
                facts.send::<FactInput>(&material.as_slice()).await?;
                let request = facts.recv::<Request<CHECK_PART_I>>().await?;
                let checked = effects.apply(request).await?;
                let actual = &checked["data"];
                if checked["tool_name"] != "fact_check"
                    || actual["agent"] != "FACT_CHECKER"
                    || actual["section"] != scope
                    || actual["check_id"].as_str().is_none_or(str::is_empty)
                    || !checked["success"].is_boolean()
                {
                    return Err(ShellError::message(
                        "minor amendment fact check differs from its scope",
                    ));
                }
                same(&actual["input_snapshot"], &snapshot)?;
                facts
                    .send::<ResultFor<CHECK_PART_I>>(&encode(&checked)?.as_slice())
                    .await?;
                (checked, Some(verdict))
            } else {
                (verdict, None)
            };
            let outcome = encode(&verdict)?;
            if verdict["success"] != true {
                facts.send::<AuthorOutcome>(&outcome.as_slice()).await?;
                async {
                    let [mut checker, mut reviewer] = facts.par()?;
                    async { checker.send::<FactInput>(&outcome.as_slice()).await };
                    async { reviewer.send::<ReviewInput>(&outcome.as_slice()).await };
                    Ok(())
                }
                .await?;
                let branch = held.offer().await?;
                let label = branch.label();
                let revised = match label {
                    RECONSIDER_1 => {
                        effects
                            .apply(branch.recv::<Request<RECONSIDER_1>>().await?)
                            .await?
                    }
                    RECONSIDER_2 => {
                        effects
                            .apply(branch.recv::<Request<RECONSIDER_2>>().await?)
                            .await?
                    }
                    _ => return Err(ShellError::message("unexpected Part I revision")),
                };
                let index = usize::from(label - RECONSIDER_1);
                let chapter = CHAPTERS[index];
                let artifact = artifact(&revised, "revise_section", index)?;
                let previous = &snapshot["sections"][chapter];
                same(&artifact["input_sections"], &previous["input_sections"])?;
                if artifact["content"] == previous["content"] {
                    return Err(ShellError::message(
                        "Part I revision did not change the manuscript",
                    ));
                }
                snapshot["sections"][chapter] = artifact;
                let response = encode(&revised)?;
                match label {
                    RECONSIDER_1 => {
                        held.send::<ResultFor<RECONSIDER_1>>(&response.as_slice())
                            .await?
                    }
                    RECONSIDER_2 => {
                        held.send::<ResultFor<RECONSIDER_2>>(&response.as_slice())
                            .await?
                    }
                    _ => unreachable!("Part I revision label"),
                }
                continue;
            }
            held.send::<FactInput>(&outcome.as_slice()).await?;
            async {
                async { held.send::<AuthorOutcome>(&outcome.as_slice()).await };
                async { facts.send::<ReviewInput>(&outcome.as_slice()).await };
                Ok(())
            }
            .await?;
            break match reviewed_minor {
                Some(review) => {
                    encode(&serde_json::json!({"review":review,"checked_minor":verdict}))?
                }
                None => outcome,
            };
        };
        facts.send::<Loan<DOCUMENT>>(held).await?;
        facts
            .send::<ResultFor<MATERIAL_PART_I>>(&scope_receipt.as_slice())
            .await?;
        if FACT == FACT_PART_I {
            return Ok(());
        }
    }
}

pub async fn fact_guest(module: &[u8], mut root: Endpoint<'_, FACT_VM>) -> ShellResult<()> {
    let file = root.recv::<Loan<FACT_FILE>>().await?;
    let mut inbox = vec![0; GUEST_MEMORY_BYTES];
    let [
        (mut one, mut reply_one),
        (mut two, mut reply_two),
        (mut three, mut reply_three),
    ] = [(); 3].map(|_| (vec![0; GUEST_MEMORY_BYTES], vec![0; GUEST_MEMORY_BYTES]));
    let file = ChoreoFsFile::new(FACT_FILES[0], file)
        .read_as::<FactInput>(&mut inbox)
        .file(FACT_FILES[1])?
        .write_as::<Request<CHECK_PART_I>>(&mut one)
        .read_as::<ResultFor<CHECK_PART_I>>(&mut reply_one)
        .file(FACT_FILES[2])?
        .write_as::<Request<CHECK_3>>(&mut two)
        .read_as::<ResultFor<CHECK_3>>(&mut reply_two)
        .file(FACT_FILES[3])?
        .write_as::<Request<CHECK_4>>(&mut three)
        .read_as::<ResultFor<CHECK_4>>(&mut reply_three);
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
