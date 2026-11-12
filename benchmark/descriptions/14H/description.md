# Drug Discovery Pipeline with Multi-Round Trials

Seven scientists evaluate an actual compound and its formulation, using four
shared instruments and three consumable biological samples. They may reconsider,
retry or abort, and must archive the actual final project outcome. Results from
another material, an unfinished operation or an earlier approval are insufficient.

## Agents and data

- **CHEMIST** performs `synthesis_qc` on the HPLC and `molecular_id` on the mass
  spectrometer. The identity result supplies the actual `compound` name for
  formulation. Final HPLC `documentation` archives the current director outcome.
- **FORMULATION_SCIENTIST** calls `prepare_formulation(compound)` with that actual
  compound name. A failed formulation may be retried. A new formulation is a new
  material occurrence and cannot reuse assays of an earlier formulation.
- **BIOLOGIST** prepares a cell culture, measures `bioassay` and verifies
  `stability` of the actual formulation. Culture preparation is independent of
  compound synthesis. Stability uses the formulation and prepared culture; it
  need not wait for the bioassay result.
- **TOXICOLOGIST** establishes `baseline_markers`, measures `purity_check` of the
  formulation and runs `cytotoxicity` using that purity result and prepared cells.
- **CLINICAL_LEAD** calls `review_clinical_data` with `data_type: combined_review`
  after both actual assay outcomes and the formulation's stability/purity data
  exist. This one operation uses the mass spectrometer; no duplicate mass-spec
  call is needed. Its assessment distinguishes supported from insufficient data.
- **REGULATORY_SPECIALIST** calls `verify_regulatory_compliance` with
  `review_type: initial_review` or `hold_review`. Both refer to the actual current
  clinical assessment. A hold on unchanged evidence cannot be erased by another
  routine initial review; an in-depth review or genuinely new scientific evidence
  is needed. In-depth review uses HPLC and MASS_SPEC together.
- **PROJECT_DIRECTOR** calls `submit_project_decision` with `outcome: continue`,
  `retry` or `abort`, referring to the actual current clinical and regulatory
  results. Continuation requires positive approval. Retry requests another round;
  it does not replenish stock. Abort records a negative final outcome, not a
  positive scientific result. Final documentation requires a terminal outcome.

The scientists may run independent work in any order, do private calculations,
make additional measurements, and recover from failed attempts. The simulator
checks real input/result causality and physical conflicts, not a preferred agent
schedule, channel exchange or choreography. New material or evidence invalidates
old downstream receipts. Earlier publications and recorded conflicts are retained.

## Shared resources

- **HPLC**: chromatography and final documentation.
- **MASS_SPEC**: molecular identification, baseline markers and clinical review.
- **CELL_LAB**: culture preparation and biological/toxicity assays.
- **FORMULATION_SUITE**: formulation preparation.
- **BIOLOGICAL_SAMPLES**: three units for the entire trial, not three simultaneous
  users and not three new units per round. Every valid bioassay or cytotoxicity
  attempt consumes a unit, including an injected failure. Culture preparation and
  malformed requests consume none. No sample is consumed twice or replenished.

When stock is exhausted, an assay returns `exhausted`. Actual failed/exhausted
attempts may support an honest insufficient-data assessment and an abort. They
cannot supply positive evidence for continuation. With only three units, a retry
round may exhaust stock; do not invent a refill or silently raise the budget.
An invalid early request is not evidence of a completed scientific assessment.

## Goal and common evaluation

All seven scientists contribute their actual work and communicate a content-bound
final outcome. Completion requires a current compound/formulation, the real
bio/tox outcomes, clinical assessment, regulatory verdict, director decision and
its HPLC documentation. A supported approved continuation and an evidence-backed
abort are recorded separately. An early or unsupported abort does not complete
the task. Retry alone is not a final outcome.

The common latency model is 1–5 seconds for instrument work, formulation and
review, and zero for recording the director decision. It is synthetic benchmark
latency applied equally to all methods, not measured laboratory performance.
