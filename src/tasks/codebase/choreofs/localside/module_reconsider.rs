//! Module custody follows the actual source and actual test results.
use super::super::global::{roles1h::*, *};
use crate::shell::{ShellError, ShellResult};
use hibana::Endpoint;
#[hibana_runtime::scope]
pub(crate) async fn auth<'run>(
    endpoint: &mut Endpoint<'run, { AUTH }>,
    mut resource: Endpoint<'run, { AUTH_RESOURCE }>,
) -> ShellResult<()> {
    for _ in 0..2 {
        endpoint.send::<Loan<AUTH_RESOURCE>>(resource).await?;
        resource = endpoint.recv::<Loan<AUTH_RESOURCE>>().await?;
    }
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            DATABASE_RESOURCE => {
                let result = branch.recv::<ModuleTestResult<DATABASE_RESOURCE>>().await?;
                let value: serde_json::Value = serde_json::from_slice(result)
                    .map_err(|error| ShellError::message(error.to_string()))?;
                match value["success"].as_bool() {
                    Some(false) => {
                        endpoint.send::<Loan<AUTH_RESOURCE>>(resource).await?;
                        resource = endpoint.recv::<Loan<AUTH_RESOURCE>>().await?;
                    }
                    Some(true) => loop {
                        let result = endpoint.recv::<ModuleTestResult<SEARCH_RESOURCE>>().await?;
                        let value: serde_json::Value = serde_json::from_slice(result)
                            .map_err(|error| ShellError::message(error.to_string()))?;
                        match value["success"].as_bool() {
                            Some(true) => return Ok(()),
                            Some(false) => {
                                endpoint.send::<Loan<AUTH_RESOURCE>>(resource).await?;
                                resource = endpoint.recv::<Loan<AUTH_RESOURCE>>().await?;
                            }
                            None => return Err(ShellError::message("missing actual test outcome")),
                        }
                    },
                    None => return Err(ShellError::message("missing actual test outcome")),
                }
            }
            SEARCH_RESOURCE => {
                let result = branch.recv::<ModuleTestResult<SEARCH_RESOURCE>>().await?;
                let value: serde_json::Value = serde_json::from_slice(result)
                    .map_err(|error| ShellError::message(error.to_string()))?;
                match value["success"].as_bool() {
                    Some(false) => {
                        endpoint.send::<Loan<AUTH_RESOURCE>>(resource).await?;
                        resource = endpoint.recv::<Loan<AUTH_RESOURCE>>().await?;
                    }
                    Some(true) => loop {
                        let result = endpoint
                            .recv::<ModuleTestResult<DATABASE_RESOURCE>>()
                            .await?;
                        let value: serde_json::Value = serde_json::from_slice(result)
                            .map_err(|error| ShellError::message(error.to_string()))?;
                        match value["success"].as_bool() {
                            Some(true) => return Ok(()),
                            Some(false) => {
                                endpoint.send::<Loan<AUTH_RESOURCE>>(resource).await?;
                                resource = endpoint.recv::<Loan<AUTH_RESOURCE>>().await?;
                            }
                            None => return Err(ShellError::message("missing actual test outcome")),
                        }
                    },
                    None => return Err(ShellError::message("missing actual test outcome")),
                }
            }
            _ => return Err(ShellError::message("unexpected actual module result")),
        }
    }
}
#[hibana_runtime::scope]
pub(crate) async fn database<'run>(
    endpoint: &mut Endpoint<'run, { DATABASE }>,
    mut resource: Endpoint<'run, { DATABASE_RESOURCE }>,
) -> ShellResult<()> {
    for _ in 0..2 {
        endpoint.send::<Loan<DATABASE_RESOURCE>>(resource).await?;
        resource = endpoint.recv::<Loan<DATABASE_RESOURCE>>().await?;
    }
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            AUTH_RESOURCE => {
                let result = branch.recv::<ModuleTestResult<AUTH_RESOURCE>>().await?;
                let value: serde_json::Value = serde_json::from_slice(result)
                    .map_err(|error| ShellError::message(error.to_string()))?;
                match value["success"].as_bool() {
                    Some(false) => {
                        endpoint.send::<Loan<DATABASE_RESOURCE>>(resource).await?;
                        resource = endpoint.recv::<Loan<DATABASE_RESOURCE>>().await?;
                    }
                    Some(true) => loop {
                        let result = endpoint.recv::<ModuleTestResult<API_RESOURCE>>().await?;
                        let value: serde_json::Value = serde_json::from_slice(result)
                            .map_err(|error| ShellError::message(error.to_string()))?;
                        match value["success"].as_bool() {
                            Some(true) => return Ok(()),
                            Some(false) => {
                                endpoint.send::<Loan<DATABASE_RESOURCE>>(resource).await?;
                                resource = endpoint.recv::<Loan<DATABASE_RESOURCE>>().await?;
                            }
                            None => return Err(ShellError::message("missing actual test outcome")),
                        }
                    },
                    None => return Err(ShellError::message("missing actual test outcome")),
                }
            }
            API_RESOURCE => {
                let result = branch.recv::<ModuleTestResult<API_RESOURCE>>().await?;
                let value: serde_json::Value = serde_json::from_slice(result)
                    .map_err(|error| ShellError::message(error.to_string()))?;
                match value["success"].as_bool() {
                    Some(false) => {
                        endpoint.send::<Loan<DATABASE_RESOURCE>>(resource).await?;
                        resource = endpoint.recv::<Loan<DATABASE_RESOURCE>>().await?;
                    }
                    Some(true) => loop {
                        let result = endpoint.recv::<ModuleTestResult<AUTH_RESOURCE>>().await?;
                        let value: serde_json::Value = serde_json::from_slice(result)
                            .map_err(|error| ShellError::message(error.to_string()))?;
                        match value["success"].as_bool() {
                            Some(true) => return Ok(()),
                            Some(false) => {
                                endpoint.send::<Loan<DATABASE_RESOURCE>>(resource).await?;
                                resource = endpoint.recv::<Loan<DATABASE_RESOURCE>>().await?;
                            }
                            None => return Err(ShellError::message("missing actual test outcome")),
                        }
                    },
                    None => return Err(ShellError::message("missing actual test outcome")),
                }
            }
            _ => return Err(ShellError::message("unexpected actual module result")),
        }
    }
}
#[hibana_runtime::scope]
pub(crate) async fn api<'run>(
    endpoint: &mut Endpoint<'run, { API }>,
    mut resource: Endpoint<'run, { API_RESOURCE }>,
) -> ShellResult<()> {
    for _ in 0..2 {
        endpoint.send::<Loan<API_RESOURCE>>(resource).await?;
        resource = endpoint.recv::<Loan<API_RESOURCE>>().await?;
    }
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            DATABASE_RESOURCE => {
                let result = branch.recv::<ModuleTestResult<DATABASE_RESOURCE>>().await?;
                let value: serde_json::Value = serde_json::from_slice(result)
                    .map_err(|error| ShellError::message(error.to_string()))?;
                match value["success"].as_bool() {
                    Some(false) => {
                        endpoint.send::<Loan<API_RESOURCE>>(resource).await?;
                        resource = endpoint.recv::<Loan<API_RESOURCE>>().await?;
                    }
                    Some(true) => loop {
                        let result = endpoint
                            .recv::<ModuleTestResult<PAYMENT_RESOURCE>>()
                            .await?;
                        let value: serde_json::Value = serde_json::from_slice(result)
                            .map_err(|error| ShellError::message(error.to_string()))?;
                        match value["success"].as_bool() {
                            Some(true) => return Ok(()),
                            Some(false) => {
                                endpoint.send::<Loan<API_RESOURCE>>(resource).await?;
                                resource = endpoint.recv::<Loan<API_RESOURCE>>().await?;
                            }
                            None => return Err(ShellError::message("missing actual test outcome")),
                        }
                    },
                    None => return Err(ShellError::message("missing actual test outcome")),
                }
            }
            PAYMENT_RESOURCE => {
                let result = branch.recv::<ModuleTestResult<PAYMENT_RESOURCE>>().await?;
                let value: serde_json::Value = serde_json::from_slice(result)
                    .map_err(|error| ShellError::message(error.to_string()))?;
                match value["success"].as_bool() {
                    Some(false) => {
                        endpoint.send::<Loan<API_RESOURCE>>(resource).await?;
                        resource = endpoint.recv::<Loan<API_RESOURCE>>().await?;
                    }
                    Some(true) => loop {
                        let result = endpoint
                            .recv::<ModuleTestResult<DATABASE_RESOURCE>>()
                            .await?;
                        let value: serde_json::Value = serde_json::from_slice(result)
                            .map_err(|error| ShellError::message(error.to_string()))?;
                        match value["success"].as_bool() {
                            Some(true) => return Ok(()),
                            Some(false) => {
                                endpoint.send::<Loan<API_RESOURCE>>(resource).await?;
                                resource = endpoint.recv::<Loan<API_RESOURCE>>().await?;
                            }
                            None => return Err(ShellError::message("missing actual test outcome")),
                        }
                    },
                    None => return Err(ShellError::message("missing actual test outcome")),
                }
            }
            _ => return Err(ShellError::message("unexpected actual module result")),
        }
    }
}
#[hibana_runtime::scope]
pub(crate) async fn payment<'run>(
    endpoint: &mut Endpoint<'run, { PAYMENT }>,
    mut resource: Endpoint<'run, { PAYMENT_RESOURCE }>,
) -> ShellResult<()> {
    for _ in 0..2 {
        endpoint.send::<Loan<PAYMENT_RESOURCE>>(resource).await?;
        resource = endpoint.recv::<Loan<PAYMENT_RESOURCE>>().await?;
    }
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            API_RESOURCE => {
                let result = branch.recv::<ModuleTestResult<API_RESOURCE>>().await?;
                let value: serde_json::Value = serde_json::from_slice(result)
                    .map_err(|error| ShellError::message(error.to_string()))?;
                match value["success"].as_bool() {
                    Some(false) => {
                        endpoint.send::<Loan<PAYMENT_RESOURCE>>(resource).await?;
                        resource = endpoint.recv::<Loan<PAYMENT_RESOURCE>>().await?;
                    }
                    Some(true) => loop {
                        let result = endpoint.recv::<ModuleTestResult<NOTIFY_RESOURCE>>().await?;
                        let value: serde_json::Value = serde_json::from_slice(result)
                            .map_err(|error| ShellError::message(error.to_string()))?;
                        match value["success"].as_bool() {
                            Some(true) => return Ok(()),
                            Some(false) => {
                                endpoint.send::<Loan<PAYMENT_RESOURCE>>(resource).await?;
                                resource = endpoint.recv::<Loan<PAYMENT_RESOURCE>>().await?;
                            }
                            None => return Err(ShellError::message("missing actual test outcome")),
                        }
                    },
                    None => return Err(ShellError::message("missing actual test outcome")),
                }
            }
            NOTIFY_RESOURCE => {
                let result = branch.recv::<ModuleTestResult<NOTIFY_RESOURCE>>().await?;
                let value: serde_json::Value = serde_json::from_slice(result)
                    .map_err(|error| ShellError::message(error.to_string()))?;
                match value["success"].as_bool() {
                    Some(false) => {
                        endpoint.send::<Loan<PAYMENT_RESOURCE>>(resource).await?;
                        resource = endpoint.recv::<Loan<PAYMENT_RESOURCE>>().await?;
                    }
                    Some(true) => loop {
                        let result = endpoint.recv::<ModuleTestResult<API_RESOURCE>>().await?;
                        let value: serde_json::Value = serde_json::from_slice(result)
                            .map_err(|error| ShellError::message(error.to_string()))?;
                        match value["success"].as_bool() {
                            Some(true) => return Ok(()),
                            Some(false) => {
                                endpoint.send::<Loan<PAYMENT_RESOURCE>>(resource).await?;
                                resource = endpoint.recv::<Loan<PAYMENT_RESOURCE>>().await?;
                            }
                            None => return Err(ShellError::message("missing actual test outcome")),
                        }
                    },
                    None => return Err(ShellError::message("missing actual test outcome")),
                }
            }
            _ => return Err(ShellError::message("unexpected actual module result")),
        }
    }
}
#[hibana_runtime::scope]
pub(crate) async fn notify<'run>(
    endpoint: &mut Endpoint<'run, { NOTIFY }>,
    mut resource: Endpoint<'run, { NOTIFY_RESOURCE }>,
) -> ShellResult<()> {
    for _ in 0..2 {
        endpoint.send::<Loan<NOTIFY_RESOURCE>>(resource).await?;
        resource = endpoint.recv::<Loan<NOTIFY_RESOURCE>>().await?;
    }
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            PAYMENT_RESOURCE => {
                let result = branch.recv::<ModuleTestResult<PAYMENT_RESOURCE>>().await?;
                let value: serde_json::Value = serde_json::from_slice(result)
                    .map_err(|error| ShellError::message(error.to_string()))?;
                match value["success"].as_bool() {
                    Some(false) => {
                        endpoint.send::<Loan<NOTIFY_RESOURCE>>(resource).await?;
                        resource = endpoint.recv::<Loan<NOTIFY_RESOURCE>>().await?;
                    }
                    Some(true) => loop {
                        let result = endpoint
                            .recv::<ModuleTestResult<ANALYTICS_RESOURCE>>()
                            .await?;
                        let value: serde_json::Value = serde_json::from_slice(result)
                            .map_err(|error| ShellError::message(error.to_string()))?;
                        match value["success"].as_bool() {
                            Some(true) => return Ok(()),
                            Some(false) => {
                                endpoint.send::<Loan<NOTIFY_RESOURCE>>(resource).await?;
                                resource = endpoint.recv::<Loan<NOTIFY_RESOURCE>>().await?;
                            }
                            None => return Err(ShellError::message("missing actual test outcome")),
                        }
                    },
                    None => return Err(ShellError::message("missing actual test outcome")),
                }
            }
            ANALYTICS_RESOURCE => {
                let result = branch
                    .recv::<ModuleTestResult<ANALYTICS_RESOURCE>>()
                    .await?;
                let value: serde_json::Value = serde_json::from_slice(result)
                    .map_err(|error| ShellError::message(error.to_string()))?;
                match value["success"].as_bool() {
                    Some(false) => {
                        endpoint.send::<Loan<NOTIFY_RESOURCE>>(resource).await?;
                        resource = endpoint.recv::<Loan<NOTIFY_RESOURCE>>().await?;
                    }
                    Some(true) => loop {
                        let result = endpoint
                            .recv::<ModuleTestResult<PAYMENT_RESOURCE>>()
                            .await?;
                        let value: serde_json::Value = serde_json::from_slice(result)
                            .map_err(|error| ShellError::message(error.to_string()))?;
                        match value["success"].as_bool() {
                            Some(true) => return Ok(()),
                            Some(false) => {
                                endpoint.send::<Loan<NOTIFY_RESOURCE>>(resource).await?;
                                resource = endpoint.recv::<Loan<NOTIFY_RESOURCE>>().await?;
                            }
                            None => return Err(ShellError::message("missing actual test outcome")),
                        }
                    },
                    None => return Err(ShellError::message("missing actual test outcome")),
                }
            }
            _ => return Err(ShellError::message("unexpected actual module result")),
        }
    }
}
#[hibana_runtime::scope]
pub(crate) async fn analytics<'run>(
    endpoint: &mut Endpoint<'run, { ANALYTICS }>,
    mut resource: Endpoint<'run, { ANALYTICS_RESOURCE }>,
) -> ShellResult<()> {
    for _ in 0..2 {
        endpoint.send::<Loan<ANALYTICS_RESOURCE>>(resource).await?;
        resource = endpoint.recv::<Loan<ANALYTICS_RESOURCE>>().await?;
    }
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            NOTIFY_RESOURCE => {
                let result = branch.recv::<ModuleTestResult<NOTIFY_RESOURCE>>().await?;
                let value: serde_json::Value = serde_json::from_slice(result)
                    .map_err(|error| ShellError::message(error.to_string()))?;
                match value["success"].as_bool() {
                    Some(false) => {
                        endpoint.send::<Loan<ANALYTICS_RESOURCE>>(resource).await?;
                        resource = endpoint.recv::<Loan<ANALYTICS_RESOURCE>>().await?;
                    }
                    Some(true) => loop {
                        let result = endpoint.recv::<ModuleTestResult<SEARCH_RESOURCE>>().await?;
                        let value: serde_json::Value = serde_json::from_slice(result)
                            .map_err(|error| ShellError::message(error.to_string()))?;
                        match value["success"].as_bool() {
                            Some(true) => return Ok(()),
                            Some(false) => {
                                endpoint.send::<Loan<ANALYTICS_RESOURCE>>(resource).await?;
                                resource = endpoint.recv::<Loan<ANALYTICS_RESOURCE>>().await?;
                            }
                            None => return Err(ShellError::message("missing actual test outcome")),
                        }
                    },
                    None => return Err(ShellError::message("missing actual test outcome")),
                }
            }
            SEARCH_RESOURCE => {
                let result = branch.recv::<ModuleTestResult<SEARCH_RESOURCE>>().await?;
                let value: serde_json::Value = serde_json::from_slice(result)
                    .map_err(|error| ShellError::message(error.to_string()))?;
                match value["success"].as_bool() {
                    Some(false) => {
                        endpoint.send::<Loan<ANALYTICS_RESOURCE>>(resource).await?;
                        resource = endpoint.recv::<Loan<ANALYTICS_RESOURCE>>().await?;
                    }
                    Some(true) => loop {
                        let result = endpoint.recv::<ModuleTestResult<NOTIFY_RESOURCE>>().await?;
                        let value: serde_json::Value = serde_json::from_slice(result)
                            .map_err(|error| ShellError::message(error.to_string()))?;
                        match value["success"].as_bool() {
                            Some(true) => return Ok(()),
                            Some(false) => {
                                endpoint.send::<Loan<ANALYTICS_RESOURCE>>(resource).await?;
                                resource = endpoint.recv::<Loan<ANALYTICS_RESOURCE>>().await?;
                            }
                            None => return Err(ShellError::message("missing actual test outcome")),
                        }
                    },
                    None => return Err(ShellError::message("missing actual test outcome")),
                }
            }
            _ => return Err(ShellError::message("unexpected actual module result")),
        }
    }
}
#[hibana_runtime::scope]
pub(crate) async fn search<'run>(
    endpoint: &mut Endpoint<'run, { SEARCH }>,
    mut resource: Endpoint<'run, { SEARCH_RESOURCE }>,
) -> ShellResult<()> {
    for _ in 0..2 {
        endpoint.send::<Loan<SEARCH_RESOURCE>>(resource).await?;
        resource = endpoint.recv::<Loan<SEARCH_RESOURCE>>().await?;
    }
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            ANALYTICS_RESOURCE => {
                let result = branch
                    .recv::<ModuleTestResult<ANALYTICS_RESOURCE>>()
                    .await?;
                let value: serde_json::Value = serde_json::from_slice(result)
                    .map_err(|error| ShellError::message(error.to_string()))?;
                match value["success"].as_bool() {
                    Some(false) => {
                        endpoint.send::<Loan<SEARCH_RESOURCE>>(resource).await?;
                        resource = endpoint.recv::<Loan<SEARCH_RESOURCE>>().await?;
                    }
                    Some(true) => loop {
                        let result = endpoint.recv::<ModuleTestResult<AUTH_RESOURCE>>().await?;
                        let value: serde_json::Value = serde_json::from_slice(result)
                            .map_err(|error| ShellError::message(error.to_string()))?;
                        match value["success"].as_bool() {
                            Some(true) => return Ok(()),
                            Some(false) => {
                                endpoint.send::<Loan<SEARCH_RESOURCE>>(resource).await?;
                                resource = endpoint.recv::<Loan<SEARCH_RESOURCE>>().await?;
                            }
                            None => return Err(ShellError::message("missing actual test outcome")),
                        }
                    },
                    None => return Err(ShellError::message("missing actual test outcome")),
                }
            }
            AUTH_RESOURCE => {
                let result = branch.recv::<ModuleTestResult<AUTH_RESOURCE>>().await?;
                let value: serde_json::Value = serde_json::from_slice(result)
                    .map_err(|error| ShellError::message(error.to_string()))?;
                match value["success"].as_bool() {
                    Some(false) => {
                        endpoint.send::<Loan<SEARCH_RESOURCE>>(resource).await?;
                        resource = endpoint.recv::<Loan<SEARCH_RESOURCE>>().await?;
                    }
                    Some(true) => loop {
                        let result = endpoint
                            .recv::<ModuleTestResult<ANALYTICS_RESOURCE>>()
                            .await?;
                        let value: serde_json::Value = serde_json::from_slice(result)
                            .map_err(|error| ShellError::message(error.to_string()))?;
                        match value["success"].as_bool() {
                            Some(true) => return Ok(()),
                            Some(false) => {
                                endpoint.send::<Loan<SEARCH_RESOURCE>>(resource).await?;
                                resource = endpoint.recv::<Loan<SEARCH_RESOURCE>>().await?;
                            }
                            None => return Err(ShellError::message("missing actual test outcome")),
                        }
                    },
                    None => return Err(ShellError::message("missing actual test outcome")),
                }
            }
            _ => return Err(ShellError::message("unexpected actual module result")),
        }
    }
}
