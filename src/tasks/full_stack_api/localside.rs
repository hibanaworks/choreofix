use super::global::*;
use crate::{
    imp::json::{decode, encode},
    shell::{
        ShellError, ShellResult,
        global::Loan,
        imp::{review_reply::ReviewReply, service::ReceivedEffects},
    },
};
use hibana::{Endpoint, g::Message};
use serde_json::{Value, json};

#[hibana_runtime::scope]
pub async fn migrate(
    db: &mut Endpoint<DB_SERVICE>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    let mut resource = db.recv::<Loan<DATABASE>>().await?;
    loop {
        let arguments = resource.recv::<MigrateSchema>().await?;
        let result = effects.apply(arguments).await?;
        let bytes = encode(&result)?;
        async {
            let [mut file, mut recipient] = resource.par()?;
            async { file.send::<SchemaMigrated>(&bytes.as_slice()).await };
            async { recipient.send::<SchemaReady>(&bytes.as_slice()).await };
        }
        .await?;

        loop {
            let branch = db.offer().await?;
            match branch.label() {
                SchemaInUse::LOGICAL_LABEL => {
                    let installed = branch.recv::<SchemaInUse>().await?.to_vec();
                    db.send::<SchemaStatus>(&installed.as_slice()).await?;
                }
                RequestMigration::LOGICAL_LABEL => {
                    let request = branch.recv::<RequestMigration>().await?.to_vec();
                    db.send::<SchemaStatus>(&request.as_slice()).await?;
                    break;
                }
                SchemaRetained::LOGICAL_LABEL => {
                    branch.recv::<SchemaRetained>().await?;
                }
                ChangeSchema::LOGICAL_LABEL => {
                    branch.recv::<ChangeSchema>().await?;
                    break;
                }
                ReleaseDatabase::LOGICAL_LABEL => {
                    branch.recv::<ReleaseDatabase>().await?;
                    db.send::<Loan<DATABASE>>(resource).await?;
                    return Ok(());
                }
                label => {
                    return Err(ShellError::message(format!(
                        "unexpected migrate continuation {label}"
                    )));
                }
            }
        }
    }
}

#[hibana_runtime::scope]
pub async fn define(
    api: &mut Endpoint<API_SERVICE>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    let mut resource = api.recv::<Loan<API_GATEWAY>>().await?;
    loop {
        let arguments = resource.recv::<DefineApi>().await?;
        let result = effects.apply(arguments).await?;
        let bytes = encode(&result)?;
        async {
            let [mut file, mut recipient] = resource.par()?;
            async { file.send::<ApiDefined>(&bytes.as_slice()).await };
            async { recipient.send::<ApiContract>(&bytes.as_slice()).await };
        }
        .await?;

        loop {
            let branch = api.offer().await?;
            match branch.label() {
                ApiRetained::LOGICAL_LABEL => {
                    branch.recv::<ApiRetained>().await?;
                }
                ChangeApi::LOGICAL_LABEL => {
                    branch.recv::<ChangeApi>().await?;
                    break;
                }
                ReleaseGateway::LOGICAL_LABEL => {
                    branch.recv::<ReleaseGateway>().await?;
                    api.send::<Loan<API_GATEWAY>>(resource).await?;
                    return Ok(());
                }
                label => {
                    return Err(ShellError::message(format!(
                        "unexpected define continuation {label}"
                    )));
                }
            }
        }
    }
}

