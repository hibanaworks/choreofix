//! Each physical owner executes only its own admitted effect and passes on its result.
use super::{
    global::*,
    publication::SECTIONS,
    quality::{AUTHORS, ChapterBytes, TOPICS},
};
use crate::{
    imp::json::{decode, encode, same},
    shell::{ShellError, ShellResult, imp::service::ReceivedEffects},
};
use hibana::{Endpoint, EndpointBorrow};
use serde_json::json;

pub async fn references<'r>(
    database: &mut Endpoint<'r, DATABASE>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    for _ in SECTIONS {
        let branch = database.offer().await?;
        let chapter = branch.label();
        let (research, result) = match chapter {
            SECTION_A => {
                let received = branch.recv::<References<SECTION_A>>().await?;
                let effect = effects.apply(received);
                let research = decode(database.recv::<ReferenceSource<SECTION_A>>().await?)?;
                (research, effect.await?)
            }
            SECTION_B => {
                let received = branch.recv::<References<SECTION_B>>().await?;
                let effect = effects.apply(received);
                let research = decode(database.recv::<ReferenceSource<SECTION_B>>().await?)?;
                (research, effect.await?)
            }
            SECTION_C => {
                let received = branch.recv::<References<SECTION_C>>().await?;
                let effect = effects.apply(received);
                let research = decode(database.recv::<ReferenceSource<SECTION_C>>().await?)?;
                (research, effect.await?)
            }
            SECTION_D => {
                let received = branch.recv::<References<SECTION_D>>().await?;
                let effect = effects.apply(received);
                let research = decode(database.recv::<ReferenceSource<SECTION_D>>().await?)?;
                (research, effect.await?)
            }
            _ => return Err(ShellError::message("unexpected survey database request")),
        };
        let index = usize::from(chapter) - 1;
        let artifact = &result["data"]["artifact"];
        if result["tool_name"] != "update_references"
            || result["success"] != true
            || result["data"]["agent"] != AUTHORS[index]
            || result["data"]["section"] != SECTIONS[index]
            || artifact["id"].as_str().is_none_or(str::is_empty)
            || !artifact["content"].is_string()
        {
            return Err(ShellError::message(
                "bibliography differs from its actual request",
            ));
        }
        same(
            &artifact["research_inputs"],
            &json!({TOPICS[index]:research["data"]["artifact"]}),
        )?;
        let bytes = encode(&result)?;
        match chapter {
            SECTION_A => {
                database
                    .send::<Referenced<SECTION_A>>(&ChapterBytes(&bytes))
                    .await?;
                database
                    .send::<ReferencesForChapter<SECTION_A>>(&ChapterBytes(&bytes))
                    .await?;
            }
            SECTION_B => {
                database
                    .send::<Referenced<SECTION_B>>(&ChapterBytes(&bytes))
                    .await?;
                database
                    .send::<ReferencesForChapter<SECTION_B>>(&ChapterBytes(&bytes))
                    .await?;
            }
            SECTION_C => {
                database
                    .send::<Referenced<SECTION_C>>(&ChapterBytes(&bytes))
                    .await?;
                database
                    .send::<ReferencesForChapter<SECTION_C>>(&ChapterBytes(&bytes))
                    .await?;
            }
            SECTION_D => {
                database
                    .send::<Referenced<SECTION_D>>(&ChapterBytes(&bytes))
                    .await?;
                database
                    .send::<ReferencesForChapter<SECTION_D>>(&ChapterBytes(&bytes))
                    .await?;
            }
            _ => unreachable!("chapter request determined the matching reply"),
        }
    }
    Ok(())
}

