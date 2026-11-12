# Experiment rules

## Application verification boundary

ChoreoFix's acceptance boundary is the actual agent execution, not another
proof of the libraries. Write the global choreography and its direct Endpoint
localsides. Verify that a real agent can freely operate hibana-wash, with WASI
requests passing through hibana-wasip1-runtime, projected Endpoints and ChoreoFS
before any shared effect. The choreography is the only coordination authority.
There is no alternate shared-effect path, app permission table, progress flag,
second state machine or hidden protocol helper.

Do not prescribe a command recipe or the agent's private reasoning. Within the
choreography's capabilities, actual legal command choices, independent work,
concurrency, revisions and reconsideration must remain possible. Invalid shell
input should return a recoverable error where the real protocol permits retry.
An unauthorized effect, missing resource or stale approval must be rejected
before the effect, not merely detected by a later log audit.

Qualify each task through the real model shell -> Wasm import -> Endpoint
request -> ChoreoFS -> physical effect -> reply path. Use actual Wasm runs,
real-model pilots, complete EPF captures and independent request/effect audits;
score the real outcome in the method-neutral common simulator. Include legal
alternative/retry/parallel executions and relevant illegal-effect probes.
Scripted native participants, compilation or protocol completion alone do not
establish this acceptance. Record finite coverage and unfinished work honestly.
Bind the EPF-checked dynamic operation occurrences, their accepted requests and
exact arguments, actual shared effects/replies, and common-simulator task scores
to the same run, binary, source and complete capture. Do not combine audit and
score successes from different executions or relabel old-source results as
current. Check source conformance, request/effect agreement and business outcome
separately: EPF acceptance or protocol completion alone is not task success.
Do not make unrelated foundation reproofs or redundant application validation
additional waiting conditions. Preserve the 48-task scope and remaining work.

Do not run ChoreoFix tasks or application receivers under Miri. Application
code remains safe Rust; the initial physical EndpointBinding construction is
an integration assertion, not a reason to interpret each whole task. Memory
safety of Hibana, hibana-runtime and hibana-wasip1-runtime is checked in those
libraries. A library memory defect exposed by an application needs a focused
regression in the owning library, not per-task Miri jobs in ChoreoFix.

Use native actual-Endpoint tests, actual Wasm/ChoreoFS execution, EPF dynamic
source checks and independent effect/scorer audits for application obligations.
Do not add Miri toolchain installation, application Miri commands or interpreter
clocks to the experiment critical path. Historical Miri evidence stays bound
to its original source and is not a new completion requirement.

## Local validation memory budget

GitHub Actions is available again. Run the required checks on the published
revision as well as the fixed-source local gates; the former Actions budget is
not a reason to skip CI. Distinguish submitted, running and successful checks.

Do not use Docker or start Docker Desktop, a daemon or a VM for measurement.
The measurement core must use no_std, no heap allocation and caller-supplied
storage. Keep host reporting separate. Distinguish CPU instructions, logical
work counts and elapsed time; do not require Linux to validate the kernel.

The user's Mac must not be driven over 4 GB by our validation workloads.
Run builds and checks through Hibana's `.github/scripts/lib/resource_guard.sh`.
It serializes local workloads, measures all descendants including CBMC/LLVM,
and kills the owned workload at 3,000,000,000 bytes with 100 ms sampling.
On macOS use the larger of RSS and physical footprint per process. Cargo and
Kani jobs are 1. `check_kani.sh` includes the guard and must be called directly.
Do not nest an outer resource guard around it. Do not run other heavy checks
beside it or rely on `ulimit -m`, which does not provide this protection.
Save budget stops as interrupted verification. Reduce proof formula size or
split equivalent obligations without reducing the proof domain or safety checks.
Never raise the budget to make a proof pass.

After every successful build, remove superseded binaries, unused target
directories, temporary logs and caches. Delete them rather than compressing
them into archives. Keep the artifacts and cache still used by the current
work, reusable profilers, necessary raw experiment data and source hashes, and
evidence for unresolved failures. Do not delete another task's files or an
active build's storage.
Check free disk space and active output paths before and after every build.
Include this task's retired diagnostic directories, `/tmp` and macOS `TMPDIR`,
not only the canonical target. Do not move unused verification output into a
`retired` directory and retain the entire tree. Keep the latest reusable cache
generation and required originals; remove obsolete profiles, generations and
duplicate content. Preserve other tasks' data and required toolchains.
Store required experiment raw, source/hash, stop evidence and profilers in the
repository's fixed output area, outside Cargo targets and Codex scratch space.



