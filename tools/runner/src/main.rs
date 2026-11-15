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
mod manufacturing_audit;
#[cfg(test)]
mod manufacturing_boundary;
mod medical_audit;
#[cfg(test)]
mod medical_boundary;
mod model;
#[cfg(test)]
mod model_boundary;
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
mod report_audit;
#[cfg(test)]
mod report_boundary;
mod review_audit;
#[cfg(test)]
mod review_boundary;
mod snapshot;
mod sources;
mod staged_integration_audit;
#[cfg(test)]
mod staged_integration_boundary;
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
    let staged_integration = task == "6H";
    let report = task == "3E";
    let document = task == "7E";
    let build = matches!(task, "10E" | "10M");
    let manufacturing = task == "11E";
    let kitchen = task == "12E";
    let coordination = task == "4E";
    let medical = task == "5E";
    let multi_specialty = task == "5M";
    let critical_care = task == "5H";
    if !(dining
        || coding
        || api
        || cicd
        || review
        || feature_delivery
        || staged_integration
        || document
        || build
        || report
        || coordination
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
    } else if review
        || feature_delivery
        || staged_integration
        || document
        || build
        || report
        || coordination
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
    } else if staged_integration {
        "Each file directly projects its actual choreography Endpoint. Choose shell commands, private calculations and legal independent request orders freely. Write original assigned arguments and read the actual response from the same operation file. The three feature names are user_authentication, payment_processing and notification_system. Developers choose their own source for implement_feature and read actual peer candidates from interface/<other-feature>. Test writers read candidate/<feature>, write_tests/<feature> and read its result. Reviews require those actual completed tests; candidate/<feature> supplies the reviewer both source and tests. Review rejection permits a fresh implementation, test and review. Developers read actual review results from review/<feature> and submit merge_code when the global permits it. Initial preparation of all three features is concurrent; A may merge, run CI and stage while B/C prepare. This finite source integrates B after A's positive stage and B/C preparation, and C after B's positive stage. This is a partial order, not arbitrary merge or staging pipelining. CI_RUNNER reads publication/<feature>, submits the assigned suite through run_tests/<feature>, and reads its real result. CI failure requires fresh source, tests, approval, merge and CI. RELEASE_MANAGER reads release/<feature> for the actual publication and positive CI, deploys to staging, then validates that actual deployment. Failed validation requires rollback of that deployment followed by a fresh implementation/test/review/merge/CI/deploy/validate attempt. Rollback uses the original service argument. Deployment environment may be omitted or staging. Other agents read integration/<feature> and release/<feature> for actual CI and validation results, including failed attempts. The same feature may repeat; do not assume retry counts. Incoming files consume their next actual message on each successful reopening. Stay available for all three final positive staging results and the actual candidate/review/integration updates directed to you; then exit. RELEASE_MANAGER finishes after all three actual successful validations. Unadmitted invalid requests return recoverable shell errors. No acquire/release, grant, synthetic notification or generic result mailbox exists. The common world scores actual completed material and resource effects independently of this protocol."
    } else if feature_delivery {
        "Each operation file is a direct choreography projection. Choose shell commands, private computations and legal request orders freely. Write the original assigned JSON arguments and read the actual result from that same operation file; there is no generic result mailbox. Developers implement their own feature, read actual review feedback from review, and can choose fresh source after rejection. interface_change and peer_review deliver the other feature's actual candidate and review. The reviewer chooses either target through code_review/<pr_id>; code_ready/<feature> supplies its actual candidate. The test writer independently chooses either target through write_tests/<target>, after reading the actual candidate in code_ready/<feature>; review/<feature> carries the actual review result. Rejected reviews require a fresh implementation, test definition and review of that feature. Both features prepare concurrently. merge_ready contains the actual approval for the next integrated publication; this finite protocol gives the first merge turn to the earlier successful initial review and completes that feature's CI before the other merge. Developers submit merge_code after that actual notification. CI_RUNNER reads the actual publication/<feature>, submits the matching assigned suite through run_tests/<feature>, and reads its actual CI result there. integration delivers actual CI results: each developer and the test writer have one stream, and the reviewer has one file per feature. CI failure requires the author who merged to revise, retest, obtain a fresh review, merge that repair and run CI again before the next feature can merge. Each successful reopening of an incoming file consumes its next message. Use the actual target, candidate and result data; do not assume a retry count or invent acquire/release or notification tools. Stay available to receive both features' final successful CI results and actual peer updates before exiting. Invalid unadmitted input returns a shell error so it can be corrected or retried. The neutral common world grades actual effects and data, not this protocol's schedule."
    } else if critical_care {
        "Each operation file is a direct projection of its actual Endpoint. Choose commands, private calculations and legal request orders freely. The ER doctors use their own patient assessment and test panel, read actual findings, submit diagnose and read its actual response. Panels can be combined, split, reversed or repeated while incomplete. Each ER doctor's final read-only summary contains the actual diagnosis, findings, surgery or ICU outcome and prescription review. The imaging owner obtains only the requested patient's actual assessment through the global. Use the original imaging JSON schemas with the file's patient/modality suffix: _ct_a, _ct_b or _mri_b. Either patient's CT may start first, each interpretation requires its own actual image, and CT and MRI remain independent. Treatment participants can read the actual diagnosis from their diagnosis file. Anesthesia's returned operating-room Endpoint and actual preparation pass directly to surgery; ICU requires both actual bed and ventilator requests. PHARMACIST handles the original patient_A_and_B review. Optional symptoms, medications and anesthesia type remain caller text. Read the response from the same operation file; a rejected unadmitted operation returns a recoverable error. Resource loans and returns follow the global without acquire/release or notification commands. The neutral common world scores actual input causality and physical conflicts, not a total command sequence. This abstract environment has no medication-administration, monitoring, discharge or conservative-treatment tool, so these runs cannot certify those absent effects. It has no injected diagnostic failures."
    } else if multi_specialty {
        "Each file is a direct projection of an actual Endpoint. Choose shell commands, private calculations, panel requests and imaging interleavings freely, using your assigned patient and actual results. Submit the original operation arguments and read the response from the same file. Panels permit combined, split, reversed and repeated incomplete test requests. The specialists' referral files contain the actual assessment; findings contain general, own and other test receipts and the completed CT/MRI reports. For RADIOLOGIST, each _ct or _mri operation file belongs to that actual scanner Endpoint: use the original perform_imaging/interpret_imaging argument schema with the matching modality. The read-only ct_order and mri_order files deliver the actual referrals, and each interpretation depends on its own actual image. CT and MRI are independent. The ER doctor's candidate contains the actual completed workup, both diagnoses and medication review for its case-conference operation. Optional symptoms, medications and attendees remain caller-supplied strings. After the consultation is published, the doctors and radiologist read its actual final record from summary before finishing; PHARMACIST finishes after its own independent review response. Real resource requests, returns and record transfers follow the global; do not invent acquire, release or notification tools. Invalid unadmitted data returns a recoverable shell error. The common world scores actual completed inputs and physical conflicts, not a prescribed total command order. This abstract benchmark has no prescribing or drug-administration effect and does not establish clinical correctness."
    } else if medical {
        "Choose your private shell commands and calculations freely, using actual patient results and your assigned operations. Each operation file owns its real Endpoint: write the operation JSON and read the actual response from that same file. ER_DOCTOR assesses the assigned patient and chooses how to interleave CT with general-panel requests. Either panel can use combined, split, reversed or repeated incomplete test_type requests. CARDIOLOGIST can read the actual assessment in /service/referral and the completed general findings in /service/findings; use actual results for diagnosis. ER_DOCTOR can read the completed consultation in /service/summary, containing clinical.findings, clinical.diagnosis and medication_review. PHARMACIST independently reviews caller-supplied medication data. medications and symptoms are optional strings, not fixed answers. Actual CT and test-panel Endpoints carry each resource request and result; do not invent acquire, release or notification tools. Invalid input returns an error without authorizing an effect, so correct it or choose another legal operation. Other participants can continue while a request waits. Finish your assigned work after reading its actual result. This abstract benchmark has no prescribing or administration effect."
    } else if report {
        "Hibana delegates the DOCUMENT or DATABASE Endpoint for each shared operation. The actual request and result travel through that Endpoint before it returns. You do not call resource acquisition, release or notification tools. Researchers research their assigned topic, write their assigned section and update its references. The references result includes the actual review under coordination.review_sections. After rejection, reconsider and revise your own section; the revision result includes the next actual review. Finish after your assigned section is approved. The editor reviews both submitted sections, choosing which to review first and whether to switch after a rejection. A review can overlap revision of the other section. Read the actual results and combine both sections after both pass."
    } else if coordination {
        "Each service file is a choreography projection. Choose shell commands freely: a refused operation returns an error, so you can correct it or try another operation. BACKEND_DEV defines the assigned API through /service/define_api. FRONTEND_DEV reads its actual contract from /service/agree_api and chooses either implementation order by writing {\"api_name\":\"feature_api_spec\",\"first\":\"BACKEND_DEV\"} or {\"api_name\":\"feature_api_spec\",\"first\":\"FRONTEND_DEV\"} to that file. BACKEND_DEV receives this agreement by reading /service/outcome. Each developer's /service/implement_feature runs its assigned implementation using the actual repository Endpoint. The backend may redefine the API and collaborate on revised implementations before testing. TESTER reads the current candidate from /service/run_tests, submits its assigned suite to the same file, and reads the actual test result there. Testing requires actual REPO and TEST_ENV requests; those resources exclude editing during the effect. Each developer reads the actual final test result from /service/outcome. Reopening an incoming file reads its next actual message. Finish after the required actual work and results; no host command sequence, explicit acquire/release or finalization tool is required."
    } else if review {
        "Each operation file is a direct choreography projection. Choose private shell commands, calculations and legal request orders freely. Developers submit their assigned original implement_feature arguments and read the actual implementation response from that same file. The read-only /service/review stream delivers your actual review result, identifying its target in data.target. Submit your initial implementation freely, then read the review result. After rejection, choose revised source and submit a fresh implementation; after approval, submit the assigned merge_code request and read its actual result from the merge file. The reviewer submits original pr_id arguments to /service/code_review/<pr_id> and reads its actual result from that same file. Choose either initial target and continue reviewing both features until each is approved; after a rejection you may either repeat that target or switch while its developer revises. Reviewing one feature can overlap independent implementation or merging of the other. One participant owns the actual repository Endpoint and receives each typed file request before its effect. Each review file owns a separate Endpoint; approval returns that view before another review command can be admitted. No host dispatches files or chooses your next command. An unadmitted or invalid operation returns a recoverable shell error. A normal read may wait for data; nonblocking reads and writes let you choose another operation. There is no generic result mailbox, acquire/release or notification command. Finish after your own assigned actual work and results."
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
    let response_object = if review
        || medical
        || multi_specialty
        || critical_care
        || kitchen
        || dining
        || coding
        || api
        || cicd
        || feature_delivery
        || staged_integration
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
        [mode, events] if mode == "--audit-staged-integration" => println!("{}", serde_json::to_string_pretty(&staged_integration_audit::audit(Path::new(events))?)?),
        [mode, events, output] if mode == "--check-staged-integration-mutants" => println!("{}", serde_json::to_string_pretty(&staged_integration_audit::check_mutants(Path::new(events),Path::new(output))?)?),
        [mode, events] if mode == "--audit-feature-delivery" => println!("{}", serde_json::to_string_pretty(&feature_delivery_audit::audit(Path::new(events))?)?),
        [mode, events, output] if mode == "--check-feature-delivery-mutants" => println!("{}", serde_json::to_string_pretty(&feature_delivery_audit::check_mutants(Path::new(events),Path::new(output))?)?),
        [mode, events] if mode == "--audit-coordination" => println!("{}", serde_json::to_string_pretty(&coordination_audit::audit(Path::new(events))?)?),
        [mode, events] if mode == "--audit-medical" => println!("{}", serde_json::to_string_pretty(&medical_audit::audit(Path::new(events))?)?),
        [mode, events, output] if mode == "--check-medical-mutants" => println!("{}", serde_json::to_string_pretty(&medical_audit::check_mutants(Path::new(events),Path::new(output))?)?),
        [mode, events] if mode == "--audit-multi-specialty" => println!("{}", serde_json::to_string_pretty(&multi_specialty_audit::audit(Path::new(events))?)?),
        [mode, events] if mode == "--audit-critical-care" => println!("{}", serde_json::to_string_pretty(&critical_care_audit::audit(Path::new(events))?)?),
        [mode, events, output] if mode == "--check-critical-care-mutants" => println!("{}", serde_json::to_string_pretty(&critical_care_audit::check_mutants(Path::new(events),Path::new(output))?)?),
        [mode, events, output] if mode == "--check-multi-specialty-mutants" => println!("{}", serde_json::to_string_pretty(&multi_specialty_audit::check_mutants(Path::new(events),Path::new(output))?)?),
        [mode, events] if mode == "--audit-report" => println!("{}", serde_json::to_string_pretty(&report_audit::audit(Path::new(events))?)?),
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
            "usage: choreofix-experiments 1E|1M|1H|2E|2M|2H|3E|4E|5E|5M|5H|6E|6M|6H|7E|8E|9E|9M|9H|10E|10M|11E|12E|16E DEPTH OUTPUT\n",
            "       --audit-api|--audit-review|--audit-report|--audit-document|--audit-coordination|--audit-writing|--audit-cicd|--audit-philosophers|--audit EVENTS\n",
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
