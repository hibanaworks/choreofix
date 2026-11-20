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
