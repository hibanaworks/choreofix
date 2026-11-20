//! Resource owners apply only the exact request received from the operation file.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::PairedEffects};
use hibana::Endpoint;

#[hibana_runtime::scope]
pub async fn repository<'run>(
    endpoint: &mut Endpoint<'run, STORE>,
    (mut repo, mut environment, controls): (
        Endpoint<'run, REPO>,
        Endpoint<'run, TEST_ENV>,
        (
            Endpoint<'run, DEFINE_CLOSE>,
            Endpoint<'run, BACKEND_CLOSE>,
            Endpoint<'run, FRONTEND_CLOSE>,
            Endpoint<'run, AGREEMENT_CLOSE>,
            Endpoint<'run, TESTER_CLOSE>,
        ),
    ),
    effects: &impl PairedEffects,
) -> ShellResult<()> {
    let (define, backend, frontend, agreement, tester) = controls;
    async {
        let [mut d, mut b, mut f, mut a, mut t] = endpoint.par()?;
        async {
            d.send::<Loan<DEFINE_CLOSE>>(define).await?;
        };
        async {
            b.send::<Loan<BACKEND_CLOSE>>(backend).await?;
        };
        async {
            f.send::<Loan<FRONTEND_CLOSE>>(frontend).await?;
        };
        async {
            a.send::<Loan<AGREEMENT_CLOSE>>(agreement).await?;
        };
        async {
            t.send::<Loan<TESTER_CLOSE>>(tester).await?;
        };
        Ok(())
    }
    .await?;
    loop {
        let operation = endpoint.offer().await?;
        match operation.label() {
            20 => {
                let admitted = operation.recv::<DefineIntent>().await?;
                endpoint.send::<Loan<REPO>>(repo).await?;
                let actual = endpoint.recv::<DefineApi>().await?;
                if crate::imp::json::decode(actual.as_bytes())? != admitted {
                    return Err(ShellError::message(
                        "API request differs from the admitted file",
                    ));
                }
                let result = effects.apply(actual).await?;
                let bytes =
                    serde_json::to_vec(&result).map_err(|e| ShellError::message(e.to_string()))?;
                let api = result;
                endpoint.send::<ApiDefined>(&bytes.as_slice()).await?;
                repo = endpoint.recv::<Loan<REPO>>().await?;
                endpoint.send::<Contract>(&bytes.as_slice()).await?;

                let choice = endpoint.offer().await?;
                let (backend, frontend) = match choice.label() {
                    21 => {
                        let encoded = choice.recv::<BackendFirst>().await?.to_vec();
                        let admitted = imp::agreement(&encoded)?;
                        if admitted["first"] != "BACKEND_DEV" {
                            return Err(ShellError::message("agreement differs from its branch"));
                        }
                        let accepted = serde_json::json!({"success":true,"api_contract":api,"first":"BACKEND_DEV"});
                        let bytes = serde_json::to_vec(&accepted)
                            .map_err(|e| ShellError::message(e.to_string()))?;
                        endpoint
                            .send::<AgreementAccepted>(&bytes.as_slice())
                            .await?;
                        let backend = {
                            let admitted = endpoint.recv::<BackendIntent>().await?;
                            endpoint.send::<Loan<REPO>>(repo).await?;
                            let actual = endpoint.recv::<ImplementBackend>().await?;
                            if crate::imp::json::decode(actual.as_bytes())? != admitted {
                                return Err(ShellError::message(
                                    "backend request differs from the admitted file",
                                ));
                            }
                            let result = effects.apply(actual).await?;
                            let bytes = serde_json::to_vec(&result)
                                .map_err(|e| ShellError::message(e.to_string()))?;
                            endpoint
                                .send::<BackendImplemented>(&bytes.as_slice())
                                .await?;
                            repo = endpoint.recv::<Loan<REPO>>().await?;

                            result
                        };
                        let frontend = {
                            let admitted = endpoint.recv::<FrontendIntent>().await?;
                            endpoint.send::<Loan<REPO>>(repo).await?;
                            let actual = endpoint.recv::<ImplementFrontend>().await?;
                            if crate::imp::json::decode(actual.as_bytes())? != admitted {
                                return Err(ShellError::message(
                                    "frontend request differs from the admitted file",
                                ));
                            }
                            let result = effects.apply(actual).await?;
                            let bytes = serde_json::to_vec(&result)
                                .map_err(|e| ShellError::message(e.to_string()))?;
                            endpoint
                                .send::<FrontendImplemented>(&bytes.as_slice())
                                .await?;
                            repo = endpoint.recv::<Loan<REPO>>().await?;

                            result
                        };
                        (backend, frontend)
                    }
                    22 => {
                        let encoded = choice.recv::<FrontendFirst>().await?.to_vec();
                        let admitted = imp::agreement(&encoded)?;
                        if admitted["first"] != "FRONTEND_DEV" {
                            return Err(ShellError::message("agreement differs from its branch"));
                        }
                        let accepted = serde_json::json!({"success":true,"api_contract":api,"first":"FRONTEND_DEV"});
                        let bytes = serde_json::to_vec(&accepted)
                            .map_err(|e| ShellError::message(e.to_string()))?;
                        endpoint
                            .send::<AgreementAccepted>(&bytes.as_slice())
                            .await?;
                        let frontend = {
                            let admitted = endpoint.recv::<FrontendIntent>().await?;
                            endpoint.send::<Loan<REPO>>(repo).await?;
                            let actual = endpoint.recv::<ImplementFrontend>().await?;
                            if crate::imp::json::decode(actual.as_bytes())? != admitted {
                                return Err(ShellError::message(
                                    "frontend request differs from the admitted file",
                                ));
                            }
                            let result = effects.apply(actual).await?;
                            let bytes = serde_json::to_vec(&result)
                                .map_err(|e| ShellError::message(e.to_string()))?;
                            endpoint
                                .send::<FrontendImplemented>(&bytes.as_slice())
                                .await?;
                            repo = endpoint.recv::<Loan<REPO>>().await?;

                            result
                        };
                        let backend = {
                            let admitted = endpoint.recv::<BackendIntent>().await?;
                            endpoint.send::<Loan<REPO>>(repo).await?;
                            let actual = endpoint.recv::<ImplementBackend>().await?;
                            if crate::imp::json::decode(actual.as_bytes())? != admitted {
                                return Err(ShellError::message(
                                    "backend request differs from the admitted file",
                                ));
                            }
                            let result = effects.apply(actual).await?;
                            let bytes = serde_json::to_vec(&result)
                                .map_err(|e| ShellError::message(e.to_string()))?;
                            endpoint
                                .send::<BackendImplemented>(&bytes.as_slice())
                                .await?;
                            repo = endpoint.recv::<Loan<REPO>>().await?;

                            result
                        };
                        (backend, frontend)
                    }
                    label => {
                        return Err(ShellError::message(format!("unexpected agreement {label}")));
                    }
                };
                let candidate = imp::candidate(&api, &backend, &frontend);
                let bytes = serde_json::to_vec(&candidate)
                    .map_err(|e| ShellError::message(e.to_string()))?;
                endpoint.send::<Ready>(&bytes.as_slice()).await?;
            }
            26 => {
                let admitted = operation.recv::<TestIntent>().await?;
                async {
                    let [mut r, mut e] = endpoint.par()?;
                    async {
                        r.send::<Loan<REPO>>(repo).await?;
                    };
                    async {
                        e.send::<Loan<TEST_ENV>>(environment).await?;
                    };
                    Ok(())
                }
                .await?;
                let result = {
                    let [mut r, mut e] = endpoint.par()?;
                    let (repository, environment) = async {
                        async { r.recv::<ReadForTests>().await };
                        async { e.recv::<RunTests>().await };
                    }
                    .await?;
                    if crate::imp::json::decode(repository.as_bytes())? != admitted {
                        return Err(ShellError::message(
                            "test resource request differs from the admitted file",
                        ));
                    }
                    effects.apply_pair(repository, environment).await?
                };
                let bytes =
                    serde_json::to_vec(&result).map_err(|e| ShellError::message(e.to_string()))?;
                async {
                    let [mut r, mut e] = endpoint.par()?;
                    async {
                        r.send::<RepositoryTested>(&bytes.as_slice()).await?;
                    };
                    async {
                        e.send::<EnvironmentTested>(&bytes.as_slice()).await?;
                    };
                    Ok(())
                }
                .await?;
                (repo, environment) = async {
                    let [mut r, mut e] = endpoint.par()?;
                    async { r.recv::<Loan<REPO>>().await };
                    async { e.recv::<Loan<TEST_ENV>>().await };
                }
                .await?;
                async {
                    let [mut b, mut f] = endpoint.par()?;
                    async {
                        b.send::<BackendResult>(&bytes.as_slice()).await?;
                    };
                    async {
                        f.send::<FrontendResult>(&bytes.as_slice()).await?;
                    };
                    Ok(())
                }
                .await?;
            }
            242..=244 => {
                drop(operation);
                let (backend, frontend, tester) = async {
                    let [mut b, mut f, mut t] = endpoint.par()?;
                    async { b.recv::<BackendExit>().await };
                    async { f.recv::<FrontendExit>().await };
                    async { t.recv::<TesterExit>().await };
                }
                .await?;
                if [backend, frontend, tester]
                    .into_iter()
                    .any(|code| code != 0)
                {
                    return Err(ShellError::message("an agent exited with an error"));
                }
                return Ok(());
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected repository operation {label}"
                )));
            }
        }
    }
}
