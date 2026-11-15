//! Actual two-patient treatment boundaries and one operating-room continuation.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::Endpoint;

#[hibana_runtime::scope]
pub async fn anesthesia<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, ANESTHESIA_SERVICE>,
    room: Endpoint<'r, OPERATING_ROOM>,
    domain: &D,
) -> ShellResult<()> {
    let diagnosis = imp::decode(endpoint.recv::<DiagnosisForAnesthesia>().await?)?;
    imp::actual_result(&diagnosis["result"], "patient_a")?;
    let admitted = endpoint.recv::<AnesthesiaIntent>().await?;
    endpoint.send::<AnesthesiaAdmitted>(&admitted).await?;
    let admitted = admitted.into_value();
    imp::patient(&admitted, "patient_a")?;
    imp::optional_text(&admitted, "type")?;
    endpoint.send::<AnesthesiaLoan>(room).await?;
    let arguments = endpoint.recv::<AnesthesiaRequest>().await?.into_value();
    imp::same(&arguments, &admitted)?;
    let result = domain
        .execute("ANESTHESIOLOGIST", "prepare_anesthesia", arguments)
        .await
        .map_err(ShellError::message)?;
    imp::actual_result(&result, "patient_a")?;
    imp::same(
        &result["data"]["inputs"]["assessment"],
        &diagnosis["findings"]["assessment"]["data"]["artifact_id"],
    )?;
    imp::same(
        &result["data"]["inputs"]["diagnosis:ER_DOCTOR_A"],
        &diagnosis["result"]["data"]["artifact_id"],
    )?;
    let bytes = imp::encode(&result)?;
    async {
        let [mut resource, mut file] = endpoint.par()?;
        async {
            resource.send::<AnesthesiaResult>(&bytes.as_slice()).await?;
        };
        async {
            file.send::<FileAnesthesiaResult>(&bytes.as_slice()).await?;
        };
        Ok::<(), ShellError>(())
    }
    .await?;
    let room = endpoint.recv::<AnesthesiaLoan>().await?;
    async {
        let [mut custody, mut preparation] = endpoint.par()?;
        async {
            custody.send::<PreparedRoom>(room).await?;
        };
        async {
            preparation
                .send::<PreparationForSurgery>(&bytes.as_slice())
                .await?;
        };
        Ok(())
    }
    .await?;
    Ok(())
}

#[hibana_runtime::scope]
pub async fn surgery<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, SURGERY_SERVICE>,
    domain: &D,
) -> ShellResult<()> {
    let diagnosis = imp::decode(endpoint.recv::<DiagnosisForSurgery>().await?)?;
    imp::actual_result(&diagnosis["result"], "patient_a")?;
    let (room, preparation) = async {
        let [mut custody, mut preparation] = endpoint.par()?;
        async { Ok(custody.recv::<PreparedRoom>().await?) };
        async { imp::decode(preparation.recv::<PreparationForSurgery>().await?) };
    }
    .await?;
    imp::actual_result(&preparation, "patient_a")?;
    imp::same(
        &preparation["data"]["inputs"]["diagnosis:ER_DOCTOR_A"],
        &diagnosis["result"]["data"]["artifact_id"],
    )?;
    let admitted = endpoint.recv::<SurgeryIntent>().await?;
    endpoint.send::<SurgeryAdmitted>(&admitted).await?;
    let admitted = admitted.into_value();
    imp::patient(&admitted, "patient_a")?;
    if admitted["procedure"] != "trauma_surgery" {
        return Err(ShellError::message("unexpected surgical procedure"));
    }
    endpoint.send::<SurgeryLoan>(room).await?;
    let arguments = endpoint.recv::<SurgeryRequest>().await?.into_value();
    imp::same(&arguments, &admitted)?;
    let result = domain
        .execute("SURGEON", "perform_surgery", arguments)
        .await
        .map_err(ShellError::message)?;
    imp::actual_result(&result, "patient_a")?;
    imp::same(
        &result["data"]["inputs"]["assessment"],
        &diagnosis["findings"]["assessment"]["data"]["artifact_id"],
    )?;
    imp::same(
        &result["data"]["inputs"]["diagnosis:ER_DOCTOR_A"],
        &diagnosis["result"]["data"]["artifact_id"],
    )?;
    imp::same(
        &result["data"]["inputs"]["anesthesia"],
        &preparation["data"]["artifact_id"],
    )?;
    let bytes = imp::encode(&result)?;
    async {
        let [mut resource, mut file] = endpoint.par()?;
        async {
            resource.send::<SurgeryResult>(&bytes.as_slice()).await?;
        };
        async {
            file.send::<FileSurgeryResult>(&bytes.as_slice()).await?;
        };
        Ok::<(), ShellError>(())
    }
    .await?;
    let _room = endpoint.recv::<SurgeryLoan>().await?;
    endpoint.send::<SurgeryForER>(&bytes.as_slice()).await?;
    Ok(())
}

