use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::Endpoint;

pub async fn hplc<'run>(
    endpoint: &mut Endpoint<'run, HPLC_OWNER>,
    domain: &impl Domain,
) -> ShellResult<()> {
    let mut instrument = endpoint.recv::<Loan<HPLC>>().await?;
    let arguments = endpoint.recv::<Request<SYNTHESIS_QC>>().await?.into_value();
    endpoint.send::<Loan<HPLC>>(instrument).await?;
    endpoint.recv::<Use<SYNTHESIS_QC>>().await?;
    let receipt = domain
        .execute("CHEMIST", "run_hplc_analysis", arguments.clone())
        .await
        .map_err(ShellError::message)?;
    imp::actual(&receipt, ("CHEMIST", "run_hplc_analysis", &arguments))?;
    let bytes = imp::encode(&receipt)?;
    endpoint
        .send::<Reply<SYNTHESIS_QC>>(&bytes.as_slice())
        .await?;
    instrument = endpoint.recv::<Loan<HPLC>>().await?;

    let arguments = endpoint.recv::<Request<PURITY>>().await?.into_value();
    endpoint.send::<Loan<HPLC>>(instrument).await?;
    endpoint.recv::<Use<PURITY>>().await?;
    let receipt = domain
        .execute("TOXICOLOGIST", "run_hplc_analysis", arguments.clone())
        .await
        .map_err(ShellError::message)?;
    imp::actual(&receipt, ("TOXICOLOGIST", "run_hplc_analysis", &arguments))?;
    let bytes = imp::encode(&receipt)?;
    endpoint.send::<Reply<PURITY>>(&bytes.as_slice()).await?;
    instrument = endpoint.recv::<Loan<HPLC>>().await?;

    let arguments = endpoint.recv::<Request<STABILITY>>().await?.into_value();
    endpoint.send::<Loan<HPLC>>(instrument).await?;
    endpoint.recv::<Use<STABILITY>>().await?;
    let receipt = domain
        .execute("BIOLOGIST", "run_hplc_analysis", arguments.clone())
        .await
        .map_err(ShellError::message)?;
    imp::actual(&receipt, ("BIOLOGIST", "run_hplc_analysis", &arguments))?;
    let bytes = imp::encode(&receipt)?;
    endpoint.send::<Reply<STABILITY>>(&bytes.as_slice()).await?;
    instrument = endpoint.recv::<Loan<HPLC>>().await?;

    let arguments = endpoint.recv::<Request<STRUCTURE>>().await?.into_value();
    endpoint.send::<Loan<HPLC>>(instrument).await?;
    endpoint.recv::<Use<STRUCTURE>>().await?;
    let receipt = domain
        .execute("LEAD_SCIENTIST", "run_hplc_analysis", arguments.clone())
        .await
        .map_err(ShellError::message)?;
    imp::actual(
        &receipt,
        ("LEAD_SCIENTIST", "run_hplc_analysis", &arguments),
    )?;
    let bytes = imp::encode(&receipt)?;
    endpoint.send::<Reply<STRUCTURE>>(&bytes.as_slice()).await?;
    instrument = endpoint.recv::<Loan<HPLC>>().await?;

    let arguments = endpoint
        .recv::<Request<DOCUMENTATION>>()
        .await?
        .into_value();
    endpoint.send::<Loan<HPLC>>(instrument).await?;
    endpoint.recv::<Use<DOCUMENTATION>>().await?;
    let receipt = domain
        .execute("CHEMIST", "run_hplc_analysis", arguments.clone())
        .await
        .map_err(ShellError::message)?;
    imp::actual(&receipt, ("CHEMIST", "run_hplc_analysis", &arguments))?;
    let bytes = imp::encode(&receipt)?;
    endpoint
        .send::<Reply<DOCUMENTATION>>(&bytes.as_slice())
        .await?;
    instrument = endpoint.recv::<Loan<HPLC>>().await?;
    drop(instrument);
    Ok(())
}

