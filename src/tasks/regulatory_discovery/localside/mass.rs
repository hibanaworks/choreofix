//! The mass spectrometer receives each real request before invoking its effect.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::Endpoint;

pub async fn run<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, MASS_SERVICE>,
    mut instrument: Endpoint<'r, MASS>,
    domain: &D,
) -> ShellResult<()> {
    {
        let admitted = imp::decode(endpoint.recv::<MassIntent<2>>().await?)?;
        endpoint.send::<Instrument<MASS>>(instrument).await?;
        let request = imp::decode(endpoint.recv::<MassRequest<2>>().await?)?;
        imp::same(&request, &admitted)?;
        let (actor, tool, _, _) = imp::analysis(2)?;
        let result = domain
            .execute(actor, tool, request)
            .await
            .map_err(ShellError::message)?;
        imp::successful(&result)?;
        let bytes = imp::encode(&result)?;
        endpoint.send::<MassResult>(&bytes.as_slice()).await?;
        instrument = endpoint.recv::<Instrument<MASS>>().await?;
    }
    {
        let admitted = imp::decode(endpoint.recv::<MassIntent<3>>().await?)?;
        endpoint.send::<Instrument<MASS>>(instrument).await?;
        let request = imp::decode(endpoint.recv::<MassRequest<3>>().await?)?;
        imp::same(&request, &admitted)?;
        let (actor, tool, _, _) = imp::analysis(3)?;
        let result = domain
            .execute(actor, tool, request)
            .await
            .map_err(ShellError::message)?;
        imp::successful(&result)?;
        let bytes = imp::encode(&result)?;
        endpoint.send::<MassResult>(&bytes.as_slice()).await?;
        instrument = endpoint.recv::<Instrument<MASS>>().await?;
    }
    {
        let admitted = imp::decode(endpoint.recv::<MassIntent<8>>().await?)?;
        endpoint.send::<Instrument<MASS>>(instrument).await?;
        let request = imp::decode(endpoint.recv::<MassRequest<8>>().await?)?;
        imp::same(&request, &admitted)?;
        let (actor, tool, _, _) = imp::analysis(8)?;
        let result = domain
            .execute(actor, tool, request)
            .await
            .map_err(ShellError::message)?;
        imp::successful(&result)?;
        let bytes = imp::encode(&result)?;
        endpoint.send::<MassResult>(&bytes.as_slice()).await?;
        instrument = endpoint.recv::<Instrument<MASS>>().await?;
    }
    {
        let admitted = imp::decode(endpoint.recv::<MassIntent<10>>().await?)?;
        endpoint.send::<Instrument<MASS>>(instrument).await?;
        let request = imp::decode(endpoint.recv::<MassRequest<10>>().await?)?;
        imp::same(&request, &admitted)?;
        let (actor, tool, _, _) = imp::analysis(10)?;
        let result = domain
            .execute(actor, tool, request)
            .await
            .map_err(ShellError::message)?;
        imp::successful(&result)?;
        let bytes = imp::encode(&result)?;
        endpoint.send::<MassResult>(&bytes.as_slice()).await?;
        instrument = endpoint.recv::<Instrument<MASS>>().await?;
    }
    let study = endpoint.recv::<Specimen<STUDY>>().await?;
    drop((instrument, study));
    Ok(())
}
