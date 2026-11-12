# Effect admission

Application verification uses native Endpoint tests, actual Wasm/ChoreoFS,
EPF and independent real-effect/scorer audits. ChoreoFix does not run Miri or
require an interpreter pass for each task. Memory-safety verification belongs
to Hibana and its runtime libraries; application-exposed library defects get
focused library regressions. Historical Miri results below describe only their
saved sources and are not current application gates.

`EffectAdmission.lean` specifies the data checks at the handoff case's physical
publication boundary. It proves, for arbitrary natural-valued targets, revisions
and contents, that publication requires both received requests, the verifier's
actual positive result, and the same current artifact. The equivalence also
checks that an exactly matching valid request is accepted. It uses Lean's kernel
with `propext`; there are no custom axioms or native evaluation.

Run with the Lean toolchain selected by the pinned Hibana proof package:

```sh
lean +leanprover/lean4:v4.30.0 proofs/EffectAdmission.lean
```

The [native case](../src/tasks/handoff) drives actual projected Endpoints and the
publication receiver. The receiver owns the repository mutably and has no await
between checking the requests and publishing. The model's natural values include
the decoded Rust `u64` values; it does not prove the Rust byte decoder.

This is the data-admission component of T4. The effect-to-receipt correspondence
is specified separately in `EffectReceipts.lean`; arbitrary host Rust remains
outside these proofs.

`endpoint-handoff/` compiles the same handoff, transport, bus and receiver sources
directly. It avoids instantiating unrelated task protocols during diagnosis and
does not contain a copied protocol, a replacement transport or a different runtime.
Its manifest resolves the same canonical sibling core/WASI sources and runtime
pin as the application; it is not another product checkout. The full application
tests remain a separate gate.

`regulatory_discovery` compiles 14M's complete business global for all nine
scientists/effect owners and checks its real QC receiver with zero through three
failures across three runtimes. The successful specimen transfer must reject an
old request on its first poll without sending a frame. Its data tests cover all
four selective-reanalysis choices, all fifteen initial/repeated analysis
contracts, both go/no-go decisions and stale regulatory receipts. Native and
Clippy checks are required in `quality.yml`. This is partial
native qualification, not a completed 14M Wasm or whole-study execution claim.

```sh
cargo +stable test --manifest-path proofs/endpoint-handoff/Cargo.toml
```

The fixture uses hibana-runtime and an in-process transport. It checks
handoff after a failed candidate, further reconsideration, two-resource admission,
old approval, changed targets and arguments, and resuming a pending receive after
the missing request arrives. It is not a remote-network or model-call experiment.

B sends its final verification to A and returns R before A forwards that
verification to the publication receiver. A then uses R for the first publication
request while B uses S for the second. This transfer removes B's ability to
repeat its earlier R operations. The receiver continues to check both requests
against the actual verification and current artifact before publishing.

The native tests use the actual host event queue. They check application
behavior; they are not a claim of general memory safety for the libraries.

The actual 13M sources are also compiled directly by `tests/lab_quality.rs`.
Ordinary application tests execute all fourteen declared preparation, biological
input, synthesis and QC receiver conditions. The focused native fixture avoids
compiling unrelated task families during diagnosis; it does not replace those
application tests. Ownership/retry, actual argument substitution and receiver
cancellation remain native application obligations. A 589-event QC prefix does
not become stronger evidence for task correctness by interpreting it in Miri.
Its earlier interpreter records remain historical evidence, without imposing
another application qualification gate.

`tests/nested_choice.rs` checks that a nested choice neither previews nor consumes
a later receive before its visible predecessor. It uses the current dependency
pins and the application's actual transport.

`SourceOccurrences.lean` imports Hibana's actual `SourceOccurrenceExecution` and
proves that every executed point remains observed and that no dynamic source
point occurs twice. Its permitted axioms are `propext` and `Quot.sound`.
Build `Hibana.SourceOccurrenceTrace` in the selected Hibana proof package, then
check this module in that package's `lake env lean` environment. The module
introduces no runtime occurrence counter or effect ledger.

`EffectReceipts.lean` composes the same source execution with the receiver
contract. Each invocation consumes two actual resource receives since the
preceding invocation, compares their data with the actual verification and
current artifact, and publishes once if those checks pass. Independent source
actions and either receive order are permitted between invocations. The relation
is erased proof data; the application stores no occurrence IDs or effect ledger.

