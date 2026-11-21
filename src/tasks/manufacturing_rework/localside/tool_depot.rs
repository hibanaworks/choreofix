//! Own the two actual tools and the accepted RPCs until their results return.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::bus::Simulator};
use hibana::Endpoint;
use serde_json::{Value, json};
use std::cell::Cell;

#[hibana_runtime::scope]
pub async fn run<'r>(
    endpoint: &mut Endpoint<'r, DEPOT>,
    (tool0, tool1): (Endpoint<'r, TOOL_0>, Endpoint<'r, TOOL_1>),
    (simulator, inputs): (&Simulator, &[Cell<Value>; 5]),
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
        async { imp::admitted(a.recv::<Intent<WORKER_A>>().await?) },
        async {
            hibana_runtime::race(
                async { imp::admitted(b.recv::<Intent<WORKER_B>>().await?) },
                async { imp::admitted(c.recv::<Intent<WORKER_C>>().await?) },
            )
            .await
        },
    )
    .await?;
    first_input.set(first.clone());
    work.send::<Loan0>(tool0).await?;
    let mut effect0 = match imp::agent(&first)? {
        "WORKER_A" => simulator.submit(work.recv::<Request0<WORKER_A>>().await?)?,
        "WORKER_B" => simulator.submit(work.recv::<Request0<WORKER_B>>().await?)?,
        "WORKER_C" => simulator.submit(work.recv::<Request0<WORKER_C>>().await?)?,
        _ => return Err(ShellError::message("unknown worker")),
    };
    // Borrowing an accepted RPC in a race never cancels or reissues that RPC.
    let arrived = hibana_runtime::race(
        async {
            let result = (&mut effect0).await?;
            imp::completed(&first, &result, None)?;
            Ok::<_, ShellError>(result)
        },
        async {
            match imp::agent(&first)? {
                "WORKER_A" => {
                    hibana_runtime::race(
                        async { imp::admitted(b.recv::<Intent<WORKER_B>>().await?) },
                        async { imp::admitted(c.recv::<Intent<WORKER_C>>().await?) },
                    )
                    .await
                }
                "WORKER_B" => {
                    hibana_runtime::race(
                        async { imp::admitted(a.recv::<Intent<WORKER_A>>().await?) },
                        async { imp::admitted(c.recv::<Intent<WORKER_C>>().await?) },
                    )
                    .await
                }
                _ => {
                    hibana_runtime::race(
                        async { imp::admitted(a.recv::<Intent<WORKER_A>>().await?) },
                        async { imp::admitted(b.recv::<Intent<WORKER_B>>().await?) },
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
        match imp::agent(&first)? {
            "WORKER_A" => {
                a.send::<StationResult<ALPHA_MACHINE>>(&bytes.as_slice())
                    .await?
            }
            "WORKER_B" => {
                b.send::<StationResult<BETA_MACHINE>>(&bytes.as_slice())
                    .await?
            }
            "WORKER_C" => {
                c.send::<StationResult<GAMMA_MACHINE>>(&bytes.as_slice())
                    .await?
            }
            _ => return Err(ShellError::message("unknown workstation")),
        }
        work.send::<Reply0>(&bytes.as_slice()).await?;
        let tool0 = work.recv::<Return0>().await?;
        let second = match imp::agent(&first)? {
            "WORKER_A" => {
                hibana_runtime::race(
                    async { imp::admitted(b.recv::<Intent<WORKER_B>>().await?) },
                    async { imp::admitted(c.recv::<Intent<WORKER_C>>().await?) },
                )
                .await?
            }
            "WORKER_B" => {
                hibana_runtime::race(
                    async { imp::admitted(a.recv::<Intent<WORKER_A>>().await?) },
                    async { imp::admitted(c.recv::<Intent<WORKER_C>>().await?) },
                )
                .await?
            }
            _ => {
                hibana_runtime::race(
                    async { imp::admitted(a.recv::<Intent<WORKER_A>>().await?) },
                    async { imp::admitted(b.recv::<Intent<WORKER_B>>().await?) },
                )
                .await?
            }
        };
        let (_, mut second_station, mut last_station) =
            match (imp::agent(&first)?, imp::agent(&second)?) {
                ("WORKER_A", "WORKER_B") => (a, b, c),
                ("WORKER_A", "WORKER_C") => (a, c, b),
                ("WORKER_B", "WORKER_A") => (b, a, c),
                ("WORKER_B", "WORKER_C") => (b, c, a),
                ("WORKER_C", "WORKER_A") => (c, a, b),
                ("WORKER_C", "WORKER_B") => (c, b, a),
                _ => return Err(ShellError::message("duplicate workstation submission")),
            };
        next_input.set(second.clone());
        work.send::<Loan0>(tool0).await?;
        let mut effect = match imp::agent(&second)? {
            "WORKER_A" => simulator.submit(work.recv::<Request0<WORKER_A>>().await?)?,
            "WORKER_B" => simulator.submit(work.recv::<Request0<WORKER_B>>().await?)?,
            "WORKER_C" => simulator.submit(work.recv::<Request0<WORKER_C>>().await?)?,
            _ => return Err(ShellError::message("unknown worker")),
        };
        let arrived = hibana_runtime::race(
            async {
                let result = (&mut effect).await?;
                imp::completed(&second, &result, None)?;
                Ok::<_, ShellError>(result)
            },
            async {
                if first["agent"] != "WORKER_A" && second["agent"] != "WORKER_A" {
                    imp::admitted(last_station.recv::<Intent<WORKER_A>>().await?)
                } else if first["agent"] != "WORKER_B" && second["agent"] != "WORKER_B" {
                    imp::admitted(last_station.recv::<Intent<WORKER_B>>().await?)
                } else {
                    imp::admitted(last_station.recv::<Intent<WORKER_C>>().await?)
                }
            },
        )
        .await?;
        arrival1_input.set(arrived.clone());
        if arrived["tool_name"] == "do_job" {
            let bytes = imp::encode(&arrived)?;
            match imp::agent(&second)? {
                "WORKER_A" => {
                    second_station
                        .send::<StationResult<ALPHA_MACHINE>>(&bytes.as_slice())
                        .await?
                }
                "WORKER_B" => {
                    second_station
                        .send::<StationResult<BETA_MACHINE>>(&bytes.as_slice())
                        .await?
                }
                "WORKER_C" => {
                    second_station
                        .send::<StationResult<GAMMA_MACHINE>>(&bytes.as_slice())
                        .await?
                }
                _ => return Err(ShellError::message("unknown workstation")),
            }
            work.send::<Reply0>(&bytes.as_slice()).await?;
            let tool0 = work.recv::<Return0>().await?;
            let third = if first["agent"] != "WORKER_A" && second["agent"] != "WORKER_A" {
                imp::admitted(last_station.recv::<Intent<WORKER_A>>().await?)?
            } else if first["agent"] != "WORKER_B" && second["agent"] != "WORKER_B" {
                imp::admitted(last_station.recv::<Intent<WORKER_B>>().await?)?
            } else {
                imp::admitted(last_station.recv::<Intent<WORKER_C>>().await?)?
            };
            work.send::<Loan0>(tool0).await?;
            let result = match imp::agent(&third)? {
                "WORKER_A" => simulator.submit(work.recv::<Request0<WORKER_A>>().await?)?,
                "WORKER_B" => simulator.submit(work.recv::<Request0<WORKER_B>>().await?)?,
                "WORKER_C" => simulator.submit(work.recv::<Request0<WORKER_C>>().await?)?,
                _ => return Err(ShellError::message("unknown worker")),
            }
            .await?;
            imp::completed(&third, &result, None)?;
            let bytes = imp::encode(&result)?;
            match imp::agent(&third)? {
                "WORKER_A" => {
                    last_station
                        .send::<StationResult<ALPHA_MACHINE>>(&bytes.as_slice())
                        .await?
                }
                "WORKER_B" => {
                    last_station
                        .send::<StationResult<BETA_MACHINE>>(&bytes.as_slice())
                        .await?
                }
                "WORKER_C" => {
                    last_station
                        .send::<StationResult<GAMMA_MACHINE>>(&bytes.as_slice())
                        .await?
                }
                _ => return Err(ShellError::message("unknown workstation")),
            }
            work.send::<Reply0>(&bytes.as_slice()).await?;
            drop(work.recv::<Return0>().await?);
        } else {
            // This route's two arms are independent; no two-job batch join.
            async {
                drop(work.offer().await?);
                let [mut finishing, mut third] = work.par()?;
                async {
                    let result = effect.await?;
                    imp::completed(&second, &result, None)?;
                    let bytes = imp::encode(&result)?;
                    match imp::agent(&second)? {
                        "WORKER_A" => {
                            second_station
                                .send::<StationResult<ALPHA_MACHINE>>(&bytes.as_slice())
                                .await?
                        }
                        "WORKER_B" => {
                            second_station
                                .send::<StationResult<BETA_MACHINE>>(&bytes.as_slice())
                                .await?
                        }
                        "WORKER_C" => {
                            second_station
                                .send::<StationResult<GAMMA_MACHINE>>(&bytes.as_slice())
                                .await?
                        }
                        _ => return Err(ShellError::message("unknown workstation")),
                    }
                    finishing.send::<Reply0>(&bytes.as_slice()).await?;
                    drop(finishing.recv::<Return0>().await?);
                };
                async {
                    third.send::<Loan1>(tool1).await?;
                    let result = match imp::agent(&arrived)? {
                        "WORKER_A" => {
                            simulator.submit(third.recv::<Request1<WORKER_A>>().await?)?
                        }
                        "WORKER_B" => {
                            simulator.submit(third.recv::<Request1<WORKER_B>>().await?)?
                        }
                        "WORKER_C" => {
                            simulator.submit(third.recv::<Request1<WORKER_C>>().await?)?
                        }
                        _ => return Err(ShellError::message("unknown worker")),
                    }
                    .await?;
                    imp::completed(&arrived, &result, None)?;
                    let bytes = imp::encode(&result)?;
                    match imp::agent(&arrived)? {
                        "WORKER_A" => {
                            last_station
                                .send::<StationResult<ALPHA_MACHINE>>(&bytes.as_slice())
                                .await?
                        }
                        "WORKER_B" => {
                            last_station
                                .send::<StationResult<BETA_MACHINE>>(&bytes.as_slice())
                                .await?
                        }
                        "WORKER_C" => {
                            last_station
                                .send::<StationResult<GAMMA_MACHINE>>(&bytes.as_slice())
                                .await?
                        }
                        _ => return Err(ShellError::message("unknown workstation")),
                    }
                    third.send::<Reply1>(&bytes.as_slice()).await?;
                    drop(third.recv::<Return1>().await?);
                };
                Ok::<(), ShellError>(())
            }
            .await?;
        }
    } else {
        let second = arrived;
        let (mut first_station, mut second_station, mut last_station) =
            match (imp::agent(&first)?, imp::agent(&second)?) {
                ("WORKER_A", "WORKER_B") => (a, b, c),
                ("WORKER_A", "WORKER_C") => (a, c, b),
                ("WORKER_B", "WORKER_A") => (b, a, c),
                ("WORKER_B", "WORKER_C") => (b, c, a),
                ("WORKER_C", "WORKER_A") => (c, a, b),
                ("WORKER_C", "WORKER_B") => (c, b, a),
                _ => return Err(ShellError::message("duplicate workstation submission")),
            };
        next_input.set(second.clone());
        work.send::<Loan1>(tool1).await?;
        let mut effect1 = match imp::agent(&second)? {
            "WORKER_A" => simulator.submit(work.recv::<Request1<WORKER_A>>().await?)?,
            "WORKER_B" => simulator.submit(work.recv::<Request1<WORKER_B>>().await?)?,
            "WORKER_C" => simulator.submit(work.recv::<Request1<WORKER_C>>().await?)?,
            _ => return Err(ShellError::message("unknown worker")),
        };
        let completed = hibana_runtime::race(
            async {
                let result = (&mut effect0).await?;
                imp::completed(&first, &result, None)?;
                Ok::<_, ShellError>((TOOL_0, result))
            },
            async {
                let result = (&mut effect1).await?;
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
                    match imp::agent(&first)? {
                        "WORKER_A" => {
                            first_station
                                .send::<StationResult<ALPHA_MACHINE>>(&bytes.as_slice())
                                .await?
                        }
                        "WORKER_B" => {
                            first_station
                                .send::<StationResult<BETA_MACHINE>>(&bytes.as_slice())
                                .await?
                        }
                        "WORKER_C" => {
                            first_station
                                .send::<StationResult<GAMMA_MACHINE>>(&bytes.as_slice())
                                .await?
                        }
                        _ => return Err(ShellError::message("unknown workstation")),
                    }
                    available.send::<Reply0>(&bytes.as_slice()).await?;
                    let tool = available.recv::<Return0>().await?;
                    let third = if first["agent"] != "WORKER_A" && second["agent"] != "WORKER_A" {
                        imp::admitted(last_station.recv::<Intent<WORKER_A>>().await?)?
                    } else if first["agent"] != "WORKER_B" && second["agent"] != "WORKER_B" {
                        imp::admitted(last_station.recv::<Intent<WORKER_B>>().await?)?
                    } else {
                        imp::admitted(last_station.recv::<Intent<WORKER_C>>().await?)?
                    };
                    available.send::<Loan0>(tool).await?;
                    let result = match imp::agent(&third)? {
                        "WORKER_A" => {
                            simulator.submit(available.recv::<Request0<WORKER_A>>().await?)?
                        }
                        "WORKER_B" => {
                            simulator.submit(available.recv::<Request0<WORKER_B>>().await?)?
                        }
                        "WORKER_C" => {
                            simulator.submit(available.recv::<Request0<WORKER_C>>().await?)?
                        }
                        _ => return Err(ShellError::message("unknown worker")),
                    }
                    .await?;
                    imp::completed(&third, &result, None)?;
                    let bytes = imp::encode(&result)?;
                    match imp::agent(&third)? {
                        "WORKER_A" => {
                            last_station
                                .send::<StationResult<ALPHA_MACHINE>>(&bytes.as_slice())
                                .await?
                        }
                        "WORKER_B" => {
                            last_station
                                .send::<StationResult<BETA_MACHINE>>(&bytes.as_slice())
                                .await?
                        }
                        "WORKER_C" => {
                            last_station
                                .send::<StationResult<GAMMA_MACHINE>>(&bytes.as_slice())
                                .await?
                        }
                        _ => return Err(ShellError::message("unknown workstation")),
                    }
                    available.send::<Reply0>(&bytes.as_slice()).await?;
                    drop(available.recv::<Return0>().await?);
                };
                async {
                    let result = effect1.await?;
                    imp::completed(&second, &result, None)?;
                    let bytes = imp::encode(&result)?;
                    match imp::agent(&second)? {
                        "WORKER_A" => {
                            second_station
                                .send::<StationResult<ALPHA_MACHINE>>(&bytes.as_slice())
                                .await?
                        }
                        "WORKER_B" => {
                            second_station
                                .send::<StationResult<BETA_MACHINE>>(&bytes.as_slice())
                                .await?
                        }
                        "WORKER_C" => {
                            second_station
                                .send::<StationResult<GAMMA_MACHINE>>(&bytes.as_slice())
                                .await?
                        }
                        _ => return Err(ShellError::message("unknown workstation")),
                    }
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
                    let result = effect0.await?;
                    imp::completed(&first, &result, None)?;
                    let bytes = imp::encode(&result)?;
                    match imp::agent(&first)? {
                        "WORKER_A" => {
                            first_station
                                .send::<StationResult<ALPHA_MACHINE>>(&bytes.as_slice())
                                .await?
                        }
                        "WORKER_B" => {
                            first_station
                                .send::<StationResult<BETA_MACHINE>>(&bytes.as_slice())
                                .await?
                        }
                        "WORKER_C" => {
                            first_station
                                .send::<StationResult<GAMMA_MACHINE>>(&bytes.as_slice())
                                .await?
                        }
                        _ => return Err(ShellError::message("unknown workstation")),
                    }
                    finishing.send::<Reply0>(&bytes.as_slice()).await?;
                    drop(finishing.recv::<Return0>().await?);
                };
                async {
                    let bytes = imp::encode(&completed.1)?;
                    match imp::agent(&second)? {
                        "WORKER_A" => {
                            second_station
                                .send::<StationResult<ALPHA_MACHINE>>(&bytes.as_slice())
                                .await?
                        }
                        "WORKER_B" => {
                            second_station
                                .send::<StationResult<BETA_MACHINE>>(&bytes.as_slice())
                                .await?
                        }
                        "WORKER_C" => {
                            second_station
                                .send::<StationResult<GAMMA_MACHINE>>(&bytes.as_slice())
                                .await?
                        }
                        _ => return Err(ShellError::message("unknown workstation")),
                    }
                    available.send::<Reply1>(&bytes.as_slice()).await?;
                    let tool = available.recv::<Return1>().await?;
                    let third = if first["agent"] != "WORKER_A" && second["agent"] != "WORKER_A" {
                        imp::admitted(last_station.recv::<Intent<WORKER_A>>().await?)?
                    } else if first["agent"] != "WORKER_B" && second["agent"] != "WORKER_B" {
                        imp::admitted(last_station.recv::<Intent<WORKER_B>>().await?)?
                    } else {
                        imp::admitted(last_station.recv::<Intent<WORKER_C>>().await?)?
                    };
                    available.send::<Loan1>(tool).await?;
                    let result = match imp::agent(&third)? {
                        "WORKER_A" => {
                            simulator.submit(available.recv::<Request1<WORKER_A>>().await?)?
                        }
                        "WORKER_B" => {
                            simulator.submit(available.recv::<Request1<WORKER_B>>().await?)?
                        }
                        "WORKER_C" => {
                            simulator.submit(available.recv::<Request1<WORKER_C>>().await?)?
                        }
                        _ => return Err(ShellError::message("unknown worker")),
                    }
                    .await?;
                    imp::completed(&third, &result, None)?;
                    let bytes = imp::encode(&result)?;
                    match imp::agent(&third)? {
                        "WORKER_A" => {
                            last_station
                                .send::<StationResult<ALPHA_MACHINE>>(&bytes.as_slice())
                                .await?
                        }
                        "WORKER_B" => {
                            last_station
                                .send::<StationResult<BETA_MACHINE>>(&bytes.as_slice())
                                .await?
                        }
                        "WORKER_C" => {
                            last_station
                                .send::<StationResult<GAMMA_MACHINE>>(&bytes.as_slice())
                                .await?
                        }
                        _ => return Err(ShellError::message("unknown workstation")),
                    }
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
