//! The receiver owns the accepted RPC while serving independent busy requests.
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
    endpoint: &mut Endpoint<'r, MILL_SERVICE>,
    (mill, request_a, request_b): (
        Endpoint<'r, MILL>,
        Endpoint<'r, MILL_REQUEST_A>,
        Endpoint<'r, MILL_REQUEST_B>,
    ),
    (domain, first, arrival): (&Bus, &Cell<Value>, &Cell<Value>),
) -> ShellResult<()> {
    let [mut a, mut b, mut work] = endpoint.par()?;
    let admitted = hibana_runtime::race(
        async {
            let arguments = a.recv::<MillIntentA>().await?.into_value();
            Ok::<_, ShellError>(json!({"agent":"TECHNICIAN_A","arguments":arguments}))
        },
        async {
            let arguments = b.recv::<MillIntentB>().await?.into_value();
            Ok::<_, ShellError>(json!({"agent":"TECHNICIAN_B","arguments":arguments}))
        },
    )
    .await?;
    first.set(admitted.clone());
    let remaining_request = if admitted["agent"] == "TECHNICIAN_A" {
        work.send::<MillRequestLoanA>(request_a).await?;
        work.send::<MillReady>(&()).await?;
        drop(work.recv::<ReturnMillRequestA>().await?);
        Err(request_b)
    } else {
        work.send::<MillRequestLoanB>(request_b).await?;
        work.send::<MillReady>(&()).await?;
        drop(work.recv::<ReturnMillRequestB>().await?);
        Ok(request_a)
    };
    work.send::<MillLoan>(mill).await?;
    let (agent, arguments) = if admitted["agent"] == "TECHNICIAN_A" {
        ("TECHNICIAN_A", work.recv::<MillJobA>().await?.into_value())
    } else {
        ("TECHNICIAN_B", work.recv::<MillJobB>().await?.into_value())
    };
    if arguments != admitted["arguments"] {
        return Err(ShellError::message(
            "mill request differs from admitted arguments",
        ));
    }
    // The concrete RPC remains owned here when a borrowed wait is cancelled.
    let mut effect = domain.execute(agent, "do_job", arguments.clone());
    let arrived = hibana_runtime::race(
        async {
            let result = (&mut effect).await.map_err(ShellError::message)?;
            imp::completed(&result, agent, "MILL", &arguments)?;
            Ok::<_, ShellError>(result)
        },
        async {
            let request = if agent == "TECHNICIAN_A" {
                let arguments = b.recv::<MillIntentB>().await?.into_value();
                json!({"agent":"TECHNICIAN_B","arguments":arguments})
            } else {
                let arguments = a.recv::<MillIntentA>().await?.into_value();
                json!({"agent":"TECHNICIAN_A","arguments":arguments})
            };
            Ok::<_, ShellError>(request)
        },
    )
    .await?;
    arrival.set(arrived.clone());
    let (mill, next) = if arrived["tool_name"] == "do_job" {
        let bytes = imp::encode(&arrived)?;
        work.send::<MillResult>(&bytes.as_slice()).await?;
        let mill = work.recv::<ReturnMill>().await?;
        let next = if agent == "TECHNICIAN_A" {
            let arguments = b.recv::<MillIntentB>().await?.into_value();
            json!({"agent":"TECHNICIAN_B","arguments":arguments})
        } else {
            let arguments = a.recv::<MillIntentA>().await?.into_value();
            json!({"agent":"TECHNICIAN_A","arguments":arguments})
        };
        match remaining_request {
            Ok(request) => work.send::<MillRequestLoanA>(request).await?,
            Err(request) => work.send::<MillRequestLoanB>(request).await?,
        }
        (mill, next)
    } else {
        // Resolve from the actual input before borrowing the parallel arms.
        drop(work.offer().await?);
        let [mut finish, mut retry] = work.par()?;
        match remaining_request {
            Ok(request) => retry.send::<MillRequestLoanA>(request).await?,
            Err(request) => retry.send::<MillRequestLoanB>(request).await?,
        }
        retry.send::<MillBusy>(&()).await?;
        // A roll has a required first visit. Preserve both independent arms
        // even when the actual effect and return beat the first retry.
        let mill = 'returned: {
            let first = hibana_runtime::race(
                async {
                    let result = (&mut effect).await.map_err(ShellError::message)?;
                    imp::completed(&result, agent, "MILL", &arguments)?;
                    Ok::<_, ShellError>(result)
                },
                async {
                    let arguments = if agent == "TECHNICIAN_A" {
                        retry.recv::<MillRetryB>().await?.into_value()
                    } else {
                        retry.recv::<MillRetryA>().await?.into_value()
                    };

                    Ok::<_, ShellError>(json!({"arguments":arguments}))
                },
            )
            .await?;
            if first["tool_name"] == "do_job" {
                let bytes = imp::encode(&first)?;
                finish.send::<MillResult>(&bytes.as_slice()).await?;
                let received = hibana_runtime::race(
                    async { Ok::<_, ShellError>(Ok(finish.recv::<ReturnMill>().await?)) },
                    async {
                        let arguments = if agent == "TECHNICIAN_A" {
                            retry.recv::<MillRetryB>().await?.into_value()
                        } else {
                            retry.recv::<MillRetryA>().await?.into_value()
                        };

                        Ok::<_, ShellError>(Err(arguments))
                    },
                )
                .await?;
                match received {
                    Ok(mill) => {
                        let arguments = if agent == "TECHNICIAN_A" {
                            retry.recv::<MillRetryB>().await?.into_value()
                        } else {
                            retry.recv::<MillRetryA>().await?.into_value()
                        };

                        if arguments != arrived["arguments"] {
                            return Err(ShellError::message("mill retry changed the admitted job"));
                        }
                        retry.send::<MillBusy>(&()).await?;

                        break 'returned mill;
                    }
                    Err(arguments) => {
                        if arguments != arrived["arguments"] {
                            return Err(ShellError::message("mill retry changed the admitted job"));
                        }
                        retry.send::<MillBusy>(&()).await?;
                    }
                }
            } else {
                let first_retry_arguments = first["arguments"].clone();
                if first_retry_arguments != arrived["arguments"] {
                    return Err(ShellError::message("mill retry changed the admitted job"));
                }
                retry.send::<MillBusy>(&()).await?;

                let result = loop {
                    let response = hibana_runtime::race(
                        async {
                            let result = (&mut effect).await.map_err(ShellError::message)?;
                            imp::completed(&result, agent, "MILL", &arguments)?;
                            Ok::<_, ShellError>(result)
                        },
                        async {
                            let arguments = if agent == "TECHNICIAN_A" {
                                retry.recv::<MillRetryB>().await?.into_value()
                            } else {
                                retry.recv::<MillRetryA>().await?.into_value()
                            };

                            Ok::<_, ShellError>(json!({"arguments":arguments}))
                        },
                    )
                    .await?;
                    if response["tool_name"] == "do_job" {
                        break response;
                    }
                    let arguments = response["arguments"].clone();
                    if arguments != arrived["arguments"] {
                        return Err(ShellError::message("mill retry changed the admitted job"));
                    }
                    retry.send::<MillBusy>(&()).await?;
                };
                let bytes = imp::encode(&result)?;
                finish.send::<MillResult>(&bytes.as_slice()).await?;
            }
            loop {
                let received = hibana_runtime::race(
                    async { Ok::<_, ShellError>(Ok(finish.recv::<ReturnMill>().await?)) },
                    async {
                        let arguments = if agent == "TECHNICIAN_A" {
                            retry.recv::<MillRetryB>().await?.into_value()
                        } else {
                            retry.recv::<MillRetryA>().await?.into_value()
                        };

                        Ok::<_, ShellError>(Err(arguments))
                    },
                )
                .await?;
                match received {
                    Ok(mill) => break mill,
                    Err(arguments) => {
                        if arguments != arrived["arguments"] {
                            return Err(ShellError::message("mill retry changed the admitted job"));
                        }
                        retry.send::<MillBusy>(&()).await?;
                    }
                }
            }
        };
        (mill, arrived)
    };
    work.send::<MillReady>(&()).await?;
    loop {
        // A retry already in flight may cross Ready. Its affine query return,
        // rather than the notification, retires that remaining continuation.
        let branch = work.offer().await?;
        use hibana::g::Message;
        match branch.label() {
            ReturnMillRequestA::LOGICAL_LABEL => {
                drop(branch.recv::<ReturnMillRequestA>().await?);
                break;
            }
            ReturnMillRequestB::LOGICAL_LABEL => {
                drop(branch.recv::<ReturnMillRequestB>().await?);
                break;
            }
            MillRetryA::LOGICAL_LABEL => {
                let arguments = branch.recv::<MillRetryA>().await?.into_value();
                if arguments != next["arguments"] {
                    return Err(ShellError::message("mill retry changed the admitted job"));
                }
                work.send::<MillBusy>(&()).await?;
            }
            MillRetryB::LOGICAL_LABEL => {
                let arguments = branch.recv::<MillRetryB>().await?.into_value();
                if arguments != next["arguments"] {
                    return Err(ShellError::message("mill retry changed the admitted job"));
                }
                work.send::<MillBusy>(&()).await?;
            }
            _ => return Err(ShellError::message("unexpected mill request continuation")),
        }
    }
    work.send::<MillLoan>(mill).await?;
    let (agent, arguments) = if next["agent"] == "TECHNICIAN_A" {
        ("TECHNICIAN_A", work.recv::<MillJobA>().await?.into_value())
    } else {
        ("TECHNICIAN_B", work.recv::<MillJobB>().await?.into_value())
    };
    if arguments != next["arguments"] {
        return Err(ShellError::message(
            "mill request differs from admitted arguments",
        ));
    }
    let result = domain
        .execute(agent, "do_job", arguments.clone())
        .await
        .map_err(ShellError::message)?;
    imp::completed(&result, agent, "MILL", &arguments)?;
    let bytes = imp::encode(&result)?;
    work.send::<MillResult>(&bytes.as_slice()).await?;
    drop(work.recv::<ReturnMill>().await?);
    Ok(())
}
