//! Direct requests and actual results, with independent dishes driven concurrently.
use super::super::{global::*, imp};
use crate::shell::{
    ShellError, ShellResult,
    imp::{bus::Bus, service::Domain},
};
use hibana::Endpoint;

#[hibana_runtime::scope]
pub async fn run(endpoint: &mut Endpoint<FINISH>, domain: &Bus) -> ShellResult<()> {
    let [mut a, mut b, mut c] = endpoint.par()?;
    async {
        let arguments = a.recv::<CombineA>().await?.value();
        let result = domain
            .execute("CHEF_A", "combine_dish", arguments.clone())
            .await
            .map_err(ShellError::message)?;
        imp::completed("combine_dish", "CHEF_A", &arguments, &result)?;
        let bytes = imp::encode(&result)?;
        a.send::<DishA>(&bytes.as_slice()).await?;
    };
    async {
        let arguments = b.recv::<CombineB>().await?.value();
        let result = domain
            .execute("CHEF_B", "combine_dish", arguments.clone())
            .await
            .map_err(ShellError::message)?;
        imp::completed("combine_dish", "CHEF_B", &arguments, &result)?;
        let bytes = imp::encode(&result)?;
        b.send::<DishB>(&bytes.as_slice()).await?;
    };
    async {
        let arguments = c.recv::<CombineC>().await?.value();
        let result = domain
            .execute("CHEF_C", "combine_dish", arguments.clone())
            .await
            .map_err(ShellError::message)?;
        imp::completed("combine_dish", "CHEF_C", &arguments, &result)?;
        let bytes = imp::encode(&result)?;
        c.send::<DishC>(&bytes.as_slice()).await?;
    };
    Ok(())
}
