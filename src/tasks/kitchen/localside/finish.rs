//! Receive the actual operation-file request and return the real effect.
use super::super::{global::*, imp};
use crate::shell::{ShellResult, imp::service::ReceivedEffects};
use hibana::Endpoint;
#[hibana_runtime::scope]
pub async fn run(
    endpoint: &mut Endpoint<FINISH>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    let [mut a, mut b, mut c] = endpoint.par()?;
    async {
        let received = a.recv::<CombineA>().await?;
        let arguments = imp::decode(received.as_bytes())?;
        let result = effects.apply(received).await?;
        imp::completed("combine_dish", "CHEF_A", &arguments, &result)?;
        let bytes = imp::encode(&result)?;
        a.send::<DishA>(&bytes.as_slice()).await?;
    };
    async {
        let received = b.recv::<CombineB>().await?;
        let arguments = imp::decode(received.as_bytes())?;
        let result = effects.apply(received).await?;
        imp::completed("combine_dish", "CHEF_B", &arguments, &result)?;
        let bytes = imp::encode(&result)?;
        b.send::<DishB>(&bytes.as_slice()).await?;
    };
    async {
        let received = c.recv::<CombineC>().await?;
        let arguments = imp::decode(received.as_bytes())?;
        let result = effects.apply(received).await?;
        imp::completed("combine_dish", "CHEF_C", &arguments, &result)?;
        let bytes = imp::encode(&result)?;
        c.send::<DishC>(&bytes.as_slice()).await?;
    };
    Ok(())
}
