//! The officer uses each received instrument itself; no token or alternate effect path.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult};
use hibana::Endpoint;

#[hibana_runtime::scope]
pub async fn run(endpoint: &mut Endpoint<SAFETY_OFFICER>) -> ShellResult<()> {
    let admitted = endpoint.recv::<FileInspection>().await?;
    endpoint.send::<InspectionAdmission>(&admitted).await?;
    let (mut spec, mut cent, mut fume, mut chroma) = async {
        let [mut spec, mut cent, mut fume, mut chroma] = endpoint.par()?;
        async {
            spec.send::<SpecInspection>(&admitted).await?;
            spec.recv::<Loan<SPECTROMETER>>().await
        };
        async {
            cent.send::<CentInspection>(&admitted).await?;
            cent.recv::<Loan<CENTRIFUGE>>().await
        };
        async {
            fume.send::<FumeInspection>(&admitted).await?;
            fume.recv::<Loan<FUME_HOOD>>().await
        };
        async {
            chroma.send::<ChromaInspection>(&admitted).await?;
            chroma.recv::<Loan<CHROMATOGRAPH>>().await
        };
    }
    .await?;
    let (spec_result, cent_result, fume_result, chroma_result) = async {
        async {
            spec.send::<SpecInspection>(&admitted).await?;
            imp::decode(spec.recv::<SpecResult>().await?)
        };
        async {
            cent.send::<CentInspection>(&admitted).await?;
            imp::decode(cent.recv::<CentResult>().await?)
        };
        async {
            fume.send::<FumeInspection>(&admitted).await?;
            imp::decode(fume.recv::<FumeResult>().await?)
        };
        async {
            chroma.send::<ChromaInspection>(&admitted).await?;
            imp::decode(chroma.recv::<ChromaResult>().await?)
        };
    }
    .await?;
    if [&cent_result, &fume_result, &chroma_result]
        .into_iter()
        .any(|result| result != &spec_result)
    {
        return Err(ShellError::message(
            "inspection instruments returned different physical results",
        ));
    }
    let bytes = imp::encode(&spec_result)?;
    async {
        let [
            mut spec_owner,
            mut cent_owner,
            mut fume_owner,
            mut chroma_owner,
            mut file,
        ] = endpoint.par()?;
        async {
            spec_owner.send::<Loan<SPECTROMETER>>(spec).await?;
        };
        async {
            cent_owner.send::<Loan<CENTRIFUGE>>(cent).await?;
        };
        async {
            fume_owner.send::<Loan<FUME_HOOD>>(fume).await?;
        };
        async {
            chroma_owner.send::<Loan<CHROMATOGRAPH>>(chroma).await?;
        };
        async {
            file.send::<FileInspectionResult>(&bytes.as_slice()).await?;
        };
        Ok(())
    }
    .await?;
    Ok(())
}
