# ChoreoFix

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
Comparison implementations, simulator, task goals and original scoring remain
unchanged. ChoreoFix may improve the coordination procedure while preserving
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
The current gate fixes one qualified core/EPF/WASI/runtime dependency set and
checks the concrete receiver/effect boundaries on those sources before running
ChoreoFix's own model trials. The minimal receiver-contract composition is
implemented; applying it to every effect entry and completing the remaining
task ports are still required. These obligations do not depend on developing a
comparison client.

The twelve migrated tasks have actual Wasm qualifications for
three/five/seven-person Dining, seven-person collaborative
coding with reconsideration, collaborative writing at three/five/seven participants,
and the three-person API, CI/CD and code-review tasks. Code review includes
revisions after rejected reviews. The API checks
preserve both the original successful outcome and terminal test failure.
Audits check resource transfers, requests, returned results, shared effects,
file reads and guest exits. Record mutations check that the audits reject
missing or altered operations.

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
The Rust [input tool](tools/inputs) restores and verifies
the two unchanged upstream source archives. Python dependencies in
`requirements.lock` support those upstream simulators and comparison clients.

Run one local build or verification workload at a time through Hibana's memory
guard. Set `HIBANA_CHECKOUT` to the checkout used for verification:

```sh
export CARGO_BUILD_JOBS=1
export HIBANA_CHECKOUT=/path/to/hibana
TOOLCHAIN=stable bash "$HIBANA_CHECKOUT/.github/scripts/lib/resource_guard.sh" \
  cargo +stable run --locked --manifest-path tools/inputs/Cargo.toml -- restore
TOOLCHAIN=stable bash "$HIBANA_CHECKOUT/.github/scripts/lib/resource_guard.sh" \
  cargo +stable run --locked --manifest-path tools/inputs/Cargo.toml -- verify
TOOLCHAIN=stable bash "$HIBANA_CHECKOUT/.github/scripts/lib/resource_guard.sh" \
  bash tools/build_shell.sh
TOOLCHAIN=stable bash "$HIBANA_CHECKOUT/.github/scripts/lib/resource_guard.sh" \
  cargo +stable test --locked
```

The [runner](tools/runner) records raw model events, tool calls, source and
binary digests, original simulator results and inclusive token usage. Failed
qualifications and negative results remain evidence. Scoring requires frozen
inputs and qualified implementations for every condition.
Saved source manifests include working-tree changes; `checkout_head` records
the repository's checked-out revision.
