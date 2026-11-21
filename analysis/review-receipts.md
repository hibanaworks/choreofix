# 6E review requests and simulator effects

The tested ChoreoFix source is [2e39c88](https://github.com/hibanaworks/choreofix/commit/2e39c88).
On November 20, all 34 scripted real-Wasm conditions completed on the same
host binary. EPF checked 158,572 producer records. The separate effect audit
matched 316 simulator calls, and all 34 simulator scores completed with zero
violations. The auditors rejected 3,080 modified journals. One trial also
exercised an invalid stdio path, a NUL in a shell command and exhaustion of
private work files. The same wash guest then read its previously written file
and finished its task.

At failure depth 2, three Luna xhigh participants completed a separate live
trial in 87.79 seconds using 608,365 inclusive tokens. That trial's EPF
audit checked 5,993 records and the receiver audit matched 10 effects, including
4 actual reviews. The common simulator reported completion and zero violations.
Neither the scripted conditions nor the live pilot count as scored comparisons.

| November 20 artifact | SHA-256 |
| --- | --- |
| Host binary in both tests | `3298a5517a9545abf48f5e4a6297d44ee65547aa4654d40ef3204372c42a3682` |
| Build source manifest | `e0c224944ce8e5da22055b1b1aa2baad6f950b6ea5c1cde9a323b92f51acfb4d` |
| Live journal re-audited by the runner | `7755efceb8cabd4ca0b147822b513b188c22f998e2dca3166f745da654f4f15c` |
| Common simulator manifest | `6fb3593772b07bc5557d20376f1ed0cfbbfb2542430e75bb43beec70bd13c8c4` |

To reproduce the scripted cases, build the runner and use
`choreofix-experiments --build-wasi NEW_DIRECTORY review` from the repository
root. Set `CHOREOFIX_REVIEW_WASI_BINARY` to the resulting review-wasi executable,
`CHOREOFIX_REVIEW_BUILD_RECEIPT` to its review-build-receipt.json,
`CHOREOFIX_AUDITOR` to the runner executable and `CHOREOFIX_EVIDENCE_ROOT` to
a fresh output directory. Run the following on the supported macOS host with
the workspace resource guard.

```sh
cargo +stable test --locked --manifest-path tools/runner/Cargo.toml \
  review_boundary:: -- --ignored --test-threads=1
```

The pilot used `choreofix-experiments 6E 2 NEW_LIVE_DIRECTORY` followed by
`choreofix-experiments --audit-run NEW_LIVE_DIRECTORY`. The public repository
contains the sources, commands and digests. The original live transcript is
held in the research archive.

## Starting procedure for the maintenance ticket

The planned change adds a second, distinct reviewer for each exact candidate.
On 22 November we checked the one-reviewer starting condition at three boundaries.

* **Upstream model.** Both pinned TraceFix distributions supply the same 6E
  translated fixture (SHA-256 `fcfef5688d735511e822018ae3c2ea248ffd93124a26fcea8357cb360fb02d20`).
  Using the TLC JAR with SHA-256 `0f56d6a813368c48008dfbecb23a73ab43a4a5e1bf09f026782a674aa9d4b3c9`,
  we checked the predicate below over `Init` and `Next` with the three role
  constants assigned distinct model values. TLC generated 1,130 states,
  visited 500 distinct states and exhausted its queue. The state dump contains
  both a completed A with B still at its first edit and a completed B with A
  still at its first edit. The model exchanges `approved` and `revise` labels.
  It has no artifact digest or physical merge receipt.
* **Observed merges.** We re-read all 34 scripted 6E journals from the November
  20 binary identified above. Each contained one terminal simulator score,
  identical to the one in its result. We verified the journal hashes and used
  the source and review indices in each merge to look up the original simulator
  events. All 68 merges named the same feature, source occurrence and candidate
  digest as their successful review. The implementation finished before that
  review started, and the review finished before the merge started. Counting
  distinct successful reviewers of that source before the merge gave one in
  all 68 cases. A merged first in 17 runs and B in 17. The schedules were
  scripted. This is a new reading of the archived trials.
* **Common scorer.** We ran five fresh depth-zero trials directly through
  `load_tools("6E", sim=SharedUtilityFileSim())` on ChoreoFix revision
  `de31bd9`, using a fresh world each time and zero simulated delay. A
  implemented `user_authentication`, followed the actions below and merged
  `dev_a_feature`. B then implemented `notification_feature`, got one review
  and merged `dev_b_feature`. Every merge executed. The score and any
  violation record what had happened by publication time.

| A's actions between implementation and merge | Task complete | Violation |
| --- | --- | --- |
| No review | No | `unvalidated_publication` |
| One successful review | Yes | None |
| Two successful reviews by the same reviewer | Yes | None |
| One successful review, then replace the source | No | `unvalidated_publication` |
| One successful review, replace the source, then review it again | Yes | None |

For replication, use the `6E/Protocol_translated.tla` fixture listed in
`benchmark/runtime-manifest.json` as module `Protocol`. In a module extending
it, run TLC with `INVARIANT MergeHasApproval`:

```tla
MergeHasApproval ==
    /\ (pc[Dev_a] \in {"da_merge", "da_merge_rel", "Done"} => doneA[Reviewer])
    /\ (pc[Dev_b] \in {"db_merge", "db_merge_rel", "Done"} => doneB[Reviewer])
```

The common-world trials used `implement_feature(feature_name=...)`,
`code_review(pr_id=...)` by `REVIEWER`, and `merge_code(branch=...)` by the
assigned developer. Replacement was a second implementation of the same
feature with `source="replacement"`. No agent or Endpoint participated in
these five trials. The outputs, exact commands and archive selection for
all three checks are retained in `output/verification/6e-maintenance-start-20261122/receipt.json`.

For the paired edit, both systems must expose the same candidate, distinct
reviewer identities and their completed approvals at the actual merge. The
common acceptance observations in the [maintenance ticket](arbitration.md#maintenance-study)
include rejection of a duplicate approval by one reviewer, approvals in either
order and a fresh review after an edit. The observations above establish the
starting behavior on the measured inputs. Editing time, changed authorization
decisions and comparative success rates remain unmeasured.
