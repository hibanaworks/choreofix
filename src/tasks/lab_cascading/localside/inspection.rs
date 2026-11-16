//! One physical inspection, after all four received requests match the admitted input.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::Endpoint;

#[hibana_runtime::scope]
pub async fn run<D: Domain>(
    endpoint: &mut Endpoint<INSPECTION_SERVICE>,
    domain: &D,
) -> ShellResult<()> {
    let admitted = endpoint.recv::<InspectionAdmission>().await?.into_value();
    let (spec, cent, fume, chroma) = async {
        let [mut spec, mut cent, mut fume, mut chroma] = endpoint.par()?;
        async { Ok(spec.recv::<SpecInspection>().await?.into_value()) };
        async { Ok(cent.recv::<CentInspection>().await?.into_value()) };
        async { Ok(fume.recv::<FumeInspection>().await?.into_value()) };
        async { Ok(chroma.recv::<ChromaInspection>().await?.into_value()) };
    }
    .await?;
    if [&spec, &cent, &fume, &chroma]
        .into_iter()
        .any(|request| request != &admitted)
    {
        return Err(ShellError::message(
            "inspection instrument request differs from the admitted file input",
        ));
    }
    let actual = domain
        .execute("SAFETY_OFFICER", "perform_inspection", admitted)
        .await
        .map_err(ShellError::message)?;
    let bytes = imp::encode(&actual)?;
    async {
        let [mut spec, mut cent, mut fume, mut chroma] = endpoint.par()?;
        async {
            spec.send::<SpecResult>(&bytes.as_slice()).await?;
        };
        async {
            cent.send::<CentResult>(&bytes.as_slice()).await?;
        };
        async {
            fume.send::<FumeResult>(&bytes.as_slice()).await?;
        };
        async {
            chroma.send::<ChromaResult>(&bytes.as_slice()).await?;
        };
        Ok(())
    }
    .await?;
    Ok(())
}
