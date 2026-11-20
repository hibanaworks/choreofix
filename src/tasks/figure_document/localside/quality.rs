//! The actual fact and review receipts gate the document's move to publication.
use super::super::{CHAPTERS, FACT_SCOPES, global::*, sections};
use crate::{
    imp::json::{decode, encode, same},
    shell::{ShellError, ShellResult, global::Loan, imp::service::Domain},
};
use hibana::{Endpoint, g::Message};
use serde_json::{Value, json};

const MATERIAL: [&str; 3] = ["sections", "references", "figures"];

fn approved_scope(actual: &Value, document: &Value, scope: &[&str]) -> bool {
    let snapshot = &actual["data"]["input_snapshot"];
    actual["success"] == true
        && snapshot
            .as_object()
            .is_some_and(|map| map.len() == MATERIAL.len())
        && MATERIAL.into_iter().all(|category| {
            let expected = &document[category];
            let material = &snapshot[category];
            material.as_object().is_some_and(|map| {
                map.len()
                    == scope
                        .iter()
                        .filter(|section| expected.get(**section).is_some())
                        .count()
                    && map.iter().all(|(name, artifact)| {
                        scope.contains(&name.as_str()) && Some(artifact) == expected.get(name)
                    })
            })
        })
}

fn completed_material(snapshot: &Value, chapters: &[&str]) -> bool {
    MATERIAL.into_iter().all(|category| {
        let required = || {
            chapters
                .iter()
                .filter(|&&name| category != "figures" || name != CHAPTERS[1])
        };
        snapshot[category]
            .as_object()
            .is_some_and(|map| map.len() == required().count())
            && required().all(|chapter| {
                let artifact = &snapshot[category][chapter];
                artifact["id"].as_str().is_some_and(|id| !id.is_empty())
                    && artifact["content"].is_string()
            })
    }) && (!chapters.contains(&CHAPTERS[0])
        || !chapters.contains(&CHAPTERS[2])
        || snapshot["figures"][CHAPTERS[0]]["id"] != snapshot["figures"][CHAPTERS[2]]["id"])
}

fn material_for(snapshot: &Value, chapters: &[&str]) -> Value {
    Value::Object(
        MATERIAL
            .into_iter()
            .map(|category| {
                let material = snapshot[category].as_object().expect("completed material");
                let scoped = material
                    .iter()
                    .filter(|(name, _)| chapters.contains(&name.as_str()))
                    .map(|(name, artifact)| (name.clone(), artifact.clone()))
                    .collect();
                (category.to_owned(), Value::Object(scoped))
            })
            .collect(),
    )
}

pub async fn facts<const ROLE: u8, const OP: u8>(
    checker: &mut Endpoint<'_, ROLE>,
    domain: &impl Domain,
) -> ShellResult<Option<Value>> {
    let branch = checker.offer().await?;
    if branch.label() == Published::<ROLE>::LOGICAL_LABEL {
        branch.recv::<Published<ROLE>>().await?;
        return Ok(None);
    }
    let arguments = branch.recv::<QualityRequest<OP>>().await?.value()?;
    let actual = domain
        .execute("FACT_CHECKER", "fact_check", arguments.clone())
        .await
        .map_err(ShellError::message)?;
    let data = &actual["data"];
    if actual["tool_name"] != "fact_check"
        || data["agent"] != "FACT_CHECKER"
        || data["section"] != arguments["section_name"]
        || !actual["success"].is_boolean()
    {
        return Err(ShellError::message("fact check returned a different scope"));
    }
    let bytes = encode(&actual)?;
    checker.send::<QualityResult<OP>>(&bytes.as_slice()).await?;
    checker.send::<QualityResult<OP>>(&bytes.as_slice()).await?;
    Ok(Some(actual))
}

