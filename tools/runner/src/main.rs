mod api_audit;
#[cfg(test)]
mod api_boundary;
mod audit;
mod build_audit;
#[cfg(test)]
mod build_boundary;
mod cascading_build_audit;
#[cfg(test)]
mod cascading_build_boundary;
mod cicd_audit;
#[cfg(test)]
mod cicd_boundary;
mod cicd_pipeline_audit;
#[cfg(test)]
mod cicd_pipeline_boundary;
mod codebase_audit;
#[cfg(test)]
mod codebase_boundary;
mod coordination_audit;
#[cfg(test)]
mod coordination_boundary;
mod critical_care_audit;
#[cfg(test)]
mod critical_care_boundary;
#[cfg(test)]
mod dining_source_trials;
mod document_audit;
#[cfg(test)]
mod document_boundary;
mod drug_discovery_audit;
#[cfg(test)]
mod drug_discovery_boundary;
mod evidence;
mod feature_delivery_audit;
#[cfg(test)]
mod feature_delivery_boundary;
mod figure_document_audit;
#[cfg(test)]
mod figure_document_boundary;
mod full_stack_api_audit;
#[cfg(test)]
mod full_stack_api_boundary;
mod journal;
mod kitchen_audit;
#[cfg(test)]
mod kitchen_boundary;
mod lab_audit;
#[cfg(test)]
mod lab_boundary;
mod lab_cascading_audit;
#[cfg(test)]
mod lab_cascading_boundary;
mod lab_quality_audit;
#[cfg(test)]
mod lab_quality_boundary;
mod large_document_audit;
#[cfg(test)]
mod large_document_boundary;
mod manufacturing_audit;
#[cfg(test)]
mod manufacturing_boundary;
mod medical_audit;
#[cfg(test)]
mod medical_boundary;
#[cfg(test)]
mod microservice_migration_boundary;
mod model;
#[cfg(test)]
mod model_boundary;
mod multi_round_discovery_audit;
#[cfg(test)]
mod multi_round_discovery_boundary;
mod multi_specialty_audit;
#[cfg(test)]
mod multi_specialty_boundary;
mod oracle;
#[path = "../../../src/tasks/philosophers/choreofs/identity.rs"]
mod philosopher_identity;
mod philosophers_audit;
#[cfg(test)]
mod philosophers_boundary;
mod process;
mod projection;
mod qualification;
mod refactor_audit;
#[cfg(test)]
mod refactor_boundary;
mod regulatory_discovery_audit;
#[cfg(test)]
mod regulatory_discovery_boundary;
mod report_audit;
#[cfg(test)]
mod report_boundary;
mod research_paper_audit;
#[cfg(test)]
mod research_paper_boundary;
mod review_audit;
#[cfg(test)]
mod review_boundary;
mod snapshot;
mod sources;
mod staged_integration_audit;
#[cfg(test)]
mod staged_integration_boundary;
mod survey_paper_audit;
#[cfg(test)]
mod survey_paper_boundary;
#[cfg(test)]
mod timeout_boundary;
mod wire;
mod writing_audit;
#[cfg(test)]
mod writing_boundary;

use evidence::{Result, Trace, save};
use hibana_runtime::{host::deadline, race};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

