# What the port lets an agent choose

Fork exclusivity answers whether two neighbors may eat together. It leaves open which neighbor may eat first. A protocol can guarantee exclusivity by fixing the winner and still remove legal behaviors from the original task. We evaluate these two properties separately.

## Reachable orders in 9E

The upstream paper fixture fixes each philosopher's fork acquisition order. Its next-state relation admits steps from any enabled philosopher. We extended it with one observation variable that appends the eater at an actual eat transition. It adds no guard to the upstream relation. With the three distinct constants in the fixture, TLC generated 276 states and exhausted 152 distinct states in the extended model. All six orders reached a terminal state with all philosophers done. The previous 9E global prescribed A, B, C. The [revised global](../src/tasks/philosophers/choreofs/global.rs) projects all six orders. A diner submits its validated meal arguments through an Endpoint after its own thought. A table participant receives a ready request and selects the corresponding global branch. The diner needs both fork Endpoints before its meal can execute. The guest's close completes when its close Endpoint is returned after the meal. The result does not establish first-come-first-served fairness, performance of an agent, or an expressiveness limit of MPST.

In six depth-zero runs, a scripted wash guest finished its meal before the next guest began thinking. Each run used the same executable and Wasm module, passed EPF with 1,468 observations, returned six fork Endpoints and scored six completed effects with zero violations. These are reachability witnesses. The waiter chooses a ready request, while the upstream fixture uses ordered acquisition at individual forks. Fairness under sustained demand and equality of every fork interleaving remain open. The six schedules use no model calls.

The paper source blob at revision `87b01a2` is `6198428d2500c58901aa6c52b4d50e155c544451`. We obtained identical terminal orders and counts with the local TLC (rev. `210df3a`, SHA-256 `0f56d6a813368c48008dfbecb23a73ab43a4a5e1bf09f026782a674aa9d4b3c9`) and the published TLC release asset (rev. `bc1b506`, SHA-256 `7e53aa636dac8a3cb0533f343115c0548e03fc0ce5edc7739becf58d3d0fb84f`). Neither JAR matches the historical comparison compiler pin. This is an independent reachability check of the original translated input. It does not qualify the comparison compiler.

To reproduce from the repository root after restoring the paper inputs, use a fresh directory. Set `TLA2TOOLS_JAR` to the absolute path of the TLC artifact being tested, or use the restored local copy. Put Java 17 or later on the search path. The upstream module declares its own name as `Protocol`. The final command prints each observed meal order in an all-done state.

```sh
root=$PWD
upstream="$root/vendor/tracefix-paper"
out=$(mktemp -d)
cp "$upstream/v3_agent_test/tests/fixtures/9E/Protocol_translated.tla" "$out/Protocol.tla"
cp analysis/dining_orders.tla analysis/dining_orders.cfg "$out/"
(
  cd "$out" || exit 1
  bash "$root/../hibana/.github/scripts/lib/resource_guard.sh" java -Xmx256m -XX:+UseParallelGC \
    -cp "${TLA2TOOLS_JAR:-$upstream/lib/tla2tools.jar}" tlc2.TLC -workers 1 \
    -dump states.txt -config dining_orders.cfg dining_orders.tla &&
  awk 'BEGIN {RS=""; FS="\n"} /"Done".*"Done".*"Done"/ {for(i=1;i<=NF;i++) if($i ~ /actions =/) print $i}' states.txt.dump | sort -u
)
```

On the same executable and v31 common world, three live Luna xhigh participants
chose their own wash commands. At depth zero the simulator observed meals in
B, A, C order and completed with zero violations. An independent audit of that
trial matched six fork requests and six returns to the effects and checked
1,578 EPF observations. The trial took 21.31 seconds and used 257,670 inclusive
tokens. The raw journal is in
`output/verification/9e-arrival-20261119.56SgM3/live-9e/` and its re-audit
is in the adjacent `live-audit.json`. This is one unscored run.

## Scoring and recorded orders

The current 9E global permits a diner to think again while the others eat.
Closing the thinking file sends the validated shell JSON to its effect owner.
The guest waits for that owner's acknowledgement after the simulator returns.
Malformed JSON leaves the same shell available for a corrected request. A meal still needs
both physical forks. To check preservation of the original visible actions,
we ran the same observer with `dining_actions.cfg`, recording thoughts in
lower case and meals in upper case. TLC exhausted 1,142 distinct states and
produced 90 distinct terminal action sequences. All 90 ran through real wash
guests and passed the effect and EPF audits on a single executable. A separate
run completed `abBacCaA`: A thought three times, B and C ate between its
thoughts, and A ate last. The upstream model visits each diner's think
instruction once. Thus the observed complete sets compare by strict inclusion
on this six-symbol alphabet. The comparison omits internal acquisition steps
and timing. It establishes no throughput or fairness advantage.

The 90 replays and the additional eight-action run used dining executable
`0290d15e8bda9049af3575103f5105fc42b4b6463a8d855b488381863acd8fa0`.
The eight-action run returned all six fork loans, recorded eight simulator
effects and passed EPF on 1,770 observations. Its wash commands were scripted.

On this executable we also ran three live Luna xhigh participants at depth
zero. Their instructions left the number of thoughts before a meal to each
participant and specified no seating priority. All three chose to think
once. A began eating before B started thinking, and the meals ran in A, C, B
order. The simulator scored completion with zero violations. The audit
matched all six effects and six fork returns and checked 1,574 EPF records.
The trial took 24.23 seconds and used 258,545 inclusive tokens. The replay
results, individual prompts, raw events and independent audit are under
`output/verification/9e-actions-20261119/{final,live-choice-9e,live-choice-audit.json}`.

The November 19 five- and seven-diner globals allowed each participant to think again
from the same shell. In the adversarial Wasm runs every diner corrected a
rejected input, completed two thoughts and ate once. Five diners used ten
fork loans and seven used fourteen. All were returned. Disjoint meals reached
peak concurrency of two and three, respectively. Those binaries still prescribed
priority at contested forks. The November 20 revision below changed the arbitration.

