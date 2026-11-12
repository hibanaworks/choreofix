# Task 4M: shared-library refactoring

Five agents collaborate on two modules that use a shared library:

- ARCHITECT creates `module_a_refactor` and `module_b_refactor` design documents.
- DEVELOPER_A refactors `module_a` and `SHARED_LIBRARY`, and publishes module A.
- DEVELOPER_B refactors `module_b` and `SHARED_LIBRARY`, and publishes module B.
- REVIEWER reviews each module's actual code snapshot.
- TESTER runs `module_a_tests` and `module_b_tests` against actual code snapshots.

The environment owns in-memory Python source files. `refactor_code` applies a
behavior-preserving source transformation, `merge_code` publishes the current
module/library bytes, and review/test tools execute those actual sources and
return content-bound results. These are coordination tasks, not an evaluation of
arbitrary model-generated programs.

Choose the coordination procedure freely. All mechanical operations can be
invoked independently of earlier designs, reviews, tests or merges. You may
repeat operations, recover, batch work, and run independent work concurrently.
A working copy may contain later experiments without changing a good publication.

The goal is to publish both refactored modules with correct results, create both
design documents, perform each agent's assigned work, and have completed
successful reviews and tests for the exact module/library contents before
publishing them. A later review or test cannot retroactively authorize an earlier
publication. Independent review/test orders are free; there is no single prescribed
sequence. An unsafe publication is recorded even if a later publication repairs
the final contents. The environment does not read a choreography,
framework identity or preferred implementation to decide completion.

REPO has capacity one for modifications/publication; TEST_ENV has capacity one
for tests. Physical concurrent resource use is observed and reported separately.
The simulator does not silently serialize competing operations.

This is the common `common-environment-v1` refinement of the original 4M task.
All methods use the same definition. Results from the original metadata-only
simulator remain separate and are not relabeled as results of this environment.
