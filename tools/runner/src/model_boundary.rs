//! Live connection qualification with original inputs and actual WASI guests.
use crate::{
    evidence::{Result, save},
    model,
};
use serde_json::{Value, json};

#[hibana_runtime::main]
#[test]
#[ignore = "uses three Luna xhigh participants, direct machine-operation files and actual Wasm"]
async fn subscription_technicians_choose_actual_machine_operations() -> Result<()> {
    subscription_case("11E", 0).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "uses three Luna xhigh participants, direct build files, actual Wasm and the common object world"]
async fn subscription_builders_use_direct_files_and_current_objects() -> Result<()> {
    subscription_case("10E", 0).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "uses three Luna xhigh participants, actual Wasm guests and the common kitchen world"]
async fn subscription_chefs_use_actual_equipment_and_ingredients() -> Result<()> {
    subscription_case("12E", 0).await
}

use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[hibana_runtime::main]
#[test]
#[ignore = "uses three Luna xhigh participants through the existing ChatGPT subscription and actual WASI guests"]
async fn subscription_participants_use_independent_wash_guests() -> Result<()> {
    subscription_case("9E", 0).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "uses three Luna xhigh participants, actual document Endpoints and content-bound review failures"]
async fn subscription_writers_reconsider_the_actual_shared_document() -> Result<()> {
    subscription_case("7E", 2).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "uses five Luna xhigh participants through the existing ChatGPT subscription and actual WASI guests"]
async fn subscription_five_philosophers_use_independent_wash_guests() -> Result<()> {
    subscription_case("9M", 0).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "uses seven Luna xhigh participants through the existing ChatGPT subscription and actual WASI guests"]
async fn subscription_seven_philosophers_use_independent_wash_guests() -> Result<()> {
    subscription_case("9H", 0).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "uses seven Luna xhigh participants, actual WASI guests and upstream test failures"]
async fn subscription_developers_reconsider_through_seven_wash_guests() -> Result<()> {
    subscription_case("1H", 2).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "uses three Luna xhigh participants and actual operation-file and module Endpoints"]
async fn subscription_three_developers_use_direct_operation_files() -> Result<()> {
    subscription_case("1E", 0).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "uses five Luna xhigh participants and actual operation-file and module Endpoints"]
async fn subscription_five_developers_use_direct_operation_files() -> Result<()> {
    subscription_case("1M", 0).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "uses three Luna xhigh participants through actual API-task resource Endpoints"]
async fn subscription_api_participants_receive_the_actual_successful_result() -> Result<()> {
    subscription_case("8E", 0).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "uses three Luna xhigh participants and preserves the original terminal test failure"]
async fn subscription_api_participants_receive_the_actual_failed_result() -> Result<()> {
    subscription_case("8E", 2).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "uses three Luna xhigh participants and actual CI/CD resource Endpoints"]
async fn subscription_cicd_participants_use_resource_endpoints() -> Result<()> {
    subscription_case("16E", 0).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "uses three Luna xhigh participants, actual repository Endpoints and upstream review failures"]
async fn subscription_review_participants_reconsider_with_resource_endpoints() -> Result<()> {
    subscription_case("6E", 2).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "uses five Luna xhigh participants, actual WASI guests and the common published-content simulator"]
async fn subscription_refactor_participants_use_actual_repository_and_test_endpoints() -> Result<()>
{
    subscription_case("4M", 0).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "uses four Luna xhigh participants, actual Wasm guests and the common instrument environment"]
async fn subscription_scientists_use_actual_instrument_endpoints() -> Result<()> {
    subscription_case("14E", 0).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "uses five Luna xhigh scientists, actual Wasm, regulatory rejection and selective reanalysis"]
async fn subscription_regulatory_scientists_reconsider_actual_results() -> Result<()> {
    subscription_case("14M", 2).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "uses seven Luna xhigh scientists, actual Wasm, finite samples and source-bound effect receipts"]
async fn subscription_seven_scientists_decide_from_actual_multi_round_evidence() -> Result<()> {
    subscription_case("14H", 0).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "uses three Luna xhigh scientists, actual Wasm guests and common material causality"]
async fn subscription_lab_scientists_use_actual_specimens_and_instruments() -> Result<()> {
    subscription_case("13E", 0).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "uses four Luna xhigh scientists, actual Wasm and biological/QC reconsideration"]
async fn subscription_lab_quality_scientists_reconsider_actual_results() -> Result<()> {
    subscription_case("13M", 2).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "uses five Luna xhigh participants, real Wasm and actual imaging/diagnosis/conference data"]
async fn subscription_multi_specialty_participants_share_actual_findings() -> Result<()> {
    subscription_case("5M", 0).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "uses seven Luna xhigh participants, real Wasm and two-patient surgery/ICU care"]
async fn subscription_critical_care_participants_share_actual_patient_findings() -> Result<()> {
    subscription_case("5H", 0).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "uses three Luna xhigh participants, real Wasm and actual clinical result data"]
async fn subscription_medical_participants_share_actual_patient_findings() -> Result<()> {
    subscription_case("5E", 0).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "uses three Luna xhigh participants and actual section Endpoints through WASI"]
async fn subscription_writing_three_participants_use_section_endpoints() -> Result<()> {
    subscription_case("2E", 0).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "uses five Luna xhigh participants and actual section Endpoints through WASI"]
async fn subscription_writing_five_participants_use_section_endpoints() -> Result<()> {
    subscription_case("2M", 0).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "uses seven Luna xhigh participants and actual section Endpoints through WASI"]