## Mandatory design constraints

Always prefer a simpler, more coherent design, including breaking changes when
they improve the ownership model and preserve the required guarantees. Existing
code and past plans are not reasons to retain a worse design. Replace obsolete
instructions when the design changes instead of appending conflicting rules.
Remove superseded structures rather than retaining aliases, wrappers, modes,
fallbacks, or a second execution path for compatibility unless the user explicitly
requires it. Renaming types or moving helpers is not an architectural improvement.
Evaluate understanding cost, guarantees, expressiveness, concurrency, compilation
cost and physical size. A rewrite alone is not evidence of improvement.
Preserve atomicity and safety; do not weaken checks or tests to simplify a design.

**No helper/wrapper hiding of localside flow. No separate state machine. No
task progress flags. No duplicate coordination or completion management.**

Do not add extra structs, enums, managers, registries or wrappers to coordinate
the application. A new type needs a concrete data-ownership responsibility that
existing types, borrows and moves cannot express. Never duplicate progress,
permission or completion already owned by the global and Hibana kernel.
Keep each participant's send/recv/offer/par, branches, loops and concurrent
completion boundaries directly in its localside body. Do not extract protocol
steps into helper functions or chains of wrappers. Codecs, data validation,
cryptography, buffer operations and actual I/O are separate from protocol flow.

Never work around a core, projection or runtime-checking defect by arranging the
localside differently. Reordering operations, adding notifications or receives,
special-case branches, serialization, helpers or flags to make a legal global
work is forbidden. Fix Hibana when a legal global permits or rejects the wrong
operation. Fix the global when its coordination specification is wrong; write
the localside directly from that corrected projection. Remove existing
workarounds at their cause without weakening regression expectations or guarantees.

Do not use `Rc`, `Arc`, `RefCell` or `Mutex` to share application resources,
file contents or coordination state. Give the data to one participant and send
requests, results and resource Endpoints through the global protocol. A resolver
may inspect its actual decision input; it must not acquire another workflow
cursor. Transport storage and external RPC correlation also need explicit owners
and must not become a second route for task permissions. Existing occurrences
are migration work, not an accepted implementation or a reason to add more.

Write coordination with Hibana `seq`, `send`, `route`, resolvers, `par`, and
`roll`. Hibana's projected runtime is the sole authority for task progress and
permission; Endpoint is the affine view used to perform its logical role's steps.
Keep the actual Endpoint operations directly visible in each localside.
The globals use `par`, `route`, explicit resolvers, `roll`, `seq`, and `send`;
the localsides use the published `offer`, `send`, and `recv` directly.
Use hibana-runtime for scoped localside execution. Remove Tokio from application,
runner, tests and resolved dependencies; do not retain a compatibility module or
a second executor. Complete this removal before starting new live-model
qualification. Host I/O owns descriptors, buffers and RPC correlation, never
protocol progress. Do not introduce a new API name or
move phase selection into a generic guest driver to work around an ambiguous
rolled request. Distinguish the projected communication and ownership instead.
Do not move ordering, authorization, retry, branch, or completion decisions into
a helper, wrapper, driver, callback, filesystem, or transport layer.
Do not recreate them as a stage enum, program counter, allowed-operation table,
approved/done boolean, owner record, or disguised equivalent.

Omit inferred lifetime arguments at every message-type use, including globals,
localsides, tests and examples: write `Loan<RESOURCE>`, `CommitCompleted<FIRST>`
and `ReadyA`, never `Loan<'_, RESOURCE>`, `CommitCompleted<'_, FIRST>` or
`ReadyA<'_>`. Apply this to all messages rather than a list of selected names.
Retain named lifetimes that express an actual borrowing relationship in type
definitions and signatures. Do not introduce aliases or wrappers to hide them.

Rewrite localsides for hibana-runtime's ownership and scoped execution model.
Replacing Tokio macro, task or channel names while retaining their structure is
forbidden. Declare participants directly, pass their borrowed or owned values,
and write each participant through its termination in its localside. Remove
startup join trees, task/result collection tables, duplicate completion checks,
Tokio-shaped wrappers and application schedulers. The I/O participant owns its
data and final I/O; hibana-runtime owns polling, cancellation and waiting for
children. Improve hibana-runtime when natural direct code cannot express a
required operation; never compensate with application adapters, helpers or flags.
Use standalone `async { ... };` children inside one `#[hibana_runtime::scope]`
function, with Endpoint operations visible in each branch. A one-call async child
is valid. Do not add per-declaration attributes, task handles, result receivers,
per-result wakers or tuple-only await syntax. The global and Endpoint::par define
protocol dependencies; the runtime drives the entire Rust child.

