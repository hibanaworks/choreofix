# Experiment runner

The runner records model requests, tool events, versions and usage. Hibana task
permissions belong to the projected Endpoints, not this process interface.
The direct-WASI command currently produces unscored qualification records.
Use the experiment matrix and qualified task artifacts for scored comparisons.

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
It supports `1E`, `1M`, `1H`, `2E`, `2M`, `2H`, `3E`, `4E`, `4M`, `5E`, `5M`, `5H`,
`6E`, `6M`, `7E`, `8E`, `9E`, `9M`, `9H`, `10E`, `10M`, `11E`, `12E`, `13E`, `13M`, `14E`,
`14M`, `14H` and `16E`, with the original failure conditions `0`, `1` and `2`. Each qualification
retains its exact source and simulator version; these are 29 accepted task ports,
not final-source certification of every family. The
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

The kitchen `12E` qualification uses all six equipment orders and both ordinary
request orders, plus one invalid-input/pipeline condition. The common entry
assigns each of these thirteen conditions once and reconciles the saved case
reports. It rejects an external `CHOREOFIX_KITCHEN_CASE` selector; that selector
is available only on the direct boundary test for explicitly partial diagnosis.
Use `--qualify-wasi NEW_OUTPUT kitchen_boundary::` for the complete kitchen
family, and `--audit-kitchen EVENTS` to recheck an existing kitchen execution
without repeating its guest effects or model calls. The source-bound scope and
the qualified pilot are recorded in `benchmark/ports/12E.md`.

Model errors or timeouts, runtime errors, nonzero runtime exits and failed
backend jobs set the run's `error` and produce a nonzero CLI exit. A completed
model response cannot hide an error reported with that response.

The multi-round discovery `14H` family assigns all twelve scientific/retry
conditions to separate test processes with
`--qualify-wasi NEW_OUTPUT multi_round_discovery_boundary::`. Missing, duplicate,
failed or mixed-artifact reports fail aggregation. The complete family rejects
an externally preset `CHOREOFIX_MULTI_ROUND_DISCOVERY_CASE`; a direct diagnostic
test may select one case and reports that partial scope. Every completed case
runs the same EPF and actual-effect auditor as the model-run audit.
`--audit-multi-round-discovery EVENTS` audits a saved record, and
`--check-multi-round-discovery-mutants EVENTS NEW_OUTPUT` tests its auditor
without executing models or effects again. The live input records original
allowed tool schemas and assigned resources, not a fixed ordered command list.
The final auditor checks those schemas against the run's frozen originals.

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
three-model pilot. The exact sources, compiled image, original score, EPF and
effect audit are retained with each run; this does not qualify other source
revisions or establish comparative performance.
`--check-report-mutants EVENTS NEW_DIRECTORY` checks omissions, coherent
forgeries and duplicates of actual business frames, owned Endpoint envelopes,
WASI closes, effect ordering, original results and completion. It operates on
the saved records without model calls or Endpoint execution. Its static
business/wire scope is separate from `--audit-report`'s EPF execution check.
Source-bound dynamic evidence on identical records is retained when adding
these static checks; the original guests do not need to run again.

The `4E` API-coordination path uses `COORDINATION` build variables and the same
normal run and `--audit-run` commands. Each of the three actual models owns one
Wasm shell. The global moves the actual repository and test-environment Endpoints;
the localsides expose their operations directly without task flags or an owner table.
Five scripted conditions cover both implementation orders, one/three API amendments
and normal delay. One three-model pilot also passes original scoring and EPF/effect
checks. These are unscored, source-bound results. `--audit-coordination EVENTS`
checks the saved protocol/effect evidence without model calls.

The `4M` shared-refactoring path uses `REFACTOR` build variables. Five actual
Wasm shells drive the architect, both developers, reviewer and tester. The
common simulator's published-content outcome is distinct from the ChoreoFix
protocol, which lends the repository and test Endpoints and connects actual
review/test results to merge requests. `--audit-refactor EVENTS` checks these
values and calls EPF for dynamic protocol conformity. `refactor_boundary::`
declares four short-delay branch combinations and one normal-delay condition;
its execution and qualification status must be read from saved receipts.

