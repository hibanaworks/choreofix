use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult};
use hibana::{Endpoint, g::Message};

#[hibana_runtime::scope]
pub async fn run<'r>(
    endpoint: &mut Endpoint<'r, BIOLOGIST>,
    mut culture: Endpoint<'r, CULTURE>,
) -> ShellResult<()> {
    loop {
        let mut instrument = endpoint.recv::<Instrument<CELL>>().await?;
        instrument.send::<Use<1>>(&()).await?;
        endpoint.send::<Instrument<CELL>>(instrument).await?;
        endpoint.send::<Specimen<CULTURE>>(culture).await?;
        let branch = endpoint.offer().await?;
        let actual = match branch.label() {
            Passed::<1>::LOGICAL_LABEL => imp::decode(branch.recv::<Passed<1>>().await?)?,
            Failed::<1>::LOGICAL_LABEL => imp::decode(branch.recv::<Failed<1>>().await?)?,
            label => {
                return Err(ShellError::message(format!(
                    "unexpected scientific result {label}"
                )));
            }
        };
        let bytes = imp::encode(&actual)?;
        endpoint.send::<FileResult<1>>(&bytes.as_slice()).await?;
        if actual["success"] == true {
            break;
        }
        culture = endpoint.recv::<Specimen<CULTURE>>().await?;
    }
    loop {
        let branch = endpoint.offer().await?;
        if branch.label() == Final::<BIOLOGIST>::LOGICAL_LABEL {
            let result = branch.recv::<Final<BIOLOGIST>>().await?.to_vec();
            endpoint
                .send::<StudyFile<BIO_FILE>>(&imp::JsonTo::<BIO_FILE>::from_bytes(&result)?)
                .await?;
            return Ok(());
        }
        drop(branch);
        let (mut sample, mut stability) = async {
            let [mut bio, mut stable] = endpoint.par()?;
            async { bio.recv::<Specimen<BIOASSAY>>().await };
            async { stable.recv::<Specimen<STABILITY>>().await };
        }
        .await?;
        async {
            let [mut bio, mut stable] = endpoint.par()?;
            async {
                loop {
                    let branch = bio.offer().await?;
                    if branch.label() == Publish::<5>::LOGICAL_LABEL {
                        branch.recv::<Publish<5>>().await?;
                        bio.send::<Specimen<BIOASSAY>>(sample).await?;
                        break;
                    }
                    drop(branch);
                    let (mut cell, mut samples) = async {
                        let [mut laboratory, mut stock] = bio.par()?;
                        async { laboratory.recv::<Instrument<CELL>>().await };
                        async { stock.recv::<Instrument<SAMPLES>>().await };
                    }
                    .await?;
                    async {
                        async { cell.send::<Use<5>>(&()).await };
                        async { samples.send::<SampleUse>(&()).await };
                    }
                    .await?;
                    async {
                        let [mut stock, mut laboratory] = bio.par()?;
                        async { stock.send::<Instrument<SAMPLES>>(samples).await };
                        async { laboratory.send::<Instrument<CELL>>(cell).await };
                    }
                    .await?;
                    bio.send::<Specimen<BIOASSAY>>(sample).await?;
                    let branch = bio.offer().await?;
                    let actual = match branch.label() {
                        Passed::<5>::LOGICAL_LABEL => {
                            imp::decode(branch.recv::<Passed<5>>().await?)?
                        }
                        Failed::<5>::LOGICAL_LABEL => {
                            imp::decode(branch.recv::<Failed<5>>().await?)?
                        }
                        label => {
                            return Err(ShellError::message(format!(
                                "unexpected assay result {label}"
                            )));
                        }
                    };
                    let bytes = imp::encode(&actual)?;
                    bio.send::<FileResult<5>>(&bytes.as_slice()).await?;
                    sample = bio.recv::<Specimen<BIOASSAY>>().await?;
                }
            };
            async {
                loop {
                    let mut instrument = stable.recv::<Instrument<HPLC>>().await?;
                    instrument.send::<Use<6>>(&()).await?;
                    stable.send::<Instrument<HPLC>>(instrument).await?;
                    stable.send::<Specimen<STABILITY>>(stability).await?;
                    let branch = stable.offer().await?;
                    let actual = match branch.label() {
                        Passed::<6>::LOGICAL_LABEL => {
                            imp::decode(branch.recv::<Passed<6>>().await?)?
                        }
                        Failed::<6>::LOGICAL_LABEL => {
                            imp::decode(branch.recv::<Failed<6>>().await?)?
                        }
                        label => {
                            return Err(ShellError::message(format!(
                                "unexpected scientific result {label}"
                            )));
                        }
                    };
                    let bytes = imp::encode(&actual)?;
                    stable.send::<FileResult<6>>(&bytes.as_slice()).await?;
                    if actual["success"] == true {
                        break;
                    }
                    stability = stable.recv::<Specimen<STABILITY>>().await?;
                }
            };
            Ok(())
        }
        .await?;
    }
}
