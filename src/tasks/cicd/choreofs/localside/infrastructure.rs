//! Physical effect owners consume the exact request and retain its real result.
use super::super::global::*;
use crate::shell::{ShellError, ShellResult, imp::service::ReceivedEffects};
use hibana::Endpoint;
#[hibana_runtime::scope]
pub async fn build_server<'run>(
    endpoint: &mut Endpoint<'run, BUILD_OWNER>,
    (mut resource, developer, tester): (
        Endpoint<'run, BUILD_SERVER>,
        Endpoint<'run, DEV_CLOSE>,
        Endpoint<'run, TEST_CLOSE>,
    ),
    effects: &impl ReceivedEffects,
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

    let admitted = endpoint.recv::<CompileIntent>().await?;
    endpoint.send::<Loan<BUILD_SERVER>>(resource).await?;
    let actual = endpoint.recv::<CompileRequest>().await?;
    if serde_json::from_slice::<serde_json::Value>(actual.as_bytes())
        .expect("validated CI/CD request")
        != admitted
    {
        return Err(ShellError::message(
            "developer request differs from the admitted file",
        ));
    }
    let result = effects.apply(actual).await?;
    let bytes = serde_json::to_vec(&result).map_err(|e| ShellError::message(e.to_string()))?;
    endpoint.send::<CompileResult>(&bytes.as_slice()).await?;
    resource = endpoint.recv::<Loan<BUILD_SERVER>>().await?;
    if result["success"] != true {
        return Err(ShellError::message(format!(
            "domain operation failed: {result}"
        )));
    }

    let admitted = endpoint.recv::<TestIntent>().await?;
    endpoint.send::<Loan<BUILD_SERVER>>(resource).await?;
    let actual = endpoint.recv::<TestRequest>().await?;
    if serde_json::from_slice::<serde_json::Value>(actual.as_bytes())
        .expect("validated CI/CD request")
        != admitted
    {
        return Err(ShellError::message(
            "tester request differs from the admitted file",
        ));
    }
    let result = effects.apply(actual).await?;
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
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    endpoint.send::<Loan<DEPLOY_CLOSE>>(control).await?;

    let admitted = endpoint.recv::<DeployIntent>().await?;
    endpoint.send::<Loan<DEPLOY_ENV>>(resource).await?;
    let actual = endpoint.recv::<DeployRequest>().await?;
    if serde_json::from_slice::<serde_json::Value>(actual.as_bytes())
        .expect("validated CI/CD request")
        != admitted
    {
        return Err(ShellError::message(
            "deployer request differs from the admitted file",
        ));
    }
    let result = effects.apply(actual).await?;
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
