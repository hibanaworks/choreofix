//! The physical owner receives actual resource requests before executing effects.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::Endpoint;
use serde_json::Value;

#[hibana_runtime::scope]
pub async fn repository<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, STORE>,
    resources: (Endpoint<'r, REPO>, Endpoint<'r, TEST_ENV>),
    domain: &D,
) -> ShellResult<()> {
    let (repo, test_env) = resources;
    endpoint.send::<RepoLoan>(repo).await?;
    let mut define = endpoint.recv::<DefineApi>().await?.to_vec();
    let candidate = loop {
        let request = imp::arguments(&define, "api_name", "feature_api_spec")?;
        let api = domain
            .execute("BACKEND_DEV", "define_api", request)
            .await
            .map_err(ShellError::message)?;
        let bytes =
            serde_json::to_vec(&api).map_err(|error| ShellError::message(error.to_string()))?;
        endpoint.send::<ApiDefined>(&bytes.as_slice()).await?;

        let branch = endpoint.offer().await?;
        let (backend, frontend) = match branch.label() {
            20 => {
                let admitted = imp::arguments(
                    branch.recv::<BackendIntent>().await?,
                    "feature_name",
                    "backend_endpoint",
                )?;
                let request = imp::arguments(
                    endpoint.recv::<ImplementBackend>().await?,
                    "feature_name",
                    "backend_endpoint",
                )?;
                if request != admitted {
                    return Err(ShellError::message(
                        "backend resource arguments differ from the accepted request",
                    ));
                }
                let backend = domain
                    .execute("BACKEND_DEV", "implement_feature", request)
                    .await
                    .map_err(ShellError::message)?;
                let bytes = serde_json::to_vec(&backend)
                    .map_err(|error| ShellError::message(error.to_string()))?;
                endpoint
                    .send::<BackendImplemented>(&bytes.as_slice())
                    .await?;
                let request = imp::arguments(
                    endpoint.recv::<ImplementFrontend>().await?,
                    "feature_name",
                    "frontend_component",
                )?;
                let frontend = domain
                    .execute("FRONTEND_DEV", "implement_feature", request)
                    .await
                    .map_err(ShellError::message)?;
                let bytes = serde_json::to_vec(&frontend)
                    .map_err(|error| ShellError::message(error.to_string()))?;
                endpoint
                    .send::<FrontendImplemented>(&bytes.as_slice())
                    .await?;
                (backend, frontend)
            }
            7 => {
                let request = imp::arguments(
                    branch.recv::<ImplementFrontend>().await?,
                    "feature_name",
                    "frontend_component",
                )?;
                let frontend = domain
                    .execute("FRONTEND_DEV", "implement_feature", request)
                    .await
                    .map_err(ShellError::message)?;
                let bytes = serde_json::to_vec(&frontend)
                    .map_err(|error| ShellError::message(error.to_string()))?;
                endpoint
                    .send::<FrontendImplemented>(&bytes.as_slice())
                    .await?;
                let admitted = imp::arguments(
                    endpoint.recv::<BackendIntent>().await?,
                    "feature_name",
                    "backend_endpoint",
                )?;
                let request = imp::arguments(
                    endpoint.recv::<ImplementBackend>().await?,
                    "feature_name",
                    "backend_endpoint",
                )?;
                if request != admitted {
                    return Err(ShellError::message(
                        "backend resource arguments differ from the accepted request",
                    ));
                }
                let backend = domain
                    .execute("BACKEND_DEV", "implement_feature", request)
                    .await
                    .map_err(ShellError::message)?;
                let bytes = serde_json::to_vec(&backend)
                    .map_err(|error| ShellError::message(error.to_string()))?;
                endpoint
                    .send::<BackendImplemented>(&bytes.as_slice())
                    .await?;
                (backend, frontend)
            }
            _ => return Err(ShellError::message("unexpected implementation order")),
        };

        let branch = endpoint.offer().await?;
        match branch.label() {
            1 => define = branch.recv::<DefineApi>().await?.to_vec(),
            12 => {
                let actual: Value = serde_json::from_slice(branch.recv::<Finalize>().await?)
                    .map_err(|error| ShellError::message(error.to_string()))?;
                if actual != imp::candidate(&api, &backend, &frontend) {
                    return Err(ShellError::message(
                        "test candidate differs from the actual implementations",
                    ));
                }
                break actual;
            }
            _ => return Err(ShellError::message("unexpected API coordination operation")),
        }
    };
    let repo = endpoint.recv::<RepoLoan>().await?;
    let bytes =
        serde_json::to_vec(&candidate).map_err(|error| ShellError::message(error.to_string()))?;
    endpoint.send::<Ready>(&bytes.as_slice()).await?;
    async {
        let [mut repository, mut environment] = endpoint.par()?;
        async {
            repository.send::<RepoLoan>(repo).await?;
        };
        async {
            environment.send::<TestLoan>(test_env).await?;
        };
        Ok(())
    }
    .await?;
    // Each borrow covers its source par. The next par starts after its join.
    let (repo_request, test_request) = async {
        let [mut repository, mut environment] = endpoint.par()?;
        async { Ok::<_, ShellError>(repository.recv::<ReadForTests>().await?.to_vec()) };
        async { Ok::<_, ShellError>(environment.recv::<RunTests>().await?.to_vec()) };
    }
    .await?;
    let repo_request = imp::arguments(&repo_request, "suite", "integration_tests")?;
    let test_request = imp::arguments(&test_request, "suite", "integration_tests")?;
    if repo_request != test_request {
        return Err(ShellError::message("test resource requests differ"));
    }
    let result = domain
        .execute("TESTER", "run_tests", test_request)
        .await
        .map_err(ShellError::message)?;
    let bytes =
        serde_json::to_vec(&result).map_err(|error| ShellError::message(error.to_string()))?;
    async {
        let [mut repository, mut environment] = endpoint.par()?;
        async {
            repository
                .send::<RepositoryTested>(&bytes.as_slice())
                .await?;
        };
        async {
            environment
                .send::<EnvironmentTested>(&bytes.as_slice())
                .await?;
        };
        Ok(())
    }
    .await?;
    let (_repo, _test_env) = async {
        let [mut repository, mut environment] = endpoint.par()?;
        async { Ok::<_, ShellError>(repository.recv::<RepoLoan>().await?) };
        async { Ok::<_, ShellError>(environment.recv::<TestLoan>().await?) };
    }
    .await?;
    let [mut backend, mut frontend] = endpoint.par()?;
    async {
        backend.send::<BackendResult>(&bytes.as_slice()).await?;
    };
    async {
        frontend.send::<FrontendResult>(&bytes.as_slice()).await?;
    };
    Ok(())
}
