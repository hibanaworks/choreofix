# ChoreoFS request-boundary audit — 2026-11-12

This is source inspection and a public-API counterexample, not a completed
architecture migration or a successful qualification gate. The source baseline
is ChoreoFix `95219d8`, WASI runtime `d2ad7f4`, core `fb7e1db2d` and EPF
`ad12c78`, published as `progress-20261112` in each repository. The existing AGENTS contract forbids the adapter described here; the new
wording makes that prohibition explicit. No model/scored run is added.

## Required boundary

ChoreoFS is an object interface to projected communication, not shared storage.
The file view can be private to a participant. "Shared operation" below means
an operation affecting a shared resource, not shared file contents. This audit
does not propose a shared filesystem or move authority into pathname metadata.

A shared file/fd must be bound to its actual owned or borrowed Endpoint before
its request is processed. The requested file operation itself must traverse the
business choreography. An independently admitted generic I/O close, followed
by application path/phase dispatch into another protocol, is insufficient.
Path lookup, fd/offset/content storage and codecs are data processing; a second
workflow cursor, permission table or completion table is forbidden.

## Confirmed common mechanism

`src/shell/localside/private_io.rs` accepts any non-result `service/*` pathname
as a writable object at `SERVICE_FD`. It materializes generic `FdCloseReqMsg`
and sends `ClosedFileMsg`, including the pathname and argument bytes. The
runtime's `exchange/files.rs::next_file` returns those data with a pending guest
close. Application code then chooses a business message and performs separate
Endpoint operations before constructing the guest's completion.

There are 65 `next_file`-using source files across 17 accepted families.
Review and dining implement the same split manually. Codebase's design path
also sends private `FdCloseReqMsg`, receives `ClosedFileMsg`, parses
`closed.arguments`, then sends `DesignRequest` on a separate task endpoint.
Its custom typed `Close` elsewhere does not eliminate that path. The confirmed
common issue therefore covers **20 families / 24 tasks** out of the 29
historical task ports.

Those business Endpoints were actually driven, so this finding is not proof
that all their effects bypassed Endpoint admission. Existing evidence retains
that narrower scope. File/Endpoint binding was already required: this is an
implementation/qualification shortfall, not an additional requirement imposed
after those runs. The historical count is not certification of this contract.

The other three families contain direct typed WASI requests and ChoreoFS
materialization as well as later resource operations. They are not all the
same mailbox implementation. Their fd/Endpoint and request/completion coupling
needs per-branch review before claiming the required boundary.

## Full source scope

All 26 current `src/tasks` directories and the common shell private-I/O path
were inspected. The table below covers all 23 families in the historical
29-task count. The other three directories are `handoff` (native mechanism),
`research_paper` (3M native draft), and `manufacturing_rework` (11M partial
draft); none has a qualified Wasm entry or another mailbox implementation to
carry forward. Other unimplemented tasks are not silently included as inspected
implementations or qualification successes.

## Entire accepted-task inventory

`M` means the common mailbox split is confirmed. `D` means direct/mixed typed
WASI handling exists; it is not a claim that every branch is compliant.
The table inventories every family in the recorded 29-task migration count.
It is an inspection inventory, not a string/file-presence CI test.

