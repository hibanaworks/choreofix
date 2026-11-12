use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::Endpoint;

pub async fn run<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, REPORT_SERVICE>,
    domain: &D,
) -> ShellResult<()> {
    let receipts = imp::decode(endpoint.recv::<CompletedJobs>().await?)?;
    let expected = imp::completed_jobs(&receipts)?;
    let arguments = imp::report_arguments(imp::decode(endpoint.recv::<CollectReports>().await?)?)?;
    let result = domain
        .execute("SUPERVISOR", "collect_reports", arguments)
        .await
        .map_err(ShellError::message)?;
    imp::report_result(&expected, &result)?;
    let bytes = imp::encode(&result)?;
    endpoint.send::<Reports>(&bytes.as_slice()).await?;
    Ok(())
}
