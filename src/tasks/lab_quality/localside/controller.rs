//! Actual file requests select the QC subject; affine candidates carry permission.
use super::super::{
    global::*,
    imp::{self, QcReply},
};
use crate::shell::{ShellError, ShellResult};
use hibana::{Endpoint, g::Message};
use serde_json::{Value, json};
#[hibana_runtime::scope]
pub async fn run(endpoint: &mut Endpoint<QC>) -> ShellResult<()> {
    let mut chemistry = endpoint.recv::<ChemAccess>().await?;
    let mut chemist_materials = Value::Null;
    let branch = endpoint.offer().await?;
    let (mut result, mut biological) = match branch.label() {
        FileQcChemist::LOGICAL_LABEL => {
            let selected = branch.recv::<FileQcChemist>().await?.into_value();
            let bytes = imp::encode(&selected)?;
            endpoint.send::<FirstChemist>(&bytes.as_slice()).await?;
            endpoint.send::<FirstChemist>(&bytes.as_slice()).await?;
            endpoint.send::<FirstChemist>(&bytes.as_slice()).await?;
            endpoint.send::<FirstChemist>(&bytes.as_slice()).await?;

            let (biological, (result, report)) = async {
                let [mut trials, mut verify] = endpoint.par()?;
                async { trials.recv::<BioAccess>().await };
                async {
                    let report = imp::decode(verify.recv::<ChemistReport>().await?)?;
                    let inputs = imp::qc_inputs(&selected, &report)?;
                    let bytes = imp::encode(&json!({"arguments":selected,"materials":report}))?;
                    chemistry.send::<QcAdmission>(&bytes.as_slice()).await?;
                    let (mut spec, mut fume) = async {
                        let [mut a, mut b] = verify.par()?;
                        async { a.recv::<SpecLoan>().await };
                        async { b.recv::<FumeLoan>().await };
                    }
                    .await?;
                    let (spec_result, fume_result) = async {
                        async {
                            spec.send::<QcSpecRequest>(&bytes.as_slice()).await?;
                            imp::decode(spec.recv::<QcSpecResult>().await?)
                        };
                        async {
                            fume.send::<QcFumeRequest>(&bytes.as_slice()).await?;
                            imp::decode(fume.recv::<QcFumeResult>().await?)
                        };
                    }
                    .await?;
                    imp::same_value(&spec_result, &fume_result)?;
                    imp::qc_result(&spec_result, &selected, &inputs)?;
                    async {
                        let [mut a, mut b] = verify.par()?;
                        async {
                            a.send::<SpecLoan>(spec).await?;
                        };
                        async {
                            b.send::<FumeLoan>(fume).await?;
                        };
                        Ok(())
                    }
                    .await?;
                    ShellResult::Ok((spec_result, report))
                };
            }
            .await?;
            chemist_materials = report;
            (result, biological)
        }
        FileQcBiologist::LOGICAL_LABEL => {
            let selected = branch.recv::<FileQcBiologist>().await?.into_value();
            let bytes = imp::encode(&selected)?;
            endpoint.send::<FirstBiologist>(&bytes.as_slice()).await?;
            endpoint.send::<FirstBiologist>(&bytes.as_slice()).await?;
            endpoint.send::<FirstBiologist>(&bytes.as_slice()).await?;
            endpoint.send::<FirstBiologist>(&bytes.as_slice()).await?;

            let mut biological = endpoint.recv::<BioAccess>().await?;
            let (result, _report) = async {
                let report = imp::decode(endpoint.recv::<BiologyReport>().await?)?;
                let inputs = imp::qc_inputs(&selected, &report)?;
                let bytes = imp::encode(&json!({"arguments":selected,"materials":report}))?;
                biological.send::<QcAdmission>(&bytes.as_slice()).await?;
                let (mut spec, mut fume) = async {
                    let [mut a, mut b] = endpoint.par()?;
                    async { a.recv::<SpecLoan>().await };
                    async { b.recv::<FumeLoan>().await };
                }
                .await?;
                let (spec_result, fume_result) = async {
                    async {
                        spec.send::<QcSpecRequest>(&bytes.as_slice()).await?;
                        imp::decode(spec.recv::<QcSpecResult>().await?)
                    };
                    async {
                        fume.send::<QcFumeRequest>(&bytes.as_slice()).await?;
                        imp::decode(fume.recv::<QcFumeResult>().await?)
                    };
                }
                .await?;
                imp::same_value(&spec_result, &fume_result)?;
                imp::qc_result(&spec_result, &selected, &inputs)?;
                async {
                    let [mut a, mut b] = endpoint.par()?;
                    async {
                        a.send::<SpecLoan>(spec).await?;
                    };
                    async {
                        b.send::<FumeLoan>(fume).await?;
                    };
                    Ok(())
                }
                .await?;
                ShellResult::Ok((spec_result, report))
            }
            .await?;
            (result, biological)
        }
        label => {
            return Err(ShellError::message(format!(
                "unexpected first QC request {label}"
            )));
        }
    };
    let first_approval = loop {
        async {
            let [mut a, mut b] = endpoint.par()?;
            async {
                a.send::<ChemAccess>(chemistry).await?;
            };
            async {
                b.send::<BioAccess>(biological).await?;
            };
            Ok(())
        }
        .await?;
        let branch = endpoint.offer().await?;
        let confirmed = match branch.label() {
            ChemistApproved::LOGICAL_LABEL => imp::decode(branch.recv::<ChemistApproved>().await?)?,
            ChemistRejected::LOGICAL_LABEL => imp::decode(branch.recv::<ChemistRejected>().await?)?,
            BiologistApproved::LOGICAL_LABEL => {
                imp::decode(branch.recv::<BiologistApproved>().await?)?
            }
            BiologistRejected::LOGICAL_LABEL => {
                imp::decode(branch.recv::<BiologistRejected>().await?)?
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected QC verdict {label}"
                )));
            }
        };
        imp::same_value(&confirmed, &result)?;
        (chemistry, biological) = async {
            let [mut a, mut b] = endpoint.par()?;
            async { a.recv::<ChemAccess>().await };
            async { b.recv::<BioAccess>().await };
        }
        .await?;
        let bytes = imp::encode(&confirmed)?;
        match (
            confirmed["data"]["scientist"].as_str(),
            confirmed["success"].as_bool(),
        ) {
            (Some("CHEMIST"), Some(true)) => {
                let reply = QcReply::<CHEMIST, true>::from_bytes(&bytes)?;
                endpoint.send::<ChemistApproved>(&reply).await?;
                endpoint.send::<ChemistApproved>(&reply).await?;
                endpoint.send::<ChemistApproved>(&reply).await?;
                endpoint.send::<ChemistApproved>(&reply).await?;
                endpoint
                    .send::<FileQcChemistResult>(&bytes.as_slice())
                    .await?;
            }
            (Some("CHEMIST"), Some(false)) => {
                let reply = QcReply::<CHEMIST, false>::from_bytes(&bytes)?;
                endpoint.send::<ChemistRejected>(&reply).await?;
                endpoint.send::<ChemistRejected>(&reply).await?;
                endpoint.send::<ChemistRejected>(&reply).await?;
                endpoint.send::<ChemistRejected>(&reply).await?;
                endpoint
                    .send::<FileQcChemistResult>(&bytes.as_slice())
                    .await?;
            }
            (Some("BIOLOGIST"), Some(true)) => {
                let reply = QcReply::<BIOLOGIST, true>::from_bytes(&bytes)?;
                endpoint.send::<BiologistApproved>(&reply).await?;
                endpoint.send::<BiologistApproved>(&reply).await?;
                endpoint.send::<BiologistApproved>(&reply).await?;
                endpoint.send::<BiologistApproved>(&reply).await?;
                endpoint
                    .send::<FileQcBiologistResult>(&bytes.as_slice())
                    .await?;
            }
            (Some("BIOLOGIST"), Some(false)) => {
                let reply = QcReply::<BIOLOGIST, false>::from_bytes(&bytes)?;
                endpoint.send::<BiologistRejected>(&reply).await?;
                endpoint.send::<BiologistRejected>(&reply).await?;
                endpoint.send::<BiologistRejected>(&reply).await?;
                endpoint.send::<BiologistRejected>(&reply).await?;
                endpoint
                    .send::<FileQcBiologistResult>(&bytes.as_slice())
                    .await?;
            }
            _ => return Err(ShellError::message("missing actual QC subject or verdict")),
        }
        if confirmed["success"] == true {
            break confirmed;
        }
        match confirmed["data"]["scientist"].as_str() {
            Some("CHEMIST") => {
                endpoint.send::<ChemAccess>(chemistry).await?;
                chemistry = endpoint.recv::<ChemAccess>().await?;

                let branch = endpoint.offer().await?;
                let selected = match branch.label() {
                    FileQcChemist::LOGICAL_LABEL => {
                        let selected = branch.recv::<FileQcChemist>().await?.into_value();
                        let bytes = imp::encode(&selected)?;
                        endpoint.send::<NextChemist>(&bytes.as_slice()).await?;
                        endpoint.send::<NextChemist>(&bytes.as_slice()).await?;
                        endpoint.send::<NextChemist>(&bytes.as_slice()).await?;
                        endpoint.send::<NextChemist>(&bytes.as_slice()).await?;

                        selected
                    }
                    FileQcBiologist::LOGICAL_LABEL => {
                        let selected = branch.recv::<FileQcBiologist>().await?.into_value();
                        let bytes = imp::encode(&selected)?;
                        endpoint.send::<NextBiologist>(&bytes.as_slice()).await?;
                        endpoint.send::<NextBiologist>(&bytes.as_slice()).await?;
                        endpoint.send::<NextBiologist>(&bytes.as_slice()).await?;
                        endpoint.send::<NextBiologist>(&bytes.as_slice()).await?;

                        selected
                    }
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected QC request {label}"
                        )));
                    }
                };

                if selected["scientist"] == "CHEMIST" {
                    let (next, report) = async {
                        let report = imp::decode(endpoint.recv::<ChemistReport>().await?)?;
                        let inputs = imp::qc_inputs(&selected, &report)?;
                        let bytes = imp::encode(&json!({"arguments":selected,"materials":report}))?;
                        chemistry.send::<QcAdmission>(&bytes.as_slice()).await?;
                        let (mut spec, mut fume) = async {
                            let [mut a, mut b] = endpoint.par()?;
                            async { a.recv::<SpecLoan>().await };
                            async { b.recv::<FumeLoan>().await };
                        }
                        .await?;
                        let (spec_result, fume_result) = async {
                            async {
                                spec.send::<QcSpecRequest>(&bytes.as_slice()).await?;
                                imp::decode(spec.recv::<QcSpecResult>().await?)
                            };
                            async {
                                fume.send::<QcFumeRequest>(&bytes.as_slice()).await?;
                                imp::decode(fume.recv::<QcFumeResult>().await?)
                            };
                        }
                        .await?;
                        imp::same_value(&spec_result, &fume_result)?;
                        imp::qc_result(&spec_result, &selected, &inputs)?;
                        async {
                            let [mut a, mut b] = endpoint.par()?;
                            async {
                                a.send::<SpecLoan>(spec).await?;
                            };
                            async {
                                b.send::<FumeLoan>(fume).await?;
                            };
                            Ok(())
                        }
                        .await?;
                        ShellResult::Ok((spec_result, report))
                    }
                    .await?;
                    result = next;
                    chemist_materials = report;
                } else {
                    let (next, _report) = async {
                        let report = imp::decode(endpoint.recv::<BiologyReport>().await?)?;
                        let inputs = imp::qc_inputs(&selected, &report)?;
                        let bytes = imp::encode(&json!({"arguments":selected,"materials":report}))?;
                        biological.send::<QcAdmission>(&bytes.as_slice()).await?;
                        let (mut spec, mut fume) = async {
                            let [mut a, mut b] = endpoint.par()?;
                            async { a.recv::<SpecLoan>().await };
                            async { b.recv::<FumeLoan>().await };
                        }
                        .await?;
                        let (spec_result, fume_result) = async {
                            async {
                                spec.send::<QcSpecRequest>(&bytes.as_slice()).await?;
                                imp::decode(spec.recv::<QcSpecResult>().await?)
                            };
                            async {
                                fume.send::<QcFumeRequest>(&bytes.as_slice()).await?;
                                imp::decode(fume.recv::<QcFumeResult>().await?)
                            };
                        }
                        .await?;
                        imp::same_value(&spec_result, &fume_result)?;
                        imp::qc_result(&spec_result, &selected, &inputs)?;
                        async {
                            let [mut a, mut b] = endpoint.par()?;
                            async {
                                a.send::<SpecLoan>(spec).await?;
                            };
                            async {
                                b.send::<FumeLoan>(fume).await?;
                            };
                            Ok(())
                        }
                        .await?;
                        ShellResult::Ok((spec_result, report))
                    }
                    .await?;
                    result = next;
                }
            }
            Some("BIOLOGIST") => {
                endpoint.send::<BioAccess>(biological).await?;

                let branch = endpoint.offer().await?;
                let selected = match branch.label() {
                    FileQcChemist::LOGICAL_LABEL => {
                        let selected = branch.recv::<FileQcChemist>().await?.into_value();
                        let bytes = imp::encode(&selected)?;
                        endpoint.send::<NextChemist>(&bytes.as_slice()).await?;
                        endpoint.send::<NextChemist>(&bytes.as_slice()).await?;
                        endpoint.send::<NextChemist>(&bytes.as_slice()).await?;
                        endpoint.send::<NextChemist>(&bytes.as_slice()).await?;

                        selected
                    }
                    FileQcBiologist::LOGICAL_LABEL => {
                        let selected = branch.recv::<FileQcBiologist>().await?.into_value();
                        let bytes = imp::encode(&selected)?;
                        endpoint.send::<NextBiologist>(&bytes.as_slice()).await?;
                        endpoint.send::<NextBiologist>(&bytes.as_slice()).await?;
                        endpoint.send::<NextBiologist>(&bytes.as_slice()).await?;
                        endpoint.send::<NextBiologist>(&bytes.as_slice()).await?;

                        selected
                    }
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected QC request {label}"
                        )));
                    }
                };

                if selected["scientist"] == "CHEMIST" {
                    let (returned, (next, report)) = async {
                        let [mut trials, mut verify] = endpoint.par()?;
                        async { trials.recv::<BioAccess>().await };
                        async {
                            let report = imp::decode(verify.recv::<ChemistReport>().await?)?;
                            let inputs = imp::qc_inputs(&selected, &report)?;
                            let bytes =
                                imp::encode(&json!({"arguments":selected,"materials":report}))?;
                            chemistry.send::<QcAdmission>(&bytes.as_slice()).await?;
                            let (mut spec, mut fume) = async {
                                let [mut a, mut b] = verify.par()?;
                                async { a.recv::<SpecLoan>().await };
                                async { b.recv::<FumeLoan>().await };
                            }
                            .await?;
                            let (spec_result, fume_result) = async {
                                async {
                                    spec.send::<QcSpecRequest>(&bytes.as_slice()).await?;
                                    imp::decode(spec.recv::<QcSpecResult>().await?)
                                };
                                async {
                                    fume.send::<QcFumeRequest>(&bytes.as_slice()).await?;
                                    imp::decode(fume.recv::<QcFumeResult>().await?)
                                };
                            }
                            .await?;
                            imp::same_value(&spec_result, &fume_result)?;
                            imp::qc_result(&spec_result, &selected, &inputs)?;
                            async {
                                let [mut a, mut b] = verify.par()?;
                                async {
                                    a.send::<SpecLoan>(spec).await?;
                                };
                                async {
                                    b.send::<FumeLoan>(fume).await?;
                                };
                                Ok(())
                            }
                            .await?;
                            ShellResult::Ok((spec_result, report))
                        };
                    }
                    .await?;
                    biological = returned;
                    result = next;
                    chemist_materials = report;
                } else {
                    biological = endpoint.recv::<BioAccess>().await?;
                    let (next, _report) = async {
                        let report = imp::decode(endpoint.recv::<BiologyReport>().await?)?;
                        let inputs = imp::qc_inputs(&selected, &report)?;
                        let bytes = imp::encode(&json!({"arguments":selected,"materials":report}))?;
                        biological.send::<QcAdmission>(&bytes.as_slice()).await?;
                        let (mut spec, mut fume) = async {
                            let [mut a, mut b] = endpoint.par()?;
                            async { a.recv::<SpecLoan>().await };
                            async { b.recv::<FumeLoan>().await };
                        }
                        .await?;
                        let (spec_result, fume_result) = async {
                            async {
                                spec.send::<QcSpecRequest>(&bytes.as_slice()).await?;
                                imp::decode(spec.recv::<QcSpecResult>().await?)
                            };
                            async {
                                fume.send::<QcFumeRequest>(&bytes.as_slice()).await?;
                                imp::decode(fume.recv::<QcFumeResult>().await?)
                            };
                        }
                        .await?;
                        imp::same_value(&spec_result, &fume_result)?;
                        imp::qc_result(&spec_result, &selected, &inputs)?;
                        async {
                            let [mut a, mut b] = endpoint.par()?;
                            async {
                                a.send::<SpecLoan>(spec).await?;
                            };
                            async {
                                b.send::<FumeLoan>(fume).await?;
                            };
                            Ok(())
                        }
                        .await?;
                        ShellResult::Ok((spec_result, report))
                    }
                    .await?;
                    result = next;
                }
            }
            _ => return Err(ShellError::message("unknown actual rejected subject")),
        }
    };
    let last_approval = match first_approval["data"]["scientist"].as_str() {
        Some("CHEMIST") => {
            endpoint.send::<ChemAccess>(chemistry).await?;
            loop {
                let selected = endpoint.recv::<FileQcBiologist>().await?.into_value();
                let bytes = imp::encode(&selected)?;
                endpoint.send::<LastBiologist>(&bytes.as_slice()).await?;
                endpoint.send::<LastBiologist>(&bytes.as_slice()).await?;
                endpoint.send::<LastBiologist>(&bytes.as_slice()).await?;

                let (result, report) = async {
                    let report = imp::decode(endpoint.recv::<BiologyReport>().await?)?;
                    let inputs = imp::qc_inputs(&selected, &report)?;
                    let bytes = imp::encode(&json!({"arguments":selected,"materials":report}))?;
                    biological.send::<QcAdmission>(&bytes.as_slice()).await?;
                    let (mut spec, mut fume) = async {
                        let [mut a, mut b] = endpoint.par()?;
                        async { a.recv::<SpecLoan>().await };
                        async { b.recv::<FumeLoan>().await };
                    }
                    .await?;
                    let (spec_result, fume_result) = async {
                        async {
                            spec.send::<QcSpecRequest>(&bytes.as_slice()).await?;
                            imp::decode(spec.recv::<QcSpecResult>().await?)
                        };
                        async {
                            fume.send::<QcFumeRequest>(&bytes.as_slice()).await?;
                            imp::decode(fume.recv::<QcFumeResult>().await?)
                        };
                    }
                    .await?;
                    imp::same_value(&spec_result, &fume_result)?;
                    imp::qc_result(&spec_result, &selected, &inputs)?;
                    async {
                        let [mut a, mut b] = endpoint.par()?;
                        async {
                            a.send::<SpecLoan>(spec).await?;
                        };
                        async {
                            b.send::<FumeLoan>(fume).await?;
                        };
                        Ok(())
                    }
                    .await?;
                    ShellResult::Ok((spec_result, report))
                }
                .await?;
                let _ = report;
                endpoint.send::<BioAccess>(biological).await?;
                let branch = endpoint.offer().await?;
                let confirmed = match branch.label() {
                    LastBiologistApproved::LOGICAL_LABEL => {
                        imp::decode(branch.recv::<LastBiologistApproved>().await?)?
                    }
                    LastBiologistRejected::LOGICAL_LABEL => {
                        imp::decode(branch.recv::<LastBiologistRejected>().await?)?
                    }
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected remaining QC verdict {label}"
                        )));
                    }
                };
                imp::same_value(&confirmed, &result)?;
                biological = endpoint.recv::<BioAccess>().await?;
                let bytes = imp::encode(&confirmed)?;
                match confirmed["success"].as_bool() {
                    Some(true) => {
                        let reply = QcReply::<BIOLOGIST, true>::from_bytes(&bytes)?;
                        endpoint.send::<LastBiologistApproved>(&reply).await?;
                        endpoint.send::<LastBiologistApproved>(&reply).await?;
                        endpoint.send::<LastBiologistApproved>(&reply).await?;
                        endpoint.send::<LastBiologistApproved>(&reply).await?;
                        endpoint
                            .send::<FileQcBiologistResult>(&bytes.as_slice())
                            .await?;

                        endpoint.send::<BioAccess>(biological).await?;
                        break confirmed;
                    }
                    Some(false) => {
                        let reply = QcReply::<BIOLOGIST, false>::from_bytes(&bytes)?;
                        endpoint.send::<LastBiologistRejected>(&reply).await?;
                        endpoint.send::<LastBiologistRejected>(&reply).await?;
                        endpoint.send::<LastBiologistRejected>(&reply).await?;
                        endpoint.send::<LastBiologistRejected>(&reply).await?;
                        endpoint
                            .send::<FileQcBiologistResult>(&bytes.as_slice())
                            .await?;

                        endpoint.send::<BioAccess>(biological).await?;
                        biological = endpoint.recv::<BioAccess>().await?;
                    }
                    None => return Err(ShellError::message("missing actual remaining QC verdict")),
                }
            }
        }
        Some("BIOLOGIST") => {
            endpoint.send::<BioAccess>(biological).await?;
            loop {
                let selected = endpoint.recv::<FileQcChemist>().await?.into_value();
                let bytes = imp::encode(&selected)?;
                endpoint.send::<LastChemist>(&bytes.as_slice()).await?;
                endpoint.send::<LastChemist>(&bytes.as_slice()).await?;
                endpoint.send::<LastChemist>(&bytes.as_slice()).await?;

                let (result, report) = async {
                    let report = imp::decode(endpoint.recv::<ChemistReport>().await?)?;
                    let inputs = imp::qc_inputs(&selected, &report)?;
                    let bytes = imp::encode(&json!({"arguments":selected,"materials":report}))?;
                    chemistry.send::<QcAdmission>(&bytes.as_slice()).await?;
                    let (mut spec, mut fume) = async {
                        let [mut a, mut b] = endpoint.par()?;
                        async { a.recv::<SpecLoan>().await };
                        async { b.recv::<FumeLoan>().await };
                    }
                    .await?;
                    let (spec_result, fume_result) = async {
                        async {
                            spec.send::<QcSpecRequest>(&bytes.as_slice()).await?;
                            imp::decode(spec.recv::<QcSpecResult>().await?)
                        };
                        async {
                            fume.send::<QcFumeRequest>(&bytes.as_slice()).await?;
                            imp::decode(fume.recv::<QcFumeResult>().await?)
                        };
                    }
                    .await?;
                    imp::same_value(&spec_result, &fume_result)?;
                    imp::qc_result(&spec_result, &selected, &inputs)?;
                    async {
                        let [mut a, mut b] = endpoint.par()?;
                        async {
                            a.send::<SpecLoan>(spec).await?;
                        };
                        async {
                            b.send::<FumeLoan>(fume).await?;
                        };
                        Ok(())
                    }
                    .await?;
                    ShellResult::Ok((spec_result, report))
                }
                .await?;
                chemist_materials = report;
                endpoint.send::<ChemAccess>(chemistry).await?;
                let branch = endpoint.offer().await?;
                let confirmed = match branch.label() {
                    LastChemistApproved::LOGICAL_LABEL => {
                        imp::decode(branch.recv::<LastChemistApproved>().await?)?
                    }
                    LastChemistRejected::LOGICAL_LABEL => {
                        imp::decode(branch.recv::<LastChemistRejected>().await?)?
                    }
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected remaining QC verdict {label}"
                        )));
                    }
                };
                imp::same_value(&confirmed, &result)?;
                chemistry = endpoint.recv::<ChemAccess>().await?;
                let bytes = imp::encode(&confirmed)?;
                match confirmed["success"].as_bool() {
                    Some(true) => {
                        let reply = QcReply::<CHEMIST, true>::from_bytes(&bytes)?;
                        endpoint.send::<LastChemistApproved>(&reply).await?;
                        endpoint.send::<LastChemistApproved>(&reply).await?;
                        endpoint.send::<LastChemistApproved>(&reply).await?;
                        endpoint.send::<LastChemistApproved>(&reply).await?;
                        endpoint
                            .send::<FileQcChemistResult>(&bytes.as_slice())
                            .await?;

                        endpoint.send::<ChemAccess>(chemistry).await?;
                        break confirmed;
                    }
                    Some(false) => {
                        let reply = QcReply::<CHEMIST, false>::from_bytes(&bytes)?;
                        endpoint.send::<LastChemistRejected>(&reply).await?;
                        endpoint.send::<LastChemistRejected>(&reply).await?;
                        endpoint.send::<LastChemistRejected>(&reply).await?;
                        endpoint.send::<LastChemistRejected>(&reply).await?;
                        endpoint
                            .send::<FileQcChemistResult>(&bytes.as_slice())
                            .await?;

                        endpoint.send::<ChemAccess>(chemistry).await?;
                        chemistry = endpoint.recv::<ChemAccess>().await?;
                    }
                    None => return Err(ShellError::message("missing actual remaining QC verdict")),
                }
            }
        }
        _ => return Err(ShellError::message("unknown actual approved subject")),
    };
    let (chemist_qc, biologist_qc) = if first_approval["data"]["scientist"] == "CHEMIST" {
        (first_approval, last_approval)
    } else {
        (last_approval, first_approval)
    };
    let approved =
        json!({"materials":chemist_materials,"chemist_qc":chemist_qc,"biologist_qc":biologist_qc});
    let bytes = imp::encode(&approved)?;
    endpoint.send::<ApprovedResults>(&bytes.as_slice()).await?;
    Ok(())
}
