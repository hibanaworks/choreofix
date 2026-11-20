use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult};
use hibana::{Endpoint, g::Message};

#[hibana_runtime::scope]
pub async fn run(endpoint: &mut Endpoint<'_, REGULATOR>) -> ShellResult<()> {
    loop {
        let branch = endpoint.offer().await?;
        if branch.label() == Final::<REGULATOR>::LOGICAL_LABEL {
            let result = branch.recv::<Final<REGULATOR>>().await?.to_vec();
            endpoint
                .send::<StudyFile<REG_FILE>>(&imp::JsonTo::<REG_FILE>::from_bytes(&result)?)
                .await?;
            return Ok(());
        }
        let mut study = branch.recv::<Specimen<STUDY>>().await?;
        let clinical = endpoint
            .recv::<ClinicalEvidence<REGULATOR>>()
            .await?
            .to_vec();
        endpoint
            .send::<StudyFile<REG_FILE>>(&imp::JsonTo::<REG_FILE>::from_bytes(&clinical)?)
            .await?;
        loop {
            let branch = endpoint.offer().await?;
            match branch.label() {
                FileRequest::<10>::LOGICAL_LABEL => {
                    branch.recv::<FileRequest<10>>().await?;
                    endpoint.send::<Specimen<STUDY>>(study).await?;
                    let branch = endpoint.offer().await?;
                    let actual = if branch.label() == Passed::<10>::LOGICAL_LABEL {
                        imp::decode(branch.recv::<Passed<10>>().await?)?
                    } else {
                        imp::decode(branch.recv::<Failed<10>>().await?)?
                    };
                    let bytes = imp::encode(&actual)?;
                    endpoint.send::<FileResult<10>>(&bytes.as_slice()).await?;
                    study = endpoint.recv::<Specimen<STUDY>>().await?;
                }
                FileRequest::<11>::LOGICAL_LABEL => {
                    let request = branch.recv::<FileRequest<11>>().await?.to_vec();
                    study
                        .send::<DeepIntent<HPLC>>(&imp::JsonTo::<HPLC>::from_bytes(&request)?)
                        .await?;
                    let mut hplc = endpoint.recv::<Instrument<HPLC>>().await?;
                    study
                        .send::<DeepIntent<MASS>>(&imp::JsonTo::<MASS>::from_bytes(&request)?)
                        .await?;
                    let mut mass = endpoint.recv::<Instrument<MASS>>().await?;
                    async {
                        async { hplc.send::<Use<11>>(&()).await };
                        async { mass.send::<Use<11>>(&()).await };
                    }
                    .await?;
                    async {
                        let [mut chromatography, mut spectrometry] = endpoint.par()?;
                        async { chromatography.send::<Instrument<HPLC>>(hplc).await };
                        async { spectrometry.send::<Instrument<MASS>>(mass).await };
                    }
                    .await?;
                    endpoint.send::<Specimen<STUDY>>(study).await?;
                    let branch = endpoint.offer().await?;
                    let actual = match branch.label() {
                        Passed::<11>::LOGICAL_LABEL => {
                            imp::decode(branch.recv::<Passed<11>>().await?)?
                        }
                        Failed::<11>::LOGICAL_LABEL => {
                            imp::decode(branch.recv::<Failed<11>>().await?)?
                        }
                        label => {
                            return Err(ShellError::message(format!(
                                "unexpected hold review result {label}"
                            )));
                        }
                    };
                    study = endpoint.recv::<Specimen<STUDY>>().await?;
                    let (hplc, mass) = async {
                        let [mut chromatography, mut spectrometry] = endpoint.par()?;
                        async { chromatography.recv::<Instrument<HPLC>>().await };
                        async { spectrometry.recv::<Instrument<MASS>>().await };
                    }
                    .await?;
                    endpoint.send::<Instrument<MASS>>(mass).await?;
                    endpoint.send::<Instrument<HPLC>>(hplc).await?;
                    let bytes = imp::encode(&actual)?;
                    endpoint.send::<FileResult<11>>(&bytes.as_slice()).await?;
                }
                Publish::<10>::LOGICAL_LABEL => {
                    branch.recv::<Publish<10>>().await?;
                    endpoint.send::<Specimen<STUDY>>(study).await?;
                    break;
                }
                label => {
                    return Err(ShellError::message(format!(
                        "unexpected regulatory event {label}"
                    )));
                }
            }
        }
    }
}
