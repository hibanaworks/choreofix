# ChoreoFix

ChoreoFix tests whether a global choreography can be the sole source of authority
for agents that share resources. Each agent chooses commands in its own
[hibana-wash](https://github.com/hibanaworks/hibana-wash) shell. WASI file
operations on ChoreoFS drive the projected [Hibana](https://github.com/hibanaworks/hibana)
endpoints. In review (6E), API development (8E) and CI/CD (16E), each
simulator effect consumes the typed request received from an Endpoint. The
[6E](analysis/review-receipts.md), [8E](analysis/8e-received-effects.json)
and [16E](analysis/cicd-receipts.md) trials exercised this path with wash and
live agents. In [13E](analysis/13e-received-effects.json), three live agents
completed the task after all eight effects moved to typed Endpoint receives.
Five qualified tasks (13M, 13H, 14E, 14M and 14H) still have receivers
that can select a simulator tool and supply its arguments directly. We check the
protocol execution with EPF, match admitted requests to actual simulator effects,
and score the business outcome. Whether this design preserves the original
choices or makes a workflow cheaper to change is an empirical question.

## Status

| Question | Evidence through 22 November 2026 |
| --- | --- |
| How many tasks have run through ChoreoFS with live agents? | 38 tasks have their recorded qualifications. In 11M, five live agents completed all six jobs with one rework order. The auditor matched all 15 simulator calls on that run. A separate depth-two run on the same binary stopped with a shell read outstanding. The qualification count excludes 11M. [11M trial evidence](analysis/11m-source-20261122.json) In the original 43 selected task/depth conditions, 39 completed and four had a terminal negative outcome. The additional [7H trial](analysis/7h-review-inbox.json) completed with seven live agents and 33 audited effects. In [15E](analysis/15e-wafer-live.json), six agents completed wafer fabrication with 18 audited effects. [Pilot inventory](analysis/README.md) |
| What remains open? | 10 task qualifications and a common final revision. In 4H, seven scripted Wasm participants exercised 36 schema and suite orders and six failed-suite repairs. An early request for A returned the missing prerequisite C in all 42 runs. DevOps deployed B first in 12 runs. The runs deployed all three tested services without overlapping operations on staging. Rollback, registration and live-agent qualification remain open. [4H runs and receipts](analysis/4h-tested-deployments.json) |
| What has been compared? | 0 of 2,592 original and supplemental episodes scored, and 0 of 1,368 additional planned comparisons. The pilots measure ChoreoFix on their own recorded conditions. |
| Which choices have been checked? | Wasm guests completed all 90 upstream 9E sequences of thoughts and meals, plus one in which B and C ate between A's three thoughts. In 9M and 9H, each seat ate first while the others stayed idle. For every first seat, the remaining diners completed in both directions around the table (24 Wasm trials). Five live diners started in B, A, C, E, D order and seven in E, D, B, G, C, A, F order. Peak overlap was two and three meals, respectively. In 1E all six commit orders ran through wash. [Reproduction and limits](analysis/arbitration.md) |
| Which participant orders changed? | On the revised globals, real Wasm guests demonstrated that any of seven developers can commit first in 1H, either author can be reviewed before the other starts research in 3E, and either approved feature can finish CI before the other starts work in 6M. In 6H all six merge orders passed with deployment in A, B, C order. Three live agents completed revised 3E. A live 6H trial merged C first, then timed out while CI waited for A. [Runs and audit evidence](analysis/arbitration.md#rechecking-the-four-participant-order-constraints) |

[Paper draft](paper/choreofix.tex) · [Pilot inventory](analysis/README.md) · [Task-specific qualification notes](benchmark/ports) ·
[Work plan and run history](plan.md) · [CI workflow](.github/workflows/quality.yml)

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

The [matrix](experiment.json) fixes failure depths, the 1,800-second episode
budget, the 200-call participant limit and the model. The four paper conditions
use upstream revision `87b01a2`, and the additional TraceFix condition uses
`36b4bd1`. Participants in the ChoreoFix qualifications use
`gpt-6-luna` with `xhigh` through Codex App Server. The common simulator and
its source manifest are in [benchmark](benchmark/). Each qualification retains
its own source, binary, simulator version, successes and failed attempts.

For a comparison intended to isolate enforcement, both arms must allow the
same schedules, expose the same commands and replies, and let an agent recover
from the same rejected calls. A native procedure that fixes resource priority
or changes acquisition order is reported separately. The [experiment
contract](benchmark/experiment-contract.json) records these distinctions.
The [maintenance study](analysis/arbitration.md#maintenance-study) will count
changes to authorization decisions. For its first ticket we checked the
[one-reviewer starting procedure](analysis/review-receipts.md#starting-procedure-for-the-maintenance-ticket)
in the upstream 6E model, archived ChoreoFix merges and the common scorer.
Paired editing costs are still unmeasured. Comparison-client development is
on hold while ChoreoFix's own task experiments continue.

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

EPF checks recorded Endpoint execution against the compiled protocol. The
runner binds each physical frame to one EPF-checked receive and hashes the
same journal and projection bytes it reads. The task ID selects both the
benchmark description and the effect checker. The journal reader verifies the
participant roster, requires one score entry per completed call and rejects a
missing or nonempty violation list. It preserves the simulator's completion
verdict, including expected failures. For each simulator call it pairs
the request, start, finish and reply by RPC ID, including when identical calls
overlap and finish in reverse order. Each task checker receives the complete
call and uses a shared comparison for scored arguments, actor, operation,
result, success and violations. The simulator request carries the RPC ID. The
Endpoint receive is recorded separately in the frame journal. Each task
checker specifies which resource receives can authorize that effect and
where the reply goes, including tasks that require two receipts. The project
maintains these rules alongside the localsides. The common simulator assigns
the business score from its own world state. Run
`choreofix-experiments --audit TASK EVENTS` on an archived
journal. The runner checks the requested task against both the compiled
projection and the recorded fixture. The 4E, 8E, dining and 16E checkers
share the reconstruction of accepted WASI bytes. A short write updates the
file by exactly the acknowledged count. The [audit validation record](analysis/effect-audit-validation.json)
identifies the replayed trials and the records changed to test rejection.

## Build and validation

Use the current stable Rust toolchain, the `wasm32-wasip1` target and the pinned
hibana-wash submodule. The host runner currently uses macOS's event queue.
Keep this checkout beside `hibana`, `hibana-epf` and
`hibana-wasip1-runtime`. Their path dependencies select the current working
sources, with one core shared by the application, runner and effect-boundary
checks. To reproduce CI, use the core, WASI runtime and EPF revisions selected
in the [quality workflow](.github/workflows/quality.yml). Use the
`.github/runtime-revision` file in that exact core checkout for hibana-runtime.
The wash submodule records its own revision.
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
