use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult};
use hibana::{Endpoint, g::Message};

#[hibana_runtime::scope]
pub async fn run(endpoint: &mut Endpoint<'_, FORMULATOR>) -> ShellResult<()> {
    let _qc = endpoint.recv::<Specimen<QC>>().await?;
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            RoundBrief::LOGICAL_LABEL => {
                let brief = imp::decode(branch.recv::<RoundBrief>().await?)?;
                let mut material = endpoint.recv::<Specimen<FORMULATION>>().await?;
                let bytes = imp::encode(&brief)?;
                endpoint
                    .send::<StudyFile<FORM_FILE>>(&imp::JsonTo::<FORM_FILE>::from_bytes(&bytes)?)
                    .await?;
                loop {
                    let mut instrument = endpoint.recv::<Instrument<FORM>>().await?;
                    instrument.send::<Use<4>>(&()).await?;
                    endpoint.send::<Instrument<FORM>>(instrument).await?;
                    endpoint.send::<Specimen<FORMULATION>>(material).await?;
                    let branch = endpoint.offer().await?;
                    let actual = match branch.label() {
                        Passed::<4>::LOGICAL_LABEL => {
                            imp::decode(branch.recv::<Passed<4>>().await?)?
                        }
                        Failed::<4>::LOGICAL_LABEL => {
                            imp::decode(branch.recv::<Failed<4>>().await?)?
                        }
                        label => {
                            return Err(ShellError::message(format!(
                                "unexpected scientific result {label}"
                            )));
                        }
                    };
                    let bytes = imp::encode(&actual)?;
                    endpoint.send::<FileResult<4>>(&bytes.as_slice()).await?;
                    if actual["success"] == true {
                        break;
                    }
                    material = endpoint.recv::<Specimen<FORMULATION>>().await?;
                }
            }
            Final::<FORMULATOR>::LOGICAL_LABEL => {
                let result = branch.recv::<Final<FORMULATOR>>().await?.to_vec();
                endpoint
                    .send::<StudyFile<FORM_FILE>>(&imp::JsonTo::<FORM_FILE>::from_bytes(&result)?)
                    .await?;
                return Ok(());
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected formulation event {label}"
                )));
            }
        }
    }
}
