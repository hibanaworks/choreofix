//! Original synthesis requests and real two-resource replies; source owns repetition.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult};
use hibana::{Endpoint, g::Message};
use serde_json::Value;

#[hibana_runtime::scope]
pub async fn run(endpoint: &mut Endpoint<CHEMIST>) -> ShellResult<Value> {
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            FileSynthesis::LOGICAL_LABEL => {
                let admitted = branch.recv::<FileSynthesis>().await?;
                endpoint.send::<SynthesisAdmission>(&admitted).await?;
                let (mut fume, mut stock) = async {
                    let [mut fume, mut stock] = endpoint.par()?;
                    async { fume.recv::<Loan<FUME_HOOD>>().await };
                    async { stock.recv::<Loan<REAGENT_SUPPLY>>().await };
                }
                .await?;
                let (fume_result, stock_result) = async {
                    async {
                        fume.send::<FumeSynthesis>(&admitted).await?;
                        imp::decode(fume.recv::<FumeSynthesisResult>().await?)
                    };
                    async {
                        stock.send::<ReagentSynthesis>(&admitted).await?;
                        imp::decode(stock.recv::<ReagentSynthesisResult>().await?)
                    };
                }
                .await?;
                if fume_result != stock_result {
                    return Err(ShellError::message(
                        "synthesis resources returned different actual results",
                    ));
                }
                imp::synthesis_result(&fume_result, &admitted.into_value())?;
                let bytes = imp::encode(&fume_result)?;
                async {
                    let [mut fume_owner, mut stock_owner, mut file] = endpoint.par()?;
                    async {
                        fume_owner.send::<Loan<FUME_HOOD>>(fume).await?;
                    };
                    async {
                        stock_owner.send::<Loan<REAGENT_SUPPLY>>(stock).await?;
                    };
                    async {
                        file.send::<FileSynthesisResult>(&bytes.as_slice()).await?;
                    };
                    Ok(())
                }
                .await?;
            }
            SynthesisFinished::LOGICAL_LABEL => {
                return Ok(branch.recv::<SynthesisFinished>().await?.into_value());
            }
            ReagentsExhausted::LOGICAL_LABEL => {
                return Ok(branch.recv::<ReagentsExhausted>().await?.into_value());
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected synthesis operation {label}"
                )));
            }
        }
    }
}
