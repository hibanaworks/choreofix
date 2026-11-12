//! The mass spectrometer owns the actual baseline, identity and clinical values.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::{Endpoint, g::Message};
use serde_json::json;

#[hibana_runtime::scope]
pub async fn run<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, MASS_SERVICE>,
    mut instrument: Endpoint<'r, MASS>,
    domain: &D,
) -> ShellResult<()> {
    let admitted = imp::decode(endpoint.recv::<Intent<2>>().await?)?;
    endpoint.send::<Instrument<MASS>>(instrument).await?;
    let request = imp::decode(endpoint.recv::<Request<2>>().await?)?;
    imp::same(&request, &admitted)?;
    let (actor, tool, _, _) = imp::analysis(2)?;
    let baseline = domain
        .execute(actor, tool, request)
        .await
        .map_err(ShellError::message)?;
    imp::reply(2, &baseline)?;
    let bytes = imp::encode(&baseline)?;
    endpoint
        .send::<Passed<2>>(&imp::AnalysisReply::<2, true>::from_bytes(&bytes)?)
        .await?;
    instrument = endpoint.recv::<Instrument<MASS>>().await?;
    endpoint
        .send::<Baseline<CELL_SERVICE>>(&imp::JsonTo::<CELL_SERVICE>::from_bytes(&bytes)?)
        .await?;

    let qc = imp::decode(endpoint.recv::<QcEvidence>().await?)?;
    let qc_id = imp::reply(0, &qc)?;
    if qc["success"] != true {
        return Err(ShellError::message(
            "molecular identification requires successful QC",
        ));
    }
    let admitted = imp::decode(endpoint.recv::<Intent<3>>().await?)?;
    endpoint.send::<Instrument<MASS>>(instrument).await?;
    let request = imp::decode(endpoint.recv::<Request<3>>().await?)?;
    imp::same(&request, &admitted)?;
    let (actor, tool, _, _) = imp::analysis(3)?;
    let identity = domain
        .execute(actor, tool, request)
        .await
        .map_err(ShellError::message)?;
    imp::reply(3, &identity)?;
    if identity["data"]["inputs"]["qc"] != qc_id {
        return Err(ShellError::message(
            "compound identity cites another QC result",
        ));
    }
    let bytes = imp::encode(&identity)?;
    endpoint
        .send::<Passed<3>>(&imp::AnalysisReply::<3, true>::from_bytes(&bytes)?)
        .await?;
    instrument = endpoint.recv::<Instrument<MASS>>().await?;
    async {
        let [mut formulator, mut formulation, mut study] = endpoint.par()?;
        async {
            formulator
                .send::<Identity<FORMULATOR>>(&imp::JsonTo::<FORMULATOR>::from_bytes(&bytes)?)
                .await?;
        };
        async {
            formulation
                .send::<Identity<FORM_SERVICE>>(&imp::JsonTo::<FORM_SERVICE>::from_bytes(&bytes)?)
                .await?;
        };
        async {
            study
                .send::<Identity<STUDY_SERVICE>>(&imp::JsonTo::<STUDY_SERVICE>::from_bytes(&bytes)?)
                .await?;
        };
        Ok(())
    }
    .await?;

    let mut formulation = imp::decode(endpoint.recv::<Formulation<MASS_SERVICE>>().await?)?;
    loop {
        imp::formulation_receipt(&identity, &formulation)?;
        let purity = imp::decode(endpoint.recv::<PurityEvidence<MASS_SERVICE>>().await?)?;
        let ((bioassay, toxicity), stability) = async {
            let [mut cells, mut hplc] = endpoint.par()?;
            async {
                let bioassay = imp::decode(cells.recv::<BioEvidence<MASS_SERVICE>>().await?)?;
                let toxicity = imp::decode(cells.recv::<ToxEvidence<MASS_SERVICE>>().await?)?;
                Ok((bioassay, toxicity))
            };
            async { imp::decode(hplc.recv::<StabilityEvidence<MASS_SERVICE>>().await?) };
        }
        .await?;
        let candidate = json!({"formulation":formulation,"bioassay":bioassay,
            "purity":purity,"stability":stability,"toxicity":toxicity});
        imp::clinical_inputs(&candidate, &baseline)?;
        let admitted = imp::decode(endpoint.recv::<Intent<9>>().await?)?;
        endpoint.send::<Instrument<MASS>>(instrument).await?;
        let request = imp::decode(endpoint.recv::<Request<9>>().await?)?;
        imp::same(&request, &admitted)?;
        instrument = endpoint.recv::<Instrument<MASS>>().await?;
        let mut study = endpoint.recv::<Specimen<STUDY>>().await?;
        let (actor, tool, _, _) = imp::analysis(9)?;
        let clinical = domain
            .execute(actor, tool, request)
            .await
            .map_err(ShellError::message)?;
        imp::clinical_receipt(&candidate, &clinical)?;
        let bytes = imp::encode(&clinical)?;
        study
            .send::<Passed<9>>(&imp::AnalysisReply::<9, true>::from_bytes(&bytes)?)
            .await?;
        endpoint.send::<Specimen<STUDY>>(study).await?;
        async {
            let [mut regulator, mut study] = endpoint.par()?;
            async {
                regulator
                    .send::<ClinicalEvidence<REGULATOR>>(&imp::JsonTo::<REGULATOR>::from_bytes(
                        &bytes,
                    )?)
                    .await?;
            };
            async {
                study
                    .send::<ClinicalEvidence<STUDY_SERVICE>>(
                        &imp::JsonTo::<STUDY_SERVICE>::from_bytes(&bytes)?,
                    )
                    .await?;
            };
            Ok(())
        }
        .await?;

        formulation = loop {
            let branch = endpoint.offer().await?;
            match branch.label() {
                RoutineReview::LOGICAL_LABEL => {
                    imp::decode(branch.recv::<RoutineReview>().await?)?;
                }
                DeepIntent::<MASS>::LOGICAL_LABEL => {
                    let admitted = imp::decode(branch.recv::<DeepIntent<MASS>>().await?)?;
                    imp::arguments(11, &admitted)?;
                    endpoint.send::<Instrument<MASS>>(instrument).await?;
                    instrument = endpoint.recv::<Instrument<MASS>>().await?;
                }
                Formulation::<MASS_SERVICE>::LOGICAL_LABEL => {
                    break imp::decode(branch.recv::<Formulation<MASS_SERVICE>>().await?)?;
                }
                Final::<MASS_SERVICE>::LOGICAL_LABEL => {
                    let final_result = imp::decode(branch.recv::<Final<MASS_SERVICE>>().await?)?;
                    imp::same(&final_result["compound"], &identity)?;
                    imp::same(&final_result["formulation"], &formulation)?;
                    imp::same(&final_result["clinical"], &clinical)?;
                    let study = endpoint.recv::<Specimen<STUDY>>().await?;
                    drop((instrument, study));
                    return Ok(());
                }
                _ => {
                    return Err(ShellError::message(
                        "unexpected mass spectrometer operation",
                    ));
                }
            }
        };
    }
}
