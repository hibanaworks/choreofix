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

The current inventory is `output/experiments/pilot-inventory-rust-v6/verified-v1/`:
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
