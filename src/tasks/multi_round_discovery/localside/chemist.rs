use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult};
use hibana::{Endpoint, g::Message};

#[hibana_runtime::scope]
pub async fn run<'r>(
    endpoint: &mut Endpoint<'r, CHEMIST>,
    mut qc: Endpoint<'r, QC>,
) -> ShellResult<()> {
    loop {
        let mut instrument = endpoint.recv::<Instrument<HPLC>>().await?;
        instrument.send::<Use<0>>(&()).await?;
        endpoint.send::<Instrument<HPLC>>(instrument).await?;
        endpoint.send::<Specimen<QC>>(qc).await?;
        let branch = endpoint.offer().await?;
        let actual = match branch.label() {
            Passed::<0>::LOGICAL_LABEL => imp::decode(branch.recv::<Passed<0>>().await?)?,
            Failed::<0>::LOGICAL_LABEL => imp::decode(branch.recv::<Failed<0>>().await?)?,
            label => {
                return Err(ShellError::message(format!(
                    "unexpected scientific result {label}"
                )));
            }
        };
        let bytes = imp::encode(&actual)?;
        endpoint.send::<FileResult<0>>(&bytes.as_slice()).await?;
        if actual["success"] == true {
            break;
        }
        qc = endpoint.recv::<Specimen<QC>>().await?;
    }
    let mut mass = endpoint.recv::<Instrument<MASS>>().await?;
    mass.send::<Use<3>>(&()).await?;
    let actual = imp::decode(mass.recv::<Passed<3>>().await?)?;
    endpoint.send::<Instrument<MASS>>(mass).await?;
    let bytes = imp::encode(&actual)?;
    endpoint.send::<FileResult<3>>(&bytes.as_slice()).await?;
    let mut study = endpoint.recv::<Specimen<STUDY>>().await?;
    let result = endpoint.recv::<Final<CHEMIST>>().await?.to_vec();
    endpoint
        .send::<StudyFile<CHEM_FILE>>(&imp::JsonTo::<CHEM_FILE>::from_bytes(&result)?)
        .await?;
    loop {
        let mut instrument = endpoint.recv::<Instrument<HPLC>>().await?;
        instrument.send::<Use<15>>(&()).await?;
        endpoint.send::<Instrument<HPLC>>(instrument).await?;
        endpoint.send::<Specimen<STUDY>>(study).await?;
        let branch = endpoint.offer().await?;
        let actual = match branch.label() {
            Passed::<15>::LOGICAL_LABEL => imp::decode(branch.recv::<Passed<15>>().await?)?,
            Failed::<15>::LOGICAL_LABEL => imp::decode(branch.recv::<Failed<15>>().await?)?,
            label => {
                return Err(ShellError::message(format!(
                    "unexpected scientific result {label}"
                )));
            }
        };
        let bytes = imp::encode(&actual)?;
        endpoint.send::<FileResult<15>>(&bytes.as_slice()).await?;
        if actual["success"] == true {
            break;
        }
        study = endpoint.recv::<Specimen<STUDY>>().await?;
    }
    Ok(())
}
