//! Both actual current objects and the file request precede the link effect.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::Endpoint;
#[hibana_runtime::scope]
pub async fn run(endpoint: &mut Endpoint<INTEGRATOR>, domain: &impl Domain) -> ShellResult<()> {
    drop(endpoint.offer().await?);
    let (a, b) = async {
        let [mut a, mut b] = endpoint.par()?;
        async { imp::decode(a.recv::<BuiltA>().await?) };
        async { imp::decode(b.recv::<BuiltB>().await?) };
    }
    .await?;
    let objects = [
        imp::compiled(&a, "BUILDER_A", "module_a")?,
        imp::compiled(&b, "BUILDER_B", "module_b")?,
    ];
    let arguments = endpoint.recv::<Link>().await?.into_value();
    if objects[0]["inputs"]["SHARED_TYPES"] != objects[1]["inputs"]["SHARED_TYPES"] {
        return Err(ShellError::message(
            "current objects were built against different types",
        ));
    }
    let result = domain
        .execute("INTEGRATOR", "link_modules", arguments)
        .await
        .map_err(ShellError::message)?;
    imp::linked(&result, &objects)?;
    let bytes = imp::encode(&result)?;
    endpoint.send::<Linked>(&bytes.as_slice()).await?;
    Ok(())
}