async fn subscription_writing_seven_participants_use_section_endpoints() -> Result<()> {
    subscription_case("2H", 0).await
}

// This harness starts the model connections and records their observations.
// Task permissions and Endpoint operations stay in the application's localsides.
async fn subscription_case(task: &str, depth: u8) -> Result<()> {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source_root = source_root.canonicalize().map_err(|error| {
        format!(
            "resolve experiment source {}: {error}",
            source_root.display()
        )
    })?;
    std::env::set_current_dir(&source_root)
        .map_err(|error| format!("open experiment source {}: {error}", source_root.display()))?;
    let directory = PathBuf::from(
        std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("CHOREOFIX_EVIDENCE_ROOT is required")?,
    )
    .join(format!(
        "subscription-{task}-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    let result = crate::run(task, depth, &directory).await?;
    println!("{}", directory.display());
    if !result["error"].is_null() {
        return Err("model run failed; see the preserved outcomes".into());
    }
    let outcomes = result["outcomes"].as_array().ok_or("run outcomes")?;
    let metadata: Value = serde_json::from_slice(&fs::read(
        directory
            .join("source/benchmark/descriptions")
            .join(task)
            .join("metadata.json"),
    )?)?;
    let participant_count = metadata["agents"]
        .as_array()
        .ok_or("original participant set")?
        .len();
    let participants: Vec<_> = outcomes
        .iter()
        .filter(|o| o["kind"] == "model_participant")
        .collect();
    if participants.len() != participant_count
        || outcomes.len() != participant_count + 1
        || participants.iter().any(|p| {
            p["result"]["status"] != "completed"
                || p["result"]["returned_model"] != model::MODEL
                || p["result"]["returned_effort"] != model::EFFORT
                || p["result"]["usage"]["totalTokens"].as_u64().is_none()
        })
    {
        return Err(
            "subscription participants did not complete; see the preserved outcomes".into(),
        );
    }
    let runtime = outcomes
        .iter()
        .find(|o| o["kind"] == "guest_runtime")
        .ok_or("runtime outcome")?;
    let expected_complete = task != "8E" || depth == 0;
    if runtime["result"]["score"]["complete"] != expected_complete
        || runtime["result"]["score"]["violations"] != json!([])
        || runtime["result"]["runtime_exit_code"] != 0
        || runtime["result"]["protocol"]["status"] != "complete"
        || runtime["result"]["backend_errors"] != json!([])
    {
        return Err("actual WASI model task did not complete; see the preserved outcomes".into());
    }
    let audited = crate::audit::run(&directory)?;
    save(&directory.join("audit.json"), &audited)?;
    let audit = &audited["protocol"];
    if task == "10E" {
        let compiles = audit["actual_compile_effects"]
            .as_u64()
            .filter(|count| (2..=3).contains(count))
            .ok_or("actual build compile count")?;
        if audit["domain_calls"].as_u64() != Some(compiles + 1)
            || audit["actual_resource_endpoint_requests"].as_u64() != Some(2 * compiles)
        {
            return Err("actual compiles, link and resource receives do not correspond".into());
        }
    }
    if audit["status"] != "passed"
        || (task == "11E"
            && (audit["actual_jobs"] != 4
                || audit["actual_reports"] != 1
                || audit["actual_typed_file_requests"] != 5
                || audit["actual_resource_endpoint_requests"] != 4
                || audit["actual_resource_endpoint_returns"] != 4
                || audit["dynamic_protocol"]["status"] != "passed"))
        || (task == "10E"
            && (audit["final_resource_endpoint_returns"] != 2
                || audit["simulator_complete"] != true
                || audit["dynamic_protocol"]["status"] != "passed"))
        || (task == "6E"
            && (audit["actual_resource_endpoint_requests"] != 4 + u64::from(depth)
                || audit["actual_resource_endpoint_returns"] != 4 + u64::from(depth)
                || audit["actual_review_requests"] != 2 + u64::from(depth)))
        || (task == "16E"
            && (audit["actual_resource_endpoint_requests"] != 3
                || audit["actual_resource_endpoint_returns"] != 3))
        || (matches!(task, "9E" | "9M" | "9H")
            && audit["actual_resource_endpoint_returns"] != 2 * participant_count)
        || (task == "1H"
            && (audit["actual_test_resolver_decisions"]
                != participant_count * (usize::from(depth) + 1)
                || audit["actual_resource_endpoint_requests"]
                    != 2 * participant_count * (usize::from(depth) + 1)
                || audit["actual_resource_endpoint_returns"]
                    != 2 * participant_count * (usize::from(depth) + 1)))
        || (matches!(task, "1E" | "1M")
            && (audit["actual_resource_endpoint_requests"] != 2 * participant_count
                || audit["actual_resource_endpoint_returns"] != 2 * participant_count
                || audit["simulator_complete"] != expected_complete))
        || (task == "8E"
            && (audit["actual_resource_endpoint_requests"] != 2
                || audit["actual_resource_endpoint_returns"] != 2
                || audit["simulator_complete"] != expected_complete))
        || (matches!(task, "2E" | "2M" | "2H")
            && (audit["actual_resource_endpoint_requests"] != 2 * participant_count
                || audit["actual_resource_endpoint_returns"] != 2 * participant_count
                || audit["simulator_complete"] != expected_complete))
    {
        return Err("delegation audit did not confirm all actual returns".into());
    }
    Ok(())
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires real Luna participants and source-qualified 10M"]
async fn subscription_cascading_build() -> Result<()> {
    subscription_case("10M", 2).await
}
