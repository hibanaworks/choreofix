//! Transfer the actual material and publish the checked decision.
use super::super::{global::*, imp};
use crate::shell::ShellResult;
use hibana::Endpoint;
use serde_json::{Value, json};

#[hibana_runtime::scope]
pub async fn run<'r>(
    (endpoint, mut study, mut formulation, mut bioassay, mut stability, mut purity, mut toxicity): (
        &mut Endpoint<'r, DIRECTOR>,
        Endpoint<'r, STUDY>,
        Endpoint<'r, FORMULATION>,
        Endpoint<'r, BIOASSAY>,
        Endpoint<'r, STABILITY>,
        Endpoint<'r, PURITY>,
        Endpoint<'r, TOXICITY>,
    ),
) -> ShellResult<()> {
    let identity = imp::decode(endpoint.recv::<Identity<DIRECTOR>>().await?)?;
    let mut previous = Value::Null;
    loop {
        let bytes = imp::encode(&json!({"compound":identity,"previous":previous}))?;
        study.send::<RoundBrief>(&bytes.as_slice()).await?;
        study.send::<Specimen<FORMULATION>>(formulation).await?;
        async {
            let [mut bio, mut stable, mut pure, mut toxic] = endpoint.par()?;
            async { bio.send::<Specimen<BIOASSAY>>(bioassay).await };
            async { stable.send::<Specimen<STABILITY>>(stability).await };
            async { pure.send::<Specimen<PURITY>>(purity).await };
            async { toxic.send::<Specimen<TOXICITY>>(toxicity).await };
        }
        .await?;
        endpoint.send::<Specimen<STUDY>>(study).await?;
        (formulation, bioassay, stability, purity, toxicity) = async {
            let [mut form, mut bio, mut stable, mut pure, mut toxic] = endpoint.par()?;
            async { form.recv::<Specimen<FORMULATION>>().await };
            async { bio.recv::<Specimen<BIOASSAY>>().await };
            async { stable.recv::<Specimen<STABILITY>>().await };
            async { pure.recv::<Specimen<PURITY>>().await };
            async { toxic.recv::<Specimen<TOXICITY>>().await };
        }
        .await?;
        let regulatory = imp::decode(endpoint.recv::<RegulatoryEvidence<DIRECTOR>>().await?)?;
        let bytes = imp::encode(&regulatory)?;
        endpoint
            .send::<StudyFile<DIR_FILE>>(&imp::JsonTo::<DIR_FILE>::from_bytes(&bytes)?)
            .await?;
        let outcome = imp::decode(endpoint.recv::<DecisionEvidence<DIRECTOR>>().await?)?;
        imp::same(&outcome["compound"], &identity)?;
        imp::same(&outcome["regulatory"], &regulatory)?;
        imp::decision_receipt(&outcome, &outcome["decision"])?;
        let bytes = imp::encode(&outcome["decision"])?;
        endpoint.send::<DecisionResult>(&bytes.as_slice()).await?;
        study = endpoint.recv::<Specimen<STUDY>>().await?;
        if outcome["outcome"] == "retry" {
            previous = outcome;
            continue;
        }
        endpoint.send::<Specimen<STUDY>>(study).await?;
        async {
            let [mut form, mut bio, mut stable, mut pure, mut toxic] = endpoint.par()?;
            async { form.send::<Archived<FORMULATION>>(formulation).await };
            async { bio.send::<Archived<BIOASSAY>>(bioassay).await };
            async { stable.send::<Archived<STABILITY>>(stability).await };
            async { pure.send::<Archived<PURITY>>(purity).await };
            async { toxic.send::<Archived<TOXICITY>>(toxicity).await };
        }
        .await?;
        let bytes = imp::encode(&outcome)?;
        async {
            let [
                mut chemist,
                mut biologist,
                mut toxicologist,
                mut formulator,
                mut clinician,
                mut regulator,
                mut hplc,
                mut mass,
                mut cells,
                mut form,
                mut review,
            ] = endpoint.par()?;
            async {
                chemist
                    .send::<Final<CHEMIST>>(&imp::FinalOutcome::<CHEMIST>::from_bytes(&bytes)?)
                    .await
            };
            async {
                biologist
                    .send::<Final<BIOLOGIST>>(&imp::FinalOutcome::<BIOLOGIST>::from_bytes(&bytes)?)
                    .await
            };
            async {
                toxicologist
                    .send::<Final<TOXICOLOGIST>>(&imp::FinalOutcome::<TOXICOLOGIST>::from_bytes(
                        &bytes,
                    )?)
                    .await
            };
            async {
                formulator
                    .send::<Final<FORMULATOR>>(&imp::FinalOutcome::<FORMULATOR>::from_bytes(
                        &bytes,
                    )?)
                    .await
            };
            async {
                clinician
                    .send::<Final<CLINICIAN>>(&imp::FinalOutcome::<CLINICIAN>::from_bytes(&bytes)?)
                    .await
            };
            async {
                regulator
                    .send::<Final<REGULATOR>>(&imp::FinalOutcome::<REGULATOR>::from_bytes(&bytes)?)
                    .await
            };
            async {
                hplc.send::<Final<HPLC_SERVICE>>(&imp::FinalOutcome::<HPLC_SERVICE>::from_bytes(
                    &bytes,
                )?)
                .await
            };
            async {
                mass.send::<Final<MASS_SERVICE>>(&imp::FinalOutcome::<MASS_SERVICE>::from_bytes(
                    &bytes,
                )?)
                .await
            };
            async {
                cells
                    .send::<Final<CELL_SERVICE>>(&imp::FinalOutcome::<CELL_SERVICE>::from_bytes(
                        &bytes,
                    )?)
                    .await
            };
            async {
                form.send::<Final<FORM_SERVICE>>(&imp::FinalOutcome::<FORM_SERVICE>::from_bytes(
                    &bytes,
                )?)
                .await
            };
            async {
                review
                    .send::<Final<STUDY_SERVICE>>(&imp::FinalOutcome::<STUDY_SERVICE>::from_bytes(
                        &bytes,
                    )?)
                    .await
            };
        }
        .await?;
        return Ok(());
    }
}