Two unscored depth-zero trials used the same executable and common simulator.
The prompts left each diner the choice of how many times to think. All twelve
live participants chose once. In 9H, A had already begun eating when E and C
first thought. Each trial passed the independent EPF and effect audit.

| Task | Live participants | Meals in observed start order | Peak simultaneous meals | EPF observations | Inclusive tokens |
| --- | ---: | --- | ---: | ---: | ---: |
| 9M | 5 | A C D B E | 2 | 3,270 | 492,341 |
| 9H | 7 | A E F G C D B | 2 | 4,308 | 667,017 |

The journal hashes, fork returns and separate scripted and live action words
are in `output/verification/dining-reconsider-20261119/summary.json`. The
executable SHA-256 is `1feebeed2379a1fbf832a6552c52a704966e4c8060d2ee62ef11ec0601c39e41`.
On that same executable we reran all 90 upstream 9E action sequences and
the additional sequence `abBacCaA`. Every run completed with zero violations,
returned six forks and passed EPF and the effect audit.

For the 90-trace replay, copy `analysis/dining_actions.cfg` into the TLC
directory above and rerun TLC with `-dump source-states.txt -config
dining_actions.cfg dining_orders.tla`. Set `CHOREOFIX_DINING_TLC` to that
directory, `CHOREOFIX_EVIDENCE_ROOT` to a fresh output directory and
`CHOREOFIX_PHILOSOPHERS_WASI_BINARY` to the selected executable. From the
repository root run the following command through the same resource guard:

```sh
cargo +stable test --locked --manifest-path tools/runner/Cargo.toml \
  dining_source_trials::upstream_visible_actions -- --ignored --nocapture
```

The common dining scorer through environment v30 accepted a meal followed by
the same diner's first thought. Environment v31 observes the completion time of
each diner's actual thought before a meal starts. An early meal still executes
in the simulator and remains a violation after a successful retry. A pending
operation earns no completion credit. The 9E scorer test drives all 720 orders
of the three thoughts and three meals through the tool registry. Exactly 90
complete without a violation, covering all six meal orders. Separate 9M and 9H
tests distinguish overlapping meals that share a fork from disjoint meals.
These tests exercise the common world's scoring. The application needs
separate evidence through its guest shells and fork Endpoints.

These are the depth-0 historical live-agent runs selected in the [paper inventory](README.md). We checked each source journal against its recorded SHA-256. A dispatch is a shell command writing to the corresponding service file. An effect start is the simulator's `domain_start` for that tool.

| Task and operation | Dispatch order | Effect start order | Global used by that run |
| --- | --- | --- | --- |
| 9E meal | A C B | A B C | A, then B, then C |
| 9M meal | A B C D E | A C D B E | A and C can overlap, as can B and D. The global assigns priority at shared forks. |
| 1E commit | B A C | A B C | Commits in A, B, C order. Preparation runs concurrently, and testing can overlap the next commit. |

For example, 1E dispatched B's write at 18.766 s and A's at 19.255 s. The simulator started A's commit at 19.296 s and B's at 20.003 s. These timestamps establish the two orders. Dispatch alone cannot establish when every prerequisite was ready. The 9M and 1E task descriptions require exclusive use of shared resources and do not prescribe those priorities. This check enumerated only the upstream 9E fixture. The three selected runs do not estimate a latency or completion-rate effect.

## Fixed priorities in previously qualified ports

The November 19 inventory recorded 36 distinct live-qualified tasks in 43 conditions. A completed run witnesses its own order. The following source inspection compares the programs used for those results with the original task descriptions.

| Task | Upstream constraint | Extra order in the November 19 executable |
| --- | --- | --- |
| 1M, 1H | Developers with overlapping module sets take turns committing. | The [codebase global](../src/tasks/codebase/choreofs/global.rs) groups developers into predetermined waves. B cannot take the database first while A is still designing. |
| 2E, 2M, 2H | Writers with overlapping sections take turns editing them after preparing their own drafts. | The [writing global](../src/tasks/writing/choreofs/global.rs) chooses A before B on their common section. The five- and seven-writer variants retain fixed waves. |
| 9M, 9H | Neighbors need both adjacent forks, and disjoint pairs can be used simultaneously. | The old executable fixed the first user of each contested fork. The [November 20 global](../src/tasks/philosophers/choreofs/global.rs) admits either neighbor first. |

The November 19 3E report port also specified an author priority. Its global placed A's first section write before B's, and A's reference update before B's. The global in the selected depth-2 live record had the same SHA-256 as that version. The [task description](../benchmark/descriptions/3E/description.md) requires exclusive access to each shared resource and review of each author's material. It gives neither author precedence. In the common environment we let the first researcher finish all three preparation steps while the other remained idle. Both authors went first, each with writing before or after their reference update, crossed with both review orders. All eight runs completed, with 72 effects and zero violations. These runs used the simulator's ordinary tools, with no Wasm guest or model. The previously qualified 3E executable has no B-first writing witness. [Calls, results and source hashes](../output/verification/3e-author-order-20261120/common-world-orders.json)

The same 3E version also made the first section review wait for both authors' sections and references. Its review validation required a two-section snapshot even though the actual review identifies one section. In two further common-world trials the editor approved the first author's section before the other author began research. A and B each went first. Both reports completed with zero violations, and each approval referred only to the material it reviewed. [Requests and review receipts](../output/verification/3e-author-order-20261120/early-review-orders.json)