fn prompt(source: &Path, task: &str, actor: &Value) -> Result<String> {
    let agent = actor["agent"].as_str().ok_or("participant name")?;
    let input = source.join("benchmark/descriptions").join(task);
    let tools: Value = serde_json::from_slice(&fs::read(input.join("tools.json"))?)?;
    let schemas: Vec<_> = tools
        .as_array()
        .ok_or("original tools")?
        .iter()
        .map(|v| &v["function"])
        .filter(|v| {
            v["agent_ids"]
                .as_array()
                .is_some_and(|a| a.contains(&json!(agent)))
        })
        .collect();
    let family = task.trim_end_matches(['E', 'M', 'H']).parse::<u8>()?;
    let dining = family == 9;
    let coding = family == 1;
    let api = task == "8E";
    let cicd = task == "16E";
    let cicd_pipeline = task == "16M";
    let review = task == "6E";
    let feature_delivery = task == "6M";
    let staged_integration = task == "6H";
    let report = task == "3E";
    let research_paper = task == "3M";
    let survey_paper = task == "3H";
    let document = task == "7E";
    let figure_document = task == "7M";
    let large_document = task == "7H";
    let build = matches!(task, "10E" | "10M");
    let manufacturing = task == "11E";
    let kitchen = task == "12E";
    let coordination = task == "4E";
    let refactor = task == "4M";
    let lab = task == "13E";
    let lab_quality = task == "13M";
    let lab_cascading = task == "13H";
    let drug_discovery = task == "14E";
    let regulatory_discovery = task == "14M";
    let multi_round_discovery = task == "14H";
    let medical = task == "5E";
    let multi_specialty = task == "5M";
    let critical_care = task == "5H";
    if !(dining
        || coding
        || api
        || task == "8M"
        || cicd
        || cicd_pipeline
        || review
        || feature_delivery
        || staged_integration
        || document
        || figure_document
        || large_document
        || build
        || report
        || research_paper
        || survey_paper
        || coordination
        || refactor
        || lab
        || lab_quality
        || lab_cascading
        || drug_discovery
        || regulatory_discovery
        || multi_round_discovery
        || medical
        || multi_specialty
        || critical_care
        || manufacturing
        || kitchen
        || family == 2)
    {
        return Err(format!("task {task} has no model prompt implementation").into());
    }
    let data = if large_document {
        json!({"participant":agent,"resources":actor["resources"]})
    } else if multi_round_discovery || cicd_pipeline || task == "8M" {
        json!({"participant":agent,"resources":actor["resources"],"operations":actor["operations"]})
    } else if dining {
        json!({"participant":agent,"resources":actor["forks"],"philosopher":agent})
    } else if coding {
        json!({"participant":agent,"resources":actor["modules"],
        "feature_name":actor["calls"][0]["arguments"]["feature_name"]})
    } else if review
        || feature_delivery
        || staged_integration
        || document
        || figure_document
        || build
        || report
        || research_paper
        || survey_paper
        || coordination
        || refactor
        || lab
        || lab_quality
        || lab_cascading
        || drug_discovery
        || regulatory_discovery
        || medical
        || multi_specialty
        || critical_care
        || manufacturing
        || kitchen
    {
        json!({"participant":agent,"resources":actor["resources"],"assignments":actor["calls"]})
    } else if api || cicd {
        json!({"participant":agent,"resources":actor["resources"],
        "arguments":actor["calls"][0]["arguments"]})
    } else {
        json!({"participant":agent,"resources":actor["sections"],
        "topic":actor["calls"][0]["arguments"]["topic"],
        "section_name":actor["calls"][1]["arguments"]["section_name"]})
    };
    let objects = if large_document {
        match agent {
            "WRITER_A" => {
                "/service/write_section, /service/update_references, /service/finalize_section, /service/revise_section and /service/apply_minor_edit each take /CHAPTER_1 or /CHAPTER_2. The read-only /service/contribution and /service/evaluation carry the actual cross-chapter inputs for finalization. /service/editorial_outcome delivers reviews and the final publication or withholding. A minor review can cover both chapters. Each listed chapter has its own amendment file and reply, using the same review_id. Their combined reassessment is delivered after all listed amendments"
            }
            "WRITER_B" => {
                "/service/write_section/CHAPTER_3, /service/update_references/CHAPTER_3, /service/revise_section/CHAPTER_3 and /service/apply_minor_edit/CHAPTER_3 return the reply to each request. /service/evaluation delivers the reviewed evaluation needed for Chapter 3 references. /service/editorial_outcome delivers fact or review decisions, including reassessment after exact minor edits. The last delivery on this file is the successful review_sections result for CHAPTER_3, or the successful fact_check of your exact minor edit. Publication is delivered to the Part I writer, checker and reviewer"
            }
            "WRITER_C" => {
                "/service/write_section/CHAPTER_4, /service/update_references/CHAPTER_4, /service/revise_section/CHAPTER_4 and /service/apply_minor_edit/CHAPTER_4 return the reply to each request. The initial draft and references can proceed in either order. /service/editorial_outcome delivers fact or review decisions, including reassessment after exact minor edits. The last delivery on this file is the successful review_sections result for CHAPTER_4, or the successful fact_check of your exact minor edit. Publication is delivered to the Part I writer, checker and reviewer"
            }
            "FIGURE_CREATOR" => {
                "/service/create_figure and /service/insert_figure, each with /CHAPTER_1, /CHAPTER_3 or /CHAPTER_4 appended. For each of those three chapters, create its own figure. Read that chapter's creation response and use its data.figure_id in that chapter's insertion request. If insertion is refused, check that you have a figure ID from this chapter's creation. Finish after you have read a successful insertion response for each chapter"
            }
            "FACT_CHECKER" => {
                "Read /service/inbox for the next delivery. A manuscript has sections, figures and references. Its sections identify CHAPTER_3, CHAPTER_4 or CHAPTERS_1_2. Submit section_name to /service/fact_check/<scope> and read the check result from that operation file. Chapter decisions, amendment checks and the final publication or withholding also arrive on the inbox. A notification can repeat a check_id from the operation reply"
            }
            "REVIEWER" => {
                "Read /service/inbox for each next delivery. A fact_check result identifies its scope in data.section and includes the manuscript in data.input_snapshot.sections. A failed check calls for a writer revision. Its disposition can carry the same check ID. A reassessment can confirm exact minor edits. Review decisions, final publication and withholding also arrive on this file. Submit your own assessment to the write-only /service/review_sections/<scope> for CHAPTERS_1_2, CHAPTER_3 or CHAPTER_4. You may accept, request a major revision or specify exact minor_edits with section_name, before and after. Any assessment can include optional feedback text for the author"
            }
            "EDITOR_IN_CHIEF" => {
                "/service/material delivers the manuscript with its assessment evidence, including reassessment after an editorial rejection. /service/make_editorial_decision accepts your own review_summary and returns the actual decision. /service/combine_sections accepts the four comma-separated chapter identifiers in sections and returns the publication result. A final withheld manuscript carries a reviews array"
            }
            _ => return Err("unknown large document participant".into()),
        }
    } else if cicd_pipeline {
        match agent {
            "BUILD_MASTER" => {
                "/service/build_frontend_first or /service/build_backend_first, and /service/repair; read /service/qa_report and /service/result"
            }
            "FRONTEND_DEV" | "BACKEND_DEV" => {
                "/service/compile_code, /service/run_unit_tests and /service/publish_artifact; read /service/start, /service/workspace, /service/revise, /service/build_verified, /service/qa_revise, /service/qa_done and /service/result"
            }
            "QA_ENGINEER" => {
                "/service/pull_artifacts, /service/run_integration_tests and /service/run_e2e_tests; read /service/start and /service/result"
            }
            "RELEASE_ENGINEER" => {
                "/service/pull_artifacts, /service/deploy_staging, /service/run_staging_smoke, /service/deploy_production and /service/run_production_smoke; read /service/start, /service/staging_ready, /service/promote, /service/abort and /service/result"
            }
            _ => return Err("unknown pipeline participant".into()),
        }
    } else if task == "8M" {
        match agent {
            "DB_ADMIN" => "migrate_schema under /service; read schema_status and outcome",
            "BACKEND_DEV" => {
                "define_api, implement_backend and request_migration under /service; read schema and outcome"
            }
            "FRONTEND_DEV" => "implement_frontend under /service; read backend and outcome",
            "REVIEWER" => {
                "code_review/backend_api, code_review/frontend_ui and repair under /service; read backend_candidate, frontend_candidate and outcome"
            }
            "TESTER" => "run_integration_tests under /service; read test_candidate",
            _ => return Err("unknown full-stack participant".into()),
        }
    } else if survey_paper {
        match agent {
            "RESEARCHER_A" | "RESEARCHER_B" | "RESEARCHER_C" | "RESEARCHER_D" => {
                "/service/research_topic, /service/write_section, /service/update_references, /service/revise_section; read-only /service/review_result"
            }
            "DATA_ANALYST" => {
                "For each section_a, section_b, section_c, section_d, under /service/<section>: research and insertion_input (read-only), create_figure and insert_figure"
            }
            "REVIEWER" => {
                "For each section_a, section_b, section_c, section_d, under /service/<section>: review_input (read-only) and peer_review"
            }
            "EDITOR_IN_CHIEF" => {
                "/service/make_editorial_decision, /service/combine_sections; read-only /service/accepted_sections"
            }
            _ => return Err("unknown survey participant".into()),
        }
    } else if research_paper {
        match agent {
            "RESEARCHER_A" | "RESEARCHER_B" | "RESEARCHER_C" => {
                "/service/research_topic, /service/supplemental_research, /service/write_section, /service/update_references and /service/revise_section; read-only /service/review"
            }
            "FACT_CHECKER" => {
                "/service/fact_check/<section>; read-only /service/section/<section> and /service/approved/<section>"
            }
            "EDITOR" => {
                "/service/review_sections/<section> and /service/combine_sections; read-only /service/section/<section>"
            }
            _ => return Err("unknown research paper participant".into()),
        }
    } else if medical {
        match agent {
            "ER_DOCTOR" => {
                "/service/assess_patient, /service/order_tests and /service/perform_imaging; read-only /service/summary"
            }
            "CARDIOLOGIST" => {
                "/service/order_tests and /service/diagnose; read-only /service/referral and /service/findings"
            }
            "PHARMACIST" => "/service/review_prescriptions",
            _ => return Err("unknown medical participant".into()),
        }
    } else if multi_specialty {
        match agent {
            "ER_DOCTOR" => {
                "/service/assess_patient, /service/order_tests and /service/conduct_case_conference; read-only /service/candidate and /service/summary"
            }
            "CARDIOLOGIST" | "NEUROLOGIST" => {
                "/service/order_tests and /service/diagnose; read-only /service/referral, /service/findings and /service/summary"
            }
            "RADIOLOGIST" => {
                "/service/perform_imaging_ct, /service/interpret_imaging_ct, /service/perform_imaging_mri and /service/interpret_imaging_mri; read-only /service/ct_order, /service/mri_order and /service/summary"
            }
            "PHARMACIST" => "/service/review_prescriptions",
            _ => return Err("unknown multi-specialty participant".into()),
        }
    } else if critical_care {
        match agent {
            "ER_DOCTOR_A" | "ER_DOCTOR_B" => {
                "/service/assess_patient, /service/order_tests and /service/diagnose; read-only /service/findings and /service/summary"
            }
            "RADIOLOGIST" => {
                "/service/perform_imaging_ct_a, /service/interpret_imaging_ct_a, /service/perform_imaging_ct_b, /service/interpret_imaging_ct_b, /service/perform_imaging_mri_b and /service/interpret_imaging_mri_b"
            }
            "ANESTHESIOLOGIST" => "/service/prepare_anesthesia; read-only /service/diagnosis",
            "SURGEON" => "/service/perform_surgery; read-only /service/diagnosis",
            "ICU_COORDINATOR" => "/service/manage_icu; read-only /service/diagnosis",
            "PHARMACIST" => "/service/review_prescriptions",
            _ => return Err("unknown critical-care participant".into()),
        }
    } else if kitchen {
        "/service/prepare_base_dish, /service/prepare_ingredient and /service/combine_dish"
    } else if manufacturing {
        match agent {
            "TECHNICIAN_A" | "TECHNICIAN_B" => "/service/do_lathe_job and /service/do_mill_job",
            "SUPERVISOR" => "/service/collect_reports",
            _ => return Err("unknown manufacturing participant".into()),
        }
    } else if build {
        match agent {
            "BUILDER_A" | "BUILDER_B" | "BUILDER_C" if task == "10M" => {
                "/service/compile_module; read-only /service/updates, /service/test_result, /service/reconsideration and /service/outcome"
            }
            "BUILDER_B" if task == "10E" => "/service/compile_module; read-only /service/updates",
            "BUILDER_A" | "BUILDER_B" | "BUILDER_C" => "/service/compile_module",
            "TEST_RUNNER" if task == "10M" => {
                "/service/run_build_tests_a, /service/run_build_tests_b and /service/run_build_tests_c; read-only /service/next_object"
            }
            "INTEGRATOR" if task == "10M" => {
                "/service/link_modules, /service/rebuild (read repair candidates, write a module name), read-only /service/candidate"
            }
            "INTEGRATOR" => "/service/link_modules",
            _ => return Err("unknown shared-build participant".into()),
        }
    } else if figure_document {
        match agent {
            "WRITER_A" | "WRITER_B" => {
                "your assigned /service/write_section/<chapter> and /service/update_references/<chapter>, /service/read_section and /service/revise_section. Read-only /service/outcome"
            }
            "FIGURE_CREATOR" => {
                "/service/create_figure/{CHAPTER_1,CHAPTER_3} and /service/insert_figure/{CHAPTER_1,CHAPTER_3}"
            }
            "FACT_CHECKER" => "/service/fact_check/<scope>. Read-only /service/outcome",
            "EDITOR" => {
                "/service/review_sections/<chapter> and /service/combine_sections. Read-only /service/review_material"
            }
            _ => return Err("unknown figure document participant".into()),
        }
    } else if document {
        match agent {
            "WRITER_A" | "WRITER_B" => {
                "/service/write_section, /service/update_references and /service/revise_section; read-only /service/feedback"
            }
            "EDITOR" => {
                "/service/allocate_document, /service/review_chapter_1, /service/review_chapter_2, /service/reconsider_a, /service/reconsider_b and /service/combine_sections"
            }
            _ => return Err("unknown document participant".into()),
        }
    } else if report {
        "Each file is a direct choreography projection. Choose shell commands and private calculations freely; an unadmitted request returns an error to the same guest. Researchers use their assigned topic and section. Write the original tool arguments to research_topic, write_section, update_references or revise_section and read the actual result from that same file. Reading feedback receives the next actual review of your section. After a negative review, choose revision text and submit it; a new review checks the current actual section and references. After your section is approved, your research work is complete and you may exit while the editor and other researcher continue. The editor chooses section order by writing original review_sections arguments to review_section_a or review_section_b and reading the result from that file. Either initial section and repeated or alternating rejection schedules are supported. Review of one section can overlap revision of the other. combine_sections submits the actual publication request; it requires both current positive reviews. The choreography then moves editing and review authority before publication. No host chooses the next request, loan or command. There is no generic result mailbox or synthetic coordination envelope. The editor finishes after receiving the actual combine result. No acquire/release or notification commands are needed."
    } else if multi_round_discovery {
        match agent {
            "CHEMIST" => {
                "/service/run_hplc_analysis/synthesis_qc, /service/run_mass_spec_analysis/molecular_id, and /service/run_hplc_analysis/documentation; read-only /service/outcome"
            }
            "BIOLOGIST" => {
                "/service/run_cell_assay/culture_prep, /service/run_cell_assay/bioassay, /service/run_hplc_analysis/stability, and /service/submit_evidence; read-only /service/study"
            }
            "TOXICOLOGIST" => {
                "/service/run_mass_spec_analysis/baseline_markers, /service/run_hplc_analysis/purity_check, /service/run_cell_assay/cytotoxicity, and /service/submit_evidence; read-only /service/study"
            }
            "FORMULATION_SCIENTIST" => "/service/prepare_formulation; read-only /service/study",
            "CLINICAL_LEAD" => "/service/review_clinical_data; read-only /service/study",
            "REGULATORY_SPECIALIST" => {
                "/service/verify_regulatory_compliance/initial_review, /service/verify_regulatory_compliance/hold_review, and /service/submit_evidence; read-only /service/study"
            }
            "PROJECT_DIRECTOR" => "/service/submit_project_decision; read-only /service/study",
            _ => return Err("unknown multi-round discovery participant".into()),
        }
    } else if regulatory_discovery {
        match agent {
            "CHEMIST" => {
                "/service/run_hplc_analysis/synthesis_qc, /service/run_mass_spec_analysis/molecular_id, /service/run_hplc_analysis/documentation; read-only /service/approval"
            }
            "BIOLOGIST" => {
                "/service/run_cell_assay/culture_prep, /service/run_cell_assay/bioassay, /service/run_hplc_analysis/stability; read-only /service/compound and /service/study"
            }
            "TOXICOLOGIST" => {
                "/service/run_mass_spec_analysis/baseline_markers, /service/run_hplc_analysis/purity_check, /service/run_cell_assay/cytotoxicity; read-only /service/compound and /service/study"
            }
            "LEAD_SCIENTIST" => {
                "/service/run_mass_spec_analysis/reference_methods, /service/run_hplc_analysis/structural_confirmation, /service/run_mass_spec_analysis/characterization, /service/submit_decision and /service/reanalysis; read-only /service/bioactivity, /service/toxicity and /service/review"
            }
            "REGULATORY_SPECIALIST" => {
                "/service/verify_regulatory_compliance; read-only /service/decision"
            }
            _ => return Err("unknown regulatory discovery participant".into()),
        }
    } else if drug_discovery {
        match agent {
            "CHEMIST" => {
                "/service/run_hplc_analysis/synthesis_qc, /service/run_mass_spec_analysis/molecular_id, /service/run_hplc_analysis/documentation; read-only /service/decision"
            }
            "BIOLOGIST" => {
                "/service/run_cell_assay/culture_prep, /service/run_cell_assay/bioassay, /service/run_hplc_analysis/stability; read-only /service/compound"
            }
            "TOXICOLOGIST" => {
                "/service/run_mass_spec_analysis/baseline_markers, /service/run_hplc_analysis/purity_check, /service/run_cell_assay/cytotoxicity; read-only /service/compound"
            }
            "LEAD_SCIENTIST" => {
                "/service/run_mass_spec_analysis/reference_methods, /service/run_hplc_analysis/structural_confirmation, /service/run_mass_spec_analysis/characterization and /service/submit_decision; read-only /service/bioactivity and /service/toxicity"
            }
            _ => return Err("unknown discovery participant".into()),
        }
    } else if lab_cascading {
        match agent {
            "CHEMIST" => {
                "/service/run_synthesis, /service/run_analysis/synthesized_compound, /service/run_separation/compound_purification, /service/run_analysis/final_validation; read-only /service/qc"
            }
            "BIOLOGIST" => {
                "/service/run_separation/cell_cultures, /service/run_analysis/fluorescence, /service/run_bioassay, /service/publish/analyst and /service/publish/qc; read-only /service/batch"
            }
            "ANALYST" => {
                "/service/run_chromatography, /service/run_analysis/molecular_detail and /service/run_separation/verification; read-only /service/batch"
            }
            "QC_CONTROLLER" => {
                "/service/verify_result/BIOLOGIST; read-only /service/bioassay and /service/qc"
            }
            "SAFETY_OFFICER" => "/service/perform_inspection",
            _ => return Err("unknown cascading laboratory participant".into()),
        }
    } else if lab_quality {
        match agent {
            "CHEMIST" => {
                "/service/run_synthesis, /service/run_analysis/synthesized_compound, /service/run_separation/compound_purification and /service/run_analysis/final_validation; read-only /service/qc and /service/quality"
            }
            "BIOLOGIST" => {
                "/service/run_separation/cell_cultures, /service/run_analysis/fluorescence and /service/run_bioassay; read-only /service/sample and /service/qc"
            }
            "ANALYST" => {
                "read-only /service/bioassay; /service/run_analysis/molecular_detail and /service/run_separation/verification"
            }
            "QC_CONTROLLER" => {
                "/service/verify_result/CHEMIST and /service/verify_result/BIOLOGIST"
            }
            _ => return Err("unknown QC scientist".into()),
        }
    } else if lab {
        match agent {
            "CHEMIST" => {
                "/service/run_analysis/raw_materials, /service/run_separation/compound, read-only /service/quality and /service/run_analysis/final_validation"
            }
            "BIOLOGIST" => {
                "/service/run_separation/cell_cultures, /service/run_analysis/fluorescence, read-only /service/sample and /service/run_bioassay"
            }
            "ANALYST" => {
                "read-only /service/bioassay, /service/run_analysis/molecular_detail and /service/run_separation/verification"
            }
            _ => return Err("unknown laboratory scientist".into()),
        }
    } else if refactor {
        match agent {
            "ARCHITECT" => {
                "/service/design_proposal/module_a and /service/design_proposal/module_b"
            }
            "DEVELOPER_A" => {
                "/service/refactor_code/module_a, /service/refactor_code/SHARED_LIBRARY and /service/merge_code; read-only /service/review and /service/test_result"
            }
            "DEVELOPER_B" => {
                "/service/refactor_code/module_b, /service/refactor_code/SHARED_LIBRARY and /service/merge_code; read-only /service/review and /service/test_result"
            }
            "REVIEWER" => "/service/code_review/module_a and /service/code_review/module_b",
            "TESTER" => "/service/run_tests/module_a_tests and /service/run_tests/module_b_tests",
            _ => return Err("unknown refactor participant".into()),
        }
    } else if coordination {
        match agent {
            "BACKEND_DEV" => "/service/define_api, /service/implement_feature and /service/outcome",
            "FRONTEND_DEV" => "/service/agree_api, /service/implement_feature and /service/outcome",
            "TESTER" => "/service/run_tests",
            _ => return Err("unknown coordination participant".into()),
        }
    } else if staged_integration {
        match agent {
            "DEVELOPER_A" | "DEVELOPER_B" | "DEVELOPER_C" => {
                "/service/implement_feature, /service/merge_code; read-only /service/review/<feature>, /service/integration/<feature>, /service/release/<feature> and /service/interface/<other-feature>"
            }
            "REVIEWER" => {
                "/service/code_review/<feature>; read-only /service/candidate/<feature>, /service/integration/<feature> and /service/release/<feature>"
            }
            "TEST_WRITER" => {
                "/service/write_tests/<feature>; read-only /service/candidate/<feature>, /service/review/<feature>, /service/integration/<feature> and /service/release/<feature>"
            }
            "CI_RUNNER" => {
                "/service/run_tests/<feature>; read-only /service/publication/<feature> and /service/release/<feature>"
            }
            "RELEASE_MANAGER" => {
                "/service/deploy/<feature>, /service/validate/<feature>, /service/rollback/<feature>; read-only /service/release/<feature>"
            }
            _ => return Err("unknown staged-integration participant".into()),
        }
    } else if feature_delivery {
        match agent {
            "DEVELOPER_A" | "DEVELOPER_B" => {
                "/service/implement_feature and /service/merge_code; read-only /service/review, /service/merge_ready, /service/integration, /service/interface_change and /service/peer_review"
            }
            "REVIEWER" => {
                "/service/code_review/<pr_id>; read-only /service/code_ready/<feature> and /service/integration/<feature>"
            }
            "TEST_WRITER" => {
                "/service/write_tests/<target>; read-only /service/code_ready/<feature>, /service/review/<feature> and /service/integration"
            }
            "CI_RUNNER" => "/service/run_tests/<feature>; read-only /service/publication/<feature>",
            _ => return Err("unknown feature-delivery participant".into()),
        }
    } else if review {
        match agent {
            "DEVELOPER_A" | "DEVELOPER_B" => {
                "/service/implement_feature and /service/merge_code; read-only /service/review"
            }
            "REVIEWER" => {
                "/service/code_review/<pr_id>, using an assigned pr_id as the final path component"
            }
            _ => return Err("unknown review participant".into()),
        }
    } else if cicd {
        match agent {
            "DEVELOPER" => "/service/compile_and_test",
            "TESTER" => "/service/run_integration_tests",
            "DEPLOYER" => "/service/deploy_application",
            _ => return Err("unknown CI/CD participant".into()),
        }
    } else if dining {
        "/service/think and /service/eat"
    } else if coding {
        "/service/design_feature, /service/implement_code, /service/commit_changes and /service/run_local_tests"
    } else if api {
        match agent {
            "DB_ADMIN" => "/service/migrate_schema",
            "BACKEND_DEV" => "/service/implement_backend",
            "TESTER" => "/service/run_integration_tests",
            _ => return Err("unknown API task participant".into()),
        }
    } else {
        "/service/read_literature, /service/draft_content, /service/write_to_document and /service/review_own_work"
    };
    let coordination = if large_document {
        "Each successful read of an incoming file consumes one delivery. Resource temporarily unavailable means that this read has no delivery ready; Permission denied means the attempted operation was refused. Neither message reports task completion. A saved copy in /work can be read again. A new blocking read waits for another delivery on that particular file, even if another file has data. For writers, data.status in the delivered outcome identifies the work. A review can include the reviewer's own comments in data.feedback. A needs_revision result calls for a substantive revision: /service/revise_section/<chapter> takes section_name and optional feedback text. A minor_revision result names exact replacements: /service/apply_minor_edit/<chapter> takes section_name and review_id. A review_id can also identify a major review; its presence alone does not mean minor_revision. A fact_check or review_sections result concerns the identified manuscript. On files that receive the whole-document decision, publication has tool_name combine_sections and success true. A withheld manuscript has a reviews array and no tool_name."
    } else if cicd_pipeline {
        match agent {
            "BUILD_MASTER" => {
                r#"Choose either build order by writing {"first":"frontend"} to /service/build_frontend_first or {"first":"backend"} to /service/build_backend_first. Read /service/qa_report to inspect the actual integration and E2E results. If the first report fails, decide which components to revise using that report and submit {"component":"frontend"}, {"component":"backend"} or {"component":"both"} to /service/repair. Read the new QA report after the revision. The task allows one QA revision cycle. Read /service/result for the final outcome, including a refusal to deploy, then exit 0."#
            }
            "FRONTEND_DEV" | "BACKEND_DEV" => {
                r#"Read /service/start and read /service/workspace each time the build environment is handed to you. Use {"component":"frontend"} or {"component":"backend"} according to your own assignment for compilation and unit tests. Publish your own {"artifact":"frontend_image"} or {"artifact":"backend_image"} after both pass. Read the actual result after each request. If an initial attempt fails, read /service/revise before rebuilding. After publication read /service/build_verified. Stay available for QA by checking /service/qa_revise, /service/build_verified, and /service/qa_done with dd if=PATH iflag=nonblock, since a blocking read of one can prevent handling the other. On a QA revision, receive the workspace, redo compile/test/publish for your component and read its verification again. If that new build fails, the next request arrives on /service/qa_revise. When the other component is revised, a second verification may confirm that your existing artifact is retained. Read /service/result after /service/qa_done and exit 0."#
            }
            "QA_ENGINEER" => {
                r#"Read /service/start before each QA round. Pull the current artifact pair with {"target":"test_harness"} and read that result. Run integration with {"suite":"api_integration"} and end-to-end with {"suite":"user_flows"} in either order. Read both actual results, even if one fails. If the first round fails, a new /service/start arrives after the selected revision; pull again and test the new pair. After the final round read /service/result and exit 0."#
            }
            "RELEASE_ENGINEER" => {
                r#"Read /service/start for the actual QA verdict. If it denies release, read /service/result and exit 0. Otherwise pull with {"target":"staging_deploy"}, deploy {"component":"full_stack"} to staging and read the actual result. If staging succeeds, read /service/staging_ready and request staging smoke with {"target":"staging"}. When staging or its smoke test fails, read /service/abort and /service/result. When both pass, read /service/promote before using the production deployment and smoke files with {"component":"full_stack"} and {"target":"production"} respectively. Read the actual responses and /service/result, then exit 0."#
            }
            _ => return Err("unknown pipeline participant".into()),
        }
    } else if task == "8M" {
        r#"Use the benchmark's object identifiers as JSON field values. For /service/define_api, set "endpoint" to "feature_api_spec". This field identifies the contract object. You can choose an HTTP method in the optional "method" field. For migrate_schema, set "migration_name" to "feature_schema". For implement_backend and implement_frontend, set "component" to "api_endpoints" and "user_interface", respectively. For the matching review file, set "pr_id" to "backend_api" or "frontend_ui". For run_integration_tests, set "suite" to "full_stack_integration".
Choose the work and shell commands for your own role. The API definition and initial schema migration can proceed independently. Backend code uses the actual schema from /service/schema. Before implementing the first backend revision, the backend developer may request an additional migration by writing {"reason":"your concrete reason"} to /service/request_migration. The DB admin reads /service/schema_status for either that request or the implementation that uses the schema. A new migration delivers its actual new schema. The UI uses the approved backend data from /service/backend. A review with status changes_requested lets the developer submit a fresh implementation through their existing operation file. The reviewer receives the new candidate through the same candidate file.
The tester reads /service/test_candidate before each test and reads the real test result from the test operation file. After a failed test, the reviewer receives its report on /service/outcome and chooses the needed correction by writing {"component":"frontend"}, {"component":"backend"}, {"component":"api"}, or {"component":"schema"} to the write-only /service/repair. The DB, backend, and frontend participants then receive {"repair":{"component":...},"test":...} on their own /service/outcome. This decision tells you which existing operation is available for correction. Backend owns API definition as well as backend implementation. A backend-only repair can retain a compatible approved UI. A schema or public API change requires new approval and fresh UI; read the new schema or backend when the corresponding repair is available. Use nonblocking reads or pipeline input to dd of=FILE oflag=nonblock to check availability while others work. Keep working until you receive the actual successful integration result, from your test operation for TESTER or /service/outcome for the other roles. Exit 0 after receiving the successful result."#
    } else if survey_paper {
        "Use the original JSON arguments for each operation. Write them to the operation file and read that same file for its actual result. The four authors can work independently. Each author researches their assigned topic, then drafts and updates their bibliography in either order. Read /service/review_result. If that result rejects your draft, submit revised content through revise_section and wait for the next review result. Once approved, the author is finished. The analyst reads the actual research for a section, chooses its figure description, and reads the returned figure ID. Read that section's insertion_input for the current draft and figure, then submit insert_figure with the actual ID and section name. The reviewer reads a section's review_input, acts through its peer_review file, and uses the outcome to decide whether that section needs another review. Choose which section to work on next. The editor-in-chief reads accepted_sections, supplies their own review summary to make_editorial_decision, and combines all four accepted sections. Reopening a read file receives its next message. After your assigned work is complete, exit 0. The shell supports private work and alternative legal orders. An invalid request returns an error, so it can be corrected."
    } else if research_paper {
        "Each operation file directly projects its typed Endpoint. A request is a complete JSON object written and closed; read the actual response from that file. Use the literal section_a, section_b or section_c in both a section path and its arguments. The authors may gather supplemental research on other topics at any time while their guest is active. After gathering their assigned research, they choose the order of drafting and updating references. They read the next actual review on /service/review. On rejection, authors decide how to revise and may use the actual review to write their own feedback. They finish after editorial approval. The fact checker chooses a section with an available snapshot and checks it through its matching operation file. A later revision, including one after an editorial rejection, brings a new snapshot to that section. The fact checker finishes after receiving editorial approval for all three sections from their /service/approved files. The editor chooses among sections with an available fact-checked snapshot, may reject and later review the revised section, and completes by receiving the result of combine_sections for all three sections. A blocking read can wait for someone else. When waiting for any new snapshot or approval, try wash's dd if=FILE iflag=nonblock on the appropriate files and pursue any available work. Save results you need after consuming a file. A recoverable shell error permits corrected or independent work."
    } else if kitchen {
        "Each operation file owns its actual projected Endpoint. Choose shell commands, private computation and the order of your base/ingredient work freely. Submit the original assigned arguments and read the actual response from that same file. Ordinary redirection is blocking. For nonblocking submission, wash supports pipeline input to dd of=FILE oflag=nonblock; an unprepared operation returns a WASI error so you can choose independent work or retry. The actual oven and, for chefs A/B, stovetop Endpoints carry each base request/result and return. Ingredient preparation needs no equipment; the completed material is delivered directly to its assigned recipient. Combine your dish after its own base and required incoming ingredient exist. Your finished dish need not wait for unrelated bases or your outgoing ingredient. Each assigned effect runs once. Complete all your own assigned work and read its actual results before exiting. Do not invent acquire/release or notification calls. The common world grades completed material inputs and physical conflicts, without prescribing a total command order."
    } else if manufacturing {
        "Each operation file owns its actual projected Endpoint. Choose your shell commands, private computations and assigned job order freely. Submit the original do_job arguments with your assigned job_id to do_lathe_job or do_mill_job and read the actual result from that same file. Ordinary redirection is blocking. For a nonblocking submission, wash supports pipeline input to dd of=FILE oflag=nonblock; an unprepared request returns a WASI error without starting that job. You can then choose the other assigned machine or retry later. Independent machines can work concurrently. Actual workstation Endpoints transfer through the choreography and are returned after use; do not invent acquire/release or notification calls. Actual completed job results reach the supervisor directly. The supervisor submits {} to collect_reports and reads the actual response from that same file; the actual collection waits for all four completed job results. Finish after receiving your actual assigned results. The common world grades actual completed report contents and machine conflicts, not a prescribed total command order."
    } else if task == "10M" {
        "Each operation file is a direct choreography projection. Choose shell commands and private computation freely. An unadmitted request returns a recoverable WASI error. Each builder uses compile_module for its assigned module and reads the actual result from that same file. Its read-only updates, test_result, reconsideration and outcome files deliver actual shared-input changes, its test result, reconsideration data, and the final linked outcome respectively. Independent file reads remain available; an unavailable operation is an error rather than permission to invent an effect. A reconsideration names its module and actual old object/test; the builder can compile a fresh object against current inputs. Reopening an incoming file consumes its next actual message. TEST_RUNNER reads next_object for an actual compiled object, submits its original run_build_tests arguments through that module's run_build_tests_a, run_build_tests_b or run_build_tests_c file, and reads the actual test result from that same file. next_object eventually delivers the final linked outcome. The integrator can read either /service/rebuild for current objects, tests and eligible modules or /service/candidate for a final verified candidate. Use nonblocking reads when waiting for either. Choose any module listed under rebuild and write its plain name to /service/rebuild. An ineligible choice returns the same offer with an error on the next read. When the final candidate arrives, request link_modules for its three modules and read its actual result; publication requires current positive tests and return of all editing Endpoints. Real resource transfers and message delivery follow the global; do not invent acquire/release or notification tools. Work independently and concurrently where permitted; use actual outcomes for reconsideration rather than a fixed attempt count. Exit after your actual final linked outcome. The common world checks actual object versions, test inputs and linked bytes, not a prescribed invocation order."
    } else if build {
        "The two actual resource Endpoints transfer automatically; no acquisition, release or notification commands are needed. Compile results contain the actual object bytes and shared type inputs. A updates the provided SHARED_TYPES definitions. B's read-only /service/updates delivers A's actual completed compile result and changed types. A blocking read can wait for that actual incoming message; nonblocking shell I/O can inspect availability instead. B may initially compile against the provided old definitions: that object is real but becomes stale when A updates them. Use the received update and actual object inputs to decide whether B needs a rebuild through the same compile service. A current object's SHARED_TYPES.modules includes module_a's declarations. The integrator's link waits for both current actual build reports. Private commands and computations remain your choice, and the common world scores the actual object and link inputs rather than a prescribed total invocation order."
    } else if figure_document {
        "Read the actual results of each operation. Writers can prepare references independently and use read_section to obtain the actual content and version of another chapter. The writers read /service/outcome for the next actual result. A fact_check or review_sections rejection requests revision; combine_sections with success gives the final artifact ID. Keep independent work available by checking notice files with nonblocking reads. The figure creator creates figures for chapters 1 and 3 in either order. Use each chapter's create_figure file for the description and read the actual figure ID. Once that chapter is written, submit the ID and matching section_name to that chapter's insert_figure file. A writer can choose which assigned chapter to write first. The fact checker reads /service/outcome for the next actual work or final result. If it is a map of scopes to material, submit the fact_check request by writing the section_name JSON to each scope's operation file and then read its reply. Both initial scopes may be checked in either order. After a revision the map contains only the changed scope. Reopening outcome waits for the next occurrence; save any received material privately to read it again. The editor reads review_material, reviews the current document with its actual fact receipts, and assembles the document after approval. Writers and the fact checker stay until /service/outcome delivers successful combine_sections. An empty or unavailable nonblocking read is temporary and does not end that role. The figure creator can exit after reading both successful insertions. The editor can exit after reading the successful combination result. Invalid or premature operations return shell errors. Use the real results to decide when to continue or try independent work."
    } else if document {
        "Each file is a direct choreography projection. The editor chooses the first writer through allocate_document with {\"first_writer\":\"WRITER_A\"} or WRITER_B. Writers' initial write and references target their assigned chapter; these operations are independent. Both writers may revise either chapter when editing is requested. Reading feedback consumes the next actual reconsideration review or the final combine_sections result; a review scope identifies the assigned writer. Both positive and negative current reviews may be reconsidered before freezing. The editor submits original review_sections arguments to review_chapter_1 or review_chapter_2 and reads the actual result from that same file. Either review order is allowed, including submitting both before reading results. Each successful reopen of an incoming file consumes its next message; private files can retain actual results. To request editing, reconsider_a or reconsider_b takes {\"review_id\":\"the actual current review_id\"}; its response is the actual revised material. Fresh reviews refer to the resulting complete document. combine_sections is the actual publication request and takes the original two-chapter sections argument. Publication requires both actual current reviews to be successful and match current material. The choreography transfers editing authority before the effect; there is no separate approval-submission operation. Writers remain active for reconsideration until feedback delivers the actual combine_sections publication result; completing the initial write and references is not their completion condition. Exiting before that result aborts the run. The editor remains active until it receives the actual publication result from combine_sections. Choose your own commands, calculations, legal request orders and reconsiderations. A refused unadmitted request returns an error to the same guest; no host chooses a next command or workflow step."
    } else if staged_integration {
        "Each file directly projects its actual choreography Endpoint. Choose shell commands, private calculations and legal independent request orders freely. Write original assigned arguments and read the actual response from the same operation file. The three feature names are user_authentication, payment_processing and notification_system. Developers choose their own source for implement_feature and read actual peer candidates from interface/<other-feature>. Test writers read candidate/<feature>, write_tests/<feature> and read its result. Reviews require those actual completed tests; candidate/<feature> supplies the reviewer both source and tests. Review rejection permits a fresh implementation, test and review. Developers and TEST_WRITER receive review/<feature> results for all three features, not just their own. These are actual incoming operations, not optional status snapshots: reading your own approval does not consume either peer's review. Any developer with a current approval may submit merge_code as soon as their feature is ready, including while another feature is still being prepared. The merge and CI order is independent of deployment order. Staging requires successful validation of A before deploying B, and of B before deploying C. A blocking read of publication/<feature> waits for that particular feature. When the first merge is unknown, dd if=/service/publication/<feature> iflag=nonblock reads an available publication or returns immediately. Its successful output is the publication itself. CI_RUNNER uses that feature, submits the assigned suite through run_tests/<feature>, and reads its real result. CI failure requires fresh source, tests, approval, merge and CI. RELEASE_MANAGER reads release/<feature> for the actual publication and positive CI, deploys to staging, then validates that actual deployment. Failed validation requires rollback of that deployment followed by a fresh implementation/test/review/merge/CI/deploy/validate attempt. Rollback uses the original service argument. Deployment environment may be omitted or staging. Other agents receive integration/<feature> and release/<feature> for all three features' actual CI and validation results, including failed attempts. A later stage result does not replace its earlier CI result. The same feature may repeat; do not assume retry counts. Incoming files consume their next actual message on each successful reopening. A refused operation does not consume its missing prerequisites: trying that same request repeatedly cannot replace reading incoming results. Stay available for all three final positive staging results and the actual candidate/review/integration updates directed to you; then exit. RELEASE_MANAGER finishes after all three actual successful validations. Unadmitted invalid requests return recoverable shell errors. No acquire/release, grant, synthetic notification or generic result mailbox exists. The common world scores actual completed material and resource effects independently of this protocol."
    } else if feature_delivery {
        "Each operation file is a direct choreography projection. Choose shell commands, private computations and legal request orders freely. Write the original assigned JSON arguments and read the actual result from that same operation file; there is no generic result mailbox. Developers implement their own feature, read actual review feedback from review, and can choose fresh source after rejection. interface_change and peer_review deliver the other feature's actual candidate and review. The reviewer chooses either target through code_review/<pr_id>; code_ready/<feature> supplies its actual candidate. The test writer independently chooses either target through write_tests/<target>, after reading the actual candidate in code_ready/<feature>; review/<feature> carries the actual review result. Rejected reviews require a fresh implementation, test definition and review of that feature. merge_ready delivers the feature's actual approval. Either developer can merge and run CI on their approved feature even before the other developer starts work. A failed CI must be repaired before the next feature is merged. A blocking read of publication/<feature> waits for that particular feature. When the first merge is unknown, dd if=/service/publication/<feature> iflag=nonblock reads an available publication or returns immediately. Its successful output is the publication itself. CI_RUNNER uses that feature, submits the matching assigned suite through run_tests/<feature>, and reads its actual CI result there. integration delivers actual CI results: each developer and the test writer have one stream, and the reviewer has one file per feature. CI failure requires the author who merged to revise, retest, obtain a fresh review, merge that repair and run CI again before the next feature can merge. Each successful reopening of an incoming file consumes its next message. Use the actual target, candidate and result data; do not assume a retry count or invent acquire/release or notification tools. Stay available to receive both features' final successful CI results and actual peer updates before exiting. Invalid unadmitted input returns a shell error so it can be corrected or retried. The neutral common world grades actual effects and data, not this protocol's schedule."
    } else if critical_care {
        "Each operation file is a direct projection of its actual Endpoint. Choose commands, private calculations and legal request orders freely. The ER doctors use their own patient assessment and test panel, read actual findings, submit diagnose and read its actual response. Panels can be combined, split, reversed or repeated while incomplete. Each ER doctor finishes after reading their own actual final record from /service/summary. It contains the diagnosis, findings, surgery or ICU outcome and prescription review. If a nonblocking read reports that the summary is temporarily unavailable, keep the guest open until you have received it. The imaging owner obtains only the requested patient's actual assessment through the global. Use the original imaging JSON schemas with the file's patient/modality suffix: _ct_a, _ct_b or _mri_b. Either patient's CT may start first, each interpretation requires its own actual image, and CT and MRI remain independent. Treatment participants can read the actual diagnosis from their diagnosis file. Anesthesia's returned operating-room Endpoint and actual preparation pass directly to surgery; ICU requires both actual bed and ventilator requests. PHARMACIST handles the original patient_A_and_B review. Optional symptoms, medications and anesthesia type remain caller text. Read the response from the same operation file; a rejected unadmitted operation returns a recoverable error. Resource loans and returns follow the global without acquire/release or notification commands. The neutral common world scores actual input causality and physical conflicts, not a total command sequence. This abstract environment has no medication-administration, monitoring, discharge or conservative-treatment tool, so these runs cannot certify those absent effects. It has no injected diagnostic failures."
    } else if multi_specialty {
        "Each file is a direct projection of an actual Endpoint. Choose shell commands, private calculations, panel requests and imaging interleavings freely, using your assigned patient and actual results. Submit the original operation arguments and read the response from the same file. Panels permit combined, split, reversed and repeated incomplete test requests. The specialists' referral files contain the actual assessment; findings contain general, own and other test receipts and the completed CT/MRI reports. For RADIOLOGIST, each _ct or _mri operation file belongs to that actual scanner Endpoint: use the original perform_imaging/interpret_imaging argument schema with the matching modality. The read-only ct_order and mri_order files deliver the actual referrals, and each interpretation depends on its own actual image. CT and MRI are independent. The ER doctor's candidate contains the actual completed workup, both diagnoses and medication review for its case-conference operation. Optional symptoms, medications and attendees remain caller-supplied strings. After the consultation is published, the doctors and radiologist read its actual final record from summary before finishing; PHARMACIST finishes after its own independent review response. Real resource requests, returns and record transfers follow the global; do not invent acquire, release or notification tools. Invalid unadmitted data returns a recoverable shell error. The common world scores actual completed inputs and physical conflicts, not a prescribed total command order. This abstract benchmark has no prescribing or drug-administration effect and does not establish clinical correctness."
    } else if medical {
        "Choose your private shell commands and calculations freely, using actual patient results and your assigned operations. Each operation file owns its real Endpoint: write the operation JSON and read the actual response from that same file. ER_DOCTOR assesses the assigned patient and chooses how to interleave CT with general-panel requests. Either panel can use combined, split, reversed or repeated incomplete test_type requests. CARDIOLOGIST can read the actual assessment in /service/referral and the completed general findings in /service/findings; use actual results for diagnosis. ER_DOCTOR can read the completed consultation in /service/summary, containing clinical.findings, clinical.diagnosis and medication_review. PHARMACIST independently reviews caller-supplied medication data. medications and symptoms are optional strings, not fixed answers. Actual CT and test-panel Endpoints carry each resource request and result; do not invent acquire, release or notification tools. Invalid input returns an error without authorizing an effect, so correct it or choose another legal operation. Other participants can continue while a request waits. Finish your assigned work after reading its actual result. This abstract benchmark has no prescribing or administration effect."
    } else if report {
        "Hibana delegates the DOCUMENT or DATABASE Endpoint for each shared operation. The actual request and result travel through that Endpoint before it returns. You do not call resource acquisition, release or notification tools. Researchers can write their own section and update its references in either order after researching their topic. Read the editor's actual review from /service/feedback. After a rejection, revise your section and read its next review there. A researcher can finish once their section is approved. The editor can review any submitted section with its references, even if the other researcher has not started. A review can overlap revision of the other section. Read the actual results and combine both sections after both pass."
    } else if multi_round_discovery {
        "Each operation file is directly bound to an Endpoint. You choose your commands, private calculations, operation timing and revisions. For a path with an operation and a suffix, the suffix is the literal selector, not a description of the material. For example, /service/run_hplc_analysis/synthesis_qc takes {\"sample\":\"synthesis_qc\"}, and /service/run_mass_spec_analysis/molecular_id takes {\"sample\":\"molecular_id\"}. Use the same suffix rule for sample, assay_type and review_type on every such path. /service/review_clinical_data takes {\"data_type\":\"combined_review\"}. CHEMIST can start synthesis QC, BIOLOGIST can prepare cultures, and TOXICOLOGIST can establish baseline markers independently. FORMULATION_SCIENTIST receives the actual compound on /service/study; use its compound.data.compound string. BIOLOGIST and TOXICOLOGIST receive the new formulation on /service/study. The biologist can independently submit bioassay and stability. Toxicity depends on current purity and baseline. The three biological sample units are shared across all rounds. Failed attempts consume units, culture preparation uses none, and an exhausted assay is real negative evidence. When satisfied with the current assay outcome, close /service/submit_evidence with {\"evidence\":\"bioassay\"} or {\"evidence\":\"cytotoxicity\"} as appropriate. CLINICAL_LEAD receives the current inputs on /service/study. REGULATORY_SPECIALIST receives the actual clinical assessment there and chooses initial_review or hold_review. Another routine review cannot erase a hold on unchanged evidence. The regulator can reconsider before closing /service/submit_evidence with {\"evidence\":\"regulatory\"}; evidence submission is write-only and refers to the current real result. PROJECT_DIRECTOR receives the submitted regulatory result on /service/study and chooses continue after approval, retry for a new round, or abort for a supported negative outcome. A new round starts with a new formulation and the stock that remains. The director can finish after its terminal decision reply. CHEMIST receives the terminal decision on /service/outcome and finishes after successful documentation. Other scientists remain available until /service/study delivers the terminal decision. Read-only files deliver the next actual message on reopening; save the data privately if you need it again. A blocking read may wait for other participants; consider your available work first."
    } else if regulatory_discovery {
        "Choose your wash commands and private work freely. An operation file takes the assigned JSON arguments and returns the actual operation result; reopening an incoming file consumes its next message. Use a successful current receipt for each prerequisite. An injected_failure is an actual negative result and permits retry, while a permission error means the endpoint did not admit that operation at this time. Read-only compound delivers the actual compound to the two evaluators. Each evaluator reads study after submitting their own initial analyses and after any requested reanalysis. It delivers an actual approval or a reanalysis request directed to you. A blocking read can remain pending through one or more reviews until one of those messages arrives. Stay available until approval. Redo your two assigned compound analyses when requested; previously completed evidence is reused for participants with no new request. The lead reads the actual bioactivity and toxicity reports and independently decides go or no-go after structural confirmation and characterization. The regulatory participant reads that actual decision from decision and uses its assigned compliance argument. Lead reads the actual result from review. After rejection, the lead decides whether to request additional biology, toxicology, both, or neither, by writing a JSON object with the boolean fields biology and toxicology to reanalysis. Reanalysis is write-only. Closing it successfully submits the choice. Read fresh reports only from the scientists you selected. Keep the existing report for each unselected scientist. Each revised decision uses those current reports and a fresh structural confirmation. The chemist waits for the actual approval before documentation. No manual notifications or resource requests are needed. The evaluators and regulator can finish after approval; the lead can finish after the positive review, and the chemist finishes after successful documentation. Use nonblocking reads when another independent action is available."
    } else if drug_discovery {
        "Each operation file consumes its own typed request and returns the actual result. Use the assigned sample or assay_type value. The lead receives both scientists' actual reports and chooses go or no-go from the results using a boolean go. Samples and the decision are received through their read-only files. Initial synthesis QC, cell preparation and mass-spectrometer work can overlap. The lead's two final analyses can be submitted independently before reading their results. An unadmitted request can be corrected or retried. Read your assigned actual results before exiting."
    } else if lab_cascading {
        "Choose your shell commands, private work and legal independent operation order freely. Files project the actual typed Endpoints. Each submitted request is one complete JSON object. Replace the file contents for a new request, including a new attempt, because appending can leave two objects and be rejected. Read the response from that same file. Reading an available message consumes that occurrence, so keep private copies of data you still need. A shell error means this operation did not return the requested data. It is recoverable, and you may choose another operation or retry when the dependency is ready. Use only real completed artifact IDs and current material. The biologist publishes a completed assay to the analyst and QC with {artifact_id}, in either order. When QC is first and rejects the assay, no chromatography is needed for that assay. The analyst’s separation verification belongs to the current bioassay. A new batch needs its own verification, even if the analyst already verified a rejected batch. Verification can run before or after chromatography and molecular analysis. For BIOLOGIST and ANALYST the read-only /service/batch file yields the next actual material or QC result. Material contains a synthesis record, with the bioassay also present for ANALYST. CHEMIST and QC_CONTROLLER receive review notices from /service/qc. A failed synthesis is reported on /service/run_synthesis to the chemist. It permits another synthesis attempt, which may be temporarily refused while the other participants return instrument access. A rejected QC review is reported on /service/qc to the chemist and also permits a new synthesis. An approval applies to its current batch. The QC controller can finish after its actual positive verify_result reply. The biologist can finish after that batch is approved and both publications have succeeded. The analyst can finish after its current-batch analyses and the positive notice on /service/batch confirming that its report was delivered. Only the chemist must continue to final_validation. The chemist can finish after its final_validation reply or a reagent_exhausted synthesis reply. On exhaustion, the other scientists receive the actual reagent_exhausted result on their batch or QC files. A blocking read can wait for another participant; when deciding between a read and other available work, wash supports dd if=FILE iflag=nonblock. The safety officer may finish after its own successful inspection result."
    } else if lab_quality {
        "Each operation file directly projects its actual typed Endpoint. Choose shell commands and private calculations freely; write the original operation's JSON arguments and read the actual response from that same file. Batch names and optional culture text are ordinary data. CHEMIST synthesizes a compound, analyzes synthesized_compound and purifies compound_purification, then reads each actual QC result from /service/qc. A rejected chemistry review requires a new synthesized_compound analysis of that same batch, not a new synthesis. After approval, read /service/quality and perform final_validation. BIOLOGIST prepares cell_cultures, calibrates fluorescence, reads /service/sample and tests the actual synthesized compound with run_bioassay. An invalid batch returns invalid_arguments without a physical trial; bioassay_failed is a real negative trial and permits another. A negative QC result from /service/qc requires a fresh bioassay; a positive result finishes that scientist's guest work. ANALYST reads the actual approved batch/trial from /service/bioassay, then independently performs molecular_detail and verification against those inputs. Both instrument operations can be submitted before reading their results. QC_CONTROLLER chooses either scientist by requesting verify_result/CHEMIST with scientist CHEMIST/data_type purification or verify_result/BIOLOGIST with scientist BIOLOGIST/data_type bioassay. It can submit the chosen review while the material is still being prepared; the result waits for actual data. It may switch subjects after rejection or continue with the rejected subject, and finishes after both actual approvals. Do not assume retry counts or wait for an invented readiness notification. Within this finite source, initial preparation has two groups and quality starts after both approvals; the neutral world does not impose those as its preferred order. File reads consume the next actual message and may wait if it has not arrived. Invalid unadmitted requests return a recoverable shell error; use actual results to reconsider. No acquire/release, generic result mailbox, host-selected command or manual notification exists. The choreography moves candidate authority into QC during review and retires an approved candidate before further reconsideration. Finish after your own required actual results."
    } else if lab {
        "Each operation file directly projects its actual typed Endpoint. Choose shell commands, private calculations and legal independent operations freely. Write original assigned JSON arguments and read the actual response from the same operation file. CHEMIST analyzes raw materials, purifies that actual compound, reads /service/quality and then submits final_validation for that same specimen. BIOLOGIST prepares cells, calibrates fluorescence for those cells, reads /service/sample and runs the bioassay on that actual compound. Optional cell_type is arbitrary string data; it may be omitted. ANALYST reads /service/bioassay and performs both molecular_detail and verification against that same actual assay; those instrument operations are independent and may be submitted before waiting for their results. Cell preparation/calibration and raw analysis/purification are independent chains; do not wait for another scientist before starting your own available work. The actual spectrometer and centrifuge Endpoints reach their I/O owners through the global; each effect consumes the real resource request before returning its result. There is no acquire/release, generic result mailbox, host-selected command or manual notification. Reading an incoming file consumes its actual message. An early, duplicate or malformed request returns a recoverable shell error, so reconsider and choose another legal action. Finish after your own actual work and results. The common environment checks completed scientific material and physical resource conflicts, without enforcing this implementation's command order."
    } else if refactor {
        "Each operation file is directly bound to its actual choreography Endpoint. Choose your own shell commands and private computation. Submit the original JSON arguments and read the actual response from that same file. The architect's two proposals and the developers' own-module/shared-library changes can be ordered independently. Each module can be reviewed, tested and published as soon as its own developer's two changes are complete. Review and test can run in either order, even while the other module is untouched. A developer reads its actual review and test_result before publishing through merge_code. There is no acquire/release or readiness command. An unadmitted request returns a WASI error, permitting a corrected or independent operation. Complete and read your assigned results before exiting. The common world scores published contents and completed matching validations, without enforcing this procedure."
    } else if coordination {
        "Each service file is a choreography projection. Choose shell commands freely: a refused operation returns an error, so you can correct it or try another operation. BACKEND_DEV defines the assigned API through /service/define_api. FRONTEND_DEV reads its actual contract from /service/agree_api and chooses either implementation order by writing {\"api_name\":\"feature_api_spec\",\"first\":\"BACKEND_DEV\"} or {\"api_name\":\"feature_api_spec\",\"first\":\"FRONTEND_DEV\"} to that file. BACKEND_DEV receives this agreement by reading /service/outcome. Each developer's /service/implement_feature runs its assigned implementation using the actual repository Endpoint. The backend may redefine the API and collaborate on revised implementations before testing. TESTER reads the current candidate from /service/run_tests, submits its assigned suite to the same file, and reads the actual test result there. Testing requires actual REPO and TEST_ENV requests; those resources exclude editing during the effect. Each developer reads the actual final test result from /service/outcome. Reopening an incoming file reads its next actual message. Finish after the required actual work and results; no host command sequence, explicit acquire/release or finalization tool is required."
    } else if review {
        "Each operation file is a direct choreography projection. Choose private shell commands, calculations and legal request orders freely. Developers submit their assigned original implement_feature arguments and read the actual implementation response from that same file. The read-only /service/review stream delivers your actual review result, identifying its target in data.target. Submit your initial implementation freely, then read the review result. After rejection, choose revised source and submit a fresh implementation; after approval, submit the assigned merge_code request and read its actual result from the merge file. The reviewer submits original pr_id arguments to /service/code_review/<pr_id> and reads its actual result from that same file. Choose either initial target and continue reviewing both features until each is approved; after a rejection you may either repeat that target or switch while its developer revises. Reviewing one feature can overlap independent implementation or merging of the other. One participant owns the actual repository Endpoint and receives each typed file request before its effect. Each review file owns a separate Endpoint; the actual response identifies the reviewed candidate. No host dispatches files or chooses your next command. An unadmitted or invalid operation returns a recoverable shell error. A normal read may wait for data; nonblocking reads and writes let you choose another operation. There is no generic result mailbox, acquire/release or notification command. Finish after your own assigned actual work and results."
    } else if cicd {
        "Your operation file is a choreography projection. Choose shell commands freely; an unadmitted operation returns an error so the same guest can correct it or try another operation. Reading that same file receives an actual message with {kind,result}. DEVELOPER receives compile_result after its operation, then deployment_complete. TESTER receives build_complete, submits its own test request, receives test_result, then deployment_complete. DEPLOYER receives tests_passed, submits its own deployment request, and receives deploy_result. Each successful reopen reads the next actual message. Every result contains the actual shared-environment response. No host chooses your commands or lends a permission in advance; the file close itself is the request. The real build-server or deployment-environment Endpoint carries the exact admitted arguments and actual result before returning. You do not call acquire/release or notification tools. Finish after your assigned work and required actual notifications. This original task has no injected domain failures or business retry loop; malformed or premature shell operations remain recoverable."
    } else if task == "9E" {
        "You may think again before eating. You choose whether to do so. You may request your meal as soon as your own thought has finished, even if the others are still thinking. Eating uses both your fork Endpoints and returns them afterwards. Your philosopher argument is display data. Read each result from the file you used."
    } else if dining {
        "You may think again before eating. Choose how many times to think. Eating uses both your fork Endpoints and returns them afterwards. Your philosopher argument is display data. Read each result from the file you used."
    } else if task == "1H" {
        "Each operation file owns its actual projected Endpoint. Choose your shell commands and private computations freely. A refused operation returns an error to the same guest, permitting correction or another operation. Design and implement your assigned feature. Committing consumes both actual module requests and returns both module Endpoints before testing. The native localsides drive those Endpoints directly; no host dispatches requests or chooses your next command. Read each actual response from the same operation file. A negative test contains the actual tested source and diagnostic. Reconsider, implement the repair, commit it and test again. Any developer may commit first when their two modules are available, and developers with disjoint modules can commit concurrently. Finish after your actual tests pass. No acquire/release or notification tools are needed."
    } else if coding {
        "Each operation file owns its actual projected Endpoint. Choose shell commands and private computations freely. A refused request returns an error so that you can correct it or choose another operation. Design and implement your assigned feature. The localsides drive both actual module Endpoints for the commit and return them before testing. Read the actual result from the same operation file; no host dispatches requests or chooses your commands. The actual test outcome is terminal, including failure; finish your own work after reading it. No acquire/release or notification commands are needed."
    } else if api {
        "Hibana passes the database or test-environment Endpoint to the WASI participant. The request and actual result travel through that Endpoint before it returns. Migration completion precedes backend work, and backend completion precedes integration testing. You do not call acquire/release or notification tools. Use your assigned arguments. DB_ADMIN and BACKEND_DEV must also read the read-only /service/outcome object, which contains the original test result. That read may wait until the test finishes while other participants continue. A failed integration test is terminal in the task description and simulator. Report its actual result and finish, without adding a revision or retry."
    } else {
        "Each file is a direct choreography projection. Choose shell commands and private computations freely. An unadmitted operation returns an error to the same guest, allowing correction or another operation. read_literature returns actual source material; draft_content uses that completed material; write_to_document uses your actual draft and both assigned section resources; review_own_work examines that actual contribution. The localsides drive the real section Endpoints and return the actual results. No host selects your commands or dispatches accepted files to another workflow. Read each actual response from the same operation file. Use your assigned topic, section or resource pair. Do not invent acquire/release or notification commands. The actual review outcome is terminal, including failure; finish your own work and exit after reading it."
    };
    let response_object = if cicd_pipeline {
        "Read a domain operation's actual reply from the same file you wrote. For example, after closing /service/compile_code, read /service/compile_code. The master's build-order and repair files are write-only. /service/result carries the final pipeline outcome. A successful read of the developer's /service/start or /service/workspace or the QA engineer's /service/start is a notification with empty output. Each read consumes that notification once."
    } else if large_document {
        "Operations with a readable reply return it on the file you wrote."
    } else if review
        || medical
        || multi_specialty
        || critical_care
        || kitchen
        || dining
        || coding
        || api
        || task == "8M"
        || cicd
        || feature_delivery
        || staged_integration
        || document
        || figure_document
        || report
        || research_paper
        || refactor
        || lab
        || lab_quality
        || lab_cascading
        || drug_discovery
        || regulatory_discovery
        || multi_round_discovery
        || task == "4E"
        || family == 2
        || build
        || manufacturing
    {
        "Read the same operation object for its actual response."
    } else {
        "Read /service/result for the actual response."
    };
    Ok(format!(
        r#"You are {agent}. Carry out your work in the original task below.
{description}

Your task input data: {data}
Your original domain operation schemas: {schemas}

You have one tool, shell. It runs your own scripts in the Rust std WASI wash
guest. All file I/O uses Hibana Endpoints and ChoreoFS. There is no host
filesystem or native process execution. Use help to see shell commands and
syntax. You may use variables, conditions, loops, pipes, redirection, expr,
private files and saved scripts. Choose your own commands and calculations.
Normal reads may wait for data, including work you have not yet requested.
While a shell command waits, that same guest cannot run another command.
`dd if=PATH iflag=nonblock` requests a nonblocking read: if no message has arrived
it returns a nonzero status without consuming a message, so you may choose
another file or retry. It grants no operation rights.
Private /work files have names you choose, with eight slots bounded by the configured guest memory profile.
A script has at most 4096 UTF-8 bytes. Each participant has its own persistent
guest and private files.

{files_heading} {objects}.
Write the matching operation's JSON argument object and close it to request
the operation. Shell redirection closes its output after writing.
Argument files contain one complete JSON object. Shell quoting does not encode
JSON for you: single quotes preserve their contents, `>` replaces bytes, and
`>>` appends bytes. Private files may assemble larger input from several commands;
each command still has the script-size bound. Appending after a closed JSON
object does not extend a string inside it. Read the reported operation and path
when correcting an error; a close error is distinct from an open or read error.
{response_object} These objects are not commands.
{coordination}
Independent participants and domain operations run concurrently. A request
waiting for its dependency does not block the other guests.

Ordinary shell errors and invalid argument data can be corrected. Use actual
responses to reconsider your work. When your assigned work is finished,
close your guest with exit 0 before your final response. After an exit_status
reply the guest accepts no further commands. If any shell calls were interrupted,
collect their saved replies with empty shell calls ({{}}) until the response is
no_interrupted_command. Collection also works after exit. Leaving before doing
the work fails the run.
"#,
        files_heading = if large_document {
            "Your role has these service files."
        } else {
            "Your writable domain objects are"
        },
        description = fs::read_to_string(input.join("description.md"))?,
        schemas = serde_json::to_string(&schemas)?
    ))
}

