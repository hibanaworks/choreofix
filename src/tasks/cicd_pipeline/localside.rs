//! Every effect request uses its actual infrastructure Endpoint.
use super::{arguments::Arguments, global::*};
use crate::{
    imp::json::encode,
    shell::{ShellError, ShellResult, imp::service::Domain},
};
use hibana::Endpoint;

use crate::shell::{global::Loan, imp::review_reply::ReviewReply};
use hibana::g::Message;

#[hibana_runtime::scope]
pub async fn developer<const DEVELOPER: u8, const COMPONENT: u8>(
    endpoint: &mut Endpoint<DEVELOPER>,
    domain: &impl Domain,
) -> ShellResult<()> {
    let agent = Arguments::<DEVELOPER>::AGENT;
    loop {
        let next = endpoint.offer().await?;
        let mut workspace = match next.label() {
            WorkspaceLoan::LOGICAL_LABEL => next.recv::<WorkspaceLoan>().await?,
            label if label == BuildVerified::LOGICAL_LABEL => {
                let published = next.recv::<BuildVerified>().await?.to_vec();
                endpoint
                    .send::<BuildVerified>(&ReviewReply::from_bytes(&published)?)
                    .await?;
                continue;
            }
            label if label == BuildEnded::<DEVELOPER>::LOGICAL_LABEL => {
                let verdict = next.recv::<BuildEnded<DEVELOPER>>().await?.to_vec();
                endpoint
                    .send::<BuildEnded<DEVELOPER>>(&verdict.as_slice())
                    .await?;
                return Ok(());
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected developer input {label}"
                )));
            }
        };

        workspace.send::<WorkspaceAvailable>(&()).await?;
        let arguments = workspace.recv::<Compile<COMPONENT>>().await?.into_value();
        let compiled = domain
            .execute(agent, "compile_code", arguments)
            .await
            .map_err(ShellError::message)?;
        let bytes = encode(&compiled)?;
        if compiled["success"] != true {
            workspace
                .send::<RegistryRejected>(&ReviewReply::from_bytes(&bytes)?)
                .await?;
            workspace
                .send::<BuildReport<DEVELOPER>>(&bytes.as_slice())
                .await?;
            workspace.send::<Compiled>(&bytes.as_slice()).await?;
            endpoint.send::<WorkspaceLoan>(workspace).await?;
            continue;
        }
        workspace.send::<Compiled>(&bytes.as_slice()).await?;

        let arguments = workspace.recv::<UnitTest<COMPONENT>>().await?.into_value();
        let tested = domain
            .execute(agent, "run_unit_tests", arguments)
            .await
            .map_err(ShellError::message)?;
        let bytes = encode(&tested)?;
        if tested["success"] != true {
            workspace
                .send::<RegistryRejected>(&ReviewReply::from_bytes(&bytes)?)
                .await?;
            workspace
                .send::<BuildReport<DEVELOPER>>(&bytes.as_slice())
                .await?;
            workspace.send::<UnitTested>(&bytes.as_slice()).await?;
            endpoint.send::<WorkspaceLoan>(workspace).await?;
            continue;
        }
        workspace.send::<UnitTested>(&bytes.as_slice()).await?;
        workspace
            .send::<RegistryAccepted<DEVELOPER>>(&ReviewReply::from_bytes(&bytes)?)
            .await?;

        let admitted = endpoint.recv::<Publish<DEVELOPER>>().await?;
        endpoint
            .send::<RegistryIntent<DEVELOPER>>(&admitted)
            .await?;
        let mut registry = endpoint.recv::<RegistryLoan>().await?;
        async {
            async { workspace.send::<ReadBuild<DEVELOPER>>(&admitted).await };
            async {
                registry
                    .send::<RegisterArtifact<DEVELOPER>>(&admitted)
                    .await
            };
            Ok::<(), ShellError>(())
        }
        .await?;
        let actual = endpoint.recv::<Published>().await?.to_vec();
        endpoint.send::<RegistryLoan>(registry).await?;
        endpoint.send::<Published>(&actual.as_slice()).await?;
        workspace
            .send::<BuildReport<DEVELOPER>>(&actual.as_slice())
            .await?;
        endpoint.send::<WorkspaceLoan>(workspace).await?;
    }
}