| Task(s) | Source family | Class | Request entry / concrete evidence |
| --- | --- | --- | --- |
| 1E, 1M, 1H | codebase | M | `src/tasks/codebase/choreofs/localside/wasi.rs` design path: private `FdCloseReqMsg` / `ClosedFileMsg` then `closed.arguments` / separate `DesignRequest`; custom typed `Close` in other paths does not make the whole family compliant |
| 2E, 2M, 2H | writing | D | `src/tasks/writing/choreofs/global.rs` declares operation-specific `Commit` payloads as `FdCloseReq`; `localside/wasi.rs` handles the actual import before resource requests |
| 3E | research_report | M | `localside/author.rs` and `editor.rs`: `next_file`, pathname decoding and business/resource sends |
| 4E | api_coordination | M | `localside/frontend.rs`, `backend.rs`, `tester.rs`: `next_file` and later business requests |
| 4M | shared_refactor | M | `localside/developer.rs`, `architect.rs`, `reviewer_wasi.rs`, `tester.rs`: generic closed-file requests precede task operations |
| 5E | medical_consultation | M | `localside/er_doctor.rs`, `cardiologist.rs`, `pharmacist.rs`: `ClosedFile` validation precedes clinical requests |
| 5M | multi_specialty | M | `localside/er_doctor.rs`, `cardiologist.rs`, `neurologist.rs`, `radiologist.rs`, `pharmacist.rs`: same generic file boundary |
| 5H | critical_care | M | all seven guest localsides call `next_file` before their clinical Endpoint operations |
| 6E | review | M | `src/tasks/review/choreofs/localside/developer_wasi.rs` and `reviewer_wasi.rs`: generic private close, `closed.path` dispatch, then task operations |
| 6M | feature_delivery | M | five `*_wasi.rs` guest localsides use generic closed-file requests before delivery/review/test/CI operations |
| 7E | shared_document | M | `localside/writer.rs` and `editor.rs`: generic file boundary and subsequent document/review operations |
| 8E | api | D | `src/tasks/api/choreofs/localside/tester.rs` receives typed open/write/commit rows, including explicit rejected WASI replies; inspect other guest/service branches too |
| 9E, 9M, 9H | philosophers | M | `src/tasks/philosophers/choreofs/localside/wasi.rs`: manually forwards generic close and then executes fork/eating operations |
| 10E | shared_build | M | `localside/builder.rs`, `integrator.rs`: closed-file arguments precede build operations |
| 10M | cascading_build | M | `localside/builder.rs`, `integrator.rs`, `tester.rs`: same generic file boundary |
| 11E | manufacturing | M | `localside/technician_a.rs`, `technician_b.rs`, `supervisor.rs`: generic file data selects intent/resource operations |
| 12E | kitchen | M | `localside/chef_a.rs`, `chef_b.rs`, `chef_c.rs`: generic file arguments precede cooking/resource operations |
| 13E | lab_synthesis | M | `localside/chemist.rs`, `analyst.rs`, `biologist.rs`: generic close precedes laboratory requests |
| 13M | lab_quality | M | `localside/chemist.rs`, `analyst.rs`, `biologist.rs`, `qc.rs`: same generic file boundary |
| 14E | drug_discovery | M | `localside/chemist.rs`, `biologist.rs`, `toxicologist.rs`, `lead.rs`: same generic file boundary |
| 14M | regulatory_discovery | M | all five scientific guest localsides use `next_file` before task operations |
| 14H | multi_round_discovery | M | all seven scientific guest localsides use `next_file` before task operations |
| 16E | cicd | D | `src/tasks/cicd/choreofs/localside/wasi.rs`: `FdClose` sends typed `Commit` directly, then receives admitted arguments and resource loans; source admission and request equality already have real tests |

## Incomplete tasks

3M has no qualified Wasm application. The two uncommitted guest adapters just
added during development repeated the mailbox design and were removed, along
with the speculative protocol changes made for them. The published 132-event
native component and its source-bound results remain; no new port is counted.
11M, 13H, 7M/7H and the other remaining tasks are likewise not silently treated
as implementations of the revised boundary. Preserve the full 48-task goal.

## Confirmed library API gap

`WasiImportPending::complete` publicly accepts a freely constructed
`WasiImportCompletion`. The real guest borrow is affine, so the same pending
call cannot be completed twice. However that API does not require an admitted
Endpoint request/reply or a ChoreoFS materialization.

A standalone program using only public runtime APIs executed a real Wasm guest
with two `fd_close` calls, constructed both success replies itself, and reached
guest exit 0. It created **zero Endpoints** and performed **zero ChoreoFS
materializations**. This is not a memory-safety result or a demonstration of a
shared simulator effect bypass. It is a concrete counterexample to the desired
public completion/admission contract.

The source, executable, compiler/build records, library hash and actual result
are retained under the WASI repository's
`output/verification/choreofs-unbound-completion-v1/`. The observed result is:

```json
{"actual_wasm_closes_completed":2,"endpoints_created":0,"choreofs_materializations":0,"guest_exit":0,"status":"confirmed_public_completion_admission_gap"}
```

The existing `ChoreoFs` operation methods already consume a nonconstructible
`ReceivedPayload<P>`, rather than a raw request. Keep that constraint. Its
current token carries payload bytes and type/lifetime, not a session/run or
dynamic source-occurrence identity. Merely wrapping a copied completion in a
new type would not establish the stronger file/call correspondence.

## Required implementation and evidence

1. Bind shared file objects to real affine Endpoint values/borrows in the WASI
   runtime. Object IDs and rights masks alone are materialization facts.
2. Keep the guest's current import, selected file/Endpoint and actual admitted
   request/reply coupled until completion. Do not export a generic closed-file
   mailbox plus a publicly forgeable completion step.
3. Remove obsolete public APIs and adapter execution paths, rather than adding
   a new mode, wrapper or fallback beside them. Preserve filename/content/fd
   handling as data and preserve actual Endpoint transfers and concurrency.
4. Check construction/borrowing/move rules with compile-fail tests and actual
   library Endpoint/Wasm executions. Cover missing binding, fabricated reply,
   wrong fd/run/occurrence, cancellation and replay. Miri belongs to affected
   runtime memory boundaries, not each safe ChoreoFix task.
5. Migrate all affected task entries, then join each actual agent/Wasm run,
   complete EPF source witness, request/effect audit and neutral world score.
   Retain all earlier evidence under its original source and scope.

## Current verified library implementation; task migration unfinished

The working WASI source replaces the public raw stepper and `next_file` mailbox
with a guest that owns sealed `ChoreoFsFiles` bindings before execution.
Each mounted `ChoreoFsFile` contains its immutable path/fd and an actual affine
Endpoint. The generic ABI driver exchanges the actual request and reply on
that endpoint and retains the original interpreter call until completion.
Mounted opens must return their declared fd; directory replies cannot alias a
mounted fd. Cancellation keeps these bindings inside the guest. Only completed
exit returns their actual Endpoint values. Application code cannot construct a
pending/completion token, supply an alternative run to `run`, or implement a
custom unbound filesystem/completion callback.

WASI commit `0362074` is published on `hibana-wasip1-runtime/main`. Its complete
local `scripts/check_runtime_gates.sh` passed on fixed source with the canonical
core `fb7e1db2d` and runtime `09a803a`:

- 88 native library tests (including both Lean exporters), plus 8 example tests.
- 10 compile-fail doctests, including the sealed request-materialization guard.
- 20 poll and 14 path-rights Lean theorems; 520 real VM decisions and 160
  ChoreoFS decisions checked against those specifications.
- Strict-provenance Miri: 86 library and 8 example tests. The two export-only
  tests are native/Lean checks, not counted as Miri passes.
- Pico 2: actual Endpoint poll exchange on host and target build/link;
  325,152-byte flash and 154,568-byte RAM including the 24-KiB reserved stack.
  Physical execution and worst-case stack are not claimed.
- Formatting, all-target Clippy, no_std target build, docs, std-Wasm shell
  execution and Cargo archive verification. Archive dependency identities match
  the same canonical core/runtime/macro worktrees.

The whole gate peaked at 702,464,000 bytes under the 2,750,000,000-byte guard.
Source hashes and all gate evidence are retained in the WASI repository at
`output/verification/choreofs-bound-files-final-source/`. CI selects those exact
sibling revisions. No ChoreoFix task Miri was run.

Each mounted file also retains its initial descriptor binding, preserving
preopened access without weakening global admission. Fd metadata now has one
indexed slot for each u8 fd; open/close updates publish after successful guest
writeback without copying the entire table. The 16-subscription poll profile
remains independently unchanged. The original public-API counterexample is
retained under its original source.

Root process I/O/unmounted directory data still require their actual root
Endpoint. Integration must mount shared objects as shared objects; no Rust
type can establish the semantics of arbitrary host business-effect code. The
receiver's ChoreoFS materialization and direct effect/reply remain a separate
obligation. This change does not certify all prior application adapters.

