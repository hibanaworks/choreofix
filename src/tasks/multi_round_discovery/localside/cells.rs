//! The cell laboratory owns actual cells/results and both physical instruments.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::{Endpoint, g::Message};
use serde_json::json;

#[hibana_runtime::scope]
pub async fn run<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, CELL_SERVICE>,
    (mut cell, mut samples): (Endpoint<'r, CELL>, Endpoint<'r, SAMPLES>),
    domain: &D,
) -> ShellResult<()> {
    let cells = loop {
        let admitted = imp::decode(endpoint.recv::<Intent<1>>().await?)?;
        endpoint.send::<Instrument<CELL>>(cell).await?;
        let request = imp::decode(endpoint.recv::<Request<1>>().await?)?;
        imp::same(&request, &admitted)?;
        cell = endpoint.recv::<Instrument<CELL>>().await?;
        let mut culture = endpoint.recv::<Specimen<CULTURE>>().await?;
        let actual = domain
            .execute("BIOLOGIST", "run_cell_assay", request)
            .await
            .map_err(ShellError::message)?;
        imp::reply(1, &actual)?;
        let bytes = imp::encode(&actual)?;
        if actual["success"] == true {
            culture
                .send::<Passed<1>>(&imp::AnalysisReply::<1, true>::from_bytes(&bytes)?)
                .await?;
            endpoint.send::<Specimen<CULTURE>>(culture).await?;
            endpoint
                .send::<Cells<HPLC_SERVICE>>(&imp::JsonTo::<HPLC_SERVICE>::from_bytes(&bytes)?)
                .await?;
            break actual;
        }
        culture
            .send::<Failed<1>>(&imp::AnalysisReply::<1, false>::from_bytes(&bytes)?)
            .await?;
        endpoint.send::<Specimen<CULTURE>>(culture).await?;
    };
    let baseline = imp::decode(endpoint.recv::<Baseline<CELL_SERVICE>>().await?)?;
    imp::reply(2, &baseline)?;
    let mut formulation = imp::decode(endpoint.recv::<Formulation<CELL_SERVICE>>().await?)?;
    loop {
        async {
            let [mut bio, mut tox] = endpoint.par()?;
            async {
                let material = bio.recv::<Specimen<BIOASSAY>>().await?;
                bio.send::<Specimen<BIOASSAY>>(material).await
            };
            async {
                let material = tox.recv::<Specimen<TOXICITY>>().await?;
                tox.send::<Specimen<TOXICITY>>(material).await
            };
        }
        .await?;
        let bio_inputs = imp::assay_inputs(5, &json!({"formulation":formulation,"cells":cells}))?;
        let mut admitted = imp::decode(endpoint.recv::<Intent<5>>().await?)?;
        let bioassay = loop {
            async {
                let [mut laboratory, mut stock] = endpoint.par()?;
                async { laboratory.send::<Instrument<CELL>>(cell).await };
                async { stock.send::<Instrument<SAMPLES>>(samples).await };
            }
            .await?;
            let (request, sample_request) = async {
                let [mut laboratory, mut stock] = endpoint.par()?;
                async { imp::decode(laboratory.recv::<Request<5>>().await?) };
                async { imp::decode(stock.recv::<SampleRequest<5>>().await?) };
            }
            .await?;
            imp::same(&request, &admitted)?;
            imp::same(&sample_request, &admitted)?;
            (samples, cell) = async {
                let [mut stock, mut laboratory] = endpoint.par()?;
                async { stock.recv::<Instrument<SAMPLES>>().await };
                async { laboratory.recv::<Instrument<CELL>>().await };
            }
            .await?;
            let mut specimen = endpoint.recv::<Specimen<BIOASSAY>>().await?;
            let actual = domain
                .execute("BIOLOGIST", "run_cell_assay", request)
                .await
                .map_err(ShellError::message)?;
            imp::assay_receipt(5, &bio_inputs, &actual)?;
            let bytes = imp::encode(&actual)?;
            if actual["success"] == true {
                specimen
                    .send::<Passed<5>>(&imp::AnalysisReply::<5, true>::from_bytes(&bytes)?)
                    .await?;
            } else {
                specimen
                    .send::<Failed<5>>(&imp::AnalysisReply::<5, false>::from_bytes(&bytes)?)
                    .await?;
            }
            endpoint.send::<Specimen<BIOASSAY>>(specimen).await?;
            let branch = endpoint.offer().await?;
            match branch.label() {
                label if label == Intent::<5>::LOGICAL_LABEL => {
                    admitted = imp::decode(branch.recv::<Intent<5>>().await?)?;
                }
                label if label == Specimen::<BIOASSAY>::LOGICAL_LABEL => {
                    let specimen = branch.recv::<Specimen<BIOASSAY>>().await?;
                    endpoint.send::<Specimen<BIOASSAY>>(specimen).await?;
                    endpoint
                        .send::<BioEvidence<MASS_SERVICE>>(
                            &imp::JsonTo::<MASS_SERVICE>::from_bytes(&bytes)?,
                        )
                        .await?;
                    break actual;
                }
                _ => {
                    return Err(ShellError::message("unexpected biological assay operation"));
                }
            }
        };
        let purity = imp::decode(endpoint.recv::<PurityEvidence<CELL_SERVICE>>().await?)?;
        let tox_inputs = imp::assay_inputs(
            8,
            &json!({"formulation":formulation,"cells":cells,"baseline":baseline,"purity":purity}),
        )?;
        let mut admitted = imp::decode(endpoint.recv::<Intent<8>>().await?)?;
        let toxicity = loop {
            async {
                let [mut laboratory, mut stock] = endpoint.par()?;
                async { laboratory.send::<Instrument<CELL>>(cell).await };
                async { stock.send::<Instrument<SAMPLES>>(samples).await };
            }
            .await?;
            let (request, sample_request) = async {
                let [mut laboratory, mut stock] = endpoint.par()?;
                async { imp::decode(laboratory.recv::<Request<8>>().await?) };
                async { imp::decode(stock.recv::<SampleRequest<8>>().await?) };
            }
            .await?;
            imp::same(&request, &admitted)?;
            imp::same(&sample_request, &admitted)?;
            (samples, cell) = async {
                let [mut stock, mut laboratory] = endpoint.par()?;
                async { stock.recv::<Instrument<SAMPLES>>().await };
                async { laboratory.recv::<Instrument<CELL>>().await };
            }
            .await?;
            let mut specimen = endpoint.recv::<Specimen<TOXICITY>>().await?;
            let actual = domain
                .execute("TOXICOLOGIST", "run_cell_assay", request)
                .await
                .map_err(ShellError::message)?;
            imp::assay_receipt(8, &tox_inputs, &actual)?;
            let bytes = imp::encode(&actual)?;
            if actual["success"] == true {
                specimen
                    .send::<Passed<8>>(&imp::AnalysisReply::<8, true>::from_bytes(&bytes)?)
                    .await?;
            } else {
                specimen
                    .send::<Failed<8>>(&imp::AnalysisReply::<8, false>::from_bytes(&bytes)?)
                    .await?;
            }
            endpoint.send::<Specimen<TOXICITY>>(specimen).await?;
            let branch = endpoint.offer().await?;
            match branch.label() {
                label if label == Intent::<8>::LOGICAL_LABEL => {
                    admitted = imp::decode(branch.recv::<Intent<8>>().await?)?;
                }
                label if label == Specimen::<TOXICITY>::LOGICAL_LABEL => {
                    let specimen = branch.recv::<Specimen<TOXICITY>>().await?;
                    endpoint.send::<Specimen<TOXICITY>>(specimen).await?;
                    endpoint
                        .send::<ToxEvidence<MASS_SERVICE>>(
                            &imp::JsonTo::<MASS_SERVICE>::from_bytes(&bytes)?,
                        )
                        .await?;
                    break actual;
                }
                _ => return Err(ShellError::message("unexpected toxicity assay operation")),
            }
        };
        let branch = endpoint.offer().await?;
        if branch.label() == Final::<CELL_SERVICE>::LOGICAL_LABEL {
            let final_result = imp::decode(branch.recv::<Final<CELL_SERVICE>>().await?)?;
            imp::same(&final_result["formulation"], &formulation)?;
            imp::same(
                &final_result["clinical"]["data"]["inputs"]["bioassay"],
                &bioassay["data"]["event_id"],
            )?;
            imp::same(
                &final_result["clinical"]["data"]["inputs"]["toxicity"],
                &toxicity["data"]["event_id"],
            )?;
            return Ok(());
        }
        formulation = imp::decode(branch.recv::<Formulation<CELL_SERVICE>>().await?)?;
    }
}