#[hibana_runtime::scope]
pub async fn registry_owner<'run>(
    endpoint: &mut Endpoint<'run, REGISTRY_OWNER>,
    mut registry: Endpoint<'run, ARTIFACT_STORE>,
    domain: &impl Domain,
) -> ShellResult<()> {
    loop {
        let next = endpoint.offer().await?;
        let (agent, component, tested, admitted, workspace, artifact) = match next.label() {
            RegistryAccepted::<FRONTEND_DEV>::LOGICAL_LABEL => {
                let tested =
                    crate::imp::json::decode(next.recv::<RegistryAccepted<FRONTEND_DEV>>().await?)?;
                let admitted = endpoint
                    .recv::<RegistryIntent<FRONTEND_DEV>>()
                    .await?
                    .into_value();
                endpoint.send::<RegistryLoan>(registry).await?;
                let (workspace, artifact) = async {
                    let [mut workspace, mut artifact] = endpoint.par()?;
                    async { workspace.recv::<ReadBuild<FRONTEND_DEV>>().await };
                    async { artifact.recv::<RegisterArtifact<FRONTEND_DEV>>().await };
                }
                .await?;
                (
                    "FRONTEND_DEV",
                    "frontend",
                    tested,
                    admitted,
                    workspace.into_value(),
                    artifact.into_value(),
                )
            }
            RegistryAccepted::<BACKEND_DEV>::LOGICAL_LABEL => {
                let tested =
                    crate::imp::json::decode(next.recv::<RegistryAccepted<BACKEND_DEV>>().await?)?;
                let admitted = endpoint
                    .recv::<RegistryIntent<BACKEND_DEV>>()
                    .await?
                    .into_value();
                endpoint.send::<RegistryLoan>(registry).await?;
                let (workspace, artifact) = async {
                    let [mut workspace, mut artifact] = endpoint.par()?;
                    async { workspace.recv::<ReadBuild<BACKEND_DEV>>().await };
                    async { artifact.recv::<RegisterArtifact<BACKEND_DEV>>().await };
                }
                .await?;
                (
                    "BACKEND_DEV",
                    "backend",
                    tested,
                    admitted,
                    workspace.into_value(),
                    artifact.into_value(),
                )
            }
            RegistryRejected::LOGICAL_LABEL => {
                next.recv::<RegistryRejected>().await?;
                continue;
            }
            FrontendVerified::LOGICAL_LABEL => {
                let published = next.recv::<FrontendVerified>().await?.to_vec();
                endpoint
                    .send::<BuildVerified>(&ReviewReply::from_bytes(&published)?)
                    .await?;
                continue;
            }
            BackendVerified::LOGICAL_LABEL => {
                let published = next.recv::<BackendVerified>().await?.to_vec();
                endpoint
                    .send::<BuildVerified>(&ReviewReply::from_bytes(&published)?)
                    .await?;
                continue;
            }
            QaSnapshot::LOGICAL_LABEL => {
                next.recv::<QaSnapshot>().await?;
                let arguments = registry.recv::<PullForQa>().await?.into_value();
                let result = domain
                    .execute("QA_ENGINEER", "pull_artifacts", arguments)
                    .await
                    .map_err(ShellError::message)?;
                registry
                    .send::<QaPulled>(&ReviewReply::from_bytes(&encode(&result)?)?)
                    .await?;
                continue;
            }
            ReleaseApproved::LOGICAL_LABEL => {
                next.recv::<ReleaseApproved>().await?;
                let arguments = registry.recv::<PullForRelease>().await?.into_value();
                let result = domain
                    .execute("RELEASE_ENGINEER", "pull_artifacts", arguments)
                    .await
                    .map_err(ShellError::message)?;
                registry
                    .send::<ReleasePulled>(&ReviewReply::from_bytes(&encode(&result)?)?)
                    .await?;
                return Ok(());
            }
            ReleaseDenied::LOGICAL_LABEL => {
                next.recv::<ReleaseDenied>().await?;
                return Ok(());
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected registry request {label}"
                )));
            }
        };
        if tested["data"]["component"] != component
            || tested["data"]["developer"] != agent
            || workspace != admitted
            || artifact != admitted
        {
            return Err(ShellError::message(
                "publish requirements differ from the admitted artifact",
            ));
        }
        let result = domain
            .execute(agent, "publish_artifact", admitted)
            .await
            .map_err(ShellError::message)?;
        let bytes = encode(&result)?;
        endpoint.send::<Published>(&bytes.as_slice()).await?;
        registry = endpoint.recv::<RegistryLoan>().await?;
    }
}

