//! Actual test results drive source branches and repairs.
use super::super::{global::*, input, wire};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::Endpoint;
#[hibana_runtime::scope]
pub(crate) async fn run<'run, const ROLE: u8, const FIRST: u8, const SECOND: u8>(
    endpoint: &mut Endpoint<'run, ROLE>,
    domain: &impl Domain,
    test_result: &input::TestResult,
) -> ShellResult<()> {
    let [mut primary, mut secondary, mut file] = endpoint.par()?;
    let input = input::assignment(ROLE);
    let arguments = file.recv::<DesignRequest<ROLE>>().await?.into_value();
    let designed = domain
        .execute(input.agent, "design_feature", arguments)
        .await
        .map_err(ShellError::message)?;
    wire::result(&designed, "design_feature", input.agent)?;
    if designed["success"] != true || !designed["data"]["artifact"]["id"].is_string() {
        return Err(ShellError::message(
            "design effect did not return its actual input",
        ));
    }
    let bytes = wire::encode(&designed)?;
    file.send::<DesignCompleted>(&bytes.as_slice()).await?;

    loop {
        let arguments = file.recv::<ImplementRequest<ROLE>>().await?.into_value();
        let implemented = domain
            .execute(input.agent, "implement_code", arguments)
            .await
            .map_err(ShellError::message)?;
        wire::result(&implemented, "implement_code", input.agent)?;
        if implemented["success"] != true
            || implemented["data"]["artifact"]["design"] != designed["data"]["artifact"]
        {
            return Err(ShellError::message(
                "implementation did not consume its actual design",
            ));
        }
        let bytes = wire::encode(&implemented)?;
        file.send::<ImplementCompleted>(&bytes.as_slice()).await?;

        let admitted = primary.recv::<CommitRequest<ROLE>>().await?;
        let arguments = wire::ModuleArguments::from_bytes(admitted.as_bytes())?;
        let (bytes, committed) = {
            let ((mut first, request_a), (mut second, request_b)) = async {
                async {
                    let mut first = primary.recv::<ModuleLoan<ROLE, FIRST>>().await?;
                    let request = async {
                        async {
                            first.send::<ModuleRequest<FIRST>>(&arguments).await?;
                        };
                        Ok(primary.recv::<ModuleRequest<FIRST>>().await?)
                    }
                    .await?;
                    Ok((first, request))
                };
                async {
                    let mut second = secondary.recv::<ModuleLoan<ROLE, SECOND>>().await?;
                    let request = async {
                        async {
                            second.send::<ModuleRequest<SECOND>>(&arguments).await?;
                        };
                        Ok(secondary.recv::<ModuleRequest<SECOND>>().await?)
                    }
                    .await?;
                    Ok((second, request))
                };
            }
            .await?;
            let admitted = admitted.into_value();
            if request_a != request_b || request_a.into_value() != admitted {
                return Err(ShellError::message(
                    "module requests differ from the actual file request",
                ));
            }
            let committed = domain
                .execute(input.agent, "commit_changes", admitted)
                .await
                .map_err(ShellError::message)?;
            wire::result(&committed, "commit_changes", input.agent)?;
            if committed["success"] != true
                || committed["data"]["artifact"]["source"] != implemented["data"]["artifact"]
            {
                return Err(ShellError::message(
                    "module effect did not consume its actual implementation",
                ));
            }
            let bytes = wire::encode(&committed)?;
            async {
                async {
                    async {
                        async {
                            primary
                                .send::<ModuleCompleted<FIRST>>(&bytes.as_slice())
                                .await?;
                        };
                        let received = first.recv::<ModuleCompleted<FIRST>>().await?;
                        if received[..] != bytes {
                            return Err(ShellError::message("first module result differs"));
                        }
                        Ok(())
                    }
                    .await?;
                    primary.send::<Loan<FIRST>>(first).await?;
                };
                async {
                    async {
                        async {
                            secondary
                                .send::<ModuleCompleted<SECOND>>(&bytes.as_slice())
                                .await?;
                        };
                        let received = second.recv::<ModuleCompleted<SECOND>>().await?;
                        if received[..] != bytes {
                            return Err(ShellError::message("second module result differs"));
                        }
                        Ok(())
                    }
                    .await?;
                    secondary.send::<Loan<SECOND>>(second).await?;
                };
                Ok(())
            }
            .await?;
            (bytes, committed)
        };
        file.send::<CommitCompleted>(&bytes.as_slice()).await?;

        let arguments = file.recv::<TestRequest<ROLE>>().await?.into_value();
        let tested = domain
            .execute(input.agent, "run_local_tests", arguments)
            .await
            .map_err(ShellError::message)?;
        wire::result(&tested, "run_local_tests", input.agent)?;
        if tested["data"]["input_snapshot"] != committed["data"]["artifact"] {
            return Err(ShellError::message(
                "test did not examine the actual module",
            ));
        }
        let bytes = wire::encode(&tested)?;
        test_result.record(tested.clone());
        let branch = file.offer().await?;
        drop(branch);
        file.send::<TestCompleted>(&bytes.as_slice()).await?;
        if tested["success"] == true {
            return Ok(());
        }
    }
}
