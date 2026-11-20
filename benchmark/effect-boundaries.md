# Shared effects at resource receivers

The model chooses shell commands in its own hibana-wash Wasm instance. WASI
requests reach ChoreoFS through projected Endpoint receives. A task object
contains request data; opening or writing it alone does not authorize its
business operation. The task global determines when the corresponding request
and resource Endpoint can be used.

A physical effect updates the versioned common simulator. In 6E (review),
6M (feature delivery), 7H (collaborative writing), 11E (machine jobs) and
16E (CI/CD), the resource receiver passes the actual Hibana `ReceivedPayload` to `ReceivedEffects::apply`. Its type fixes the
participant and operation. Its bytes supply the arguments, and passing it
consumes that receive. In 6M the submitted merge can be forwarded. Only the
repository-side request type authorizes the corresponding simulator operation.
In 11E the file submission can be forwarded to the machine. The technician
compares the returned machine request with that submission, then consumes
the machine request to execute the job.
In 7H the two figure-lookup messages carry data. The
document owner compares the figure store's confirmation with the original
lookup before consuming the confirmation to insert the figure. The two final
15E chamber checks also use received requests.

In 1E/1M/1H a commit consumes the two module requests. In 2E/2M/2H a write
consumes the two section requests. The receiver checks the submitted arguments
against the resource requests. The simulator call consumes both receives and
checks that their arguments agree before sending the request. The same
two-receive boundary applies to the integration test in 4E and the oven and
stove operations that require both in 12E. Operations requiring one resource,
including the 8M API operations, consume its received request directly.

In 5E the panel and scanner submissions carry the arguments to their resource
Endpoints. The simulator consumes the request received from the panel or
scanner. In 5M the conference receiver checks the case in the room request
against the collected findings and the arguments against the file submission.
The simulator consumes that same room receive and passes its arguments member
to the conference operation. In 5H the ICU receiver checks requests from the bed and ventilator
against the admitted patient data. One ICU effect consumes both receives.
In 6H the staging receiver consumes the deployment request, sends
the stage check and compares the reply with the requested service and previous
validations. The pending simulator call owns its arguments and sends them only
when awaited after those checks. Dropping it before then issues no RPC.

The other task receivers using `Domain::execute(agent, tool, arguments)` still
select the operation and arguments in native code. The experiment trusts their
implementation to match the received request and satisfy any additional resource
conditions. Sharing terminal, private-file or observation I/O gives a participant
no simulator entry point. Native integration binds the session, physical
resources and simulator process for the trial. A received payload carries no
external trial identifier. EPF and the effect auditor check the recorded run.

## Receiver contract

1. Receive each actual request through the projected Endpoint before the effect.
   For a shared operation requiring several resources, receive every required
   resource's request. Their arguments must agree.
2. If an earlier WASI close or business message admitted the arguments, compare
   the resource request with those actual arguments before calling the domain.
   A matching JSON schema alone does not establish that equality.
3. Bind the caller and target to the projected task participant and object.
   Preserve the original argument semantics, including intentionally ignored
   fields, and record stricter interface checks in the experiment contract.
4. Invoke the effect once for that received operation and return its actual
   result through the same global. Do not synthesize a successful result from
   protocol completion or reuse a previous result as a new operation's result.
5. Keep the requests, results and Endpoint returns in the global. Request data
   retained for comparison is data, not another permission token or phase flag.

Physical resource identity and aliasing are the initial binding integration's
responsibility. The transport must satisfy the selected core's delivery and
receiver-reachability contract. An I/O error or deadline does not revoke a
remote Endpoint or permit a new initial binding. An external effect may have
succeeded before its reply is lost; this contract does not supply exactly-once
external APIs or crash recovery.

## Research report qualification

The new 3E [global](../src/tasks/research_report/global.rs) and
[localsides](../src/tasks/research_report/localside) replace its old native
unit-grant implementation. The DOCUMENT and DATABASE owners receive the actual
request through the delegated resource Endpoint and compare it with the admitted
arguments before the effect. Final combination is outside the review repetition;
the DOCUMENT owner also compares both forwarded positive replies with the actual
review replies it received. Input codecs retain optional notes and feedback.

