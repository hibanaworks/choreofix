# ChoreoFS request-boundary audit — 2026-11-12

This is source inspection and a public-API counterexample, not a completed
architecture migration or a successful qualification gate. The source baseline
is ChoreoFix `95219d8`, WASI runtime `d2ad7f4`, core `fb7e1db2d` and EPF
`ad12c78`, published as `progress-20261112` in each repository. Subsequent AGENTS
changes prohibit the adapter described here. No model/scored run is added.

## Required boundary

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

There are 65 `next_file`-using source files across 17 accepted families. The
older review and dining implementations implement the same close/mailbox split
manually. That establishes a common architectural issue in 19 families,
covering 21 of the 29 previously source-bound accepted tasks. This does not
establish that their shared effects bypassed their business Endpoints: those
Endpoints are actually driven, and existing tests/audits retain that scope.

The other four families contain direct typed WASI requests and ChoreoFS
materialization as well as later resource operations. They are not all the
same mailbox implementation. Their fd/Endpoint and request/completion coupling
needs per-branch review before claiming the strengthened boundary.

## Entire accepted-task inventory

`M` means the common mailbox split is confirmed. `D` means direct/mixed typed
WASI handling exists; it is not a claim that every branch is compliant.
The table inventories every family in the recorded 29-task migration count.
It is an inspection inventory, not a string/file-presence CI test.

| Task(s) | Source family | Class | Request entry / concrete evidence |
| --- | --- | --- | --- |
| 1E, 1M, 1H | codebase | D | `src/tasks/codebase/choreofs/wire.rs`: `Close<TASK,ROLE,OPERATION>` carries the actual fd-close and implements `ChoreoFsRequest`; inspect every operation-specific WASI sender and resource receiver |
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

The rules and counterexample are established. The bound-file API replacement,
its negative type tests and the task migrations are **not implemented yet**.
