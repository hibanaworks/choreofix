//! The two source branches use separate borrowed fragments, including real work.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::Endpoint;

#[hibana_runtime::scope]
pub async fn run(endpoint: &mut Endpoint<TECHNICIAN_B>, domain: &impl Domain) -> ShellResult<()> {
    let [mut lathe_work, mut mill_work] = endpoint.par()?;
    async {
        let admitted = lathe_work.recv::<RequestLatheB>().await?;
        let mut resource = lathe_work.recv::<LatheLoan>().await?;
        let actual = async {
            async {
                resource.send::<LatheJobB>(&admitted).await?;
            };
            Ok::<_, ShellError>(lathe_work.recv::<LatheJobB>().await?)
        }
        .await?;
        let admitted = admitted.into_value();
        if actual.into_value() != admitted {
            return Err(ShellError::message(
                "lathe resource differs from admitted file request",
            ));
        }
        let result = domain
            .execute("TECHNICIAN_B", "do_job", admitted.clone())
            .await
            .map_err(ShellError::message)?;
        imp::completed(&result, "TECHNICIAN_B", "LATHE", &admitted)?;
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
                file.send::<FileLatheResultB>(&bytes.as_slice()).await?;
            };
            async {
                report.send::<LatheDoneB>(&bytes.as_slice()).await?;
            };
            async {
                returned.send::<ReturnLathe>(resource).await?;
            };
            Ok::<(), ShellError>(())
        }
        .await?;
    };
    async {
        let admitted = mill_work.recv::<RequestMillB>().await?;
        let mut resource = mill_work.recv::<MillLoan>().await?;
        let actual = async {
            async {
                resource.send::<MillJobB>(&admitted).await?;
            };
            Ok::<_, ShellError>(mill_work.recv::<MillJobB>().await?)
        }
        .await?;
        let admitted = admitted.into_value();
        if actual.into_value() != admitted {
            return Err(ShellError::message(
                "mill resource differs from admitted file request",
            ));
        }
        let result = domain
            .execute("TECHNICIAN_B", "do_job", admitted.clone())
            .await
            .map_err(ShellError::message)?;
        imp::completed(&result, "TECHNICIAN_B", "MILL", &admitted)?;
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
                file.send::<FileMillResultB>(&bytes.as_slice()).await?;
            };
            async {
                report.send::<MillDoneB>(&bytes.as_slice()).await?;
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
