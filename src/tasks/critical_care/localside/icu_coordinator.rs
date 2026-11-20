//! Both actual resource requests precede the one ICU effect.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult};
use hibana::Endpoint;
#[hibana_runtime::scope]
pub async fn run(endpoint: &mut Endpoint<ICU_COORDINATOR>) -> ShellResult<()> {
    let request = imp::EffectRequest(endpoint.recv::<IcuAdmitted>().await?);
    let (mut bed, mut ventilator) = async {
        let [mut bed, mut vent] = endpoint.par()?;
        async { bed.recv::<ResourceLoan<ICU_BEDS>>().await };
        async { vent.recv::<ResourceLoan<VENTILATOR>>().await };
    }
    .await?;
    async {
        async {
            bed.send::<BedRequest>(&request).await?;
        };
        async {
            ventilator.send::<VentilatorRequest>(&request).await?;
        };
        Ok::<(), ShellError>(())
    }
    .await?;
    let (result, other) = async {
        async { imp::decode(bed.recv::<BedResult>().await?) };
        async { imp::decode(ventilator.recv::<VentilatorResult>().await?) };
    }
    .await?;
    imp::same(&result, &other)?;
    imp::actual_result(&result, "patient_b")?;
    async {
        let [mut b, mut v] = endpoint.par()?;
        async {
            b.send::<ResourceReturn<ICU_BEDS>>(bed).await?;
        };
        async {
            v.send::<ResourceReturn<VENTILATOR>>(ventilator).await?;
        };
        Ok::<(), ShellError>(())
    }
    .await?;
    Ok(())
}