The `14E` instrument path uses `DISCOVERY` build variables. Four Wasm shells
use the three actual instrument Endpoints; `--audit-discovery EVENTS` matches
accepted inputs, resource requests, actual analyses, reports and the go/no-go
decision and calls EPF for dynamic conformity. `discovery_boundary::` declares
both decisions at short latency and one normal-latency condition. Its port and
qualification scope is recorded in `benchmark/ports/14E.md`.

The `13E` synthesis path uses `LAB` build variables. Three Wasm shells use the
spectrometer and centrifuge Endpoints. `--audit-lab EVENTS` matches actual
admissions, instrument requests, completed specimen/assay/quality results and
calls EPF for dynamic conformity. `lab_boundary::` covers invalid early and
duplicate requests, optional culture data, chunked input and normal-delay
instrument overlap. See `benchmark/ports/13E.md` for its qualification scope.

The `13M` path uses `LAB_QUALITY` build variables and four independent
Wasm scientists. `--audit-lab-quality EVENTS` checks one-use actual admissions,
both QC resource requests, scientific receipts and real results, then invokes
EPF for dynamic conformity. `lab_quality_boundary::` declares ten real-Wasm
conditions: both first subjects, depths 0/1/2, repeated last-subject rejection
and four rejections while switching subjects. The source-bound migration,
successful live pilot and remaining limits are in `benchmark/ports/13M.md`.
The test saves all runtime cases before auditing their records. Progress reports
runtime completion separately from audited completion; success still requires
every dynamic/effect audit and the record mutations. Interrupted audit work
does not erase completed guest evidence or turn it into a qualified case.

`5H` uses `CRITICAL_CARE` build variables and seven independent Wasm guests.
`critical_care_boundary::` runs eight conditions through common scoring, EPF and
the actual-effect auditor, including both real ICU resource requests and the
anesthesia-to-surgery OR handoff. `--audit-critical-care EVENTS` independently
checks the saved record. Its actual seven-model qualification is documented in
`benchmark/ports/5H.md`; it is not a scored comparison.

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
and adversarial input for each participant count. Nineteen record mutations
must fail. EPF checks the complete source execution; the dining effect check
only binds its committed receive occurrences to admitted file bytes, actual
resource requests, physical effects/replies and the common simulator score.
It does not recreate protocol progress or a current resource-holder table.
Five mutations retain EPF-valid execution while changing effect arguments,
file data, result data, effect timing or effect multiplicity; the effect check
must reject these separately. This is recorded-data validation, not nineteen
runtime attacks.
The selected runtime requires `CHOREOFIX_PHILOSOPHERS_WASI_BINARY` and
`CHOREOFIX_PHILOSOPHERS_BUILD_RECEIPT`; `CHOREOFIX_EVIDENCE_ROOT` selects a new
evidence directory. No model client is required for these scripted checks.

```sh
cargo +stable test --locked --manifest-path tools/runner/Cargo.toml \
  philosophers_boundary:: -- --ignored --test-threads=1 --nocapture
```

The CI/CD record auditor runs without starting a model or guest:

```sh
cargo +stable run --locked --manifest-path tools/runner/Cargo.toml -- --audit-cicd EVENTS.jsonl
cargo +stable run --locked --manifest-path tools/runner/Cargo.toml -- --check-cicd-mutants EVENTS.jsonl NEW_DIRECTORY
```

It checks the actual resource loans, requests, returned results, notifications
and original simulator score. Mutation checks remove whole frames or alter
every observation of the same frame, and include overlapping build-server loans.

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
or evaluated snippet is used. `common-environment-v2` models actual 4M repository
contents and completed exact-content review/test receipts before publication.
14E checks material inputs, completed analyses and the actual submitted decision
before documentation. Independent legal orders remain free. Every
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
