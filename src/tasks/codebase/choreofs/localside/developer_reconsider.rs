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
    let input = input::assignment(ROLE);
    let arguments = endpoint.recv::<DesignRequest<ROLE>>().await?.into_value();
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
    endpoint.send::<DesignCompleted>(&bytes.as_slice()).await?;

    loop {
        let arguments = endpoint
            .recv::<ImplementRequest<ROLE>>()
            .await?
            .into_value();
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
        endpoint
            .send::<ImplementCompleted>(&bytes.as_slice())
            .await?;

        let admitted = endpoint.recv::<CommitRequest<ROLE>>().await?;
        let arguments = wire::ModuleArguments::from_bytes(admitted.as_bytes())?;
        let (mut first, mut second) = async {
            let [mut a, mut b] = endpoint.par()?;
            async { a.recv::<Loan<FIRST>>().await };
            async { b.recv::<Loan<SECOND>>().await };
        }
        .await?;
        let (request_a, request_b) = async {
            let [mut a, mut b] = endpoint.par()?;
            async {
                async {
                    first.send::<ModuleRequest<FIRST>>(&arguments).await?;
                };
                Ok(a.recv::<ModuleRequest<FIRST>>().await?)
            };
            async {
                async {
                    second.send::<ModuleRequest<SECOND>>(&arguments).await?;
                };
                Ok(b.recv::<ModuleRequest<SECOND>>().await?)
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
            let [mut a, mut b] = endpoint.par()?;
            async {
                async {
                    a.send::<ModuleCompleted<FIRST>>(&bytes.as_slice()).await?;
                };
                let received = first.recv::<ModuleCompleted<FIRST>>().await?;
                if received[..] != bytes {
                    return Err(ShellError::message("first module result differs"));
                }
                Ok(())
            };
            async {
                async {
                    b.send::<ModuleCompleted<SECOND>>(&bytes.as_slice()).await?;
                };
                let received = second.recv::<ModuleCompleted<SECOND>>().await?;
                if received[..] != bytes {
                    return Err(ShellError::message("second module result differs"));
                }
                Ok(())
            };
            Ok(())
        }
        .await?;
        async {
            let [mut a, mut b] = endpoint.par()?;
            async {
                a.send::<Loan<FIRST>>(first).await?;
            };
            async {
                b.send::<Loan<SECOND>>(second).await?;
            };
            Ok(())
        }
        .await?;
        endpoint.send::<CommitCompleted>(&bytes.as_slice()).await?;

        let arguments = endpoint.recv::<TestRequest<ROLE>>().await?.into_value();
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
        let branch = endpoint.offer().await?;
        drop(branch);
        async {
            let [mut file, mut first_owner, mut second_owner] = endpoint.par()?;
            async {
                file.send::<TestCompleted>(&bytes.as_slice()).await?;
            };
            async {
                first_owner
                    .send::<ModuleTestResult<SECOND>>(&bytes.as_slice())
                    .await?;
            };
            async {
                second_owner
                    .send::<ModuleTestResult<FIRST>>(&bytes.as_slice())
                    .await?;
            };
            Ok(())
        }
        .await?;
        if tested["success"] == true {
            return Ok(());
        }
    }
}
