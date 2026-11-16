//! Independent actual design operations, directly on their projected fragments.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::Endpoint;
#[hibana_runtime::scope]
pub async fn run<D: Domain>(architect: &mut Endpoint<ARCHITECT>, domain: &D) -> ShellResult<()> {
    let [mut a, mut b] = architect.par()?;
    async {
        let args = a.recv::<DesignA>().await?.into_value();
        let value = domain
            .execute("ARCHITECT", "design_proposal", args)
            .await
            .map_err(ShellError::message)?;
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
        let args = b.recv::<DesignB>().await?.into_value();
        let value = domain
            .execute("ARCHITECT", "design_proposal", args)
            .await
            .map_err(ShellError::message)?;
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
