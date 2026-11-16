//! One QC I/O participant consumes both real resource requests before one effect.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::Endpoint;
use serde_json::Value;

#[hibana_runtime::scope]
pub async fn run<D: Domain>(
    owner: &mut Endpoint<QC_SERVICE>,
    (domain, actual_materials): (&D, &Value),
) -> ShellResult<()> {
    let admitted = imp::decode(owner.recv::<QcAdmission>().await?)?;
    let (spectrometer, fume_hood) = async {
        let [mut spectrometer, mut fume_hood] = owner.par()?;
        async { imp::decode(spectrometer.recv::<QcSpecRequest>().await?) };
        async { imp::decode(fume_hood.recv::<QcFumeRequest>().await?) };
    }
    .await?;
    let inputs = imp::qc_packet(&admitted, actual_materials)?;
    if spectrometer != admitted || fume_hood != admitted {
        return Err(ShellError::message(
            "QC resource request differs from the admitted actual request",
        ));
    }
    let result = domain
        .execute(
            "QC_CONTROLLER",
            "verify_result",
            admitted["arguments"].clone(),
        )
        .await
        .map_err(ShellError::message)?;
    imp::qc_result(&result, &admitted["arguments"], &inputs)?;
    let bytes = imp::encode(&result)?;
    async {
        let [mut spectrometer, mut fume_hood] = owner.par()?;
        async {
            spectrometer.send::<QcSpecResult>(&bytes.as_slice()).await?;
        };
        async {
            fume_hood.send::<QcFumeResult>(&bytes.as_slice()).await?;
        };
        Ok(())
    }
    .await?;
    owner.send::<QcResult>(&bytes.as_slice()).await?;
    Ok(())
}
