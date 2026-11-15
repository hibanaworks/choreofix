# Task 4E: Frontend-Backend API Coordination

A **FRONTEND_DEV** developer and a **BACKEND_DEV** developer are building a feature that requires them to agree on an API contract. A **TESTER** validates the integrated feature once both sides are ready.

## Agents

- **FRONTEND_DEV**: implements the user-facing component that calls the **BACKEND_DEV** API
- **BACKEND_DEV**: implements the API endpoint that serves data to the **FRONTEND_DEV**
- **TESTER**: runs integration tests once both **FRONTEND_DEV** and **BACKEND_DEV** are ready

## Shared Resources

- **REPO**: shared code repository containing the API specification; only one agent can write to it at a time
- **TEST_ENV**: test environment for running integration tests; only one agent can use it at a time

## Workflow

**BACKEND_DEV** defines the API specification in **REPO**, then both **FRONTEND_DEV** and **BACKEND_DEV** implement their respective components. Neither developer begins implementation until the API contract is agreed upon. When both sides are ready, they notify **TESTER**, who runs integration tests in **TEST_ENV** and reports results back to both developers. If either developer needs to change the API spec after implementation has started, the other developer must be notified and may need to adapt their code.

## Goal

All three agents complete their work (API definition, implementation, testing) without conflicts over shared resources.

## Observable Results

The common simulator materializes an API document and two small source modules
in the shared repository. The provided API maps an integer `value` to
`value * 2 + 1`; the frontend renders the backend result as `feature=RESULT`.
Each implementation records the completed API document it used. Integration
tests execute both modules on negative, zero and positive inputs and record
the exact repository contents tested.

Tools remain callable in any order. A premature implementation still changes
the repository, and a premature test reports its actual failure. Later work
does not turn that earlier test into a passing result. Both implementation
orders are valid; no choreography, Endpoint state, fixed command sequence or
method-specific record is consulted by the scorer. Completion requires a
finished passing integration test of the current repository. Repeating work
that produces identical bytes does not invalidate that evidence. Physical
resource conflicts are reported separately from functional completion.
