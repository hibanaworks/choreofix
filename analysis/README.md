# Pilot evidence inventory

`choreofix-pilot-inventory INPUT_ROOT AUDIT_REFERENCES NEW_OUTPUT` reads saved model trials. It
does not call a model, rerun historical auditors, qualify a task, or pool trials
with different source and simulator versions. Build and run it under the shared
resource guard, using the existing runner target for reusable registry builds:

```sh
CARGO_TARGET_DIR=tools/runner/target cargo +stable build --locked --manifest-path analysis/Cargo.toml
tools/runner/target/debug/choreofix-pilot-inventory output AUDIT_REFERENCES NEW_OUTPUT
```

The JSON, CSV and Markdown retain each trial's task, condition, source and binary
hashes, recorded auditor verdict, runtime, model settings and inclusive tokens.
Input plus output is the inclusive total; reasoning is not added again. Raw
event streams with identical hashes are marked as duplicates. Missing source,
binary, scorer or audit evidence remains explicit. A hash match does not prove
task correctness or comparison superiority.

The JSON and Markdown also list attempts without an aggregate model report. On a
closed archive the reader recognizes an explicit runner error, a simulator
fixture with a model-connection trace but no aggregate result, or a model
run result with no participant reports. Each directory appears once. These
entries retain the available trace hashes and reported errors. Individual
participants may have finished. Trial-wide token counts and completion remain
unknown until the corresponding evidence is available.
A mechanism fixture alone does not identify a model attempt.
If every recorded attempt lacks an aggregate model report, the tool writes the
inventory and returns an error. The final inclusive token total requires a
completed model turn and a valid usage report from every expected participant.
If the group was interrupted and every participant reported usage, the sum of
their last updates appears under `interrupted_reported_tokens`. Those updates
can precede the interruption. When a participant's usage is missing, both
token columns are empty. A completed model group retains its final token total
even when the common simulator records an unsuccessful task outcome.

When an interruption prevents the guest-runtime report from being saved, the
common simulator can still have recorded its final score in the same trial.
The reader accepts a single complete score message from the simulator in the
hash-verified raw trace. The `world_score_source` column identifies its source.
The guest-runtime report remains missing and the trial retains its failure.

The reference JSON maps each model `result.json`, relative to `INPUT_ROOT`, to
its exact result hash and selected saved audit. An audit reference names a file,
its hash and a JSON pointer (empty for a complete audit, `/baseline` for an audit
inside a mutation report). `audit: null` explicitly retains a run without a
selected audit. Every model result needs one declaration, and unused declarations
are rejected. References cannot escape the evidence root. The reader still checks
raw-event, executable, task and inclusive-token correspondence; selecting a file
does not override those checks. It never guesses the latest successful report
or copies historical audits into another filename. The output saves and hashes
the exact reference map.

The initial Rust inventory is at
`output/experiments/pilot-inventory-rust-v1/verified-v1/`: 13 distinct saved model
trials across 11 task identifiers. Nine archives satisfy the declared integrity
checks. Four have incomplete historical evidence and are retained with their
specific errors. This is an evidence inventory, not 13 newly executed trials or
a final-source all-task certification. The output embeds the exact compiled
inventory source, Cargo manifest and lockfile, and records the inventory binary
hash.

Historical completeness errors have different causes. The 4E pilot declares
`rust_upstream_api`, and its entire archived source map matches, including the
upstream simulator. It predates the common-simulator manifest required by this
inventory; this error does not mean its simulator source was lost. The 3E pilot
also predates that manifest and lacks a colocated full `audit.json`; its saved
summary is not a replacement for that certificate. The older 6E and 9H archives
have missing source files and projection evidence. Keep these distinctions when
selecting paper data, and never relabel an upstream-scored pilot as a run of the
newer causal scoring contract.

The November 11 refresh is at
`output/experiments/pilot-inventory-rust-v2/verified-v1/`: 23 distinct saved raw
streams across 16 task identifiers, including failed attempts. Eleven rows meet
this reader's colocated-evidence checks. This is neither 23 new executions nor
25 separate live-model task certifications. The accepted task-migration count
and the available per-task pilot streams describe different scopes.

The reader used for those v1/v2 inventories required `audit.json` inside the run directory. It therefore
reports an audit-location limitation for the newer 10E, 10M and successful 11E
archives: their final audit is respectively in the parent `audit-v2.json`, the
qualification's `run-record-mutants-v2/report.json:baseline`, and the linked
11E run-record-mutants `report.json:baseline`. Their recorded qualification
references remain authoritative; the inventory does not discover those links
and must not be cited as evidence that those audits are missing or failed.
Those historical inventories remain preliminary and unchanged. The current
reader consumes explicit references instead of imposing that file placement.
The model and Wasm runs were not repeated to refresh it.