fn trial_error(outcomes: &[Value]) -> Option<String> {
    outcomes
        .iter()
        .find_map(|outcome| match outcome["kind"].as_str() {
            Some("model_participant") => {
                let result = &outcome["result"];
                (result["status"] != "completed" || !result["error"].is_null()).then(|| {
                    format!(
                        "model participant {}: status={}, error={}",
                        result["participant"], result["status"], result["error"]
                    )
                })
            }
            Some("guest_runtime") => {
                let result = &outcome["result"];
                (!result["error"].is_null()
                    || result["runtime_exit_code"] != 0
                    || result["protocol"]["status"] != "complete"
                    || result["backend_errors"] != json!([]))
                .then(|| {
                    format!(
                        "guest runtime failed: exit={}, protocol={}, error={}, backend_errors={}",
                        result["runtime_exit_code"],
                        result["protocol"],
                        result["error"],
                        result["backend_errors"]
                    )
                })
            }
            _ => Some(format!("trial job failed: {outcome}")),
        })
}

// A failed peer can end live I/O. The other owner still finishes its report
// and the guest owner drains all accepted simulator effects before scoring.
#[hibana_runtime::scope]
async fn record_outcomes(
    output: &Path,
    (guest, runtime): (&process::Guest, impl Future<Output = Result<Value>>),
    participants: impl AsyncFnOnce(&mut hibana_runtime::host::Io) -> Result<Vec<Value>>,
) -> Result<()> {
    let (failed, mut guest_failure) = hibana_runtime::host::notification()?;
    async {
        let report = match runtime.await {
            Ok(value) => json!({"kind":"guest_runtime","result":value}),
            Err(error) => json!({"status":"harness_error","error":error.to_string()}),
        };
        save(&output.join("guest-outcome.json"), &report)?;
        if trial_error(std::slice::from_ref(&report)).is_some() {
            failed.notify()?;
        }
    };
    async {
        let reports = match participants(&mut guest_failure).await {
            Ok(results) => results
                .into_iter()
                .map(|result| json!({"kind":"model_participant","result":result}))
                .collect::<Vec<_>>(),
            Err(error) => vec![json!({"status":"harness_error","error":error.to_string()})],
        };
        save(&output.join("model-outcomes.json"), &json!(reports))?;
        if let Some(error) = trial_error(&reports) {
            guest.stop(error)?;
        }
    };
    Ok(())
}

