# Experiment runner

The runner records model requests, tool events, versions and usage. Hibana task
permissions belong to the projected Endpoints, not this process interface.
The direct-WASI command currently produces unscored qualification records.

The 13M direct QC port's `lab_quality_boundary::` entry requires all eight
actual four-Wasm conditions: both initial subjects, depths 0/1/2, reconsideration
and a normal-delay overlap case. All pass EPF and independent scientific
request/material/effect/reply/common-world checks (112 effects, 37,128 capture
records). Eighteen record counterexamples are refused. The four-Luna/xhigh
pilot also completes, but its independent run audit is pending; 13M is not yet
included in the qualified task count. See [13M](../../benchmark/ports/13M.md).

The 13E direct scientific port is source-bound qualified as task 26 of 48. Its declared
`lab_boundary::` domain has 24 causal-order/quality-order/culture combinations
and one normal-delay parallel-instrument case. The physical instrument owners
receive actual Endpoint requests and exact material data before effects. Its
current scope is recorded in [13E](../../benchmark/ports/13E.md).
All 25 conditions passed with 200 effects and 55,648 capture records. Three
Luna/xhigh participants completed the same source in 76.462663417 seconds,
564,435 inclusive tokens, eight effects and 4,044 complete EPF records. Actual
request/material/reply and common-world audits passed with zero violations.

The new 4M direct-file port uses five actual Wasm guests and twelve original
effects. Its declared eight-case domain is `refactor_boundary::`; all eight
cases and one five-person Luna/xhigh trial passed complete EPF, effect/reply and
common-world audits. The live run took 79.728423 seconds and 687,694 inclusive
tokens; it was the 25th source-bound task. Common-environment-v24 exposes current
code identities in refactor responses for every method without changing scoring
or permitted calls. See [the current port](../../benchmark/ports/4M.md).

The common WASI qualification declares both 6H seven-guest tests under
`staged_integration_boundary::`: the depth-0 baseline and depth-2 recovery.
The latter requires two review rejections, two CI failures and two staging
failures/rollbacks before completion, for 53 actual effects. Both use actual
Wasm with no model calls. `wasi-v92` passes both after the staging-owner
handoff: 15,587 / 32,522 complete EPF records, effect/world audit passed,
zero violations. Both share runtime SHA-256
`c17c574596ea3f48d01e84b3ca2ca6fd8ba52162c16f9a730cc70e3bacf2cbe2`.
The baseline checks 21 real effects and five refused requests followed by recovery.
The additional probes leave peer reviews or earlier CI unread, then verify
that the same guest recovers by consuming its actual inputs. The shell chooses
its commands; the runner does not drain those messages on its behalf.
The malformed-JSON probe checks that wash names the failed close and path,
then accepts corrected data from the same guest without an extra domain effect.
The later `wasi-directory-v97` checks actual nested `ls` output and the recovery
path's nonblocking early reads. Both seven-Wasm conditions pass: 21 / 53 effects,
15,906 / 32,918 complete EPF records, common-world completion and zero
violations. Runtime SHA-256 is
`295b8bdf4ae086fd2e22cced011c66d531954799fd7ec13794ffdd63b5edcfc2`.
These records include the caller-owned ChoreoFS object-storage and directory
repair; the older `wasi-v92` results remain bound to their earlier source.
These finite mechanism conditions alone do not qualify a model trial. The
separate `live-directory-v101` seven-model depth-2 run now passes the complete
run audit on that same runtime: 53 effects, 50,851 complete EPF records, neutral
world completion and zero violations. It takes 517.343330458 seconds and
8,533,959 inclusive tokens. Its source-bound qualification is
`qualification-live-v102.json`. The normal runner accepts 6H;
saved executions use `--audit-staged-integration EVENTS`, with record
counterexamples through `--check-staged-integration-mutants EVENTS NEW_OUTPUT`.
Use the experiment matrix and qualified task artifacts for scored comparisons.
The model-trial budget is defined in experiment.json (now 1,800 seconds for
every future condition). Old 600-second pilots keep their original budget and
are reported separately. Deadline exhaustion measures the bounded trial; it
does not establish a choreography or localside defect. The seven-model 6H
depth-0 pilot live-v78 retains its old source and budget. The new depth-2
qualification above uses the uniform 1,800-second budget.

