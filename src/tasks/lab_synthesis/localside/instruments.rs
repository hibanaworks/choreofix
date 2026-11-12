//! Physical receivers own the actual assay data and compare it before effects.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::Endpoint;
use serde_json::json;

#[hibana_runtime::scope]
pub async fn spectrometer<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, SPEC_SERVICE>,
    mut instrument: Endpoint<'r, SPECTROMETER>,
    domain: &D,
) -> ShellResult<()> {
    let raw = {
        let admitted = imp::decode(endpoint.recv::<SpecIntent>().await?)?;
        imp::assigned(&admitted, "sample", "raw_materials")?;
        endpoint.send::<SpecLoan>(instrument).await?;
        let request = imp::decode(endpoint.recv::<SpecRequest>().await?)?;
        imp::same(&request, &admitted)?;
        let result = domain
            .execute("CHEMIST", "run_analysis", request)
            .await
            .map_err(ShellError::message)?;
        imp::successful(&result)?;
        let bytes = imp::encode(&result)?;
        endpoint.send::<SpecResult>(&bytes.as_slice()).await?;
        instrument = endpoint.recv::<SpecLoan>().await?;
        result
    };
    let calibration = {
        let admitted = imp::decode(endpoint.recv::<SpecIntent>().await?)?;
        imp::assigned(&admitted, "sample", "fluorescence")?;
        endpoint.send::<SpecLoan>(instrument).await?;
        let request = imp::decode(endpoint.recv::<SpecRequest>().await?)?;
        imp::same(&request, &admitted)?;
        let result = domain
            .execute("BIOLOGIST", "run_analysis", request)
            .await
            .map_err(ShellError::message)?;
        imp::successful(&result)?;
        let bytes = imp::encode(&result)?;
        endpoint.send::<SpecResult>(&bytes.as_slice()).await?;
        instrument = endpoint.recv::<SpecLoan>().await?;
        result
    };
    let biology = imp::decode(endpoint.recv::<BioProof>().await?)?;
    let molecular = {
        let admitted = imp::decode(endpoint.recv::<SpecIntent>().await?)?;
        imp::assigned(&admitted, "sample", "molecular_detail")?;
        endpoint.send::<SpecLoan>(instrument).await?;
        let request = imp::decode(endpoint.recv::<SpecRequest>().await?)?;
        imp::same(&request, &admitted)?;
        let result = domain
            .execute("ANALYST", "run_analysis", request)
            .await
            .map_err(ShellError::message)?;
        imp::successful(&result)?;
        let bytes = imp::encode(&result)?;
        endpoint.send::<SpecResult>(&bytes.as_slice()).await?;
        instrument = endpoint.recv::<SpecLoan>().await?;
        result
    };
    let (verification, quality) = async {
        let [mut separation, mut report] = endpoint.par()?;
        async { imp::decode(separation.recv::<Verification>().await?) };
        async { imp::decode(report.recv::<QualityForOwner>().await?) };
    }
    .await?;
    imp::same(&quality["biology"]["sample"]["raw"], &raw)?;
    imp::same(
        &quality["biology"]["sample"]["purification"],
        &biology["purification"],
    )?;
    imp::same(&quality["biology"]["prep"], &biology["prep"])?;
    imp::same(&quality["biology"]["calibration"], &calibration)?;
    imp::same(&quality["biology"]["assay"], &biology["result"])?;
    imp::same(&quality["molecular"], &molecular)?;
    imp::same(&quality["separation"], &verification)?;
    let admitted = imp::decode(endpoint.recv::<SpecIntent>().await?)?;
    imp::assigned(&admitted["arguments"], "sample", "final_validation")?;
    imp::same(&admitted["quality"], &quality)?;
    endpoint.send::<SpecLoan>(instrument).await?;
    let request = imp::decode(endpoint.recv::<SpecRequest>().await?)?;
    imp::same(&request, &admitted)?;
    let result = domain
        .execute("CHEMIST", "run_analysis", request["arguments"].clone())
        .await
        .map_err(ShellError::message)?;
    imp::successful(&result)?;
    let bytes = imp::encode(&result)?;
    endpoint.send::<SpecResult>(&bytes.as_slice()).await?;
    let instrument = endpoint.recv::<SpecLoan>().await?;
    drop(instrument);
    Ok(())
}

#[hibana_runtime::scope]
pub async fn centrifuge<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, CENT_SERVICE>,
    mut instrument: Endpoint<'r, CENTRIFUGE>,
    domain: &D,
) -> ShellResult<()> {
    let prep = {
        let admitted = imp::decode(endpoint.recv::<CentIntent>().await?)?;
        imp::assigned(&admitted, "material", "cell_cultures")?;
        endpoint.send::<CentLoan>(instrument).await?;
        let request = imp::decode(endpoint.recv::<CentRequest>().await?)?;
        imp::same(&request, &admitted)?;
        let result = domain
            .execute("BIOLOGIST", "run_separation", request)
            .await
            .map_err(ShellError::message)?;
        imp::successful(&result)?;
        let bytes = imp::encode(&result)?;
        endpoint.send::<CentResult>(&bytes.as_slice()).await?;
        instrument = endpoint.recv::<CentLoan>().await?;
        result
    };
    let purification = {
        let admitted = imp::decode(endpoint.recv::<CentIntent>().await?)?;
        imp::assigned(&admitted, "material", "compound_purification")?;
        endpoint.send::<CentLoan>(instrument).await?;
        let request = imp::decode(endpoint.recv::<CentRequest>().await?)?;
        imp::same(&request, &admitted)?;
        let result = domain
            .execute("CHEMIST", "run_separation", request)
            .await
            .map_err(ShellError::message)?;
        imp::successful(&result)?;
        let bytes = imp::encode(&result)?;
        endpoint.send::<CentResult>(&bytes.as_slice()).await?;
        instrument = endpoint.recv::<CentLoan>().await?;
        result
    };
    let request = imp::decode(endpoint.recv::<BioRequest>().await?)?;
    imp::assay_arguments(&request["arguments"])?;
    imp::same(&request["purification"], &purification)?;
    imp::same(&request["prep"], &prep)?;
    let result = domain
        .execute("BIOLOGIST", "run_bioassay", request["arguments"].clone())
        .await
        .map_err(ShellError::message)?;
    imp::successful(&result)?;
    let proof = json!({"purification":purification,"prep":prep,"arguments":request["arguments"],"result":result});
    let proof_bytes = imp::encode(&proof)?;
    let bytes = imp::encode(&result)?;
    async {
        let [mut reply, mut observer] = endpoint.par()?;
        async {
            reply.send::<BioResult>(&bytes.as_slice()).await?;
        };
        async {
            observer.send::<BioProof>(&proof_bytes.as_slice()).await?;
        };
        Ok(())
    }
    .await?;
    let admitted = imp::decode(endpoint.recv::<CentIntent>().await?)?;
    imp::assigned(&admitted, "material", "verification")?;
    endpoint.send::<CentLoan>(instrument).await?;
    let request = imp::decode(endpoint.recv::<CentRequest>().await?)?;
    imp::same(&request, &admitted)?;
    let result = domain
        .execute("ANALYST", "run_separation", request)
        .await
        .map_err(ShellError::message)?;
    imp::successful(&result)?;
    let bytes = imp::encode(&result)?;
    endpoint.send::<CentResult>(&bytes.as_slice()).await?;
    let instrument = endpoint.recv::<CentLoan>().await?;
    endpoint.send::<Verification>(&bytes.as_slice()).await?;
    drop(instrument);
    Ok(())
}