The 3E rewrite gives the first editor request to the author whose material it names. The initial document owner passes the actual section and reference artifacts and the editable Endpoint to the subsequent document owner. Both recheck the first receipt against its section. In two real three-guest Wasm runs the editor returned a review to the shell before the second author started research, once in each direction. Each run completed nine effects, including both reviews and final assembly, with zero simulator violations and zero model calls. Four more runs on the same binary covered either author going first and either writing or updating references first. All six completed 54 effects with zero simulator violations. All six complete journals subsequently passed EPF replay and the independent request-to-effect audit. The early-review witness needed 1,379 derivation steps. The old initial workspace held 1,047 and the search ran for 309 seconds before it was stopped. The runner now also reserves room for the compiled source events before it enumerates smaller witnesses. The proof limit and final checker are unchanged. [Six same-run audits](../output/verification/3e-author-order-20261120/same-run-audits/summary.json). [Early-review scores and exact journals](../output/verification/3e-author-order-20261120/reviewed-material-score-summary.json)

The earlier 4M global joined all four edits before validation. The common world accepted either module publishing before the other developer began. We tested both first developers, both edit orders and both validation orders. All eight trials completed twelve effects each with zero violations. The second developer's shared-library refactor produces the same bytes, so the first module's receipts stay current. [Common-world schedules](../output/verification/4m-independent-validation-20261120/common-world-orders.json)

In the [revised global](../src/tasks/shared_refactor/global.rs) each module reads its own candidate from the sole repository after its developer's two edits. A reviewer and a tester can work independently. Publication rechecks the current physical code against both results. Five real Wasm guests completed the eight module-at-a-time schedules and the eight interleaved schedules on one binary. All sixteen passed the EPF, effect and common-score audits (192 effects, 48,592 observations, zero violations). The production receiver on the same global rejected twelve forged validation inputs. Five live Luna participants completed a separate trial on that binary in 41.22 seconds with twelve effects and zero violations. The gate exercises those sixteen orders. The port still specifies one validation of each kind for each module. [Source-bound results](../output/verification/4m-independent-validation-20261120/verified-results.json)

Two joins in the previously qualified programs restricted integration. The earlier 6M global placed integration after both initial preparations. The [original workflow](../benchmark/descriptions/6M/description.md) allows an approved developer to merge and requires any failed CI to be repaired before the next merge. The common simulator completed a ten-effect trial in which B prepared, merged and passed CI before A started. In [6H](../src/tasks/staged_integration/global.rs), release B followed the join of preparations B and C. The [task](../benchmark/descriptions/6H/description.md) explicitly requires deployment in A, B, C order. The common simulator completed a 21-effect trial in that order in which C first started work after B had already deployed and validated. Both trials used the ordinary tool registry with zero delay and no injected failures. Both scored complete with zero violations. The requests, results, event records and source hashes are in `output/verification/6mh-preparation-order-20261120/common-world-orders.json`. The revised 6H global permits each developer to start implementing after the preceding feature has already passed staging. The seven-guest Wasm test completed all 21 effects on that schedule, including B’s merge before C started implementing. The pre-change executable rejected the same B merge. The separate question of a later feature merging before the preceding stage has passed remains open: the upstream workflow fixes deployment order, while the port also fixes merge order. The 6M schedule still needs actual Wasm evidence. Its resolver also chooses the earlier approval even if both features are eligible and the later-approved developer wants to merge first. The common simulator accepted both merge orders after each approval order (four runs, ten effects each, zero violations). See `output/verification/6mh-preparation-order-20261120/6m-approval-and-merge-orders.json`.

In 6M, two real five-guest Wasm trials finished both implementations, wrote both test suites and approved both candidates. Each time the later-approved developer alone opened `/service/merge_ready`. Both reads remained pending until the 45-second guest deadline, with six completed effects and no merge. The opposite orders blocked the opposite developers. On that same executable an approval-aligned control completed ten effects and passed the audit. The common simulator accepts both crossed orders. The ordinary family gate now assigns review, test-writing and first-merge order independently (eight initial cases and three repair cases). The global used in those trials gave the earlier approval merge priority. One trial rewrite denied the other developer's independent CI-result read; a parallel rewrite made the original and retry submissions ambiguous during projection. Neither replaced the application. See [the journals and build hashes](../output/verification/6m-merge-submissions-20261120/verified-results.json).

On November 20 a candidate 6M global admitted the first actual merge request from either approved developer. The five Wasm guests completed all eight combinations of review, test preparation and first merge, with 80 effects, 48,704 EPF records and zero violations. On **that same executable**, a failed CI followed by a rejected repair review left a developer waiting on `/service/merge_ready` after 14 effects. A different layout passed two recovery cases with 21 and 32 effects. Its third recovery case stalled on `/service/peer_review` after 17 effects. Removing the separate readiness file exposed an earlier wait on a premature merge. We retained the runnable port and the full qualification matrix. Its rebuilt binary matched the pre-experiment hash. Repeating the third repair case on that binary completed 26 effects, with 15,117 EPF records, 488 rejected record mutations and zero simulator violations. These candidates used no model calls. [Run records and executable hashes](../output/verification/6m-merge-arbitration-20261120/verified-results.json)

A further 6M candidate moved each readiness read onto the developer's review file. With both features approved, the other developer's valid integration read was denied after six effects. The repair trial also stopped before any effect completed. The candidate was discarded. Rebuilding the previous source reproduced its exact executable hash. [Failed trials and restored binary](../output/verification/6m-independent-merge-20261120/verified-results.json)

The revised 6M repository accepts either approved developer's actual merge request. The global delivers each interface change before it splits the review and test work. It sends the peer's review feedback on the same sequential path before the other review replies. After a failed CI, a developer can try an obsolete peer read, get a shell error, and continue the repair. The five Wasm guests exercised all eight combinations of initial review, test-writing and merge order, including B merging while the earlier-approved A withheld its request, and the converse. All three review-rejection and CI-repair cases passed on that same binary. EPF checked 93,184 producer records across the eleven trials, the effect audit matched 159 simulator calls, and each trial scored complete with zero violations. One run ran out of disk space while saving setup for a recovery case. The retained and resumed cases have matching source, executable, harness and projection hashes. Five live agents then completed depth 2 on the same binary in 477.89 seconds, with 26 effects and zero violations. Integration still follows both initial preparations. [Cases, journals and audit hashes](../output/verification/6m-peer-feedback-20261120/verified-results.json)