The proofs establish that every publication has both correctly bound source
receipts and matching data, that distinct resource roles require distinct
receipts, and that neither receipt can authorize a second publication. Receipt
uniqueness follows from Hibana's source execution; an injective effect assignment
is not assumed. `resources_and_effect_receipts_compose` applies custody to both
distinct resource roles and ties each request's sender to its resource. The two
custody histories must project to the same source word as the receiver trace.
It gives an authority bound for each resource, distinct receipts for the two
requests, and no receipt reuse across publications. The permitted axioms remain
`propext` and `Quot.sound`.

After building the pinned Hibana Lean package, check all three modules together:

```sh
export HIBANA_PROOFS=/path/to/pinned/hibana/proofs/lean
mkdir -p target/effect-proofs
export LEAN_PATH="$PWD/target/effect-proofs:$HIBANA_PROOFS/.lake/build/lib/lean"
for module in EffectAdmission SourceOccurrences EffectReceipts; do
  lean +leanprover/lean4:v4.30.0 -j 1 -R proofs \
    -o "target/effect-proofs/$module.olean" "proofs/$module.lean"
done
```

T3 reuses Hibana's `OwnedCallHistory`, `OccurrenceResourceCustody` and source
occurrence results under their binding and transport assumptions. A legitimate
request can remain executable after its sender drops its Endpoint; authorization
comes from that admitted operation. Requiring a live sender Endpoint at effect
time would incorrectly discard such pending work. Local errors neither undo an
effect nor authorize initial reissuance.

The adapter obligation is explicit: each modeled receive must be the actual
Endpoint receive used by the physical call, with its exact decoded arguments,
verification and current artifact. The theorem does not derive that obligation
from arbitrary Rust or an unqualified frame log. The native receiver tests cover
the concrete call path; qualification of the remaining receivers and their
projection/trace correspondence is still required. The artifact model covers
target, revision and contents; other simulator arguments need their own bindings.

The 6M fixture in `endpoint-handoff/tests/feature_delivery.rs` now uses the
whole delivery source and current receiver bodies. Its four review/test-order
combinations deliberately stop after preparation, once both actual test
definitions and review outcomes have arrived. This Rust completion boundary
does not claim whole-task protocol completion. The first review starts before
the other implementation finishes in all four cases. Two negative tests cover
four substitutions: the feature request, physical repository request's source
or target, and the TEST_ENV target. Rejection precedes the corresponding effect.
TEST_WRITER retains the same real TEST_ENV through both feature tests. The
whole source returns it after final CI; that final return is outside this prefix.
Previous receiver Miri results remain attached to their historical source.
Current Repository and CI behavior is checked by native/Wasm execution and
the independent source/effect audits, without another application Miri gate.

`feature_delivery_source.rs` checks the 768-event source with forbidden
long-running const evaluation and reads every projected row for all 16 requested
views (11 physical actors). `feature_delivery_data.rs` covers 54 merge-data
cases, including both actual requests, the exact approved candidate and the
previous integrated repository. These targets are explicit normal CI
requirements. Old preparation-only execution and CI source paths were removed.
Wasm guests, model-facing interface-change delivery and model qualification
remain required before counting the 6M port.

`feature_delivery_whole.rs` runs 20 whole native retry/merge/CI conditions and
18 substituted input/result cases against the same full source and production
receivers. It replaces the separate CI source and its activation messages.
`feature_delivery_ownership.rs` isolates eight Miri ownership paths using the
same production CI receiver and canonical CI source fragments: success,
ordinary failure followed by cancellation after actual authority return,
rejection before CI and rejection after CI, for both resource identities. Its
fixture-only outer choice explicitly informs conditional peers. It is not
another application implementation. Whole workflow/retry behavior and the
remaining metadata variants stay in the native matrix. No cases are omitted
from either declared domain. CI
results are tied to actual publication/test data and a later CI occurrence;
the common world exports the actual immutable test inputs for this comparison.
Miri does not establish whole-Wasm or whole-workflow conformance. An attempted
whole-source Miri run timed out and remains failed, separately from this
declared memory-boundary domain.