The trace file and versioned common simulator each have one I/O owner. Callers send
records or simulator requests through channels. A record returns after its
write and flush complete, including any I/O error. The simulator starts the
common environment's asynchronous tools concurrently and returns each actual result on its
request's reply channel.

The guest process and model connection run as scoped Hibana localsides with
borrowed inputs. Their I/O owners retain descriptors, partial JSON lines and
RPC request IDs. External callbacks enqueue the actual reply and notify the
owning reader through hibana-runtime. Calls do not allocate spawned tasks or
require a task-result collection loop. Already accepted simulator requests are
observed before scoring, including after a guest error. Cancellation does not
undo their effects. The runner and its tests have no Tokio dependency.

Runs and boundary tests require a selected runtime and its build receipt.
Set `CHOREOFIX_<FAMILY>_WASI_BINARY` and
`CHOREOFIX_<FAMILY>_BUILD_RECEIPT`, where `FAMILY` is `CODEBASE`, `WRITING`, `API`,
`CICD`, `REVIEW`, `FEATURE_DELIVERY`, `REPORT`, `DOCUMENT`, `COORDINATION`, `REFACTOR`,
`DISCOVERY`, `REGULATORY_DISCOVERY`, `MULTI_ROUND_DISCOVERY`, `LAB`, `LAB_QUALITY`,
`MEDICAL`, `MULTI_SPECIALTY`, `CRITICAL_CARE`,
`BUILD`, `CASCADING_BUILD`, `MANUFACTURING`, `KITCHEN` or `PHILOSOPHERS`, as applicable to
the task. A receipt contains
`binary_sha256`, `source_manifest`, `source_manifest_sha256` and
`source_unchanged: true`, plus `source_git` recording each repository's Git base
and dirty patch. Resolve the input manifest relative to the receipt.
The build must compare that source manifest before and after compilation.
It includes the application and every selected local dependency. Manifest paths
are relative to the evidence workspace: `source/` holds the application and
`hibana/`, `hibana-epf/` and `hibana-wasip1-runtime/` hold the adjacent libraries.
The relative Cargo dependencies survive moving the complete bundle. Missing,
changed or newly added inputs prevent reusing a build receipt.

Each evidence bundle saves the checked manifest, original receipt and a receipt
whose manifest path is relative to the bundle. Missing or mismatched files stop
the qualification before starting a guest. Existing build archives cannot be
overwritten. These records identify the measured build; they do not establish
that its implementation or task outcome is correct.

The normal command and live subscription tests use the same execution path.
It supports `1E`, `1M`, `1H`, `2E`, `2M`, `2H`, `3E`, `4E`, `4M`, `5E`, `5M`,
`5H`, `6E`, `6M`, `6H`, `7E`, `8E`, `9E`, `9M`, `9H`, `10E`, `10M`, `11E`, `12E`,
`13E`, `13M` and `16E`, with the original failure conditions `0`, `1` and `2`.
Each qualification retains its exact source and simulator version. Twenty-six
source-bound tasks have completed direct-projection mechanism and live-model
checks; the other 22 remain unqualified. The older 4M, 13E/13M and 14E/14M/14H
adapters and their execution/audit commands have been removed. Their original
worlds, descriptions and historical source-bound results remain. These tasks
other than 4M/13E still require direct qualification; 13M's current mechanism
domain is implemented above. The new qualification is separate from those
retired adapters.
The rewritten 7E passes
twelve real-wash mechanism cases, EPF/effect audits and 1,503 record mutations.
Three Luna/xhigh participants also completed depth 2 on the same runtime binary;
the same run passes the model, EPF, effect and common-world audits. Its actual
publication request selects the global's final branch directly; there is no
separate approval-submission operation. These are finite, unscored results. The
original `12E`, `13E` and `14E` environments have no injected failures. Run from the
repository root and choose a new output directory:

