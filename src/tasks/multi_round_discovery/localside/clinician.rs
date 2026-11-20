use super::super::{global::*, imp};
use crate::shell::ShellResult;
use hibana::{Endpoint, g::Message};

#[hibana_runtime::scope]
pub async fn run(endpoint: &mut Endpoint<'_, CLINICIAN>) -> ShellResult<()> {
    let _culture = endpoint.recv::<Specimen<CULTURE>>().await?;
    loop {
        let branch = endpoint.offer().await?;
        if branch.label() == Final::<CLINICIAN>::LOGICAL_LABEL {
            let result = branch.recv::<Final<CLINICIAN>>().await?.to_vec();
            endpoint
                .send::<StudyFile<CLIN_FILE>>(&imp::JsonTo::<CLIN_FILE>::from_bytes(&result)?)
                .await?;
            return Ok(());
        }
        let formulation = branch.recv::<Specimen<FORMULATION>>().await?;
        let study = endpoint.recv::<Specimen<STUDY>>().await?;
        let purity = endpoint.recv::<Specimen<PURITY>>().await?;
        let ((bioassay, toxicity), stability) = async {
            let [mut cells, mut hplc] = endpoint.par()?;
            async {
                let bioassay = cells.recv::<Specimen<BIOASSAY>>().await?;
                let toxicity = cells.recv::<Specimen<TOXICITY>>().await?;
                Ok((bioassay, toxicity))
            };
            async { hplc.recv::<Specimen<STABILITY>>().await };
        }
        .await?;
        let mut mass = endpoint.recv::<Instrument<MASS>>().await?;
        mass.send::<Use<9>>(&()).await?;
        endpoint.send::<Instrument<MASS>>(mass).await?;
        endpoint.send::<Specimen<STUDY>>(study).await?;
        let actual = imp::decode(endpoint.recv::<Passed<9>>().await?)?;
        let bytes = imp::encode(&actual)?;
        endpoint.send::<FileResult<9>>(&bytes.as_slice()).await?;
        async {
            let [mut form, mut bio, mut stable, mut pure, mut toxic] = endpoint.par()?;
            async { form.send::<Specimen<FORMULATION>>(formulation).await };
            async { bio.send::<Specimen<BIOASSAY>>(bioassay).await };
            async { stable.send::<Specimen<STABILITY>>(stability).await };
            async { pure.send::<Specimen<PURITY>>(purity).await };
            async { toxic.send::<Specimen<TOXICITY>>(toxicity).await };
        }
        .await?;
    }
}
