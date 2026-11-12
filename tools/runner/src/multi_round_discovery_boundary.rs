//! Seven actual Wasm scientists, finite stock and the common scientific world.
use crate::{
    evidence::{Result, Trace, archive_build_receipt, digest, save},
    oracle, process,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

fn response(reply: &Value) -> Result<Value> {
    let text = reply["stdout"]
        .as_str()
        .ok_or("scientific terminal output")?;
    let at = text
        .find('{')
        .ok_or_else(|| format!("scientific response missing: {reply}"))?;
    serde_json::Deserializer::from_str(&text[at..])
        .into_iter::<Value>()
        .next()
        .ok_or("empty scientific response")?
        .map_err(Into::into)
}

#[hibana_runtime::main]
#[test]
#[ignore = "requires the current-source seven-scientist runtime, receipt, wash and common world"]
async fn seven_actual_wasm_guests_retest_review_exhaust_and_reconsider() -> Result<()> {
    std::env::set_current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))?;
    let binary = PathBuf::from(
        std::env::var_os("CHOREOFIX_MULTI_ROUND_DISCOVERY_WASI_BINARY").ok_or("14H runtime")?,
    )
    .canonicalize()?;
    let receipt = PathBuf::from(
        std::env::var_os("CHOREOFIX_MULTI_ROUND_DISCOVERY_BUILD_RECEIPT").ok_or("14H receipt")?,
    );
    let root =
        PathBuf::from(std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("14H evidence root")?);
    fs::create_dir_all(&root)?;
    let inputs = crate::sources::inputs(&std::env::current_dir()?)?;
    let source_before = crate::sources::hashes(&inputs)?;
    let selected = std::env::var("CHOREOFIX_MULTI_ROUND_DISCOVERY_CASE")
        .ok()
        .map(|v| v.parse::<usize>())
        .transpose()?;
    if selected.is_some_and(|n| n >= crate::qualification::MULTI_ROUND_DISCOVERY_CASES) {
        return Err("14H diagnostic case must be 0..12".into());
    }
    let mut completed = Vec::new();
    for case in 0..crate::qualification::MULTI_ROUND_DISCOVERY_CASES {
        if selected.is_some_and(|n| n != case) {
            continue;
        }
        let depth = match case {
            2 => 1,
            3 => 2,
            _ => 0,
        };
        let bio_attempts = if matches!(case, 4 | 6 | 7) { 2 } else { 1 };
        let tox_attempts = if matches!(case, 5 | 6) { 2 } else { 1 };
        let analyses = if case == 9 { [6, 5] } else { [5, 6] };
        let reviews: &[&str] = match case {
            2 | 6 => &["initial_review", "hold_review"],
            3 | 10 => &["initial_review", "initial_review", "hold_review"],
            _ => &["initial_review"],
        };
        let decisions: &[&str] = match case {
            1 => &["abort"],
            8 => &["retry", "automatic"],
            _ => &["automatic"],
        };
        let failures: &[(&str, u32)] = match case {
            7 => &[("run_cell_assay", 1)],
            10 => &[("verify_regulatory_compliance", 0)],
            11 => &[("run_hplc_analysis", 1), ("run_hplc_analysis", 2)],
            _ => &[],
        };
        let delay = if case == 9 { 1.0 } else { 0.01 };
        let output = root.join(format!(
            "multi-round-discovery-case{case}-{}",
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        ));
        fs::create_dir(&output)?;
        fs::write(
            output.join("mechanism-driver.rs"),
            include_str!("multi_round_discovery_boundary.rs"),
        )?;
        let build = archive_build_receipt(&binary, &receipt, &output)?;
        let module = Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
        crate::projection::archive(&binary, &output, "14H")?;
        save(
            &output.join("case.json"),
            &json!({"task":"14H","case":case,"depth":depth,
            "bio_attempts_per_round":bio_attempts,"tox_attempts_per_round":tox_attempts,"analysis_order":analyses,
            "reviews_per_round":reviews,"decisions":decisions,"failure_overrides":failures,"delay_multiplier":delay,
            "runtime_binary_sha256":digest(&binary)?,"guest_wasm_sha256":digest(module)?,
            "test_driver_sha256":digest(&std::env::current_exe()?)?,
            "test_driver_source_sha256":digest(&output.join("mechanism-driver.rs"))?,
            "compiled_projection_sha256":digest(&output.join("projection.json"))?,"build":build,
            "model_calls":0,"scored_benchmark":false,"accepted_port":false}),
        )?;
        let trace = Trace::create(&output.join("events.jsonl"))?;
        let (simulator, fixture) =
            oracle::Simulator::new("14H", depth, delay, trace.clone(), failures)?;
        save(&output.join("simulator-fixture.json"), &fixture)?;
        let (guest, runtime) =
            process::guest(&binary, module, &["14H"], &output, trace, simulator)?;
        let((),(),(),(),(),(),(),result)=hibana_runtime::race(async{
            async{
                loop{
                    let actual=response(&guest.command("CHEMIST",r#"echo '{"sample":"synthesis_qc"}' > /service/run_hplc_analysis; cat /service/result"#.into()).await?)?;
                    if actual["success"]==true{break;}
                }
                assert_eq!(response(&guest.command("CHEMIST",r#"echo '{"sample":"molecular_id"}' > /service/run_mass_spec_analysis; cat /service/result"#.into()).await?)?["success"],true);
                loop{
                    let actual=response(&guest.command("CHEMIST",r#"echo '{"sample":"documentation"}' > /service/run_hplc_analysis; cat /service/result"#.into()).await?)?;
                    if actual["success"]==true{break;}
                }
                assert_eq!(guest.command("CHEMIST","exit 0".into()).await?["exit_status"],0);
                Ok(())
            };
            async{
                loop{
                    let actual=response(&guest.command("BIOLOGIST",r#"echo '{"assay_type":"culture_prep"}' > /service/run_cell_assay; cat /service/result"#.into()).await?)?;
                    if actual["success"]==true{break;}
                }
                let mut rounds=0;
                loop{
                    rounds+=1;
                    for analysis in analyses{
                        if analysis==5{
                            for _ in 0..bio_attempts{
                                let actual=response(&guest.command("BIOLOGIST",r#"echo '{"assay_type":"bioassay"}' > /service/run_cell_assay; cat /service/result"#.into()).await?)?;
                                assert_eq!(actual["tool_name"],"run_cell_assay");
                                assert!(actual["data"]["event_id"].is_u64());
                            }
                        }else{
                            loop{
                                let actual=response(&guest.command("BIOLOGIST",r#"echo '{"sample":"stability"}' > /service/run_hplc_analysis; cat /service/result"#.into()).await?)?;
                                if actual["success"]==true{break;}
                            }
                        }
                    }
                    let actual=response(&guest.command("BIOLOGIST",r#"echo '{"evidence":"bioassay"}' > /service/submit_evidence; cat /service/result"#.into()).await?)?;
                    assert_eq!(actual["evidence"],"bioassay");
                    if actual["coordination"]["outcome"].is_string(){break;}
                    assert_eq!(actual["coordination"]["received_specimens"],json!(["BIOASSAY","STABILITY"]));
                }
                assert_eq!(rounds,decisions.len());
                assert_eq!(guest.command("BIOLOGIST","exit 0".into()).await?["exit_status"],0);
                Ok(())
            };
            async{
                assert_eq!(response(&guest.command("TOXICOLOGIST",r#"echo '{"sample":"baseline_markers"}' > /service/run_mass_spec_analysis; cat /service/result"#.into()).await?)?["success"],true);
                let mut rounds=0;
                loop{
                    rounds+=1;
                    loop{
                        let actual=response(&guest.command("TOXICOLOGIST",r#"echo '{"sample":"purity_check"}' > /service/run_hplc_analysis; cat /service/result"#.into()).await?)?;
                        if actual["success"]==true{break;}
                    }
                    for _ in 0..tox_attempts{
                        let actual=response(&guest.command("TOXICOLOGIST",r#"echo '{"assay_type":"cytotoxicity"}' > /service/run_cell_assay; cat /service/result"#.into()).await?)?;
                        assert_eq!(actual["tool_name"],"run_cell_assay");
                        assert!(actual["data"]["event_id"].is_u64());
                    }
                    let actual=response(&guest.command("TOXICOLOGIST",r#"echo '{"evidence":"cytotoxicity"}' > /service/submit_evidence; cat /service/result"#.into()).await?)?;
                    assert_eq!(actual["evidence"],"cytotoxicity");
                    if actual["coordination"]["outcome"].is_string(){break;}
                    assert_eq!(actual["coordination"]["received_specimens"],json!(["PURITY","TOXICITY"]));
                }
                assert_eq!(rounds,decisions.len());
                assert_eq!(guest.command("TOXICOLOGIST","exit 0".into()).await?["exit_status"],0);
                Ok(())
            };
            async{
                let identity=response(&guest.command("FORMULATION_SCIENTIST","cat /service/result".into()).await?)?;
                let arguments=json!({"compound":identity["data"]["compound"]});
                let mut rounds=0;
                loop{
                    rounds+=1;
                    let actual=loop{
                        let actual=response(&guest.command("FORMULATION_SCIENTIST",format!("echo '{arguments}' > /service/prepare_formulation; cat /service/result")).await?)?;
                        if actual["success"]==true{break actual;}
                    };
                    if actual["coordination"]["outcome"].is_string(){break;}
                    assert_eq!(actual["coordination"]["received_specimens"],json!(["FORMULATION"]));
                }
                assert_eq!(rounds,decisions.len());
                assert_eq!(guest.command("FORMULATION_SCIENTIST","exit 0".into()).await?["exit_status"],0);
                Ok(())
            };
            async{
                let mut rounds=0;
                loop{
                    rounds+=1;
                    let actual=response(&guest.command("CLINICAL_LEAD",r#"echo '{"data_type":"combined_review"}' > /service/review_clinical_data; cat /service/result"#.into()).await?)?;
                    assert_eq!(actual["success"],true);
                    assert_eq!(actual["tool_name"],"review_clinical_data");
                    if actual["coordination"]["outcome"].is_string(){break;}
                    assert_eq!(actual["coordination"]["received_specimens"],json!(["STUDY"]));
                }
                assert_eq!(rounds,decisions.len());
                assert_eq!(guest.command("CLINICAL_LEAD","exit 0".into()).await?["exit_status"],0);
                Ok(())
            };
            async{
                let mut clinical=response(&guest.command("REGULATORY_SPECIALIST","cat /service/result".into()).await?)?;
                let mut rounds=0;
                loop{
                    rounds+=1;
                    assert_eq!(clinical["tool_name"],"review_clinical_data");
                    for review in reviews{
                        let arguments=json!({"review_type":review});
                        let actual=response(&guest.command("REGULATORY_SPECIALIST",format!("echo '{arguments}' > /service/verify_regulatory_compliance; cat /service/result")).await?)?;
                        assert_eq!(actual["tool_name"],"verify_regulatory_compliance");
                        assert_eq!(actual["data"]["review_type"],*review);
                    }
                    let actual=response(&guest.command("REGULATORY_SPECIALIST",r#"echo '{"evidence":"regulatory"}' > /service/submit_evidence; cat /service/result"#.into()).await?)?;
                    assert_eq!(actual["evidence"],"regulatory");
                    if actual["coordination"]["outcome"].is_string(){break;}
                    clinical=actual["coordination"]["clinical"].clone();
                }
                assert_eq!(rounds,decisions.len());
                assert_eq!(guest.command("REGULATORY_SPECIALIST","exit 0".into()).await?["exit_status"],0);
                Ok(())
            };
            async{
                let mut regulatory=response(&guest.command("PROJECT_DIRECTOR","cat /service/result".into()).await?)?;
                for decision in decisions{
                    assert_eq!(regulatory["tool_name"],"verify_regulatory_compliance");
                    let outcome=if *decision=="automatic"{if regulatory["success"]==true{"continue"}else{"abort"}}else{decision};
                    let arguments=json!({"outcome":outcome});
                    let actual=response(&guest.command("PROJECT_DIRECTOR",format!("echo '{arguments}' > /service/submit_project_decision; cat /service/result")).await?)?;
                    assert_eq!(actual["success"],true);
                    assert_eq!(actual["data"]["outcome"],outcome);
                    if outcome=="retry"{regulatory=actual["coordination"]["regulatory"].clone();}
                }
                assert_eq!(guest.command("PROJECT_DIRECTOR","exit 0".into()).await?["exit_status"],0);
                Ok(())
            };
            async{runtime.await};
        },hibana_runtime::host::deadline(Duration::from_secs(240))).await?;
        save(&output.join("result.json"), &result)?;
        assert_eq!(
            result["protocol"],
            json!({"type":"run_result","status":"complete"}),
            "{result}"
        );
        assert_eq!(result["runtime_exit_code"], 0, "{result}");
        assert_eq!(result["backend_errors"], json!([]), "{result}");
        assert_eq!(result["score"]["complete"], true, "{result}");
        assert_eq!(result["score"]["violations"], json!([]), "{result}");
        assert_eq!(
            result["score"]["progress"]["final_outcome"],
            if matches!(case, 1..=3 | 6 | 8) {
                "abort"
            } else {
                "continue"
            }
        );
        assert_eq!(
            result["score"]["progress"]["samples_remaining"],
            if matches!(case, 4..=8) { 0 } else { 1 }
        );
        let events = result["score"]["events"]
            .as_array()
            .ok_or("scientific events")?;
        assert_eq!(
            events
                .iter()
                .filter(|e| e["tool"] == "submit_project_decision")
                .count(),
            decisions.len()
        );
        if matches!(case, 6 | 8) {
            assert!(events.iter().any(|e| e["result"]["status"] == "exhausted"));
        }
        if case == 10 {
            let actual: Vec<_> = events
                .iter()
                .filter(|e| e["tool"] == "verify_regulatory_compliance")
                .map(|e| e["success"].clone())
                .collect();
            assert_eq!(actual, vec![json!(false), json!(false), json!(true)]);
        }
        if case == 9 {
            assert!(
                events
                    .iter()
                    .any(|a| events.iter().any(|b| a["agent"] != b["agent"]
                        && a["completed_at"]
                            .as_f64()
                            .zip(b["timestamp"].as_f64())
                            .is_some_and(|(end, start)| start < end)
                        && b["completed_at"]
                            .as_f64()
                            .zip(a["timestamp"].as_f64())
                            .is_some_and(|(end, start)| start < end))),
                "independent effects did not overlap"
            );
        }
        if crate::sources::hashes(&inputs)? != source_before {
            return Err("scientific validation source changed during execution".into());
        }
        eprintln!("14H case {case}: Wasm complete; checking source occurrences and actual effects");
        let audit = crate::multi_round_discovery_audit::audit(&output.join("events.jsonl"))?;
        assert_eq!(audit["domain_calls"], events.len());
        assert_eq!(audit["effect_receives_bound_to_checked_occurrences"], true);
        save(
            &output.join("dynamic-audit.json"),
            &audit["dynamic_protocol"],
        )?;
        save(&output.join("effect-audit.json"), &audit)?;
        save(
            &output.join("qualification.json"),
            &json!({"task":"14H","case":case,"status":"mechanism_case_passed",
            "declared_conditions":crate::qualification::MULTI_ROUND_DISCOVERY_CASES,
            "source_unchanged":true,"frame_contracts":audit["frame_contracts"],"scientific_effects":events.len(),"model_calls":0,
            "runtime_binary_sha256":digest(&binary)?,"compiled_projection_sha256":digest(&output.join("projection.json"))?,
            "test_driver_sha256":digest(&std::env::current_exe()?)?,
            "scored_runs":0,"accepted_port":false,"dynamic_protocol_qualified":true,
            "separate_effect_audit_qualified":true,"effect_receives_bound_to_checked_occurrences":true}),
        )?;
        completed.push(case);
        eprintln!("14H case {case}: dynamic audit passed");
    }
    assert_eq!(
        completed.len(),
        if selected.is_some() {
            1
        } else {
            crate::qualification::MULTI_ROUND_DISCOVERY_CASES
        }
    );
    save(
        &root.join(format!(
            "matrix-{}.json",
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        )),
        &json!({"task":"14H","conditions":completed,"full_matrix":selected.is_none(),"model_calls":0,"accepted_port":false}),
    )?;
    Ok(())
}
