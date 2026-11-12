//! The effect owner holds the actual synthesis, measurements and QC results.
use super::super::{
    global::*,
    imp::{self, QcReply},
};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::Endpoint;
use serde_json::Value;
use std::cell::Cell;

#[hibana_runtime::scope]
pub async fn run<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, FUME_SERVICE>,
    (mut instrument, chemistry): (Endpoint<'r, FUME_HOOD>, Endpoint<'r, CHEMISTRY>),
    (domain, actual): (&D, &Cell<Value>),
) -> ShellResult<()> {
    endpoint.send::<ChemAccess>(chemistry).await?;
    let synthesis = {
        let admitted = imp::decode(endpoint.recv::<Synthesis>().await?)?;
        if admitted.as_object().is_none_or(|object| object.len() != 1)
            || !admitted["compound"].is_string()
        {
            return Err(ShellError::message("expected actual synthesis compound"));
        }
        endpoint.send::<FumeLoan>(instrument).await?;
        let request = imp::decode(endpoint.recv::<SynthesisRequest>().await?)?;
        imp::same(&request, &admitted)?;
        let result = domain
            .execute("CHEMIST", "run_synthesis", request)
            .await
            .map_err(ShellError::message)?;
        imp::successful(&result)?;
        imp::same(&result["data"]["compound"], &admitted["compound"])?;
        let bytes = imp::encode(&result)?;
        endpoint.send::<InstrumentResult>(&bytes.as_slice()).await?;
        instrument = endpoint.recv::<FumeLoan>().await?;
        result
    };
    let mut analysis = imp::decode(endpoint.recv::<AnalysisProof>().await?)?;
    let purification = imp::decode(endpoint.recv::<PurificationProof>().await?)?;
    imp::same(
        &purification["data"]["compound"],
        &synthesis["data"]["compound"],
    )?;
    let branch = endpoint.offer().await?;
    let (mut result, mut biology) = match branch.label() {
        50 => {
            let selected = imp::decode(branch.recv::<FirstChemist>().await?)?;
            imp::qc_arguments(&selected, "CHEMIST")?;
            let ((result, returned), biology) = async {
                let [mut trials, mut qc] = endpoint.par()?;
                async {
                    let admitted = imp::decode(qc.recv::<QcAdmission>().await?)?;
                    imp::qc_admission(
                        &admitted,
                        &selected,
                        (&synthesis, &purification, &analysis),
                    )?;
                    qc.send::<FumeLoan>(instrument).await?;
                    let (spec, fume) = async {
                        let [mut spec, mut fume] = qc.par()?;
                        async { imp::decode(spec.recv::<QcSpecRequest>().await?) };
                        async { imp::decode(fume.recv::<QcFumeRequest>().await?) };
                    }
                    .await?;
                    imp::same(&spec, &admitted)?;
                    imp::same(&fume, &admitted)?;
                    let result = domain
                        .execute(
                            "QC_CONTROLLER",
                            "verify_result",
                            admitted["arguments"].clone(),
                        )
                        .await
                        .map_err(ShellError::message)?;
                    imp::qc_result(&result, &selected)?;
                    let bytes = imp::encode(&result)?;
                    async {
                        let [mut spec, mut fume] = qc.par()?;
                        async {
                            spec.send::<QcSpecResult>(&bytes.as_slice()).await?;
                        };
                        async {
                            fume.send::<QcFumeResult>(&bytes.as_slice()).await?;
                        };
                        Ok(())
                    }
                    .await?;
                    let instrument = qc.recv::<FumeLoan>().await?;
                    Ok((result, instrument))
                };
                async {
                    loop {
                        let branch = trials.offer().await?;
                        match branch.label() {
                            35 => break Ok(imp::decode(branch.recv::<BioFumePassed>().await?)?),
                            36 => {
                                branch.recv::<BioFumeFailed>().await?;
                            }
                            label => {
                                return Err(ShellError::message(format!(
                                    "unexpected initial trial result {label}"
                                )));
                            }
                        }
                    }
                };
            }
            .await?;
            instrument = returned;
            (result, biology)
        }
        51 => {
            let selected = imp::decode(branch.recv::<FirstBiologist>().await?)?;
            imp::qc_arguments(&selected, "BIOLOGIST")?;
            let biology = loop {
                let branch = endpoint.offer().await?;
                match branch.label() {
                    35 => break imp::decode(branch.recv::<BioFumePassed>().await?)?,
                    36 => {
                        branch.recv::<BioFumeFailed>().await?;
                    }
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected initial trial result {label}"
                        )));
                    }
                }
            };
            let admitted = imp::decode(endpoint.recv::<QcAdmission>().await?)?;
            imp::qc_admission(&admitted, &selected, (&synthesis, &purification, &biology))?;
            endpoint.send::<FumeLoan>(instrument).await?;
            let (spec, fume) = async {
                let [mut spec, mut fume] = endpoint.par()?;
                async { imp::decode(spec.recv::<QcSpecRequest>().await?) };
                async { imp::decode(fume.recv::<QcFumeRequest>().await?) };
            }
            .await?;
            imp::same(&spec, &admitted)?;
            imp::same(&fume, &admitted)?;
            let result = domain
                .execute(
                    "QC_CONTROLLER",
                    "verify_result",
                    admitted["arguments"].clone(),
                )
                .await
                .map_err(ShellError::message)?;
            imp::qc_result(&result, &selected)?;
            let bytes = imp::encode(&result)?;
            async {
                let [mut spec, mut fume] = endpoint.par()?;
                async {
                    spec.send::<QcSpecResult>(&bytes.as_slice()).await?;
                };
                async {
                    fume.send::<QcFumeResult>(&bytes.as_slice()).await?;
                };
                Ok(())
            }
            .await?;
            instrument = endpoint.recv::<FumeLoan>().await?;
            (result, biology)
        }
        label => {
            return Err(ShellError::message(format!(
                "unexpected first QC selection {label}"
            )));
        }
    };
    loop {
        actual.set(result.clone());
        let (chemistry, biological) = async {
            let [mut chemist, mut biologist] = endpoint.par()?;
            async { Ok(chemist.recv::<ChemAccess>().await?) };
            async { Ok(biologist.recv::<BioAccess>().await?) };
        }
        .await?;
        let bytes = imp::encode(&result)?;
        match (
            result["success"].as_bool(),
            result["data"]["scientist"].as_str(),
        ) {
            (Some(true), Some("CHEMIST")) => {
                let reply = QcReply::<CHEMIST, true>::from_bytes(&bytes)?;
                endpoint.send::<ChemistApproved>(&reply).await?;
            }
            (Some(false), Some("CHEMIST")) => {
                let reply = QcReply::<CHEMIST, false>::from_bytes(&bytes)?;
                endpoint.send::<ChemistRejected>(&reply).await?;
            }
            (Some(true), Some("BIOLOGIST")) => {
                let reply = QcReply::<BIOLOGIST, true>::from_bytes(&bytes)?;
                endpoint.send::<BiologistApproved>(&reply).await?;
            }
            (Some(false), Some("BIOLOGIST")) => {
                let reply = QcReply::<BIOLOGIST, false>::from_bytes(&bytes)?;
                endpoint.send::<BiologistRejected>(&reply).await?;
            }
            _ => return Err(ShellError::message("missing actual QC subject or verdict")),
        }
        async {
            let [mut chemist, mut biologist] = endpoint.par()?;
            async {
                chemist.send::<ChemAccess>(chemistry).await?;
            };
            async {
                biologist.send::<BioAccess>(biological).await?;
            };
            Ok(())
        }
        .await?;
        if result["success"] == true {
            break;
        }
        match result["data"]["scientist"].as_str() {
            Some("CHEMIST") => {
                analysis = imp::decode(endpoint.recv::<AnalysisProof>().await?)?;
                let branch = endpoint.offer().await?;
                let selected = match branch.label() {
                    52 => {
                        let selected = imp::decode(branch.recv::<NextChemist>().await?)?;
                        imp::qc_arguments(&selected, "CHEMIST")?;
                        selected
                    }
                    53 => {
                        let selected = imp::decode(branch.recv::<NextBiologist>().await?)?;
                        imp::qc_arguments(&selected, "BIOLOGIST")?;
                        selected
                    }
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected next QC selection {label}"
                        )));
                    }
                };
                let admitted = imp::decode(endpoint.recv::<QcAdmission>().await?)?;
                let evidence = if selected["scientist"] == "CHEMIST" {
                    &analysis
                } else {
                    &biology
                };
                imp::qc_admission(&admitted, &selected, (&synthesis, &purification, evidence))?;
                endpoint.send::<FumeLoan>(instrument).await?;
                let (spec, fume) = async {
                    let [mut spec, mut fume] = endpoint.par()?;
                    async { imp::decode(spec.recv::<QcSpecRequest>().await?) };
                    async { imp::decode(fume.recv::<QcFumeRequest>().await?) };
                }
                .await?;
                imp::same(&spec, &admitted)?;
                imp::same(&fume, &admitted)?;
                result = domain
                    .execute(
                        "QC_CONTROLLER",
                        "verify_result",
                        admitted["arguments"].clone(),
                    )
                    .await
                    .map_err(ShellError::message)?;
                imp::qc_result(&result, &selected)?;
                let bytes = imp::encode(&result)?;
                async {
                    let [mut spec, mut fume] = endpoint.par()?;
                    async {
                        spec.send::<QcSpecResult>(&bytes.as_slice()).await?;
                    };
                    async {
                        fume.send::<QcFumeResult>(&bytes.as_slice()).await?;
                    };
                    Ok(())
                }
                .await?;
                instrument = endpoint.recv::<FumeLoan>().await?;
            }
            Some("BIOLOGIST") => {
                let branch = endpoint.offer().await?;
                (result, biology) = match branch.label() {
                    52 => {
                        let selected = imp::decode(branch.recv::<NextChemist>().await?)?;
                        imp::qc_arguments(&selected, "CHEMIST")?;
                        let ((result, returned), biology) = async {
                            let [mut trials, mut qc] = endpoint.par()?;
                            async {
                                let admitted = imp::decode(qc.recv::<QcAdmission>().await?)?;
                                imp::qc_admission(
                                    &admitted,
                                    &selected,
                                    (&synthesis, &purification, &analysis),
                                )?;
                                qc.send::<FumeLoan>(instrument).await?;
                                let (spec, fume) = async {
                                    let [mut spec, mut fume] = qc.par()?;
                                    async { imp::decode(spec.recv::<QcSpecRequest>().await?) };
                                    async { imp::decode(fume.recv::<QcFumeRequest>().await?) };
                                }
                                .await?;
                                imp::same(&spec, &admitted)?;
                                imp::same(&fume, &admitted)?;
                                let result = domain
                                    .execute(
                                        "QC_CONTROLLER",
                                        "verify_result",
                                        admitted["arguments"].clone(),
                                    )
                                    .await
                                    .map_err(ShellError::message)?;
                                imp::qc_result(&result, &selected)?;
                                let bytes = imp::encode(&result)?;
                                async {
                                    let [mut spec, mut fume] = qc.par()?;
                                    async {
                                        spec.send::<QcSpecResult>(&bytes.as_slice()).await?;
                                    };
                                    async {
                                        fume.send::<QcFumeResult>(&bytes.as_slice()).await?;
                                    };
                                    Ok(())
                                }
                                .await?;
                                let instrument = qc.recv::<FumeLoan>().await?;
                                Ok((result, instrument))
                            };
                            async {
                                loop {
                                    let branch = trials.offer().await?;
                                    match branch.label() {
                                        103 => {
                                            break Ok(imp::decode(
                                                branch.recv::<RetestFumePassed>().await?,
                                            )?);
                                        }
                                        104 => {
                                            branch.recv::<RetestFumeFailed>().await?;
                                        }
                                        label => {
                                            return Err(ShellError::message(format!(
                                                "unexpected revised trial result {label}"
                                            )));
                                        }
                                    }
                                }
                            };
                        }
                        .await?;
                        instrument = returned;
                        (result, biology)
                    }
                    53 => {
                        let selected = imp::decode(branch.recv::<NextBiologist>().await?)?;
                        imp::qc_arguments(&selected, "BIOLOGIST")?;
                        let biology = loop {
                            let branch = endpoint.offer().await?;
                            match branch.label() {
                                103 => {
                                    break imp::decode(branch.recv::<RetestFumePassed>().await?)?;
                                }
                                104 => {
                                    branch.recv::<RetestFumeFailed>().await?;
                                }
                                label => {
                                    return Err(ShellError::message(format!(
                                        "unexpected revised trial result {label}"
                                    )));
                                }
                            }
                        };
                        let admitted = imp::decode(endpoint.recv::<QcAdmission>().await?)?;
                        imp::qc_admission(
                            &admitted,
                            &selected,
                            (&synthesis, &purification, &biology),
                        )?;
                        endpoint.send::<FumeLoan>(instrument).await?;
                        let (spec, fume) = async {
                            let [mut spec, mut fume] = endpoint.par()?;
                            async { imp::decode(spec.recv::<QcSpecRequest>().await?) };
                            async { imp::decode(fume.recv::<QcFumeRequest>().await?) };
                        }
                        .await?;
                        imp::same(&spec, &admitted)?;
                        imp::same(&fume, &admitted)?;
                        let result = domain
                            .execute(
                                "QC_CONTROLLER",
                                "verify_result",
                                admitted["arguments"].clone(),
                            )
                            .await
                            .map_err(ShellError::message)?;
                        imp::qc_result(&result, &selected)?;
                        let bytes = imp::encode(&result)?;
                        async {
                            let [mut spec, mut fume] = endpoint.par()?;
                            async {
                                spec.send::<QcSpecResult>(&bytes.as_slice()).await?;
                            };
                            async {
                                fume.send::<QcFumeResult>(&bytes.as_slice()).await?;
                            };
                            Ok(())
                        }
                        .await?;
                        instrument = endpoint.recv::<FumeLoan>().await?;
                        (result, biology)
                    }
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected next QC selection {label}"
                        )));
                    }
                };
            }
            _ => return Err(ShellError::message("unknown rejected QC subject")),
        }
    }
    // The first approval's Endpoint now stays at QC. Each remaining decision
    // receives and returns only the other candidate's actual continuation.
    loop {
        let branch = endpoint.offer().await?;
        let selected = match branch.label() {
            54 => {
                let selected = imp::decode(branch.recv::<LastChemist>().await?)?;
                imp::qc_arguments(&selected, "CHEMIST")?;
                selected
            }
            55 => {
                let selected = imp::decode(branch.recv::<LastBiologist>().await?)?;
                imp::qc_arguments(&selected, "BIOLOGIST")?;
                selected
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected remaining QC selection {label}"
                )));
            }
        };
        let admitted = imp::decode(endpoint.recv::<QcAdmission>().await?)?;
        let evidence = if selected["scientist"] == "CHEMIST" {
            &analysis
        } else {
            &biology
        };
        imp::qc_admission(&admitted, &selected, (&synthesis, &purification, evidence))?;
        endpoint.send::<FumeLoan>(instrument).await?;
        let (spec, fume) = async {
            let [mut spec, mut fume] = endpoint.par()?;
            async { imp::decode(spec.recv::<QcSpecRequest>().await?) };
            async { imp::decode(fume.recv::<QcFumeRequest>().await?) };
        }
        .await?;
        imp::same(&spec, &admitted)?;
        imp::same(&fume, &admitted)?;
        result = domain
            .execute(
                "QC_CONTROLLER",
                "verify_result",
                admitted["arguments"].clone(),
            )
            .await
            .map_err(ShellError::message)?;
        imp::qc_result(&result, &selected)?;
        let bytes = imp::encode(&result)?;
        async {
            let [mut spec, mut fume] = endpoint.par()?;
            async {
                spec.send::<QcSpecResult>(&bytes.as_slice()).await?;
            };
            async {
                fume.send::<QcFumeResult>(&bytes.as_slice()).await?;
            };
            Ok(())
        }
        .await?;
        instrument = endpoint.recv::<FumeLoan>().await?;
        actual.set(result.clone());
        match selected["scientist"].as_str() {
            Some("CHEMIST") => {
                let candidate = endpoint.recv::<ChemAccess>().await?;
                match result["success"].as_bool() {
                    Some(true) => {
                        let reply = QcReply::<CHEMIST, true>::from_bytes(&bytes)?;
                        endpoint.send::<LastChemistApproved>(&reply).await?;
                        endpoint.send::<ChemAccess>(candidate).await?;
                        break;
                    }
                    Some(false) => {
                        let reply = QcReply::<CHEMIST, false>::from_bytes(&bytes)?;
                        endpoint.send::<LastChemistRejected>(&reply).await?;
                        endpoint.send::<ChemAccess>(candidate).await?;
                        analysis = imp::decode(endpoint.recv::<AnalysisProof>().await?)?;
                    }
                    None => return Err(ShellError::message("missing actual QC decision")),
                }
            }
            Some("BIOLOGIST") => {
                let candidate = endpoint.recv::<BioAccess>().await?;
                match result["success"].as_bool() {
                    Some(true) => {
                        let reply = QcReply::<BIOLOGIST, true>::from_bytes(&bytes)?;
                        endpoint.send::<LastBiologistApproved>(&reply).await?;
                        endpoint.send::<BioAccess>(candidate).await?;
                        break;
                    }
                    Some(false) => {
                        let reply = QcReply::<BIOLOGIST, false>::from_bytes(&bytes)?;
                        endpoint.send::<LastBiologistRejected>(&reply).await?;
                        endpoint.send::<BioAccess>(candidate).await?;
                        biology = loop {
                            let branch = endpoint.offer().await?;
                            match branch.label() {
                                113 => {
                                    break imp::decode(
                                        branch.recv::<LastRetestFumePassed>().await?,
                                    )?;
                                }
                                114 => {
                                    branch.recv::<LastRetestFumeFailed>().await?;
                                }
                                label => {
                                    return Err(ShellError::message(format!(
                                        "unexpected remaining trial result {label}"
                                    )));
                                }
                            }
                        };
                    }
                    None => return Err(ShellError::message("missing actual QC decision")),
                }
            }
            _ => return Err(ShellError::message("unknown remaining QC subject")),
        }
    }
    drop(instrument);
    Ok(())
}