Task globals/localsides must now be migrated to direct file/Endpoint operations;
there is no compatibility `next_file` or raw completion fallback. **No task is
qualified under this boundary yet**, and existing application sources depending
on the removed API require that migration. The 29 historical ports and all 48
target tasks remain recorded; do not count a broken or unexecuted port as done.

## Object-close distinction and adapter removal in progress

WASI `dcfb2c5` distinguishes `FdCloseObjectReq` from an ordinary read-only close.
A typed `FdCloseRejectedRet` requires a nonzero errno and preserves a retry path.
A real-Wasm fixture performs rejected write-close, retry, successful object close,
then read-only reopen/close on the same fd. Only the successful object-close
causes the fixture's effect; reading/closing the result cannot repeat it.

The ordinary library gate passed on unchanged source: 90 native library tests,
8 example tests, 88 library and 8 example strict-provenance Miri tests, the same
Lean correspondence checks, docs, Clippy, std-Wasm execution and package check.
Pico 2 link reports 327,136 bytes of flash and 154,568 bytes of RAM including
reserved stack; the gate's maximum descendant memory was 702,545,920 bytes.
Evidence is `hibana-wasip1-runtime/output/verification/choreofs-object-close-v1/`.
These are library checks, not an application qualification or AI experiment.

The common `file_wire` mailbox, result-chunk protocol and service-path dispatch
were deleted from the working ChoreoFix shell. The common library builds.
The old task adapters still require replacement; neither that library build nor
the new WASI gate certifies them. No additional qualified task or scored run is
counted.

## Dining replacement implemented; real-Wasm qualification unfinished

The working dining global and localsides for 9E/9M/9H now use the bound guest
driver. They no longer forward raw pending calls or dispatch closed pathname
mailboxes. Each think/eat object has an actual file-operation view delegated to
the guest. The receiver consumes standard received WASI requests through
ChoreoFS. Ordinary read close cannot initiate an effect. Invalid argument close
returns EINVAL without modifying the file's bytes, permitting a corrected retry.

An eat effect receiver obtains two real fork requests and both actual fork
Endpoint values, compares their exact arguments with the admitted file bytes,
executes the effect, returns the actual result through both resource Endpoints,
then returns those Endpoints. File-operation views are consumed at guest exit;
they are not physical forks and have no redundant return for future reuse.
The real fork return remains mandatory. The root I/O choreography ends only
after the declared guest/file completion, without an application phase flag.

The complete product binary (including 3, 5 and 7 participants) builds with
`long_running_const_eval=forbid`; the guarded build took 3m08s and peaked at
656,326,656 bytes. All-source hashes, dirty patches, canonical dependencies,
actual binary hash and build-only receipt are retained in
`output/experiments/dining-bound-choreofs-v1/build-final/`. The result explicitly
states `built_not_qualified`, model calls 0 and scored runs 0.

The dining independent auditor and scripted real-Wasm boundary tests still
describe the previous protocol and must be replaced before qualification. The
application's old native task tests also require replacement/removal with the
corresponding new real-Wasm coverage. No historical pass certifies this source.

## Direct localside bodies and resolver registration repair

The application `localside::philosopher::File` wrapper has been deleted. The
3/5/7-participant localsides now declare the file owners as inline scoped async
bodies. Endpoint transfer, receive, offer, parallel resource operations, actual
effect, replies and resource returns remain visible in those bodies. There is
no replacement protocol object or method chain. `FileData` owns bytes, offset
and descriptor data only; it owns no Endpoint and performs no communication.
The application rules explicitly forbid reintroducing this wrapper pattern.

The source audit inspected the function/type declarations across 166 production
localside files in 25 task families, and Endpoint-containing types in the task
and shell code. This is a source inspection, not qualification of every task.
The older two-stage adapters listed above remain migration work.

