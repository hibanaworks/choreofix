//! Physical owners consume actual requests before calling the original domain.
use super::super::{global::*, imp};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::Endpoint;
use serde_json::{Value, json};
#[hibana_runtime::scope]
pub async fn repository<'r, D: Domain>(
    endpoint: &mut Endpoint<'r, STORE>,
    resources: (Endpoint<'r, REPO>, Endpoint<'r, TEST_ENV>),
    domain: &D,
) -> ShellResult<()> {
    let (repo, environment) = resources;
    let branch = endpoint.offer().await?;
    let (request, first_agent, first_target, second_agent, second_target) = match branch.label() {
        1 => (
            imp::arguments(
                branch.recv::<DesignA>().await?,
                "title",
                "module_a_refactor",
            )?,
            "DEVELOPER_A",
            "module_a",
            "DEVELOPER_B",
            "module_b",
        ),
        2 => (
            imp::arguments(
                branch.recv::<DesignB>().await?,
                "title",
                "module_b_refactor",
            )?,
            "DEVELOPER_B",
            "module_b",
            "DEVELOPER_A",
            "module_a",
        ),
        _ => return Err(ShellError::message("unexpected design request")),
    };
    let first_design = domain
        .execute("ARCHITECT", "design_proposal", request)
        .await
        .map_err(ShellError::message)?;
    imp::successful(&first_design)?;
    let bytes = imp::encode(&first_design)?;
    endpoint.send::<DesignResult>(&bytes.as_slice()).await?;
    endpoint.send::<DesignResult>(&bytes.as_slice()).await?;
    let (second_design, (repo, environment, first_candidate, first_tests)) = async {
        let [mut design, mut operation] = endpoint.par()?;
        async {
            let branch = design.offer().await?;
            let request = match branch.label() {
                1 => imp::arguments(
                    branch.recv::<DesignA>().await?,
                    "title",
                    "module_a_refactor",
                )?,
                2 => imp::arguments(
                    branch.recv::<DesignB>().await?,
                    "title",
                    "module_b_refactor",
                )?,
                _ => return Err(ShellError::message("unexpected second design")),
            };
            let result = domain
                .execute("ARCHITECT", "design_proposal", request)
                .await
                .map_err(ShellError::message)?;
            imp::successful(&result)?;
            let bytes = imp::encode(&result)?;
            design.send::<DesignResult>(&bytes.as_slice()).await?;
            design.send::<DesignResult>(&bytes.as_slice()).await?;
            Ok::<_, ShellError>(result)
        };
        async {
            operation.send::<RepoLoan>(repo).await?;
            let mut changes = Vec::new();
            for _ in 0..2 {
                let branch = operation.offer().await?;
                let admitted = match branch.label() {
                    4 => imp::arguments(branch.recv::<OwnIntent>().await?, "target", first_target)?,
                    5 => imp::arguments(
                        branch.recv::<SharedIntent>().await?,
                        "target",
                        "SHARED_LIBRARY",
                    )?,
                    _ => return Err(ShellError::message("unexpected refactor intent")),
                };
                let request = imp::decode(operation.recv::<Refactor>().await?)?;
                imp::same(&request, &admitted)?;
                let result = domain
                    .execute(first_agent, "refactor_code", request.clone())
                    .await
                    .map_err(ShellError::message)?;
                imp::successful(&result)?;
                let bytes = imp::encode(&result)?;
                operation.send::<Refactored>(&bytes.as_slice()).await?;
                changes.push(json!({"arguments":request,"result":result}));
            }
            let repo = operation.recv::<RepoLoan>().await?;
            let first_candidate = imp::candidate(&Value::Null, &changes[0], &changes[1]);
            let bytes = imp::encode(&first_candidate)?;
            operation.send::<TestPlan>(&bytes.as_slice()).await?;
            async {
                let [mut r, mut e] = operation.par()?;
                async {
                    r.send::<RepoLoan>(repo).await?;
                };
                async {
                    e.send::<TestLoan>(environment).await?;
                };
                Ok(())
            }
            .await?;
            let mut first_tests = Vec::new();
            for _ in 0..2 {
                let branch = operation.offer().await?;
                let admitted = match branch.label() {
                    13 => imp::arguments(branch.recv::<TestA>().await?, "suite", "module_a_tests")?,
                    14 => imp::arguments(branch.recv::<TestB>().await?, "suite", "module_b_tests")?,
                    _ => return Err(ShellError::message("unexpected suite request")),
                };
                let (read, run) = async {
                    let [mut r, mut e] = operation.par()?;
                    async { imp::decode(r.recv::<ReadSnapshot>().await?) };
                    async { imp::decode(e.recv::<RunTests>().await?) };
                }
                .await?;
                let expected = json!({"candidate":first_candidate,"arguments":admitted});
                imp::same(&read, &expected)?;
                imp::same(&run, &expected)?;
                let result = domain
                    .execute("TESTER", "run_tests", admitted.clone())
                    .await
                    .map_err(ShellError::message)?;
                imp::successful(&result)?;
                let tested =
                    json!({"candidate":first_candidate,"arguments":admitted,"result":result});
                let bytes = imp::encode(&tested)?;
                async {
                    let [mut r, mut e] = operation.par()?;
                    async {
                        r.send::<RepositoryTested>(&bytes.as_slice()).await?;
                    };
                    async {
                        e.send::<EnvironmentTested>(&bytes.as_slice()).await?;
                    };
                    Ok(())
                }
                .await?;
                first_tests.push(tested);
            }
            let (repo, environment) = async {
                let [mut r, mut e] = operation.par()?;
                async { Ok::<_, ShellError>(r.recv::<RepoLoan>().await?) };
                async { Ok::<_, ShellError>(e.recv::<TestLoan>().await?) };
            }
            .await?;
            Ok::<_, ShellError>((repo, environment, first_candidate, first_tests))
        };
    }
    .await?;
    endpoint.send::<RepoLoan>(repo).await?;
    let mut changes = Vec::new();
    for _ in 0..2 {
        let branch = endpoint.offer().await?;
        let admitted = match branch.label() {
            4 => imp::arguments(branch.recv::<OwnIntent>().await?, "target", second_target)?,
            5 => imp::arguments(
                branch.recv::<SharedIntent>().await?,
                "target",
                "SHARED_LIBRARY",
            )?,
            _ => return Err(ShellError::message("unexpected refactor intent")),
        };
        let request = imp::decode(endpoint.recv::<Refactor>().await?)?;
        imp::same(&request, &admitted)?;
        let result = domain
            .execute(second_agent, "refactor_code", request.clone())
            .await
            .map_err(ShellError::message)?;
        imp::successful(&result)?;
        let bytes = imp::encode(&result)?;
        endpoint.send::<Refactored>(&bytes.as_slice()).await?;
        changes.push(json!({"arguments":request,"result":result}));
    }
    let repo = endpoint.recv::<RepoLoan>().await?;
    let candidate = imp::candidate(&first_candidate, &changes[0], &changes[1]);
    let candidate = json!({"designs":[first_design,second_design],"repository":candidate});
    let bytes = imp::encode(&candidate)?;
    async {
        let [mut caller, mut owner] = endpoint.par()?;
        async {
            caller.send::<Candidate>(&bytes.as_slice()).await?;
        };
        async {
            owner.send::<CandidateForOwner>(&bytes.as_slice()).await?;
        };
        Ok(())
    }
    .await?;
    let branch = endpoint.offer().await?;
    let (
        accepted,
        first_review_agent,
        first_review_target,
        second_review_agent,
        second_review_target,
    ) = match branch.label() {
        23 => (
            imp::arguments(branch.recv::<AcceptedA>().await?, "pr_id", "module_a")?,
            "DEVELOPER_A",
            "module_a",
            "DEVELOPER_B",
            "module_b",
        ),
        24 => (
            imp::arguments(branch.recv::<AcceptedB>().await?, "pr_id", "module_b")?,
            "DEVELOPER_B",
            "module_b",
            "DEVELOPER_A",
            "module_a",
        ),
        _ => return Err(ShellError::message("unexpected accepted review")),
    };
    let (first_review, (repo, environment, final_tests)) = async {
        let [mut review, mut tests] = endpoint.par()?;
        async {
            let result = imp::decode(review.recv::<Approved>().await?)?;
            imp::same(&result["candidate"], &candidate)?;
            imp::same(&result["arguments"], &accepted)?;
            imp::successful(&result["result"])?;
            Ok::<_, ShellError>(result)
        };
        async {
            let bytes = imp::encode(&candidate)?;
            tests.send::<TestPlan>(&bytes.as_slice()).await?;
            async {
                let [mut r, mut e] = tests.par()?;
                async {
                    r.send::<RepoLoan>(repo).await?;
                };
                async {
                    e.send::<TestLoan>(environment).await?;
                };
                Ok(())
            }
            .await?;
            let mut final_tests = Vec::new();
            for _ in 0..2 {
                let branch = tests.offer().await?;
                let admitted = match branch.label() {
                    13 => imp::arguments(branch.recv::<TestA>().await?, "suite", "module_a_tests")?,
                    14 => imp::arguments(branch.recv::<TestB>().await?, "suite", "module_b_tests")?,
                    _ => return Err(ShellError::message("unexpected suite request")),
                };
                let (read, run) = async {
                    let [mut r, mut e] = tests.par()?;
                    async { imp::decode(r.recv::<ReadSnapshot>().await?) };
                    async { imp::decode(e.recv::<RunTests>().await?) };
                }
                .await?;
                let expected = json!({"candidate":candidate,"arguments":admitted});
                imp::same(&read, &expected)?;
                imp::same(&run, &expected)?;
                let result = domain
                    .execute("TESTER", "run_tests", admitted.clone())
                    .await
                    .map_err(ShellError::message)?;
                imp::successful(&result)?;
                let tested = json!({"candidate":candidate,"arguments":admitted,"result":result});
                let bytes = imp::encode(&tested)?;
                async {
                    let [mut r, mut e] = tests.par()?;
                    async {
                        r.send::<RepositoryTested>(&bytes.as_slice()).await?;
                    };
                    async {
                        e.send::<EnvironmentTested>(&bytes.as_slice()).await?;
                    };
                    Ok(())
                }
                .await?;
                final_tests.push(tested);
            }
            let (repo, environment) = async {
                let [mut r, mut e] = tests.par()?;
                async { Ok::<_, ShellError>(r.recv::<RepoLoan>().await?) };
                async { Ok::<_, ShellError>(e.recv::<TestLoan>().await?) };
            }
            .await?;
            let summary = json!({"candidate":candidate,"rounds":[first_tests,final_tests]});
            let bytes = imp::encode(&summary)?;
            async {
                let [mut a, mut b] = tests.par()?;
                async {
                    a.send::<TestSummary>(&bytes.as_slice()).await?;
                };
                async {
                    b.send::<TestSummaryB>(&bytes.as_slice()).await?;
                };
                Ok(())
            }
            .await?;
            Ok::<_, ShellError>((repo, environment, summary))
        };
    }
    .await?;
    let all_tests = final_tests;
    let (repo, second_review) = async {
        let [mut merging, mut review] = endpoint.par()?;
        async {
            let ready = json!({"candidate":candidate,"tests":all_tests,"review":first_review});
            let bytes = imp::encode(&ready)?;
            merging.send::<MergeReady>(&bytes.as_slice()).await?;
            merging.send::<RepoLoan>(repo).await?;
            let admitted = imp::arguments(
                merging.recv::<MergeIntent>().await?,
                "branch",
                first_review_target,
            )?;
            let request = imp::decode(merging.recv::<MergeRequest>().await?)?;
            imp::same(&request, &json!({"arguments":admitted,"checked":ready}))?;
            let result = domain
                .execute(first_review_agent, "merge_code", admitted)
                .await
                .map_err(ShellError::message)?;
            imp::successful(&result)?;
            let bytes = imp::encode(&result)?;
            merging.send::<Merged>(&bytes.as_slice()).await?;
            let repo = merging.recv::<RepoLoan>().await?;
            Ok::<_, ShellError>(repo)
        };
        async {
            let branch = review.offer().await?;
            let accepted = match branch.label() {
                23 => imp::arguments(
                    branch.recv::<AcceptedA>().await?,
                    "pr_id",
                    second_review_target,
                )?,
                24 => imp::arguments(
                    branch.recv::<AcceptedB>().await?,
                    "pr_id",
                    second_review_target,
                )?,
                _ => return Err(ShellError::message("unexpected second review")),
            };
            let result = imp::decode(review.recv::<Approved>().await?)?;
            imp::same(&result["candidate"], &candidate)?;
            imp::same(&result["arguments"], &accepted)?;
            imp::successful(&result["result"])?;
            Ok::<_, ShellError>(result)
        };
    }
    .await?;
    let ready = json!({"candidate":candidate,"tests":all_tests,"review":second_review});
    let bytes = imp::encode(&ready)?;
    endpoint.send::<MergeReady>(&bytes.as_slice()).await?;
    endpoint.send::<RepoLoan>(repo).await?;
    let admitted = imp::arguments(
        endpoint.recv::<MergeIntent>().await?,
        "branch",
        second_review_target,
    )?;
    let request = imp::decode(endpoint.recv::<MergeRequest>().await?)?;
    imp::same(&request, &json!({"arguments":admitted,"checked":ready}))?;
    let result = domain
        .execute(second_review_agent, "merge_code", admitted)
        .await
        .map_err(ShellError::message)?;
    imp::successful(&result)?;
    let bytes = imp::encode(&result)?;
    endpoint.send::<Merged>(&bytes.as_slice()).await?;
    let repo = endpoint.recv::<RepoLoan>().await?;
    drop((repo, environment));
    Ok(())
}