On the current checking core, this implementation passes 18 ordinary and eight
extended-rejection Wasm conditions, plus a dedicated invalid-input condition.
The latter rejects premature combination, malformed or wrong-target requests,
all four researcher operations after approval and revision after the actual
combine result, then completes the original task. These are actual guest
operations, separate from the native receiver-prefix tests. The source also
has one three-model pilot with original completion and passing EPF/effect
audits. The 28 primary records reject 6,797 static wire/business-record
mutations. This qualifies the declared mechanism cases; it is not a general
Rust refinement proof or the final core/consumer release qualification.

## Migrated task receivers

This inventory covers the twenty-seven migrated tasks using twenty-one Wasm task
families below. It does not qualify the other twenty-one task ports or legacy
native entry points. Source and executable identity are recorded separately for
each qualification.

| Tasks | Domain operations | Actual receiver and checks |
| --- | --- | --- |
| 1E/1M/1H | `design_feature`, `implement_code`, `run_local_tests` | [Developer](../src/tasks/codebase/choreofs/localside/developer.rs) and [reconsidering developer](../src/tasks/codebase/choreofs/localside/developer_reconsider.rs): typed close data, assigned feature, received ChoreoFS request. |
| 1E/1M/1H | `commit_changes` | The same receivers await both module requests, compare their bytes and materialize both ChoreoFS closes before the effect. The [codec](../src/tasks/codebase/choreofs/wire.rs) checks the assigned physical module pair. |
| 2E/2M/2H | `read_literature`, `draft_content`, `review_own_work` | [Researcher](../src/tasks/writing/choreofs/localside/researcher.rs): received file requests and the actual validation result for that close. |
| 2E/2M/2H | `write_to_document` | The researcher awaits both section requests and compares them with each other and the arguments admitted from the file close. |
| 3E | `research_topic`, `update_references` | [Resource receivers](../src/tasks/research_report/localside/resources.rs): assigned topic/section and the actual request; database operations compare admission and delegated request before the effect. |
| 3E | `write_section`, `revise_section`, `combine_sections` | The DOCUMENT receiver compares actual admission and delegated request. Revision requires the actual negative review; combination requires both forwarded positive replies to equal the actual replies received by the owner. |
| 3E | `review_sections` | [Reviewer A](../src/tasks/research_report/localside/reviewer_a.rs) and [reviewer B](../src/tasks/research_report/localside/reviewer_b.rs): actual typed request and target; actual result determines the typed reply and route. |
| 4E | `define_api`, `implement_feature` | [Repository receiver](../src/tasks/api_coordination/localside/resources.rs): actual resource request and assigned API/feature; backend admission must equal the delegated request. Actual results determine the finalized candidate. |
| 4E | `run_tests` | The same receiver consumes both the REPO and TEST_ENV requests, requires equal assigned arguments and the current candidate's actual API/implementation results, and returns the one actual test result through both resource Endpoints. |
| 4M | `design_proposal`, `refactor_code`, `run_tests`, `merge_code` | [Repository receiver](../src/tasks/shared_refactor/localside/resources.rs): actual normal admission and delegated requests; both REPO/TEST_ENV requests for tests; merge requests must equal the current candidate, actual tests and actual review results before the effect. |
| 4M | `code_review` | [Review receiver](../src/tasks/shared_refactor/localside/reviewer.rs): actual typed request and assigned target; returns the actual result with the received candidate and arguments. The common simulator checks published contents against completed content-bound validation receipts. |
| 13E | `run_analysis`, `run_separation`, `run_bioassay` | [Instrument receivers](../src/tasks/lab_synthesis/localside/instruments.rs): actual WASI admission equals the instrument request before the effect. Bioassay compares actual preparation/purification; final validation also compares the received assay and both quality results with the owners' actual results. The common scorer separately checks completed input occurrences and physical conflicts. |
| 13M | `run_synthesis`, `run_analysis`, `run_separation`, `run_bioassay`, `verify_result` | [Resource owners](../src/tasks/lab_quality/localside): compare actual WASI admissions, received scientific data and both required QC instrument requests before each effect. Actual failed results drive a fresh measurement/trial. Returned approvals and final validation refer to actual current data; no owner/progress table is introduced. |
| 5E | `assess_patient`, `order_tests`, `perform_imaging`, `diagnose`, `review_prescriptions` | [Clinical receivers](../src/tasks/medical_consultation/localside/services.rs): each effect follows its received request. CT arguments must equal the admitted intent. Test-panel Endpoints return for partial results and leave the retry owner at completion. Referral/findings carry actual results. The auditor consumes actual file admissions, requests and results once; EPF checks dynamic order. Absent prescribing/administration effects are not certified. |
| 5M | `assess_patient`, `order_tests`, `perform_imaging`, `interpret_imaging`, `diagnose`, `review_prescriptions`, `conduct_case_conference` | [Multi-specialty receivers](../src/tasks/multi_specialty/localside/services.rs): actual panel/image/report artifacts reach the diagnosis and conference receivers. Each scanner request must equal the admitted WASI arguments. The real room request must equal its admission before the one conference effect. Changed patient, modality, attendees and diagnosis inputs are tested against actual receivers; the Domain invocation journal stays empty on rejection. |
| 5H | Patient assessment/panels/diagnosis, CT/MRI imaging and interpretation, anesthesia, surgery, ICU and prescription review | [Patient](../src/tasks/critical_care/localside/patients.rs), [imaging](../src/tasks/critical_care/localside/imaging.rs) and [care](../src/tasks/critical_care/localside/care.rs) receivers bind actual patient artifacts and admitted arguments to real resource requests. The same OR Endpoint moves from anesthesia to surgery. ICU receives matching requests through both bed and ventilator Endpoints before one effect. Twelve substituted requests are rejected with zero Domain invocations, including both ICU sides, and pass strict-provenance Miri. Eight actual-Wasm conditions and a seven-model pilot pass EPF and the separate effect auditor. |
| 14E | `run_hplc_analysis`, `run_mass_spec_analysis`, `run_cell_assay` | [Instrument receivers](../src/tasks/drug_discovery/localside/instruments.rs): compare the accepted WASI argument with the actual request received through the instrument Endpoint before the effect; return the actual result. Final documentation also carries the actual decision and completed analysis data. The common simulator checks assigned analyses and physical instrument conflicts without enforcing the ChoreoFix operation order. |
| 6E | `implement_feature`, `merge_code` | [Repository](../src/tasks/review/choreofs/localside/repo.rs): the repository consumes the actual typed file request for the assigned feature or branch. |
| 6E | `code_review` | [Feature reviewers](../src/tasks/review/choreofs/localside/reviewer.rs): each consumes its typed review request after receiving the successful implementation. Its actual result feeds the resolver. |
| 6M | `implement_feature`, `write_tests`, `code_review`, `merge_code`, `run_tests` | [Repository](../src/tasks/feature_delivery/localside/repository.rs), [review](../src/tasks/feature_delivery/localside/reviews.rs) and [CI](../src/tasks/feature_delivery/localside/ci.rs) receivers match actual requests to the current source, completed review/test receipts and published occurrence. CI receives both actual feature and BUILD_SERVER requests. [6M](ports/6M.md) records recovery, resource boundaries and qualification. |
| 7E | `write_section`, `update_references`, `review_sections`, `revise_section`, `combine_sections` | [Document receiver](../src/tasks/shared_document/localside/document.rs): actual admitted arguments and delegated document requests; actual review results and current document data accompany revision and final combination. Qualification is bound to the sources recorded in [7E](ports/7E.md). |
| 10E | `compile_module`, `link_modules` | [Build receiver](../src/tasks/shared_build/localside/resources.rs): both resource requests agree with the admitted compile arguments; linking checks both actual current objects and the requested modules. The shared world independently grades actual input contents. |
| 10M | `compile_module`, `run_build_tests`, `link_modules` | [Compiler](../src/tasks/cascading_build/localside/compiler.rs) and [test receiver](../src/tasks/cascading_build/localside/testing.rs): actual admitted arguments, paired resource requests, current objects and completed positive test receipts. Substituted compiler arguments are rejected before any effect in native/Miri. [10M](ports/10M.md) records the source-bound qualification. |
| 11E | `do_job`, `collect_reports` | [Lathe](../src/tasks/manufacturing/localside/lathe.rs) and [mill](../src/tasks/manufacturing/localside/mill.rs) receivers consume the actual workstation request, validate the assigned job and return its actual result. The [report receiver](../src/tasks/manufacturing/localside/reports.rs) checks four actual completed results and the actual report reply. Four invalid wire inputs are rejected by the production lathe receiver before any effect in native/Miri. [11E](ports/11E.md) records the finite qualification and preserved first-pilot failure. |
| 12E | `prepare_base_dish`, `prepare_ingredient`, `combine_dish` | [Cooking](../src/tasks/kitchen/localside/cooking.rs) consumes actual oven and, for A/B, stovetop requests and compares them with the admitted input before one effect. [Ingredients](../src/tasks/kitchen/localside/ingredients.rs) sends the actual completed result to its producer and consumer; [finish](../src/tasks/kitchen/localside/finish.rs) consumes the actual typed combination request. Independent EPF/effect audits connect those receives with the original completed base and ingredient occurrences. [12E](ports/12E.md) bounds the thirteen Wasm conditions, pilot and unchanged receiver Miri coverage. |
| 8E | `migrate_schema`, `implement_backend`, `run_integration_tests` | [API receivers](../src/tasks/api/choreofs/localside): received file requests, validation and actual resource requests. Resource codecs accept only the original task's fixed migration/suite arguments. |
| 9E/9M/9H | `think`, `eat` | [Philosopher](../src/tasks/philosophers/choreofs/localside/philosopher.rs): actual request; eating awaits both fork requests with equal payloads. The projected caller determines the agent, matching the original simulator. |
| 16E | `compile_and_test`, `run_integration_tests`, `deploy_application` | [CI/CD receivers](../src/tasks/cicd/choreofs/localside): received file close followed by the actual infrastructure request. Each receiver compares those two argument values before its effect. |

