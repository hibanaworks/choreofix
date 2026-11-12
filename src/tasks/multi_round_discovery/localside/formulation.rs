//! The formulation owner retains its actual compound and lends its instrument.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::{Endpoint, g::Message};

#[hibana_runtime::scope]
pub async fn run<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, FORM_SERVICE>,
    mut instrument: Endpoint<'r, FORM>,
    domain: &D,
) -> ShellResult<()> {
    let identity = imp::decode(endpoint.recv::<Identity<FORM_SERVICE>>().await?)?;
    imp::reply(3, &identity)?;
    let mut admitted = imp::decode(endpoint.recv::<Intent<4>>().await?)?;
    loop {
        imp::formulation_input(&identity, &admitted)?;
        endpoint.send::<Instrument<FORM>>(instrument).await?;
        let request = imp::decode(endpoint.recv::<Request<4>>().await?)?;
        imp::same(&request, &admitted)?;
        instrument = endpoint.recv::<Instrument<FORM>>().await?;
        let mut material = endpoint.recv::<Specimen<FORMULATION>>().await?;
        let (actor, tool, _, _) = imp::analysis(4)?;
        let result = domain
            .execute(actor, tool, request)
            .await
            .map_err(ShellError::message)?;
        imp::formulation_receipt(&identity, &result)?;
        let bytes = imp::encode(&result)?;
        if result["success"] == true {
            material
                .send::<Passed<4>>(&imp::AnalysisReply::<4, true>::from_bytes(&bytes)?)
                .await?;
            endpoint.send::<Specimen<FORMULATION>>(material).await?;
            async {
                let [mut hplc, mut cells, mut mass, mut study] = endpoint.par()?;
                async {
                    hplc.send::<Formulation<HPLC_SERVICE>>(
                        &imp::JsonTo::<HPLC_SERVICE>::from_bytes(&bytes)?,
                    )
                    .await?;
                };
                async {
                    cells
                        .send::<Formulation<CELL_SERVICE>>(
                            &imp::JsonTo::<CELL_SERVICE>::from_bytes(&bytes)?,
                        )
                        .await?;
                };
                async {
                    mass.send::<Formulation<MASS_SERVICE>>(
                        &imp::JsonTo::<MASS_SERVICE>::from_bytes(&bytes)?,
                    )
                    .await?;
                };
                async {
                    study
                        .send::<Formulation<STUDY_SERVICE>>(
                            &imp::JsonTo::<STUDY_SERVICE>::from_bytes(&bytes)?,
                        )
                        .await?;
                };
                Ok(())
            }
            .await?;
            let branch = endpoint.offer().await?;
            if branch.label() == Final::<FORM_SERVICE>::LOGICAL_LABEL {
                let final_result = imp::decode(branch.recv::<Final<FORM_SERVICE>>().await?)?;
                imp::same(&final_result["compound"], &identity)?;
                imp::same(&final_result["formulation"], &result)?;
                return Ok(());
            }
            admitted = imp::decode(branch.recv::<Intent<4>>().await?)?;
        } else {
            material
                .send::<Failed<4>>(&imp::AnalysisReply::<4, false>::from_bytes(&bytes)?)
                .await?;
            endpoint.send::<Specimen<FORMULATION>>(material).await?;
            admitted = imp::decode(endpoint.recv::<Intent<4>>().await?)?;
        }
    }
}
