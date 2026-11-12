use super::super::{global::*, imp};
use crate::shell::{
    ShellError, ShellResult,
    global::Exit as IoExit,
    imp::file_wire::{ClosedFile, ResultChunk, ResultChunkMsg, SERVICE_FD},
};
use hibana_wasip1_runtime::{FileBoundary, HibanaWasiGuest, WasiImportCompletion, protocol};
use serde_json::{Value, json};

#[hibana_runtime::scope]
pub async fn run(
    mut guest: &mut HibanaWasiGuest,
    endpoint: &mut hibana::Endpoint<BACKEND>,
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    let mut repo = task.recv::<RepoLoan>().await?;
    let mut api = loop {
        match guest
            .next_file::<ClosedFile, BACKEND>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Exit(_) => {
                return Err(ShellError::message("backend exited before API definition"));
            }
            FileBoundary::Closed { pending, file } => {
                let response =
                    match file
                        .arguments(b"service/define_api", "api_name")
                        .and_then(|v| {
                            imp::arguments(
                                &serde_json::to_vec(&v)
                                    .map_err(|e| ShellError::message(e.to_string()))?,
                                "api_name",
                                "feature_api_spec",
                            )
                        }) {
                        Err(error) => Err(error),
                        Ok(arguments) => {
                            let bytes = serde_json::to_vec(&arguments)
                                .map_err(|e| ShellError::message(e.to_string()))?;
                            repo.send::<DefineApi>(&bytes.as_slice()).await?;
                            let bytes = repo.recv::<ApiDefined>().await?.to_vec();
                            task.send::<Contract>(&bytes.as_slice()).await?;
                            if task.recv::<Agreement>().await? != bytes {
                                return Err(ShellError::message(
                                    "agreement differs from the defined API",
                                ));
                            }
                            Ok(serde_json::from_slice::<Value>(&bytes)
                                .map_err(|e| ShellError::message(e.to_string()))?)
                        }
                    };
                let (reply, errno) = match &response {
                    Ok(value) => (value.clone(), 0),
                    Err(error) => (
                        json!({"status":"invalid_arguments","error":error.to_string()}),
                        28,
                    ),
                };

                let bytes =
                    serde_json::to_vec(&reply).map_err(|e| ShellError::message(e.to_string()))?;
                for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
                    private
                        .send::<ResultChunkMsg>(&ResultChunk::new(
                            index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                            chunk,
                        )?)
                        .await?;
                }
                guest = pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
                    protocol::FdClosed::new_with_errno(SERVICE_FD, errno),
                )))?;

                if let Ok(api) = response {
                    break api;
                }
            }
        }
    };
    'implementation: loop {
        let (backend, frontend, mut next) = loop {
            match guest
                .next_file::<ClosedFile, BACKEND>(&mut private, SERVICE_FD)
                .await?
            {
                FileBoundary::Exit(_) => {
                    return Err(ShellError::message("backend exited before implementation"));
                }
                FileBoundary::Closed { pending, file } => {
                    let requested = match file.path.as_slice() {
                        b"service/implement_feature" => file
                            .arguments(b"service/implement_feature", "feature_name")
                            .and_then(|v| {
                                imp::arguments(
                                    &serde_json::to_vec(&v)
                                        .map_err(|e| ShellError::message(e.to_string()))?,
                                    "feature_name",
                                    "backend_endpoint",
                                )
                            }),
                        b"service/loan_frontend" => file
                            .arguments(b"service/loan_frontend", "feature_name")
                            .and_then(|v| {
                                imp::arguments(
                                    &serde_json::to_vec(&v)
                                        .map_err(|e| ShellError::message(e.to_string()))?,
                                    "feature_name",
                                    "frontend_component",
                                )
                            }),
                        _ => Err(ShellError::message(
                            "expected implementation or an actual REPO handoff",
                        )),
                    };
                    let arguments = match requested {
                        Ok(arguments) => arguments,
                        Err(error) => {
                            let reply =
                                json!({"status":"invalid_arguments","error":error.to_string()});
                            let errno = 28;

                            let bytes = serde_json::to_vec(&reply)
                                .map_err(|e| ShellError::message(e.to_string()))?;
                            for (index, chunk) in
                                bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                            {
                                private
                                    .send::<ResultChunkMsg>(&ResultChunk::new(
                                        index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                        chunk,
                                    )?)
                                    .await?;
                            }
                            guest = pending.complete(WasiImportCompletion::FdClose(
                                protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                                    SERVICE_FD, errno,
                                )),
                            ))?;

                            continue;
                        }
                    };
                    if file.path == b"service/loan_frontend" {
                        task.send::<RepoLoan>(repo).await?;
                        let reply = json!({"status":"accepted","resource":"REPO","recipient":"FRONTEND_DEV"});
                        let errno = 0;

                        let bytes = serde_json::to_vec(&reply)
                            .map_err(|e| ShellError::message(e.to_string()))?;
                        for (index, chunk) in
                            bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                        {
                            private
                                .send::<ResultChunkMsg>(&ResultChunk::new(
                                    index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                    chunk,
                                )?)
                                .await?;
                        }
                        guest = pending.complete(WasiImportCompletion::FdClose(
                            protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                                SERVICE_FD, errno,
                            )),
                        ))?;

                        let ((front, returned), mut input) = async {
                            async {
                                let front = task.recv::<FrontendReady>().await?.to_vec();
                                let repo = task.recv::<RepoLoan>().await?;
                                Ok::<_, ShellError>((front, repo))
                            };
                            async {
                                match guest
                                    .next_file::<ClosedFile, BACKEND>(&mut private, SERVICE_FD)
                                    .await?
                                {
                                    FileBoundary::Exit(_) => Err(ShellError::message(
                                        "backend exited before its implementation",
                                    )),
                                    input => Ok(input),
                                }
                            };
                        }
                        .await?;
                        repo = returned;
                        let implemented = loop {
                            match input {
                                FileBoundary::Exit(_) => {
                                    return Err(ShellError::message(
                                        "backend exited before its implementation",
                                    ));
                                }
                                FileBoundary::Closed { pending, file } => {
                                    let response = match file
                                        .arguments(b"service/implement_feature", "feature_name")
                                        .and_then(|v| {
                                            imp::arguments(
                                                &serde_json::to_vec(&v).map_err(|e| {
                                                    ShellError::message(e.to_string())
                                                })?,
                                                "feature_name",
                                                "backend_endpoint",
                                            )
                                        }) {
                                        Err(error) => Err(error),
                                        Ok(arguments) => {
                                            let bytes = serde_json::to_vec(&arguments)
                                                .map_err(|e| ShellError::message(e.to_string()))?;
                                            task.send::<BackendIntent>(&bytes.as_slice()).await?;
                                            repo.send::<ImplementBackend>(&bytes.as_slice())
                                                .await?;
                                            Ok(repo.recv::<BackendImplemented>().await?.to_vec())
                                        }
                                    };
                                    let (reply, errno) = match &response {
                                        Ok(bytes) => (
                                            serde_json::from_slice::<Value>(bytes)
                                                .map_err(|e| ShellError::message(e.to_string()))?,
                                            0,
                                        ),
                                        Err(error) => (
                                            json!({"status":"invalid_arguments","error":error.to_string()}),
                                            28,
                                        ),
                                    };

                                    let bytes = serde_json::to_vec(&reply)
                                        .map_err(|e| ShellError::message(e.to_string()))?;
                                    for (index, chunk) in
                                        bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                                    {
                                        private
                                            .send::<ResultChunkMsg>(&ResultChunk::new(
                                                index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                                chunk,
                                            )?)
                                            .await?;
                                    }
                                    guest = pending.complete(WasiImportCompletion::FdClose(
                                        protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                                            SERVICE_FD, errno,
                                        )),
                                    ))?;

                                    if let Ok(result) = response {
                                        break result;
                                    }
                                }
                            }
                            input = guest
                                .next_file::<ClosedFile, BACKEND>(&mut private, SERVICE_FD)
                                .await?;
                        };
                        let next = guest
                            .next_file::<ClosedFile, BACKEND>(&mut private, SERVICE_FD)
                            .await?;
                        break (implemented, front, next);
                    } else {
                        let bytes = serde_json::to_vec(&arguments)
                            .map_err(|e| ShellError::message(e.to_string()))?;
                        task.send::<BackendIntent>(&bytes.as_slice()).await?;
                        repo.send::<ImplementBackend>(&bytes.as_slice()).await?;
                        let implemented = repo.recv::<BackendImplemented>().await?.to_vec();
                        let reply: Value = serde_json::from_slice(&implemented)
                            .map_err(|e| ShellError::message(e.to_string()))?;
                        let errno = 0;

                        let bytes = serde_json::to_vec(&reply)
                            .map_err(|e| ShellError::message(e.to_string()))?;
                        for (index, chunk) in
                            bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                        {
                            private
                                .send::<ResultChunkMsg>(&ResultChunk::new(
                                    index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                    chunk,
                                )?)
                                .await?;
                        }
                        guest = pending.complete(WasiImportCompletion::FdClose(
                            protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                                SERVICE_FD, errno,
                            )),
                        ))?;

                        task.send::<RepoLoan>(repo).await?;
                        let ((front, returned), next) = async {
                            async {
                                let front = task.recv::<FrontendReady>().await?.to_vec();
                                let repo = task.recv::<RepoLoan>().await?;
                                Ok::<_, ShellError>((front, repo))
                            };
                            async {
                                match guest
                                    .next_file::<ClosedFile, BACKEND>(&mut private, SERVICE_FD)
                                    .await?
                                {
                                    FileBoundary::Exit(_) => Err(ShellError::message(
                                        "backend exited before finalization",
                                    )),
                                    input => Ok(input),
                                }
                            };
                        }
                        .await?;
                        repo = returned;
                        break (implemented, front, next);
                    }
                }
            }
        };
        let backend: Value =
            serde_json::from_slice(&backend).map_err(|e| ShellError::message(e.to_string()))?;
        let frontend: Value =
            serde_json::from_slice(&frontend).map_err(|e| ShellError::message(e.to_string()))?;
        let candidate = imp::candidate(&api, &backend, &frontend);
        loop {
            match next {
                FileBoundary::Exit(_) => {
                    return Err(ShellError::message("backend exited before finalization"));
                }
                FileBoundary::Closed { pending, file } => {
                    let requested = match file.path.as_slice() {
                        b"service/define_api" => file
                            .arguments(b"service/define_api", "api_name")
                            .and_then(|v| {
                                imp::arguments(
                                    &serde_json::to_vec(&v)
                                        .map_err(|e| ShellError::message(e.to_string()))?,
                                    "api_name",
                                    "feature_api_spec",
                                )
                            }),
                        b"service/finalize" => file
                            .arguments(b"service/finalize", "api_name")
                            .and_then(|v| {
                                imp::arguments(
                                    &serde_json::to_vec(&v)
                                        .map_err(|e| ShellError::message(e.to_string()))?,
                                    "api_name",
                                    "feature_api_spec",
                                )
                            }),
                        _ => Err(ShellError::message(
                            "expected a new API or finalization of the current candidate",
                        )),
                    };
                    let arguments = match requested {
                        Ok(arguments) => arguments,
                        Err(error) => {
                            let reply =
                                json!({"status":"invalid_arguments","error":error.to_string()});
                            let errno = 28;

                            let bytes = serde_json::to_vec(&reply)
                                .map_err(|e| ShellError::message(e.to_string()))?;
                            for (index, chunk) in
                                bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                            {
                                private
                                    .send::<ResultChunkMsg>(&ResultChunk::new(
                                        index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                        chunk,
                                    )?)
                                    .await?;
                            }
                            guest = pending.complete(WasiImportCompletion::FdClose(
                                protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                                    SERVICE_FD, errno,
                                )),
                            ))?;

                            next = guest
                                .next_file::<ClosedFile, BACKEND>(&mut private, SERVICE_FD)
                                .await?;
                            continue;
                        }
                    };
                    if file.path == b"service/define_api" {
                        let bytes = serde_json::to_vec(&arguments)
                            .map_err(|e| ShellError::message(e.to_string()))?;
                        repo.send::<DefineApi>(&bytes.as_slice()).await?;
                        let bytes = repo.recv::<ApiDefined>().await?.to_vec();
                        task.send::<Contract>(&bytes.as_slice()).await?;
                        if task.recv::<Agreement>().await? != bytes {
                            return Err(ShellError::message(
                                "agreement differs from the changed API",
                            ));
                        }
                        api = serde_json::from_slice(&bytes)
                            .map_err(|e| ShellError::message(e.to_string()))?;
                        let reply = api.clone();
                        let errno = 0;

                        let bytes = serde_json::to_vec(&reply)
                            .map_err(|e| ShellError::message(e.to_string()))?;
                        for (index, chunk) in
                            bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                        {
                            private
                                .send::<ResultChunkMsg>(&ResultChunk::new(
                                    index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                    chunk,
                                )?)
                                .await?;
                        }
                        guest = pending.complete(WasiImportCompletion::FdClose(
                            protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                                SERVICE_FD, errno,
                            )),
                        ))?;

                        continue 'implementation;
                    } else {
                        let bytes = serde_json::to_vec(&candidate)
                            .map_err(|e| ShellError::message(e.to_string()))?;
                        repo.send::<Finalize>(&bytes.as_slice()).await?;
                        task.send::<RepoLoan>(repo).await?;
                        let reply = json!({"status":"accepted","candidate":candidate});
                        let errno = 0;

                        let bytes = serde_json::to_vec(&reply)
                            .map_err(|e| ShellError::message(e.to_string()))?;
                        for (index, chunk) in
                            bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                        {
                            private
                                .send::<ResultChunkMsg>(&ResultChunk::new(
                                    index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                    chunk,
                                )?)
                                .await?;
                        }
                        guest = pending.complete(WasiImportCompletion::FdClose(
                            protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                                SERVICE_FD, errno,
                            )),
                        ))?;

                        break 'implementation;
                    }
                }
            }
        }
    }
    let (result, mut call) = async {
        async { Ok::<_, ShellError>(task.recv::<BackendResult>().await?.to_vec()) };
        async {
            match guest
                .next_file::<ClosedFile, BACKEND>(&mut private, SERVICE_FD)
                .await?
            {
                FileBoundary::Exit(_) => Err(ShellError::message(
                    "backend exited without its final report",
                )),
                input => Ok(input),
            }
        };
    }
    .await?;
    loop {
        match call {
            FileBoundary::Exit(_) => {
                return Err(ShellError::message(
                    "backend exited without its final report",
                ));
            }
            FileBoundary::Closed { pending, file } => {
                let requested = file
                    .arguments(b"service/await_tests", "suite")
                    .and_then(|v| {
                        imp::arguments(
                            &serde_json::to_vec(&v)
                                .map_err(|e| ShellError::message(e.to_string()))?,
                            "suite",
                            "integration_tests",
                        )
                    });
                let (reply, errno) = match &requested {
                    Ok(_) => (
                        serde_json::from_slice::<Value>(&result)
                            .map_err(|e| ShellError::message(e.to_string()))?,
                        0,
                    ),
                    Err(error) => (
                        json!({"status":"invalid_arguments","error":error.to_string()}),
                        28,
                    ),
                };

                let bytes =
                    serde_json::to_vec(&reply).map_err(|e| ShellError::message(e.to_string()))?;
                for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
                    private
                        .send::<ResultChunkMsg>(&ResultChunk::new(
                            index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                            chunk,
                        )?)
                        .await?;
                }
                guest = pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
                    protocol::FdClosed::new_with_errno(SERVICE_FD, errno),
                )))?;

                if requested.is_ok() {
                    break;
                }
            }
        }
        call = guest
            .next_file::<ClosedFile, BACKEND>(&mut private, SERVICE_FD)
            .await?;
    }
    drop(task);
    loop {
        match guest
            .next_file::<ClosedFile, BACKEND>(&mut private, SERVICE_FD)
            .await?
        {
            FileBoundary::Closed { pending, .. } => {
                let reply =
                    json!({"status":"out_of_order","error":"the backend Endpoint has completed"});
                let errno = 8;

                let bytes =
                    serde_json::to_vec(&reply).map_err(|e| ShellError::message(e.to_string()))?;
                for (index, chunk) in bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate() {
                    private
                        .send::<ResultChunkMsg>(&ResultChunk::new(
                            index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                            chunk,
                        )?)
                        .await?;
                }
                guest = pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
                    protocol::FdClosed::new_with_errno(SERVICE_FD, errno),
                )))?;
            }
            FileBoundary::Exit(exit) => {
                let status = exit.status() as i32;
                private.send::<IoExit>(&status).await?;
                return Ok(status);
            }
        }
    }
}