The earlier explicit-reference inventory is
`output/experiments/pilot-inventory-rust-v3/verified-v1/`, using the adjacent
`audit-references.json`. It retains the same 23 distinct streams and 16 task
identifiers. Fourteen records now satisfy the declared source, binary, raw,
usage and saved-audit integrity checks, including the existing final audits for
10E, 10M and the successful 11E run. The nine other records remain with their
individual completeness errors; no failed attempt is removed. Two utility tests
and strict Clippy pass. Changed audit hashes, missing/non-object JSON selections
and escaping references are rejected. The full scan used at most 243,793,920
bytes under the shared guard. These are source-bound pilot observations, not
new runs, final-source all-task qualification, success-rate estimates or matched
comparisons.

The earlier fourth inventory is `output/experiments/pilot-inventory-rust-v4/verified-v1/`.
It adds the current-source 6M pilot: 24 distinct raw streams across the same 16
task identifiers, with 15 records satisfying the declared integrity checks and
nine retained unresolved records. The reader source and binary are unchanged;
only the explicit reference map and saved input set changed. The scan used at
most 79,396,864 bytes. This adds one actual unscored pilot, not a scored comparison
or a success-rate estimate across the differing source/scorer versions.

The November 12 snapshot is `output/experiments/pilot-inventory-rust-v5/verified-v1/`:
27 distinct saved streams across 18 task identifiers. Its explicit references add
the two kitchen pilots and the regulatory-discovery pilot. The first kitchen
trial remains an instrumentation failure without a selected passing audit.
No model or historical auditor was rerun by this inventory.

The unchanged reader now reports one complete archive and 26 incomplete ones.
Earlier cleanup deliberately removed superseded experiment-runner executables,
retaining raw records and source/hash; the reader requires every recorded binary
to remain present, including that historical runner. The removals and their
original hashes are in core `output/cleanup-20261112-free-space-v3/deletion-plan.json`.
Keep these current artifact-availability errors explicit. They are neither new
task failures nor a reason to change recorded hashes or relabel earlier
complete-archive observations as current completeness. The individual CSV/JSON
rows retain their actual saved audits, runtime, source and remaining errors.
This snapshot predates the new 14H model trials; those are not included in its
27 rows. No scored comparison or statistical pool is established.

The sixth inventory is `output/experiments/pilot-inventory-rust-v6/verified-v1/`:
29 distinct streams across 19 task identifiers, adding both 14H trials. The
143.119385333-second incomplete attempt remains alongside the 193.161955167-second
successful pilot. Their input/prompt versions differ and are not pooled into a
success rate. The latter uses 1,913,110 inclusive tokens and passes the explicit
saved source/effect/model audit; its exact auditor overlay is retained separately
from the original run's runner and frozen source.

The unchanged reader reports two complete archives and 27 incomplete records.
The added failed 14H trial has no selected passing audit; the historical missing
runner executables remain explicit. These are integrity/completeness categories,
not task success rates. No raw stream, historical model call or historical audit
was rerun by the refresh. Peak scan memory was 79,560,704 bytes. The reference
map, reader hashes, JSON/CSV/table and qualification receipt remain together.

The November 15 inventory is `output/experiments/pilot-inventory-rust-v7/verified-v1/`:
51 distinct saved model streams across 29 task identifiers. The unchanged
reader checks all explicitly referenced raw streams, model usage, source files,
executables, common-simulator manifests and saved audits. Twenty-one archives
satisfy those integrity checks; 30 retain explicit incompleteness errors.
Historical missing executables and source files remain visible. The failed
direct-file 7E and 3E attempts are included, as is the successful 8E execution
whose actual domain result is a terminal test failure. These categories are not
a task success rate or 29 current-path task certifications.

The added references select the exact saved audit and bind its file hash and
raw-event hash. No historical model or auditor was rerun. Source and simulator
versions remain separate, with no statistical pooling or scored comparisons.
The reader source/binary match v6 exactly; the guarded scan peaked at
166,395,904 bytes. The current 10M mechanism matrix is separate and remains
unfinished after 18 normal conditions and a private-descriptor collision in
the following recovery condition. It is not counted as a new model pilot.

