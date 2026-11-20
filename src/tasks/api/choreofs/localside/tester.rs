use super::super::global::*;
use crate::shell::{ShellError, ShellResult, imp::service::ReceivedEffects};
use hibana::Endpoint;

#[hibana_runtime::scope]
pub(crate) async fn run<'run>(
    control: &mut Endpoint<'run, TESTER_CLOSE>,
    (admin_read, backend_read): (
        Endpoint<'run, ADMIN_OUTCOME_READ>,
        Endpoint<'run, BACKEND_OUTCOME_READ>,
    ),
    (effects, result_input): (&impl ReceivedEffects, &super::super::super::imp::TestInput),
) -> ShellResult<()> {
    control.recv::<BackendReady>().await?;
    let admitted = control.recv::<TestIntent>().await?;
    control.send::<TestIntent>(&admitted).await?;
    let request = control.recv::<TestRequest>().await?;
    if request.as_bytes() != admitted.0 {
        return Err(ShellError::message(
            "resource request differs from admitted test",
        ));
    }
    let effect = effects.apply(request);
    let mut environment = control.recv::<Loan<TEST_RESOURCE>>().await?;
    let result = effect.await?;
    let bytes = serde_json::to_vec(&result).map_err(|e| ShellError::message(e.to_string()))?;
    environment.send::<TestCompleted>(&bytes.as_slice()).await?;
    control.send::<Loan<TEST_RESOURCE>>(environment).await?;
    result_input.record(result);
    let decision = control.offer().await?;
    let label = decision.label();
    drop(decision);
    {
        let [mut admin, mut backend] = control.par()?;
        match label {
            50 | 51 => {
                async {
                    async {
                        admin.send::<PassedAdmin>(&bytes.as_slice()).await?;
                        admin.send::<Loan<ADMIN_OUTCOME_READ>>(admin_read).await?;
                    };
                    async {
                        backend.send::<PassedBackend>(&bytes.as_slice()).await?;
                        backend
                            .send::<Loan<BACKEND_OUTCOME_READ>>(backend_read)
                            .await?;
                    };
                    Ok(())
                }
                .await?;
            }
            60 | 61 => {
                async {
                    async {
                        admin.send::<FailedAdmin>(&bytes.as_slice()).await?;
                        admin.send::<Loan<ADMIN_OUTCOME_READ>>(admin_read).await?;
                    };
                    async {
                        backend.send::<FailedBackend>(&bytes.as_slice()).await?;
                        backend
                            .send::<Loan<BACKEND_OUTCOME_READ>>(backend_read)
                            .await?;
                    };
                    Ok(())
                }
                .await?;
            }
            _ => return Err(ShellError::message("unexpected test-result branch")),
        }
    }
    control.send::<AppliedResult>(&bytes.as_slice()).await?;
    Ok(())
}