A trial that started the first integration alongside the other preparation failed projection. Its alternatives had different first visible controllers. The approved developer's ability to merge before the other preparation is still open on the runnable global. [Compiler diagnostic](../output/verification/6m-peer-feedback-20261120/early-integration-build.log)

Two additional five-guest 6M regressions submit the approved merge while the other developer stays idle. A and B each finished implementation, test writing and approval, and read their merge readiness. In both directions the merge file operation waited until the 45-second test deadline and the simulator recorded no merge. These cases are in the ordinary family inventory. The auditor now checks the test receipt actually delivered before each publication and requires current test material for every published feature. That version joined both first submissions. [A-first and B-first runtime journals](../output/verification/6m-peer-feedback-20261120/early-ci-volume/)

The revised [6M global](../src/tasks/feature_delivery/global.rs) lets each approved developer submit its merge independently. The repository owner receives the actual request on the delegated merge endpoint. It admits the next publication after CI on the previous one, and a failed CI admits the repaired feature. Each review owner delivers its feature's actual CI result to the existing integration files. The repository finishes with the two returned review endpoints.

All ten initial-order conditions and three recovery conditions passed on one executable. In two conditions one developer completed its merge and CI while the other had not started implementation. Across the 13 five-guest Wasm runs, EPF and the effect auditor checked 179 simulator calls. Recovery runs rejected 1,554 record mutations. The common simulator scored all 13 complete with zero violations. These scripted runs made zero model calls. [Initial orders](../output/verification/6m-peer-feedback-20261120/ready-feature-first/initial-orders.log) · [Recovery conditions](../output/verification/6m-peer-feedback-20261120/ready-feature-first/repair-orders.log)

The 6H common-world check then tried all six permutations of the three merges. Each merge was followed by CI. All six runs deployed and validated A, B and C in that order after the last merge, and each scored complete with zero violations. The extra merge order remains a confirmed difference. The revised port passed the paced and concurrent 21-effect Wasm trials and its 53-effect recovery trial. Seven live agents passed the separate depth-2 recovery pilot, its EPF audit and the same-run effect and score checks. The pilot used the current ordered merges. Its complete record, exact executable and the six common-world schedules are under `output/verification/6h-late-preparation-20261120/`.

The 6H repository retains each publication and its CI receipt. The staging data checks distinguish the feature publication from the later main revision on which CI ran. All six simulator merge orders passed the Rust checks for 18 deployments and their validations. The 27 native tests include rejection of mismatched inputs and invalid rollbacks. [Rust receiver checks](../output/verification/6h-late-preparation-20261120/integrated-material-orders.log)

In the subsequent 6H revision the repository takes an eligible publication request from any developer. Staging receives the requested feature's publication and the current tested repository head before deployment. The staging Endpoint passes from A to B to C after successful validation. Seven real wash guests merged C, B, A and then deployed and validated A, B, C. Concurrent and paced trials also passed on that executable. Each run completed 21 effects with zero simulator violations and passed its own EPF and effect audits. The depth-2 run completed all 53 effects, including two rollbacks, and passed both audits and the common score. The additional premature-release assertion still failed. For four of the five nonblocking reads the shell reported permission denied. For the fifth it reported temporarily unavailable. All five returned control without delivering a release. The test retains the required error on every read and saves the completed effect audit before reporting that discrepancy. These four runs used no model calls. [Run evidence](../output/verification/6h-late-preparation-20261120/independent-publication/)


On the frozen executable the seven-guest Wasm gate passed the ordinary 21-effect run, including the check that B cannot deploy before A validates. In a separate run all three features passed review, then C alone submitted the first merge. Wash rejected the close of C's merge file with permission denied. The reverse-order regression requires ordinary C, B, A merges followed by A, B, C staging and remains failing. [Ordinary Wasm result](../output/verification/6h-late-preparation-20261120/checked-staging-inputs/6h-correct-order-guard.log) and [reverse-order Wasm result](../output/verification/6h-late-preparation-20261120/checked-staging-inputs/6h-reverse-first.log).

The table describes the binaries from the November 19 inventory. We subsequently qualified revised 1M, 2E, 2M, 2H, 9M and 9H programs. The November 19 seven-developer 1H program fixed the initial commit waves and the test continuations. The remaining inventory has not been exhaustively compared with upstream schedules.

A paced real-Wasm trial reproduced the extra 1H order. Only B was allowed to issue commands. B completed design and implementation while the six other developers stayed idle. Both of B's modules were free, yet no commit effect occurred before the 180-second deadline. The journal and test result are under `output/verification/1h-module-arrival-20261120/b-first-before` and `b-first-before.log`. The codebase boundary suite now specifies each of the seven first developers, a complete reverse order, and a failed first test that B must repair before anyone else starts. The common simulator and tool registry accepted the B-first repair order with the first test of each developer forced to fail. Each developer repaired and passed before the next started designing, for 49 effects and zero violations. See `common-world-b-repairs-first.json` in the same directory. That timeout used the previous wave-based executable.

Three trial globals exposed different constraints while removing the waves. The first put different developers' test replies into one shared module's repeating branch, which imposed an unsupported receive order. Moving shell exits out of that branch still imposed an order on revised submissions from different files. When each module received its own requests directly, the initial and repeated operations overlapped on the same public Endpoint selector in parallel branches. The compiler refused all three. The frozen input and diagnostic are in `design-test-replies`, `design-file-exits`, and `design-direct-owners` beside the trial.

Two further 1H candidates compiled. In one, B received its first grant and the next parallel Endpoint operation failed with a phase invariant error. In the other, processing B's request from the file succeeded, but the module's outgoing grant was unavailable while the other developer was idle. Both stopped after design and implementation, before any commit effect. These two candidates were discarded. [First Wasm trial](../output/verification/1h-module-arrival-20261120/arrival-wasm.log) and [second Wasm trial](../output/verification/1h-module-arrival-20261120/ordered-commit-wasm.log).