```sh
export PYTHONHASHSEED=0 PYTHONDONTWRITEBYTECODE=1
export CHOREOFIX_REVIEW_WASI_BINARY=/path/to/review-runtime
export CHOREOFIX_REVIEW_BUILD_RECEIPT=/path/to/build-receipt.json
cargo +stable run --locked --manifest-path tools/runner/Cargo.toml -- 6E 2 output/review-run
cargo +stable run --locked --manifest-path tools/runner/Cargo.toml -- --audit-run output/review-run
```

The run freezes the selected runtime, receipt, source, wash guest, runner,
participant inputs and dependency tree before starting any models. The audit
checks those files, each model's actual tool exchanges and usage, the task's
Endpoint messages, and its recorded common-simulator outcome. A negative domain result
is preserved as a domain result. It is not changed into successful work by a
completed protocol. The same runner audits each migrated task's records.

EPF checks dynamic source execution, including operation order, route/roll
choices and affine delegation. Effect auditors do not reconstruct loan windows,
current holders or another workflow. They bind accepted requests to the actual
arguments, effects, replies and common-simulator scores. Final returns remain
completion evidence: acceptance of a finite source prefix alone is insufficient.

The kitchen `12E` finite domain crosses all six equipment orders with all eight
per-chef base/ingredient orders, plus three controlled-delay refusal/recovery cases
that finish a dish before unrelated bases and its own outgoing ingredient.
The common entry assigns each of these 51 conditions once and reconciles the saved case
reports. It rejects an external `CHOREOFIX_KITCHEN_CASE` selector; that selector
is available only on the direct boundary test for explicitly partial diagnosis.
Use `--qualify-wasi NEW_OUTPUT kitchen_boundary::` for the complete kitchen
family, and `--audit-kitchen EVENTS` to recheck an existing kitchen execution
without repeating its guest effects or model calls. The source-bound scope and
the qualified pilot are recorded in `benchmark/ports/12E.md`.
The original kitchen tools have zero processing delay. The three concurrency
fixtures explicitly set 150 ms base and 100 ms ingredient delays through the
test-only environment connection; raw records retain that profile. Live-model
trials retain the original timing, and these controlled delays are not performance data.

Model errors or timeouts, runtime errors, nonzero runtime exits and failed
backend jobs set the run's `error` and produce a nonzero CLI exit. A completed
model response cannot hide an error reported with that response.

When a guest exits or closes its output, the I/O owner explicitly fails both
forwarded and queued shell requests, including callers sharing one live reply
channel. It then reaps the child and drains already-admitted effects before
scoring. Ordinary CI explicitly runs the isolated process regression:

```sh
PYTHONHASHSEED=0 cargo +stable test --locked --manifest-path tools/runner/Cargo.toml process::tests::runtime_shutdown_completes_pending_and_queued_shared_channel_replies -- --exact --ignored --test-threads=1
```

This uses real child stdio and the shared simulator connection, without Wasm or
model calls. It checks failed verdict and EOF; cancellation of the entire owner
is a separate boundary.

For the feature-delivery family, `--qualify-wasi NEW_OUTPUT feature_delivery_boundary::`
assigns all four initial orders and both recovery conditions to six separate
test processes. It uses the same finite input constants as the boundary tests,
records each command and selector, and requires exactly one test per assignment.
This bounds cumulative fixture memory without changing the domain or skipping
cases. An external `CHOREOFIX_FEATURE_DELIVERY_CASE` is rejected by qualification;
it is only a direct-test diagnostic selector, whose saved scope is explicit.
This family filter does not claim all other families passed.
`--check-run-mutants RUN NEW_DIRECTORY` checks the saved model records against
deliberately altered records without making model calls.