async fn review_request(domain: &impl Domain, arguments: Value) -> ShellResult<Vec<u8>> {
    let actual = domain
        .execute("EDITOR", "review_sections", arguments.clone())
        .await
        .map_err(ShellError::message)?;
    if actual["tool_name"] != "review_sections"
        || actual["data"]["agent"] != "EDITOR"
        || actual["data"]["scope"] != arguments["section_name"]
        || !actual["success"].is_boolean()
    {
        return Err(ShellError::message(
            "editorial result names a different scope",
        ));
    }
    encode(&actual)
}

#[hibana_runtime::scope]
pub async fn editor(
    editor: &mut Endpoint<EDITOR>,
    domain: &impl Domain,
) -> ShellResult<Option<Value>> {
    let branch = editor.offer().await?;
    match branch.label() {
        ReviewMaterial::LOGICAL_LABEL => {
            let material = branch.recv::<ReviewMaterial>().await?.to_vec();
            editor.send::<ReviewMaterial>(&material.as_slice()).await?;
        }
        FactsFailed::LOGICAL_LABEL => {
            branch.recv::<FactsFailed>().await?;
            return Ok(Some(decode(editor.recv::<Reconsider>().await?)?));
        }
        label => {
            return Err(ShellError::message(format!(
                "unexpected editorial input {label}"
            )));
        }
    }
    async {
        let [mut a, mut b, mut c] = editor.par()?;
        async {
            let args = a.recv::<QualityRequest<REVIEW_1>>().await?.value()?;
            let actual = review_request(domain, args).await?;
            a.send::<QualityResult<REVIEW_1>>(&actual.as_slice())
                .await?;
            a.send::<QualityResult<REVIEW_1>>(&actual.as_slice())
                .await?;
        };
        async {
            let args = b.recv::<QualityRequest<REVIEW_2>>().await?.value()?;
            let actual = review_request(domain, args).await?;
            b.send::<QualityResult<REVIEW_2>>(&actual.as_slice())
                .await?;
            b.send::<QualityResult<REVIEW_2>>(&actual.as_slice())
                .await?;
        };
        async {
            let args = c.recv::<QualityRequest<REVIEW_3>>().await?.value()?;
            let actual = review_request(domain, args).await?;
            c.send::<QualityResult<REVIEW_3>>(&actual.as_slice())
                .await?;
            c.send::<QualityResult<REVIEW_3>>(&actual.as_slice())
                .await?;
        };
        Ok(())
    }
    .await?;
    let branch = editor.offer().await?;
    match branch.label() {
        Loan::<DOCUMENT>::LOGICAL_LABEL => {
            let mut document = branch.recv::<Loan<DOCUMENT>>().await?;
            let admitted = editor.recv::<AdmittedCombine>().await?;
            document.send::<CombineOnDocument>(&admitted).await?;
            editor.send::<Loan<DOCUMENT>>(document).await?;
        }
        Reconsider::LOGICAL_LABEL => {
            return Ok(Some(decode(branch.recv::<Reconsider>().await?)?));
        }
        label => {
            return Err(ShellError::message(format!(
                "unexpected publication continuation {label}"
            )));
        }
    }
    Ok(None)
}