Where values merge or borrows must end, await an async body ending in child
declarations. One child returns its successful value; several return a tuple in
declaration order; fixed-array child declarations return arrays of their actual
N. Explicit parent success retains its own value after finishing registered
children. Errors destroy unfinished futures in place. Parent effects between
children must not change output grouping, and ordinary stored futures stay lazy.
Use `for value in values { async move { ... }; }` for fixed arrays; ordinary loops
without child declarations remain sequential. Preserve ordinary Future composition
through variables and function returns. Keep input storage, pinning and child
completion inside the runtime, with one final slot per input.

Use one function attribute, `#[hibana_runtime::scope]`, with Rust async/await
as the basic interface. Inside that function, write independent branches directly:

```rust
async {
    let [mut a, mut b] = endpoint.par()?;

    async { a.send::<First>(&first).await?; };
    async { b.send::<Second>(&second).await?; };

    Ok(())
}.await?;
```

The runtime drives these children concurrently and finishes them before the outer
await completes. A single-call async child is valid. Keep Endpoint operations
visible; the runtime owns pinning, task handles, join construction and child
completion. Keep the existing declaration grammar; do not add custom concurrency
syntax to replace these Rust forms.

Do not recreate Tokio's task or channel framework under different names in the
application. Repeated participants are direct localside declarations from their
actual inputs, without task handles or a result-collection loop. Keep physical
I/O buffers and RPC correlations with their I/O owner, outside protocol authority.

Low-level codecs, buffers, and object materialization belong in `imp/`; they do
not acquire choreography authority by being placed there. Review this rule in
the live source and verification evidence before reporting completion.

Use hibana-wasip1-runtime's ChoreoFS for WASI file operations. Pass the request
received through Endpoint to ChoreoFS and apply its result to the owned data.
Do not add another filesystem, file-permission table, admission check or bypass
beside it. Names, contents, offsets and required I/O data may be stored without
becoming a second coordination specification. Keep ChoreoFS calls visible at
the localside's receive/effect/reply boundary. Prefer ChoreoFix's correct design
over reproducing TraceFix's existing file-operation procedure.

Build task globals from actual business requests, resource Endpoint transfers,
results and returns. Write their localsides from that global. Do not retain a
token-era workflow and replace individual grants with Endpoint payloads.
Remove replaced native coordinators and their admission tables; test the evaluated
delegated protocol rather than keeping a second implementation for qualification.
Private file operations remain in the WASI syscall choreography; completing a
file operation or one rolled visit does not establish business completion.
Place approval-dependent effects and successful termination inside the
corresponding result branch. A continuation after an elastic `roll` does not
establish that a particular outcome occurred inside that roll.

Application code denies unsafe code. The sole exception is initial
`EndpointBinding::new()` at `src/shell/imp/bus.rs`, where the CLI establishes
one fresh simulator trial and distinct physical resource bindings. Keep that
allowance on the construction statement only. All runtimes reaching a resource
share the same binding. Moving or dropping an Endpoint and starting another
session must not mint a new binding. This is the physical integration boundary,
not another source of task permissions.

The experiment covers all 48 TraceFix tasks. Keep comparison implementations
unchanged. The user authorizes repairing a coarse simulator when a reproducible
counterexample violates the stated task. The canonical common simulator is
`benchmark/`, with one fixed source manifest used for every experimental arm.
Do not tune a scorer to ChoreoFix's own procedure, assume misconduct, or infer
superiority without matched evidence. Preserve original-source results and
record simulator changes before runs; never pool incompatible scorer versions.
The simulator must not execute, enforce or grade a preferred choreography or
operation sequence. It models world data, effects and physical resource use.
Outcome evaluation uses actual published artifacts and content-bound review/test
results, equally for every method. Required data causality is part of the common
score: inputs must exist before use, and exact-content review/test results must
be complete before publication. Later evidence cannot authorize an earlier
effect. Preserve freedom among independent legal orders and allow recovery from
failed attempts; never erase an already-observed unsafe effect as recovery.
Keep choreography conformance in EPF; do not copy it into the environment.
Do not use `vendor/tracefix/benchmark` as a fallback or second active simulator. In ChoreoFix, preserve the task goals, participants, resource safety
and necessary data dependencies, while improving the coordination procedure
where the original procedure needlessly restricts independent work or recovery.
Record each procedural change before scoring and identify the shared simulator
contract separately from any additional ChoreoFix criterion. Preserve earlier
original-simulator results under their recorded version. Do not turn a hard
task into a terminal single attempt merely to make its port pass.