The November 19 inventories for the archived 7M and 8M development trials are in
`output/verification/paper-7m-8m-20261119.MZ3ZSr/`. For 7M, one five-participant
trial reached publication with 27 effects and 2,600,761 inclusive tokens. In
the other aggregate report, only one of the five participants finished. Their
last usage reports sum to 2,185,945 tokens and the final total is unknown.
Two further attempts have no aggregate report.

On November 20, five scripted wash guests published the 7M document in each
of six runs on the typed receiver implementation. In each of the 12 figure
insertions, the document Endpoint received the file request, forwarded it to
the figure store, and received the store's confirmation. The simulator call
followed both receives at the document Endpoint. The table gives the frame,
source occurrence and journal row for all three messages, and the row and ID
of the simulator call. All six runs passed dynamic EPF replay and the common
simulator reported completion with zero violations.
The [per-run measurements](7m-figure-receipts.json) identify the input build,
the captures and the rejected figure-ID substitution in the depth-2 journal.

A five-agent depth-one run ended after the fact checker received the revised
chapter and exited. A subsequent scripted run on the same executable submitted
the recheck and published the document. The [paired records](7m-fact-recheck.json)
identify the shell response, simulator outcomes and audit of the scripted run.

For 8M, all four archived model results have matching saved audits, and three
additional attempts have no aggregate report. The three v26 trials at depths
0, 1 and 2 account for 36 effects and 5,882,177 inclusive tokens. The fourth
model result uses v25. The reference maps, raw-result and audit hash checks,
per-trial CSVs and reader source are archived together. These runs were unscored.

For 16M, the November 19 recheck covers eight aggregate reports and one
attempt without a report. The two interrupted depth-1 groups last reported
93,703 and 1,029,054 tokens. Both final totals are unknown. The completed
depth-1 and depth-2 groups used 3,034,750 and 3,190,652 tokens respectively.
Both passed the saved protocol audit and received a negative task score.
The individual results and original audit references are in
`output/verification/7h-paper-inventory.viAUKS/16m-usage-recheck/`.

On 21 November, seven live agents completed 7H at depth zero. The same-run
audit linked 33 effects and their replies to 147,787 EPF records; the common
simulator recorded publication and zero violations. This adds a 37th task and
a 30th family. The original 43-condition cohort below is preserved for its
published aggregates. [7H source, binary, run and audit hashes](7h-review-inbox.json)

The 19 November paper inventory joins the 36 source-bound task qualifications
in `output/verification/paper-36-ports-20261119.CogEVn/verified-conditions.json`.
`conditions.csv` gives the 43 selected task--depth rows with links to their
original result and audit through the archive directory and hashes. These
rows include accepted replacements. Use the per-task attempt inventories
when counting interruptions and retries. Four of the 43 qualified executions
received a negative common-world score. The separate
`16m-effect-intervals.json` preserves request IDs and monotonic intervals for
all 70 simulator calls in the three 16M conditions.

For 3E, 7E, 8E, 13E/M/H and 14E/M/H, the archived task receivers still call
`Domain::execute` with a participant, tool and JSON arguments. These tasks
contribute 12 selected conditions and 178 simulator calls. Each condition's
`directory/source/src/tasks` preserves the receiver, and `build-source.sha256`
authenticates its contents against that condition's recorded runtime manifest.
The 178 counts all calls in those conditions. It does not measure bypasses.

On 21 November, the three 8E operations were rerun with typed Endpoint receives.
The three scripted Wasm trials and two live trials used the same application
binary. The live trials preserved both the successful outcome at depth 0 and
the terminal test failure at depth 2. The [receipts and hashes](8e-received-effects.json)
supplement the original 43 selected conditions.

The new [7E receiver trials](7e-received-effects.json) likewise supplement the
original cohort. Both initial writer orders and revisions across chapters passed
the twelve scripted conditions. The depth-two live trial on the same binary
completed after two negative reviews, a revision and two positive reviews.
The [3E receiver trials](3e-received-effects.json) cover both authors on the
typed effect interface. All 33 scripted Wasm cases passed scoring and audit,
including either author receiving a review before the other starts research.
Three live participants completed the depth-two task on the same binary after
two rejected reviews. The EPF audit checked 5,834 capture records and the effect
audit matched all 13 simulator calls. This rerun adds evidence for an existing
task in the cohort.

