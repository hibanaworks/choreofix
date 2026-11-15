//! The actual selected request determines which chef receives the equipment.
use super::super::global::*;
use crate::shell::{ShellError, ShellResult};
use hibana::{Endpoint, g::Message};
#[hibana_runtime::scope]
pub async fn run<'r>(
    endpoint: &mut Endpoint<'r, COOKING>,
    resources: (Endpoint<'r, OVEN>, Endpoint<'r, STOVE>),
) -> ShellResult<()> {
    let (mut oven, mut stove) = resources;
    for _ in 0..3 {
        let label = endpoint.offer().await?.label();
        match label {
            x if x == OvenLoanA::LOGICAL_LABEL || x == StoveLoanA::LOGICAL_LABEL => {
                async {
                    let [mut o, mut s] = endpoint.par()?;
                    async {
                        o.send::<OvenLoanA>(oven).await?;
                    };
                    async {
                        s.send::<StoveLoanA>(stove).await?;
                    };
                    Ok::<(), ShellError>(())
                }
                .await?;
                (oven, stove) = async {
                    let [mut o, mut s] = endpoint.par()?;
                    async { Ok::<_, ShellError>(o.recv::<ReturnOvenA>().await?) };
                    async { Ok::<_, ShellError>(s.recv::<ReturnStoveA>().await?) };
                }
                .await?;
            }
            x if x == OvenLoanB::LOGICAL_LABEL || x == StoveLoanB::LOGICAL_LABEL => {
                async {
                    let [mut o, mut s] = endpoint.par()?;
                    async {
                        o.send::<OvenLoanB>(oven).await?;
                    };
                    async {
                        s.send::<StoveLoanB>(stove).await?;
                    };
                    Ok::<(), ShellError>(())
                }
                .await?;
                (oven, stove) = async {
                    let [mut o, mut s] = endpoint.par()?;
                    async { Ok::<_, ShellError>(o.recv::<ReturnOvenB>().await?) };
                    async { Ok::<_, ShellError>(s.recv::<ReturnStoveB>().await?) };
                }
                .await?;
            }
            x if x == OvenLoanC::LOGICAL_LABEL => {
                endpoint.send::<OvenLoanC>(oven).await?;
                oven = endpoint.recv::<ReturnOvenC>().await?;
            }
            _ => {
                return Err(ShellError::message(
                    "unexpected kitchen equipment operation",
                ));
            }
        }
    }
    Ok(())
}