Give the model freedom to choose commands, private computations and ways to
solve the task. Protect shared effects with the projected choreography, not a
command recipe or prompt compliance. After a recoverable rejection or failed
attempt, leave a route to reconsider, revise and try again. Express the choice
with `route`, connect an actual result or local decision with its resolver, use
`roll` for repeated work and `par` for independent work. Show the corresponding
`offer`, `send` and `recv` directly in the localsides. A resolver may read the
decision's input; it must not become another workflow cursor or completion table.

Model participants use the existing ChatGPT subscription through Codex App
Server, with `gpt-6-luna` and `xhigh` checked explicitly. Require ChatGPT login
and never fall back to API-key billing, a paid provider, or another model.
Do not request an API key. Keep the comparison runtimes unchanged. Do not spend
development time building a comparison connection or reproducing its response
boundary. Complete Hibana and ChoreoFix first; run ChoreoFix experiments under
the available conditions and record them. An unavailable comparison remains
unexecuted and does not block ChoreoFix's own experiments. Local response
fixtures do not establish that a live connection works.

Hibana global protocols are the only source of coordination rules in the Hibana
condition. The projected runtime owns protocol progress. Do not add a second state machine,
program counter, approved/done flags, lock-owner table, or pending-step tracker
to authorize coordination. Do not serialize independent work to make a task pass.

Keep each task's protocol in `src/tasks/<task>/global.rs` and its participant
functions in `src/tasks/<task>/localside/`. Keep transport, model access, and
domain integration in `imp/`. The global protocol and each role's localside
must be traceable like hibana-quic. Use the actual
published Hibana API. Do not modify Hibana core to add agent-specific APIs.

Each application binary includes its own canonical task module. The shared
library contains host I/O and data codecs, not an unconditional registry of
every task's projected global. Keep task-native Endpoint tests with that binary
so another task's const projections do not accumulate in the same compiler
evaluation. Shared codecs have one definition; do not import an unrelated task
just to reuse a request data type. Preserve every native and effect-boundary
test when changing compilation boundaries.

Check API examples against the current method definitions. The localside uses
`endpoint.send::<M>(...)`, `recv` and `offer`, and `RouteBranch::label`, `send`
and `recv`. Stale design comments do not authorize adding or renaming methods.
Label owned-payload examples as proposed until the real send/recv path works.

If a suspected Hibana bug appears, retain a minimal reproduction and report the
affected version and exact observed behavior. Check the remote current source
before calling it a current bug. Fix confirmed defects in Hibana itself; an
application workaround is not completion. Keep the reproducing regression test
and verify the repair with Lean, Z3, Miri, and Rust tests. Use Kani where bounded
checks of the affected boundary or transition are useful. Check its impact on
the current hibana-quic with the fixed core actually selected. Keep
unrelated/dirty working copies intact. Do
not claim these gates from stale logs or an unpatched consumer build.

Hibana core remains `#![no_std]` with no heap allocation or normal dependencies.
Pico is the size baseline. If required functionality exceeds it in actual size
measurements, the user permits Pico 2 as the core target. Preserve guarantees
and expressiveness, use one API across embedded and host builds, and qualify
stack, SRAM and flash for the selected target. This is not permission for
unbounded storage or arbitrary fixed capacities.
Necessary costs for the current ownership and concurrency guarantees are acceptable.
Do not treat a historical per-workload measurement as a hardware limit or keep
retesting old implementations. Validate the current design with latest stable,
remove clear waste, and justify resource budgets using the total target envelope.
During delegation completion the provisional runtime stack budget is 5 KiB
(5,120 bytes). Defer detailed stack tuning and final Pico qualification until
delegation works; keep the required semantics and concurrent execution intact.
This does not change the separate 4 GB validation ceiling or its 3 GB stop.
Owned model requests, `Vec` and host-side I/O queues belong only to the
application or integrations. They must not move into core. Qualify core
repairs with no-default-feature host and the selected Pico target builds,
including the embedded projection example. The current checked target is still
thumbv6m; Pico 2 is permitted, not yet selected or qualified. Keep the
caller-supplied runtime storage path.

