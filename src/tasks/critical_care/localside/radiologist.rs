//! CT's six causal orders and the independent MRI continuation.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult};
use hibana::{Endpoint, g::Message};
#[hibana_runtime::scope]
pub async fn run(endpoint: &mut Endpoint<RADIOLOGIST>) -> ShellResult<()> {
    async {
        let [mut ct, mut mri] = endpoint.par()?;
        async {
            for _ in 0..4 {
                let branch = ct.offer().await?;
                match branch.label() {
                    CtPerformAdmittedA::LOGICAL_LABEL => {
                        let admitted = branch.recv::<CtPerformAdmittedA>().await?;
                        let mut scanner = ct.recv::<CtLoan>().await?;
                        scanner.send::<CtPerformRequestA>(&admitted).await?;
                        let result = imp::decode(scanner.recv::<CtImageA>().await?)?;
                        imp::actual_result(&result, "patient_a")?;
                        ct.send::<CtLoan>(scanner).await?;
                    }
                    CtInterpretAdmittedA::LOGICAL_LABEL => {
                        let admitted = branch.recv::<CtInterpretAdmittedA>().await?;
                        let mut scanner = ct.recv::<CtLoan>().await?;
                        scanner.send::<CtInterpretRequestA>(&admitted).await?;
                        let result = imp::decode(scanner.recv::<CtReportA>().await?)?;
                        imp::actual_result(&result, "patient_a")?;
                        ct.send::<CtLoan>(scanner).await?;
                    }
                    CtPerformAdmittedB::LOGICAL_LABEL => {
                        let admitted = branch.recv::<CtPerformAdmittedB>().await?;
                        let mut scanner = ct.recv::<CtLoan>().await?;
                        scanner.send::<CtPerformRequestB>(&admitted).await?;
                        let result = imp::decode(scanner.recv::<CtImageB>().await?)?;
                        imp::actual_result(&result, "patient_b")?;
                        ct.send::<CtLoan>(scanner).await?;
                    }
                    CtInterpretAdmittedB::LOGICAL_LABEL => {
                        let admitted = branch.recv::<CtInterpretAdmittedB>().await?;
                        let mut scanner = ct.recv::<CtLoan>().await?;
                        scanner.send::<CtInterpretRequestB>(&admitted).await?;
                        let result = imp::decode(scanner.recv::<CtReportB>().await?)?;
                        imp::actual_result(&result, "patient_b")?;
                        ct.send::<CtLoan>(scanner).await?;
                    }
                    _ => return Err(ShellError::message("unexpected actual CT admission")),
                }
            }
            Ok::<(), ShellError>(())
        };
        async {
            let admitted = mri.recv::<MriPerformAdmitted>().await?;
            let mut scanner = mri.recv::<MriLoan>().await?;
            scanner.send::<MriPerformRequest>(&admitted).await?;
            let result = imp::decode(scanner.recv::<MriImage>().await?)?;
            imp::actual_result(&result, "patient_b")?;
            mri.send::<MriLoan>(scanner).await?;
            let admitted = mri.recv::<MriInterpretAdmitted>().await?;
            let mut scanner = mri.recv::<MriLoan>().await?;
            scanner.send::<MriInterpretRequest>(&admitted).await?;
            let result = imp::decode(scanner.recv::<MriReport>().await?)?;
            imp::actual_result(&result, "patient_b")?;
            mri.send::<MriLoan>(scanner).await?;
            Ok::<(), ShellError>(())
        };
        Ok::<(), ShellError>(())
    }
    .await?;
    Ok(())
}
