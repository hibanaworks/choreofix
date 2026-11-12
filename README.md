# ChoreoFix

Development checkpoint `progress-20261112` publishes the current implementation,
including the 29 source-bound accepted ports and unfinished 3M work. It is not
final all-family certification or a scored comparison. For the canonical sibling
path dependencies, select the same checkpoint tag of Hibana, hibana-epf and
hibana-wasip1-runtime. The runtime keeps its recorded `09a803a` Git pin.
Ordinary CI runs on branch pushes and pull requests, not progress tags. A
checkpoint tag is a source backup, never a successful or skipped certification.

ChoreoFix evaluates multi-agent coordination with
[Hibana](https://github.com/hibanaworks/hibana). A global protocol defines the
communication and transfer of resource endpoints. Participants choose their
shell commands and how to perform their work; projected endpoints check shared
operations before those operations reach the task simulator.

Each model has its own [hibana-wash](https://github.com/hibanaworks/hibana-wash)
Wasm instance. WASI requests pass through Hibana endpoints to ChoreoFS. Each
instance has a separate, caller-supplied 2 MiB memory region. Participants run
concurrently on [hibana-runtime](https://github.com/hibanaworks/hibana-runtime).

## Experiment

The experiment uses all 48 TraceFix tasks: 16 scenarios at three difficulty
levels, with three injected failure conditions and three repetitions.

| Condition | Runs |
| --- | ---: |
| Paper release: topology monitoring, mediator enforcement, prompt-only and chat-only | 1,728 |
| ChoreoFix | 432 |
| Latest TraceFix monitoring runtime | 432 |
| Total | 2,592 |

The [updated research plan](plan.md) also covers ZipperGen, Concordat, handoff,
causal comparisons and two real-artifact cases: 1,368 additional planned runs
and a reserve of at most 600. The overall budget is 4,560 runs. These additions
still require task, connection and statistical-design qualification; the current
executable matrix covers the 2,592 runs above. The first comparison remains
latest TraceFix against ChoreoFix, with 432 runs each.

The paper comparison is pinned to
`87b01a2c0ae53c5045c9ce3d43cd0235b1831275`; the latest comparison and benchmark
inputs are pinned to `36b4bd108fcd74f7efb17613aebbe3fdf55499a9`.
Comparison implementations remain unchanged. The shared simulator lives in
`benchmark/` and is fixed by `simulator-manifest.json`. Its versioned
contracts model 4M's repository contents, publication
and completed exact-content validation before publication. It checks necessary
causality while allowing independent legal operation orders. The same
environment and outcome criteria apply to every condition; earlier results
retain their recorded simulator version.
The `common-environment-v12` added actual shared build inputs, object files,
completed test/validation receipts and captured link inputs for 10E/10M/10H.
It preserves both first-builder orders in 10E, including B's rebuild after A
changes the provided type definitions, and actual two-slot capacity in 10H.
The same world shares document/figure data and material-bound cascading laboratory QC.
Its 136 direct tests include 7E's 48 valid orders
among 720 preparation/review permutations and all 720 orders of 7M's independent
initial writes and reference updates. Later evidence cannot authorize an earlier
publication. These data tests do not qualify the three application ports.
7M's completed-content reads pass direct tests, but its application port and
7H's minor-edit distinction remain open.
The version-7 rules also bind 6E/6M/6H source candidates,
reviews, merges, CI and staging to completed, content-matching data. Unsafe
publication/deployment remains an observable effect; later evidence cannot
authorize it retroactively. These development tools are abstract world models,
not a compiler. Its repair records distinguish replacement of a failed
publication from an unrelated next merge. Draft fixes and reviews may run
while CI is pending; publication waits for the completed failure evidence.
The recorded v7 qualification passed 87 tests, including 720 reordered 6M test/review/merge
schedules with exactly 80 valid outcomes. Earlier qualified ports keep their
original world versions. The 7E port now passes eleven actual-Wasm conditions
and a three-model pilot, as recorded below; other document ports remain unfinished.
The 11E environment maps each job to its actual workstation and returns completed
job data in reports. It accepts all 24 job orders, concurrent independent machine
use and replacement of an early report; it does not impose a workflow. These
simulator corrections alone do not qualify a ChoreoFix port. The 11E port now
passes the actual-Wasm and repaired-source model qualification described below.
See the [source-bound scope](benchmark/ports/11E.md).
The current `common-environment-v13` gives 11M actual part identities,
current completed quality receipts, part-bound rework orders and the original
two-tool/single-inspector capacities. Independent job and inspection orders stay
free. All 147 common-world tests and 17 Rust input-tool tests pass, including
720 job orders and 720 inspection orders in separate finite groups. The 11M
application port remains unfinished; these world tests do not qualify it.
See the [world preparation](benchmark/ports/11M.md). Existing qualifications keep
their actual earlier world versions.
The 14E environment checks prepared-cell and compound inputs, binds an actual
lead decision to completed analyses, and requires that decision before final
documentation. Its synthetic latency model is common to every method and makes
exclusive-instrument conflicts and independent-instrument overlap observable.
The 13E environment binds prepared cells, purified compound, assay and quality
results to completed input occurrences. Final validation needs both quality
results for the same assay and compound; independent legal orders stay valid.
Its three actual-Wasm conditions and one three-model pilot pass common scoring,
EPF and effect audits on their recorded source.
The 5E/5M/5H medical world uses actual completed patient results and repairs the
public `test_type` input. Independent test panels, imaging and medication review
remain independent; diagnosis needs its own current results. Seventy common-world
tests pass, including all 72 data-valid 5E orders. These tests qualify the world;
the separate 5E participant qualification is described below. They do not
observe absent prescription/administration effects or relabel earlier pilots
as contract version 3.
ChoreoFix may improve the coordination procedure while preserving
participants, resource safety and necessary data dependencies. Those changes
are recorded in the [experiment contract](benchmark/experiment-contract.json)
before scoring. The [matrix](experiment.json) fixes the model, repetitions,
timeouts and tool-call limits.

Participants use `gpt-6-luna` with `xhigh` through the existing ChatGPT login
and Codex App Server. The runner checks the returned model, effort and login
type. Development of the comparison clients' subscription connection is on hold;
it does not gate ChoreoFix's own experiments. A future same-condition comparison
will require qualification preserving the clients' original prompts, history,
tool schemas and execution order.

## Status

**Scored runs: 0 / 2,592.** The full task port and controlled comparison are
unfinished. Existing results are integration qualifications, not benchmark
success rates. The additional planned comparisons also have zero scored runs.
Development uses the current canonical core, EPF and WASI worktrees, including
the core source undergoing validation. Each qualification freezes its actual
sources and checks the concrete receiver/effect boundaries on that combination.
Using the current source does not establish that its qualification is complete.
Older pilots below retain their recorded version-1 environments. Separate
version-2 qualification has passed for 4M and 14E: eight real-Wasm conditions
and one five-model/four-model pilot respectively, with common scoring and
EPF/effect audits. The subsequent 13E qualification adds three Wasm conditions
and one three-model pilot. These source-bound results do not certify the other
tasks' common causal scoring.
The minimal receiver-contract composition is
implemented; applying it to every effect entry and completing the remaining
task ports are still required. These obligations do not depend on developing a
comparison client.

The twenty-eight migrated tasks have actual Wasm qualifications for
three/five/seven-person Dining, seven-person collaborative
coding with reconsideration, collaborative writing at three/five/seven participants,
the three-person API, API-coordination, CI/CD, code-review and research-report tasks,
five-person shared-library refactoring, four-person drug discovery and
three-person pharmaceutical synthesis, four-person synthesis/QC reconsideration,
three-person medical consultation, five-person multi-specialty care and
seven-person two-patient emergency care, the three-person shared document and
the two-builder and three-builder shared-library builds with their integrator,
the two-technician manufacturing task with its supervisor, five-person feature
delivery, the three-chef kitchen with delegated equipment and actual ingredients,
and five-person regulatory discovery with actual scientific results and reconsideration.
Code review includes
revisions after rejected reviews. The API checks
preserve both the original successful outcome and terminal test failure.
Audits check resource transfers, requests, returned results, shared effects,
file reads and guest exits. Record mutations check that the audits reject
missing or altered operations.

Shared refactoring (`4M`) has 64 native/EPF conditions, two receiver-negative
probes, five actual-Wasm conditions and one five-model pilot. The pilot passed
the common published-content outcome and EPF/effect audits in 61.07 seconds,
using 660,689 inclusive tokens. Its 14 actual effects and 7,191 captured records
are bound to the saved binary and source. This revised common task is identified
separately from the upstream task; these are unscored qualifications. Twenty
task ports and final-source certification remain unfinished.

Regulatory discovery (`14M`) passes twelve actual-Wasm conditions with 231
effects, dynamic EPF/effect audits and 1,890 rejected record mutations. One
five-model depth-two pilot completes in 288.26 seconds with 2,005,631 inclusive
tokens, 38 effects, two selected reanalysis plans and zero violations. Three
independent effects overlap. The saved pilot passes the corrected auditor;
its original harness failure from omitted report metadata remains recorded.
These are source-bound qualifications, not scored comparisons. See the
[scope and evidence](benchmark/ports/14M.md).

Shared document (`7E`) passes eleven actual-Wasm conditions and a three-model
depth-two pilot in 189.48 seconds with 919,803 inclusive tokens. Both actual
review failures lead to reconsideration before publication; all guests complete,
with ten effects, zero violations and passing dynamic EPF/effect audits. Eleven
altered run records are rejected. Earlier failed pilots remain preserved and
are not converted into successful measurements. See the [source-bound scope](benchmark/ports/7E.md).

Shared build (`10E`) passes eight actual-Wasm conditions and one three-model
pilot in 48.05 seconds with 261,400 inclusive tokens. Both initial builder
orders are tested, including B's rebuild after A changes the shared types.
Complete EPF capture and separate effect audits pass, with zero violations.
The eight mechanism records reject 420 frame mutations; eleven altered model-run
records are also rejected. These are source-bound qualifications, not scored
comparisons or execution of a real compiler. See the [scope and evidence](benchmark/ports/10E.md).

Three-builder build (`10M`) passes all 26 actual-Wasm conditions and one
five-model depth-two pilot in 115.34 seconds with 769,827 inclusive tokens.
The 478-event global projects 15 roles; its actual objects, tests, repairs and
link inputs pass separate EPF and effect audits, with zero violations.
All five model turns overlap, and one actual build/test pair overlaps.
The mechanism records reject 7,102 frame mutations; eleven altered model-run
records are also rejected. A production receiver rejects changed resource
arguments before any effect in native and strict-provenance Miri tests.
These are source-bound qualifications using a synthetic compiler data model.
See the [scope and evidence](benchmark/ports/10M.md).

Manufacturing (`11E`) passes 36 actual-Wasm conditions, including all 24 job
orders and eight contended-machine/retry schedules. The records pass dynamic EPF
and independent effect audits, rejecting 2,040 frame mutations. On the repaired
source, three Luna xhigh participants complete the task in 38.28 seconds with
280,303 inclusive tokens, five effects, zero violations and one actual pair of
overlapping independent jobs. Eleven altered model-run records are rejected.
The first pilot failed because its localside skipped a required retry; that
failure remains preserved. This is an unscored qualification, not a success-rate
estimate. See the [scope and evidence](benchmark/ports/11E.md).

Pharmaceutical synthesis (`13E`) passes three scripted Wasm conditions and one
three-model pilot. The pilot completed in 42.35 seconds with 356,591 inclusive
tokens, eight actual effects, no violations and passing EPF/effect audits.
The common world tests input causality without reading the task choreography.
See the [source-bound scope](benchmark/ports/13E.md).

Medical consultation (`5E`) passes nine scripted real-Wasm conditions, three
strict-provenance Miri functions and one three-model pilot. The pilot takes
31.45 seconds and 257,121 inclusive tokens, with six actual effects, zero
violations and passing dynamic EPF/effect audits. Scripted conditions preserve
partial-panel reconsideration and CT before, between or after panel requests.
The common world checks actual patient inputs without prescribing a workflow.
Its absent prescribing/administration effects are not certified. See the
[source-bound scope and limits](benchmark/ports/5E.md).

The subsequent common-world version 4 corrects 5H's ICU bed capacity from the
default one to the specified two. All 73 world tests pass, including 90 legal
5H image orders and actual overlapping ICU calls. Effect/scorer code is unchanged;
earlier pilot records keep their original contracts and hashes.
The 5H port passes eight real-Wasm conditions and a seven-model pilot through
common scoring, dynamic EPF and actual-effect audits. The pilot takes 48.16
seconds, 679,177 inclusive tokens and completes 16 effects with zero violations.
Its actual effects have seven overlapping pairs. Source-bound scope and limits
are recorded in [the 5H port](benchmark/ports/5H.md).

Multi-specialty care (`5M`) passes eight scripted real-Wasm conditions and one
five-model pilot. The pilot completes in 48.37 seconds with 571,513 inclusive
tokens, 12 actual effects, zero violations and passing dynamic EPF/effect audits.
Its original test stopped at an auditor metadata bug after execution succeeded;
the corrected reader passed an offline audit of the unchanged saved run.
The original failed test remains recorded. See [the exact scope](benchmark/ports/5M.md).

Drug discovery (`14E`) has two production-receiver negative probes, three
actual-Wasm conditions covering both decisions, and one four-model pilot. The
pilot passed the common outcome and EPF/effect audits in 57.86 seconds, with
12 actual analyses, no resource conflicts and 492,253 inclusive tokens. Its
5 overlaps between independent instrument effects and 4,744 captured records
are retained with the exact source and binary. These are unscored results;
the shared simulator does not enforce this port's choreography. The
[port contract](benchmark/ports/14E.md) separates the two.

The report port has 27 scripted Wasm conditions and one three-model pilot,
with original scoring, EPF execution checks and actual-effect audits.
Its current evidence rejects 6,797 record mutations across those 28 traces.
These are mechanism qualifications on their recorded sources, not scored
comparisons or a completed qualification of every port on the final core.

The API-coordination port (`4E`) has five actual-Wasm conditions, including both
implementation orders and repeated API changes, plus one three-model pilot.
All pass original scoring, EPF source-execution interpretation and the separate
actual-effect audit. The pilot took 38.38 seconds and 287,517 inclusive tokens.
These results are tied to their recorded sources and are not scored comparisons.

The plan records unscored subscription pilots with their exact sources and
binaries. They do not qualify later revisions or establish benchmark success rates.

The [work plan](plan.md) distinguishes implemented ports, verified paths,
remaining tasks and scored results. [The paper draft](paper/choreofix.tex)
remains a draft until the experiments and analysis are complete.

## Protocols and participant code

Resource endpoints are moved with ordinary `send` and `recv`. The holder sends
the resource request, receives its result and returns the endpoint. Resolvers
choose a route from an actual result or local decision. `roll` expresses
reconsideration; `par` permits independent work to run concurrently.

| Task | Global | Localsides |
| --- | --- | --- |
| Collaborative coding | [protocol](src/tasks/codebase/choreofs/global.rs) | [participants](src/tasks/codebase/choreofs/localside) |
| API integration | [protocol](src/tasks/api/choreofs/global.rs) | [participants](src/tasks/api/choreofs/localside) |
| Dining philosophers | [protocol](src/tasks/philosophers/choreofs/global.rs) | [participants](src/tasks/philosophers/choreofs/localside) |
| Collaborative writing | [protocol](src/tasks/writing/choreofs/global.rs) | [participants](src/tasks/writing/choreofs/localside) |
| Code review | [protocol](src/tasks/review/choreofs/global.rs) | [participants](src/tasks/review/choreofs/localside) |
| CI/CD | [protocol](src/tasks/cicd/choreofs/global.rs) | [participants](src/tasks/cicd/choreofs/localside) |

Endpoint operations remain visible in the localsides. Filesystem data and
external RPC bookkeeping belong to their I/O owners. Task permissions and
progress belong to the projected protocol. The external simulator evaluates
domain effects and outcomes.

## Build and validation

Use the current stable Rust toolchain, the `wasm32-wasip1` target and the pinned
hibana-wash submodule. The host runner currently uses macOS's event queue.
Keep this checkout beside `hibana`, `hibana-epf` and
`hibana-wasip1-runtime`. Their path dependencies select the current working
sources, with one core shared by the application, runner and effect-boundary
checks. Do not replace them with an older Git pin while development continues.
The Rust [input tool](tools/inputs) restores and verifies
the two unchanged upstream source archives. Python dependencies in
`requirements.lock` support those upstream simulators and comparison clients.

Run one local build or verification workload at a time through Hibana's memory
guard. `HIBANA_CHECKOUT` is the same sibling core selected by Cargo:

```sh
export CARGO_BUILD_JOBS=1
export HIBANA_CHECKOUT="$(cd ../hibana && pwd -P)"
TOOLCHAIN=stable bash "$HIBANA_CHECKOUT/.github/scripts/lib/resource_guard.sh" \
  cargo +stable run --locked --manifest-path tools/inputs/Cargo.toml -- restore
TOOLCHAIN=stable bash "$HIBANA_CHECKOUT/.github/scripts/lib/resource_guard.sh" \
  cargo +stable run --locked --manifest-path tools/inputs/Cargo.toml -- verify
TOOLCHAIN=stable bash "$HIBANA_CHECKOUT/.github/scripts/lib/resource_guard.sh" \
  bash tools/build_shell.sh
TOOLCHAIN=stable bash "$HIBANA_CHECKOUT/.github/scripts/lib/resource_guard.sh" \
  cargo +stable test --locked
TOOLCHAIN=stable bash "$HIBANA_CHECKOUT/.github/scripts/lib/resource_guard.sh" \
  cargo +stable test --locked --manifest-path proofs/endpoint-handoff/Cargo.toml --lib handoff::
```

Each `*-wasi` binary compiles its own canonical task global and localsides.
The library holds shared host I/O and codecs. `cargo test --locked` includes
the task-native tests in every binary; the separate handoff proof crate retains
its receiver and ownership checks.

The [runner](tools/runner) records raw model events, tool calls, source and
binary digests, original simulator results and inclusive token usage. Failed
qualifications and negative results remain evidence. Scoring requires frozen
inputs and qualified implementations for every condition.
Saved source manifests include working-tree changes in the application and
local dependencies. Bundles retain their sibling layout for relative Cargo
paths. Git bases and dirty patches identify the worktrees; hashes identify the
actual files. `checkout_head` alone does not identify a build.
