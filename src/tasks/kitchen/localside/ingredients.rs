//! Receive the actual operation-file request and return the real effect.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::Endpoint;
#[hibana_runtime::scope]
pub async fn run(endpoint: &mut Endpoint<INGREDIENTS>, domain: &impl Domain) -> ShellResult<()> {
    let [mut a, mut b, mut c] = endpoint.par()?;
    async {
        let arguments = a.recv::<PrepareSauce>().await?.value();
        let result = domain
            .execute("CHEF_A", "prepare_ingredient", arguments.clone())
            .await
            .map_err(ShellError::message)?;
        imp::completed("prepare_ingredient", "CHEF_A", &arguments, &result)?;
        let bytes = imp::encode(&result)?;
        async {
            let [mut file, mut consumer] = a.par()?;
            async {
                file.send::<SaucePrepared>(&bytes.as_slice()).await?;
            };
            async {
                consumer.send::<Sauce>(&bytes.as_slice()).await?;
            };
            Ok::<(), ShellError>(())
        }
        .await?;
    };
    async {
        let arguments = b.recv::<PrepareGarnish>().await?.value();
        let result = domain
            .execute("CHEF_B", "prepare_ingredient", arguments.clone())
            .await
            .map_err(ShellError::message)?;
        imp::completed("prepare_ingredient", "CHEF_B", &arguments, &result)?;
        let bytes = imp::encode(&result)?;
        async {
            let [mut file, mut consumer] = b.par()?;
            async {
                file.send::<GarnishPrepared>(&bytes.as_slice()).await?;
            };
            async {
                consumer.send::<Garnish>(&bytes.as_slice()).await?;
            };
            Ok::<(), ShellError>(())
        }
        .await?;
    };
    async {
        let arguments = c.recv::<PrepareGlaze>().await?.value();
        let result = domain
            .execute("CHEF_C", "prepare_ingredient", arguments.clone())
            .await
            .map_err(ShellError::message)?;
        imp::completed("prepare_ingredient", "CHEF_C", &arguments, &result)?;
        let bytes = imp::encode(&result)?;
        async {
            let [mut file, mut consumer] = c.par()?;
            async {
                file.send::<GlazePrepared>(&bytes.as_slice()).await?;
            };
            async {
                consumer.send::<Glaze>(&bytes.as_slice()).await?;
            };
            Ok::<(), ShellError>(())
        }
        .await?;
    };
    Ok(())
}