#[hibana_runtime::scope]
pub async fn implement(
    backend: &mut Endpoint<BACKEND_SERVICE>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    let mut schema = backend.recv::<SchemaReady>().await?.to_vec();
    backend.send::<SchemaReady>(&schema.as_slice()).await?;
    let branch = backend.offer().await?;
    let first = match branch.label() {
        NeedSchema::LOGICAL_LABEL => {
            let request = decode(branch.recv::<NeedSchema>().await?.as_bytes())?;
            let migration = decode(&schema)?;
            let bytes = encode(&json!({
                "schema": migration["data"]["revision"],
                "reason": request["reason"],
                "status": "migration_requested",
            }))?;
            backend.send::<RequestMigration>(&bytes.as_slice()).await?;
            schema = backend.recv::<SchemaReady>().await?.to_vec();
            backend.send::<SchemaReady>(&schema.as_slice()).await?;
            backend.recv::<ImplementBackend>().await?
        }
        ImplementBackend::LOGICAL_LABEL => branch.recv::<ImplementBackend>().await?,
        label => {
            return Err(ShellError::message(format!(
                "unexpected initial implementation choice {label}"
            )));
        }
    };
    let result = effects.apply(first).await?;
    if result["data"]["inputs"]["schema"] != decode(&schema)?["data"]["revision"] {
        return Err(ShellError::message(
            "backend used a different schema revision",
        ));
    }
    let bytes = encode(&result)?;
    async {
        let [mut file, mut reviewer] = backend.par()?;
        async { file.send::<BackendImplemented>(&bytes.as_slice()).await };
        async { reviewer.send::<BackendCode>(&bytes.as_slice()).await };
    }
    .await?;
    backend.send::<SchemaInUse>(&bytes.as_slice()).await?;
    'schema: loop {
        let migration = decode(&schema)?;
        let revision = migration["data"]["revision"]
            .as_u64()
            .ok_or_else(|| ShellError::message("migration returned no schema revision"))?;
        loop {
            loop {
                let branch = backend.offer().await?;
                match branch.label() {
                    BackendAccepted::LOGICAL_LABEL => {
                        branch.recv::<BackendAccepted>().await?;
                    }
                    BackendRetained::LOGICAL_LABEL => {
                        branch.recv::<BackendRetained>().await?;
                    }
                    BackendChanges::LOGICAL_LABEL => {
                        branch.recv::<BackendChanges>().await?;
                        break;
                    }
                    BackendChangesAfterTest::LOGICAL_LABEL => {
                        branch.recv::<BackendChangesAfterTest>().await?;
                        break;
                    }
                    BackendRepair::LOGICAL_LABEL => {
                        branch.recv::<BackendRepair>().await?;
                        break;
                    }
                    BackendSchemaInvalidated::LOGICAL_LABEL => {
                        branch.recv::<BackendSchemaInvalidated>().await?;
                        schema = backend.recv::<SchemaReady>().await?.to_vec();
                        let expected_schema = decode(&schema)?["data"]["revision"].clone();
                        async {
                            let [mut notice, mut implementation] = backend.par()?;
                            async {
                                notice.send::<SchemaReady>(&schema.as_slice()).await?;
                            };
                            async {
                                let arguments = implementation.recv::<ImplementBackend>().await?;
                                let result = effects.apply(arguments).await?;
                                if result["data"]["inputs"]["schema"] != expected_schema {
                                    return Err(ShellError::message(
                                        "backend used a different schema revision",
                                    ));
                                }
                                let reply = encode(&result)?;
                                async {
                                    let [mut file, mut reviewer] = implementation.par()?;
                                    async {
                                        file.send::<BackendImplemented>(&reply.as_slice()).await
                                    };
                                    async { reviewer.send::<BackendCode>(&reply.as_slice()).await };
                                }
                                .await?;
                            };
                            Ok(())
                        }
                        .await?;
                        continue 'schema;
                    }
                    BackendIntegrationPassed::LOGICAL_LABEL => {
                        branch.recv::<BackendIntegrationPassed>().await?;
                        return Ok(());
                    }
                    other => {
                        return Err(ShellError::message(format!(
                            "unexpected backend continuation {other}"
                        )));
                    }
                }
            }
            let arguments = backend.recv::<ImplementBackend>().await?;
            let result = effects.apply(arguments).await?;
            if result["data"]["inputs"]["schema"] != revision {
                return Err(ShellError::message(
                    "backend used a different schema revision",
                ));
            }
            let bytes = encode(&result)?;
            async {
                let [mut file, mut reviewer] = backend.par()?;
                async { file.send::<BackendImplemented>(&bytes.as_slice()).await };
                async { reviewer.send::<BackendCode>(&bytes.as_slice()).await };
            }
            .await?;
        }
    }
}

