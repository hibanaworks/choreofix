//! Own the two actual tools and the accepted RPCs until their results return.
use super::super::{global::*, imp};
use crate::shell::{
    ShellError, ShellResult,
    imp::{bus::Bus, service::Domain},
};
use hibana::Endpoint;
use serde_json::{Value, json};
use std::cell::Cell;

#[hibana_runtime::scope]
pub async fn run<'r>(
    endpoint: &mut Endpoint<'r, DEPOT>,
    (tool0, tool1): (Endpoint<'r, TOOL_0>, Endpoint<'r, TOOL_1>),
    (domain, inputs): (&Bus, &[Cell<Value>; 5]),
) -> ShellResult<()> {
    // Each cell is the actual input for one resolver, not an allocation table.
    let [
        first_input,
        next_input,
        arrival0_input,
        arrival1_input,
        finish_input,
    ] = inputs;
    let [mut a, mut b, mut c, mut work] = endpoint.par()?;
    let first = hibana_runtime::race(
        async { imp::admitted(&a.recv::<Intent<WORKER_A>>().await?) },
        async {
            hibana_runtime::race(
                async { imp::admitted(&b.recv::<Intent<WORKER_B>>().await?) },
                async { imp::admitted(&c.recv::<Intent<WORKER_C>>().await?) },
            )
            .await
        },
    )
    .await?;
    first_input.set(first.clone());
    work.send::<Loan0>(tool0).await?;
    let received = match imp::agent(&first)? {
        "WORKER_A" => work.recv::<Request0<WORKER_A>>().await?.value(),
        "WORKER_B" => work.recv::<Request0<WORKER_B>>().await?.value(),
        "WORKER_C" => work.recv::<Request0<WORKER_C>>().await?.value(),
        _ => return Err(ShellError::message("unknown worker")),
    };
    let arguments0 = imp::arguments(&first, received)?;
    let mut effect0 = domain.execute(imp::agent(&first)?, "do_job", arguments0);
    // Borrowing an accepted RPC in a race never cancels or reissues that RPC.
    let arrived = hibana_runtime::race(
        async {
            let result = (&mut effect0).await.map_err(ShellError::message)?;
            imp::completed(&first, &result, None)?;
            Ok::<_, ShellError>(result)
        },
        async {
            match imp::agent(&first)? {
                "WORKER_A" => {
                    hibana_runtime::race(
                        async { imp::admitted(&b.recv::<Intent<WORKER_B>>().await?) },
                        async { imp::admitted(&c.recv::<Intent<WORKER_C>>().await?) },
                    )
                    .await
                }
                "WORKER_B" => {
                    hibana_runtime::race(
                        async { imp::admitted(&a.recv::<Intent<WORKER_A>>().await?) },
                        async { imp::admitted(&c.recv::<Intent<WORKER_C>>().await?) },
                    )
                    .await
                }
                _ => {
                    hibana_runtime::race(
                        async { imp::admitted(&a.recv::<Intent<WORKER_A>>().await?) },
                        async { imp::admitted(&b.recv::<Intent<WORKER_B>>().await?) },
                    )
                    .await
                }
            }
        },
    )
    .await?;
    arrival0_input.set(arrived.clone());
    if arrived["tool_name"] == "do_job" {
        let bytes = imp::encode(&arrived)?;
        work.send::<Reply0>(&bytes.as_slice()).await?;
        let tool0 = work.recv::<Return0>().await?;
        let second = match imp::agent(&first)? {
            "WORKER_A" => {
                hibana_runtime::race(
                    async { imp::admitted(&b.recv::<Intent<WORKER_B>>().await?) },
                    async { imp::admitted(&c.recv::<Intent<WORKER_C>>().await?) },
                )
                .await?
            }
            "WORKER_B" => {
                hibana_runtime::race(
                    async { imp::admitted(&a.recv::<Intent<WORKER_A>>().await?) },
                    async { imp::admitted(&c.recv::<Intent<WORKER_C>>().await?) },
                )
                .await?
            }
            _ => {
                hibana_runtime::race(
                    async { imp::admitted(&a.recv::<Intent<WORKER_A>>().await?) },
                    async { imp::admitted(&b.recv::<Intent<WORKER_B>>().await?) },
                )
                .await?
            }
        };
        next_input.set(second.clone());
        work.send::<Loan0>(tool0).await?;
        let received = match imp::agent(&second)? {
            "WORKER_A" => work.recv::<Request0<WORKER_A>>().await?.value(),
            "WORKER_B" => work.recv::<Request0<WORKER_B>>().await?.value(),
            "WORKER_C" => work.recv::<Request0<WORKER_C>>().await?.value(),
            _ => return Err(ShellError::message("unknown worker")),
        };
        let arguments = imp::arguments(&second, received)?;
        let mut effect = domain.execute(imp::agent(&second)?, "do_job", arguments);
        let arrived = hibana_runtime::race(
            async {
                let result = (&mut effect).await.map_err(ShellError::message)?;
                imp::completed(&second, &result, None)?;
                Ok::<_, ShellError>(result)
            },
            async {
                if first["agent"] != "WORKER_A" && second["agent"] != "WORKER_A" {
                    imp::admitted(&a.recv::<Intent<WORKER_A>>().await?)
                } else if first["agent"] != "WORKER_B" && second["agent"] != "WORKER_B" {
                    imp::admitted(&b.recv::<Intent<WORKER_B>>().await?)
                } else {
                    imp::admitted(&c.recv::<Intent<WORKER_C>>().await?)
                }
            },
        )
        .await?;
        arrival1_input.set(arrived.clone());
        if arrived["tool_name"] == "do_job" {
            let bytes = imp::encode(&arrived)?;
            work.send::<Reply0>(&bytes.as_slice()).await?;
            let tool0 = work.recv::<Return0>().await?;
            let third = if first["agent"] != "WORKER_A" && second["agent"] != "WORKER_A" {
                imp::admitted(&a.recv::<Intent<WORKER_A>>().await?)?
            } else if first["agent"] != "WORKER_B" && second["agent"] != "WORKER_B" {
                imp::admitted(&b.recv::<Intent<WORKER_B>>().await?)?
            } else {
                imp::admitted(&c.recv::<Intent<WORKER_C>>().await?)?
            };
            work.send::<Loan0>(tool0).await?;
            let received = match imp::agent(&third)? {
                "WORKER_A" => work.recv::<Request0<WORKER_A>>().await?.value(),
                "WORKER_B" => work.recv::<Request0<WORKER_B>>().await?.value(),
                "WORKER_C" => work.recv::<Request0<WORKER_C>>().await?.value(),
                _ => return Err(ShellError::message("unknown worker")),
            };
            let arguments = imp::arguments(&third, received)?;
            let result = domain
                .execute(imp::agent(&third)?, "do_job", arguments)
                .await
                .map_err(ShellError::message)?;
            imp::completed(&third, &result, None)?;
            let bytes = imp::encode(&result)?;
            work.send::<Reply0>(&bytes.as_slice()).await?;
            drop(work.recv::<Return0>().await?);
        } else {
            // This route's two arms are independent; no two-job batch join.
            async {
                drop(work.offer().await?);
                let [mut finishing, mut third] = work.par()?;
                async {
                    let result = effect.await.map_err(ShellError::message)?;
                    imp::completed(&second, &result, None)?;
                    let bytes = imp::encode(&result)?;
                    finishing.send::<Reply0>(&bytes.as_slice()).await?;
                    drop(finishing.recv::<Return0>().await?);
                };
                async {
                    third.send::<Loan1>(tool1).await?;
                    let received = match imp::agent(&arrived)? {
                        "WORKER_A" => third.recv::<Request1<WORKER_A>>().await?.value(),
                        "WORKER_B" => third.recv::<Request1<WORKER_B>>().await?.value(),
                        "WORKER_C" => third.recv::<Request1<WORKER_C>>().await?.value(),
                        _ => return Err(ShellError::message("unknown worker")),
                    };
                    let arguments = imp::arguments(&arrived, received)?;
                    let result = domain
                        .execute(imp::agent(&arrived)?, "do_job", arguments)
                        .await
                        .map_err(ShellError::message)?;
                    imp::completed(&arrived, &result, None)?;
                    let bytes = imp::encode(&result)?;
                    third.send::<Reply1>(&bytes.as_slice()).await?;
                    drop(third.recv::<Return1>().await?);
                };
                Ok::<(), ShellError>(())
            }
            .await?;
        }
    } else {
        let second = arrived;
        next_input.set(second.clone());
        work.send::<Loan1>(tool1).await?;
        let received = match imp::agent(&second)? {
            "WORKER_A" => work.recv::<Request1<WORKER_A>>().await?.value(),
            "WORKER_B" => work.recv::<Request1<WORKER_B>>().await?.value(),
            "WORKER_C" => work.recv::<Request1<WORKER_C>>().await?.value(),
            _ => return Err(ShellError::message("unknown worker")),
        };
        let arguments1 = imp::arguments(&second, received)?;
        let mut effect1 = domain.execute(imp::agent(&second)?, "do_job", arguments1);
        let completed = hibana_runtime::race(
            async {
                let result = (&mut effect0).await.map_err(ShellError::message)?;
                imp::completed(&first, &result, None)?;
                Ok::<_, ShellError>((TOOL_0, result))
            },
            async {
                let result = (&mut effect1).await.map_err(ShellError::message)?;
                imp::completed(&second, &result, None)?;
                Ok::<_, ShellError>((TOOL_1, result))
            },
        )
        .await?;
        finish_input.set(json!({"resource":completed.0,"result":completed.1}));
        drop(work.offer().await?);
        if completed.0 == TOOL_0 {
            async {
                let [mut available, mut finishing] = work.par()?;
                async {
                    let bytes = imp::encode(&completed.1)?;
                    available.send::<Reply0>(&bytes.as_slice()).await?;
                    let tool = available.recv::<Return0>().await?;
                    let third = if first["agent"] != "WORKER_A" && second["agent"] != "WORKER_A" {
                        imp::admitted(&a.recv::<Intent<WORKER_A>>().await?)?
                    } else if first["agent"] != "WORKER_B" && second["agent"] != "WORKER_B" {
                        imp::admitted(&b.recv::<Intent<WORKER_B>>().await?)?
                    } else {
                        imp::admitted(&c.recv::<Intent<WORKER_C>>().await?)?
                    };
                    available.send::<Loan0>(tool).await?;
                    let received = match imp::agent(&third)? {
                        "WORKER_A" => available.recv::<Request0<WORKER_A>>().await?.value(),
                        "WORKER_B" => available.recv::<Request0<WORKER_B>>().await?.value(),
                        "WORKER_C" => available.recv::<Request0<WORKER_C>>().await?.value(),
                        _ => return Err(ShellError::message("unknown worker")),
                    };
                    let arguments = imp::arguments(&third, received)?;
                    let result = domain
                        .execute(imp::agent(&third)?, "do_job", arguments)
                        .await
                        .map_err(ShellError::message)?;
                    imp::completed(&third, &result, None)?;
                    let bytes = imp::encode(&result)?;
                    available.send::<Reply0>(&bytes.as_slice()).await?;
                    drop(available.recv::<Return0>().await?);
                };
                async {
                    let result = effect1.await.map_err(ShellError::message)?;
                    imp::completed(&second, &result, None)?;
                    let bytes = imp::encode(&result)?;
                    finishing.send::<Reply1>(&bytes.as_slice()).await?;
                    drop(finishing.recv::<Return1>().await?);
                };
                Ok::<(), ShellError>(())
            }
            .await?;
        } else {
            async {
                let [mut finishing, mut available] = work.par()?;
                async {
                    let result = effect0.await.map_err(ShellError::message)?;
                    imp::completed(&first, &result, None)?;
                    let bytes = imp::encode(&result)?;
                    finishing.send::<Reply0>(&bytes.as_slice()).await?;
                    drop(finishing.recv::<Return0>().await?);
                };
                async {
                    let bytes = imp::encode(&completed.1)?;
                    available.send::<Reply1>(&bytes.as_slice()).await?;
                    let tool = available.recv::<Return1>().await?;
                    let third = if first["agent"] != "WORKER_A" && second["agent"] != "WORKER_A" {
                        imp::admitted(&a.recv::<Intent<WORKER_A>>().await?)?
                    } else if first["agent"] != "WORKER_B" && second["agent"] != "WORKER_B" {
                        imp::admitted(&b.recv::<Intent<WORKER_B>>().await?)?
                    } else {
                        imp::admitted(&c.recv::<Intent<WORKER_C>>().await?)?
                    };
                    available.send::<Loan1>(tool).await?;
                    let received = match imp::agent(&third)? {
                        "WORKER_A" => available.recv::<Request1<WORKER_A>>().await?.value(),
                        "WORKER_B" => available.recv::<Request1<WORKER_B>>().await?.value(),
                        "WORKER_C" => available.recv::<Request1<WORKER_C>>().await?.value(),
                        _ => return Err(ShellError::message("unknown worker")),
                    };
                    let arguments = imp::arguments(&third, received)?;
                    let result = domain
                        .execute(imp::agent(&third)?, "do_job", arguments)
                        .await
                        .map_err(ShellError::message)?;
                    imp::completed(&third, &result, None)?;
                    let bytes = imp::encode(&result)?;
                    available.send::<Reply1>(&bytes.as_slice()).await?;
                    drop(available.recv::<Return1>().await?);
                };
                Ok::<(), ShellError>(())
            }
            .await?;
        }
    }
    Ok(())
}