#[hibana_runtime::scope]
pub async fn reconsider<'r>(
    owner: &mut Endpoint<'r, DOCUMENT_OWNER>,
    mut document: Endpoint<'r, DOCUMENT>,
    (mut snapshot, domain): (Value, &impl Domain),
) -> ShellResult<Value> {
    if !completed_material(&snapshot, &CHAPTERS) {
        return Err(ShellError::message(
            "document lacks complete sections, references or distinct figures",
        ));
    }
    let initial = json!({
        FACT_SCOPES[0]: material_for(&snapshot, &CHAPTERS[..2]),
        FACT_SCOPES[1]: material_for(&snapshot, &CHAPTERS[2..]),
    });
    owner
        .send::<FactOutcome>(&encode(&initial)?.as_slice())
        .await?;
    let (mut facts12, mut facts3) = async {
        let [mut a, mut b] = owner.par()?;
        async { decode(a.recv::<QualityResult<CHECK_12>>().await?) };
        async { decode(b.recv::<QualityResult<CHECK_3>>().await?) };
    }
    .await?;
    loop {
        let can_review = [&facts12, &facts3]
            .into_iter()
            .zip([
                (FACT_SCOPES[0], &CHAPTERS[..2]),
                (FACT_SCOPES[1], &CHAPTERS[2..]),
            ])
            .all(|(actual, (name, chapters))| {
                actual["tool_name"] == "fact_check"
                    && actual["data"]["agent"] == "FACT_CHECKER"
                    && actual["data"]["section"] == name
                    && actual["data"]["check_id"]
                        .as_str()
                        .is_some_and(|id| !id.is_empty())
                    && approved_scope(actual, &snapshot, chapters)
            });
        let result = if can_review {
            let facts = json!({
                FACT_SCOPES[0]: facts12["data"]["check_id"],
                FACT_SCOPES[1]: facts3["data"]["check_id"],
            });
            if facts[FACT_SCOPES[0]] == facts[FACT_SCOPES[1]] {
                return Err(ShellError::message("fact receipts reuse the same ID"));
            }
            let material = json!({"input_snapshot":snapshot,"fact_receipts":facts});
            owner
                .send::<ReviewMaterial>(&encode(&material)?.as_slice())
                .await?;
            let (first, second, third) = async {
                let [mut a, mut b, mut c] = owner.par()?;
                async { decode(a.recv::<QualityResult<REVIEW_1>>().await?) };
                async { decode(b.recv::<QualityResult<REVIEW_2>>().await?) };
                async { decode(c.recv::<QualityResult<REVIEW_3>>().await?) };
            }
            .await?;
            let mut review_ids = serde_json::Map::new();
            for (chapter, actual) in CHAPTERS.into_iter().zip([&first, &second, &third]) {
                let data = &actual["data"];
                if actual["tool_name"] == "review_sections"
                    && actual["success"] == true
                    && data["agent"] == "EDITOR"
                    && data["scope"] == chapter
                    && data["input_snapshot"] == snapshot
                    && data["fact_receipts"] == facts
                    && data["review_id"].as_str().is_some_and(|id| !id.is_empty())
                    && !review_ids.values().any(|id| id == &data["review_id"])
                {
                    review_ids.insert(chapter.to_owned(), data["review_id"].clone());
                }
            }
            if review_ids.len() == CHAPTERS.len() {
                owner.send::<Loan<DOCUMENT>>(document).await?;
                let admitted = owner.recv::<Combine>().await?;
                owner.send::<AdmittedCombine>(&admitted).await?;
                let arguments = admitted.value()?;
                same(
                    &owner.recv::<CombineOnDocument>().await?.value()?,
                    &arguments,
                )?;
                let published = domain
                    .execute("EDITOR", "combine_sections", arguments.clone())
                    .await
                    .map_err(ShellError::message)?;
                let data = &published["data"];
                if published["tool_name"] != "combine_sections"
                    || published["success"] != true
                    || data["agent"] != "EDITOR"
                {
                    return Err(ShellError::message("missing actual assembled document"));
                }
                let artifact = data["artifact_id"]
                    .as_str()
                    .filter(|id| !id.is_empty())
                    .ok_or_else(|| ShellError::message("missing published artifact ID"))?
                    .as_bytes();
                let order: Vec<_> = sections(
                    arguments["sections"]
                        .as_str()
                        .expect("typed combine arguments"),
                )
                .collect();
                same(&data["sections"], &json!(order))?;
                same(&data["input_snapshot"], &snapshot)?;
                same(&data["fact_receipts"], &facts)?;
                same(&data["review_receipts"], &Value::Object(review_ids))?;
                let published_bytes = encode(&published)?;
                owner.send::<Combined>(&published_bytes.as_slice()).await?;
                async {
                    let [mut file, mut fact12, mut fact3, mut writer_a, mut writer_b] =
                        owner.par()?;
                    async {
                        file.send::<FactOutcome>(&published_bytes.as_slice())
                            .await?;
                    };
                    async {
                        fact12.send::<Published<FACTS_12>>(&artifact).await?;
                    };
                    async {
                        fact3.send::<Published<FACTS_3>>(&artifact).await?;
                    };
                    async {
                        writer_a
                            .send::<WriterOutcome<WRITER_FILE>>(&published_bytes.as_slice())
                            .await?;
                    };
                    async {
                        writer_b
                            .send::<WriterOutcome<WRITER_B_FILE>>(&published_bytes.as_slice())
                            .await?;
                    };
                    Ok(())
                }
                .await?;
                return Ok(published);
            }
            json!({"reviews":[first, second, third]})
        } else {
            owner.send::<FactsFailed>(&()).await?;
            json!({"facts":[&facts12, &facts3]})
        };
        document
            .send::<Reconsider>(&encode(&result)?.as_slice())
            .await?;

        let rejection = ["facts", "reviews"]
            .into_iter()
            .filter_map(|source| result[source].as_array())
            .flatten()
            .find(|receipt| {
                receipt["success"] == false && receipt["data"]["status"] == "needs_revision"
            })
            .ok_or_else(|| {
                ShellError::message("reconsideration lacks an actual revision request")
            })?;
        let data = &rejection["data"];
        let scope = data["scope"]
            .as_str()
            .or_else(|| data["section"].as_str())
            .ok_or_else(|| ShellError::message("revision request lacks its scope"))?;
        let bytes = encode(rejection)?;
        let (writer, arguments) = match scope {
            "CHAPTERS_1_2" | "CHAPTER_1" | "CHAPTER_2" => {
                owner
                    .send::<WriterOutcome<WRITER_FILE>>(&bytes.as_slice())
                    .await?;
                (
                    "WRITER_A",
                    document.recv::<Revise<REVISE_A>>().await?.value()?,
                )
            }
            "CHAPTER_3" => {
                owner
                    .send::<WriterOutcome<WRITER_B_FILE>>(&bytes.as_slice())
                    .await?;
                (
                    "WRITER_B",
                    document.recv::<Revise<REVISE_B>>().await?.value()?,
                )
            }
            _ => {
                return Err(ShellError::message(
                    "revision scope is outside the document",
                ));
            }
        };
        let chapter = arguments["section_name"].as_str().expect("typed revision");
        if chapter != scope && !(scope == "CHAPTERS_1_2" && CHAPTERS[..2].contains(&chapter)) {
            return Err(ShellError::message("revision targets a different scope"));
        }
        let revised = domain
            .execute(writer, "revise_section", arguments.clone())
            .await
            .map_err(ShellError::message)?;
        let data = &revised["data"];
        let written = &data["artifact"];
        if revised["tool_name"] != "revise_section"
            || revised["success"] != true
            || data["section"] != chapter
            || data["agent"] != writer
            || written["id"].as_str().is_none_or(str::is_empty)
            || !written["content"].is_string()
            || written["id"] == snapshot["sections"][chapter]["id"]
        {
            return Err(ShellError::message(
                "revision did not produce the requested chapter version",
            ));
        }
        snapshot["sections"][chapter] = written.clone();
        document
            .send::<Revised>(&encode(&revised)?.as_slice())
            .await?;
        if chapter == CHAPTERS[2] {
            let material = json!({FACT_SCOPES[1]:material_for(&snapshot, &CHAPTERS[2..])});
            owner
                .send::<FactOutcome>(&encode(&material)?.as_slice())
                .await?;
            facts3 = decode(owner.recv::<QualityResult<CHECK_3>>().await?)?;
        } else {
            let material = json!({FACT_SCOPES[0]:material_for(&snapshot, &CHAPTERS[..2])});
            owner
                .send::<FactOutcome>(&encode(&material)?.as_slice())
                .await?;
            facts12 = decode(owner.recv::<QualityResult<CHECK_12>>().await?)?;
        }
    }
}