#[hibana_runtime::scope]
pub async fn mass<'run>(
    endpoint: &mut Endpoint<'run, MASS_OWNER>,
    domain: &impl Domain,
) -> ShellResult<()> {
    let instrument = endpoint.recv::<Loan<MASS>>().await?;
    let (mut instrument, reference) = async {
        let [mut baseline, mut reference] = endpoint.par()?;
        async {
            let arguments = baseline.recv::<Request<BASELINE>>().await?.into_value();
            baseline.send::<Loan<MASS>>(instrument).await?;
            baseline.recv::<Use<BASELINE>>().await?;
            let receipt = domain
                .execute("TOXICOLOGIST", "run_mass_spec_analysis", arguments.clone())
                .await
                .map_err(ShellError::message)?;
            imp::actual(
                &receipt,
                ("TOXICOLOGIST", "run_mass_spec_analysis", &arguments),
            )?;
            let bytes = imp::encode(&receipt)?;
            baseline.send::<Reply<BASELINE>>(&bytes.as_slice()).await?;
            Ok(baseline.recv::<Loan<MASS>>().await?)
        };
        async { Ok(reference.recv::<Request<REFERENCE>>().await?.into_value()) };
    }
    .await?;
    endpoint.send::<Loan<MASS>>(instrument).await?;
    endpoint.recv::<Use<REFERENCE>>().await?;
    let receipt = domain
        .execute(
            "LEAD_SCIENTIST",
            "run_mass_spec_analysis",
            reference.clone(),
        )
        .await
        .map_err(ShellError::message)?;
    imp::actual(
        &receipt,
        ("LEAD_SCIENTIST", "run_mass_spec_analysis", &reference),
    )?;
    let bytes = imp::encode(&receipt)?;
    endpoint.send::<Reply<REFERENCE>>(&bytes.as_slice()).await?;
    instrument = endpoint.recv::<Loan<MASS>>().await?;

    let arguments = endpoint.recv::<Request<MOLECULAR_ID>>().await?.into_value();
    endpoint.send::<Loan<MASS>>(instrument).await?;
    endpoint.recv::<Use<MOLECULAR_ID>>().await?;
    let receipt = domain
        .execute("CHEMIST", "run_mass_spec_analysis", arguments.clone())
        .await
        .map_err(ShellError::message)?;
    imp::actual(&receipt, ("CHEMIST", "run_mass_spec_analysis", &arguments))?;
    let bytes = imp::encode(&receipt)?;
    endpoint
        .send::<Reply<MOLECULAR_ID>>(&bytes.as_slice())
        .await?;
    instrument = endpoint.recv::<Loan<MASS>>().await?;

    let arguments = endpoint
        .recv::<Request<CHARACTERIZATION>>()
        .await?
        .into_value();
    endpoint.send::<Loan<MASS>>(instrument).await?;
    endpoint.recv::<Use<CHARACTERIZATION>>().await?;
    let receipt = domain
        .execute(
            "LEAD_SCIENTIST",
            "run_mass_spec_analysis",
            arguments.clone(),
        )
        .await
        .map_err(ShellError::message)?;
    imp::actual(
        &receipt,
        ("LEAD_SCIENTIST", "run_mass_spec_analysis", &arguments),
    )?;
    let bytes = imp::encode(&receipt)?;
    endpoint
        .send::<Reply<CHARACTERIZATION>>(&bytes.as_slice())
        .await?;
    instrument = endpoint.recv::<Loan<MASS>>().await?;
    drop(instrument);
    Ok(())
}

pub async fn cell<'run>(
    endpoint: &mut Endpoint<'run, CELL_OWNER>,
    domain: &impl Domain,
) -> ShellResult<()> {
    let mut instrument = endpoint.recv::<Loan<CELL>>().await?;
    let arguments = endpoint.recv::<Request<CULTURE_PREP>>().await?.into_value();
    endpoint.send::<Loan<CELL>>(instrument).await?;
    endpoint.recv::<Use<CULTURE_PREP>>().await?;
    let receipt = domain
        .execute("BIOLOGIST", "run_cell_assay", arguments.clone())
        .await
        .map_err(ShellError::message)?;
    imp::actual(&receipt, ("BIOLOGIST", "run_cell_assay", &arguments))?;
    let bytes = imp::encode(&receipt)?;
    endpoint
        .send::<Reply<CULTURE_PREP>>(&bytes.as_slice())
        .await?;

    let arguments = endpoint.recv::<Request<BIOASSAY>>().await?.into_value();
    endpoint.recv::<Use<BIOASSAY>>().await?;
    let receipt = domain
        .execute("BIOLOGIST", "run_cell_assay", arguments.clone())
        .await
        .map_err(ShellError::message)?;
    imp::actual(&receipt, ("BIOLOGIST", "run_cell_assay", &arguments))?;
    let bytes = imp::encode(&receipt)?;
    endpoint.send::<Reply<BIOASSAY>>(&bytes.as_slice()).await?;
    instrument = endpoint.recv::<Loan<CELL>>().await?;

    let arguments = endpoint.recv::<Request<CYTOTOXICITY>>().await?.into_value();
    endpoint.send::<Loan<CELL>>(instrument).await?;
    endpoint.recv::<Use<CYTOTOXICITY>>().await?;
    let receipt = domain
        .execute("TOXICOLOGIST", "run_cell_assay", arguments.clone())
        .await
        .map_err(ShellError::message)?;
    imp::actual(&receipt, ("TOXICOLOGIST", "run_cell_assay", &arguments))?;
    let bytes = imp::encode(&receipt)?;
    endpoint
        .send::<Reply<CYTOTOXICITY>>(&bytes.as_slice())
        .await?;
    instrument = endpoint.recv::<Loan<CELL>>().await?;
    drop(instrument);
    Ok(())
}
