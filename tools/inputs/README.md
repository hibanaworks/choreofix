# Benchmark input and experiment records

This Rust host tool restores the two pinned upstream source archives, checks
their recorded inputs and runtimes, and generates the full experiment matrix.
It never runs a benchmark participant or owns an Endpoint, task permission,
resource loan or task-progress state.

From the repository root:

```sh
cargo +stable run --locked --manifest-path tools/inputs/Cargo.toml -- restore
cargo +stable run --locked --manifest-path tools/inputs/Cargo.toml -- verify
cargo +stable run --locked --manifest-path tools/inputs/Cargo.toml -- matrix --check
```

`restore` fetches immutable Git revisions only when a source directory is
missing. It stages an archive outside the repository, checks its bytes against
the recorded manifests, and then installs it. It rejects links and submodules.
An existing modified source is an error and is never repaired or overwritten.
`--root DIRECTORY` before the command selects a different experiment directory.

`verify` checks every original description, environment input and simulator
source, all 48 participant/resource/tool/checklist records, and both baseline
runtime/compiler inventories. Python bytecode caches are excluded. No upstream
Python source is evaluated to perform these checks. The runner and its offline
auditor use this same Rust benchmark verifier.

Experiment source snapshots include this Rust tool and its locked dependencies.

`matrix --check` checks existing bytes without rewriting them. `matrix` writes
both derived JSONL files. The original 2,160 identities and supplementary 432
identities are retained, as are the selected model, effort, task/depth/repetition
coverage and source revisions. These records are a plan, not completed episodes.

```sh
cargo +stable test --locked --manifest-path tools/inputs/Cargo.toml
cargo +stable clippy --locked --manifest-path tools/inputs/Cargo.toml --all-targets -- -D warnings
```

The behavioral tests mutate source bytes, input metadata, task/file sets and
experiment conditions. They also test real Git archive extraction, existing
file preservation, missing revisions and rejection of a tracked symlink.
Saved source inventories retain their own declared simulator contract. Verifying
an archive recomputes every world byte, task and metadata entry against that
archive's manifest; it does not replace its version label with the current
manifest-writer version. This is input integrity, not a new semantic
qualification or permission to pool experiments with different world versions.

The same tool inspects Wasm declarations and checks the shell build without
executing a guest or importing Python:

```sh
cargo +stable run --locked --manifest-path tools/inputs/Cargo.toml -- wasm vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm
cargo +stable run --locked --manifest-path tools/inputs/Cargo.toml -- wash vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm
```

`wasm` reports the exact file digest and byte size, imports, memory/table
declarations, function counts, parameter/result limits and declared locals.
`wash` additionally requires only WASIp1 function imports and one memory with
one initial 64-KiB page and no declared maximum. The host runtime still owns
the limit on memory growth. This is declaration inspection, not instruction
validation, a Wasm engine or a measurement of maximum runtime heap use.
