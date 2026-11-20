//! Actual machine-result deliveries, the actual file request, then report I/O.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::ReceivedEffects};
use hibana::{Endpoint, g::Message};

#[hibana_runtime::scope]
pub async fn run(
    endpoint: &mut Endpoint<SUPERVISOR>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    let (lathe, mill) = async {
        let [mut lathe, mut mill] = endpoint.par()?;
        async {
            let mut results = Vec::with_capacity(2);
            for _ in 0..2 {
                let incoming = lathe.offer().await?;
                let result = match incoming.label() {
                    LatheDoneA::LOGICAL_LABEL => imp::decode(incoming.recv::<LatheDoneA>().await?)?,
                    LatheDoneB::LOGICAL_LABEL => imp::decode(incoming.recv::<LatheDoneB>().await?)?,
                    _ => return Err(ShellError::message("unexpected lathe report")),
                };
                results.push(result);
            }
            Ok::<_, ShellError>(results)
        };
        async {
            let mut results = Vec::with_capacity(2);
            for _ in 0..2 {
                let incoming = mill.offer().await?;
                let result = match incoming.label() {
                    MillDoneA::LOGICAL_LABEL => imp::decode(incoming.recv::<MillDoneA>().await?)?,
                    MillDoneB::LOGICAL_LABEL => imp::decode(incoming.recv::<MillDoneB>().await?)?,
                    _ => return Err(ShellError::message("unexpected mill report")),
                };
                results.push(result);
            }
            Ok::<_, ShellError>(results)
        };
    }
    .await?;
    let receipts = lathe.into_iter().chain(mill).collect::<Vec<_>>();
    let expected = imp::completed_jobs(&receipts)?;
    let arguments = endpoint.recv::<CollectReports>().await?;
    let result = effects.apply(arguments).await?;
    imp::report_result(&expected, &result)?;
    let bytes = imp::encode(&result)?;
    endpoint.send::<Reports>(&bytes.as_slice()).await?;
    Ok(())
}