## Evidence and remaining obligations

13M is the eighteenth migrated task, with source-bound finite qualification.
Its [centrifuge and biological receiver](../src/tasks/lab_quality/localside/centrifuge.rs)
compares admitted/resource arguments and checks actual preparation, purification,
calibration and compound data before trial effects. Seven direct receiver
conditions pass native/Miri: three preparation cases and four substituted trial
inputs. The latter execute only the two fixture separations, with no biological
effect. Its [spectrometer](../src/tasks/lab_quality/localside/spectrometer.rs) and
[analyst](../src/tasks/lab_quality/localside/analyst.rs) bodies compile and pass
Clippy and are exercised by the complete four-Wasm application.
The [synthesis and QC owner](../src/tasks/lab_quality/localside/fume.rs) is now
written. Three synthesis and four first-CHEMIST QC conditions pass native tests.
The QC fixture drives all three production instrument receivers: changed
admission evidence and either substituted resource request cause no QC effect;
unchanged requests produce one effect and identical actual replies. The QC
result branches, both first subjects, repeated and switching QC rejections pass
all ten real-Wasm conditions, with common causal scoring, EPF dynamic audit and
admission/effect audit. There are no violations; 198 saved-record mutations are
rejected. One four-model Luna/xhigh pilot also passes those audits: 86.04 seconds,
710,689 inclusive tokens and 17 actual effects. The offline dynamic/effect audit
takes 2,179.55 seconds. The original test was explicitly stopped during its
first audit; the preserved model/runtime evidence was independently reaudited
without repeating model calls. This qualifies the recorded sources and finite
conditions, not all reconsideration paths, final all-family certification or a
scored comparison. See the [13M scope and evidence](ports/13M.md).

