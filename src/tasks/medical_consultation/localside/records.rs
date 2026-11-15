//! Own the completed test records and publish their actual consultation data.
use super::super::global::*;
use crate::{
    imp::{json, patient},
    shell::{ShellError, ShellResult},
};
use hibana::Endpoint;

#[hibana_runtime::scope]
pub async fn run(endpoint: &mut Endpoint<RECORDS>) -> ShellResult<()> {
    let ((_general_panel, general_data), (_cardiac_panel, cardiac_data)) = async {
        let [mut general, mut cardiac] = endpoint.par()?;
        async {
            let data = json::decode(general.recv::<GeneralCompletedData>().await?)?;
            if !patient::panel_complete(&data["tests"], &["vitals", "basic_labs"])? {
                return Err(ShellError::message("general record is incomplete"));
            }
            patient::actual_result(&data["attempt"])?;
            if !data["tests"]
                .as_object()
                .is_some_and(|tests| tests.values().any(|result| result == &data["attempt"]))
            {
                return Err(ShellError::message(
                    "last general result is absent from its record",
                ));
            }
            let panel = general.recv::<GeneralPanelCompleted>().await?;
            let result = json::encode(&data["attempt"])?;
            general
                .send::<FileGeneralResult>(&result.as_slice())
                .await?;
            Ok::<_, ShellError>((panel, data))
        };
        async {
            let data = json::decode(cardiac.recv::<CardiacCompletedData>().await?)?;
            if !patient::panel_complete(&data["tests"], &["ecg", "cardiac_enzymes"])? {
                return Err(ShellError::message("cardiac record is incomplete"));
            }
            patient::actual_result(&data["attempt"])?;
            if !data["tests"]
                .as_object()
                .is_some_and(|tests| tests.values().any(|result| result == &data["attempt"]))
            {
                return Err(ShellError::message(
                    "last cardiac result is absent from its record",
                ));
            }
            let panel = cardiac.recv::<CardiacPanelCompleted>().await?;
            let result = json::encode(&data["attempt"])?;
            cardiac
                .send::<FileCardiacResult>(&result.as_slice())
                .await?;
            Ok::<_, ShellError>((panel, data))
        };
    }
    .await?;
    let summary = json::decode(endpoint.recv::<CompletedConsultation>().await?)?;
    let findings = &summary["clinical"]["findings"];
    json::same(&findings["tests"], &general_data["tests"])?;
    json::same(&findings["cardiac_tests"], &cardiac_data["tests"])?;
    patient::actual_result(&findings["assessment"])?;
    patient::actual_result(&findings["imaging"])?;
    patient::actual_result(&summary["clinical"]["diagnosis"])?;
    patient::actual_result(&summary["medication_review"])?;
    let bytes = json::encode(&summary)?;
    endpoint.send::<Summary>(&bytes.as_slice()).await?;
    Ok(())
}