The `3E` report path uses the same command and `REPORT` build variables.
Its current source has passed the 18 ordinary conditions, eight extended
rejection conditions, one invalid-input/post-completion condition and a
three-model pilot. Researchers can exit after their own positive section review;
their native localsides retain the remaining publication communication. No
extra shell read, host completion flag or automatic reply-draining path is used.
The exact sources, compiled image, original score, EPF and
effect audit are retained with each run; this does not qualify other source
revisions or establish comparative performance.
`--check-report-mutants EVENTS NEW_DIRECTORY` checks omissions, coherent
forgeries and duplicates of actual business values, wire headers and the actual
receive/effect/result boundaries. It operates on
the saved records without model calls or Endpoint execution. The production
auditor checks EPF source execution and binds its successful receive commits to
actual request values, effects and results. It does not reconstruct another
resource-holder or transfer-order model. Each original capture is checked once.
Value/header/effect counterexamples retain its record contents and positions;
frame omissions fail because a checked receive loses its matching frame.
Changes to producer records must be checked by EPF again. The boundary test
reuses the baseline audit saved by
this command rather than running EPF twice. No source copy, defect restoration
or rebuild is involved.
Current evidence is in
`output/experiments/research-report-direct-files-20261114/qualification-v30.json`.

The `4E` API-coordination path uses `COORDINATION` build variables and the same
normal run and `--audit-run` commands. Each of the three actual models owns one
Wasm shell. The global moves the actual repository and test-environment Endpoints;
the localsides expose their operations directly without task flags or an owner table.
The files own their projected Endpoints from the outset. Contract, agreement,
candidate and result reads use the SDK's actual message receive projection;
the host does not choose the shell's next operation. A refused unadmitted call
returns an error so the same guest can choose another operation or retry.

With common world v19, five actual-Wasm conditions pass both implementation
orders, one/three API amendments, invalid-input recovery and normal delay.
The first three records were reused after correcting occurrence matching in
the effect auditor; only the remaining two conditions needed new Wasm runs.
The earlier failed command remains recorded as failed. A three-person Luna
xhigh pilot using the same runtime binary also passes independent EPF/effect
and common-scorer checks: 39.862 seconds, 352,337 inclusive total tokens,
four actual effects and zero violations. These are finite, unscored,
source-bound results, not qualification of all families or scored comparisons.
See `output/experiments/coordination-direct-projection-20261113/qualification-summary.json`
and `output/experiments/coordination-free-shell-current-20261113-live/`.
`--audit-coordination EVENTS` checks saved evidence without model calls.

The v19 world executes actual backend/frontend code and binds completed
integration tests to the current repository contents. Its world tests cover
both implementation orders and all 24 serial tool orders. World behavior,
EPF conformance and the actual request/effect audit are separate checks.

`5H` uses `CRITICAL_CARE` build variables and seven independent Wasm guests.
`critical_care_boundary::` runs eleven direct-file conditions through common
scoring, complete EPF and the actual-effect auditor. Nine matrix cases cover CT
orders, partial panels, rejection recovery and actual concurrent CT/MRI effects;
two schedules demonstrate each patient's care finishing before the other's assessment.
Both real ICU resource requests and the anesthesia-to-surgery OR handoff remain
in the global. `--audit-critical-care EVENTS` independently checks the saved
record. The same-binary seven-model qualification completes in 68.85 seconds
with zero violations. Scope and evidence are in `benchmark/ports/5H.md`; this
is not a scored comparison.

Run all declared families' scripted WASI regressions from one source
snapshot with a new output directory:

```sh
cargo +stable build --locked --manifest-path tools/runner/Cargo.toml
tools/runner/target/debug/choreofix-experiments --qualify-wasi output/wasi-boundary
```