#[hibana_runtime::scope]
pub async fn review(
    reviewer: &mut Endpoint<REVIEW_SERVICE>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    let (mut contract, mut backend) = async {
        let [mut code, mut spec] = reviewer.par()?;
        async { decode(spec.recv::<ApiContract>().await?) };
        async { decode(code.recv::<BackendCode>().await?) };
    }
    .await?;
    let mut retained_ui: Option<Value> = None;
    let mut failed_test: Option<Value> = None;
    let actual = 'backend: loop {
        let approved_backend = loop {
            let subject = json!({
                "schema":backend["data"]["inputs"]["schema"],
                "api":contract["data"]["contract"],
                "code":backend["data"]["revision"],
            });
            if let Some(ui) = &retained_ui
                && (ui["data"]["inputs"]["schema"] != subject["schema"]
                    || ui["data"]["inputs"]["api"] != subject["api"])
            {
                return Err(ShellError::message(
                    "backend repair would invalidate the retained UI",
                ));
            }
            let bytes = encode(&subject)?;
            reviewer.send::<BackendCandidate>(&bytes.as_slice()).await?;
            let arguments = reviewer.recv::<ReviewBackend>().await?;
            let result = effects.apply(arguments).await?;
            if result["data"]["inputs"] != subject {
                return Err(ShellError::message("review used a different backend"));
            }
            let bytes = encode(&result)?;
            if result["success"] == true {
                reviewer
                    .send::<BackendAccepted>(&ReviewReply::from_bytes(&bytes)?)
                    .await?;
                async {
                    let [mut file, mut backend, mut frontend] = reviewer.par()?;
                    async { file.send::<BackendReviewed>(&bytes.as_slice()).await };
                    async {
                        backend
                            .send::<Outcome<BACKEND_FILE>>(&bytes.as_slice())
                            .await
                    };
                    async { frontend.send::<BackendApproval>(&bytes.as_slice()).await };
                }
                .await?;
                break result;
            }
            let rejection = ReviewReply::from_bytes(&bytes)?;
            if failed_test.is_some() {
                reviewer.send::<BackendChangesAfterTest>(&rejection).await?;
            } else {
                reviewer.send::<BackendChanges>(&rejection).await?;
            }
            let ((), revised) = async {
                let [mut file, mut backend] = reviewer.par()?;
                async { file.send::<BackendReviewed>(&bytes.as_slice()).await };
                async {
                    backend
                        .send::<Outcome<BACKEND_FILE>>(&bytes.as_slice())
                        .await?;
                    decode(backend.recv::<BackendCode>().await?)
                };
            }
            .await?;
            backend = revised;
        };

        loop {
            let approved_ui = if let Some(ui) = retained_ui.take() {
                ui
            } else {
                loop {
                    let ui = decode(reviewer.recv::<UiCode>().await?)?;
                    let subject = json!({
                        "schema":ui["data"]["inputs"]["schema"],
                        "api":ui["data"]["inputs"]["api"],
                        "code":ui["data"]["revision"],
                    });
                    if let Some(failure) = &failed_test
                        && subject["code"] == failure["data"]["inputs"]["candidate"]["frontend"]
                    {
                        return Err(ShellError::message(
                            "frontend revision already failed integration",
                        ));
                    }
                    let bytes = encode(&subject)?;
                    reviewer.send::<UiCandidate>(&bytes.as_slice()).await?;
                    let arguments = reviewer.recv::<ReviewFrontend>().await?;
                    let reviewed = effects.apply(arguments).await?;
                    if reviewed["data"]["inputs"] != subject {
                        return Err(ShellError::message("review used a different frontend"));
                    }
                    let bytes = encode(&reviewed)?;
                    if reviewed["success"] == true {
                        reviewer
                            .send::<UiAccepted>(&ReviewReply::from_bytes(&bytes)?)
                            .await?;
                    } else {
                        let rejection = ReviewReply::from_bytes(&bytes)?;
                        if failed_test.is_some() {
                            reviewer.send::<UiChangesAfterTest>(&rejection).await?;
                        } else {
                            reviewer.send::<UiChanges>(&rejection).await?;
                        }
                    }
                    async {
                        let [mut file, mut frontend] = reviewer.par()?;
                        async { file.send::<FrontendReviewed>(&bytes.as_slice()).await };
                        async {
                            frontend
                                .send::<Outcome<FRONTEND_FILE>>(&bytes.as_slice())
                                .await
                        };
                    }
                    .await?;
                    if reviewed["success"] == true {
                        break reviewed;
                    }
                }
            };
            let expected = encode(&json!({
                "candidate": {
                    "schema":approved_ui["data"]["inputs"]["schema"],
                    "api":approved_ui["data"]["inputs"]["api"],
                    "backend":backend["data"]["revision"],
                    "frontend":approved_ui["data"]["inputs"]["code"],
                },
                "backend_review":approved_backend["data"]["revision"],
                "frontend_review":approved_ui["data"]["revision"],
            }))?;
            reviewer.send::<TestInputs>(&expected.as_slice()).await?;
            let actual = reviewer.recv::<IntegrationReport>().await?.to_vec();
            let failure = decode(&actual)?;
            if failure["success"] == true {
                break 'backend actual;
            }
            reviewer
                .send::<Outcome<REVIEW_FILE>>(&actual.as_slice())
                .await?;

            let repair = decode(reviewer.recv::<RepairDecision>().await?.as_bytes())?;
            let reply = ReviewReply::from_bytes(&actual)?;
            let notice = encode(&json!({"repair": repair, "test": failure}))?;
            let component = repair["component"].as_str();
            if matches!(component, Some("frontend" | "backend")) {
                reviewer.send::<SchemaRetained>(&reply).await?;
                async {
                    let [mut api, mut db] = reviewer.par()?;
                    async { api.send::<ApiRetained>(&reply).await };
                    async { db.send::<Outcome<DB_FILE>>(&notice.as_slice()).await };
                }
                .await?;
            }
            match component {
                Some("frontend") => {
                    reviewer.send::<IntegrationFailed>(&reply).await?;
                    async {
                        let [mut frontend, mut backend] = reviewer.par()?;
                        async {
                            frontend
                                .send::<Outcome<FRONTEND_FILE>>(&notice.as_slice())
                                .await
                        };
                        async {
                            backend.send::<BackendRetained>(&reply).await?;
                            backend
                                .send::<Outcome<BACKEND_FILE>>(&notice.as_slice())
                                .await
                        };
                    }
                    .await?;
                    failed_test = Some(failure);
                }
                Some("backend") => {
                    reviewer.send::<BackendRepair>(&reply).await?;
                    async {
                        let [mut frontend, mut backend] = reviewer.par()?;
                        async {
                            frontend.send::<UiRetained>(&reply).await?;
                            frontend
                                .send::<Outcome<FRONTEND_FILE>>(&notice.as_slice())
                                .await
                        };
                        async {
                            backend
                                .send::<Outcome<BACKEND_FILE>>(&notice.as_slice())
                                .await
                        };
                    }
                    .await?;
                    backend = decode(reviewer.recv::<BackendCode>().await?)?;
                    if backend["data"]["revision"]
                        == failure["data"]["inputs"]["candidate"]["backend"]
                    {
                        return Err(ShellError::message(
                            "backend revision already failed integration",
                        ));
                    }
                    retained_ui = Some(approved_ui);
                    failed_test = Some(failure);
                    continue 'backend;
                }
                Some("api") => {
                    reviewer.send::<ChangeApi>(&reply).await?;
                    async {
                        let [mut db, mut ui, mut backend] = reviewer.par()?;
                        async {
                            db.send::<SchemaRetained>(&reply).await?;
                            db.send::<Outcome<DB_FILE>>(&notice.as_slice()).await
                        };
                        async {
                            ui.send::<UiInvalidated>(&reply).await?;
                            ui.send::<Outcome<FRONTEND_FILE>>(&notice.as_slice()).await
                        };
                        async {
                            backend.send::<BackendRetained>(&reply).await?;
                            backend
                                .send::<Outcome<BACKEND_FILE>>(&notice.as_slice())
                                .await
                        };
                    }
                    .await?;
                    contract = decode(reviewer.recv::<ApiContract>().await?)?;
                    failed_test = Some(failure);
                    continue 'backend;
                }
                Some("schema") => {
                    reviewer.send::<ChangeSchema>(&reply).await?;
                    async {
                        let [mut api, mut db, mut ui, mut backend] = reviewer.par()?;
                        async { api.send::<ApiRetained>(&reply).await };
                        async { db.send::<Outcome<DB_FILE>>(&notice.as_slice()).await };
                        async {
                            ui.send::<UiInvalidated>(&reply).await?;
                            ui.send::<Outcome<FRONTEND_FILE>>(&notice.as_slice()).await
                        };
                        async {
                            backend.send::<BackendSchemaInvalidated>(&reply).await?;
                            backend
                                .send::<Outcome<BACKEND_FILE>>(&notice.as_slice())
                                .await
                        };
                    }
                    .await?;
                    backend = decode(reviewer.recv::<BackendCode>().await?)?;
                    if backend["data"]["inputs"]["schema"]
                        == failure["data"]["inputs"]["candidate"]["schema"]
                    {
                        return Err(ShellError::message(
                            "schema repair reused the failed schema",
                        ));
                    }
                    failed_test = Some(failure);
                    continue 'backend;
                }
                _ => unreachable!("validated repair component"),
            }
        }
    };
    let reply = ReviewReply::from_bytes(&actual)?;
    async {
        let [mut db, mut own, mut ui, mut backend] = reviewer.par()?;
        async { db.send::<Outcome<DB_FILE>>(&actual.as_slice()).await };
        async { own.send::<Outcome<REVIEW_FILE>>(&actual.as_slice()).await };
        async {
            ui.send::<IntegrationPassed>(&reply).await?;
            ui.send::<Outcome<FRONTEND_FILE>>(&actual.as_slice()).await
        };
        async {
            backend.send::<BackendIntegrationPassed>(&reply).await?;
            backend
                .send::<Outcome<BACKEND_FILE>>(&actual.as_slice())
                .await
        };
    }
    .await?;
    Ok(())
}