pub async fn figures<'r>(
    figures: &mut Endpoint<'r, FIGURES>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    for _ in SECTIONS {
        let branch = figures.offer().await?;
        let chapter = branch.label();
        let (args, research, result) = match chapter {
            SECTION_A => {
                let received = branch.recv::<CreateFigure<SECTION_A>>().await?;
                let args = decode(received.as_bytes())?;
                let effect = effects.apply(received);
                let research = decode(figures.recv::<FigureSource<SECTION_A>>().await?)?;
                (args, research, effect.await?)
            }
            SECTION_B => {
                let received = branch.recv::<CreateFigure<SECTION_B>>().await?;
                let args = decode(received.as_bytes())?;
                let effect = effects.apply(received);
                let research = decode(figures.recv::<FigureSource<SECTION_B>>().await?)?;
                (args, research, effect.await?)
            }
            SECTION_C => {
                let received = branch.recv::<CreateFigure<SECTION_C>>().await?;
                let args = decode(received.as_bytes())?;
                let effect = effects.apply(received);
                let research = decode(figures.recv::<FigureSource<SECTION_C>>().await?)?;
                (args, research, effect.await?)
            }
            SECTION_D => {
                let received = branch.recv::<CreateFigure<SECTION_D>>().await?;
                let args = decode(received.as_bytes())?;
                let effect = effects.apply(received);
                let research = decode(figures.recv::<FigureSource<SECTION_D>>().await?)?;
                (args, research, effect.await?)
            }
            _ => return Err(ShellError::message("unexpected survey figure request")),
        };
        let index = usize::from(chapter) - 1;
        let artifact = &result["data"]["artifact"];
        if result["tool_name"] != "create_figure"
            || result["success"] != true
            || result["data"]["agent"] != "DATA_ANALYST"
            || artifact["id"].as_str().is_none_or(str::is_empty)
            || result["data"]["figure_id"] != artifact["id"]
        {
            return Err(ShellError::message(
                "figure differs from its actual request",
            ));
        }
        same(&artifact["content"], &args["description"])?;
        same(
            &artifact["research_inputs"][SECTIONS[index]],
            &json!({TOPICS[index]:research["data"]["artifact"]}),
        )?;
        let bytes = encode(&result)?;
        match chapter {
            SECTION_A => {
                figures
                    .send::<FigureCreated<SECTION_A>>(&ChapterBytes(&bytes))
                    .await?;
                figures
                    .send::<FigureForChapter<SECTION_A>>(&ChapterBytes(&bytes))
                    .await?;
            }
            SECTION_B => {
                figures
                    .send::<FigureCreated<SECTION_B>>(&ChapterBytes(&bytes))
                    .await?;
                figures
                    .send::<FigureForChapter<SECTION_B>>(&ChapterBytes(&bytes))
                    .await?;
            }
            SECTION_C => {
                figures
                    .send::<FigureCreated<SECTION_C>>(&ChapterBytes(&bytes))
                    .await?;
                figures
                    .send::<FigureForChapter<SECTION_C>>(&ChapterBytes(&bytes))
                    .await?;
            }
            SECTION_D => {
                figures
                    .send::<FigureCreated<SECTION_D>>(&ChapterBytes(&bytes))
                    .await?;
                figures
                    .send::<FigureForChapter<SECTION_D>>(&ChapterBytes(&bytes))
                    .await?;
            }
            _ => unreachable!("figure request determined the matching reply"),
        }
    }
    Ok(())
}

pub async fn insert<const SECTION: u8>(
    analyst: &mut EndpointBorrow<'_, '_, ANALYST>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    let chapter = SECTIONS[usize::from(SECTION) - 1];
    let snapshot = decode(analyst.recv::<Prepared<SECTION>>().await?)?;
    let mut section = analyst.recv::<Accepted<SECTION>>().await?;
    let material = encode(&snapshot)?;
    section
        .send::<InsertionInput<SECTION>>(&ChapterBytes(&material))
        .await?;
    let received = section.recv::<InsertFigure<SECTION>>().await?;
    let args = decode(received.as_bytes())?;
    same(&args["figure_id"], &snapshot["figures"][chapter]["id"])?;
    let result = effects.apply(received).await?;
    if result["tool_name"] != "insert_figure"
        || result["success"] != true
        || result["data"]["agent"] != "DATA_ANALYST"
        || result["data"]["section"] != chapter
    {
        return Err(ShellError::message(
            "figure was not inserted in its actual chapter",
        ));
    }
    same(&result["data"]["artifact"], &snapshot["figures"][chapter])?;
    same(
        &result["data"]["figure_id"],
        &snapshot["figures"][chapter]["id"],
    )?;
    let reply = encode(&result)?;
    section
        .send::<Inserted<SECTION>>(&ChapterBytes(&reply))
        .await?;
    section
        .send::<Draft<SECTION>>(&ChapterBytes(&material))
        .await?;
    analyst.send::<Accepted<SECTION>>(section).await?;
    Ok(())
}