This builds the wash guest and task hosts, saves their source and receipts,
and runs each family's ignored boundary tests. Missing tests, a failed build or
a source change stop qualification. `CHOREOFIX_APP_TARGET_DIR` and
`CHOREOFIX_RUNNER_TARGET_DIR` can select separate reusable build directories.
Run local builds and qualification under the documented resource guard.

To prepare a current-source binary and receipt for a live trial without repeating
scripted execution, use `--build-wasi NEW_OUTPUT feature_delivery` (or a
comma-separated list of declared family names). This uses the same build,
source archive and receipt path as qualification. Its result is explicitly
`built_not_qualified`, with `qualification_tests_run: false`; building alone
does not certify a task or count a measured trial.

These scripted tests make no model calls. They check actual wash guests,
resource requests and returns, independent work, malformed and duplicate
requests, reconsideration where the task permits it, and terminal domain failures.

Protocol interpretation belongs to EPF. The kitchen and manufacturing effect
auditors use its checked receive commits, without replaying loan intervals,
return order, Ready retirement or Busy/retry sequencing. They still match actual
file arguments, resource requests, effects, replies and common-world outcomes.
Checking those values is separate from source conformance; a valid EPF prefix
alone does not certify an application result.

The code-review auditor checks the review-control Endpoint, repository Endpoint, actual WASI
arguments, simulator requests and results, approval before merging, and guest
exits. Each review uses distinct intent deliveries, a request, an argument-file
close and its projected result deliveries: five while both features await approval,
three after the other feature is approved. It matches the initial control Endpoint
issuance, its loan around every review, and its final transfer through both
developers to the repository. Each review consumes its own control loan and return.
Repeated arguments cannot reuse earlier
occurrences. A repeated review also requires a new completed implementation.
Every review-task frame is matched against `projection.json`, exported from the
selected executable before the run. Its hash and executable hash are recorded in
the case or model-run receipt. The runtime records the same compiled data from
the programs it actually attaches. Checks include full wire labels, lanes, peers,
delegated resource envelopes and the review messages' logical labels.
The auditor also calls EPF to construct and check a finite source execution from
the complete producer capture. Business-effect correspondence is checked separately.
Other task auditors
retain their documented checks. Keep the projection and receipt beside
`EVENTS.jsonl`.
The auditor reports overlapping domain operations and runs without models:

```sh
cargo +stable run --locked --manifest-path tools/runner/Cargo.toml -- --audit-review EVENTS.jsonl
cargo +stable run --locked --manifest-path tools/runner/Cargo.toml -- --check-review-mutants EVENTS.jsonl NEW_DIRECTORY
```

The review regression requires `CHOREOFIX_REVIEW_WASI_BINARY`,
`CHOREOFIX_REVIEW_BUILD_RECEIPT`, `CHOREOFIX_AUDITOR` and
`CHOREOFIX_EVIDENCE_ROOT`. `--qualify-wasi` supplies its frozen CLI as the
auditor. Each case runs the same audit in a fresh process, so large temporary
allocations end with that process rather than accumulating across cases.
The receipt binds
the binary to its unchanged build-source manifest. Each run saves that manifest,
the receipt, runtime and harness hashes, traces and mutation verdicts under the
evidence root. The workflow runs all 34 ignored review cases individually and
retains these files. A native test checks that the case indices cover each
declared input exactly once.
The saved receipt names the manifest relative to its evidence bundle, so moving
the bundle preserves the reference. The original input receipt is saved with its hash.
It covers failure conditions 0/1/2, both first
review targets and repeated or alternating reviews. Eight additional cases use
the original delays and require observed overlap, for 26 cases in total.
Eight separate mechanism cases inject third and fourth rejections through the
unchanged simulator's failure API, covering both first targets and both ways
of revisiting rejected work. They do not change the scored 0/1/2 matrix.
Every trace checks 28 record mutations, including missing control Endpoint loans,
returns and final closing transfer, plus three that remove a later
same-argument request, argument-file close or result when reviews repeat.
These are integration checks, separate from scored experiments.

