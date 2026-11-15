//! Physical effect owners consume the exact request and retain its real result.
use super::super::global::*;
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::Endpoint;
#[hibana_runtime::scope]
pub async fn build_server<'run>(
    endpoint: &mut Endpoint<'run, BUILD_OWNER>,
    (mut resource, developer, tester): (
        Endpoint<'run, BUILD_SERVER>,
        Endpoint<'run, DEV_CLOSE>,
        Endpoint<'run, TEST_CLOSE>,
    ),
    domain: &impl Domain,
) -> ShellResult<()> {
    async {
        let [mut d, mut t] = endpoint.par()?;
        async {
            d.send::<Loan<DEV_CLOSE>>(developer).await?;
        };
        async {
            t.send::<Loan<TEST_CLOSE>>(tester).await?;
        };
        Ok(())
    }
    .await?;

    let admitted = endpoint.recv::<CompileIntent>().await?.into_value();
    endpoint.send::<Loan<BUILD_SERVER>>(resource).await?;
    let actual = endpoint.recv::<CompileRequest>().await?.into_value();
    if actual != admitted {
        return Err(ShellError::message(
            "developer request differs from the admitted file",
        ));
    }
    let result = domain
        .execute("DEVELOPER", "compile_and_test", actual)
        .await
        .map_err(ShellError::message)?;
    let bytes = serde_json::to_vec(&result).map_err(|e| ShellError::message(e.to_string()))?;
    endpoint.send::<CompileResult>(&bytes.as_slice()).await?;
    resource = endpoint.recv::<Loan<BUILD_SERVER>>().await?;
    if result["success"] != true {
        return Err(ShellError::message(format!(
            "domain operation failed: {result}"
        )));
    }

    let admitted = endpoint.recv::<TestIntent>().await?.into_value();
    endpoint.send::<Loan<BUILD_SERVER>>(resource).await?;
    let actual = endpoint.recv::<TestRequest>().await?.into_value();
    if actual != admitted {
        return Err(ShellError::message(
            "tester request differs from the admitted file",
        ));
    }
    let result = domain
        .execute("TESTER", "run_integration_tests", actual)
        .await
        .map_err(ShellError::message)?;
    let bytes = serde_json::to_vec(&result).map_err(|e| ShellError::message(e.to_string()))?;
    endpoint.send::<TestResult>(&bytes.as_slice()).await?;
    resource = endpoint.recv::<Loan<BUILD_SERVER>>().await?;
    if result["success"] != true {
        return Err(ShellError::message(format!(
            "domain operation failed: {result}"
        )));
    }
    drop(resource);
    Ok(())
}
#[hibana_runtime::scope]
pub async fn deploy_env<'run>(
    endpoint: &mut Endpoint<'run, DEPLOY_OWNER>,
    (mut resource, control): (Endpoint<'run, DEPLOY_ENV>, Endpoint<'run, DEPLOY_CLOSE>),
    domain: &impl Domain,
) -> ShellResult<()> {
    endpoint.send::<Loan<DEPLOY_CLOSE>>(control).await?;

    let admitted = endpoint.recv::<DeployIntent>().await?.into_value();
    endpoint.send::<Loan<DEPLOY_ENV>>(resource).await?;
    let actual = endpoint.recv::<DeployRequest>().await?.into_value();
    if actual != admitted {
        return Err(ShellError::message(
            "deployer request differs from the admitted file",
        ));
    }
    let result = domain
        .execute("DEPLOYER", "deploy_application", actual)
        .await
        .map_err(ShellError::message)?;
    let bytes = serde_json::to_vec(&result).map_err(|e| ShellError::message(e.to_string()))?;
    endpoint.send::<DeployResult>(&bytes.as_slice()).await?;
    resource = endpoint.recv::<Loan<DEPLOY_ENV>>().await?;
    if result["success"] != true {
        return Err(ShellError::message(format!(
            "domain operation failed: {result}"
        )));
    }
    drop(resource);
    Ok(())
}
