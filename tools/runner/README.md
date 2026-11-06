# Experiment runner

The runner records model requests, tool events, versions and usage. Hibana task
permissions belong to the projected Endpoints, not this process interface.
The direct-WASI command currently produces unscored qualification records.
Use the experiment matrix and qualified task artifacts for scored comparisons.

The trace file and original simulator each have one I/O owner. Callers send
records or simulator requests through channels. A record returns after its
write and flush complete, including any I/O error. The simulator starts the
original asynchronous tools concurrently and returns each actual result on its
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
`CICD`, `REVIEW` or `PHILOSOPHERS`, as applicable to the task. A receipt contains
`binary_sha256`, `source_manifest`, `source_manifest_sha256` and
`source_unchanged: true`. Resolve the input manifest relative to the receipt.
The build must compare that source manifest before and after compilation.
It must include every file under `src`, `Cargo.toml`, `Cargo.lock`, and any
local Cargo configuration or `build.rs` used by the runtime.

Each evidence bundle saves the checked manifest, original receipt and a receipt
whose manifest path is relative to the bundle. Missing or mismatched files stop
the qualification before starting a guest. Existing build archives cannot be
overwritten. These records identify the measured build; they do not establish
that its implementation or task outcome is correct.

The normal command and live subscription tests use the same execution path.
It supports `1E`, `1M`, `1H`, `2E`, `2M`, `2H`, `6E`, `8E`, `9E`, `9M`, `9H`
and `16E`, with the original failure conditions `0`, `1` and `2`. Run from the
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
Endpoint messages, and its original simulator outcome. A negative domain result
is preserved as a domain result. It is not changed into successful work by a
completed protocol. The same runner audits each migrated task's records.
Model errors or timeouts, runtime errors, nonzero runtime exits and failed
backend jobs set the run's `error` and produce a nonzero CLI exit. A completed
model response cannot hide an error reported with that response.
`--check-run-mutants RUN NEW_DIRECTORY` checks the saved model records against
deliberately altered records without making model calls.

Run all six migrated families' scripted WASI regressions from one source
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

These scripted tests make no model calls. They check actual wash guests,
resource requests and returns, independent work, malformed and duplicate
requests, reconsideration where the task permits it, and terminal domain failures.

The completed code-review auditor checks the repository Endpoint, actual WASI
arguments, simulator requests and results, approval before merging, and guest
exits. Each review uses distinct intent deliveries, a request, an argument-file
close and five result deliveries. Repeated arguments cannot reuse earlier
occurrences. A repeated review also requires a new completed implementation.
Every review-task frame is matched against `projection.json`, exported from the
selected executable before the run. Its hash and executable hash are recorded in
the case or model-run receipt. The runtime records the same compiled data from
the programs it actually attaches. Checks include full wire labels, lanes, peers,
delegated resource envelopes and the review messages' logical labels.
This checks static wire contracts and business-event correspondence. Dynamic
branch and iteration eligibility need separate checks. Other task auditors
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
Every trace checks 25 record mutations, plus three that remove a later
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
and observed overlap of independent meals. Nine record mutations must fail.
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
PyO3 loads the unchanged upstream simulator; no Python adapter or evaluated
snippet is used.

See [Codex authentication](https://developers.openai.com/codex/auth) and
[App Server](https://developers.openai.com/codex/app-server) for the login and
participant connection.