```sh
cargo +stable build --locked --manifest-path tools/runner/Cargo.toml
export CHOREOFIX_AUDITOR="$PWD/tools/runner/target/debug/choreofix-experiments"
cargo +stable test --locked --manifest-path tools/runner/Cargo.toml \
  review_boundary:: -- --ignored --test-threads=1 --nocapture
```

Dining checks use the original 3/5/7-person environments through the Rust
connection and actual wash guests. They cover private work, early and duplicate
operations, malformed arguments, argument preservation, both fork Endpoints,
and observed overlap of independent meals. There are six conditions: normal
and adversarial input for each participant count. Twenty record mutations
must fail. EPF checks the complete source execution; the dining effect check
only binds its committed receive occurrences to admitted file bytes, actual
resource requests, physical effects/replies and the common simulator score.
It does not recreate protocol progress or a current resource-holder table.
EPF also owns communication order and delegation-envelope validation. The
effect check requires the actual requests before an effect and its result
before the reply; it does not recheck ordering between protocol operations.
Five mutations retain EPF-valid execution while changing effect arguments,
file data, result data, effect timing or effect multiplicity; the effect check
must reject these separately. This is recorded-data validation, not twenty
runtime attacks.
The selected runtime requires `CHOREOFIX_PHILOSOPHERS_WASI_BINARY` and
`CHOREOFIX_PHILOSOPHERS_BUILD_RECEIPT`; `CHOREOFIX_EVIDENCE_ROOT` selects a new
evidence directory. No model client is required for these scripted checks.

```sh
cargo +stable test --locked --manifest-path tools/runner/Cargo.toml \
  philosophers_boundary:: -- --ignored --test-threads=1 --nocapture
```

For the complete source-bound qualification use `--qualify-wasi NEW_DIRECTORY
philosophers_boundary::`. It runs each of the six conditions in its own process
and requires all six reports against one runtime binary. This releases each
condition's audit search workspace before starting the next condition.

The codebase and writing effect checks also require EPF source conformance and
bind requests to its committed receives. Their duplicate holder/transfer tables
and transport transition implementations have been removed; they share the
journal reader used by dining. Complete captures are required, with no old-audit
fallback. Earlier static-only records are historical evidence and do not qualify
these revised checks. The tasks still require their new ChoreoFS boundary ports
and actual Wasm/model qualification. An EPF-valid finite prefix is insufficient
when a request's committed receive is absent. Task completion also requires
observed final resource returns; these are counted from the checked journal,
without reconstructing holders or transfer states. Effects and delivered values
are read once at their EPF receive commits, not again at transport-send rows.

The CI/CD record auditor runs without starting a model or guest:

The current 16E files own their projected Endpoints. Closing one submits that
operation; the host does not preselect a command or send a request-ready grant.
Reads of the same file receive actual `{kind,result}` messages: operation
results, build-complete, tests-passed and both deployment-complete notifications.
Private work stays available independently. Unadmitted and malformed requests
return errors so that the same shell can continue.

The declared Wasm assignments are `adversarial_depth_0`, `ordinary_depth_1`,
`normal_delay_depth_2` and `real_wasi_cicd_early_exit_has_no_domain_effect`.
All four passed; the three completed records reject 16 record counterexamples
each. Early exit has no actual domain effect. The final common-world change
retains unvalidated-publication violations after repair; its 10 world tests
pass, while native Rust, SDK, runtime and runner bytes remain unchanged.
Their existing boundary evidence is retained, with one new ordinary Wasm run
and a fresh model pilot under the final scorer. The final Luna xhigh pilot
uses three actual models and passes independent EPF/effect/common-score checks:
32.079833 seconds, 386,756 inclusive total tokens, three effects, seven actual
incoming messages and zero violations. These are finite, unscored results.
See `output/experiments/cicd-direct-projection-20261113/qualification-summary.json`.