async fn run(task: &str, depth: u8, output: &Path) -> Result<Value> {
    if depth > 2 {
        return Err("expected depth 0..2".into());
    }
    fs::create_dir_all(output.parent().ok_or("output parent")?)?;
    fs::create_dir(output)?;
    let mut versions = snapshot::freeze(task, output)?;
    versions["model_connection"] = model::identity()?;
    save(&output.join("versions.json"), &versions)?;
    let config: Value = serde_json::from_slice(&fs::read("experiment.json")?)?;
    let seconds = config["run_timeout_seconds"]
        .as_u64()
        .ok_or("run timeout")?;
    let budget = config["participant_tool_call_limit"]
        .as_u64()
        .ok_or("tool budget")?;
    let trace = Trace::create(&output.join("events.jsonl"))?;
    let started = Instant::now();
    let (simulator, fixture) = process::simulator(task, depth, output, trace.clone()).await?;
    let (guest, guest_loop) = process::guest(
        &output.join("runtime").canonicalize()?,
        &output.join("wash.wasm").canonicalize()?,
        &[task],
        output,
        trace,
        simulator,
    )?;
    let mut people = Vec::new();
    for actor in fixture["fixtures"].as_array().ok_or("actor fixtures")? {
        let agent = actor["agent"].as_str().ok_or("actor name")?.to_owned();
        let instructions = prompt(Path::new("."), task, actor)?;
        fs::write(output.join(format!("{agent}-prompt.txt")), &instructions)?;
        people.push(model::Participant {
            output: output.join(&agent),
            agent,
            instructions,
        });
    }
    let finished = race(
        record_outcomes(output, (&guest, guest_loop), async |guest_failure| {
            model::run(
                people,
                model::Limits {
                    timeout: Duration::from_secs(seconds),
                    tool_calls: budget,
                },
                &guest,
                guest_failure,
            )
            .await
        }),
        deadline(Duration::from_secs(seconds + 20)),
    )
    .await;
    // These files are experiment reports, including partial reports after a
    // timeout. They never decide which participant or Endpoint may progress.
    let mut outcomes = Vec::new();
    match fs::read(output.join("guest-outcome.json")) {
        Ok(bytes) => outcomes.push(serde_json::from_slice(&bytes)?),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    match fs::read(output.join("model-outcomes.json")) {
        Ok(bytes) => outcomes.extend(serde_json::from_slice::<Vec<Value>>(&bytes)?),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    save(&output.join("outcomes.json"), &json!(outcomes))?;
    let error = finished
        .err()
        .map(|error| error.to_string())
        .or_else(|| trial_error(&outcomes));
    let result = json!({"task":task,"depth":depth,"kind":"unscored_direct_wasi_model_preflight",
        "scored_benchmark":false,"error":error,"elapsed_seconds":started.elapsed().as_secs_f64(),
        "versions":versions,"outcomes":outcomes,"events_sha256":evidence::digest(&output.join("events.jsonl"))?});
    save(&output.join("result.json"), &result)?;
    Ok(result)
}

#[hibana_runtime::main]
async fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [mode, output, families] if mode == "--build-wasi" => println!(
            "{}",
            serde_json::to_string_pretty(&qualification::build_only(Path::new(output), families)?)?
        ),
        [mode, output] if mode == "--qualify-wasi" => println!(
            "{}",
            serde_json::to_string_pretty(&qualification::run(Path::new(output), None)?)?
        ),
        [mode, output, filter] if mode == "--qualify-wasi" => println!(
            "{}",
            serde_json::to_string_pretty(&qualification::run(Path::new(output), Some(filter))?)?
        ),
        [mode, events] if mode == "--audit-philosophers" => println!("{}", serde_json::to_string_pretty(&philosophers_audit::audit(Path::new(events))?)?),
        [mode, events, output] if mode == "--check-philosophers-mutants" => println!("{}", serde_json::to_string_pretty(&philosophers_audit::check_mutants(Path::new(events), Path::new(output))?.1)?),
        [mode, events] if mode == "--audit-api" => println!("{}", serde_json::to_string_pretty(&api_audit::audit(Path::new(events))?)?),
        [mode, events] if mode == "--audit-full-stack-api" => println!("{}", serde_json::to_string_pretty(&full_stack_api_audit::audit(Path::new(events))?)?),
        [mode, events, output] if mode == "--check-api-mutants" => println!("{}", serde_json::to_string_pretty(&api_audit::check_mutants(Path::new(events), Path::new(output))?)?),
        [mode, events] if mode == "--audit-review" => println!("{}", serde_json::to_string_pretty(&review_audit::audit(Path::new(events))?)?),
        [mode, events] if mode == "--audit-staged-integration" => println!("{}", serde_json::to_string_pretty(&staged_integration_audit::audit(Path::new(events))?)?),
        [mode, events, output] if mode == "--check-staged-integration-mutants" => println!("{}", serde_json::to_string_pretty(&staged_integration_audit::check_mutants(Path::new(events),Path::new(output))?)?),
        [mode, events] if mode == "--audit-feature-delivery" => println!("{}", serde_json::to_string_pretty(&feature_delivery_audit::audit(Path::new(events))?)?),
        [mode, events, output] if mode == "--check-feature-delivery-mutants" => println!("{}", serde_json::to_string_pretty(&feature_delivery_audit::check_mutants(Path::new(events),Path::new(output))?)?),
        [mode, events] if mode == "--audit-refactor" => println!("{}", serde_json::to_string_pretty(&refactor_audit::audit(Path::new(events))?)?),
        [mode, events] if mode == "--audit-lab-quality" => println!("{}", serde_json::to_string_pretty(&lab_quality_audit::audit(Path::new(events))?)?),
        [mode, events] if mode == "--audit-lab-cascading" => println!("{}", serde_json::to_string_pretty(&lab_cascading_audit::audit(Path::new(events))?)?),
        [mode, events] if mode == "--audit-drug-discovery" => println!("{}", serde_json::to_string_pretty(&drug_discovery_audit::audit(Path::new(events))?)?),
        [mode, events] if mode == "--audit-regulatory-discovery" => println!("{}", serde_json::to_string_pretty(&regulatory_discovery_audit::audit(Path::new(events))?)?),
        [mode, events] if mode == "--audit-multi-round-discovery" => println!("{}", serde_json::to_string_pretty(&multi_round_discovery_audit::audit(Path::new(events))?)?),
        [mode, events, output] if mode == "--check-lab-quality-mutants" => println!("{}", serde_json::to_string_pretty(&lab_quality_audit::check_mutants(Path::new(events), Path::new(output))?.1)?),
        [mode, events] if mode == "--audit-lab" => println!("{}", serde_json::to_string_pretty(&lab_audit::audit(Path::new(events))?)?),
        [mode, events] if mode == "--audit-coordination" => println!("{}", serde_json::to_string_pretty(&coordination_audit::audit(Path::new(events))?)?),
        [mode, events] if mode == "--audit-medical" => println!("{}", serde_json::to_string_pretty(&medical_audit::audit(Path::new(events))?)?),
        [mode, events, output] if mode == "--check-medical-mutants" => println!("{}", serde_json::to_string_pretty(&medical_audit::check_mutants(Path::new(events),Path::new(output))?)?),
        [mode, events] if mode == "--audit-multi-specialty" => println!("{}", serde_json::to_string_pretty(&multi_specialty_audit::audit(Path::new(events))?)?),
        [mode, events] if mode == "--audit-critical-care" => println!("{}", serde_json::to_string_pretty(&critical_care_audit::audit(Path::new(events))?)?),
        [mode, events, output] if mode == "--check-critical-care-mutants" => println!("{}", serde_json::to_string_pretty(&critical_care_audit::check_mutants(Path::new(events),Path::new(output))?)?),
        [mode, events, output] if mode == "--check-multi-specialty-mutants" => println!("{}", serde_json::to_string_pretty(&multi_specialty_audit::check_mutants(Path::new(events),Path::new(output))?)?),
        [mode, events] if mode == "--audit-report" => println!("{}", serde_json::to_string_pretty(&report_audit::audit(Path::new(events))?)?),
        [mode, events] if mode == "--audit-research-paper" => println!("{}", serde_json::to_string_pretty(&research_paper_audit::audit(Path::new(events))?)?),
        [mode, events] if mode == "--audit-survey-paper" => println!("{}", serde_json::to_string_pretty(&survey_paper_audit::audit(Path::new(events))?)?),
        [mode, events] if mode == "--audit-figure-document" => println!("{}", serde_json::to_string_pretty(&figure_document_audit::audit(Path::new(events))?)?),
        [mode, events] if mode == "--audit-large-document" => println!("{}", serde_json::to_string_pretty(&large_document_audit::audit(Path::new(events))?)?),
        [mode, events] if mode == "--audit-document" => println!("{}", serde_json::to_string_pretty(&document_audit::audit(Path::new(events))?)?),
        [mode, events] if mode == "--audit-kitchen" => println!("{}", serde_json::to_string_pretty(&kitchen_audit::audit(Path::new(events))?)?),
        [mode, events, output] if mode == "--check-kitchen-mutants" => println!("{}", serde_json::to_string_pretty(&kitchen_audit::check_mutants(Path::new(events),Path::new(output))?)?),
        [mode, events] if mode == "--audit-manufacturing" => println!("{}", serde_json::to_string_pretty(&manufacturing_audit::audit(Path::new(events))?)?),
        [mode, events, output] if mode == "--check-manufacturing-mutants" => println!("{}", serde_json::to_string_pretty(&manufacturing_audit::check_mutants(Path::new(events), Path::new(output))?)?),
        [mode, events] if mode == "--audit-build" => println!("{}", serde_json::to_string_pretty(&build_audit::audit(Path::new(events))?)?),
        [mode, events] if mode == "--audit-cascading-build" => println!("{}", serde_json::to_string_pretty(&cascading_build_audit::audit(Path::new(events))?)?),
        [mode, events, output] if mode == "--check-cascading-build-mutants" => println!("{}", serde_json::to_string_pretty(&cascading_build_audit::check_mutants(Path::new(events),Path::new(output))?)?),
        [mode, events, output] if mode == "--check-build-mutants" => println!("{}", serde_json::to_string_pretty(&build_audit::check_mutants(Path::new(events), Path::new(output))?)?),
        [mode, events, output] if mode == "--check-document-mutants" => println!("{}", serde_json::to_string_pretty(&document_audit::check_mutants(Path::new(events), Path::new(output))?)?),
        [mode, events, output] if mode == "--check-report-mutants" => println!("{}", serde_json::to_string_pretty(&report_audit::check_mutants(Path::new(events), Path::new(output))?)?),
        [mode, events, output] if mode == "--check-review-mutants" => println!("{}", serde_json::to_string_pretty(&review_audit::check_mutants(Path::new(events), Path::new(output))?)?),
        [mode, events] if mode == "--audit-writing" => println!("{}", serde_json::to_string_pretty(&writing_audit::audit(Path::new(events))?)?),
        [mode, events, output] if mode == "--check-writing-mutants" => println!("{}", serde_json::to_string_pretty(&writing_audit::check_mutants(Path::new(events), Path::new(output))?)?),
        [mode, events] if mode == "--audit-cicd" => println!(
            "{}",
            serde_json::to_string_pretty(&cicd_audit::audit(Path::new(events))?)?
        ),
        [mode, events] if mode == "--audit-cicd-pipeline" => println!(
            "{}",
            serde_json::to_string_pretty(&cicd_pipeline_audit::audit(Path::new(events))?)?
        ),
        [mode, events, output] if mode == "--check-cicd-mutants" => println!(
            "{}",
            serde_json::to_string_pretty(&cicd_audit::check_mutants(Path::new(events), Path::new(output))?)?
        ),
        [mode, events] if mode == "--audit" => println!(
            "{}",
            serde_json::to_string_pretty(&codebase_audit::audit(Path::new(events))?)?
        ),
        [mode, events] if mode == "--audit-protocol" => {
            let events = Path::new(events);
            let artifact = projection::load(events)?;
            let rows = journal::records(events)?;
            let frames = projection::frames(&artifact, &rows)?;
            println!("{}", serde_json::to_string_pretty(&json!({
                "status":"passed_protocol_audit", "wire_frames":frames.len(),
                "dynamic_protocol":projection::check_dynamic(&artifact, &rows)?,
                "application_effects_certified":false
            }))?);
        },
        [mode, events, output] if mode == "--check-protocol-mutants" => println!(
            "{}",
            serde_json::to_string_pretty(&codebase_audit::check_mutants(Path::new(events), Path::new(output))?)?
        ),
        [mode, input, output] if mode == "--check-run-mutants" => println!(
            "{}",
            serde_json::to_string_pretty(&audit::check_mutants(
                Path::new(input),
                Path::new(output)
            )?)?
        ),
        [mode, output] if mode == "--audit-run" => println!(
            "{}",
            serde_json::to_string_pretty(&audit::run(Path::new(output))?)?
        ),
        [task, depth, output] => {
            let result = match run(task, depth.parse()?, Path::new(output)).await {
                Ok(result) => result,
                Err(error) => {
                    let directory = Path::new(output);
                    if directory.exists() {
                        save(
                            &directory.join("harness-error.json"),
                            &json!({"kind":"harness_error","error":error.to_string(),"scored_benchmark":false}),
                        )?;
                    }
                    return Err(error);
                }
            };
            println!("{}", serde_json::to_string(&result)?);
            if let Some(error) = result["error"].as_str() {
                return Err(error.to_owned().into());
            }
        }
        _ => return Err(concat!(
            "usage: choreofix-experiments 1E|1M|1H|2E|2M|2H|3E|3M|3H|4E|4M|5E|5M|5H|6E|6M|6H|7E|7M|7H|8E|8M|9E|9M|9H|10E|10M|11E|12E|13E|13M|13H|14E|14M|14H|16E|16M DEPTH OUTPUT\n",
            "       --audit-api|--audit-review|--audit-report|--audit-document|--audit-coordination|--audit-writing|--audit-cicd|--audit-cicd-pipeline|--audit-philosophers|--audit-lab|--audit-lab-quality|--audit EVENTS\n",
            "       --audit-protocol EVENTS\n",
            "       --check-api-mutants|--check-review-mutants|--check-report-mutants|--check-document-mutants|--check-writing-mutants|--check-cicd-mutants|--check-philosophers-mutants|--check-protocol-mutants EVENTS NEW_OUTPUT\n",
            "       --audit-run RUN\n",
            "       --build-wasi NEW_OUTPUT FAMILY[,FAMILY...]\n",
            "       --qualify-wasi NEW_OUTPUT [DIAGNOSTIC_TEST_FILTER]\n",
            "       --check-run-mutants RUN NEW_OUTPUT"
        ).into()),
    }
    Ok(())
}