#[hibana_runtime::scope]
pub async fn icu<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, ICU_SERVICE>,
    (bed, ventilator): (Endpoint<'r, ICU_BEDS>, Endpoint<'r, VENTILATOR>),
    domain: &D,
) -> ShellResult<()> {
    let diagnosis = imp::decode(endpoint.recv::<DiagnosisForIcu>().await?)?;
    imp::actual_result(&diagnosis["result"], "patient_b")?;
    let admitted = endpoint.recv::<IcuIntent>().await?;
    endpoint.send::<IcuAdmitted>(&admitted).await?;
    let admitted = admitted.into_value();
    imp::patient(&admitted, "patient_b")?;
    async {
        let [mut bed_branch, mut vent_branch] = endpoint.par()?;
        async {
            bed_branch.send::<ResourceLoan<ICU_BEDS>>(bed).await?;
        };
        async {
            vent_branch
                .send::<ResourceLoan<VENTILATOR>>(ventilator)
                .await?;
        };
        Ok(())
    }
    .await?;
    let (bed_request, vent_request) = async {
        let [mut bed_branch, mut vent_branch] = endpoint.par()?;
        async { Ok::<_, ShellError>(bed_branch.recv::<BedRequest>().await?.into_value()) };
        async { Ok::<_, ShellError>(vent_branch.recv::<VentilatorRequest>().await?.into_value()) };
    }
    .await?;
    imp::same(&bed_request, &admitted)?;
    imp::same(&vent_request, &admitted)?;
    let result = domain
        .execute("ICU_COORDINATOR", "manage_icu", admitted)
        .await
        .map_err(ShellError::message)?;
    imp::actual_result(&result, "patient_b")?;
    imp::same(
        &result["data"]["inputs"]["assessment"],
        &diagnosis["findings"]["assessment"]["data"]["artifact_id"],
    )?;
    imp::same(
        &result["data"]["inputs"]["diagnosis:ER_DOCTOR_B"],
        &diagnosis["result"]["data"]["artifact_id"],
    )?;
    let bytes = imp::encode(&result)?;
    async {
        let [mut bed_branch, mut vent_branch, mut file] = endpoint.par()?;
        async {
            bed_branch.send::<BedResult>(&bytes.as_slice()).await?;
        };
        async {
            vent_branch
                .send::<VentilatorResult>(&bytes.as_slice())
                .await?;
        };
        async {
            file.send::<FileIcuResult>(&bytes.as_slice()).await?;
        };
        Ok(())
    }
    .await?;
    async {
        let [mut bed_branch, mut vent_branch] = endpoint.par()?;
        async {
            drop(bed_branch.recv::<ResourceReturn<ICU_BEDS>>().await?);
        };
        async {
            drop(vent_branch.recv::<ResourceReturn<VENTILATOR>>().await?);
        };
        Ok(())
    }
    .await?;
    endpoint.send::<IcuForER>(&bytes.as_slice()).await?;
    Ok(())
}

#[hibana_runtime::scope]
pub async fn pharmacy<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, PHARMACY_SERVICE>,
    domain: &D,
) -> ShellResult<()> {
    let arguments = endpoint.recv::<ReviewRequest>().await?.into_value();
    imp::patient(&arguments, "patient_A_and_B")?;
    imp::optional_text(&arguments, "medications")?;
    let result = domain
        .execute("PHARMACIST", "review_prescriptions", arguments)
        .await
        .map_err(ShellError::message)?;
    imp::actual_result(&result, "patient_A_and_B")?;
    let bytes = imp::encode(&result)?;
    async {
        let [mut person, mut file, mut a, mut b] = endpoint.par()?;
        async {
            person.send::<ReviewResult>(&bytes.as_slice()).await?;
        };
        async {
            file.send::<FileReviewResult>(&bytes.as_slice()).await?;
        };
        async {
            a.send::<ReviewForA>(&bytes.as_slice()).await?;
        };
        async {
            b.send::<ReviewForB>(&bytes.as_slice()).await?;
        };
        Ok::<(), ShellError>(())
    }
    .await?;
    Ok(())
}