The [13E receiver trials](13e-received-effects.json) cover all eight calls on
typed Endpoint receives. All 25 scripted Wasm conditions completed the common
task, with 200 effects and 55,648 EPF records. On the same binary, three live
agents completed the depth-zero task. The audit checked 2,416 EPF records,
matched all eight effects to receives and found zero violations.
Five production tasks still use the caller-selected simulator API (13M, 13H,
14E, 14M and 14H, 77 call sites). They account for seven of the original 43
selected conditions and 141 recorded simulator calls.
The archived events, audit and simulator result for each of the 43
conditions match their SHA-256 digests in `verified-conditions.json`.

The [11M component results](11m-received-effects.json) cover 30 cabinet
schedules and 24 inspection/rework runs after the receivers switched to typed
requests. EPF checked all 6,394 records. The controlled world recorded 246
effects. The six-guest 15E Wasm regression exercised the same simulator reply
code. Both sets of trials used scripted participants.

In the [11M workstation trials](11m-workstation-tools.json), three wash guests
submitted the tool-requiring jobs in all six orders. The common simulator
recorded two overlapping jobs when the guests submitted all three before
reading, and one active job when each guest read its part before the next
submission. All 36 effects were traced back to the station and tool receives.
The other three jobs and the inspection/rework cycle still require a combined
five-participant run.

The [11M inspector trials](11m-inspector-parts.json) added a fourth wash guest.
Across 12 scripted runs the inspector read all 36 parts, including in reverse
submission order after all three workers submitted. EPF checked 20,020
records. Each part read through wash matched the neutral simulator's result.

The cohort-wide measurement is reproducible from the same source-bound trials.
From the repository root, supply the verified cohort and a new output file:

```sh
cargo run --release --locked --manifest-path analysis/Cargo.toml -- --effect-overlap output/verification/paper-36-ports-20261119.CogEVn/verified-conditions.json effect-overlap.json
```

The reader verifies the original event, audit and result hashes and matches every
simulator callback to one domain request and its completed result. It counts a
pair when two different participants' intervals share positive time on the same
runner clock. The report retains both request IDs and measures the peak number
of distinct participants with calls in progress. Calls include simulated delays.

The same reader records every simulator result with `success: false` and the
first subsequent start of the same tool by the same participant strictly after
that call finished. A missing follow-up is recorded explicitly. For a follow-up
the reader compares the complete JSON arguments and records its actual result.
This includes decisions such as a requested revision. A later call may address
a different artifact, and earlier calls can have changed the world even when
the JSON arguments match. Requests refused before the simulator are outside
this measurement. Per-trial counts and source hashes are in
[the simulator outcome summary](pilot-effect-followups.json).

The 43 archived build manifests contain two versions of the Hibana core sources.
Both differ from the core source set in the 7H terminal-delivery build.
`core-source-comparison.json` records the manifest hash for each trial and
the differing paths. The historical qualifications retain their original
scope. Testing the current build requires fresh regression runs.

The companion `model-delivery.json` checks the hash of each of the 197
participant traces. All 2,466 shell calls have a subsequent server completion
whose call ID, arguments and returned contents match the exchange.
The separate 7H cancellation record is in
`output/verification/7h-schema-guide-20261119.FTJvex/material-delivery-depth-0.json`.


### 4H service migration before deployment

The [4H effect table](4h-preparation-effects.csv) identifies the shell request,
effect-bearing Endpoint receive and shell reply for each of the 576 simulator
calls in the [42 archived seven-guest Wasm trials](4h-staging-custody.json).
Frame identifiers are local to each trial. EPF checked 233,130 records from
those executions. The simulator recorded 42 design proposals, 138 builds,
138 reviews, 132 tests and 126 schema migrations. All 42 scores were incomplete
with zero violations. No deployments or registrations were executed.

In 14 trials DevOps read a passing test for a service with no declared
dependencies after one schema migration had returned and before either of
the remaining two requests reached staging. This occurred for B in four trials
and C in ten. Waiting for every schema would delay staging an eligible service.
The [audit results](4h-preparation-effects.json) identify the exact read for
each observation and the six fresh scripted repair trials. At ten of these
handoffs the simulator had no outstanding calls. In ten separate common-world
runs, replaying those prefixes with zero delay reproduced all 94 preceding
effect results. The unchanged simulator then accepted a deployment of C at
each handoff using the same test receipt, with zero violations. The four
remaining handoffs had an in-flight effect and were excluded from this probe.
The Wasm guests did not initiate these deployments.

In the repeated
test case the auditor rejected a retry moved before its own committed request.
It also rejected removal of a failed test RPC and its score entry while the
shell and resource records remained. These checks used no model calls.
