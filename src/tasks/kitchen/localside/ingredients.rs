//! Receive the actual operation-file request and return the real effect.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::ReceivedEffects};
use hibana::Endpoint;
#[hibana_runtime::scope]
pub async fn run(
    endpoint: &mut Endpoint<INGREDIENTS>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    let [mut a, mut b, mut c] = endpoint.par()?;
    async {
        let received = a.recv::<PrepareSauce>().await?;
        let arguments = imp::decode(received.as_bytes())?;
        let result = effects.apply(received).await?;
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
        let received = b.recv::<PrepareGarnish>().await?;
        let arguments = imp::decode(received.as_bytes())?;
        let result = effects.apply(received).await?;
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
        let received = c.recv::<PrepareGlaze>().await?;
        let arguments = imp::decode(received.as_bytes())?;
        let result = effects.apply(received).await?;
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
