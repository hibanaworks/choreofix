use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult};
use hibana::{Endpoint, g::Message};
pub async fn run(endpoint: &mut Endpoint<'_, REGULATOR>) -> ShellResult<()> {
    loop {
        let decision = imp::decode(endpoint.recv::<Decision>().await?)?;
        imp::artifact(&decision)?;
        let bytes = imp::encode(&decision)?;
        endpoint.send::<Decision>(&bytes.as_slice()).await?;
        let mut study = endpoint.recv::<Loan<STUDY>>().await?;
        study.send::<ReviewInput>(&bytes.as_slice()).await?;
        endpoint.send::<Loan<STUDY>>(study).await?;
        let branch = endpoint.offer().await?;
        let receipt = match branch.label() {
            Passed::LOGICAL_LABEL => imp::decode(branch.recv::<Passed>().await?)?,
            ComplianceRejected::LOGICAL_LABEL => {
                imp::decode(branch.recv::<ComplianceRejected>().await?)?
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected regulatory result {label}"
                )));
            }
        };
        let bytes = imp::encode(&receipt)?;
        endpoint
            .send::<Reply<COMPLIANCE>>(&bytes.as_slice())
            .await?;
        if receipt["success"] == true {
            return Ok(());
        }
    }
}
