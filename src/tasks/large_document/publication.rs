//! Whole-document effects require both the checked material and the DOCUMENT Endpoint.
use super::{
    facts::{FACT_EDITORIAL, FACT_FILE, FactInput, MATERIAL_PART_I, part_i_fact_flow},
    preparation::{
        CHAPTERS, DOCUMENT, DOCUMENT_OWNER, REVISE_3, REVISE_4, Request, ResultFor, WRITER_A_FILE,
        artifact,
    },
    review::{REVIEW_FILE, ReviewInput},
    writers::{material, operation},
};
use crate::{
    imp::json::{decode, encode, same},
    shell::{
        GUEST_MEMORY_BYTES, ShellError, ShellResult,
        global::{Exit, FileExit, Loan},
        imp::{review_reply::ReviewReply, service::ReceivedEffects},
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
use serde_json::json;

pub const EDITOR: u8 = 35;
pub const EDITOR_FILE: u8 = 36;
pub const EDITOR_DATA: u8 = 37;
pub const EDITOR_VM: u8 = 38;
pub const EDITOR_IO: u8 = 39;

pub const EDITORIAL: u8 = 29;
pub const COMBINE: u8 = 30;
pub const RECONSIDER_1: u8 = 37;
pub const RECONSIDER_2: u8 = 38;

pub type PublicationMaterial<'a> = g::Msg<31, &'a [u8]>;
type WithheldDocument<'a> = g::Msg<32, &'a [u8]>;
type EditorialApproval<'a> = g::Msg<33, ReviewReply<'a, true>>;
type EditorialRejection<'a> = g::Msg<34, ReviewReply<'a, false>>;
pub type AuthorOutcome<'a, const LABEL: u8 = 35> = g::Msg<LABEL, &'a [u8]>;
pub type UpdatedPartI<'a> = g::Msg<36, &'a [u8]>;
pub type PartIClosed<'a> = g::Msg<39, &'a [u8]>;

pub const EDITOR_FILES: [ChoreoFsObject<'static>; 3] = [
    material(b"service/material", 4),
    operation(b"service/make_editorial_decision", 5),
    operation(b"service/combine_sections", 6),
];

pub(super) fn part_i_revision() -> impl Projectable {
    g::route(
        g::seq(
            g::send::<WRITER_A_FILE, DOCUMENT, Request<RECONSIDER_1>>(),
            g::send::<DOCUMENT, WRITER_A_FILE, ResultFor<RECONSIDER_1>>(),
        ),
        g::seq(
            g::send::<WRITER_A_FILE, DOCUMENT, Request<RECONSIDER_2>>(),
            g::send::<DOCUMENT, WRITER_A_FILE, ResultFor<RECONSIDER_2>>(),
        ),
    )
}

pub fn publication() -> impl Projectable {
    let decision = || {
        g::seq(
            g::send::<DOCUMENT_OWNER, EDITOR, Loan<DOCUMENT>>(),
            g::seq(
                g::send::<DOCUMENT_OWNER, EDITOR, PublicationMaterial>(),
                g::seq(
                    g::send::<DOCUMENT, EDITOR_FILE, PublicationMaterial>(),
                    g::send::<EDITOR_FILE, DOCUMENT, Request<EDITORIAL>>(),
                ),
            ),
        )
    };
    let approved = || {
        g::seq(
            g::send::<DOCUMENT, DOCUMENT_OWNER, EditorialApproval>(),
            g::seq(
                g::send::<DOCUMENT, EDITOR_FILE, ResultFor<EDITORIAL>>(),
                g::seq(
                    g::send::<EDITOR_FILE, DOCUMENT, Request<COMBINE>>(),
                    g::seq(
                        g::send::<DOCUMENT, EDITOR_FILE, ResultFor<COMBINE>>(),
                        g::seq(
                            g::par(
                                g::send::<EDITOR, WRITER_A_FILE, AuthorOutcome>(),
                                g::par(
                                    g::send::<EDITOR, FACT_EDITORIAL, PartIClosed>(),
                                    g::par(
                                        g::send::<EDITOR, FACT_FILE, FactInput>(),
                                        g::send::<EDITOR, REVIEW_FILE, ReviewInput>(),
                                    ),
                                ),
                            ),
                            g::send::<EDITOR, DOCUMENT_OWNER, Loan<DOCUMENT>>(),
                        ),
                    ),
                ),
            ),
        )
    };
    let repair = g::seq(
        g::send::<DOCUMENT, DOCUMENT_OWNER, EditorialRejection>(),
        g::seq(
            g::send::<DOCUMENT_OWNER, EDITOR_FILE, ResultFor<EDITORIAL>>(),
            g::seq(
                g::send::<EDITOR, DOCUMENT_OWNER, Loan<DOCUMENT>>(),
                g::seq(
                    g::send::<DOCUMENT, WRITER_A_FILE, AuthorOutcome>(),
                    g::seq(
                        part_i_revision(),
                        g::seq(
                            g::send::<DOCUMENT, FACT_EDITORIAL, UpdatedPartI>(),
                            g::seq(
                                g::par(
                                    g::send::<FACT_EDITORIAL, FACT_FILE, FactInput>(),
                                    g::send::<FACT_EDITORIAL, REVIEW_FILE, ReviewInput>(),
                                ),
                                part_i_fact_flow::<FACT_EDITORIAL>(),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );
    g::route(
        g::seq(
            decision(),
            g::route(
                approved(),
                g::seq(g::seq(repair, decision()).roll(), approved()),
            ),
        ),
        g::seq(
            g::send::<DOCUMENT_OWNER, EDITOR, WithheldDocument>(),
            g::par(
                g::send::<EDITOR, EDITOR_FILE, PublicationMaterial>(),
                g::par(
                    g::send::<DOCUMENT_OWNER, WRITER_A_FILE, AuthorOutcome>(),
                    g::par(
                        g::send::<DOCUMENT_OWNER, FACT_EDITORIAL, PartIClosed>(),
                        g::par(
                            g::send::<DOCUMENT_OWNER, FACT_FILE, FactInput>(),
                            g::send::<DOCUMENT_OWNER, REVIEW_FILE, ReviewInput>(),
                        ),
                    ),
                ),
            ),
        ),
    )
}

#[hibana_runtime::scope]
pub async fn document_owner<'run>(
    owner: &mut Endpoint<'run, DOCUMENT_OWNER>,
    document: Endpoint<'run, DOCUMENT>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    owner.send::<Loan<DOCUMENT>>(document).await?;
    let fourth = decode(owner.recv::<ResultFor<REVISE_4>>().await?)?;
    let third = decode(owner.recv::<ResultFor<REVISE_3>>().await?)?;
    let document = owner.recv::<Loan<DOCUMENT>>().await?;
    owner.send::<Loan<DOCUMENT>>(document).await?;
    let mut document = owner.recv::<Loan<DOCUMENT>>().await?;
    let part_i = decode(owner.recv::<ResultFor<MATERIAL_PART_I>>().await?)?;
    let mut scopes = [part_i, third, fourth];
    loop {
        let mut snapshot = json!({"sections":{},"references":{},"figures":{}});
        let mut fact_receipts = serde_json::Map::new();
        let mut review_receipts = serde_json::Map::new();
        let mut checked_minor = serde_json::Map::new();
        let reviews = scopes
            .each_ref()
            .map(|scope| scope.get("review").unwrap_or(scope));
        for (receipt, review) in scopes.iter().zip(reviews) {
            let data = &review["data"];
            let checked = receipt.get("checked_minor");
            let material = checked.map_or(
                &data["input_snapshot"],
                |check| &check["data"]["input_snapshot"],
            );
            for category in ["sections", "references", "figures"] {
                let scoped = material[category]
                    .as_object()
                    .ok_or_else(|| ShellError::message("missing scoped publication material"))?;
                let combined = snapshot[category]
                    .as_object_mut()
                    .expect("publication category");
                for (section, artifact) in scoped {
                    if combined.insert(section.clone(), artifact.clone()).is_some() {
                        return Err(ShellError::message("overlapping review scopes"));
                    }
                }
            }
            let scope = data["scope"]
                .as_str()
                .ok_or_else(|| ShellError::message("missing review scope"))?;
            let fact = checked.map_or(
                &data["fact_receipts"][scope],
                |check| &check["data"]["check_id"],
            );
            if let Some(check) = checked {
                checked_minor.insert(scope.to_owned(), check.clone());
            }
            fact_receipts.insert(scope.to_owned(), fact.clone());
            review_receipts.insert(scope.to_owned(), data["review_id"].clone());
        }
        let evidence = encode(
            &json!({"input_snapshot":snapshot,"fact_receipts":fact_receipts,
            "review_receipts":review_receipts,"reviews":reviews,"checked_minor":checked_minor}),
        )?;
        if reviews.iter().any(|review| {
            review["success"] != true
                && !review["data"]["scope"].as_str().is_some_and(|scope| {
                    checked_minor
                        .get(scope)
                        .is_some_and(|check| check["success"] == true)
                })
        }) {
            owner.send::<WithheldDocument>(&evidence.as_slice()).await?;
            async {
                let [mut author, mut facts, mut checker, mut reviewer] = owner.par()?;
                async { author.send::<AuthorOutcome>(&evidence.as_slice()).await };
                async { facts.send::<PartIClosed>(&evidence.as_slice()).await };
                async { checker.send::<FactInput>(&evidence.as_slice()).await };
                async { reviewer.send::<ReviewInput>(&evidence.as_slice()).await };
                Ok(())
            }
            .await?;
            return Ok(());
        }
        owner.send::<Loan<DOCUMENT>>(document).await?;
        owner
            .send::<PublicationMaterial>(&evidence.as_slice())
            .await?;
        let branch = owner.offer().await?;
        let refused = match branch.label() {
            EditorialApproval::LOGICAL_LABEL => {
                branch.recv::<EditorialApproval>().await?;
                drop(owner.recv::<Loan<DOCUMENT>>().await?);
                return Ok(());
            }
            EditorialRejection::LOGICAL_LABEL => {
                branch.recv::<EditorialRejection>().await?.to_vec()
            }
            _ => return Err(ShellError::message("unexpected editorial outcome")),
        };
        owner
            .send::<ResultFor<EDITORIAL>>(&refused.as_slice())
            .await?;
        document = owner.recv::<Loan<DOCUMENT>>().await?;
        document.send::<AuthorOutcome>(&refused.as_slice()).await?;
        let branch = document.offer().await?;
        let label = branch.label();
        let actual = match label {
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
            _ => {
                return Err(ShellError::message(
                    "unexpected chapter for editorial revision",
                ));
            }
        };
        let index = usize::from(label - RECONSIDER_1);
        let revised = artifact(&actual, "revise_section", index)?;
        if revised["content"] == snapshot["sections"][CHAPTERS[index]]["content"] {
            return Err(ShellError::message(
                "editorial revision did not change the manuscript",
            ));
        }
        same(
            &revised["input_sections"],
            &json!({(CHAPTERS[2]):snapshot["sections"][CHAPTERS[2]],
            (CHAPTERS[3]):snapshot["sections"][CHAPTERS[3]]}),
        )?;
        let bytes = encode(&actual)?;
        match label {
            RECONSIDER_1 => {
                document
                    .send::<ResultFor<RECONSIDER_1>>(&bytes.as_slice())
                    .await?
            }
            RECONSIDER_2 => {
                document
                    .send::<ResultFor<RECONSIDER_2>>(&bytes.as_slice())
                    .await?
            }
            _ => unreachable!("editorial revision label"),
        }
        let mut updated = scopes[0]
            .get("checked_minor")
            .map_or(&reviews[0]["data"]["input_snapshot"], |checked| {
                &checked["data"]["input_snapshot"]
            })
            .clone();
        updated["sections"][CHAPTERS[index]] = revised;
        let editorial = decode(&refused)?;
        let update = encode(&json!({"material":updated,"editorial":editorial}))?;
        document.send::<UpdatedPartI>(&update.as_slice()).await?;
        owner.send::<Loan<DOCUMENT>>(document).await?;
        document = owner.recv::<Loan<DOCUMENT>>().await?;
        scopes[0] = decode(owner.recv::<ResultFor<MATERIAL_PART_I>>().await?)?;
    }
}

#[hibana_runtime::scope]
pub async fn edit_document(
    editor: &mut Endpoint<'_, EDITOR>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    let branch = editor.offer().await?;
    let mut document = match branch.label() {
        WithheldDocument::LOGICAL_LABEL => {
            let evidence = branch.recv::<WithheldDocument>().await?.to_vec();
            editor
                .send::<PublicationMaterial>(&evidence.as_slice())
                .await?;
            return Ok(());
        }
        Loan::<DOCUMENT>::LOGICAL_LABEL => branch.recv::<Loan<DOCUMENT>>().await?,
        _ => return Err(ShellError::message("unexpected document handoff")),
    };
    loop {
        let evidence = editor.recv::<PublicationMaterial>().await?.to_vec();
        let material = decode(&evidence)?;
        document
            .send::<PublicationMaterial>(&evidence.as_slice())
            .await?;
        let arguments = document.recv::<Request<EDITORIAL>>().await?;
        let summary = decode(arguments.as_bytes())?["review_summary"].take();
        let decision = effects.apply(arguments).await?;
        let data = &decision["data"];
        if decision["tool_name"] != "make_editorial_decision"
            || data["agent"] != "EDITOR_IN_CHIEF"
            || data["decision_type"] != "final"
            || data["review_summary"] != summary
            || data["decision_id"].as_str().is_none_or(str::is_empty)
            || !decision["success"].is_boolean()
        {
            return Err(ShellError::message(
                "editorial decision differs from its request",
            ));
        }
        same(&data["input_snapshot"], &material["input_snapshot"])?;
        same(&data["review_receipts"], &material["review_receipts"])?;
        let receipt = encode(&decision)?;
        if decision["success"] == true {
            document
                .send::<EditorialApproval>(&ReviewReply::from_bytes(&receipt)?)
                .await?;
            document
                .send::<ResultFor<EDITORIAL>>(&receipt.as_slice())
                .await?;
            let arguments = document.recv::<Request<COMBINE>>().await?;
            let output = effects.apply(arguments).await?;
            let result = &output["data"];
            if output["tool_name"] != "combine_sections"
                || output["success"] != true
                || result["agent"] != "EDITOR_IN_CHIEF"
                || result["status"] != "combined"
                || result["artifact_id"].as_str().is_none_or(str::is_empty)
            {
                return Err(ShellError::message("publication failed"));
            }
            for field in ["input_snapshot", "review_receipts", "fact_receipts"] {
                same(&result[field], &material[field])?;
            }
            same(
                &result["editorial_receipts"],
                &json!({"final":data["decision_id"]}),
            )?;
            let sections = result["sections"]
                .as_array()
                .ok_or_else(|| ShellError::message("missing combined sections"))?;
            if sections.len() != CHAPTERS.len()
                || !CHAPTERS
                    .iter()
                    .all(|name| sections.iter().any(|section| section == name))
            {
                return Err(ShellError::message(
                    "publication returned different sections",
                ));
            }
            let published = encode(&output)?;
            document
                .send::<ResultFor<COMBINE>>(&published.as_slice())
                .await?;
            async {
                let [mut author, mut facts, mut checker, mut reviewer] = editor.par()?;
                async { author.send::<AuthorOutcome>(&published.as_slice()).await };
                async { facts.send::<PartIClosed>(&published.as_slice()).await };
                async { checker.send::<FactInput>(&published.as_slice()).await };
                async { reviewer.send::<ReviewInput>(&published.as_slice()).await };
                Ok(())
            }
            .await?;
            editor.send::<Loan<DOCUMENT>>(document).await?;
            return Ok(());
        } else {
            document
                .send::<EditorialRejection>(&ReviewReply::from_bytes(&receipt)?)
                .await?;
        }
        editor.send::<Loan<DOCUMENT>>(document).await?;
        document = editor.recv::<Loan<DOCUMENT>>().await?;
    }
}

pub async fn editor_guest(module: &[u8], mut root: Endpoint<'_, EDITOR_VM>) -> ShellResult<()> {
    let file = root.recv::<Loan<EDITOR_FILE>>().await?;
    let [
        mut material,
        mut decision,
        mut decision_reply,
        mut combine,
        mut combine_reply,
    ] = [(); 5].map(|_| vec![0; GUEST_MEMORY_BYTES]);
    let file = ChoreoFsFile::new(EDITOR_FILES[0], file)
        .read_as::<PublicationMaterial>(&mut material)
        .file(EDITOR_FILES[1])?
        .write_as::<Request<EDITORIAL>>(&mut decision)
        .read_as::<ResultFor<EDITORIAL>>(&mut decision_reply)
        .file(EDITOR_FILES[2])?
        .write_as::<Request<COMBINE>>(&mut combine)
        .read_as::<ResultFor<COMBINE>>(&mut combine_reply);
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
