//! Match a workstation request with one of the two returned physical tools.
use super::super::global::*;
use crate::shell::{ShellError, ShellResult, global::FileExit};
use hibana::Endpoint;

pub(super) enum Tool<'r> {
    First(Endpoint<'r, TOOL_0>),
    Second(Endpoint<'r, TOOL_1>),
}

#[hibana_runtime::scope]
pub async fn run(cabinet: &mut Endpoint<CABINET>) -> ShellResult<()> {
    let [
        mut alpha,
        mut beta,
        mut gamma,
        mut first,
        mut second,
        mut close,
    ] = cabinet.par()?;
    loop {
        let job = hibana_runtime::race(
            async {
                close.recv::<FileExit>().await?;
                Ok::<_, ShellError>(None)
            },
            async {
                hibana_runtime::race(
                    async {
                        alpha.recv::<MachineRequest<WORKER_A>>().await?;
                        Ok(Some(WORKER_A))
                    },
                    async {
                        hibana_runtime::race(
                            async {
                                beta.recv::<MachineRequest<WORKER_B>>().await?;
                                Ok(Some(WORKER_B))
                            },
                            async {
                                gamma.recv::<MachineRequest<WORKER_C>>().await?;
                                Ok(Some(WORKER_C))
                            },
                        )
                        .await
                    },
                )
                .await
            },
        )
        .await?;
        let Some(job) = job else { break };
        let available = hibana_runtime::race(
            async {
                Ok::<_, ShellError>(Tool::First(first.recv::<SlotLoan<TOOL_0, TOOL_0>>().await?))
            },
            async {
                Ok(Tool::Second(
                    second.recv::<SlotLoan<TOOL_1, TOOL_1>>().await?,
                ))
            },
        )
        .await?;
        match (available, job) {
            (Tool::First(tool), WORKER_A) => first.send::<SlotLoan<TOOL_0, 60>>(tool).await?,
            (Tool::First(tool), WORKER_B) => first.send::<SlotLoan<TOOL_0, 61>>(tool).await?,
            (Tool::First(tool), WORKER_C) => first.send::<SlotLoan<TOOL_0, 62>>(tool).await?,
            (Tool::Second(tool), WORKER_A) => second.send::<SlotLoan<TOOL_1, 63>>(tool).await?,
            (Tool::Second(tool), WORKER_B) => second.send::<SlotLoan<TOOL_1, 64>>(tool).await?,
            (Tool::Second(tool), WORKER_C) => second.send::<SlotLoan<TOOL_1, 65>>(tool).await?,
            _ => return Err(ShellError::message("unknown cabinet workstation")),
        }
    }
    async {
        let tool = first.recv::<SlotLoan<TOOL_0, TOOL_0>>().await?;
        first.send::<SlotLoan<TOOL_0, TOOL_0>>(tool).await?;
    };
    let tool = second.recv::<SlotLoan<TOOL_1, TOOL_1>>().await?;
    second.send::<SlotLoan<TOOL_1, TOOL_1>>(tool).await?;
    Ok(())
}