pub async fn qa<'run>(
    endpoint: &mut Endpoint<'run, QA>,
    mut test: Endpoint<'run, TEST_ENV>,
    domain: &impl Domain,
) -> ShellResult<()> {
    loop {
        let workspace = endpoint.recv::<WorkspaceLoan>().await?;
        let first = test.offer().await?;
        let (integration, e2e) = match first.label() {
            Integration::LOGICAL_LABEL => {
                let arguments = first.recv::<Integration>().await?.into_value();
                let integration = domain
                    .execute("QA_ENGINEER", "run_integration_tests", arguments)
                    .await
                    .map_err(ShellError::message)?;
                test.send::<Integrated>(&encode(&integration)?.as_slice())
                    .await?;

                let arguments = test.recv::<E2e>().await?.into_value();
                let e2e = domain
                    .execute("QA_ENGINEER", "run_e2e_tests", arguments)
                    .await
                    .map_err(ShellError::message)?;
                test.send::<E2eTested>(&encode(&e2e)?.as_slice()).await?;
                (integration, e2e)
            }
            E2e::LOGICAL_LABEL => {
                let arguments = first.recv::<E2e>().await?.into_value();
                let e2e = domain
                    .execute("QA_ENGINEER", "run_e2e_tests", arguments)
                    .await
                    .map_err(ShellError::message)?;
                test.send::<E2eTested>(&encode(&e2e)?.as_slice()).await?;

                let arguments = test.recv::<Integration>().await?.into_value();
                let integration = domain
                    .execute("QA_ENGINEER", "run_integration_tests", arguments)
                    .await
                    .map_err(ShellError::message)?;
                test.send::<Integrated>(&encode(&integration)?.as_slice())
                    .await?;
                (integration, e2e)
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected first QA input {label}"
                )));
            }
        };
        let passed = integration["success"] == true && e2e["success"] == true;
        let report =
            encode(&serde_json::json!({"success":passed,"integration":integration,"e2e":e2e}))?;
        if passed {
            endpoint
                .send::<QaPassed>(&ReviewReply::from_bytes(&report)?)
                .await?;
            endpoint.recv::<ReleaseApproved>().await?;
            endpoint.send::<Loan<TEST_ENV>>(test).await?;
            drop(endpoint.recv::<Loan<TEST_ENV>>().await?);
            return Ok(());
        }
        endpoint
            .send::<QaFailed>(&ReviewReply::from_bytes(&report)?)
            .await?;
        let decision = endpoint.offer().await?;
        match decision.label() {
            ReconsiderQa::LOGICAL_LABEL => {
                decision.recv::<ReconsiderQa>().await?;
                endpoint.send::<WorkspaceLoan>(workspace).await?;
            }
            ReleaseDenied::LOGICAL_LABEL => {
                decision.recv::<ReleaseDenied>().await?;
                return Ok(());
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected QA continuation {label}"
                )));
            }
        }
    }
}

