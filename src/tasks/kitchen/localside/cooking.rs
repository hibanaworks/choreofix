//! Own the actual equipment and execute only the matching received requests.
use super::super::{global::*, imp};
use crate::shell::{
    ShellError, ShellResult,
    imp::{bus::Bus, service::Domain},
};
use hibana::Endpoint;
use serde_json::Value;
use std::cell::Cell;

#[hibana_runtime::scope]
pub async fn run<'r>(
    endpoint: &mut Endpoint<'r, COOKING>,
    (oven, stove): (Endpoint<'r, OVEN>, Endpoint<'r, STOVE>),
    (domain, inputs): (&Bus, &[Cell<Value>; 2]),
) -> ShellResult<()> {
    let [first_input, next_input] = inputs;
    let [mut a, mut b, mut c, mut work] = endpoint.par()?;
    let first = hibana_runtime::race(
        async { Ok::<_, ShellError>(imp::admitted(&a.recv::<BaseIntentA>().await?)) },
        async {
            hibana_runtime::race(
                async { Ok::<_, ShellError>(imp::admitted(&b.recv::<BaseIntentB>().await?)) },
                async { Ok::<_, ShellError>(imp::admitted(&c.recv::<BaseIntentC>().await?)) },
            )
            .await
        },
    )
    .await?;
    first_input.set(first.clone());
    let chef = imp::chef(&first)?;
    drop(work.offer().await?);
    let (oven, stove) = if chef == "CHEF_C" {
        work.send::<OvenLoan>(oven).await?;
        let arguments = imp::arguments(&first, work.recv::<OvenRequestC>().await?.value())?;
        let result = domain
            .execute(chef, "prepare_base_dish", arguments.clone())
            .await
            .map_err(ShellError::message)?;
        imp::completed("prepare_base_dish", chef, &arguments, &result)?;
        let bytes = imp::encode(&result)?;
        work.send::<OvenResult>(&bytes.as_slice()).await?;
        (work.recv::<ReturnOven>().await?, stove)
    } else {
        async {
            let [mut o, mut s] = work.par()?;
            async {
                o.send::<OvenLoan>(oven).await?;
            };
            async {
                s.send::<StoveLoan>(stove).await?;
            };
            Ok::<(), ShellError>(())
        }
        .await?;
        let (oven_arguments, stove_arguments) = async {
            let [mut o, mut s] = work.par()?;
            async {
                match chef {
                    "CHEF_A" => Ok::<_, ShellError>(o.recv::<OvenRequestA>().await?.value()),
                    _ => Ok(o.recv::<OvenRequestB>().await?.value()),
                }
            };
            async {
                match chef {
                    "CHEF_A" => Ok::<_, ShellError>(s.recv::<StoveRequestA>().await?.value()),
                    _ => Ok(s.recv::<StoveRequestB>().await?.value()),
                }
            };
        }
        .await?;
        let arguments = imp::arguments(&first, oven_arguments)?;
        imp::arguments(&first, stove_arguments)?;
        let result = domain
            .execute(chef, "prepare_base_dish", arguments.clone())
            .await
            .map_err(ShellError::message)?;
        imp::completed("prepare_base_dish", chef, &arguments, &result)?;
        let bytes = imp::encode(&result)?;
        async {
            let [mut o, mut s] = work.par()?;
            async {
                o.send::<OvenResult>(&bytes.as_slice()).await?;
            };
            async {
                s.send::<StoveResult>(&bytes.as_slice()).await?;
            };
            Ok::<(), ShellError>(())
        }
        .await?;
        async {
            let [mut o, mut s] = work.par()?;
            async { o.recv::<ReturnOven>().await };
            async { s.recv::<ReturnStove>().await };
        }
        .await?
    };
    let second = match imp::chef(&first)? {
        "CHEF_A" => {
            hibana_runtime::race(
                async { Ok::<_, ShellError>(imp::admitted(&b.recv::<BaseIntentB>().await?)) },
                async { Ok::<_, ShellError>(imp::admitted(&c.recv::<BaseIntentC>().await?)) },
            )
            .await?
        }
        "CHEF_B" => {
            hibana_runtime::race(
                async { Ok::<_, ShellError>(imp::admitted(&a.recv::<BaseIntentA>().await?)) },
                async { Ok::<_, ShellError>(imp::admitted(&c.recv::<BaseIntentC>().await?)) },
            )
            .await?
        }
        _ => {
            hibana_runtime::race(
                async { Ok::<_, ShellError>(imp::admitted(&a.recv::<BaseIntentA>().await?)) },
                async { Ok::<_, ShellError>(imp::admitted(&b.recv::<BaseIntentB>().await?)) },
            )
            .await?
        }
    };
    next_input.set(second.clone());
    let chef = imp::chef(&second)?;
    drop(work.offer().await?);
    let (oven, stove) = if chef == "CHEF_C" {
        work.send::<OvenLoan>(oven).await?;
        let arguments = imp::arguments(&second, work.recv::<OvenRequestC>().await?.value())?;
        let result = domain
            .execute(chef, "prepare_base_dish", arguments.clone())
            .await
            .map_err(ShellError::message)?;
        imp::completed("prepare_base_dish", chef, &arguments, &result)?;
        let bytes = imp::encode(&result)?;
        work.send::<OvenResult>(&bytes.as_slice()).await?;
        (work.recv::<ReturnOven>().await?, stove)
    } else {
        async {
            let [mut o, mut s] = work.par()?;
            async {
                o.send::<OvenLoan>(oven).await?;
            };
            async {
                s.send::<StoveLoan>(stove).await?;
            };
            Ok::<(), ShellError>(())
        }
        .await?;
        let (oven_arguments, stove_arguments) = async {
            let [mut o, mut s] = work.par()?;
            async {
                match chef {
                    "CHEF_A" => Ok::<_, ShellError>(o.recv::<OvenRequestA>().await?.value()),
                    _ => Ok(o.recv::<OvenRequestB>().await?.value()),
                }
            };
            async {
                match chef {
                    "CHEF_A" => Ok::<_, ShellError>(s.recv::<StoveRequestA>().await?.value()),
                    _ => Ok(s.recv::<StoveRequestB>().await?.value()),
                }
            };
        }
        .await?;
        let arguments = imp::arguments(&second, oven_arguments)?;
        imp::arguments(&second, stove_arguments)?;
        let result = domain
            .execute(chef, "prepare_base_dish", arguments.clone())
            .await
            .map_err(ShellError::message)?;
        imp::completed("prepare_base_dish", chef, &arguments, &result)?;
        let bytes = imp::encode(&result)?;
        async {
            let [mut o, mut s] = work.par()?;
            async {
                o.send::<OvenResult>(&bytes.as_slice()).await?;
            };
            async {
                s.send::<StoveResult>(&bytes.as_slice()).await?;
            };
            Ok::<(), ShellError>(())
        }
        .await?;
        async {
            let [mut o, mut s] = work.par()?;
            async { o.recv::<ReturnOven>().await };
            async { s.recv::<ReturnStove>().await };
        }
        .await?
    };
    let third = if first["chef"] != "CHEF_A" && second["chef"] != "CHEF_A" {
        imp::admitted(&a.recv::<BaseIntentA>().await?)
    } else if first["chef"] != "CHEF_B" && second["chef"] != "CHEF_B" {
        imp::admitted(&b.recv::<BaseIntentB>().await?)
    } else {
        imp::admitted(&c.recv::<BaseIntentC>().await?)
    };
    let chef = imp::chef(&third)?;
    drop(work.offer().await?);
    let (oven, stove) = if chef == "CHEF_C" {
        work.send::<OvenLoan>(oven).await?;
        let arguments = imp::arguments(&third, work.recv::<OvenRequestC>().await?.value())?;
        let result = domain
            .execute(chef, "prepare_base_dish", arguments.clone())
            .await
            .map_err(ShellError::message)?;
        imp::completed("prepare_base_dish", chef, &arguments, &result)?;
        let bytes = imp::encode(&result)?;
        work.send::<OvenResult>(&bytes.as_slice()).await?;
        (work.recv::<ReturnOven>().await?, stove)
    } else {
        async {
            let [mut o, mut s] = work.par()?;
            async {
                o.send::<OvenLoan>(oven).await?;
            };
            async {
                s.send::<StoveLoan>(stove).await?;
            };
            Ok::<(), ShellError>(())
        }
        .await?;
        let (oven_arguments, stove_arguments) = async {
            let [mut o, mut s] = work.par()?;
            async {
                match chef {
                    "CHEF_A" => Ok::<_, ShellError>(o.recv::<OvenRequestA>().await?.value()),
                    _ => Ok(o.recv::<OvenRequestB>().await?.value()),
                }
            };
            async {
                match chef {
                    "CHEF_A" => Ok::<_, ShellError>(s.recv::<StoveRequestA>().await?.value()),
                    _ => Ok(s.recv::<StoveRequestB>().await?.value()),
                }
            };
        }
        .await?;
        let arguments = imp::arguments(&third, oven_arguments)?;
        imp::arguments(&third, stove_arguments)?;
        let result = domain
            .execute(chef, "prepare_base_dish", arguments.clone())
            .await
            .map_err(ShellError::message)?;
        imp::completed("prepare_base_dish", chef, &arguments, &result)?;
        let bytes = imp::encode(&result)?;
        async {
            let [mut o, mut s] = work.par()?;
            async {
                o.send::<OvenResult>(&bytes.as_slice()).await?;
            };
            async {
                s.send::<StoveResult>(&bytes.as_slice()).await?;
            };
            Ok::<(), ShellError>(())
        }
        .await?;
        async {
            let [mut o, mut s] = work.par()?;
            async { o.recv::<ReturnOven>().await };
            async { s.recv::<ReturnStove>().await };
        }
        .await?
    };
    drop((oven, stove));
    Ok(())
}
