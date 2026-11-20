use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::Endpoint;
use serde_json::json;

#[hibana_runtime::scope]
pub async fn run<'run>(
    endpoint: &mut Endpoint<'run, LEAD>,
    domain: &impl Domain,
) -> ShellResult<()> {
    let mut instrument = endpoint.recv::<Loan<MASS>>().await?;
    instrument.send::<Use<REFERENCE>>(&()).await?;
    let reference = imp::decode(instrument.recv::<Reply<REFERENCE>>().await?)?;
    endpoint.send::<Loan<MASS>>(instrument).await?;
    let bytes = imp::encode(&reference)?;
    endpoint.send::<Reply<REFERENCE>>(&bytes.as_slice()).await?;

    let (bio, tox) = async {
        let [mut bio, mut tox] = endpoint.par()?;
        async {
            let report = imp::decode(bio.recv::<BioReport>().await?)?;
            let bytes = imp::encode(&report)?;
            bio.send::<BioReport>(&bytes.as_slice()).await?;
            Ok(report)
        };
        async {
            let report = imp::decode(tox.recv::<ToxReport>().await?)?;
            let bytes = imp::encode(&report)?;
            tox.send::<ToxReport>(&bytes.as_slice()).await?;
            Ok(report)
        };
    }
    .await?;
    let compound = imp::compound(&bio["compound"])?;
    if bio["compound"] != tox["compound"]
        || tox["toxicity"]["data"]["inputs"]["prepared_cells"] != imp::artifact(&bio["cells"])?
    {
        return Err(ShellError::message(
            "discovery reports refer to different material",
        ));
    }
    let reference = imp::artifact(&reference)?;
    let (structural, characterization) = async {
        let [mut structure_lane, mut characterization_lane] = endpoint.par()?;
        async {
            let mut instrument = structure_lane.recv::<Loan<HPLC>>().await?;
            instrument.send::<Use<STRUCTURE>>(&()).await?;
            let structure = imp::decode(instrument.recv::<Reply<STRUCTURE>>().await?)?;
            imp::inputs(&structure, &json!({"compound":compound}))?;
            structure_lane.send::<Loan<HPLC>>(instrument).await?;
            let bytes = imp::encode(&structure)?;
            structure_lane
                .send::<Reply<STRUCTURE>>(&bytes.as_slice())
                .await?;
            imp::artifact(&structure)
        };
        async {
            let mut instrument = characterization_lane.recv::<Loan<MASS>>().await?;
            instrument.send::<Use<CHARACTERIZATION>>(&()).await?;
            let characterization =
                imp::decode(instrument.recv::<Reply<CHARACTERIZATION>>().await?)?;
            imp::inputs(
                &characterization,
                &json!({"compound":compound,"reference":reference}),
            )?;
            characterization_lane.send::<Loan<MASS>>(instrument).await?;
            let bytes = imp::encode(&characterization)?;
            characterization_lane
                .send::<Reply<CHARACTERIZATION>>(&bytes.as_slice())
                .await?;
            imp::artifact(&characterization)
        };
    }
    .await?;
    let arguments = endpoint.recv::<Request<DECISION>>().await?.into_value();
    let result = domain
        .execute("LEAD_SCIENTIST", "submit_decision", arguments.clone())
        .await
        .map_err(ShellError::message)?;
    imp::actual(&result, ("LEAD_SCIENTIST", "submit_decision", &arguments))?;
    imp::inputs(
        &result,
        &json!({
            "compound":compound,"bioassay":imp::artifact(&bio["assay"])?,
            "stability":imp::artifact(&bio["stability"])?,"toxicity":imp::artifact(&tox["toxicity"])?,
            "structural":structural,"characterization":characterization
        }),
    )?;
    let bytes = imp::encode(&result)?;
    async {
        let [mut file, mut chemistry] = endpoint.par()?;
        async {
            file.send::<Reply<DECISION>>(&bytes.as_slice()).await?;
        };
        async {
            chemistry.send::<DecisionReceipt>(&bytes.as_slice()).await?;
        };
        Ok(())
    }
    .await
}