The current 1H global gives each module owner the typed requests from its contenders. The developer holds its file and two module fragments across its whole participation, including test and repair. The order of acquiring modules prevents circular acquisition. A developer does not wait for an idle developer to be chosen. A real seven-guest trial let B finish its initial commit, fail its test, repair, recommit and pass before C started. It then completed C, D, E, F, G, A. The journal passed EPF and effect auditing, and the neutral simulator scored complete with zero violations. [Same-run journal, audit, simulator score and record mutations](../output/verification/1h-module-arrival-20261120/module-scope-wasm/codebase-direct-1795137643230772000/1H-b-repairs-first/)

On that same binary, seven paced trials placed A through G first in turn. Each first developer finished testing before the next started designing. All seven completed 196 effects with zero violations and passed the EPF and effect audits. Separate trials covered the reverse order, three simultaneous commits on disjoint modules at the original delays, and recovery after repeated test failures. Including the B-first repair, these eleven scripted runs completed 413 effects. They used zero model calls. [Seven first developers and journal hashes](../output/verification/1h-module-arrival-20261120/seven-first-commits.json).

In the previous 1M program, B had completed design and implementation. Both modules were free, and the four other developers remained idle. B could not commit during the 180-second test. In the revised program each module owner receives a typed request from either contender. The owner requests the second module only when the first is available. This preserves an acquisition order between resources while letting any developer go first. Eagerly reserving both had serialized five simultaneous requests in the original-delay test.

All eleven revised 1M Wasm conditions share one executable. Six trials let each developer go first, including a complete reverse order. The normal-delay concurrency trial observed two overlapping commits. All eleven journals passed EPF and the effect audit, and the common simulator scored ten complete. The external-test-failure fixture produced its intended negative business result. Five live agents subsequently committed in the order C, D, B, E, A in 31.21 seconds. Two independent commits overlapped. The audit checked twenty effects and ten resource returns with zero violations. These observations establish alternative orders and overlap for the tested inputs. The source, journals and hashes are under `output/verification/1m-module-arrival-20261120/`.

In the revised 2E [global](../src/tasks/writing/choreofs/global.rs), each writer submits its own prepared draft. The editor can receive any of the three submissions and lends the corresponding pair of section Endpoints. Every pair of writers shares a section. Six [real Wasm trials](../tools/runner/src/writing_boundary.rs) ran ABC, ACB, BAC, BCA, CAB and CBA. In each trial one writer finished its review before the next writer started reading. Each journal contained twelve effects, six returned section Endpoints and 10,776 EPF observations. The common score recorded completion and zero violations. These six scripted trials do not measure how simultaneous submissions are scheduled. The frozen source and journals are under `output/verification/2e-writer-order-20261120/`.

The revised five- and seven-writer programs arbitrate each section independently using the submitted draft. Six five-writer schedules and eight seven-writer schedules placed each writer first and included a complete reverse order. Each writer finished the review of its actual edit before the next began reading. The same executable also passed all fifteen standard writing trials across 2E, 2M and 2H. In each family the injected terminal review failure produced a negative business score. With all drafts ready, two five-writer edits and three seven-writer edits overlapped on disjoint sections. Five live agents started writing in C, D, E, A, B order in 26.55 seconds, with a peak of one write. Seven started in A, B, E, G, D, C, F order in 28.23 seconds, with a peak of two. Both live records passed EPF, effect matching and common-world scoring with zero violations. The frozen build, order trials and live audits are under `output/verification/2mh-section-arrival-20261120/`.

In the original five- and seven-seat [dining trials](../tools/runner/src/philosophers_boundary.rs), A finished thinking and eating and B finished thinking. B then submitted its meal while C and the other diners were idle. Both forks were free. In both trials the simulator saw no B meal before the 60-second test deadline. The scripts used zero model calls. The recorded simulator effect order was A think, A eat, B think. The two journals and the candidate projection diagnostics are under `output/verification/9mh-arbitration-20261120/`.

The revised five- and seven-seat globals receive actual meal submissions. The first submission determines which diner has first use of both adjacent forks. Each remaining fork can serve either neighbor first. A diner must hold both physical fork Endpoints to eat. On the same executable each of the five or seven diners ate first while the others were idle. Six five-seat schedules and eight seven-seat schedules completed, including reverse order. All fourteen real Wasm journals passed EPF, effect matching and the common simulator score. These paced trials exercise the named first seats and both relative orders of every adjacent pair. They do not measure the scheduler's choice between simultaneous arrivals.

On the same dining executable, five live Luna agents started their meals in B, A, C, E, D order (26.68 seconds). Seven started in E, D, B, G, C, A, F order (33.33 seconds). At peak, two and three diners respectively ate at the same time with disjoint forks. The two journals passed EPF and independently matched all 10 and 14 business effects and fork returns. The common simulator completed both trials with zero violations. The three-, five- and seven-seat normal and adversarial Wasm regressions also passed all six cases, including 120 rejected record mutations. The per-run journals, hashes, build receipt and audits are indexed by `output/verification/9mh-arbitration-20261120/verified-dining-results.json`.

We also held the first diner fixed. After A ate, the five-seat guests completed both B, C, D, E and E, D, C, B. With seven seats, the guests completed both B through G and G through B after A. Each next diner started thinking only after its predecessor finished eating. This tests whether a first grant to A imposes a direction around the rest of the table. The ordinary qualification now includes both orders. On the same dining executable all sixteen five- and seven-seat schedules passed the same-run EPF, effect and score checks. The journals contain 196 effects, 196 fork returns and 46,892 EPF records. The schedules used no model calls. See `output/verification/dining-after-first-20261120/verified-orders.json`.