```sh
cargo +stable run --locked --manifest-path tools/runner/Cargo.toml -- --audit-cicd EVENTS.jsonl
cargo +stable run --locked --manifest-path tools/runner/Cargo.toml -- --check-cicd-mutants EVENTS.jsonl NEW_DIRECTORY
```

It checks EPF source conformance, actual requests, returned results,
notifications and the original simulator score. Mutation checks remove whole
frames or alter every observation of the same frame. Moving transport
observations alone is not a source-execution counterexample; early-effect
checks must move the effect before its required EPF receive commit.

The writing auditor checks both section Endpoints, actual requests and replies,
returns, original outcomes and disjoint write overlap for 3/5/7 participants:

```sh
cargo +stable run --locked --manifest-path tools/runner/Cargo.toml -- --audit-writing EVENTS.jsonl
cargo +stable run --locked --manifest-path tools/runner/Cargo.toml -- --check-writing-mutants EVENTS.jsonl NEW_DIRECTORY
```

The actual-WASI writing checks require `CHOREOFIX_WRITING_WASI_BINARY`,
`CHOREOFIX_WRITING_BUILD_RECEIPT` and `CHOREOFIX_EVIDENCE_ROOT`. The receipt
identifies the binary and unchanged source manifest used to build it.

```sh
cargo +stable test --locked --manifest-path tools/runner/Cargo.toml \
  writing_boundary:: -- --ignored --test-threads=1 --nocapture
```

## Model connection

Participants use the existing ChatGPT login through Codex App Server. They
request `gpt-6-luna` with `xhigh` and reject a substituted model or effort.
The runner requires ChatGPT authentication and removes API-key environment
variables from the child process. It does not call the paid OpenAI API.

Check the login before building:

```sh
codex login status
cargo +stable build --locked --manifest-path tools/runner/Cargo.toml
```

ChatGPT subscription limits still apply. Raw model events, tool calls and
inclusive token usage are recorded for each participant. Reasoning tokens are
part of output tokens and are not counted twice. Missing usage remains unknown.

ChoreoFix experiments use the available subscription connection and record its
model, settings and execution conditions. Comparison connection development is
deferred. An unavailable comparator is recorded as unexecuted and does not block
ChoreoFix runs. Results with different conditions are not reported as matched
comparisons. Keep the original comparison implementations unchanged.

Run builds and verification under Hibana’s resource guard with one Cargo job.
PyO3 loads the canonical common simulator from `benchmark/`; no Python adapter
or evaluated snippet is used. `common-environment-v21` binds actual material
and completed validation to publication. In 3E, drafts and references name the
research they used; each section's review names that section's current material.
Independent author, write/reference and review orders remain free. This common
world qualification does not certify the unfinished 3E Endpoint/file port. Every
condition uses the same environment. Each fixture records its contract and
manifest hash; earlier results retain their recorded simulator version.

See [Codex authentication](https://developers.openai.com/codex/auth) and
[App Server](https://developers.openai.com/codex/app-server) for the login and
participant connection.

`--qualify-wasi NEW_OUTPUT` builds and checks every declared WASI family.

The feature-delivery reconsideration test normally executes both declared
conditions. To execute only its remaining input during diagnosis, use
`CHOREOFIX_FEATURE_DELIVERY_CASE=1` with the explicit diagnostic filter
`feature_delivery_boundary::actual_wasi_review_rejection_and_ci_repair`.
The evidence records the selected indices and `full_matrix: false`; this is
not a complete-matrix qualification. An out-of-domain index is an error. The
unfiltered all-family command and every CI qualification reject this diagnostic selector.
For local diagnosis, the same entry accepts a Rust test filter, for example
`--qualify-wasi NEW_OUTPUT review_boundary::repeated_01`. The artifact records
that filter and always sets `all_families_passed` to false for a filtered run.
Comma-separated filters select complementary families under one frozen source
and one build, for example `discovery_boundary::,refactor_boundary::`.
A repeated family or unknown/empty selector fails before starting the build.
An empty or unmatched test result fails. Normal CI uses the unfiltered command.