The manufacturing fixture has one native test entry:
`cargo +stable test --locked --manifest-path proofs/endpoint-handoff/Cargo.toml --test manufacturing`.
It imports the canonical global, WASI localsides and physical receivers, and drives
all 24 job orders plus early/late-first Busy/retry/switching and crossed-Ready schedules through
actual Endpoints and the real RPC queue. Its world data fixture and controlled
latencies are not an actual-Wasm or model experiment. The shared RPC borrow
boundary has a separate strict-provenance Miri test. Another Miri test runs the
actual lathe receiver across two runtimes and rejects four invalid wire payloads
before any effect. These do not certify all receiver paths, whole Wasm or host FFI.
The production binary does not compile
a second copy of these native tests.

`multi_round_discovery.rs` imports the production 14H global, data contract and
instrument receivers. Its clinical-owner fixture drives actual MASS and STUDY
Endpoint transfers across two runtimes. Four assay-verdict combinations finish;
four mixed-material variants stop before clinical execution. All eight conditions
pass native and strict-provenance Miri. Other scientific records are supplied
fixture data. The native host uses the real driver; Miri uses the existing
polling clock with the same hibana-runtime scope and receiver. This does not
certify host FFI, complete Wasm, models or the full task.

The study receiver's nine native conditions cover repeated routine/deep reviews,
both instruments held across a pending effect, and material/decision rejections.
Twelve cell conditions cover retests, exhausted response fixtures, replacement of
positive evidence, both actual instruments, purity lineage and retired sends.
Eight joined HPLC/study conditions add QC/purity/stability/documentation retries,
both deep-review tools and five actual material archives. The director receives
the complete actual outcome rather than reconstructing it from IDs. These five
receivers have 47 native fixture conditions; other scientific responses are supplied
values, not a complete common-world/Wasm execution.

The corrected evaluation source orders actual purity delivery before toxicity.
Actual Endpoint probes also retain independent bioassay/stability starts and a
real stability HPLC loan without starting bioassay. Earlier component qualification
passed 35 native functions; an integrated binary later passed 23 native tests.
All seven scientist localsides and startup are implemented. The current material
authority passes through each actual effect owner after formulation delivery.
Twelve source-bound actual-Wasm conditions pass the common world and EPF dynamic
interpretation, including a complete director retry across two scientific rounds.
They contain 192 effects and 106,584 capture records, with zero model calls.
The independent effect auditor now binds all 192 effects to 231 required,
EPF-checked receive commits and rejects 852 saved-record mutations. A separate
seven-model pilot completes thirteen effects and passes source, effect and
model-run audits. These are source-bound port results, not whole-core or
final all-family certification; see `benchmark/ports/14H.md` and its aggregate
`application-qualification-v1.json`.

An earlier combined Miri run was explicitly stopped while finishing the director
message. Although its cell function reached `ok`, the HPLC assignment and
process-end leak check did not finish. Its log, matching before/after hashes and
four subsequently changed original files remain under `purity-causality-v3/`.
Do not relabel that interrupted command or earlier cell/study results as current
full-source Miri success. The subsequent final-source invocation passes mass,
cells and joined HPLC/study: three functions and 28 component conditions,
including process-end leak checks. Its 920-file before/after manifests and
current-source recheck match. Peak memory is 1,371,602,944 bytes; the 3,719.49
interpreter seconds are not native performance data. Evidence is under
`round-matrix-v5/receiver-miri-v9/`. This is historical evidence; normal CI
now executes application receiver tests natively.

### 3M research-paper development

The actual 132-event whole-paper source projects all 13 roles. Nine native
functions cover whole three-author work, repeated factual/editorial rejection,
early archive attempts, changed evidence and missing final custody. Five
conditions call the real common world for all 95 effects. Each passes the
original score, complete EPF capture audit and unique request/effect/custody
correspondence. Forty-seven observation mutations are rejected. These are
scripted native participants, not Wasm or AI qualification.

The previous full-application Miri invocation was deliberately stopped. It is
not a success and is no longer a task acceptance requirement. The application
uses safe Rust; memory-safety checks belong to the libraries. The new native
checks retain the same original cases and assertions. Actual ChoreoFS/Wasm
command freedom, retries and live AI remain to implement and qualify; the port
count stays 29/48. See `benchmark/ports/3M.md` and the source-bound records under
`output/experiments/research-paper-3M-development-v1/`.