The [experiment contract](experiment-contract.json) and [work plan](../plan.md)
distinguish qualification on a local core patch from qualification on the normal
Git dependencies. Actual-Wasm tests exercise model-selected shell interfaces;
native Endpoint probes can also bypass the intended guest localside and test
the production receiver directly. Record mutations measure auditor sensitivity
and are separate from runtime rejection before an effect.

The [6E regression](../src/tasks/review/choreofs/tests.rs) issues a late
rejection directly through a real Endpoint after approval, on both the direct
and previously rejected paths. It checks that the late call fails without
emitting a frame and that the approved feature can still merge.
The [CI/CD receiver test](../src/tasks/cicd/choreofs/tests.rs)
substitutes a compile argument after admission through an actual received
Endpoint. That test exposed a missing receiver check: the old receiver invoked
the domain with the substituted argument. The corrected receiver rejects it
before any domain call. This checks the production receiver directly, rather
than relying on the intended guest to forward the admitted arguments.

These host-side probes deliberately bypass the guest adapter. A substituted
host request ends that run; the receiver does not roll back a completed
Endpoint receive. Ordinary malformed guest arguments have their existing
rejection route before resource use, so the model can revise them and continue.

T3/T4 must connect resource custody, remaining source protocol and this adapter
contract. Their composition, target/version binding in the new artifact cases,
all effect entries in the remaining task ports, and final dependency
qualification remain required. This inventory is not a formal proof that every
host call obeys the contract.