The user's proposed core prerequisite is affine Endpoint delegation. Preserve
ordinary send/recv at the localside while making transfer consume the resource
Endpoint and preserve its current continuation. Initial issuance must establish
one authority for the same resource; merely making Endpoint non-Clone is not
enough. Cover send/drop/failure, distributed peer binding and replay, and the
dependencies of delegated operations, ordinary communication and related sessions
in projection and progress verification. Do not serialize a Rust pointer, revive
uncertain ownership on timeout, or replace this guarantee with an application
owner table. The selected Hibana 0.10 development branch implements owned
send/recv and concurrent endpoint fragments. During development, use the canonical
sibling worktrees for core, EPF and WASI, including the latest core source currently
undergoing validation. Keep application, runner, effect-boundary checks and both
libraries on that one core; verify actual Cargo metadata and locks. Do not select
an older Git pin as a fallback or create another checkout. Keep the core source
frozen during its active validation. Record its Git base, dirty patch and source
hashes, and distinguish development results from completed qualification of that
source. Core examples and proofs do not establish that
ChoreoFix has been ported or that its experiments have passed.

For a shared operation requiring multiple resources, the effect service must
receive the actual request through each required resource Endpoint before it
performs the effect. Use the real result as the reply before returning the
Endpoints. Put those messages and their dependencies in the same task global,
and write their send/recv operations directly in the localsides. Acquire and
return independent Endpoints with par and hibana-runtime. Do not replace the old resource
broker with another permission token plus a separate effect path.

Delegation projection must place an owned endpoint's operations and its
holder's other communications in the same local ordering. Moving a unique
value without those dependencies is insufficient. At route joins, check the
endpoints used by the continuation or a later roll visit; do not require a
redundant return for an endpoint that will never be used again. Preserve safe
path-dependent ownership in the finished design rather than declaring every
divergent join unsafe. Ownership-only reference-model acceptance is not full
Hibana projection acceptance or evidence of distributed deadlock freedom.

Progress verification follows the candidate's actual source dependencies.
Do not require every earlier actor-local index to form a closed history.
Independent parallel work and old continuations after a rolled body can be
unrelated to that candidate. Prove closure as a property of existing progress,
without introducing another runtime state field or an application tracker.

Replace the core's one-Endpoint/one-cursor ownership structure for delegation.
Each actor owns one projected semantic progress, canonical route/roll history
and frontier. Each logical Endpoint is its unique operation view. Keep per-view
pending I/O distinct from that common progress and retain logical ports for the
run across transfers. A view move/drop must not discard accepted frames or
destroy the actor owner. Acquire the common mutation lease before forming a
mutable kernel reference, and release it within each poll so independent views
can remain Pending concurrently. Prepare and publish view issuance/retirement,
progress and required control context atomically. Do not layer a delegation
manager or synchronized independent cursors over the old ownership structure.
Use the same kernel for ordinary communication and delegation, without a public
mode or a compatibility wrapper. Private owned executions do not establish
general distributed continuation or public projection acceptance. Report the
checked source and the exact unfinished boundary.

Keep run identity, faults, registry access and shared workspace in the actor's
session context, outside mutable protocol progress and individual view ports.
Retiring a view's I/O must not remove access to shared progress inspection.
Bind the mutation lease to its actual actor, including actors in one rendezvous.

Color parallel lanes using the physical actor candidates derived from the source
ownership DAG. Distinct logical resource views held by one actor must retain
independent frontier slots. Preserve logical roles and wire peers. Reuse the
existing lane matcher instead of adding a runtime holder table or synchronization
flags. Candidate membership supplies storage and navigation facts; resolver authority still
requires the issued logical view to match the global controller. Resident row
and lane-head locators are caches. Relocating them must not commit events or
select branches.

Preserve valid composition rather than accidental acceptance. In core work,
retain independent `par` branches containing `roll`, ordinary sequential reuse,
receive-headed request/reply repetition, exclusive route alternatives, and
valid outer repetition. Validate ambiguity after role projection, including
global prefixes erased for a particular role. Do not infer that an elastic roll
has retired merely because a later operation committed. Existing passing runs
do not establish that its public sends are unambiguous. Do not add public aliases,
operation modes, or application progress state for compatibility.

