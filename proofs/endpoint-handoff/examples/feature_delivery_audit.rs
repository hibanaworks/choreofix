//! Run the production dynamic auditor over one saved whole-task native capture.
mod evidence {
    pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
}
#[path = "../../../tools/runner/src/projection/dynamic.rs"]
mod dynamic;
use serde_json::{Value, json};

fn main() -> evidence::Result<()> {
    let mut args = std::env::args_os().skip(1);
    let projection = args.next().ok_or("expected projection and case file")?;
    let case = args.next().ok_or("expected case file")?;
    if args.next().is_some() {
        return Err("expected exactly projection and case file".into());
    }
    let artifact: Value = serde_json::from_reader(std::fs::File::open(projection)?)?;
    let case: Value = serde_json::from_reader(std::fs::File::open(case)?)?;
    if !case["runtime_error"].is_null() {
        return Err("an incomplete failed native run is not a complete execution".into());
    }
    let mut rows = vec![
        json!({"direction":"runtime","message":{"type":"protocol_capture","producer":0,"records":case["records"]}}),
        json!({"direction":"runtime","message":{"type":"protocol_checkpoint","producer":0,"session":case["session"],"checkpoint":case["checkpoint"]}}),
    ];
    rows.extend(
        case["frames"]
            .as_array()
            .ok_or("missing frame observations")?
            .iter()
            .map(|frame| json!({"direction":"runtime","message":frame})),
    );
    let report = dynamic::check(&artifact, &rows)?;
    println!(
        "{}",
        json!({"scope":"saved native whole-task capture","model_calls":0,"report":report})
    );
    Ok(())
}
