//! Actual base request, equipment ownership and incoming completed ingredient.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::Endpoint;
#[hibana_runtime::scope]
pub async fn run(endpoint: &mut Endpoint<B>, domain: &impl Domain) -> ShellResult<()> {
    let [mut base, mut incoming] = endpoint.par()?;
    async {
        let admitted = base.recv::<BaseIntentB>().await?;
        let (mut oven, mut stove) = async {
            let [mut o, mut s] = base.par()?;
            async { Ok::<_, ShellError>(o.recv::<OvenLoanB>().await?) };
            async { Ok::<_, ShellError>(s.recv::<StoveLoanB>().await?) };
        }
        .await?;
        let (left, right) = async {
            let [mut o, mut s] = base.par()?;
            async {
                async {
                    oven.send::<OvenRequestB>(&admitted).await?;
                };
                Ok::<_, ShellError>(o.recv::<OvenRequestB>().await?)
            };

            async {
                async {
                    stove.send::<StoveRequestB>(&admitted).await?;
                };
                Ok::<_, ShellError>(s.recv::<StoveRequestB>().await?)
            };
        }
        .await?;

        let arguments = admitted.value();
        if left.value() != arguments || right.value() != arguments {
            return Err(ShellError::message(
                "equipment request differs from admitted base input",
            ));
        }
        let result = domain
            .execute("CHEF_B", "prepare_base_dish", arguments.clone())
            .await
            .map_err(ShellError::message)?;
        imp::completed("prepare_base_dish", "CHEF_B", &arguments, &result)?;
        let bytes = imp::encode(&result)?;
        async {
            let [mut o, mut s] = base.par()?;
            async {
                async {
                    o.send::<OvenResultB>(&bytes.as_slice()).await?;
                };
                let echoed = oven.recv::<OvenResultB>().await?;
                if echoed[..] != bytes {
                    return Err(ShellError::message(
                        "equipment result differs from actual base effect",
                    ));
                }
                Ok::<(), ShellError>(())
            };
            async {
                async {
                    s.send::<StoveResultB>(&bytes.as_slice()).await?;
                };
                let echoed = stove.recv::<StoveResultB>().await?;
                if echoed[..] != bytes {
                    return Err(ShellError::message(
                        "equipment result differs from actual base effect",
                    ));
                }
                Ok::<(), ShellError>(())
            };
        }
        .await?;
        async {
            let [mut file, mut oven_return, mut stove_return] = base.par()?;
            async {
                file.send::<BaseResultB>(&bytes.as_slice()).await?;
            };
            async {
                oven_return.send::<ReturnOvenB>(oven).await?;
            };
            async {
                stove_return.send::<ReturnStoveB>(stove).await?;
            };
            Ok::<(), ShellError>(())
        }
        .await?;
    };
    async {
        let actual = imp::decode(incoming.recv::<Glaze>().await?)?;
        imp::completed(
            "prepare_ingredient",
            "CHEF_C",
            &imp::expected(5).expect("assigned ingredient"),
            &actual,
        )?;
    };
    Ok(())
}
