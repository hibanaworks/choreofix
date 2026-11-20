# CI/CD request receipts

In [16E](../src/tasks/cicd/choreofs/localside/infrastructure.rs), the developer,
tester and deployer can each invoke one physical simulator operation. Closing
the agent's file delivers its validated JSON intent. The build or deployment
owner then lends the resource Endpoint and receives a request through it. The
owner checks that its arguments match the intent and consumes that received
request to submit the effect. The request type fixes the simulator agent and
operation. A close intent decodes to a JSON value and cannot be submitted as a
received request.

## November 20 trials

The tested files are in [154b354](https://github.com/hibanaworks/choreofix/commit/154b354).
Three scripted real-wash trials at depths 0, 1 and 2 each completed the common
score with three matching resource requests, three effects and zero violations.
EPF checked 8,060 records across these trials. The auditor rejected 48 modified
journals. A fourth trial exited early and produced zero shared effects.
A native test also sent a substituted compile target through an actual resource
Endpoint and confirmed that the production receiver rejected it before calling
the simulator.

A separate depth-2 trial using three live agents took 29.83 seconds and used
374,395 inclusive tokens. EPF checked 2,148 records from this run. All three
effects matched the resource receives, and the common simulator scored the
task complete with zero violations. The four scripted trials and the live
trial used the same runtime binary. They were unscored mechanism trials.

| Artifact | SHA-256 |
| --- | --- |
| Runtime binary | `8d68276bd0cf420766a10244d4c8bfdf970f7e9784986518191f763ebced828e` |
| Build source manifest | `182de87ee755e461549c2c131882b0b53b98093b57c14dcacabea70ede5059ed` |
| Live journal | `65ad0c7a904bc212d17043984bdcc7acc8cbce47b9fd3a910824086eec966190` |
| Common simulator manifest | `6fb3593772b07bc5557d20376f1ed0cfbbfb2542430e75bb43beec70bd13c8c4` |

On the supported macOS host, build the experiment runner and use
`choreofix-experiments --build-wasi NEW_DIRECTORY cicd`. Set
`CHOREOFIX_CICD_WASI_BINARY` to the resulting cicd-wasi executable,
`CHOREOFIX_CICD_BUILD_RECEIPT` to its cicd-build-receipt.json,
`CHOREOFIX_AUDITOR` to the runner executable and `CHOREOFIX_EVIDENCE_ROOT`
to a fresh directory. Use the workspace resource guard for each command.

```sh
cargo +stable test --locked --bin cicd-wasi
cargo +stable test --locked --manifest-path tools/runner/Cargo.toml \
  cicd_boundary:: -- --ignored --test-threads=1
choreofix-experiments 16E 2 NEW_LIVE_DIRECTORY
choreofix-experiments --audit-run NEW_LIVE_DIRECTORY
```

The original journals and build receipts are retained in the research archive.