#[hibana_runtime::scope]
pub async fn frontend(
    frontend: &mut Endpoint<FRONTEND_SERVICE>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    let mut approval = frontend.recv::<BackendApproval>().await?.to_vec();
    'interface: loop {
        let backend = decode(&approval)?;
        let mut expected = json!({
            "schema":backend["data"]["inputs"]["schema"],
            "api":backend["data"]["inputs"]["api"],
            "backend_review":backend["data"]["revision"],
        });
        frontend.send::<UiInputs>(&approval.as_slice()).await?;

        loop {
            let arguments = frontend.recv::<ImplementFrontend>().await?;
            let result = effects.apply(arguments).await?;
            if result["data"]["inputs"] != expected {
                return Err(ShellError::message(
                    "frontend used a different approved backend",
                ));
            }
            let bytes = encode(&result)?;
            async {
                let [mut file, mut reviewer] = frontend.par()?;
                async { file.send::<FrontendImplemented>(&bytes.as_slice()).await };
                async { reviewer.send::<UiCode>(&bytes.as_slice()).await };
            }
            .await?;
            let branch = frontend.offer().await?;
            match branch.label() {
                UiChanges::LOGICAL_LABEL => {
                    branch.recv::<UiChanges>().await?;
                }
                UiChangesAfterTest::LOGICAL_LABEL => {
                    branch.recv::<UiChangesAfterTest>().await?;
                }
                UiAccepted::LOGICAL_LABEL => {
                    branch.recv::<UiAccepted>().await?;
                    loop {
                        let verdict = frontend.offer().await?;
                        match verdict.label() {
                            IntegrationFailed::LOGICAL_LABEL => {
                                verdict.recv::<IntegrationFailed>().await?;
                                break;
                            }
                            UiRetained::LOGICAL_LABEL => {
                                verdict.recv::<UiRetained>().await?;
                                let updated = decode(frontend.recv::<BackendApproval>().await?)?;
                                if updated["data"]["inputs"]["schema"] != expected["schema"]
                                    || updated["data"]["inputs"]["api"] != expected["api"]
                                {
                                    return Err(ShellError::message(
                                        "retained UI no longer matches the approved backend",
                                    ));
                                }
                                expected["backend_review"] = updated["data"]["revision"].clone();
                            }
                            UiInvalidated::LOGICAL_LABEL => {
                                verdict.recv::<UiInvalidated>().await?;
                                approval = frontend.recv::<BackendApproval>().await?.to_vec();
                                continue 'interface;
                            }
                            IntegrationPassed::LOGICAL_LABEL => {
                                verdict.recv::<IntegrationPassed>().await?;
                                return Ok(());
                            }
                            label => {
                                return Err(ShellError::message(format!(
                                    "unexpected integration result {label}"
                                )));
                            }
                        }
                    }
                }
                label => {
                    return Err(ShellError::message(format!(
                        "unexpected UI outcome {label}"
                    )));
                }
            }
        }
    }
}

#[hibana_runtime::scope]
pub async fn test(
    tester: &mut Endpoint<TEST_SERVICE>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    let mut environment = tester.recv::<Loan<TEST_ENV>>().await?;
    loop {
        let expected = environment.recv::<TestInputs>().await?.to_vec();
        environment.send::<TestInputs>(&expected.as_slice()).await?;
        let arguments = environment.recv::<RunIntegration>().await?;
        let result = effects.apply(arguments).await?;
        if result["data"]["inputs"] != decode(&expected)? {
            return Err(ShellError::message(
                "integration ran against different approved inputs",
            ));
        }
        let bytes = encode(&result)?;
        if result["success"] == true {
            environment
                .send::<IntegrationPassed>(&ReviewReply::from_bytes(&bytes)?)
                .await?;
            environment
                .send::<IntegrationResult>(&bytes.as_slice())
                .await?;
            break;
        }
        environment
            .send::<IntegrationFailed>(&ReviewReply::from_bytes(&bytes)?)
            .await?;
        environment
            .send::<IntegrationResult>(&bytes.as_slice())
            .await?;
    }
    tester.send::<Loan<TEST_ENV>>(environment).await?;
    Ok(())
}
