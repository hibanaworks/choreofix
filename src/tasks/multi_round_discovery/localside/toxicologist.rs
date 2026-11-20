use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult};
use hibana::{Endpoint, g::Message};

#[hibana_runtime::scope]
pub async fn run(endpoint: &mut Endpoint<'_, TOXICOLOGIST>) -> ShellResult<()> {
    let mut mass = endpoint.recv::<Instrument<MASS>>().await?;
    mass.send::<Use<2>>(&()).await?;
    let actual = imp::decode(mass.recv::<Passed<2>>().await?)?;
    endpoint.send::<Instrument<MASS>>(mass).await?;
    let bytes = imp::encode(&actual)?;
    endpoint.send::<FileResult<2>>(&bytes.as_slice()).await?;
    loop {
        let branch = endpoint.offer().await?;
        if branch.label() == Final::<TOXICOLOGIST>::LOGICAL_LABEL {
            let result = branch.recv::<Final<TOXICOLOGIST>>().await?.to_vec();
            endpoint
                .send::<StudyFile<TOX_FILE>>(&imp::JsonTo::<TOX_FILE>::from_bytes(&result)?)
                .await?;
            return Ok(());
        }
        drop(branch);
        let (mut purity_cap, mut toxicity) = async {
            let [mut pure, mut toxic] = endpoint.par()?;
            async { pure.recv::<Specimen<PURITY>>().await };
            async { toxic.recv::<Specimen<TOXICITY>>().await };
        }
        .await?;
        let purity = loop {
            let mut instrument = endpoint.recv::<Instrument<HPLC>>().await?;
            instrument.send::<Use<7>>(&()).await?;
            endpoint.send::<Instrument<HPLC>>(instrument).await?;
            endpoint.send::<Specimen<PURITY>>(purity_cap).await?;
            let branch = endpoint.offer().await?;
            let actual = match branch.label() {
                Passed::<7>::LOGICAL_LABEL => imp::decode(branch.recv::<Passed<7>>().await?)?,
                Failed::<7>::LOGICAL_LABEL => imp::decode(branch.recv::<Failed<7>>().await?)?,
                label => {
                    return Err(ShellError::message(format!(
                        "unexpected scientific result {label}"
                    )));
                }
            };
            let bytes = imp::encode(&actual)?;
            endpoint.send::<FileResult<7>>(&bytes.as_slice()).await?;
            if actual["success"] == true {
                break actual;
            }
            purity_cap = endpoint.recv::<Specimen<PURITY>>().await?;
        };
        let bytes = imp::encode(&purity)?;
        endpoint
            .send::<PurityEvidence<CELL_SERVICE>>(&imp::JsonTo::<CELL_SERVICE>::from_bytes(&bytes)?)
            .await?;
        loop {
            let branch = endpoint.offer().await?;
            if branch.label() == Publish::<8>::LOGICAL_LABEL {
                branch.recv::<Publish<8>>().await?;
                endpoint.send::<Specimen<TOXICITY>>(toxicity).await?;
                break;
            }
            drop(branch);
            let (mut cell, mut samples) = async {
                let [mut laboratory, mut stock] = endpoint.par()?;
                async { laboratory.recv::<Instrument<CELL>>().await };
                async { stock.recv::<Instrument<SAMPLES>>().await };
            }
            .await?;
            async {
                async { cell.send::<Use<8>>(&()).await };
                async { samples.send::<SampleUse>(&()).await };
            }
            .await?;
            async {
                let [mut stock, mut laboratory] = endpoint.par()?;
                async { stock.send::<Instrument<SAMPLES>>(samples).await };
                async { laboratory.send::<Instrument<CELL>>(cell).await };
            }
            .await?;
            endpoint.send::<Specimen<TOXICITY>>(toxicity).await?;
            let branch = endpoint.offer().await?;
            let actual = match branch.label() {
                Passed::<8>::LOGICAL_LABEL => imp::decode(branch.recv::<Passed<8>>().await?)?,
                Failed::<8>::LOGICAL_LABEL => imp::decode(branch.recv::<Failed<8>>().await?)?,
                label => {
                    return Err(ShellError::message(format!(
                        "unexpected assay result {label}"
                    )));
                }
            };
            let bytes = imp::encode(&actual)?;
            endpoint.send::<FileResult<8>>(&bytes.as_slice()).await?;
            toxicity = endpoint.recv::<Specimen<TOXICITY>>().await?;
        }
    }
}
