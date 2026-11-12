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
mod codebase_audit;
#[cfg(test)]
mod codebase_boundary;
mod coordination_audit;
#[cfg(test)]
mod coordination_boundary;
mod critical_care_audit;
#[cfg(test)]
mod critical_care_boundary;
mod discovery_audit;
#[cfg(test)]
mod discovery_boundary;
mod document_audit;
#[cfg(test)]
mod document_boundary;
mod evidence;
mod feature_delivery_audit;
#[cfg(test)]
mod feature_delivery_boundary;
mod journal;
mod kitchen_audit;
#[cfg(test)]
mod kitchen_boundary;
mod lab_audit;
#[cfg(test)]
mod lab_boundary;
mod lab_quality_audit;
#[cfg(test)]
mod lab_quality_boundary;
mod manufacturing_audit;
#[cfg(test)]
mod manufacturing_boundary;
mod medical_audit;
#[cfg(test)]
mod medical_boundary;
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
mod review_audit;
#[cfg(test)]
mod review_boundary;
mod snapshot;
mod sources;
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
    let review = task == "6E";
    let feature_delivery = task == "6M";
    let report = task == "3E";
    let document = task == "7E";
    let build = matches!(task, "10E" | "10M");
    let manufacturing = task == "11E";
    let kitchen = task == "12E";
    let coordination = task == "4E";
    let refactor = task == "4M";
    let discovery = task == "14E";
    let regulatory_discovery = task == "14M";
    let multi_round_discovery = task == "14H";
    let lab = task == "13E";
    let lab_quality = task == "13M";
    let medical = task == "5E";
    let multi_specialty = task == "5M";
    let critical_care = task == "5H";
    if !(dining
        || coding
        || api
        || cicd
        || review
        || feature_delivery
        || document
        || build
        || report
        || coordination
        || refactor
        || discovery
        || regulatory_discovery
        || multi_round_discovery
        || lab
        || lab_quality
        || medical
        || multi_specialty
        || critical_care
        || manufacturing
        || kitchen
        || family == 2)
    {
        return Err(format!("task {task} has no model prompt implementation").into());
    }
    let data = if dining {
        json!({"participant":agent,"resources":actor["forks"],"philosopher":agent})
    } else if coding {
        json!({"participant":agent,"resources":actor["modules"],
        "feature_name":actor["calls"][0]["arguments"]["feature_name"]})
    } else if multi_round_discovery {
        json!({"participant":agent,"resources":actor["resources"],"domain_operations":actor["operations"]})
    } else if review
        || feature_delivery
        || document
        || build
        || report
        || coordination
        || refactor
        || discovery
        || regulatory_discovery
        || multi_round_discovery
        || lab
        || lab_quality
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
    let objects = if kitchen {
        "/service/prepare_base_dish, /service/prepare_ingredient and /service/combine_dish"
    } else if manufacturing {
        match agent {
            "TECHNICIAN_A" | "TECHNICIAN_B" => "/service/do_job",
            "SUPERVISOR" => "/service/collect_reports",
            _ => return Err("unknown manufacturing participant".into()),
        }
    } else if build {
        match agent {
            "BUILDER_A" | "BUILDER_B" | "BUILDER_C" => "/service/compile_module",
            "TEST_RUNNER" if task == "10M" => "/service/run_build_tests",
            "INTEGRATOR" => "/service/link_modules",
            _ => return Err("unknown shared-build participant".into()),
        }
    } else if document {
        match agent {
            "WRITER_A" | "WRITER_B" => {
                "/service/write_section, /service/update_references, /service/revise_section and /service/feedback"
            }
            "EDITOR" => "/service/review_sections and /service/combine_sections",
            _ => return Err("unknown document participant".into()),
        }
    } else if report {
        match agent {
            "RESEARCHER_A" | "RESEARCHER_B" => {
                "/service/research_topic, /service/write_section, /service/update_references and /service/revise_section"
            }
            "EDITOR" => "/service/review_sections and /service/combine_sections",
            _ => return Err("unknown report participant".into()),
        }
    } else if critical_care {
        match agent {
            "ER_DOCTOR_A" | "ER_DOCTOR_B" => {
                "/service/assess_patient, /service/order_tests and /service/diagnose"
            }
            "RADIOLOGIST" => "/service/perform_imaging and /service/interpret_imaging",
            "SURGEON" => "/service/perform_surgery",
            "ANESTHESIOLOGIST" => "/service/prepare_anesthesia",
            "ICU_COORDINATOR" => "/service/manage_icu",
            "PHARMACIST" => "/service/review_prescriptions",
            _ => return Err("unknown critical-care participant".into()),
        }
    } else if multi_specialty {
        match agent {
            "ER_DOCTOR" => {
                "/service/assess_patient, /service/order_tests and /service/conduct_case_conference"
            }
            "CARDIOLOGIST" | "NEUROLOGIST" => "/service/order_tests and /service/diagnose",
            "RADIOLOGIST" => "/service/perform_imaging and /service/interpret_imaging",
            "PHARMACIST" => "/service/review_prescriptions",
            _ => return Err("unknown multi-specialty participant".into()),
        }
    } else if medical {
        match agent {
            "ER_DOCTOR" => {
                "/service/assess_patient, /service/order_tests and /service/perform_imaging"
            }
            "CARDIOLOGIST" => "/service/order_tests and /service/diagnose",
            "PHARMACIST" => "/service/review_prescriptions",
            _ => return Err("unknown consultation participant".into()),
        }
    } else if multi_round_discovery {
        match agent {
            "CHEMIST" => "/service/run_hplc_analysis and /service/run_mass_spec_analysis",
            "BIOLOGIST" => {
                "/service/run_cell_assay, /service/run_hplc_analysis and /service/submit_evidence"
            }
            "TOXICOLOGIST" => {
                "/service/run_mass_spec_analysis, /service/run_hplc_analysis, /service/run_cell_assay and /service/submit_evidence"
            }
            "FORMULATION_SCIENTIST" => "/service/prepare_formulation",
            "CLINICAL_LEAD" => "/service/review_clinical_data",
            "REGULATORY_SPECIALIST" => {
                "/service/verify_regulatory_compliance and /service/submit_evidence"
            }
            "PROJECT_DIRECTOR" => "/service/submit_project_decision",
            _ => return Err("unknown multi-round scientist".into()),
        }
    } else if regulatory_discovery {
        match agent {
            "CHEMIST" => "/service/run_hplc_analysis and /service/run_mass_spec_analysis",
            "BIOLOGIST" | "TOXICOLOGIST" => {
                "/service/run_cell_assay, /service/run_hplc_analysis and /service/await_review"
            }
            "LEAD_SCIENTIST" => {
                "/service/run_mass_spec_analysis, /service/run_hplc_analysis, /service/submit_decision and /service/reconsider"
            }
            "REGULATORY_SPECIALIST" => "/service/verify_regulatory_compliance",
            _ => return Err("unknown regulatory-study participant".into()),
        }
    } else if lab_quality {
        match agent {
            "CHEMIST" => {
                "/service/run_synthesis, /service/run_analysis and /service/run_separation"
            }
            "BIOLOGIST" => {
                "/service/run_separation, /service/run_analysis and /service/run_bioassay"
            }
            "ANALYST" => "/service/run_analysis and /service/run_separation",
            "QC_CONTROLLER" => "/service/verify_result",
            _ => return Err("unknown QC laboratory participant".into()),
        }
    } else if lab {
        match agent {
            "BIOLOGIST" => {
                "/service/run_separation, /service/run_analysis and /service/run_bioassay"
            }
            "CHEMIST" | "ANALYST" => "/service/run_analysis and /service/run_separation",
            _ => return Err("unknown laboratory participant".into()),
        }
    } else if discovery {
        match agent {
            "BIOLOGIST" => "/service/run_cell_assay and /service/run_hplc_analysis",
            "CHEMIST" | "TOXICOLOGIST" => {
                "/service/run_hplc_analysis, /service/run_mass_spec_analysis and /service/run_cell_assay where assigned"
            }
            "LEAD_SCIENTIST" => {
                "/service/run_mass_spec_analysis, /service/run_hplc_analysis and /service/submit_decision"
            }
            _ => return Err("unknown instrument-task participant".into()),
        }
    } else if refactor {
        match agent {
            "ARCHITECT" => "/service/design_proposal",
            "DEVELOPER_A" | "DEVELOPER_B" => "/service/refactor_code and /service/merge_code",
            "REVIEWER" => "/service/code_review",
            "TESTER" => "/service/run_tests",
            _ => return Err("unknown shared-refactor participant".into()),
        }
    } else if coordination {
        match agent {
            "BACKEND_DEV" => {
                "/service/define_api, /service/implement_feature, /service/loan_frontend, /service/finalize and /service/await_tests"
            }
            "FRONTEND_DEV" => "/service/agree_api and /service/implement_feature",
            "TESTER" => "/service/run_tests",
            _ => return Err("unknown API coordination participant".into()),
        }
    } else if feature_delivery {
        match agent {
            "DEVELOPER_A" | "DEVELOPER_B" => "/service/implement_feature and /service/merge_code",
            "REVIEWER" => "/service/code_review",
            "TEST_WRITER" => "/service/write_tests",
            "CI_RUNNER" => "/service/run_tests",
            _ => return Err("unknown feature-delivery participant".into()),
        }
    } else if review {
        match agent {
            "DEVELOPER_A" | "DEVELOPER_B" => "/service/implement_feature and /service/merge_code",
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
    let coordination = if kitchen {
        "Each chef prepares its assigned base and outgoing ingredient, choosing either order. The oven and, for chefs A/B only, stovetop Endpoints carry the actual base request/result and return. Ingredient preparation needs no equipment; its completed result goes directly to the assigned recipient. Combine your own dish after its base and required received ingredient exist. A combine request may wait for the actual incoming ingredient while other chefs continue. You do not call acquire/release or notification tools. Each of the three assigned effects runs once. Read actual results, use private shell computation freely and finish your own work. The common world checks completed material inputs and physical conflicts, without prescribing this protocol or a total operation order."
    } else if manufacturing {
        "Choose your assigned job and your private shell computations freely. Actual workstation and request Endpoints transfer automatically through the global; do not invent acquire/release or notification calls. Write the original do_job arguments to /service/do_job and read /service/result. A busy status means that no job effect ran for that attempt: choose your other assigned job while that workstation is occupied, then retry the pending job. Actual completed job results go to the supervisor automatically. The supervisor writes {} to /service/collect_reports; that call waits for all four actual job results. Exit after your actual assigned work is complete. The common world grades actual completed report contents and machine conflicts, not one prescribed total order."
    } else if task == "10M" {
        "Each builder uses its assigned module and both actual resource Endpoints, which transfer and return automatically. Compile responses contain the actual test under coordination.test, actual shared updates and a continuation. A continuation naming your module requests a fresh compile even if an earlier test passed: later shared-input changes can make an old object stale. A continuation with tool_name link_modules is the final linked outcome; then finish. TEST_RUNNER tests actual submitted objects. A wrong module returns invalid_arguments with actual_build, so use that actual module rather than guessing a fixed total schedule. Each actual test response contains coordination.next: either the next real compile report with data.module, or the final link_modules result. Continue with those data until the linked outcome. INTEGRATOR requests link_modules for its three assigned modules; it waits for current positive tests and return of all editing Endpoints. No acquire/release, notification or repair command is needed. Private commands and computations remain your choice. The common environment grades real object versions and captured link/test inputs, not one prescribed total invocation order."
    } else if build {
        "The two actual resource Endpoints transfer automatically; no acquisition, release or notification commands are needed. Compile results contain the actual object bytes and shared type inputs. A updates the provided SHARED_TYPES definitions. B may initially compile against the provided old definitions: that object is real but becomes stale when A updates them. B can use the same service to rebuild against the actual new definitions before finishing. A current object's SHARED_TYPES.modules includes module_a's declarations. The integrator's link waits for both current actual build reports. Private commands and computations remain your choice, and the common world scores the actual object and link inputs rather than a prescribed total invocation order."
    } else if document {
        "Both writers may revise either chapter. Initially write your assigned chapter and update its references, choosing either order. Then write {} to /service/feedback and read /service/result. An actual negative review reaches both writers. The writer assigned the rejected chapter may revise either chapter to address it, then request feedback again; the other writer waits for the next feedback. The editor reviews both complete-document scopes each round, in either order. After a rejection, request both reviews again after the actual revision; a review request may wait for that material. Combine CHAPTER_1 and CHAPTER_2 only after both current reviews succeed. The actual document Endpoint transfers among writers and editor, then returns before publication. Feedback eventually carries the combined document to both writers. Finish your work and exit after that result. You do not call acquire/release or notification tools, and private shell computation remains free. The common world grades actual contents and completed content-bound receipts, not one total operation order."
    } else if feature_delivery {
        "Use your assigned feature, branch, review target, test target or suite. The reviewer and test writer choose their initial feature order. Actual responses carry coordination data: reviewed candidates, tests, interface-change data and CI outcomes. Rejected reviews and failed CI require fresh implementation, tests and review for that feature; continue from the actual response rather than assuming a fixed retry count. Merge returns the actual CI outcome. A wrong CI suite returns invalid_arguments with the actual publication so it can be corrected. The final responses report both features' CI outcomes. Finish the required work and receive the final coordination outcomes before exiting; independent participants and operations remain concurrent."
    } else if critical_care {
        "Use your assigned patient and actual results. Each ER doctor assesses its own patient, requests the assigned panel (together or as partial tests), then requests diagnose. A final diagnosis response contains the actual findings, surgery or ICU outcome, and medication review. Neither patient's care requires the other patient's assessment or completed care. The radiologist chooses the order of the two patients' CT operations and patient B's MRI operations, keeping each interpretation after its matching actual image. The anesthesiologist prepares patient A; the actual operating-room Endpoint and preparation then travel to surgery. The surgeon performs the assigned procedure on patient A. ICU_COORDINATOR requests the actual patient B care with both bed and ventilator Endpoints. PHARMACIST reviews the assigned patient_A_and_B medication data. Resource loans and result forwarding happen through the global; do not invent acquisition, release or notification commands. Symptoms, medications and anesthesia type are optional caller strings. Choose your private shell commands and calculations freely. Read /service/result after each request and exit when your own work is complete. This abstract environment has no medication-administration, monitoring or discharge tool, and these runs cannot certify those absent effects. Its diagnostic failure injection is disabled; a depth number is not evidence of an injected-failure test."
    } else if multi_specialty {
        "Use your actual patient findings and assigned operations. The ER doctor assesses the patient and requests the general tests. The cardiologist and neurologist receive that actual referral, request their own tests and supply diagnoses using the shared investigation results. Each test panel may be combined, split or repeated until all assigned tests exist. The radiologist chooses the order of CT and MRI operations; each interpretation needs its matching actual image, and independent CT/MRI operations need not wait for each other. All completed findings are delivered automatically through the global. Each specialist's diagnosis response and the radiologist's final interpretation response also contain the completed consultation. After the ER doctor's final general test response contains both diagnoses and the medication review, the ER doctor requests conduct_case_conference for the actual patient; optional attendees is caller-supplied text. Optional symptoms and medications are caller-supplied strings. The real test-panel, imaging and consultation-room Endpoints carry requests, results and returns; do not invent acquire/release/notification commands. Independent participants continue while a request waits for input. Choose your private shell commands and calculations freely. Read /service/result after each request, then exit after your own work. This abstract benchmark has no prescribing or administration operation and does not represent diagnostic disagreement; do not claim either absent effect."
    } else if medical {
        "Use the actual patient findings and your assigned operations. ER_DOCTOR first assesses the patient, then chooses the order of general-panel tests and CT imaging. Each panel can be requested together or as separate test_type requests; repeat incomplete tests if useful, and supply every assigned test. CT can be requested before, between or after partial general-panel requests. The cardiologist receives the actual referral automatically, requests the cardiac panel and supplies its diagnosis. The diagnosis response includes the actual shared general findings under consultation. The ER doctor's final test or imaging response includes the actual shared findings, cardiac diagnosis and medication review under consultation. PHARMACIST reviews caller-supplied medication data; medications and symptoms are optional string arguments, not fixed answers. The actual CT and test-panel Endpoints are passed through the global; do not invent acquire, release or notification commands. Independent participants continue while your request waits for required input. You choose your private shell commands and calculations. Read /service/result after every request and finish after your own work. This abstract benchmark has no prescribing or administration tool, and these results do not certify absent clinical effects."
    } else if multi_round_discovery {
        "Choose private shell commands and calculations freely, use actual scientific results, and reconsider failures. Resource and material Endpoints carry requests and results automatically. CHEMIST has three required analyses: synthesis_qc, molecular_id and documentation. A successful molecular_id is an intermediate result, not completion of the chemist assignment. Request HPLC documentation even if it must wait for the director; exit only after its actual documentation result. Documentation archives either terminal outcome, including abort. FORMULATION_SCIENTIST reads /service/result for the actual compound identity, supplies its data.compound to prepare_formulation, and retries failed preparation. BIOLOGIST performs culture_prep, then may choose the order of bioassay and stability for each actual formulation. It may retest the same bioassay material while held; submit its chosen final outcome with {\"evidence\":\"bioassay\"} to /service/submit_evidence. TOXICOLOGIST performs baseline_markers, purity_check and cytotoxicity, may retest toxicity while held, and submits {\"evidence\":\"cytotoxicity\"}. A submitted assay material cannot be edited again in that round. CLINICAL_LEAD requests combined_review of the actual completed evidence. REGULATORY_SPECIALIST chooses initial_review or hold_review, may repeat or deepen review, and submits its actual verdict with {\"evidence\":\"regulatory\"}. A held review on unchanged evidence cannot be made positive by routine repetition. PROJECT_DIRECTOR reads the actual regulatory result, then chooses continue, retry or abort; continue requires positive approval. Retry starts another formulation and assessment round, without refilling the three biological samples. Failed or exhausted assays can support an honest insufficient-data assessment and abort. The final documentation archives the actual terminal outcome. After evidence submission or a previous round, read actual responses and coordination data: they carry fresh material or the final outcome. Each scientist stays available for a requested new round and exits after the final outcome. Do not invent acquisition, release, refill or notification commands. These shell paths are the available interface, not an instruction to fabricate results or follow one total agent schedule."
    } else if regulatory_discovery {
        "The original scientific assignment is carried through actual instrument and study Endpoints. You choose private shell computations and commands, read every actual result, and retry your own failed analysis. CHEMIST runs synthesis_qc, molecular_id and finally documentation; documentation waits for actual regulatory approval. BIOLOGIST runs culture_prep, bioassay and stability. TOXICOLOGIST runs baseline_markers, purity_check and cytotoxicity. Both then query /service/await_review with {}. A reanalysis response specifies whether to repeat your two compound assays; after completing those selected assays, query await_review again. A regulatory result with success:true is final. LEAD_SCIENTIST runs reference_methods, structural_confirmation and characterization, choosing either order for the last two. The characterization result includes actual compound and report data under context. Choose either go or no-go by writing {\"go\":true} or {\"go\":false} to /service/submit_decision. Its response includes the actual regulatory_review. After a rejection, choose whether biology and/or toxicology should be repeated by writing two booleans, for example {\"biology\":true,\"toxicology\":false}, to /service/reconsider, then submit your reconsidered decision. All four choices, including neither and both, are available; no fixed rejection count is assumed. REGULATORY_SPECIALIST runs verify_regulatory_compliance with {\"decision\":\"go_no_go\"} and repeats after rejection. Approvals and actual data travel through the choreography. Do not invent acquire/release or notification commands. Independent private computation remains available while instrument calls wait. Finish your own work only after its actual successful outcome."
    } else if lab_quality {
        "Use your assigned laboratory operations and actual results. The chemist synthesizes the assigned compound, measures synthesized_compound, purifies compound_purification and finally validates final_validation. A submitted purification or later chemist remeasurement returns {operation, verification}: read the actual verification, and if rejected, reconsider and measure synthesized_compound again. The biologist prepares cell_cultures and fluorescence, then submits run_bioassay for the actual synthesized compound; optional cell_type is caller data. A failed trial returns its actual failure immediately: reconsider and run a fresh trial. A successful trial returns {operation, verification}; QC rejection also requires a fresh trial. QC_CONTROLLER chooses either assigned scientist per verify_result request and may switch subjects after rejection. Verify both scientists until each has a successful actual approval. ANALYST performs molecular_detail and verification for the delivered approved biological result. Independent biology can overlap chemist QC. The real scientific and physical instrument Endpoints carry the requests, results and returns. You do not send acquisition, release or notification tools. Services may wait for actual inputs and resources while independent participants continue. Read /service/result after each service request, use private shell commands freely, and finish after your own assigned work. The common scorer checks actual inputs, current approvals, final results and physical conflicts for every method; it does not grade this choreography."
    } else if lab {
        "Use your assigned laboratory operations and actual results. The chemist analyzes raw_materials, purifies compound_purification and validates final_validation after the actual quality report. The biologist prepares cell_cultures, calibrates fluorescence and runs the bioassay on the delivered compound_purification; cell_type is optional and may be any string. The analyst uses molecular_detail and verification after the actual assay report. The specimen, reports and instrument Endpoints travel through the global automatically; do not invent acquisition or notification commands. A service may wait while independent scientists continue. You choose your own private commands and computations. The common simulator checks necessary material/result causality and instrument conflicts for every method."
    } else if discovery {
        "Run your assigned instrument analyses from the task description. The chemist runs synthesis_qc, molecular_id and then documentation after the actual lead decision receipt. The biologist runs culture_prep, bioassay and stability; actual compound data arrives automatically. The toxicologist runs baseline_markers, purity_check and cytotoxicity. The lead runs reference_methods, structural_confirmation and characterization. The characterization response includes the actual compound and both reports under context. Read that data, choose go or no-go, then write one boolean go field to /service/submit_decision and read the actual result. Either true or false is allowed. Compound samples, reports, instrument Endpoints and their returns follow the global automatically; do not invent acquire/release or notification commands. Instrument calls may wait while other participants continue. You choose your private commands and computations. Required data must exist before use, and the common simulator checks those causal dependencies for every method."
    } else if report {
        "Hibana delegates the DOCUMENT or DATABASE Endpoint for each shared operation. The actual request and result travel through that Endpoint before it returns. You do not call resource acquisition, release or notification tools. Researchers research their assigned topic, write their assigned section and update its references. The references result includes the actual review under coordination.review_sections. After rejection, reconsider and revise your own section; the revision result includes the next actual review. Finish after your assigned section is approved. The editor reviews both submitted sections, choosing which to review first and whether to switch after a rejection. A review can overlap revision of the other section. Read the actual results and combine both sections after both pass."
    } else if refactor {
        "The architect chooses which assigned proposal to create first. Each developer refactors its own module and SHARED_LIBRARY, choosing either order, then merges its assigned branch. The actual repository Endpoint is transferred and returned; no acquire/release or notification calls are needed. The tester runs both assigned suites twice: once for each repository snapshot supplied by the choreography, choosing either suite order in each round. Reviews can overlap independent tests and merging: the reviewer chooses either assigned module first and reviews both. Service calls may wait for required data and resources while other participants continue. Read each actual result and finish your own work. Private shell computation remains available independently. These are this implementation's protocol capabilities. The common simulator grades actual published contents and requires completed exact-content review and test results before publication, equally for every method. It does not grade this choreography or one total operation order."
    } else if coordination {
        "The backend defines the assigned API; its actual result is sent to the frontend for agreement. FRONTEND_DEV writes {\"api_name\":\"feature_api_spec\"} to /service/agree_api and reads /service/result. An API response contains api_contract: implement your assigned frontend feature after agreement. After each frontend implementation, call agree_api again; an actual response with tool_name run_tests is the final test outcome, so finish. BACKEND_DEV defines the API and implements its assigned backend feature; the actual REPO Endpoint then passes to the frontend and returns. To choose frontend-first instead, write {\"feature_name\":\"frontend_component\"} to /service/loan_frontend before implementing the backend; this moves the actual REPO Endpoint, not a domain operation. You may change the API by defining it again after both implementations; the frontend must agree again and both features must be reimplemented. Do not invent changes merely to retry: the original task has no injected failures. When satisfied, the backend writes {\"api_name\":\"feature_api_spec\"} to /service/finalize; this returns the editing Endpoint. Then write {\"suite\":\"integration_tests\"} to /service/await_tests and read the actual final result before finishing. TESTER writes its assigned run_tests request; it waits for finalization, then uses both actual resource Endpoints. You do not acquire locks or send notification tools. Private computation and all guest I/O remain available independently. Read each actual result."
    } else if review {
        "Hibana delegates the repository Endpoint during implementation and merging; each actual request and result travels through it before it returns. Code review does not hold the repository. You do not call resource acquisition, release or notification tools. A developer's implementation response includes the actual review under coordination.code_review. If review fails, reconsider and implement again; merge your assigned branch after approval. The reviewer chooses the order of the assigned features and may switch to the other feature after a rejection while the developer revises. Review both features until each passes. Reviewing one feature can overlap independent work on the other feature. Read every actual result and finish after your own assigned work is complete."
    } else if cicd {
        "Hibana passes the build-server or deployment-environment Endpoint to your WASI participant. Your actual request and result travel through that Endpoint before it returns. The developer's build precedes integration testing, which precedes deployment. Hibana sends build-complete, tests-passed and both deployment-complete notifications. You do not call acquire/release or notification tools. Read your actual result, then finish. This original task has no retry loop or injected domain failures."
    } else if dining {
        "Hibana passes both fork endpoints to your guest. Eating requires the requests sent through both endpoints, and both endpoints return afterwards. You do not call acquire/release tools. Your operations always apply to your participant. The philosopher string is passed to the original simulator. Both effects run once, then finish."
    } else if task == "1H" {
        "Hibana passes both module endpoints to your guest. Committing requires requests through both endpoints. You do not call acquire/release tools. This replaces the original sequential acquisition procedure. The feature and module pair must match your assignment. Module endpoints return before testing so that independent developers can continue. Read each actual test result. If it fails, reconsider your implementation, then implement, commit and test again. Finish after your tests pass."
    } else if coding {
        "Hibana passes both module endpoints to your guest. Committing requires the requests sent through both endpoints. You do not call acquire/release tools. This replaces the original sequential acquisition procedure. The feature and module pair must match your assignment. For this task the test outcome is terminal, including failure. Do not add retries. Module endpoints are returned before testing so that another developer can commit while your test runs."
    } else if api {
        "Hibana passes the database or test-environment Endpoint to the WASI participant. The request and actual result travel through that Endpoint before it returns. Migration completion precedes backend work, and backend completion precedes integration testing. You do not call acquire/release or notification tools. Use your assigned arguments. DB_ADMIN and BACKEND_DEV must also read the read-only /service/outcome object, which contains the original test result. That read may wait until the test finishes while other participants continue. A failed integration test is terminal in the task description and simulator. Report its actual result and finish, without adding a revision or retry."
    } else {
        "Hibana passes both section Endpoints to your WASI guest in parallel. Your actual write request travels through both Endpoints; the actual result comes back through both before they return. You do not call acquire/release tools. This deliberately replaces the original sequential acquisition procedure. The target must match your assigned topic, section or pair of resource names. The review result is terminal, so do not add retries to a failed review."
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
Private /work files have names you choose, with eight slots of 8192 bytes.
A script has at most 4096 UTF-8 bytes. Each participant has its own persistent
guest and private files.

Your writable domain objects are {objects}.
Write the matching operation's JSON argument object and close it to request
the operation. Shell redirection closes its output after writing.
Read /service/result for the actual response. These objects are not commands.
{coordination}
Independent participants and domain operations run concurrently. A request
waiting for its dependency does not block the other guests.

Ordinary shell errors and invalid argument data can be corrected. Use actual
responses to reconsider your work. When your assigned work is finished,
close your guest with exit 0 before your final response. After an exit_status
reply, do not call shell again. Leaving before doing the work fails the run.
"#,
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

#[hibana_runtime::scope]
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
    let finished: Result<()> = race(
        async {
            async {
                let result = match guest_loop.await {
                    Ok(value) => json!({"kind":"guest_runtime","result":value}),
                    Err(error) => json!({"status":"harness_error","error":error.to_string()}),
                };
                save(&output.join("guest-outcome.json"), &result)?;
            };
            async {
                let result = match model::run(
                    people,
                    model::Limits {
                        timeout: Duration::from_secs(seconds),
                        tool_calls: budget,
                    },
                    &guest,
                )
                .await
                {
                    Ok(results) => json!(
                        results
                            .into_iter()
                            .map(|result| json!({"kind":"model_participant","result":result}))
                            .collect::<Vec<_>>()
                    ),
                    Err(error) => json!([{"status":"harness_error","error":error.to_string()}]),
                };
                save(&output.join("model-outcomes.json"), &result)?;
            };
            Ok(())
        },
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
    let error = match finished {
        Ok(()) => trial_error(&outcomes),
        Err(error) => Some(error.to_string()),
    };
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
        [mode, events, output] if mode == "--check-philosophers-mutants" => println!("{}", serde_json::to_string_pretty(&philosophers_audit::check_mutants(Path::new(events), Path::new(output))?)?),
        [mode, events] if mode == "--audit-api" => println!("{}", serde_json::to_string_pretty(&api_audit::audit(Path::new(events))?)?),
        [mode, events, output] if mode == "--check-api-mutants" => println!("{}", serde_json::to_string_pretty(&api_audit::check_mutants(Path::new(events), Path::new(output))?)?),
        [mode, events] if mode == "--audit-review" => println!("{}", serde_json::to_string_pretty(&review_audit::audit(Path::new(events))?)?),
        [mode, events] if mode == "--audit-feature-delivery" => println!("{}", serde_json::to_string_pretty(&feature_delivery_audit::audit(Path::new(events))?)?),
        [mode, events, output] if mode == "--check-feature-delivery-mutants" => println!("{}", serde_json::to_string_pretty(&feature_delivery_audit::check_mutants(Path::new(events),Path::new(output))?)?),
        [mode, events] if mode == "--audit-coordination" => println!("{}", serde_json::to_string_pretty(&coordination_audit::audit(Path::new(events))?)?),
        [mode, events] if mode == "--audit-refactor" => println!("{}", serde_json::to_string_pretty(&refactor_audit::audit(Path::new(events))?)?),
        [mode, events] if mode == "--audit-discovery" => println!("{}", serde_json::to_string_pretty(&discovery_audit::audit(Path::new(events))?)?),
        [mode, events] if mode == "--audit-lab" => println!("{}", serde_json::to_string_pretty(&lab_audit::audit(Path::new(events))?)?),
        [mode, events] if mode == "--audit-lab-quality" => println!("{}", serde_json::to_string_pretty(&lab_quality_audit::audit(Path::new(events))?)?),
        [mode, events] if mode == "--audit-regulatory-discovery" => println!("{}", serde_json::to_string_pretty(&regulatory_discovery_audit::audit(Path::new(events))?)?),
        [mode, events] if mode == "--audit-medical" => println!("{}", serde_json::to_string_pretty(&medical_audit::audit(Path::new(events))?)?),
        [mode, events, output] if mode == "--check-medical-mutants" => println!("{}", serde_json::to_string_pretty(&medical_audit::check_mutants(Path::new(events),Path::new(output))?)?),
        [mode, events] if mode == "--audit-multi-specialty" => println!("{}", serde_json::to_string_pretty(&multi_specialty_audit::audit(Path::new(events))?)?),
        [mode, events] if mode == "--audit-critical-care" => println!("{}", serde_json::to_string_pretty(&critical_care_audit::audit(Path::new(events))?)?),
        [mode, events, output] if mode == "--check-critical-care-mutants" => println!("{}", serde_json::to_string_pretty(&critical_care_audit::check_mutants(Path::new(events),Path::new(output))?)?),
        [mode, events, output] if mode == "--check-multi-specialty-mutants" => println!("{}", serde_json::to_string_pretty(&multi_specialty_audit::check_mutants(Path::new(events),Path::new(output))?)?),
        [mode, events, output] if mode == "--check-lab-quality-mutants" => println!("{}", serde_json::to_string_pretty(&lab_quality_audit::check_mutants(Path::new(events),Path::new(output))?)?),
        [mode, events, output] if mode == "--check-regulatory-discovery-mutants" => println!("{}", serde_json::to_string_pretty(&regulatory_discovery_audit::check_mutants(Path::new(events),Path::new(output))?)?),
        [mode, events, output] if mode == "--check-lab-mutants" => println!("{}", serde_json::to_string_pretty(&lab_audit::check_mutants(Path::new(events),Path::new(output))?)?),
        [mode, events, output] if mode == "--check-discovery-mutants" => println!("{}", serde_json::to_string_pretty(&discovery_audit::check_mutants(Path::new(events),Path::new(output))?)?),
        [mode, events, output] if mode == "--check-refactor-mutants" => println!("{}", serde_json::to_string_pretty(&refactor_audit::check_mutants(Path::new(events),Path::new(output))?)?),
        [mode, events] if mode == "--audit-report" => println!("{}", serde_json::to_string_pretty(&report_audit::audit(Path::new(events))?)?),
        [mode, events] if mode == "--audit-document" => println!("{}", serde_json::to_string_pretty(&document_audit::audit(Path::new(events))?)?),
        [mode, events] if mode == "--audit-kitchen" => println!("{}", serde_json::to_string_pretty(&kitchen_audit::audit(Path::new(events))?)?),
        [mode, events, output] if mode == "--check-kitchen-mutants" => println!("{}", serde_json::to_string_pretty(&kitchen_audit::check_mutants(Path::new(events),Path::new(output))?)?),
        [mode, events] if mode == "--audit-manufacturing" => println!("{}", serde_json::to_string_pretty(&manufacturing_audit::audit(Path::new(events))?)?),
        [mode, events, output] if mode == "--check-manufacturing-mutants" => println!("{}", serde_json::to_string_pretty(&manufacturing_audit::check_mutants(Path::new(events), Path::new(output))?)?),
        [mode, events] if mode == "--audit-build" => println!("{}", serde_json::to_string_pretty(&build_audit::audit(Path::new(events))?)?),
        [mode, events] if mode == "--audit-cascading-build" => println!("{}", serde_json::to_string_pretty(&cascading_build_audit::audit(Path::new(events))?)?),
        [mode, events] if mode == "--audit-multi-round-discovery" => println!("{}", serde_json::to_string_pretty(&multi_round_discovery_audit::audit(Path::new(events))?)?),
        [mode, events, output] if mode == "--check-multi-round-discovery-mutants" => println!("{}", serde_json::to_string_pretty(&multi_round_discovery_audit::check_mutants(Path::new(events),Path::new(output))?)?),
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
            "usage: choreofix-experiments 1E|1M|1H|2E|2M|2H|3E|4E|4M|5E|5M|5H|6E|6M|7E|8E|9E|9M|9H|10E|10M|11E|12E|13E|13M|14E|14M|14H|16E DEPTH OUTPUT\n",
            "       --audit-api|--audit-review|--audit-report|--audit-document|--audit-coordination|--audit-refactor|--audit-discovery|--audit-lab|--audit-writing|--audit-cicd|--audit-philosophers|--audit EVENTS\n",
            "       --audit-protocol EVENTS\n",
            "       --audit-multi-round-discovery EVENTS\n",
            "       --check-multi-round-discovery-mutants EVENTS NEW_OUTPUT\n",
            "       --check-api-mutants|--check-review-mutants|--check-report-mutants|--check-document-mutants|--check-writing-mutants|--check-cicd-mutants|--check-philosophers-mutants|--check-protocol-mutants EVENTS NEW_OUTPUT\n",
            "       --audit-run RUN\n",
            "       --build-wasi NEW_OUTPUT FAMILY[,FAMILY...]\n",
            "       --qualify-wasi NEW_OUTPUT [DIAGNOSTIC_TEST_FILTER]\n",
            "       --check-run-mutants RUN NEW_OUTPUT"
        ).into()),
    }
    Ok(())
}
