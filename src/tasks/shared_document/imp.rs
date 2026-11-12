//! Material values and completed review receipts, independent of coordination.
use super::global::{CHAPTER_A, CHAPTER_B};
use crate::shell::{ShellError, ShellResult};
use serde_json::Value;

pub fn decode(bytes: &[u8]) -> ShellResult<Value> {
    serde_json::from_slice(bytes).map_err(|e| ShellError::message(e.to_string()))
}
pub fn encode(value: &Value) -> ShellResult<Vec<u8>> {
    serde_json::to_vec(value).map_err(|e| ShellError::message(e.to_string()))
}
pub fn chapter(value: &Value) -> ShellResult<&str> {
    value["section_name"]
        .as_str()
        .filter(|name| matches!(*name, CHAPTER_A | CHAPTER_B))
        .ok_or_else(|| ShellError::message("request names a chapter outside this document"))
}
pub fn same(first: &Value, second: &Value) -> ShellResult<()> {
    if first != second {
        return Err(ShellError::message(
            "document request differs from admitted arguments",
        ));
    }
    Ok(())
}
pub fn material(
    result: &Value,
    arguments: &Value,
    (agent, tool): (&str, &str),
) -> ShellResult<Value> {
    let section = chapter(arguments)?;
    if result["tool_name"] != tool
        || result["success"] != true
        || result["data"]["agent"] != agent
        || result["data"]["section"] != section
        || !result["data"]["artifact"]["id"].is_string()
        || !result["data"]["artifact"]["content"].is_string()
    {
        return Err(ShellError::message(
            "effect did not return this chapter's actual material",
        ));
    }
    Ok(result["data"]["artifact"].clone())
}
pub fn reviewed(result: &Value, snapshot: &Value, scope: &str) -> ShellResult<()> {
    for category in ["sections", "references"] {
        if snapshot[category]
            .as_object()
            .is_none_or(|rows| rows.len() != 2)
            || [CHAPTER_A, CHAPTER_B].into_iter().any(|section| {
                !snapshot[category][section]["id"].is_string()
                    || !snapshot[category][section]["content"].is_string()
            })
        {
            return Err(ShellError::message(
                "review needs both actual chapters and references",
            ));
        }
    }
    if result["tool_name"] != "review_sections"
        || !result["success"].is_boolean()
        || result["data"]["agent"] != "EDITOR"
        || result["data"]["scope"] != scope
        || result["data"]["input_snapshot"] != *snapshot
        || !result["data"]["review_id"].is_string()
        || result["data"]["status"]
            != if result["success"] == true {
                "reviewed"
            } else {
                "needs_revision"
            }
    {
        return Err(ShellError::message(
            "review differs from the current complete document",
        ));
    }
    Ok(())
}
pub fn combine_inputs(
    admitted: &Value,
    resource: &Value,
    (snapshot, reviews): (&Value, &[Value; 2]),
) -> ShellResult<()> {
    same(admitted, resource)?;
    let mut sections: Vec<_> = admitted["sections"]
        .as_str()
        .ok_or_else(|| ShellError::message("expected the actual section list"))?
        .split(',')
        .map(str::trim)
        .filter(|section| !section.is_empty())
        .collect();
    sections.sort_unstable();
    if sections != [CHAPTER_A, CHAPTER_B] {
        return Err(ShellError::message(
            "combination must preserve both actual chapters",
        ));
    }
    for (review, scope) in reviews.iter().zip([CHAPTER_A, CHAPTER_B]) {
        reviewed(review, snapshot, scope)?;
        if review["success"] != true {
            return Err(ShellError::message(
                "combination needs both actual positive reviews",
            ));
        }
    }
    if reviews[0]["data"]["review_id"] == reviews[1]["data"]["review_id"] {
        return Err(ShellError::message(
            "one review cannot stand for two chapter reviews",
        ));
    }
    Ok(())
}
pub fn combined(result: &Value, snapshot: &Value, reviews: &[Value; 2]) -> ShellResult<()> {
    let mut sections: Vec<_> = result["data"]["sections"]
        .as_array()
        .ok_or_else(|| ShellError::message("combined result is missing its actual chapter list"))?
        .iter()
        .filter_map(Value::as_str)
        .collect();
    sections.sort_unstable();
    if result["tool_name"] != "combine_sections"
        || result["success"] != true
        || result["data"]["agent"] != "EDITOR"
        || result["data"]["input_snapshot"] != *snapshot
        || !result["data"]["artifact_id"].is_string()
        || sections != [CHAPTER_A, CHAPTER_B]
        || result["data"]["sections"]
            .as_array()
            .is_none_or(|items| items.len() != 2)
        || result["data"]["review_receipts"][CHAPTER_A] != reviews[0]["data"]["review_id"]
        || result["data"]["review_receipts"][CHAPTER_B] != reviews[1]["data"]["review_id"]
    {
        return Err(ShellError::message(
            "combined material differs from the checked document and reviews",
        ));
    }
    Ok(())
}