The current gate fixes **each** diner in turn as the first, then tries both directions around the remaining seats. On the same dining executable, all ten five-seat and fourteen seven-seat trials passed. For each physical fork we ordered the two actual Endpoint receipts by their positions in that trial's journal. For every choice of first diner, both neighbors obtained each fork outside that diner's pair first in separate trials. All 296 fork Endpoints were returned. The 296 effects matched the 24 trials' journals and common scores, with 70,816 EPF records and zero violations. These runs used scripted wash commands and paced arrivals. Simultaneous-arrival fairness remains unmeasured. [Grant order by fork and first diner, with journal hashes](../output/verification/dining-seat-orders-20261120/verified-orders.json)

Three live model participants chose A, B, C on the same revised 2E binary. All three finished in 28.14 seconds. The independently audited journal matched twelve effects and six section returns, with 2,823 EPF observations and no simulator violations. The fifteen existing writing trials also passed. In the three trials with an injected terminal review failure the simulator recorded an incomplete task. The run records and hashes are under `output/verification/2e-writer-order-20261120/{live,family}/`.

## Ready developers in 1E

Each pair of developers shares one of the three modules. All pairs of commits therefore conflict, but the task gives no developer priority. In the revised [global](../src/tasks/codebase/choreofs/global.rs), each developer can submit as soon as its own implementation is ready. The repository receives an actual typed request, forwards its arguments to the submitting developer and lends that developer the two module Endpoints. It can grant the next pair when the previous loans return. A developer can run its local tests while the repository serves another developer.

Six scripted trials delayed the other two developers until the first had finished testing, then delayed the third until the second had finished. The common simulator observed all six complete orders. Every trial recorded twelve business effects, returned six module Endpoints and passed EPF with 3,879 observations. Another five mechanism trials exercised malformed or early shell requests, externally injected test failure and the original operation delays. The injected failure produced an unsuccessful business score through a normal protocol exit. All eleven records passed the independent audit. These trials establish each developer can go first while the others are idle. They do not establish FIFO fairness for simultaneous requests.

Three live Luna xhigh participants used the same binary without a prescribed commit order. The simulator observed commits by C, B and A, in that order, and scored completion with zero violations. The audit matched twelve effects, six returned module Endpoints and 4,957 EPF observations. The trial took 35.05 seconds and used 527,383 inclusive tokens. The scripted records and frozen source are under `output/verification/1e-arrival-20261119/{orders,build}/`, and the live record and its independent audit are under `{live,live-audit.json}` in the same directory. The source task permits all six winners' orders. The strict extension over upstream visible traces is the separate 9E result above.

## Choosing a migration order in 4H

The original task assigns one staging environment to the deployment pipeline. Its resource map puts schema migrations on that environment. A single resource imposes mutual exclusion. It does not select which service goes first. The revised [4H protocol](../src/tasks/microservice_migration/preparation.rs) lends the staging Endpoint after design. DevOps chooses a service by writing to its ChoreoFS file; each successful request and reply uses the borrowed staging Endpoint. Testing on the separate test environment can proceed throughout this loan.

We tested all six orders of the three schema migrations using seven real wash guests per run. The tester independently used C, A, B in every run. Each guest could correct an early or mistargeted request in the same shell. The test also submitted the next service's valid arguments before reading the first service's reply. The shell reported an error, and DevOps subsequently performed that migration successfully. Each run produced exactly three migration effects. EPF recorded the staging loan, the request and reply for each migration, and the returned Endpoint. The auditor matched each request and reply to the original simulator call and result.

These are scripted depth-zero runs through the preparation part of 4H. The original task allows the migration orders; this experiment establishes that all six remain available. The upstream 9E comparison above measures a strict extension of a task's visible behavior. The full 4H task still requires deployment, registration and recovery.

The [nine seven-guest recovery runs](../output/verification/4h-test-recovery-20261119/summary.json) also show when testing could start. In each run the common simulator recorded C's test after C's review completed, during B's review and before A's review started. We matched C's candidate and review receipt across the two calls before comparing their start and completion times. In two of the three injected-failure runs, the developer started rebuilding the failed service while another service still had a test outstanding. The third injection hit the last test. These scripts chose C to test first in every run. The nine results establish an executable path through this prefix; none completed deployment and registration. The summary and its individual result hashes are identified by SHA-256 `d14de3700374a0818f45a3c496cfad8ddeaf7369f532e00dfd9fc5f8a651a0a9`.

We next varied test order independently of migration order on one executable. All 36 pairs of orders completed the preparation protocol. Each service tested first in twelve of those runs. Three further runs injected a test failure at the first, second or third test. Across the 39 runs the simulator recorded 510 effects and zero violations, and EPF checked 192,696 observations. In 27 runs at least one test began before a different service's review started. In 15 it began during another service's review. Every test cited its own candidate and a review that had already completed. The reviewer's shell received all 114 positive test results and the three fresh candidates rebuilt after a failed test. Those runs ended before a second review of the repaired candidates. The guests ran scripted commands. Deployment and registration were outside this executable's scope, so all 39 task scores were incomplete. [Per-run orders, counts, revision receipts, and journal hashes](4h-preparation-orders.json)

On a later executable the original reviewer also performed the new review. We failed each of the three test positions in turn. For each position we tried immediate approval of the repair and two consecutive requests for changes before approval. Across six seven-guest runs this produced twelve new reviews, each tied to a distinct revision within its run. The reviewer received the service Endpoint back from the developer. We matched the request, the simulator's decision and both shell replies to that revision. EPF checked 35,130 records. The common simulator recorded 102 effects and zero violations. All six runs ended before retesting or deployment and scored incomplete. [Review receipts and journal hashes](4h-repair-reviews.json)

### Deployment and registration in the common simulator

Can DevOps register A while B and C still await deployment? We tested the common simulator with three services whose reviews, tests and schema migrations had succeeded. For each of the 25 acyclic dependency graphs an architect can declare on these services, we tried all 90 orders of the six remaining actions in which each deployment precedes its own registration. We injected no failures. Across the 2,250 graph/order pairs, 720 completed with zero violations. The other 1,530 recorded violations. The zero-violation cases were exactly those where every declared dependency had already deployed when its dependent service deployed.

