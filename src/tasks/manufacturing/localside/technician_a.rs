//! Both independent jobs retain their actual file request and machine Endpoint.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::Endpoint;

#[hibana_runtime::scope]
pub async fn run(endpoint: &mut Endpoint<TECHNICIAN_A>, domain: &impl Domain) -> ShellResult<()> {
    let [mut lathe_work, mut mill_work] = endpoint.par()?;
    async {
        let admitted = lathe_work.recv::<RequestLatheA>().await?;
        let mut resource = lathe_work.recv::<LatheLoan>().await?;
        let actual = async {
            async {
                resource.send::<LatheJobA>(&admitted).await?;
            };
            Ok::<_, ShellError>(lathe_work.recv::<LatheJobA>().await?)
        }
        .await?;
        let admitted = admitted.into_value();
        if actual.into_value() != admitted {
            return Err(ShellError::message(
                "lathe resource differs from admitted file request",
            ));
        }
        let result = domain
            .execute("TECHNICIAN_A", "do_job", admitted.clone())
            .await
            .map_err(ShellError::message)?;
        imp::completed(&result, "TECHNICIAN_A", "LATHE", &admitted)?;
        let bytes = imp::encode(&result)?;
        async {
            async {
                lathe_work.send::<LatheResult>(&bytes.as_slice()).await?;
            };
            let echoed = resource.recv::<LatheResult>().await?;
            if echoed[..] != bytes {
                return Err(ShellError::message(
                    "lathe reply differs from actual effect",
                ));
            }
            Ok::<(), ShellError>(())
        }
        .await?;
        async {
            let [mut file, mut report, mut returned] = lathe_work.par()?;
            async {
                file.send::<FileLatheResultA>(&bytes.as_slice()).await?;
            };
            async {
                report.send::<LatheDoneA>(&bytes.as_slice()).await?;
            };
            async {
                returned.send::<ReturnLathe>(resource).await?;
            };
            Ok::<(), ShellError>(())
        }
        .await?;
    };
    async {
        let admitted = mill_work.recv::<RequestMillA>().await?;
        let mut resource = mill_work.recv::<MillLoan>().await?;
        let actual = async {
            async {
                resource.send::<MillJobA>(&admitted).await?;
            };
            Ok::<_, ShellError>(mill_work.recv::<MillJobA>().await?)
        }
        .await?;
        let admitted = admitted.into_value();
        if actual.into_value() != admitted {
            return Err(ShellError::message(
                "mill resource differs from admitted file request",
            ));
        }
        let result = domain
            .execute("TECHNICIAN_A", "do_job", admitted.clone())
            .await
            .map_err(ShellError::message)?;
        imp::completed(&result, "TECHNICIAN_A", "MILL", &admitted)?;
        let bytes = imp::encode(&result)?;
        async {
            async {
                mill_work.send::<MillResult>(&bytes.as_slice()).await?;
            };
            let echoed = resource.recv::<MillResult>().await?;
            if echoed[..] != bytes {
                return Err(ShellError::message("mill reply differs from actual effect"));
            }
            Ok::<(), ShellError>(())
        }
        .await?;
        async {
            let [mut file, mut report, mut returned] = mill_work.par()?;
            async {
                file.send::<FileMillResultA>(&bytes.as_slice()).await?;
            };
            async {
                report.send::<MillDoneA>(&bytes.as_slice()).await?;
            };
            async {
                returned.send::<ReturnMill>(resource).await?;
            };
            Ok::<(), ShellError>(())
        }
        .await?;
    };
    Ok(())
}
