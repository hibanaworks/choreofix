//! Direct requests and actual results, with independent dishes driven concurrently.
use super::super::{global::*, imp};
use crate::shell::{
    ShellError, ShellResult,
    imp::{bus::Bus, service::Domain},
};
use hibana::Endpoint;

#[hibana_runtime::scope]
pub async fn run(endpoint: &mut Endpoint<INGREDIENTS>, domain: &Bus) -> ShellResult<()> {
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
            let [mut own, mut consumer] = a.par()?;
            async {
                own.send::<SaucePrepared>(&bytes.as_slice()).await?;
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
            let [mut own, mut consumer] = b.par()?;
            async {
                own.send::<GarnishPrepared>(&bytes.as_slice()).await?;
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
            let [mut own, mut consumer] = c.par()?;
            async {
                own.send::<GlazePrepared>(&bytes.as_slice()).await?;
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
