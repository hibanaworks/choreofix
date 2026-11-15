# Simple CI/CD Pipeline

Three engineers coordinate to build, test, and deploy a single-service application through a linear pipeline. They share two pieces of infrastructure: a build server and a deployment environment.

## Agents
- **DEVELOPER**: compiles code and runs unit tests on the build server
- **TESTER**: runs integration tests using the build server for test execution
- **DEPLOYER**: deploys the tested application to the deployment environment

## Shared Resources
- **BUILD_SERVER**: continuous integration build server for compilation and testing
- **DEPLOY_ENV**: deployment environment for the live application

## Communication Channels
- **DEVELOPER → TESTER**: build-complete notification
- **DEPLOYER → DEVELOPER**: deployment-complete notification
- **DEPLOYER → TESTER**: deployment-complete notification

## Workflow

### DEVELOPER
The **DEVELOPER** compiles the application code and runs unit tests on the **BUILD_SERVER**. After unit tests pass, the **DEVELOPER** sends a build-complete notification to the **TESTER**.

The **DEVELOPER** then waits for the deployment-complete notification from the **DEPLOYER** and is done.

### TESTER
The **TESTER** receives the build-complete notification from the **DEVELOPER**. The **TESTER** then uses the **BUILD_SERVER** to run integration tests against the compiled artifacts. After the integration tests pass, the **TESTER** sends a tests-passed notification to the **DEPLOYER**.

The **TESTER** then waits for the deployment-complete notification from the **DEPLOYER** and is done.

### DEPLOYER
The **DEPLOYER** receives the tests-passed notification from the **TESTER**. The **DEPLOYER** uses the **DEPLOY_ENV** to deploy the application and runs a smoke test. After the smoke test passes, the **DEPLOYER** sends a deployment-complete notification to the **DEVELOPER** and a deployment-complete notification to the **TESTER**. The **DEPLOYER** is done.

## Goal
All three engineers complete their work: the **DEVELOPER** has built and tested the code, the **TESTER** has validated with integration tests, and the **DEPLOYER** has deployed the application and notified everyone.

## Observable Results

The common environment compiles the application's source into an actual code
artifact, executes unit and integration checks, and copies the observed artifact
to production. Each test and deployment records the source and artifact hashes.
Completion requires a finished deployment of the current artifact with matching,
completed integration evidence already available when deployment began.

All tools remain callable in any order. Deploying before integration testing
still changes production when an artifact exists, but a later passing test does
not validate that earlier deployment. Identical rebuilds preserve equal-content
evidence. The scorer reads actual artifacts and completed observations, never
Hibana state, a choreography or one method's command sequence. Physical resource
conflicts are reported independently of functional completion.
An actual deployment without matching completed integration evidence remains
an unvalidated-publication violation even after a later correct deployment.