| Dependency graphs | Legal orders per graph | Graph/order pairs with zero violations |
| ---: | ---: | ---: |
| 1 | 90 | 90 |
| 6 | 45 | 270 |
| 6 | 30 | 180 |
| 12 | 15 | 180 |

A barrier requiring all deployments to finish before any registration would lose 432 of the 720 legal pairs. This count follows from the ordering constraints. Each fixed deployment order admits 15 interleavings with the registrations. Only six put all three registrations last. For example, with no declared dependencies, deploying and registering A before deploying B is a valid completion prefix. In a graph where B depends on A, B may deploy once A has deployed, even if A has yet to register.

These measurements exercise the scoring environment. The seven-guest Wasm experiments above cover preparation and schema migration. The [per-graph measurements and simulator input hashes](4h-deployment-orders.json) identify the exact evaluation inputs. The summary's SHA-256 is `bdec659bc61b38a1907a6e864d448f61c21ff9f166ae02618c991b1625b0af9f`.

To reproduce the seven-guest preparation test from the repository root, build the `microservice_migration` family with the runner's `--build-wasi` option. Set `CHOREOFIX_MICROSERVICE_MIGRATION_WASI_BINARY` to the built executable and `CHOREOFIX_EVIDENCE_ROOT` to a fresh directory. Run the following under the repository's resource guard.

```sh
cargo +stable test --locked --manifest-path tools/runner/Cargo.toml \
  microservice_migration_boundary:: -- --ignored --nocapture
```

## Choosing a rebuild in 10M

Two scripted, five-guest Wasm runs began with the same A, B, C build order. Each used the same executable, shell module, depth, delay, invalid-input probes and injection settings. The first test of A found unavailable inputs. The first tests of B and C failed by injection. All three modules were therefore eligible for reconsideration. The integrator chose a module by writing its name to the projected `/service/rebuild` file. The next fresh assessment also failed by injection.

The other five injection cases varied the initial build order as well. The table lists the actual order of simulator effects. Every trial reached the final link with zero violations.

| Initial builds | Guest preference among eligible modules | Actual rebuilds | Matched effects | EPF records |
| --- | --- | --- | ---: | ---: |
| ABC | ABC | AABC | 15 | 48,589 |
| ABC | CBA | CCBA | 15 | 46,070 |
| ACB | CAB | CCAB | 15 | 47,049 |
| BAC | BCA | BBCA | 15 | 46,884 |
| BCA | BAC | BBAC | 15 | 48,633 |
| CAB | ACB | AACB | 15 | 46,246 |
| CBA | ABC | AABC | 15 | 46,576 |

Every module was rebuilt first in at least one trial. The first two rows also isolate the effect of changing the guest's preference with the initial order held fixed. In both, the effect auditor rejected a coherent substitution of another module name in the recorded shell choice. This checks whether the auditor can distinguish the choice from the module actually rebuilt. All seven trials used scripted guests and zero model calls.

The runtime SHA-256 was `a6a95a2f92881be1a91dcc9a623f96a66783fedee928a66df66e01f6c50fdc3d`. The journals have SHA-256 `106d0ad5ed6be29a3c21b043ba8f685027dc88ed2dd6a9cac0a2cad065f4930d` and `6d19548df1247ad3c67d2f2e309209a5ca44256ea58b9a3392d27a28b638e5aa`. The [Wasm regression](../tools/runner/src/cascading_build_boundary.rs) includes these two conditions in the `cascading_build_boundary::` qualification.

Across all 27 scripted conditions on that executable, the common simulator finished and the EPF and effect audits passed. A separate depth-2 pilot used the same executable with five Luna xhigh participants. At 164.51 seconds the integrator selected module B for a rebuild. After that selection, builder B tried the update, compile and outcome files and did not read its reconsideration file. At 308.68 seconds it issued `exit 0` before a final link. Hibana reported `EndpointDropped`. The simulator recorded eight effects and an incomplete task with zero violations. This live pilot failed. Its journal SHA-256 is `455b05f96c448df8045db7df6e33d3474da5e6eb5fab2ac6e21a5225bca7d334`.

## Admission to a mechanism comparison

For each paired task, record which first contenders are legal, which independent effects may overlap, when the agent receives a reply, and how the same agent proceeds after an early or malformed request. Use the same legal and prohibited examples, simulator, prompt information, action interface, budget and deadlines on both arms. Exercise an early call followed by a corrected call in the same session and observe whether either call produced an effect. A blocked call, a recoverable error and a terminated run are different outcomes.

In the public `main` bridge checkpoint (`c70615f`), an early 9E eat returned a trial-wide failure. In the saved November 17 adversarial Wasm run (journal SHA-256 `770d288588f35baa91d0c4aa5dc5befe6208cdff7252c8ed95a6bfc2709b1100`), command 8 from A returned a shell permission error; the same guest later completed think and eat. Six domain calls were recorded across the three diners and the simulator reported completion and zero violations. The current [dining boundary test](../tools/runner/src/philosophers_boundary.rs) submits the early call through wash, checks the returned shell error and zero effects, then continues in the same guest. In the pinned latest TraceFix monitor (`36b4bd1`), the agent dispatcher catches a coordination protocol violation and returns a tool error; out-of-order operations can return corrective guidance. Its separate domain-tool dispatch handles an eat call. Rejecting a coordination message and rejecting a domain operation are distinct events. The same user-visible recovery still needs a paired trial before attributing an outcome difference to enforcement.

A procedure mismatch stays visible in the full task inventory. Such a pair can support a native-system comparison if labeled with its procedure, but is ineligible for an estimate attributed to enforcement alone. A resource acquisition optimization is a separate procedure change. These criteria apply before scored results are inspected.

## Maintenance study

The following paired change requests are specified before any maintenance results. Each starts independently at a frozen revision of each system with a working, matched initial procedure.

