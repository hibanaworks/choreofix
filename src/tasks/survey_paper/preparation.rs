//! Each physical owner executes only its own admitted effect and passes on its result.
use super::{
    global::*,
    publication::SECTIONS,
    quality::{AUTHORS, ChapterBytes, TOPICS},
};
use crate::{
    imp::json::{decode, encode, same},
    shell::{ShellError, ShellResult, imp::service::Domain},
};
use hibana::{Endpoint, EndpointBorrow};
use serde_json::json;

pub async fn references<'r>(
    database: &mut Endpoint<'r, DATABASE>,
    domain: &impl Domain,
) -> ShellResult<()> {
    for _ in SECTIONS {
        let branch = database.offer().await?;
        let chapter = branch.label();
        let args = match chapter {
            SECTION_A => branch.recv::<References<SECTION_A>>().await?.into_value(),
            SECTION_B => branch.recv::<References<SECTION_B>>().await?.into_value(),
            SECTION_C => branch.recv::<References<SECTION_C>>().await?.into_value(),
            SECTION_D => branch.recv::<References<SECTION_D>>().await?.into_value(),
            _ => return Err(ShellError::message("unexpected survey database request")),
        };
        let research = match chapter {
            SECTION_A => decode(database.recv::<ReferenceSource<SECTION_A>>().await?)?,
            SECTION_B => decode(database.recv::<ReferenceSource<SECTION_B>>().await?)?,
            SECTION_C => decode(database.recv::<ReferenceSource<SECTION_C>>().await?)?,
            SECTION_D => decode(database.recv::<ReferenceSource<SECTION_D>>().await?)?,
            _ => unreachable!("chapter request determined the matching source"),
        };
        let index = usize::from(chapter) - 1;
        let result = domain
            .execute(AUTHORS[index], "update_references", args)
            .await
            .map_err(ShellError::message)?;
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
    domain: &impl Domain,
) -> ShellResult<()> {
    for _ in SECTIONS {
        let branch = figures.offer().await?;
        let chapter = branch.label();
        let args = match chapter {
            SECTION_A => branch.recv::<CreateFigure<SECTION_A>>().await?.into_value(),
            SECTION_B => branch.recv::<CreateFigure<SECTION_B>>().await?.into_value(),
            SECTION_C => branch.recv::<CreateFigure<SECTION_C>>().await?.into_value(),
            SECTION_D => branch.recv::<CreateFigure<SECTION_D>>().await?.into_value(),
            _ => return Err(ShellError::message("unexpected survey figure request")),
        };
        let research = match chapter {
            SECTION_A => decode(figures.recv::<FigureSource<SECTION_A>>().await?)?,
            SECTION_B => decode(figures.recv::<FigureSource<SECTION_B>>().await?)?,
            SECTION_C => decode(figures.recv::<FigureSource<SECTION_C>>().await?)?,
            SECTION_D => decode(figures.recv::<FigureSource<SECTION_D>>().await?)?,
            _ => unreachable!("figure request determined the matching source"),
        };
        let index = usize::from(chapter) - 1;
        let result = domain
            .execute("DATA_ANALYST", "create_figure", args.clone())
            .await
            .map_err(ShellError::message)?;
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
    domain: &impl Domain,
) -> ShellResult<()> {
    let chapter = SECTIONS[usize::from(SECTION) - 1];
    let snapshot = decode(analyst.recv::<Prepared<SECTION>>().await?)?;
    let mut section = analyst.recv::<Accepted<SECTION>>().await?;
    let material = encode(&snapshot)?;
    section
        .send::<InsertionInput<SECTION>>(&ChapterBytes(&material))
        .await?;
    let args = section.recv::<InsertFigure<SECTION>>().await?.into_value();
    same(&args["figure_id"], &snapshot["figures"][chapter]["id"])?;
    let result = domain
        .execute("DATA_ANALYST", "insert_figure", args)
        .await
        .map_err(ShellError::message)?;
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
