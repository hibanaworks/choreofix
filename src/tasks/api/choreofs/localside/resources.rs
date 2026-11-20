//! The physical resource owner sends the actual request before moving its Endpoint.
use super::super::{global::*, imp::ApiArguments};
use crate::shell::ShellResult;
use hibana::Endpoint;

#[hibana_runtime::scope]
pub(crate) async fn database<'run>(
    endpoint: &mut Endpoint<'run, DATABASE>,
    mut resource: Endpoint<'run, DATABASE_RESOURCE>,
) -> ShellResult<()> {
    let arguments = endpoint.recv::<MigrationIntent>().await?;
    resource
        .send::<MigrationRequest>(&ApiArguments(arguments.0))
        .await?;
    endpoint.send::<Loan<DATABASE_RESOURCE>>(resource).await?;
    endpoint.recv::<MigrationCompleted>().await?;
    let _resource = endpoint.recv::<Loan<DATABASE_RESOURCE>>().await?;
    Ok(())
}

#[hibana_runtime::scope]
pub(crate) async fn test_environment<'run>(
    endpoint: &mut Endpoint<'run, TEST_ENV>,
    mut resource: Endpoint<'run, TEST_RESOURCE>,
) -> ShellResult<()> {
    let arguments = endpoint.recv::<TestIntent>().await?;
    resource
        .send::<TestRequest>(&ApiArguments(arguments.0))
        .await?;
    endpoint.send::<Loan<TEST_RESOURCE>>(resource).await?;
    endpoint.recv::<TestCompleted>().await?;
    let _resource = endpoint.recv::<Loan<TEST_RESOURCE>>().await?;
    Ok(())
}
