//! Actual base request, equipment ownership and incoming completed ingredient.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::ReceivedEffects};
use hibana::Endpoint;
#[hibana_runtime::scope]
pub async fn run(endpoint: &mut Endpoint<C>, effects: &impl ReceivedEffects) -> ShellResult<()> {
    let [mut base, mut incoming] = endpoint.par()?;
    async {
        let admitted = base.recv::<BaseIntentC>().await?;
        let mut oven = base.recv::<OvenLoanC>().await?;
        let actual = async {
            async {
                oven.send::<OvenRequestC>(&admitted).await?;
            };
            Ok::<_, ShellError>(base.recv::<OvenRequestC>().await?)
        }
        .await?;
        let arguments = admitted.value();
        if imp::decode(actual.as_bytes())? != arguments {
            return Err(ShellError::message(
                "oven request differs from admitted base input",
            ));
        }
        let result = effects.apply(actual).await?;
        imp::completed("prepare_base_dish", "CHEF_C", &arguments, &result)?;
        let bytes = imp::encode(&result)?;
        async {
            async {
                base.send::<OvenResultC>(&bytes.as_slice()).await?;
            };
            let echoed = oven.recv::<OvenResultC>().await?;
            if echoed[..] != bytes {
                return Err(ShellError::message(
                    "oven result differs from actual base effect",
                ));
            }
            Ok::<(), ShellError>(())
        }
        .await?;
        async {
            let [mut file, mut oven_return] = base.par()?;
            async {
                file.send::<BaseResultC>(&bytes.as_slice()).await?;
            };
            async {
                oven_return.send::<ReturnOvenC>(oven).await?;
            };
            Ok::<(), ShellError>(())
        }
        .await?;
    };
    async {
        let actual = imp::decode(incoming.recv::<Sauce>().await?)?;
        imp::completed(
            "prepare_ingredient",
            "CHEF_A",
            &imp::expected(3).expect("assigned ingredient"),
            &actual,
        )?;
    };
    Ok(())
}
