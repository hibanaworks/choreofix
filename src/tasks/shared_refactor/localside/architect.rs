//! Independent actual design operations, directly on their projected fragments.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::ReceivedEffects};
use hibana::Endpoint;
#[hibana_runtime::scope]
pub async fn run(
    architect: &mut Endpoint<ARCHITECT>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    let [mut a, mut b] = architect.par()?;
    async {
        let received = a.recv::<DesignA>().await?;
        let value = effects.apply(received).await?;
        if value["success"] != true
            || value["data"]["proposal"] != "module_a_refactor"
            || !value["data"]["document"].is_string()
        {
            return Err(ShellError::message(
                "design result does not match its request",
            ));
        }
        let bytes = imp::encode(&value)?;
        a.send::<DesignedA>(&bytes.as_slice()).await?;
    };
    async {
        let received = b.recv::<DesignB>().await?;
        let value = effects.apply(received).await?;
        if value["success"] != true
            || value["data"]["proposal"] != "module_b_refactor"
            || !value["data"]["document"].is_string()
        {
            return Err(ShellError::message(
                "design result does not match its request",
            ));
        }
        let bytes = imp::encode(&value)?;
        b.send::<DesignedB>(&bytes.as_slice()).await?;
    };
    Ok(())
}
