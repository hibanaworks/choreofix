use super::super::global::*;
use crate::shell::{
    ShellError, ShellResult,
    imp::service::{Domain, File, Validation},
};
use hibana::Endpoint;
use hibana_wasip1_runtime::protocol;
use serde_json::json;
#[hibana_runtime::scope]
pub async fn run<D: Domain>(
    endpoint: &mut Endpoint<TESTER>,
    domain: &D,
    validation: &Validation,
    result_input: &super::super::super::imp::TestInput,
) -> ShellResult<()> {
    endpoint.recv::<BackendReady>().await?;
    let mut file = File::new(b"service/run_integration_tests", SERVICE_FD);
    loop {
        endpoint.send::<Request>(&()).await?;
        endpoint.send::<RequestReady>(&()).await?;
        let branch = endpoint.offer().await?;
        match branch.label() {
            100 => {
                let request = branch.recv::<Open>().await?;
                let response = file.open(request);
                endpoint.send::<Opened>(&response).await?;
            }
            102 => {
                let request = branch.recv::<Write>().await?;
                let response = file.write(request);
                endpoint.send::<Written>(&response).await?;
            }
            104 => {
                let request = branch.recv::<Commit>().await?;
                validation.record(file.arguments(request, "suite").and_then(|v| {
                    if v["suite"] == "backend_schema_integration" {
                        Ok(v)
                    } else {
                        Err(ShellError::message(
                            "arguments name a different task object",
                        ))
                    }
                }));
                let decision = endpoint.offer().await?;
                match decision.label() {
                    111 => {
                        decision.recv::<Validate>().await?;
                        let arguments = serde_json::to_vec(&validation.take()?)
                            .map_err(|error| ShellError::message(error.to_string()))?;
                        endpoint.send::<Admitted>(&arguments.as_slice()).await?;
                        break;
                    }
                    106 => {
                        let bytes = serde_json::to_vec(
                            &json!({"status":"invalid_arguments","error":validation.rejection()?}),
                        )
                        .map_err(|e| ShellError::message(e.to_string()))?;
                        decision.send::<Rejected>(&bytes.as_slice()).await?;
                    }
                    _ => return Err(ShellError::message("unexpected argument decision")),
                }
            }
            _ => return Err(ShellError::message("unexpected service request")),
        }
    }
    let arguments = endpoint.recv::<TestRequest>().await?.into_value();
    let result = domain
        .execute("TESTER", "run_integration_tests", arguments)
        .await
        .map_err(ShellError::message)?;
    let bytes = serde_json::to_vec(&result).map_err(|e| ShellError::message(e.to_string()))?;
    let response = bytes.as_slice();
    endpoint.send::<TestCompleted>(&response).await?;
    result_input.record(result);
    let decision = endpoint.offer().await?;
    let label = decision.label();
    drop(decision);
    {
        let [mut admin, mut backend] = endpoint.par()?;
        let response = bytes.as_slice();
        match label {
            50 | 51 => {
                async {
                    async {
                        admin.send::<PassedAdmin>(&response).await?;
                    };
                    async {
                        backend.send::<PassedBackend>(&response).await?;
                    };
                    Ok(())
                }
                .await?;
            }
            60 | 61 => {
                async {
                    async {
                        admin.send::<FailedAdmin>(&response).await?;
                    };
                    async {
                        backend.send::<FailedBackend>(&response).await?;
                    };
                    Ok(())
                }
                .await?;
            }
            _ => return Err(ShellError::message("unexpected test outcome")),
        }
    }
    loop {
        endpoint.send::<RejectRequest>(&()).await?;
        endpoint.send::<RequestReady>(&()).await?;
        let branch = endpoint.offer().await?;
        match branch.label() {
            100 => {
                branch.recv::<Open>().await?;
                let response = protocol::PathOpenedRet(protocol::PathOpened::new(0, 8));
                endpoint.send::<Opened>(&response).await?;
            }
            102 => {
                let r = branch.recv::<Write>().await?;
                let response = protocol::FdWriteDoneRet(protocol::FdWriteDone::new_with_errno(
                    hibana_wasip1_runtime::choreofs::ChoreoFsRequest::request(&r)
                        .0
                        .fd(),
                    0,
                    8,
                ));
                endpoint.send::<Written>(&response).await?;
            }
            104 => {
                let r = branch.recv::<Commit>().await?;
                let response = protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                    hibana_wasip1_runtime::choreofs::ChoreoFsRequest::request(&r)
                        .0
                        .fd(),
                    8,
                ));
                endpoint.send::<Unavailable>(&response).await?;
            }
            241 => {
                let status = branch.recv::<ServiceExit>().await?;
                if status != 0 {
                    return Err(ShellError::message("guest failed"));
                }
                return Ok(());
            }
            _ => return Err(ShellError::message("unexpected completed service request")),
        }
    }
}