pub async fn release<'run>(
    endpoint: &mut Endpoint<'run, RELEASE>,
    domain: &impl Domain,
) -> ShellResult<()> {
    let permission = endpoint.offer().await?;
    match permission.label() {
        ReleaseApproved::LOGICAL_LABEL => {
            permission.recv::<ReleaseApproved>().await?;
        }
        ReleaseDenied::LOGICAL_LABEL => {
            let rejected = permission.recv::<ReleaseDenied>().await?.to_vec();
            endpoint
                .send::<StagingAborted>(&ReviewReply::from_bytes(&rejected)?)
                .await?;
            return Ok(());
        }
        label => {
            return Err(ShellError::message(format!(
                "unexpected release permission {label}"
            )));
        }
    }
    let mut test = endpoint.recv::<Loan<TEST_ENV>>().await?;
    let arguments = test.recv::<Stage>().await?.into_value();
    let deployed = domain
        .execute("RELEASE_ENGINEER", "deploy_staging", arguments)
        .await
        .map_err(ShellError::message)?;
    let bytes = encode(&deployed)?;
    test.send::<Staged>(&bytes.as_slice()).await?;

    let (staging, staging_bytes) = if deployed["success"] == true {
        endpoint
            .send::<StagingReady>(&ReviewReply::from_bytes(&bytes)?)
            .await?;
        let arguments = test.recv::<SmokeStaging>().await?.into_value();
        let checked = domain
            .execute("RELEASE_ENGINEER", "run_smoke_tests", arguments)
            .await
            .map_err(ShellError::message)?;
        let bytes = encode(&checked)?;
        test.send::<StagingResult>(&bytes.as_slice()).await?;
        (checked, bytes)
    } else {
        (deployed, bytes)
    };
    endpoint.send::<Loan<TEST_ENV>>(test).await?;
    if staging["success"] == false {
        let rejected = ReviewReply::from_bytes(&staging_bytes)?;
        endpoint.send::<StagingAborted>(&rejected).await?;
        endpoint.send::<StagingAborted>(&rejected).await?;
        return Ok(());
    }
    endpoint
        .send::<StagingSmoke>(&ReviewReply::from_bytes(&staging_bytes)?)
        .await?;
    let approval = endpoint.recv::<Promote>().await?.to_vec();
    endpoint
        .send::<Promote>(&ReviewReply::from_bytes(&approval)?)
        .await?;

    let mut production = endpoint.recv::<Loan<PROD_ENV>>().await?;
    let arguments = production.recv::<Deploy>().await?.into_value();
    let result = domain
        .execute("RELEASE_ENGINEER", "deploy_production", arguments)
        .await
        .map_err(ShellError::message)?;
    production
        .send::<Deployed>(&ReviewReply::from_bytes(&encode(&result)?)?)
        .await?;

    let arguments = production.recv::<SmokeProduction>().await?.into_value();
    let result = domain
        .execute("RELEASE_ENGINEER", "run_smoke_tests", arguments)
        .await
        .map_err(ShellError::message)?;
    let production_result = encode(&result)?;
    production
        .send::<ProductionSmoke>(&production_result.as_slice())
        .await?;
    endpoint.send::<Loan<PROD_ENV>>(production).await?;
    endpoint
        .send::<PipelineResult>(&production_result.as_slice())
        .await?;
    Ok(())
}

