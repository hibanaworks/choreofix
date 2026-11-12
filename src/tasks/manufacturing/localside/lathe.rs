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
    endpoint: &mut Endpoint<'r, LATHE_SERVICE>,
    (lathe, request_a, request_b): (
        Endpoint<'r, LATHE>,
        Endpoint<'r, LATHE_REQUEST_A>,
        Endpoint<'r, LATHE_REQUEST_B>,
    ),
    (domain, first, arrival): (&Bus, &Cell<Value>, &Cell<Value>),
) -> ShellResult<()> {
    let [mut a, mut b, mut work] = endpoint.par()?;
    let admitted = hibana_runtime::race(
        async {
            let arguments = a.recv::<LatheIntentA>().await?.into_value();
            Ok::<_, ShellError>(json!({"agent":"TECHNICIAN_A","arguments":arguments}))
        },
        async {
            let arguments = b.recv::<LatheIntentB>().await?.into_value();
            Ok::<_, ShellError>(json!({"agent":"TECHNICIAN_B","arguments":arguments}))
        },
    )
    .await?;
    first.set(admitted.clone());
    let remaining_request = if admitted["agent"] == "TECHNICIAN_A" {
        work.send::<LatheRequestLoanA>(request_a).await?;
        work.send::<LatheReady>(&()).await?;
        drop(work.recv::<ReturnLatheRequestA>().await?);
        Err(request_b)
    } else {
        work.send::<LatheRequestLoanB>(request_b).await?;
        work.send::<LatheReady>(&()).await?;
        drop(work.recv::<ReturnLatheRequestB>().await?);
        Ok(request_a)
    };
    work.send::<LatheLoan>(lathe).await?;
    let (agent, arguments) = if admitted["agent"] == "TECHNICIAN_A" {
        ("TECHNICIAN_A", work.recv::<LatheJobA>().await?.into_value())
    } else {
        ("TECHNICIAN_B", work.recv::<LatheJobB>().await?.into_value())
    };
    if arguments != admitted["arguments"] {
        return Err(ShellError::message(
            "lathe request differs from admitted arguments",
        ));
    }
    // The concrete RPC remains owned here when a borrowed wait is cancelled.
    let mut effect = domain.execute(agent, "do_job", arguments.clone());
    let arrived = hibana_runtime::race(
        async {
            let result = (&mut effect).await.map_err(ShellError::message)?;
            imp::completed(&result, agent, "LATHE", &arguments)?;
            Ok::<_, ShellError>(result)
        },
        async {
            let request = if agent == "TECHNICIAN_A" {
                let arguments = b.recv::<LatheIntentB>().await?.into_value();
                json!({"agent":"TECHNICIAN_B","arguments":arguments})
            } else {
                let arguments = a.recv::<LatheIntentA>().await?.into_value();
                json!({"agent":"TECHNICIAN_A","arguments":arguments})
            };
            Ok::<_, ShellError>(request)
        },
    )
    .await?;
    arrival.set(arrived.clone());
    let (lathe, next) = if arrived["tool_name"] == "do_job" {
        let bytes = imp::encode(&arrived)?;
        work.send::<LatheResult>(&bytes.as_slice()).await?;
        let lathe = work.recv::<ReturnLathe>().await?;
        let next = if agent == "TECHNICIAN_A" {
            let arguments = b.recv::<LatheIntentB>().await?.into_value();
            json!({"agent":"TECHNICIAN_B","arguments":arguments})
        } else {
            let arguments = a.recv::<LatheIntentA>().await?.into_value();
            json!({"agent":"TECHNICIAN_A","arguments":arguments})
        };
        match remaining_request {
            Ok(request) => work.send::<LatheRequestLoanA>(request).await?,
            Err(request) => work.send::<LatheRequestLoanB>(request).await?,
        }
        (lathe, next)
    } else {
        // Resolve from the actual input before borrowing the parallel arms.
        drop(work.offer().await?);
        let [mut finish, mut retry] = work.par()?;
        match remaining_request {
            Ok(request) => retry.send::<LatheRequestLoanA>(request).await?,
            Err(request) => retry.send::<LatheRequestLoanB>(request).await?,
        }
        retry.send::<LatheBusy>(&()).await?;
        // A roll has a required first visit. Preserve both independent arms
        // even when the actual effect and return beat the first retry.
        let lathe = 'returned: {
            let first = hibana_runtime::race(
                async {
                    let result = (&mut effect).await.map_err(ShellError::message)?;
                    imp::completed(&result, agent, "LATHE", &arguments)?;
                    Ok::<_, ShellError>(result)
                },
                async {
                    let arguments = if agent == "TECHNICIAN_A" {
                        retry.recv::<LatheRetryB>().await?.into_value()
                    } else {
                        retry.recv::<LatheRetryA>().await?.into_value()
                    };

                    Ok::<_, ShellError>(json!({"arguments":arguments}))
                },
            )
            .await?;
            if first["tool_name"] == "do_job" {
                let bytes = imp::encode(&first)?;
                finish.send::<LatheResult>(&bytes.as_slice()).await?;
                let received = hibana_runtime::race(
                    async { Ok::<_, ShellError>(Ok(finish.recv::<ReturnLathe>().await?)) },
                    async {
                        let arguments = if agent == "TECHNICIAN_A" {
                            retry.recv::<LatheRetryB>().await?.into_value()
                        } else {
                            retry.recv::<LatheRetryA>().await?.into_value()
                        };

                        Ok::<_, ShellError>(Err(arguments))
                    },
                )
                .await?;
                match received {
                    Ok(lathe) => {
                        let arguments = if agent == "TECHNICIAN_A" {
                            retry.recv::<LatheRetryB>().await?.into_value()
                        } else {
                            retry.recv::<LatheRetryA>().await?.into_value()
                        };

                        if arguments != arrived["arguments"] {
                            return Err(ShellError::message(
                                "lathe retry changed the admitted job",
                            ));
                        }
                        retry.send::<LatheBusy>(&()).await?;

                        break 'returned lathe;
                    }
                    Err(arguments) => {
                        if arguments != arrived["arguments"] {
                            return Err(ShellError::message(
                                "lathe retry changed the admitted job",
                            ));
                        }
                        retry.send::<LatheBusy>(&()).await?;
                    }
                }
            } else {
                let first_retry_arguments = first["arguments"].clone();
                if first_retry_arguments != arrived["arguments"] {
                    return Err(ShellError::message("lathe retry changed the admitted job"));
                }
                retry.send::<LatheBusy>(&()).await?;

                let result = loop {
                    let response = hibana_runtime::race(
                        async {
                            let result = (&mut effect).await.map_err(ShellError::message)?;
                            imp::completed(&result, agent, "LATHE", &arguments)?;
                            Ok::<_, ShellError>(result)
                        },
                        async {
                            let arguments = if agent == "TECHNICIAN_A" {
                                retry.recv::<LatheRetryB>().await?.into_value()
                            } else {
                                retry.recv::<LatheRetryA>().await?.into_value()
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
                        return Err(ShellError::message("lathe retry changed the admitted job"));
                    }
                    retry.send::<LatheBusy>(&()).await?;
                };
                let bytes = imp::encode(&result)?;
                finish.send::<LatheResult>(&bytes.as_slice()).await?;
            }
            loop {
                let received = hibana_runtime::race(
                    async { Ok::<_, ShellError>(Ok(finish.recv::<ReturnLathe>().await?)) },
                    async {
                        let arguments = if agent == "TECHNICIAN_A" {
                            retry.recv::<LatheRetryB>().await?.into_value()
                        } else {
                            retry.recv::<LatheRetryA>().await?.into_value()
                        };

                        Ok::<_, ShellError>(Err(arguments))
                    },
                )
                .await?;
                match received {
                    Ok(lathe) => break lathe,
                    Err(arguments) => {
                        if arguments != arrived["arguments"] {
                            return Err(ShellError::message(
                                "lathe retry changed the admitted job",
                            ));
                        }
                        retry.send::<LatheBusy>(&()).await?;
                    }
                }
            }
        };
        (lathe, arrived)
    };
    work.send::<LatheReady>(&()).await?;
    loop {
        // A retry already in flight may cross Ready. Its affine query return,
        // rather than the notification, retires that remaining continuation.
        let branch = work.offer().await?;
        use hibana::g::Message;
        match branch.label() {
            ReturnLatheRequestA::LOGICAL_LABEL => {
                drop(branch.recv::<ReturnLatheRequestA>().await?);
                break;
            }
            ReturnLatheRequestB::LOGICAL_LABEL => {
                drop(branch.recv::<ReturnLatheRequestB>().await?);
                break;
            }
            LatheRetryA::LOGICAL_LABEL => {
                let arguments = branch.recv::<LatheRetryA>().await?.into_value();
                if arguments != next["arguments"] {
                    return Err(ShellError::message("lathe retry changed the admitted job"));
                }
                work.send::<LatheBusy>(&()).await?;
            }
            LatheRetryB::LOGICAL_LABEL => {
                let arguments = branch.recv::<LatheRetryB>().await?.into_value();
                if arguments != next["arguments"] {
                    return Err(ShellError::message("lathe retry changed the admitted job"));
                }
                work.send::<LatheBusy>(&()).await?;
            }
            _ => return Err(ShellError::message("unexpected lathe request continuation")),
        }
    }
    work.send::<LatheLoan>(lathe).await?;
    let (agent, arguments) = if next["agent"] == "TECHNICIAN_A" {
        ("TECHNICIAN_A", work.recv::<LatheJobA>().await?.into_value())
    } else {
        ("TECHNICIAN_B", work.recv::<LatheJobB>().await?.into_value())
    };
    if arguments != next["arguments"] {
        return Err(ShellError::message(
            "lathe request differs from admitted arguments",
        ));
    }
    let result = domain
        .execute(agent, "do_job", arguments.clone())
        .await
        .map_err(ShellError::message)?;
    imp::completed(&result, agent, "LATHE", &arguments)?;
    let bytes = imp::encode(&result)?;
    work.send::<LatheResult>(&bytes.as_slice()).await?;
    drop(work.recv::<ReturnLathe>().await?);
    Ok(())
}
