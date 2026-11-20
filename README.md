# ChoreoFix

ChoreoFix tests whether a global choreography can be the sole source of authority
for agents that share resources. Each agent chooses commands in its own
[hibana-wash](https://github.com/hibanaworks/hibana-wash) shell. WASI file
operations on ChoreoFS drive the projected [Hibana](https://github.com/hibanaworks/hibana)
endpoints, and a shared effect requires the corresponding endpoint. We check the
protocol execution with EPF, match admitted requests to actual simulator effects,
and score the business outcome. Whether this design preserves the original
choices or makes a workflow cheaper to change is an empirical question.

## Status

| Question | Evidence through 20 November 2026 |
| --- | --- |
| How many tasks have run through the direct boundary with live agents? | 36 of 48 tasks across 29 families, on their recorded sources. The [inventory](analysis/README.md) identifies 43 selected task/depth conditions. |
| What remains open? | 12 task qualifications and a common final revision. In 4H, seven scripted Wasm participants run through design, implementation, review, testing and schema migration. DevOps can choose any of the six migration orders while holding the staging Endpoint. A failed test now returns the service Endpoint so its developer can submit a fresh implementation. Re-review and retesting of that repair, deployment, rollback and registration remain open. [Development records](plan.md) |
| What has been compared? | 0 of 2,592 original and supplemental episodes scored, and 0 of 1,368 additional planned comparisons. The pilots measure ChoreoFix on their own recorded conditions. |
| Which choices have been checked? | Wasm guests completed all 90 upstream 9E sequences of thoughts and meals, plus one in which B and C ate between A's three thoughts. In 9M and 9H, each seat ate first while the others stayed idle. For every first seat, the remaining diners completed in both directions around the table (24 Wasm trials). Five live diners started in B, A, C, E, D order and seven in E, D, B, G, C, A, F order. Peak overlap was two and three meals, respectively. In 1E all six commit orders ran through wash. [Reproduction and limits](analysis/arbitration.md) |
| Which extra ordering constraints are known in the qualified tasks? | 1H still groups commits into fixed waves. In 3E, A has priority over B for both the initial section write and the reference update. In 6M either approved developer can merge first, although integration still waits for both initial preparations. 6H also orders merges by the required deployment order. In the revised 4M, either module can publish before the other developer starts. The 36 task qualifications each cover the schedules recorded for that version. [Task-by-task evidence](analysis/arbitration.md#fixed-priorities-in-previously-qualified-ports) |

[Paper draft](paper/choreofix.tex) · [Per-task evidence](benchmark/ports) ·
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
The [maintenance study](analysis/arbitration.md#maintenance-study) measures
edits to authorization decisions directly. Comparison-client development is
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