Keep a send-selected controller's committed choice while locating its receive
continuation. Completing that send alone does not retire the choice. Completed
observer/resolver visits must still yield to enabled independent resolver work.
Scheduling an enabled entry does not itself authorize either branch.

Use `g::par` for independent work and `g::route` for alternatives. Connect
explicit decision sites to the actual domain result through a resolver. Use
`.roll()` for repeated work and revision cycles. Keep the repetition boundary
and its exit visible in the global protocol and localside functions. Do not
replace them with a stage enum, approval flags, or an application program counter.
Do not add choices or repeated work that the task does not require.

`roll` makes a fragment repeatable; it does not give a route implicit
"right retries, left exits" semantics. Keep any success-dependent continuation
inside the successful branch. Putting that continuation after a rolled route
can admit it after a completed rejection visit too. Check the global rather
than relying on the intended localside to stop only after success.

Resolver registration is keyed by the compiled global program and resolver ID,
not by the role argument alone. Independent owners with independent validation
results need distinct resolver IDs. Reuse an ID only for the same policy owner
and callback data; a RoleProgram argument does not create a role-local namespace.

Preserve parallel acquisition of independent resources in the global and in
actual execution. Issue both asynchronous acquisition requests before waiting
for either grant. Require both grants before the effect and returns after it.
Do not serialize a parallel fragment in the localside. Preserve the task's
outcome, participants, resource constraints and necessary data dependencies;
improve the coordination procedure using Hibana rather than copying legacy
control flow. Keep baselines unchanged and record procedural differences before
scored runs, including departure from sequential-acquisition checklist text.
Run independent model participants and already-admitted domain operations as
concurrent async tasks too. A shared stdout reader must keep reading while an
independent simulator operation awaits I/O. Measure observed overlap from raw
timestamps; declared `par` nodes alone are not evidence of concurrent execution.
Do not retain a serial localside merely to match an older experimental recipe.
Start independent work before awaiting its results and join only where the
global requires both. Keep overlapping participant calls and admitted effects
visible in the persisted evidence. Treat resource conflicts and data dependencies
separately from historical procedural order when documenting fidelity.
Async test fixtures establish scheduling behavior, not measured model overlap.
Keep their controlled delays and barriers labeled in the evidence. Scored runs
must record overlapping actual model calls and independent admitted effects
without inserting test barriers to manufacture overlap.
One physical owner Endpoint alone does not prove exclusive grants: independent
`par` branches can permit several loans. Test the global by driving other legal
Endpoint interleavings as well as the intended localside. Do not qualify a port
when exclusion depends only on waiting for a return before calling `offer` again.
Keep failed global-authority audits explicit and out of scored experiments.

## Shell, WASI, and ChoreoFS boundary

The model chooses its own shell commands, scripts, calculations, and private
work. Do not prescribe a command recipe or use a small command allow-list as
the coordination mechanism. Shell freedom is a requirement, not a claim that
the current bounded guest has already met.

Use the real `hibana-wasip1-runtime` and ChoreoFS. Do not substitute a native
host shell, host filesystem, another Wasm engine, or a second virtual filesystem.
The shell is the small, dependency-free `hibana-wash` WASI guest, pinned as the
`vendor/hibana-wash` submodule from its own repository. Keep shell
syntax and file I/O here, never task-specific commands, permission tables, or
coordination state. Do not grow the runtime merely to embed a general Bash
implementation. Shell variables and syntax are computation, not task authority.
ChoreoFS is the object interface to projected communication. It is not a second
authorization engine. A pathname, fd fact, rights mask, directory listing, or
successful open does not authorize a task effect.

Do not accept shared operations into a generic request mailbox and, after
close, use application path/phase branches to dispatch them to a separate
business Endpoint. Passing generic WASI I/O through an Endpoint does not bind
the file operation to its business choreography. Bind each shared file/fd to
the actual owned or borrowed Endpoint from the outset, so that the operation
itself is admitted by the global. Do not join unrelated private-I/O and business
graphs with an application adapter, callback, permission table or phase logic.
Path resolution, fd/offset/content storage and argument codecs remain data
processing; they must not reimplement allowed operations or expected progress.

Audit the actual request path of every task against this boundary. If ChoreoFS
cannot express it, evolve hibana-wasip1-runtime rather than copying the adapter
or introducing application wrappers. Break the API when needed. A shared file
must carry its real Endpoint authority in its type. The safe public path must
exclude unbound requests, fabricated completion and reuse for another fd, run
or request occurrence. Unsafe integration promises or after-the-fact audit are
not substitutes. Preserve earlier source-bound results without treating them
as qualification of this revised file/Endpoint boundary.

