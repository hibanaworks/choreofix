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
    let objects = if medical {
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
            "INTEGRATOR" if task == "10M" => "/service/link_modules; read-only /service/candidate",
            "INTEGRATOR" => "/service/link_modules",
            _ => return Err("unknown shared-build participant".into()),
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
            "BACKEND_DEV" => "/service/define_api, /service/implement_feature and /service/outcome",
            "FRONTEND_DEV" => "/service/agree_api, /service/implement_feature and /service/outcome",
            "TESTER" => "/service/run_tests",
            _ => return Err("unknown coordination participant".into()),
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
        "Each operation file owns its actual projected Endpoint. Choose shell commands, private computation and the order of your base/ingredient work freely. Submit the original assigned arguments and read the actual response from that same file. Ordinary redirection is blocking. For nonblocking submission, wash supports pipeline input to dd of=FILE oflag=nonblock; an unprepared operation returns a WASI error so you can choose independent work or retry. The actual oven and, for chefs A/B, stovetop Endpoints carry each base request/result and return. Ingredient preparation needs no equipment; the completed material is delivered directly to its assigned recipient. Combine your dish after its own base and required incoming ingredient exist. Your finished dish need not wait for unrelated bases or your outgoing ingredient. Each assigned effect runs once. Complete all your own assigned work and read its actual results before exiting. Do not invent acquire/release or notification calls. The common world grades completed material inputs and physical conflicts, without prescribing a total command order."
    } else if manufacturing {
        "Each operation file owns its actual projected Endpoint. Choose your shell commands, private computations and assigned job order freely. Submit the original do_job arguments with your assigned job_id to do_lathe_job or do_mill_job and read the actual result from that same file. Ordinary redirection is blocking. For a nonblocking submission, wash supports pipeline input to dd of=FILE oflag=nonblock; an unprepared request returns a WASI error without starting that job. You can then choose the other assigned machine or retry later. Independent machines can work concurrently. Actual workstation Endpoints transfer through the choreography and are returned after use; do not invent acquire/release or notification calls. Actual completed job results reach the supervisor directly. The supervisor submits {} to collect_reports and reads the actual response from that same file; the actual collection waits for all four completed job results. Finish after receiving your actual assigned results. The common world grades actual completed report contents and machine conflicts, not a prescribed total command order."
    } else if task == "10M" {
        "Each operation file is a direct choreography projection. Choose shell commands and private computation freely. An unadmitted request returns a recoverable WASI error. Each builder uses compile_module for its assigned module and reads the actual result from that same file. Its read-only updates, test_result, reconsideration and outcome files deliver actual shared-input changes, its test result, reconsideration data, and the final linked outcome respectively. Independent file reads remain available; an unavailable operation is an error rather than permission to invent an effect. A reconsideration names its module and actual old object/test; the builder can compile a fresh object against current inputs. Reopening an incoming file consumes its next actual message. TEST_RUNNER reads next_object for an actual compiled object, submits its original run_build_tests arguments through that module's run_build_tests_a, run_build_tests_b or run_build_tests_c file, and reads the actual test result from that same file. next_object eventually delivers the final linked outcome. The integrator reads its actual candidate (objects and test receipts) from /service/candidate, requests link_modules for its three modules and reads its actual result; publication requires current positive tests and return of all editing Endpoints. Real resource transfers and message delivery follow the global; do not invent acquire/release or notification tools. Work independently and concurrently where permitted; use actual outcomes for reconsideration rather than a fixed attempt count. Exit after your actual final linked outcome. The common world checks actual object versions, test inputs and linked bytes, not a prescribed invocation order."
    } else if build {
        "The two actual resource Endpoints transfer automatically; no acquisition, release or notification commands are needed. Compile results contain the actual object bytes and shared type inputs. A updates the provided SHARED_TYPES definitions. B's read-only /service/updates delivers A's actual completed compile result and changed types. A blocking read can wait for that actual incoming message; nonblocking shell I/O can inspect availability instead. B may initially compile against the provided old definitions: that object is real but becomes stale when A updates them. Use the received update and actual object inputs to decide whether B needs a rebuild through the same compile service. A current object's SHARED_TYPES.modules includes module_a's declarations. The integrator's link waits for both current actual build reports. Private commands and computations remain your choice, and the common world scores the actual object and link inputs rather than a prescribed total invocation order."
    } else if document {
        "Each file is a direct choreography projection. The editor chooses the first writer through allocate_document with {\"first_writer\":\"WRITER_A\"} or WRITER_B. Writers' initial write and references target their assigned chapter; these operations are independent. Both writers may revise either chapter when editing is requested. Reading feedback consumes the next actual reconsideration review or the final combine_sections result; a review scope identifies the assigned writer. Both positive and negative current reviews may be reconsidered before freezing. The editor submits original review_sections arguments to review_chapter_1 or review_chapter_2 and reads the actual result from that same file. Either review order is allowed, including submitting both before reading results. Each successful reopen of an incoming file consumes its next message; private files can retain actual results. To request editing, reconsider_a or reconsider_b takes {\"review_id\":\"the actual current review_id\"}; its response is the actual revised material. Fresh reviews refer to the resulting complete document. combine_sections is the actual publication request and takes the original two-chapter sections argument. Publication requires both actual current reviews to be successful and match current material. The choreography transfers editing authority before the effect; there is no separate approval-submission operation. Writers remain active for reconsideration until feedback delivers the actual combine_sections publication result; completing the initial write and references is not their completion condition. Exiting before that result aborts the run. The editor remains active until it receives the actual publication result from combine_sections. Choose your own commands, calculations, legal request orders and reconsiderations. A refused unadmitted request returns an error to the same guest; no host chooses a next command or workflow step."
    } else if feature_delivery {
        "Use your assigned feature, branch, review target, test target or suite. The reviewer and test writer choose their initial feature order. Actual responses carry coordination data: reviewed candidates, tests, interface-change data and CI outcomes. Rejected reviews and failed CI require fresh implementation, tests and review for that feature; continue from the actual response rather than assuming a fixed retry count. Merge returns the actual CI outcome. A wrong CI suite returns invalid_arguments with the actual publication so it can be corrected. The final responses report both features' CI outcomes. Finish the required work and receive the final coordination outcomes before exiting; independent participants and operations remain concurrent."
    } else if critical_care {
        "Use your assigned patient and actual results. Each ER doctor assesses its own patient, requests the assigned panel (together or as partial tests), then requests diagnose. A final diagnosis response contains the actual findings, surgery or ICU outcome, and medication review. Neither patient's care requires the other patient's assessment or completed care. The radiologist chooses the order of the two patients' CT operations and patient B's MRI operations, keeping each interpretation after its matching actual image. The anesthesiologist prepares patient A; the actual operating-room Endpoint and preparation then travel to surgery. The surgeon performs the assigned procedure on patient A. ICU_COORDINATOR requests the actual patient B care with both bed and ventilator Endpoints. PHARMACIST reviews the assigned patient_A_and_B medication data. Resource loans and result forwarding happen through the global; do not invent acquisition, release or notification commands. Symptoms, medications and anesthesia type are optional caller strings. Choose your private shell commands and calculations freely. Read /service/result after each request and exit when your own work is complete. This abstract environment has no medication-administration, monitoring or discharge tool, and these runs cannot certify those absent effects. Its diagnostic failure injection is disabled; a depth number is not evidence of an injected-failure test."
    } else if multi_specialty {
        "Each file is a direct projection of an actual Endpoint. Choose shell commands, private calculations, panel requests and imaging interleavings freely, using your assigned patient and actual results. Submit the original operation arguments and read the response from the same file. Panels permit combined, split, reversed and repeated incomplete test requests. The specialists' referral files contain the actual assessment; findings contain general, own and other test receipts and the completed CT/MRI reports. For RADIOLOGIST, each _ct or _mri operation file belongs to that actual scanner Endpoint: use the original perform_imaging/interpret_imaging argument schema with the matching modality. The read-only ct_order and mri_order files deliver the actual referrals, and each interpretation depends on its own actual image. CT and MRI are independent. The ER doctor's candidate contains the actual completed workup, both diagnoses and medication review for its case-conference operation. Optional symptoms, medications and attendees remain caller-supplied strings. After the consultation is published, the doctors and radiologist read its actual final record from summary before finishing; PHARMACIST finishes after its own independent review response. Real resource requests, returns and record transfers follow the global; do not invent acquire, release or notification tools. Invalid unadmitted data returns a recoverable shell error. The common world scores actual completed inputs and physical conflicts, not a prescribed total command order. This abstract benchmark has no prescribing or drug-administration effect and does not establish clinical correctness."
    } else if medical {
        "Choose your private shell commands and calculations freely, using actual patient results and your assigned operations. Each operation file owns its real Endpoint: write the operation JSON and read the actual response from that same file. ER_DOCTOR assesses the assigned patient and chooses how to interleave CT with general-panel requests. Either panel can use combined, split, reversed or repeated incomplete test_type requests. CARDIOLOGIST can read the actual assessment in /service/referral and the completed general findings in /service/findings; use actual results for diagnosis. ER_DOCTOR can read the completed consultation in /service/summary, containing clinical.findings, clinical.diagnosis and medication_review. PHARMACIST independently reviews caller-supplied medication data. medications and symptoms are optional strings, not fixed answers. Actual CT and test-panel Endpoints carry each resource request and result; do not invent acquire, release or notification tools. Invalid input returns an error without authorizing an effect, so correct it or choose another legal operation. Other participants can continue while a request waits. Finish your assigned work after reading its actual result. This abstract benchmark has no prescribing or administration effect."
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
        "Each service file is a choreography projection. Choose shell commands freely: a refused operation returns an error, so you can correct it or try another operation. BACKEND_DEV defines the assigned API through /service/define_api. FRONTEND_DEV reads its actual contract from /service/agree_api and chooses either implementation order by writing {\"api_name\":\"feature_api_spec\",\"first\":\"BACKEND_DEV\"} or {\"api_name\":\"feature_api_spec\",\"first\":\"FRONTEND_DEV\"} to that file. BACKEND_DEV receives this agreement by reading /service/outcome. Each developer's /service/implement_feature runs its assigned implementation using the actual repository Endpoint. The backend may redefine the API and collaborate on revised implementations before testing. TESTER reads the current candidate from /service/run_tests, submits its assigned suite to the same file, and reads the actual test result there. Testing requires actual REPO and TEST_ENV requests; those resources exclude editing during the effect. Each developer reads the actual final test result from /service/outcome. Reopening an incoming file reads its next actual message. Finish after the required actual work and results; no host command sequence, explicit acquire/release or finalization tool is required."
    } else if review {
        "Hibana delegates the repository Endpoint during implementation and merging; each actual request and result travels through it before it returns. Code review does not hold the repository. You do not call resource acquisition, release or notification tools. A developer's implementation response includes the actual review under coordination.code_review. If review fails, reconsider and implement again; merge your assigned branch after approval. The reviewer chooses the order of the assigned features and may switch to the other feature after a rejection while the developer revises. Review both features until each passes. Reviewing one feature can overlap independent work on the other feature. Read every actual result and finish after your own assigned work is complete."
    } else if cicd {
        "Your operation file is a choreography projection. Choose shell commands freely; an unadmitted operation returns an error so the same guest can correct it or try another operation. Reading that same file receives an actual message with {kind,result}. DEVELOPER receives compile_result after its operation, then deployment_complete. TESTER receives build_complete, submits its own test request, receives test_result, then deployment_complete. DEPLOYER receives tests_passed, submits its own deployment request, and receives deploy_result. Each successful reopen reads the next actual message. Every result contains the actual shared-environment response. No host chooses your commands or lends a permission in advance; the file close itself is the request. The real build-server or deployment-environment Endpoint carries the exact admitted arguments and actual result before returning. You do not call acquire/release or notification tools. Finish after your assigned work and required actual notifications. This original task has no injected domain failures or business retry loop; malformed or premature shell operations remain recoverable."
    } else if dining {
        "Your think and eat objects are bound to their actual projected Endpoint operations. Eating uses both actual fork Endpoints at the effect owner and returns them afterwards. You do not call acquire/release tools. The philosopher string is caller-supplied display data; it does not change your participant or its physical fork pair. Read each actual result from the same operation object. Both effects run once, then finish."
    } else if task == "1H" {
        "Each operation file owns its actual projected Endpoint. Choose your shell commands and private computations freely. A refused operation returns an error to the same guest, permitting correction or another operation. Design and implement your assigned feature. Committing consumes both actual module requests and returns both module Endpoints before testing. The native localsides drive those Endpoints directly; no host dispatches requests or chooses your next command. Read each actual response from the same operation file. A negative test contains the actual tested source and diagnostic. Reconsider, implement the repair, commit it and test again; either other participants may continue independently. Finish after your actual tests pass. No acquire/release or notification tools are needed."
    } else if coding {
        "Each operation file owns its actual projected Endpoint. Choose shell commands and private computations freely. A refused request returns an error so that you can correct it or choose another operation. Design and implement your assigned feature. The localsides drive both actual module Endpoints for the commit and return them before testing. Read the actual result from the same operation file; no host dispatches requests or chooses your commands. The actual test outcome is terminal, including failure; finish your own work after reading it. No acquire/release or notification commands are needed."
    } else if api {
        "Hibana passes the database or test-environment Endpoint to the WASI participant. The request and actual result travel through that Endpoint before it returns. Migration completion precedes backend work, and backend completion precedes integration testing. You do not call acquire/release or notification tools. Use your assigned arguments. DB_ADMIN and BACKEND_DEV must also read the read-only /service/outcome object, which contains the original test result. That read may wait until the test finishes while other participants continue. A failed integration test is terminal in the task description and simulator. Report its actual result and finish, without adding a revision or retry."
    } else {
        "Each file is a direct choreography projection. Choose shell commands and private computations freely. An unadmitted operation returns an error to the same guest, allowing correction or another operation. read_literature returns actual source material; draft_content uses that completed material; write_to_document uses your actual draft and both assigned section resources; review_own_work examines that actual contribution. The localsides drive the real section Endpoints and return the actual results. No host selects your commands or dispatches accepted files to another workflow. Read each actual response from the same operation file. Use your assigned topic, section or resource pair. Do not invent acquire/release or notification commands. The actual review outcome is terminal, including failure; finish your own work and exit after reading it."
    };
    let response_object = if medical
        || multi_specialty
        || kitchen
        || dining
        || coding
        || api
        || cicd
        || document
        || report
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

Your writable domain objects are {objects}.
Write the matching operation's JSON argument object and close it to request
the operation. Shell redirection closes its output after writing.
{response_object} These objects are not commands.
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
        [mode, events, output] if mode == "--check-philosophers-mutants" => println!("{}", serde_json::to_string_pretty(&philosophers_audit::check_mutants(Path::new(events), Path::new(output))?.1)?),
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