| Task | Requested change | Acceptance observations in both arms |
| --- | --- | --- |
| 6E review | Require an additional independent approval for the exact candidate before merging. | Either approval can arrive first. One approval, duplicate approval by one reviewer, or approval of an earlier candidate is insufficient. The two current approvals permit one merge. |
| 16E deployment | After a failed deployment check, require a fresh successful check of the replacement artifact before deployment. | A stale success and a failed recheck both leave deployment unavailable. A successful recheck on the replacement permits deployment. |
| 9E dining | Starting from the same fixed order, admit any first diner while retaining both-fork exclusivity and progress to all three meals. | Exercise each of the six completed meal orders and competing neighbor requests. The common world records the forks held during each meal and checks completion. |

For the 9E ticket, record the upstream model's existing coverage separately. If matched starting implementations cannot be established, report the ticket as unmatched; do not charge either system for an artificial change. Freeze the initial programs, ticket text, acceptance cases, toolchain, editing budget and editor assignment before collecting results. Counterbalance system order across editors. Record unfinished attempts, the first failing observation and time spent.

The primary measure is the number of independently maintained authorization decisions whose definitions must change. Count two manual rules that can disagree about the same request as two sites, even in one file. Count references and generated artifacts separately. Two reviewers label the edits independently and record disagreements and their resolution. Include the edits to task-specific auditors, scripted guests and acceptance tests in the total change cost. Report those edits separately from global, localside, simulator and engine edits. Also report elapsed editing time and failed acceptance cases. Report each paired ticket individually. The completion rate or token use of an agent episode does not measure any of these quantities.

## Rechecking the four participant-order constraints

| Task | Observed freedom on the revised choreography | Evidence |
| --- | --- | --- |
| 1H | Each of A through G completed a first commit while the other developers were idle. A separate schedule overlapped three independent commits. | [Seven first-committer journals](../output/verification/1h-module-arrival-20261120/seven-first-commits.json) |
| 3E | Either author can write and update references first. The editor can review B before A starts research. Six scripted Wasm runs completed with 54 effects. Three live agents then chose A first and reviewed A before B finished updating references. Their run took 58.41 seconds, used 534,606 inclusive tokens, and passed EPF, the nine-effect audit and the common score. | [Six scripted runs](../output/verification/3e-author-order-20261120/same-run-audits/summary.json) · [Live audit](../output/verification/arbitration-live-20261120/3e-authors-audit.json) |
| 6M | Either approved feature can merge and pass CI before the peer starts. The ten-condition Wasm rerun checked 100 effects. In both one-feature-first cases a nonblocking read of the other publication returned control and the correct feature was subsequently checked. | [Wasm outcomes](../output/verification/arbitration-live-20261120/independent-reads/feature-delivery-rust-boundary-1795149056854127000/outcomes.json) |
| 6H | Seven Wasm guests exercised all six orders of three approved merges on one binary. In each trial CI probed a different, unpublished feature before receiving the first actual publication. CI completed in merge order, then the guests deployed and validated A, B, C. All six common scores were complete, with 126 audited effects, 92,628 EPF observations and zero violations in total. | [All six orders, audit counts and journal hashes](../output/verification/6h-merge-permutations-20261120/verified-orders.json) · [Interrupted live trial](../output/verification/arbitration-live-20261120/6h-independent-merge/interruption-reason.json) |

In a second depth-2 live 6H trial on the same Wasm and host executable, the CI participant tried nonblocking reads of all three publication files by 20.3 seconds. Each returned control. At 42.9 seconds it opened A's file in blocking mode. C merged at 110.6 seconds. The reviewer later waited for C's CI result, and A's read never returned. The trial reached its 1,800-second deadline after 14 effects. The common score was incomplete with zero recorded violations. This records a real failure to finish even though C was allowed to merge first. [Commands, merge time and journal hash](../output/verification/arbitration-live-20261120/6h-publication-choice-summary.json)

The simulator score, the permission to take an alternative order, and the order chosen by live agents are recorded per trial. These runs were unscored pilots.

## Inspecting the reference wafer in 15E

The original description has the integration engineer check film integrity and then etch uniformity. Neither tool takes the other instrument's reading as an input. We kept the requirement to complete both checks on the same aligned reference wafer and let the engineer choose the first instrument. The published task has five instrument tools. The common environment applies this change to every method.

In the [final inspection fragment](../src/tasks/wafer_fabrication/spot_checks.rs), the aligner lends the wafer Endpoint to the integration engineer. The holder's first typed request selects the branch. Each chamber receives the request and returns its actual result. Both branches return the same Endpoint after the second result.

Two native trials used the same compiled fragment and common environment v32. In each trial the runner issued 17 instrument calls before the loan, including a second thickness measurement before either other fabrication stream started. It then exercised the two final effects through the projected Endpoints. One trial used the original film–etch order and the other used etch–film. Both completed with 19 instrument effects and zero violations. For each final pair, EPF checked 28 records covering six accepted transfers. The test independently matched the two committed requests and both replies to the actual simulator calls, including arguments, results and their positions in the journal.

The evidence and source digests are under `output/verification/wafer-15e-20261119/`. These runs cover the native final inspection fragment. A six-agent wash trial and EPF verification of the upstream fabrication, measurement reports and final dispositions are still required for a 15E task qualification. No model calls or scored comparisons were made.

On November 20 the [two chamber receivers](https://github.com/hibanaworks/choreofix/commit/0481bfc)
were retested using the typed resource request as the sole input to each final
simulator effect. Both orders passed with six checked Endpoint send occurrences
and 28 EPF records per trial. The common score completed with zero violations.
The preceding 17 simulator calls were setup for this native fragment. Full
six-agent Wasm and live-model qualification of 15E remains open. The film-first
and etch-first journal SHA-256 digests are
`dfd6656802ea5f22248ec5048f46c64f0846c9b21a8083240d40bf853e0aa364`
and `253898335dc0ed2b1843c461f68306480cd18ae795d2f013b3f64c731bc0997b`.