Shell freedom does not grant access to other participants' private objects or
the host filesystem. Private computation may read and write its own objects
without task ordering constraints. If an original task constrains a shared
read, put that read's request, dependency, and reply in the global too. Do not
silently treat all reads as unrestricted or claim that 16E proves such a gate.

Put shared-service WASI requests and task dependencies in the global protocol.
Keep the corresponding `Endpoint::send`, `recv`, `offer`, and branch operations
visible in the localsides. The receiver must consume the projected request and
any required materialization permission before applying the shared effect.
Use ChoreoFS operation results to materialize the admitted object operation.
Static object binding identifies the operation; parsing payloads does not infer
a route or repair an out-of-order protocol.

Pass the `ReceivedPayload` returned by an actual Endpoint receive into ChoreoFS.
This applies to private argument-buffer open/write/close as well as shared
effects. Raw WASI request data and successful codec validation are not received
requests. Complete the ChoreoFS operation before advancing its borrowed Endpoint.
Use the distinct base-write and object-write request types and respect the fd's
binding. Keep private I/O freely selectable through its global route/roll; do not
restore a direct ChoreoFS call to avoid writing the corresponding communication.

The target in an operation's arguments must agree with the projected service
and domain object. Validate that correspondence before materialization. An ABI
conversion may reject malformed or wrong-object input before constructing a typed
request, without sending a message or faulting an Endpoint. The receiver must
independently validate the exact wire payload before committing that request.
Successful payload conversion grants no task step: projection still determines
which message may be sent and what effects may follow its receive. Use explicit
resolver/rejection routes when rejection itself is a shared protocol event, and
keep any dependent effect inside the accepted branch. Do not add a validation
loop or permission flag when a finite typed request expresses the dependency.
Preserve and report differences from the unchanged baseline's argument matching.

The WASI-driving role itself must send each shared-operation request through
its projected Endpoint. The global must connect that request to the service
owner's receive, the admitted operation, and the reply to the WASI role:

WASI Endpoint -> global request -> owner Endpoint -> ChoreoFS -> global reply.

Putting WASI and task protocols beside each other with `par` does not establish
this connection. Do not forward shared requests between those graphs with
`mpsc`, oneshot, callbacks, a shared queue, or another unprojected channel and
then validate them at a later Endpoint. Channels may implement the Transport
that carries projected messages; they may not supply missing choreography edges.
Keep these Endpoint sends and receives visible in the actual WASI localside.
Do not introduce a relay role, a progress flag, an operation-permission table,
or a second state machine to repair missing global connections. Payload, fd and
path conversion belongs at the ABI boundary; coordination belongs in the global.
One Wasm guest may directly own several projected roles from that same global.
Use independent service endpoints when their lifetimes progress independently.
An unqualified `offer()` observes any enabled branch of its endpoint, not the
response for a particular fd or previous request. Do not assume that a request
implicitly scopes the next offer, or drain unrelated previews into progress flags.
Use `offer()` for a projected choice and `recv::<M>()` for a known sequential
receive. A return after a rolled choice is not itself a branch. If a service
must distinguish further requests from guest termination, put that distinction
in the global, receive the actual exit result, then receive the returned Endpoint.
Component tests, simulator completion and green CI cannot establish this
architecture without checking the actual request/reply path.

Updating the Hibana dependency does not complete either the WASI or ChoreoFix
rewrite. Their Hibana localsides must pass the actual operation Endpoint and
use that received value for the request, result and return in the same global.
Do not retain a unit grant that authorizes an operation through another path.

A generic `/tools/call` JSON service followed by a separate task validator is
not the target architecture. Passing that integration does not qualify a direct
ChoreoFS boundary. Do not hide this gap by renaming a helper or a file.

Ordinary shell failures, malformed argument data, and rejected requests return
an error to the model so that it can reconsider and continue. Represent shared
request/rejection/reconsideration cycles with explicit routes, resolvers and
repetition. Distinguish a recoverable domain result from a transport fault or
lost ownership. A retry must remain possible under the domain's actual rules;
never resume a faulted Endpoint, rebuild it to erase progress, or maintain a
second expected-step/allowed-operation table. Record added recovery paths as
ChoreoFix procedural changes rather than silently changing a comparison arm.