#[hibana_runtime::scope]
pub async fn master<'run>(
    endpoint: &mut Endpoint<'run, BUILD_MASTER>,
    workspace: Endpoint<'run, BUILD_SERVER>,
) -> ShellResult<()> {
    async {
        let [mut frontend, mut backend] = endpoint.par()?;
        async {
            frontend.send::<BeginFrontend>(&()).await?;
        };
        async {
            backend.send::<BeginBackend>(&()).await?;
        };
        Ok::<(), ShellError>(())
    }
    .await?;

    let first = endpoint.offer().await?;
    let (mut workspace, mut frontend, mut backend) = match first.label() {
        ChooseFrontend::LOGICAL_LABEL => {
            first.recv::<ChooseFrontend>().await?;
            endpoint.send::<WorkspaceLoan>(workspace).await?;
            let frontend = endpoint.recv::<BuildReport<FRONTEND_DEV>>().await?.to_vec();
            let workspace = endpoint.recv::<WorkspaceLoan>().await?;
            endpoint.send::<WorkspaceLoan>(workspace).await?;
            let backend = endpoint.recv::<BuildReport<BACKEND_DEV>>().await?.to_vec();
            (endpoint.recv::<WorkspaceLoan>().await?, frontend, backend)
        }
        ChooseBackend::LOGICAL_LABEL => {
            first.recv::<ChooseBackend>().await?;
            endpoint.send::<WorkspaceLoan>(workspace).await?;
            let backend = endpoint.recv::<BuildReport<BACKEND_DEV>>().await?.to_vec();
            let workspace = endpoint.recv::<WorkspaceLoan>().await?;
            endpoint.send::<WorkspaceLoan>(workspace).await?;
            let frontend = endpoint.recv::<BuildReport<FRONTEND_DEV>>().await?.to_vec();
            (endpoint.recv::<WorkspaceLoan>().await?, frontend, backend)
        }
        label => {
            return Err(ShellError::message(format!(
                "unexpected build request {label}"
            )));
        }
    };

    while crate::imp::json::decode(&frontend)?["success"] == false {
        endpoint
            .send::<Revise<FRONTEND_DEV>>(&ReviewReply::from_bytes(&frontend)?)
            .await?;
        endpoint.send::<WorkspaceLoan>(workspace).await?;
        frontend = endpoint.recv::<BuildReport<FRONTEND_DEV>>().await?.to_vec();
        workspace = endpoint.recv::<WorkspaceLoan>().await?;
    }
    endpoint
        .send::<FrontendVerified>(&ReviewReply::from_bytes(&frontend)?)
        .await?;
    while crate::imp::json::decode(&backend)?["success"] == false {
        endpoint
            .send::<Revise<BACKEND_DEV>>(&ReviewReply::from_bytes(&backend)?)
            .await?;
        endpoint.send::<WorkspaceLoan>(workspace).await?;
        backend = endpoint.recv::<BuildReport<BACKEND_DEV>>().await?.to_vec();
        workspace = endpoint.recv::<WorkspaceLoan>().await?;
    }
    endpoint
        .send::<BackendVerified>(&ReviewReply::from_bytes(&backend)?)
        .await?;
    endpoint.send::<WorkspaceLoan>(workspace).await?;
    endpoint.send::<QaSnapshot>(&()).await?;
    endpoint.send::<ReadyForQa>(&()).await?;
    let verdict = endpoint.offer().await?;
    let report = match verdict.label() {
        QaPassed::LOGICAL_LABEL => verdict.recv::<QaPassed>().await?.to_vec(),
        QaFailed::LOGICAL_LABEL => {
            let failure = verdict.recv::<QaFailed>().await?.to_vec();
            endpoint.send::<QaReport>(&failure.as_slice()).await?;
            let repair = endpoint.recv::<RepairDecision>().await?.into_value();
            endpoint.send::<ReconsiderQa>(&()).await?;
            let mut workspace = endpoint.recv::<WorkspaceLoan>().await?;
            let component = repair["component"]
                .as_str()
                .expect("validated repair decision");

            if matches!(component, "frontend" | "both") {
                let mut cause = failure.clone();
                loop {
                    endpoint
                        .send::<QaReviseFrontend>(&ReviewReply::from_bytes(&cause)?)
                        .await?;
                    endpoint.send::<WorkspaceLoan>(workspace).await?;
                    frontend = endpoint.recv::<BuildReport<FRONTEND_DEV>>().await?.to_vec();
                    workspace = endpoint.recv::<WorkspaceLoan>().await?;
                    if crate::imp::json::decode(&frontend)?["success"] == true {
                        break;
                    }
                    cause = std::mem::take(&mut frontend);
                }
            }
            endpoint
                .send::<FrontendVerified>(&ReviewReply::from_bytes(&frontend)?)
                .await?;
            if matches!(component, "backend" | "both") {
                let mut cause = failure;
                loop {
                    endpoint
                        .send::<QaReviseBackend>(&ReviewReply::from_bytes(&cause)?)
                        .await?;
                    endpoint.send::<WorkspaceLoan>(workspace).await?;
                    backend = endpoint.recv::<BuildReport<BACKEND_DEV>>().await?.to_vec();
                    workspace = endpoint.recv::<WorkspaceLoan>().await?;
                    if crate::imp::json::decode(&backend)?["success"] == true {
                        break;
                    }
                    cause = std::mem::take(&mut backend);
                }
            }
            endpoint
                .send::<BackendVerified>(&ReviewReply::from_bytes(&backend)?)
                .await?;

            endpoint.send::<WorkspaceLoan>(workspace).await?;
            endpoint.send::<QaSnapshot>(&()).await?;
            endpoint.send::<ReadyForQa>(&()).await?;
            let verdict = endpoint.offer().await?;
            match verdict.label() {
                QaPassed::LOGICAL_LABEL => verdict.recv::<QaPassed>().await?.to_vec(),
                QaFailed::LOGICAL_LABEL => verdict.recv::<QaFailed>().await?.to_vec(),
                label => {
                    return Err(ShellError::message(format!(
                        "unexpected second QA verdict {label}"
                    )));
                }
            }
        }
        label => {
            return Err(ShellError::message(format!(
                "unexpected QA verdict {label}"
            )));
        }
    };
    let bytes = report.as_slice();
    endpoint.send::<QaReport>(&bytes).await?;
    async {
        let [mut front, mut back] = endpoint.par()?;
        async {
            front.send::<BuildEnded<FRONTEND_DEV>>(&bytes).await?;
        };
        async {
            back.send::<BuildEnded<BACKEND_DEV>>(&bytes).await?;
        };
        Ok::<(), ShellError>(())
    }
    .await?;
    let result = if crate::imp::json::decode(&report)?["success"] == true {
        let approval = ReviewReply::from_bytes(&report)?;
        endpoint.send::<ReleaseApproved>(&approval).await?;
        endpoint.send::<ReleaseApproved>(&approval).await?;
        endpoint.send::<ReleaseApproved>(&approval).await?;
        endpoint.send::<ReadyForRelease>(&bytes).await?;
        let decision = endpoint.offer().await?;
        match decision.label() {
            StagingSmoke::LOGICAL_LABEL => {
                let checked = decision.recv::<StagingSmoke>().await?.to_vec();
                let promotion = ReviewReply::from_bytes(&checked)?;
                endpoint.send::<Promote>(&promotion).await?;
                endpoint.send::<Promote>(&promotion).await?;
                endpoint.recv::<PipelineResult>().await?.to_vec()
            }
            StagingAborted::LOGICAL_LABEL => {
                let failure = decision.recv::<StagingAborted>().await?.to_vec();
                endpoint
                    .send::<StagingAborted>(&ReviewReply::from_bytes(&failure)?)
                    .await?;
                failure
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected release verdict {label}"
                )));
            }
        }
    } else {
        let denial = ReviewReply::from_bytes(&report)?;
        endpoint.send::<ReleaseDenied>(&denial).await?;
        endpoint.send::<ReleaseDenied>(&denial).await?;
        endpoint.send::<ReleaseDenied>(&denial).await?;
        endpoint.send::<ReadyForRelease>(&bytes).await?;
        report
    };
    let bytes = result.as_slice();
    endpoint.send::<QaFinished>(&bytes).await?;
    let [mut frontend, mut backend, mut release, mut master] = endpoint.par()?;
    async {
        frontend.send::<FrontendFinished>(&bytes).await?;
    };
    async {
        backend.send::<BackendFinished>(&bytes).await?;
    };
    async {
        release.send::<ReleaseFinished>(&bytes).await?;
    };
    async {
        master.send::<FinalResult>(&bytes).await?;
    };
    Ok(())
}
