//! Actual base request, equipment ownership and incoming completed ingredient.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::PairedEffects};
use hibana::Endpoint;
#[hibana_runtime::scope]
pub async fn run(endpoint: &mut Endpoint<A>, effects: &impl PairedEffects) -> ShellResult<()> {
    let [mut base, mut incoming] = endpoint.par()?;
    async {
        let admitted = base.recv::<BaseIntentA>().await?;
        let (mut oven, mut stove) = async {
            let [mut o, mut s] = base.par()?;
            async { Ok::<_, ShellError>(o.recv::<OvenLoanA>().await?) };
            async { Ok::<_, ShellError>(s.recv::<StoveLoanA>().await?) };
        }
        .await?;
        let arguments = admitted.value();
        let result = {
            let [mut o, mut s] = base.par()?;
            let (left, right) = async {
                async {
                    async {
                        oven.send::<OvenRequestA>(&admitted).await?;
                    };
                    Ok::<_, ShellError>(o.recv::<OvenRequestA>().await?)
                };

                async {
                    async {
                        stove.send::<StoveRequestA>(&admitted).await?;
                    };
                    Ok::<_, ShellError>(s.recv::<StoveRequestA>().await?)
                };
            }
            .await?;

            if imp::decode(left.as_bytes())? != arguments {
                return Err(ShellError::message(
                    "equipment request differs from admitted base input",
                ));
            }
            effects.apply_pair(left, right).await?
        };
        imp::completed("prepare_base_dish", "CHEF_A", &arguments, &result)?;
        let bytes = imp::encode(&result)?;
        async {
            let [mut o, mut s] = base.par()?;
            async {
                async {
                    o.send::<OvenResultA>(&bytes.as_slice()).await?;
                };
                let echoed = oven.recv::<OvenResultA>().await?;
                if echoed[..] != bytes {
                    return Err(ShellError::message(
                        "equipment result differs from actual base effect",
                    ));
                }
                Ok::<(), ShellError>(())
            };
            async {
                async {
                    s.send::<StoveResultA>(&bytes.as_slice()).await?;
                };
                let echoed = stove.recv::<StoveResultA>().await?;
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
                file.send::<BaseResultA>(&bytes.as_slice()).await?;
            };
            async {
                oven_return.send::<ReturnOvenA>(oven).await?;
            };
            async {
                stove_return.send::<ReturnStoveA>(stove).await?;
            };
            Ok::<(), ShellError>(())
        }
        .await?;
    };
    async {
        let actual = imp::decode(incoming.recv::<Garnish>().await?)?;
        imp::completed(
            "prepare_ingredient",
            "CHEF_B",
            &imp::expected(4).expect("assigned ingredient"),
            &actual,
        )?;
    };
    Ok(())
}