File contents, offsets, buffers, transport queues, and RPC correlation are data.
They must not become permission or task progress. Use affine resources and
receiver lifetimes where appropriate; do not disguise an approved/done flag as
an enum, `Option`, callback, channel status, or another filesystem.

Qualify the actual path from model shell command to WASI import, projected
request, ChoreoFS materialization, and unchanged simulator effect. Test invalid
arguments followed by valid work, early and duplicate effects, chunked I/O,
error recovery, independent work, and the original resource constraints.
Compilation, synthetic import tests, and previous generic-RPC episodes are not
substitutes for that end-to-end evidence.

Read `plan.md` before continuing this work. Keep its verified state, next action,
and remaining gates current. Do not replace incomplete evidence with a success
claim or count unaccepted ports as accepted full-task implementations.

The 2026-10-30 research plan makes resource ownership, remaining protocol and
actual effects the central claim. Establish the T3/T4 minimal composition and
its real effect-boundary counterexamples before spending the scored-run budget.
Reuse the existing core semantics for T1/T2, including evaluated par and roll;
record the exact correspondence and assumptions for T5/T6. Ghost ownership and
occurrence histories belong in the model, never in a new application authority
table. Do not expand completion into a proof of arbitrary Rust or all failures.

After a core change, run the complete local gate set, including protocol fuzz,
on the final source. A consumer-discovered core defect needs an actual Endpoint
regression and coverage of its structure, repeated message contracts and schedule
in generated tests. Do not restore old defects for a separate rebuild. Identify whether
the formal model omitted the execution or the implementation correspondence was
unchecked, and strengthen that boundary. Individual proofs or fixtures are not a
successful complete gate.

Audit every effect entry: WASI imports, host APIs, argument codecs and physical
bindings. `Domain::execute` is ordinary host Rust. Its agreement with the
received request, exact arguments and required resource Endpoints is an explicit
adapter trust boundary. Record-mutation rejection only tests the auditor;
test runtime rejection at the effect boundary separately. A local I/O error or
deadline does not revoke remote ownership or authorize reissuance.

Keep the existing 2,592-run comparison and its 864-run first stage. Additional
ZipperGen, Concordat, handoff and causal comparisons are planned in `plan.md`,
not qualified by the existing matrix. Audit native expressiveness before task
selection. Preserve comparator engines and distinguish native system comparison
from matched causal comparison. Count unsupported tasks separately from failed
runs. Freeze precision, reserve allocation and rerun rules before scored results;
do not repeat until the preferred result appears. Preserve negative evidence.

The canonical common simulator is an external environment and scorer. Its
resource and artifact state must not authorize Hibana endpoint transitions.
Simulator state describes the external world, never Endpoint permissions.
Static compilation data and append-only observations must not become a second
runtime progress authority.
Build qualification records may select and pin the measured executable before
a trial. Verify them against the execution's Git commit and preserve the source,
checks and exact artifact hashes. Never consult these records from a guest,
task localside or resolver to grant an operation or advance the protocol.

Rejected operations do not commit progress or cause domain side effects. A
process crash is a failed run, not a successful protocol completion. Protocol
completion does not prove domain task correctness. Score both independently.

All participant model calls use GPT-6 Luna with xhigh reasoning. Record the
requested and returned model, reasoning effort, usage, raw tool events, source
versions, and failures. Never silently substitute a model or a protocol.

Use Rust for application code, experiment tooling, runners, audits, and
integration code. The user's simulator-repair instruction permits maintaining
the original Python simulator and its direct behavioral tests inside
`benchmark/`. External QUIC interoperability tests may also use Python. Do not
add Python adapters, source packaging, model access, WASI integration, profiling
or experiment-analysis scripts. Do not embed Python snippets in Rust or shell
commands to evade this rule.
Existing self-authored Python is migration debt, not an exception. Replace it
with Rust while retaining its behavioral coverage. Keep the original vendored
TraceFix comparison implementation byte-for-byte unchanged. Its original
language belongs to that external comparison and does not authorize Python in
the Hibana implementation or harness.

Do not report a run as measured until its persisted events and simulator result
have been independently checked. Keep harness errors, protocol rejection, domain
failure, timeout, deadlock, and incomplete work distinct.

Use Hayato Ohhashi <o8@vmm.dev> for commits. Do not add AI coauthor trailers.

Do not use development subagents. The separately recorded Luna xhigh benchmark
participants are the authorized experimental subjects, not development workers.
