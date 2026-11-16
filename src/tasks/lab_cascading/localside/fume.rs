//! The physical world owns reagent quantity; actual source requests own access.
use super::super::{
    global::*,
    imp::{self, SynthesisTerminal},
};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::Endpoint;

#[hibana_runtime::scope]
pub async fn run<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, FUME_OWNER>,
    (mut fume, mut stock): (Endpoint<'r, FUME_HOOD>, Endpoint<'r, REAGENT_SUPPLY>),
    domain: &D,
) -> ShellResult<Endpoint<'r, FUME_HOOD>> {
    loop {
        let admitted = endpoint.recv::<SynthesisAdmission>().await?.into_value();
        async {
            let [mut fume_owner, mut stock_owner] = endpoint.par()?;
            async {
                fume_owner.send::<Loan<FUME_HOOD>>(fume).await?;
            };
            async {
                stock_owner.send::<Loan<REAGENT_SUPPLY>>(stock).await?;
            };
            Ok(())
        }
        .await?;
        let (fume_request, stock_request) = async {
            let [mut fume_request, mut stock_request] = endpoint.par()?;
            async { Ok(fume_request.recv::<FumeSynthesis>().await?.into_value()) };
            async { Ok(stock_request.recv::<ReagentSynthesis>().await?.into_value()) };
        }
        .await?;
        if fume_request != admitted || stock_request != admitted {
            return Err(ShellError::message(
                "synthesis resource request differs from admitted file arguments",
            ));
        }
        let actual = domain
            .execute("CHEMIST", "run_synthesis", admitted.clone())
            .await
            .map_err(ShellError::message)?;
        imp::synthesis_result(&actual, &admitted)?;
        let bytes = imp::encode(&actual)?;
        async {
            let [mut fume_result, mut stock_result] = endpoint.par()?;
            async {
                fume_result
                    .send::<FumeSynthesisResult>(&bytes.as_slice())
                    .await?;
            };
            async {
                stock_result
                    .send::<ReagentSynthesisResult>(&bytes.as_slice())
                    .await?;
            };
            Ok(())
        }
        .await?;
        (fume, stock) = async {
            let [mut fume_owner, mut stock_owner] = endpoint.par()?;
            async { fume_owner.recv::<Loan<FUME_HOOD>>().await };
            async { stock_owner.recv::<Loan<REAGENT_SUPPLY>>().await };
        }
        .await?;
        if actual["success"] == true {
            let terminal = SynthesisTerminal::<true>::try_from(bytes.as_slice())?;
            endpoint.send::<SynthesisFinished>(&terminal).await?;
            endpoint.send::<SynthesisFinished>(&terminal).await?;
            endpoint.send::<Loan<REAGENT_SUPPLY>>(stock).await?;
            return Ok(fume);
        }
        if actual["data"]["status"] == "reagent_exhausted" {
            let terminal = SynthesisTerminal::<false>::try_from(bytes.as_slice())?;
            endpoint.send::<ReagentsExhausted>(&terminal).await?;
            endpoint.send::<ReagentsExhausted>(&terminal).await?;
            endpoint.send::<Loan<REAGENT_SUPPLY>>(stock).await?;
            return Ok(fume);
        }
    }
}