The first resolved-input version incorrectly registered the same resolver ID
for distinct decisions in one compiled global. Resolver registration is
program-wide, so later registrations replaced earlier input references. This
was an application construction error. Each independent decision now has a
distinct ID in the global and its matching registration. No core production
code was changed, and no localside ordering workaround was added.

The product binary for all three sizes built on unchanged inputs in 3m07s,
peaking at 883,261,440 bytes. Its build-only receipt is under
`output/experiments/dining-bound-choreofs-v1/build-direct-localsides/`.

The actual 9E Wasm run in
`philosophers-rust-boundary-1794492659965890000/9E-normal/` completed all three
guests, all six effects, protocol completion and the common simulator score,
with zero simulator violations. The same run's complete 3,078-record capture
passed the shared static projection and EPF dynamic source audit. This audit
does not certify application effects. The legacy dining effect auditor rejects
the new role layout; replacement of that auditor and the remaining five Wasm
conditions are unfinished. This was scripted Wasm execution, with model calls 0
and scored experimental runs 0, not a real-AI qualification.

The two focused core input-prefix/parallel-offer tests pass. Running the whole
`callsite_regression` target yielded 21 passes and one failure in the existing
`routefree_parallel_progress_preserves_all_completion_and_fault_cases` test:
its send-only `MisplacedOffer` probe returned Pending where immediate rejection
is required. That unresolved result is not counted as a full core gate pass.

## EPF and effect-data correspondence; serial guest self-wait

The dining auditor's separate `held`, `in_flight` and protocol-progress replay
have been deleted. It uses the shared journal reader and EPF's checked dynamic
receive occurrences. Its additional obligation is data correspondence: actual
WASI open/write/read data and close occurrence, actual effect arguments, both
committed resource requests and Endpoint receives, actual effect result and
resource replies, and the common simulator's real score. The receipt report
retains the effect/RPC ID, object frame and dynamic receive, and both resource
receipts; this is offline evidence, not runtime authority or a second source.

Nineteen record mutations are rejected on the saved 9E trace. Five preserve
EPF-valid source execution: coherent effect-argument substitution, coherent
file-write substitution, coherent result-read substitution, an effect before
its second resource receive, and a fresh RPC ID reusing one close for a second
effect. These are the concrete reason to retain the value/effect check in
addition to EPF. The remaining protocol mutations are rejected through the
shared projection/EPF/journal checks, without a dining ownership model.

The source-fixed product build in `build-effect-receipts/` took 3m08s and peaked
at 871,170,048 bytes. Product Clippy passed. Runner native tests were 52 passed,
98 ignored; the ignored mechanism test was then explicitly executed. Its
normal 9E/9M/9H cases completed with matching EPF/effect/score evidence, actual
meal overlap 1/2/3 and all 19 mutations rejected per case. The result is under
`philosophers-rust-boundary-1794494955552470000/`.

The full six-condition test stopped at the first adversarial case: all three
guests attempted `eat` before `think`, and the fd-close imports waited instead
of returning a recoverable error. No domain effect ran in this failing case.
The actual raw trace is retained; the remaining two adversarial cases did not
run and are not counted as passes. No model or scored experiment ran.

This was reduced to a three-role real-Wasm library test, with close order
4,3,4 and required errno results EACCES,success,success. It reproduces in 0.01s:
the guest owns both root and file views, but its early file request waits for
the root action that the same serial interpreter cannot execute while blocked.
Generic `OtherViewPending` is needed by legitimate concurrent Endpoint views;
turning all such waits into errors would break that contract. The correction
must express the actual closed borrow/ownership of the guest's views at the
library boundary, preserving asynchronous waits on independently driven views.
No application flag, phase adapter, request reordering or shortened test is an
acceptable substitute. No core production change has been made for this case.

Library evidence is `hibana-wasip1-runtime/output/verification/choreofs-self-wait-v1/`.
Before adding this newly exposed obligation, the existing ordinary WASI gate
passed on unchanged source: 91 native tests, 8 example tests, 89 Miri library
tests and 8 Miri example tests; Lean, Pico 2 link and packaging also passed.
The new reproducer fails, so those earlier successes are not a claim that the
current library or the six application conditions are fully qualified.
