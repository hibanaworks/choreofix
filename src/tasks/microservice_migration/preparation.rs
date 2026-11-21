use crate::{
    imp::json::{decode, encode, same},
    shell::{
        ShellError, ShellResult,
        global::{Exit, Loan},
        imp::{
            review_reply::ReviewReply,
            service::{ReceivedEffects, SimulatorOperation},
        },
    },
};
use hibana::{
    Endpoint, g,
    runtime::{
        program::Projectable,
        wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload},
    },
};
use serde_json::{Map, Value};

pub(super) const ARCHITECT_FILE: u8 = 0;
pub(super) const ARCHITECT: u8 = 1;
pub(super) const DEVELOPER_A: u8 = 2;
pub(super) const DEVELOPER_B: u8 = 3;
pub(super) const DEVELOPER_C: u8 = 4;
pub(super) const DEVELOPER_A_FILE: u8 = 5;
pub(super) const DEVELOPER_B_FILE: u8 = 6;
pub(super) const DEVELOPER_C_FILE: u8 = 7;
pub(super) const REVIEWER_A: u8 = 8;
pub(super) const REVIEWER_B: u8 = 9;
pub(super) const REVIEWER_C: u8 = 10;
pub(super) const REVIEW_FILE: u8 = 11;
pub(super) const SERVICE_A: u8 = 12;
pub(super) const SERVICE_B: u8 = 13;
pub(super) const SERVICE_C: u8 = 14;
pub(super) const TESTER_A: u8 = 15;
pub(super) const TEST_FILE: u8 = 16;
pub(super) const TESTER_B: u8 = 17;
pub(super) const TESTER_C: u8 = 18;
pub(super) const TEST_ENV: u8 = 19;
pub(super) const ENV_OWNER: u8 = 20;

pub(super) const DEVOPS_FILE: u8 = 41;
pub(super) const DEVOPS: u8 = 42;
pub(super) const STAGING_ENV: u8 = 46;
pub(super) const STAGING_SERVER: u8 = 47;
pub(super) const STAGING_OWNER: u8 = 49;
pub(super) const SCHEMA_A: u8 = 90;
pub(super) const SCHEMA_B: u8 = 91;
pub(super) const SCHEMA_C: u8 = 92;
pub(super) const DEPLOY_A: u8 = 96;
pub(super) const DEPLOY_B: u8 = 97;
pub(super) const DEPLOY_C: u8 = 98;
pub(super) type Migrate<const S: u8> = g::Msg<S, Arguments<S>>;
pub(super) type Migrated<'a, const S: u8> = g::Msg<S, &'a [u8]>;
pub(super) type Deploy<const SERVICE: u8> = g::Msg<SERVICE, Arguments<SERVICE>>;
pub(super) type Deployment<'a, const OP: u8> = g::Msg<OP, &'a [u8]>;
type OnStaging<'a, const OP: u8> = g::Msg<OP, StagingRequest<'a, OP>>;
type StagingResult<'a> = g::Msg<101, &'a [u8]>;
type Deferred<'a, const SERVICE: u8> = g::Msg<SERVICE, &'a [u8]>;
pub(super) const REGISTRY: u8 = 48;

const SERVICES: [&str; 3] = ["microservice_a", "microservice_b", "microservice_c"];
const PLAN: u8 = 0;

pub(super) type Propose = g::Msg<80, Arguments<PLAN>>;
pub(super) type Proposed<'a, const VIEW: u8> = g::Msg<VIEW, &'a [u8]>;
pub(super) type Implement<const DEVELOPER: u8> = g::Msg<82, Arguments<DEVELOPER>>;
pub(super) type Implemented<'a> = g::Msg<83, &'a [u8]>;
pub(super) type Candidate<'a, const SERVICE: u8> = g::Msg<SERVICE, &'a [u8]>;
pub(super) type Review<const SERVICE: u8> = g::Msg<SERVICE, Arguments<SERVICE>>;
pub(super) type Reviewed<'a, const DEVELOPER: u8> = g::Msg<DEVELOPER, &'a [u8]>;
type Approved<'a> = g::Msg<84, ReviewReply<'a, true>>;
type Rejected<'a> = g::Msg<85, ReviewReply<'a, false>>;
pub(super) type TestCandidate<'a, const SERVICE: u8> = g::Msg<SERVICE, ReviewReply<'a, true>>;

type Passed<'a> = g::Msg<93, ReviewReply<'a, true>>;
type Failed<'a> = g::Msg<94, ReviewReply<'a, false>>;
pub(super) type StagingCandidate<'a, const TESTER: u8> = g::Msg<TESTER, ReviewReply<'a, true>>;
pub(super) type TestReport<'a> = g::Msg<95, &'a [u8]>;

pub(super) type RunTests<const TESTER: u8> = g::Msg<TESTER, Arguments<TESTER>>;
type EnvironmentRequest<'a, const TESTER: u8> = g::Msg<TESTER, TestRequest<'a, TESTER>>;
pub(super) type Tested<'a, const TESTER: u8> = g::Msg<TESTER, &'a [u8]>;

pub(super) struct Arguments<const OPERATION: u8>(Vec<u8>);

impl<const OPERATION: u8> WireEncode for Arguments<OPERATION> {
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, CodecError> {
        output
            .get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(&self.0);
        Ok(self.0.len())
    }
}

impl<const OPERATION: u8> WirePayload for Arguments<OPERATION> {
    const SCHEMA_ID: u32 = 0xac04_4800 | OPERATION as u32;
    type Decoded<'a> = ReceivedPayload<'a, Self>;

    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        let args = value.as_object().ok_or(CodecError::Malformed)?;
        if OPERATION == PLAN {
            if args.get("title").and_then(Value::as_str) != Some("migration_plan")
                || args
                    .keys()
                    .any(|key| key != "title" && key != "dependencies")
            {
                return Err(CodecError::Malformed);
            }
            dependencies(args.get("dependencies"))?;
            return Ok(());
        }
        if matches!(OPERATION, DEPLOY_A | DEPLOY_B | DEPLOY_C) {
            let service = SERVICES[usize::from(OPERATION - DEPLOY_A)];
            return (args.len() == 2
                && value["service"] == service
                && value["environment"] == "staging")
                .then_some(())
                .ok_or(CodecError::Malformed);
        }
        let (key, service) = match OPERATION {
            DEVELOPER_A => ("feature_name", SERVICES[0]),
            DEVELOPER_B => ("feature_name", SERVICES[1]),
            DEVELOPER_C => ("feature_name", SERVICES[2]),
            TESTER_A => ("suite", SERVICES[0]),
            TESTER_B => ("suite", SERVICES[1]),
            TESTER_C => ("suite", SERVICES[2]),
            SCHEMA_A => ("migration_name", "microservice_a_schema"),
            SCHEMA_B => ("migration_name", "microservice_b_schema"),
            SCHEMA_C => ("migration_name", "microservice_c_schema"),
            SERVICE_A => ("pr_id", SERVICES[0]),
            SERVICE_B => ("pr_id", SERVICES[1]),
            SERVICE_C => ("pr_id", SERVICES[2]),
            _ => return Err(CodecError::Malformed),
        };
        if args.len() != 1 || args.get(key).and_then(Value::as_str) != Some(service) {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }

    fn decode_validated_payload(input: ReceivedPayload<'_, Self>) -> Self::Decoded<'_> {
        input
    }
}

impl<const OPERATION: u8> SimulatorOperation for Arguments<OPERATION> {
    const RPC: (&'static str, &'static str) = match OPERATION {
        PLAN => ("ARCHITECT", "design_proposal"),
        DEVELOPER_A => ("DEVELOPER_A", "implement_feature"),
        DEVELOPER_B => ("DEVELOPER_B", "implement_feature"),
        DEVELOPER_C => ("DEVELOPER_C", "implement_feature"),
        SERVICE_A | SERVICE_B | SERVICE_C => ("REVIEWER", "code_review"),
        _ => panic!("not a physical migration request"),
    };
}

/// The physical staging server accepts the exact bytes received on the resource.
struct StagingRequest<'a, const OP: u8>(ReceivedPayload<'a, Arguments<OP>>);

impl<const OP: u8> WireEncode for StagingRequest<'_, OP> {
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, CodecError> {
        self.0.as_bytes().encode_into(output)
    }
}
impl<const OP: u8> WirePayload for StagingRequest<'_, OP> {
    const SCHEMA_ID: u32 = 0xac04_5000 | OP as u32;
    type Decoded<'a> = ReceivedPayload<'a, Self>;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        Arguments::<OP>::validate_payload(input)
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self::Decoded<'a> {
        input
    }
}
impl<const OP: u8> SimulatorOperation for StagingRequest<'_, OP> {
    const RPC: (&'static str, &'static str) = match OP {
        SCHEMA_A | SCHEMA_B | SCHEMA_C => ("DEVOPS", "migrate_schema"),
        DEPLOY_A | DEPLOY_B | DEPLOY_C => ("DEVOPS", "deploy_service"),
        _ => panic!("unknown staging operation"),
    };
}

/// The environment forwards the request consumed from this suite's projected file.
struct TestRequest<'a, const TESTER: u8>(ReceivedPayload<'a, Arguments<TESTER>>);

impl<const TESTER: u8> WireEncode for TestRequest<'_, TESTER> {
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, CodecError> {
        self.0.as_bytes().encode_into(output)
    }
}

impl<const TESTER: u8> WirePayload for TestRequest<'_, TESTER> {
    const SCHEMA_ID: u32 = Arguments::<TESTER>::SCHEMA_ID | 0x100;
    type Decoded<'a> = ReceivedPayload<'a, Self>;

    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        Arguments::<TESTER>::validate_payload(input)
    }

    fn decode_validated_payload(input: ReceivedPayload<'_, Self>) -> Self::Decoded<'_> {
        input
    }
}

impl<const TESTER: u8> SimulatorOperation for TestRequest<'_, TESTER> {
    const RPC: (&'static str, &'static str) = ("TESTER", "run_tests");
}

/// Keep the authored dependencies and expand omitted services to empty lists.
fn dependencies(value: Option<&Value>) -> Result<Map<String, Value>, CodecError> {
    let empty = Map::new();
    let selected = match value {
        Some(value) => value.as_object().ok_or(CodecError::Malformed)?,
        None => &empty,
    };
    if selected
        .keys()
        .any(|name| !SERVICES.contains(&name.as_str()))
    {
        return Err(CodecError::Malformed);
    }
    let mut normalized = Map::new();
    let mut edges = [0_u8; 3];
    for (service, name) in SERVICES.into_iter().enumerate() {
        let names = selected
            .get(name)
            .map(|value| value.as_array().ok_or(CodecError::Malformed))
            .transpose()?;
        for dependency in names.into_iter().flatten() {
            let required = SERVICES
                .iter()
                .position(|other| dependency.as_str() == Some(*other))
                .ok_or(CodecError::Malformed)?;
            let edge = 1 << required;
            if edges[service] & edge != 0 || required == service {
                return Err(CodecError::Malformed);
            }
            edges[service] |= edge;
        }
        normalized.insert(
            name.to_owned(),
            Value::Array(names.cloned().unwrap_or_default()),
        );
    }
    let mut remaining = (1 << SERVICES.len()) - 1;
    for _ in SERVICES {
        let ready = edges
            .iter()
            .enumerate()
            .find(|&(service, edges)| remaining & (1 << service) != 0 && remaining & *edges == 0)
            .ok_or(CodecError::Malformed)?;
        remaining &= !(1 << ready.0);
    }
    Ok(normalized)
}

fn review_candidate<const DEVELOPER: u8, const REVIEWER: u8, const SERVICE: u8>()
-> impl Projectable + Copy {
    g::seq(
        g::send::<DEVELOPER, REVIEWER, Loan<SERVICE>>(),
        g::seq(
            g::send::<DEVELOPER, REVIEWER, Candidate<SERVICE>>(),
            g::seq(
                g::send::<SERVICE, REVIEW_FILE, Candidate<SERVICE>>(),
                g::send::<REVIEW_FILE, SERVICE, Review<SERVICE>>(),
            ),
        ),
    )
}

fn reviewed_revision<const DEVELOPER: u8, const FILE: u8, const SERVICE: u8, const VERDICT: u8>(
    review: impl Projectable + Copy,
    returned: impl Projectable,
) -> impl Projectable {
    let attempt = g::seq(
        g::send::<FILE, SERVICE, Implement<DEVELOPER>>(),
        g::seq(g::send::<SERVICE, FILE, Implemented>(), review),
    );
    let report = g::par(
        g::send::<SERVICE, REVIEW_FILE, Reviewed<DEVELOPER>>(),
        g::send::<DEVELOPER, FILE, Reviewed<DEVELOPER>>(),
    );
    let approved = g::seq(g::send::<VERDICT, DEVELOPER, Approved>(), report);
    let rejected = g::seq(
        g::send::<VERDICT, DEVELOPER, Rejected>(),
        g::seq(report, returned),
    );
    g::seq(
        attempt,
        g::route(approved, g::seq(g::seq(rejected, attempt).roll(), approved)),
    )
}

fn build<const DEVELOPER: u8, const FILE: u8, const SERVICE: u8, const TESTER: u8>(
    review: impl Projectable + Copy,
    returned: impl Projectable,
    handoff: impl Projectable,
) -> impl Projectable {
    g::seq(
        g::send::<ARCHITECT, DEVELOPER, Proposed<DEVELOPER>>(),
        g::seq(
            g::send::<ARCHITECT, FILE, Proposed<DEVELOPER>>(),
            g::seq(
                reviewed_revision::<DEVELOPER, FILE, SERVICE, SERVICE>(review, returned),
                g::seq(
                    g::send::<SERVICE, TESTER, TestCandidate<SERVICE>>(),
                    handoff,
                ),
            ),
        ),
    )
}

fn test_session<const TESTER: u8>() -> impl Projectable + Copy {
    g::seq(
        g::send::<TEST_ENV, TESTER, EnvironmentRequest<TESTER>>(),
        g::seq(
            g::send::<ENV_OWNER, TESTER, Loan<TEST_ENV>>(),
            g::send::<TESTER, ENV_OWNER, Loan<TEST_ENV>>(),
        ),
    )
}

fn request_test<const SERVICE: u8, const TESTER: u8>() -> impl Projectable + Copy {
    g::seq(
        g::send::<SERVICE, TEST_FILE, TestCandidate<SERVICE>>(),
        g::send::<TEST_FILE, ENV_OWNER, RunTests<TESTER>>(),
    )
}

fn test_result<
    const SERVICE: u8,
    const TESTER: u8,
    const REVIEWER: u8,
    const DEVELOPER: u8,
    const FILE: u8,
    const DEPLOY: u8,
>() -> impl Projectable {
    let report = g::send::<DEVELOPER, FILE, TestReport>();
    let reply = g::seq(
        g::send::<SERVICE, ENV_OWNER, TestReport>(),
        g::send::<ENV_OWNER, TEST_FILE, Tested<TESTER>>(),
    );
    let passed = g::seq(
        g::seq(g::send::<SERVICE, DEVELOPER, Passed>(), reply),
        g::par(
            report,
            g::seq(
                g::send::<SERVICE, REVIEWER, Passed>(),
                g::par(
                    g::send::<REVIEWER, REVIEW_FILE, Candidate<SERVICE>>(),
                    g::seq(
                        g::send::<TESTER, DEVOPS, StagingCandidate<TESTER>>(),
                        g::seq(
                            g::send::<TESTER, DEVOPS, Loan<SERVICE>>(),
                            g::seq(
                                g::send::<SERVICE, DEVOPS_FILE, StagingCandidate<TESTER>>(),
                                deploy_candidate::<SERVICE, DEPLOY>(),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );
    let repair = reviewed_revision::<DEVELOPER, FILE, SERVICE, REVIEWER>(
        review_candidate::<DEVELOPER, REVIEWER, SERVICE>(),
        g::send::<REVIEWER, DEVELOPER, Loan<SERVICE>>(),
    );
    let failed = g::seq(
        g::seq(g::send::<SERVICE, DEVELOPER, Failed>(), reply),
        g::seq(
            report,
            g::seq(
                g::send::<TESTER, DEVELOPER, Loan<SERVICE>>(),
                g::seq(
                    repair,
                    g::seq(
                        g::send::<SERVICE, TESTER, TestCandidate<SERVICE>>(),
                        g::seq(
                            g::send::<REVIEWER, TESTER, Loan<SERVICE>>(),
                            request_test::<SERVICE, TESTER>(),
                        ),
                    ),
                ),
            ),
        ),
    );
    g::seq(
        request_test::<SERVICE, TESTER>(),
        g::route(passed, g::seq(failed.roll(), passed)),
    )
}

fn deploy_candidate<const SERVICE: u8, const OP: u8>() -> impl Projectable {
    let request = g::seq(
        g::send::<DEVOPS_FILE, SERVICE, Deploy<OP>>(),
        g::seq(
            g::send::<SERVICE, STAGING_OWNER, StagingCandidate<SERVICE>>(),
            g::seq(
                g::send::<DEVOPS, STAGING_OWNER, Loan<SERVICE>>(),
                g::send::<DEVOPS, STAGING_OWNER, Deploy<OP>>(),
            ),
        ),
    );
    let answer = g::seq(
        g::send::<STAGING_OWNER, DEVOPS, Loan<SERVICE>>(),
        g::send::<SERVICE, DEVOPS_FILE, Deployment<OP>>(),
    );
    let retry = g::seq(
        g::send::<SERVICE, DEVOPS, Deferred<SERVICE>>(),
        g::seq(answer, request),
    );
    let done = g::seq(
        g::send::<SERVICE, DEVOPS, Deployment<OP>>(),
        g::seq(answer, g::send::<DEVOPS, REGISTRY, Loan<SERVICE>>()),
    );
    g::seq(request, g::route(done, g::seq(retry.roll(), done)))
}

fn schema<const S: u8>() -> impl Projectable + Copy {
    g::seq(
        g::send::<DEVOPS_FILE, STAGING_OWNER, Migrate<S>>(),
        g::send::<STAGING_OWNER, DEVOPS_FILE, Migrated<S>>(),
    )
}

pub(super) fn preparation() -> impl Projectable {
    let ma = schema::<SCHEMA_A>();
    let mb = schema::<SCHEMA_B>();
    let mc = schema::<SCHEMA_C>();
    let migrations = g::route(
        g::seq(ma, g::route(g::seq(mb, mc), g::seq(mc, mb))),
        g::route(
            g::seq(mb, g::route(g::seq(ma, mc), g::seq(mc, ma))),
            g::seq(mc, g::route(g::seq(ma, mb), g::seq(mb, ma))),
        ),
    );
    let operations = g::route(
        g::route(
            g::send::<STAGING_ENV, STAGING_SERVER, OnStaging<SCHEMA_A>>(),
            g::route(
                g::send::<STAGING_ENV, STAGING_SERVER, OnStaging<SCHEMA_B>>(),
                g::send::<STAGING_ENV, STAGING_SERVER, OnStaging<SCHEMA_C>>(),
            ),
        ),
        g::route(
            g::send::<STAGING_ENV, STAGING_SERVER, OnStaging<DEPLOY_A>>(),
            g::route(
                g::send::<STAGING_ENV, STAGING_SERVER, OnStaging<DEPLOY_B>>(),
                g::send::<STAGING_ENV, STAGING_SERVER, OnStaging<DEPLOY_C>>(),
            ),
        ),
    );
    let physical = g::seq(
        g::seq(
            operations,
            g::send::<STAGING_SERVER, STAGING_ENV, StagingResult>(),
        )
        .roll(),
        g::send::<STAGING_ENV, STAGING_SERVER, Exit>(),
    );
    let service_a = g::seq(
        build::<DEVELOPER_A, DEVELOPER_A_FILE, SERVICE_A, TESTER_A>(
            review_candidate::<DEVELOPER_A, REVIEWER_A, SERVICE_A>(),
            g::send::<REVIEWER_A, DEVELOPER_A, Loan<SERVICE_A>>(),
            g::send::<REVIEWER_A, TESTER_A, Loan<SERVICE_A>>(),
        ),
        test_result::<SERVICE_A, TESTER_A, REVIEWER_A, DEVELOPER_A, DEVELOPER_A_FILE, DEPLOY_A>(),
    );
    let service_b = g::seq(
        build::<DEVELOPER_B, DEVELOPER_B_FILE, SERVICE_B, TESTER_B>(
            review_candidate::<DEVELOPER_B, REVIEWER_B, SERVICE_B>(),
            g::send::<REVIEWER_B, DEVELOPER_B, Loan<SERVICE_B>>(),
            g::send::<REVIEWER_B, TESTER_B, Loan<SERVICE_B>>(),
        ),
        test_result::<SERVICE_B, TESTER_B, REVIEWER_B, DEVELOPER_B, DEVELOPER_B_FILE, DEPLOY_B>(),
    );
    let service_c = g::seq(
        build::<DEVELOPER_C, DEVELOPER_C_FILE, SERVICE_C, TESTER_C>(
            review_candidate::<DEVELOPER_C, REVIEWER_C, SERVICE_C>(),
            g::send::<REVIEWER_C, DEVELOPER_C, Loan<SERVICE_C>>(),
            g::send::<REVIEWER_C, TESTER_C, Loan<SERVICE_C>>(),
        ),
        test_result::<SERVICE_C, TESTER_C, REVIEWER_C, DEVELOPER_C, DEVELOPER_C_FILE, DEPLOY_C>(),
    );
    let a = test_session::<TESTER_A>().roll();
    let b = test_session::<TESTER_B>().roll();
    let c = test_session::<TESTER_C>().roll();
    let tests = g::seq(
        g::route(
            g::seq(a, g::route(g::seq(b, c), g::seq(c, b))),
            g::route(
                g::seq(b, g::route(g::seq(a, c), g::seq(c, a))),
                g::seq(c, g::route(g::seq(a, b), g::seq(b, a))),
            ),
        ),
        g::send::<TEST_FILE, ENV_OWNER, Exit>(),
    );
    let workshop = g::par(g::par(service_a, g::par(service_b, service_c)), tests);
    g::seq(
        g::send::<ARCHITECT_FILE, ARCHITECT, Propose>(),
        g::par(
            g::send::<ARCHITECT, ARCHITECT_FILE, Proposed<ARCHITECT_FILE>>(),
            g::seq(
                g::send::<ARCHITECT, STAGING_OWNER, Proposed<STAGING_OWNER>>(),
                g::seq(
                    g::send::<ARCHITECT, STAGING_OWNER, Loan<STAGING_ENV>>(),
                    g::seq(
                        g::par(
                            workshop,
                            g::par(
                                g::seq(
                                    g::send::<STAGING_OWNER, DEVOPS_FILE, Proposed<DEVOPS_FILE>>(),
                                    g::seq(
                                        migrations,
                                        g::send::<DEVOPS_FILE, STAGING_OWNER, Exit>(),
                                    ),
                                ),
                                physical,
                            ),
                        ),
                        g::send::<STAGING_OWNER, ARCHITECT, Loan<STAGING_ENV>>(),
                    ),
                ),
            ),
        ),
    )
}

#[hibana_runtime::scope]
pub(super) async fn architect<'run>(
    endpoint: &mut Endpoint<'run, ARCHITECT>,
    staging: Endpoint<'run, STAGING_ENV>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    let received = endpoint.recv::<Propose>().await?;
    let arguments = decode(received.as_bytes())?;
    let expected = Value::Object(dependencies(arguments.get("dependencies"))?);
    let result = effects.apply(received).await?;
    let data = &result["data"];
    if result["tool_name"] != "design_proposal"
        || result["success"] != true
        || data["agent"] != "ARCHITECT"
        || data["title"] != "migration_plan"
        || data["status"] != "proposed"
        || data["receipt"].as_u64().is_none_or(|receipt| receipt == 0)
    {
        return Err(ShellError::message(
            "design result differs from the admitted plan",
        ));
    }
    same(&data["dependencies"], &expected)?;
    let bytes = encode(&result)?;
    let [mut author, mut work] = endpoint.par()?;
    async {
        author
            .send::<Proposed<ARCHITECT_FILE>>(&bytes.as_slice())
            .await
    };
    work.send::<Proposed<STAGING_OWNER>>(&bytes.as_slice())
        .await?;
    work.send::<Loan<STAGING_ENV>>(staging).await?;
    async {
        let [mut a, mut b, mut c] = work.par()?;
        async {
            a.send::<Proposed<DEVELOPER_A>>(&bytes.as_slice()).await?;
            a.send::<Proposed<DEVELOPER_A>>(&bytes.as_slice()).await
        };
        async {
            b.send::<Proposed<DEVELOPER_B>>(&bytes.as_slice()).await?;
            b.send::<Proposed<DEVELOPER_B>>(&bytes.as_slice()).await
        };
        async {
            c.send::<Proposed<DEVELOPER_C>>(&bytes.as_slice()).await?;
            c.send::<Proposed<DEVELOPER_C>>(&bytes.as_slice()).await
        };
        Ok(())
    }
    .await?;
    let _staging = work.recv::<Loan<STAGING_ENV>>().await?;
    Ok(())
}

fn checked_migration(result: &Value, service: &str, name: &Value, plan: &Value) -> ShellResult<()> {
    let data = &result["data"];
    if result["tool_name"] != "migrate_schema"
        || result["success"] != true
        || data["agent"] != "DEVOPS"
        || data["status"] != "migrated"
        || data["service"] != service
        || &data["migration_name"] != name
        || &data["plan"] != plan
        || data["receipt"].as_u64().is_none_or(|id| id == 0)
    {
        return Err(ShellError::message(
            "schema migration reply differs from its request or plan",
        ));
    }
    Ok(())
}

fn deployment_inputs(
    required: &Value,
    deployed: &[Option<Value>; 3],
) -> Result<Map<String, Value>, Value> {
    let mut receipts = Map::new();
    for name in required.as_array().expect("normalized migration plan") {
        let i = SERVICES
            .iter()
            .position(|s| name == *s)
            .expect("validated service");
        let Some(result) = &deployed[i] else {
            return Err(
                serde_json::json!({"success":false,"status":"dependency_pending","service":name}),
            );
        };
        receipts.insert(SERVICES[i].into(), result["data"]["receipt"].clone());
    }
    Ok(receipts)
}

fn checked_deployment(
    result: &Value,
    test: &Value,
    dependencies: &Map<String, Value>,
) -> ShellResult<()> {
    let data = &result["data"];
    if result["tool_name"] != "deploy_service"
        || data["agent"] != "DEVOPS"
        || data["service"] != test["data"]["service"]
        || data["plan"] != test["data"]["plan"]
        || data["candidate"] != test["data"]["candidate"]
        || data["test"] != test["data"]["receipt"]
        || data["dependencies"].as_object() != Some(dependencies)
        || data["receipt"].as_u64().is_none_or(|id| id == 0)
        || !matches!(
            (result["success"].as_bool(), data["status"].as_str()),
            (Some(true), Some("deployed")) | (Some(false), Some("deployment_failed"))
        )
    {
        return Err(ShellError::message(
            "deployment differs from its staging request or prerequisites",
        ));
    }
    Ok(())
}

pub(super) async fn staging_owner(owner: &mut Endpoint<'_, STAGING_OWNER>) -> ShellResult<()> {
    let bytes = owner.recv::<Proposed<STAGING_OWNER>>().await?.to_vec();
    let plan = decode(&bytes)?;
    let mut staging = owner.recv::<Loan<STAGING_ENV>>().await?;
    owner
        .send::<Proposed<DEVOPS_FILE>>(&bytes.as_slice())
        .await?;
    let mut deployed = [None, None, None];
    loop {
        let request = owner.offer().await?;
        match request.label() {
            SCHEMA_A => {
                let admitted = request.recv::<Migrate<SCHEMA_A>>().await?;
                let args = decode(admitted.as_bytes())?;
                staging
                    .send::<OnStaging<SCHEMA_A>>(&StagingRequest(admitted))
                    .await?;
                let result = staging.recv::<StagingResult>().await?.to_vec();
                checked_migration(
                    &decode(&result)?,
                    SERVICES[0],
                    &args["migration_name"],
                    &plan["data"]["receipt"],
                )?;
                owner.send::<Migrated<SCHEMA_A>>(&result.as_slice()).await?;
            }
            SCHEMA_B => {
                let admitted = request.recv::<Migrate<SCHEMA_B>>().await?;
                let args = decode(admitted.as_bytes())?;
                staging
                    .send::<OnStaging<SCHEMA_B>>(&StagingRequest(admitted))
                    .await?;
                let result = staging.recv::<StagingResult>().await?.to_vec();
                checked_migration(
                    &decode(&result)?,
                    SERVICES[1],
                    &args["migration_name"],
                    &plan["data"]["receipt"],
                )?;
                owner.send::<Migrated<SCHEMA_B>>(&result.as_slice()).await?;
            }
            SCHEMA_C => {
                let admitted = request.recv::<Migrate<SCHEMA_C>>().await?;
                let args = decode(admitted.as_bytes())?;
                staging
                    .send::<OnStaging<SCHEMA_C>>(&StagingRequest(admitted))
                    .await?;
                let result = staging.recv::<StagingResult>().await?.to_vec();
                checked_migration(
                    &decode(&result)?,
                    SERVICES[2],
                    &args["migration_name"],
                    &plan["data"]["receipt"],
                )?;
                owner.send::<Migrated<SCHEMA_C>>(&result.as_slice()).await?;
            }
            SERVICE_A => {
                let test = decode(request.recv::<StagingCandidate<SERVICE_A>>().await?)?;
                if test["data"]["service"] != SERVICES[0]
                    || test["data"]["plan"] != plan["data"]["receipt"]
                {
                    return Err(ShellError::message(
                        "tested service differs from the deployment plan",
                    ));
                }
                let mut service = owner.recv::<Loan<SERVICE_A>>().await?;
                let admitted = owner.recv::<Deploy<DEPLOY_A>>().await?;
                let required = &plan["data"]["dependencies"][SERVICES[0]];
                let prerequisites = match deployment_inputs(required, &deployed) {
                    Ok(receipts) => receipts,
                    Err(missing) => {
                        service
                            .send::<Deferred<SERVICE_A>>(&encode(&missing)?.as_slice())
                            .await?;
                        owner.send::<Loan<SERVICE_A>>(service).await?;
                        continue;
                    }
                };
                staging
                    .send::<OnStaging<DEPLOY_A>>(&StagingRequest(admitted))
                    .await?;
                let bytes = staging.recv::<StagingResult>().await?.to_vec();
                let result = decode(&bytes)?;
                checked_deployment(&result, &test, &prerequisites)?;
                if result["success"] == true {
                    deployed[0] = Some(result);
                }
                service
                    .send::<Deployment<DEPLOY_A>>(&bytes.as_slice())
                    .await?;
                owner.send::<Loan<SERVICE_A>>(service).await?;
            }
            SERVICE_B => {
                let test = decode(request.recv::<StagingCandidate<SERVICE_B>>().await?)?;
                if test["data"]["service"] != SERVICES[1]
                    || test["data"]["plan"] != plan["data"]["receipt"]
                {
                    return Err(ShellError::message(
                        "tested service differs from the deployment plan",
                    ));
                }
                let mut service = owner.recv::<Loan<SERVICE_B>>().await?;
                let admitted = owner.recv::<Deploy<DEPLOY_B>>().await?;
                let required = &plan["data"]["dependencies"][SERVICES[1]];
                let prerequisites = match deployment_inputs(required, &deployed) {
                    Ok(receipts) => receipts,
                    Err(missing) => {
                        service
                            .send::<Deferred<SERVICE_B>>(&encode(&missing)?.as_slice())
                            .await?;
                        owner.send::<Loan<SERVICE_B>>(service).await?;
                        continue;
                    }
                };
                staging
                    .send::<OnStaging<DEPLOY_B>>(&StagingRequest(admitted))
                    .await?;
                let bytes = staging.recv::<StagingResult>().await?.to_vec();
                let result = decode(&bytes)?;
                checked_deployment(&result, &test, &prerequisites)?;
                if result["success"] == true {
                    deployed[1] = Some(result);
                }
                service
                    .send::<Deployment<DEPLOY_B>>(&bytes.as_slice())
                    .await?;
                owner.send::<Loan<SERVICE_B>>(service).await?;
            }
            SERVICE_C => {
                let test = decode(request.recv::<StagingCandidate<SERVICE_C>>().await?)?;
                if test["data"]["service"] != SERVICES[2]
                    || test["data"]["plan"] != plan["data"]["receipt"]
                {
                    return Err(ShellError::message(
                        "tested service differs from the deployment plan",
                    ));
                }
                let mut service = owner.recv::<Loan<SERVICE_C>>().await?;
                let admitted = owner.recv::<Deploy<DEPLOY_C>>().await?;
                let required = &plan["data"]["dependencies"][SERVICES[2]];
                let prerequisites = match deployment_inputs(required, &deployed) {
                    Ok(receipts) => receipts,
                    Err(missing) => {
                        service
                            .send::<Deferred<SERVICE_C>>(&encode(&missing)?.as_slice())
                            .await?;
                        owner.send::<Loan<SERVICE_C>>(service).await?;
                        continue;
                    }
                };
                staging
                    .send::<OnStaging<DEPLOY_C>>(&StagingRequest(admitted))
                    .await?;
                let bytes = staging.recv::<StagingResult>().await?.to_vec();
                let result = decode(&bytes)?;
                checked_deployment(&result, &test, &prerequisites)?;
                if result["success"] == true {
                    deployed[2] = Some(result);
                }
                service
                    .send::<Deployment<DEPLOY_C>>(&bytes.as_slice())
                    .await?;
                owner.send::<Loan<SERVICE_C>>(service).await?;
            }
            <Exit as g::Message>::LOGICAL_LABEL => {
                let status = request.recv::<Exit>().await?;
                staging.send::<Exit>(&status).await?;
                owner.send::<Loan<STAGING_ENV>>(staging).await?;
                return Ok(());
            }
            _ => return Err(ShellError::message("unexpected staging request")),
        }
    }
}

pub(super) async fn staging_server(
    server: &mut Endpoint<'_, STAGING_SERVER>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    loop {
        let ready = server.offer().await?;
        let result = match ready.label() {
            SCHEMA_A => {
                effects
                    .apply(ready.recv::<OnStaging<SCHEMA_A>>().await?)
                    .await?
            }
            SCHEMA_B => {
                effects
                    .apply(ready.recv::<OnStaging<SCHEMA_B>>().await?)
                    .await?
            }
            SCHEMA_C => {
                effects
                    .apply(ready.recv::<OnStaging<SCHEMA_C>>().await?)
                    .await?
            }
            DEPLOY_A => {
                effects
                    .apply(ready.recv::<OnStaging<DEPLOY_A>>().await?)
                    .await?
            }
            DEPLOY_B => {
                effects
                    .apply(ready.recv::<OnStaging<DEPLOY_B>>().await?)
                    .await?
            }
            DEPLOY_C => {
                effects
                    .apply(ready.recv::<OnStaging<DEPLOY_C>>().await?)
                    .await?
            }
            <Exit as g::Message>::LOGICAL_LABEL => {
                ready.recv::<Exit>().await?;
                return Ok(());
            }
            _ => return Err(ShellError::message("unexpected physical staging operation")),
        };
        server
            .send::<StagingResult>(&encode(&result)?.as_slice())
            .await?;
    }
}

pub(super) async fn devops_service<const TESTER: u8, const SERVICE: u8, const OP: u8>(
    mut endpoint: hibana::EndpointBorrow<'_, '_, DEVOPS>,
) -> ShellResult<()> {
    let tested = endpoint.recv::<StagingCandidate<TESTER>>().await?.to_vec();
    let test = decode(&tested)?;
    let mut service = endpoint.recv::<Loan<SERVICE>>().await?;
    service
        .send::<StagingCandidate<TESTER>>(&ReviewReply::from_bytes(&tested)?)
        .await?;
    loop {
        let received = service.recv::<Deploy<OP>>().await?;
        let args = Arguments::<OP>(received.as_bytes().to_vec());
        same(&decode(&args.0)?["service"], &test["data"]["service"])?;
        service
            .send::<StagingCandidate<SERVICE>>(&ReviewReply::from_bytes(&tested)?)
            .await?;
        endpoint.send::<Loan<SERVICE>>(service).await?;
        endpoint.send::<Deploy<OP>>(&args).await?;
        let choice = endpoint.offer().await?;
        match choice.label() {
            label if label == SERVICE => {
                let blocked = choice.recv::<Deferred<SERVICE>>().await?.to_vec();
                service = endpoint.recv::<Loan<SERVICE>>().await?;
                service.send::<Deployment<OP>>(&blocked.as_slice()).await?;
            }
            label if label == OP => {
                let bytes = choice.recv::<Deployment<OP>>().await?.to_vec();
                service = endpoint.recv::<Loan<SERVICE>>().await?;
                let result = decode(&bytes)?;
                let data = &result["data"];
                if data["candidate"] != test["data"]["candidate"]
                    || data["test"] != test["data"]["receipt"]
                    || data["plan"] != test["data"]["plan"]
                    || data["service"] != test["data"]["service"]
                {
                    return Err(ShellError::message(
                        "deployment does not match the tested service",
                    ));
                }
                service.send::<Deployment<OP>>(&bytes.as_slice()).await?;
                endpoint.send::<Loan<SERVICE>>(service).await?;
                return Ok(());
            }
            _ => return Err(ShellError::message("unexpected staging response")),
        }
    }
}
fn checked_build(result: &Value, agent: &str, service: &str, plan: &Value) -> ShellResult<()> {
    let data = &result["data"];
    if result["tool_name"] != "implement_feature"
        || result["success"] != true
        || data["agent"] != agent
        || data["feature_name"] != service
        || data["service"] != service
        || data["status"] != "implemented"
        || data["plan"] != plan["data"]["receipt"]
        || data["receipt"].as_u64().is_none_or(|receipt| receipt == 0)
    {
        return Err(ShellError::message(
            "implementation result differs from its service or plan",
        ));
    }
    Ok(())
}

pub(super) async fn developer<'run, const DEVELOPER: u8, const SERVICE: u8>(
    endpoint: &mut Endpoint<'run, DEVELOPER>,
    mut candidate: Endpoint<'run, SERVICE>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    let (agent, service) = match DEVELOPER {
        DEVELOPER_A => ("DEVELOPER_A", SERVICES[0]),
        DEVELOPER_B => ("DEVELOPER_B", SERVICES[1]),
        DEVELOPER_C => ("DEVELOPER_C", SERVICES[2]),
        _ => return Err(ShellError::message("unknown migration developer")),
    };
    let plan = decode(endpoint.recv::<Proposed<DEVELOPER>>().await?)?;
    loop {
        let received = candidate.recv::<Implement<DEVELOPER>>().await?;
        let result = effects.apply(received).await?;
        checked_build(&result, agent, service, &plan)?;
        let implemented = encode(&result)?;
        candidate
            .send::<Implemented>(&implemented.as_slice())
            .await?;
        endpoint.send::<Loan<SERVICE>>(candidate).await?;
        endpoint
            .send::<Candidate<SERVICE>>(&implemented.as_slice())
            .await?;
        let branch = endpoint.offer().await?;
        match branch.label() {
            <Approved as g::Message>::LOGICAL_LABEL => {
                let result = branch.recv::<Approved>().await?.to_vec();
                endpoint
                    .send::<Reviewed<DEVELOPER>>(&result.as_slice())
                    .await?;
                loop {
                    let tested = endpoint.offer().await?;
                    match tested.label() {
                        <Passed as g::Message>::LOGICAL_LABEL => {
                            let result = tested.recv::<Passed>().await?.to_vec();
                            endpoint.send::<TestReport>(&result.as_slice()).await?;
                            return Ok(());
                        }
                        <Failed as g::Message>::LOGICAL_LABEL => {
                            let result = tested.recv::<Failed>().await?.to_vec();
                            endpoint.send::<TestReport>(&result.as_slice()).await?;
                            let failed = decode(&result)?;
                            let mut prior = failed["data"]["candidate"].clone();
                            loop {
                                let mut returned = endpoint.recv::<Loan<SERVICE>>().await?;
                                let received = returned.recv::<Implement<DEVELOPER>>().await?;
                                let repair = effects.apply(received).await?;
                                checked_build(&repair, agent, service, &plan)?;
                                if repair["data"]["receipt"] == prior {
                                    return Err(ShellError::message(
                                        "repair reused the failed candidate",
                                    ));
                                }
                                prior = repair["data"]["receipt"].clone();
                                let repaired = encode(&repair)?;
                                returned.send::<Implemented>(&repaired.as_slice()).await?;
                                endpoint.send::<Loan<SERVICE>>(returned).await?;
                                endpoint
                                    .send::<Candidate<SERVICE>>(&repaired.as_slice())
                                    .await?;
                                let decision = endpoint.offer().await?;
                                match decision.label() {
                                    <Approved as g::Message>::LOGICAL_LABEL => {
                                        let result = decision.recv::<Approved>().await?.to_vec();
                                        endpoint
                                            .send::<Reviewed<DEVELOPER>>(&result.as_slice())
                                            .await?;
                                        break;
                                    }
                                    <Rejected as g::Message>::LOGICAL_LABEL => {
                                        let result = decision.recv::<Rejected>().await?.to_vec();
                                        endpoint
                                            .send::<Reviewed<DEVELOPER>>(&result.as_slice())
                                            .await?;
                                    }
                                    _ => {
                                        return Err(ShellError::message(
                                            "unknown service review outcome",
                                        ));
                                    }
                                }
                            }
                        }
                        _ => return Err(ShellError::message("unknown service test outcome")),
                    }
                }
            }
            <Rejected as g::Message>::LOGICAL_LABEL => {
                let result = branch.recv::<Rejected>().await?.to_vec();
                endpoint
                    .send::<Reviewed<DEVELOPER>>(&result.as_slice())
                    .await?;
                candidate = endpoint.recv::<Loan<SERVICE>>().await?;
            }
            _ => return Err(ShellError::message("unknown service review outcome")),
        }
    }
}

fn checked_review(result: &Value, pr: &Value, candidate: &Value) -> ShellResult<()> {
    let data = &result["data"];
    if result["tool_name"] != "code_review"
        || data["agent"] != "REVIEWER"
        || data["pr_id"] != *pr
        || data["service"] != *pr
        || data["candidate"] != candidate["data"]["receipt"]
        || data["plan"] != candidate["data"]["plan"]
        || data["receipt"].as_u64().is_none_or(|receipt| receipt == 0)
    {
        return Err(ShellError::message(
            "review result differs from its admitted candidate",
        ));
    }
    Ok(())
}

pub(super) async fn reviewer<const REVIEWER: u8, const DEVELOPER: u8, const SERVICE: u8>(
    endpoint: &mut Endpoint<'_, REVIEWER>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    loop {
        let mut service = endpoint.recv::<Loan<SERVICE>>().await?;
        let bytes = endpoint.recv::<Candidate<SERVICE>>().await?.to_vec();
        let candidate = decode(&bytes)?;
        service
            .send::<Candidate<SERVICE>>(&bytes.as_slice())
            .await?;
        let received = service.recv::<Review<SERVICE>>().await?;
        let arguments = decode(received.as_bytes())?;
        let pr = arguments["pr_id"].clone();
        same(&pr, &candidate["data"]["service"])?;
        let result = effects.apply(received).await?;
        checked_review(&result, &pr, &candidate)?;
        let result_bytes = encode(&result)?;
        match (
            result["success"].as_bool(),
            result["data"]["status"].as_str(),
        ) {
            (Some(true), Some("reviewed")) => {
                let approval = ReviewReply::<true>::from_bytes(&result_bytes)?;
                service.send::<Approved>(&approval).await?;
                service
                    .send::<Reviewed<DEVELOPER>>(&result_bytes.as_slice())
                    .await?;
                service.send::<TestCandidate<SERVICE>>(&approval).await?;
                endpoint.send::<Loan<SERVICE>>(service).await?;
                break;
            }
            (Some(false), Some("changes_requested")) => {
                service
                    .send::<Rejected>(&ReviewReply::<false>::from_bytes(result_bytes.as_slice())?)
                    .await?;
                service
                    .send::<Reviewed<DEVELOPER>>(&result_bytes.as_slice())
                    .await?;
                endpoint.send::<Loan<SERVICE>>(service).await?;
            }
            _ => {
                return Err(ShellError::message(
                    "unknown result from the migration reviewer",
                ));
            }
        }
    }
    loop {
        let outcome = endpoint.offer().await?;
        match outcome.label() {
            <Passed as g::Message>::LOGICAL_LABEL => {
                let tested = outcome.recv::<Passed>().await?.to_vec();
                endpoint
                    .send::<Candidate<SERVICE>>(&tested.as_slice())
                    .await?;
                return Ok(());
            }
            <Loan<SERVICE> as g::Message>::LOGICAL_LABEL => {
                let mut service = outcome.recv::<Loan<SERVICE>>().await?;
                loop {
                    let bytes = endpoint.recv::<Candidate<SERVICE>>().await?.to_vec();
                    let candidate = decode(&bytes)?;
                    service
                        .send::<Candidate<SERVICE>>(&bytes.as_slice())
                        .await?;
                    let received = service.recv::<Review<SERVICE>>().await?;
                    let arguments = decode(received.as_bytes())?;
                    same(&arguments["pr_id"], &candidate["data"]["service"])?;
                    let result = effects.apply(received).await?;
                    checked_review(&result, &arguments["pr_id"], &candidate)?;
                    let bytes = encode(&result)?;
                    match (
                        result["success"].as_bool(),
                        result["data"]["status"].as_str(),
                    ) {
                        (Some(true), Some("reviewed")) => {
                            let approval = ReviewReply::<true>::from_bytes(&bytes)?;
                            endpoint.send::<Approved>(&approval).await?;
                            service
                                .send::<Reviewed<DEVELOPER>>(&bytes.as_slice())
                                .await?;

                            service.send::<TestCandidate<SERVICE>>(&approval).await?;
                            endpoint.send::<Loan<SERVICE>>(service).await?;
                            break;
                        }
                        (Some(false), Some("changes_requested")) => {
                            endpoint
                                .send::<Rejected>(&ReviewReply::<false>::from_bytes(&bytes)?)
                                .await?;
                            service
                                .send::<Reviewed<DEVELOPER>>(&bytes.as_slice())
                                .await?;
                            endpoint.send::<Loan<SERVICE>>(service).await?;
                            service = endpoint.recv::<Loan<SERVICE>>().await?;
                        }
                        _ => {
                            return Err(ShellError::message(
                                "unknown result from the migration reviewer",
                            ));
                        }
                    }
                }
            }
            _ => {
                return Err(ShellError::message(
                    "unexpected migration review continuation",
                ));
            }
        }
    }
}

pub(super) async fn tester<'run, const TESTER: u8, const SERVICE: u8>(
    endpoint: &mut Endpoint<'run, TESTER>,
    effects: &impl ReceivedEffects,
) -> ShellResult<Value> {
    loop {
        let approval = endpoint.recv::<TestCandidate<SERVICE>>().await?.to_vec();
        let mut service = endpoint.recv::<Loan<SERVICE>>().await?;
        service
            .send::<TestCandidate<SERVICE>>(&ReviewReply::from_bytes(&approval)?)
            .await?;
        let reviewed = decode(&approval)?;
        let received = endpoint.recv::<EnvironmentRequest<TESTER>>().await?;
        let arguments = decode(received.as_bytes())?;
        same(&arguments["suite"], &reviewed["data"]["service"])?;
        let run_tests = effects.apply(received);
        let environment = endpoint.recv::<Loan<TEST_ENV>>().await?;
        let result = run_tests.await?;
        let data = &result["data"];
        if result["tool_name"] != "run_tests"
            || data["agent"] != "TESTER"
            || data["suite"] != reviewed["data"]["service"]
            || data["service"] != data["suite"]
            || data["candidate"] != reviewed["data"]["candidate"]
            || data["review"] != reviewed["data"]["receipt"]
            || data["plan"] != reviewed["data"]["plan"]
            || data["receipt"].as_u64().is_none_or(|id| id == 0)
            || !matches!(
                (result["success"].as_bool(), data["status"].as_str()),
                (Some(true), Some("tests passed")) | (Some(false), Some("tests_failed"))
            )
        {
            return Err(ShellError::message(
                "test result differs from its approved service",
            ));
        }
        let bytes = encode(&result)?;
        if result["success"] == true {
            service
                .send::<Passed>(&ReviewReply::from_bytes(&bytes)?)
                .await?;
            service.send::<TestReport>(&bytes.as_slice()).await?;
            endpoint.send::<Loan<TEST_ENV>>(environment).await?;
            service
                .send::<Passed>(&ReviewReply::from_bytes(&bytes)?)
                .await?;
            endpoint
                .send::<StagingCandidate<TESTER>>(&ReviewReply::from_bytes(&bytes)?)
                .await?;
            endpoint.send::<Loan<SERVICE>>(service).await?;
            return Ok(result);
        } else {
            service
                .send::<Failed>(&ReviewReply::from_bytes(&bytes)?)
                .await?;
            service.send::<TestReport>(&bytes.as_slice()).await?;
            endpoint.send::<Loan<TEST_ENV>>(environment).await?;
            endpoint.send::<Loan<SERVICE>>(service).await?;
        }
    }
}

pub(super) async fn test_environment<'run>(
    owner: &mut Endpoint<'run, ENV_OWNER>,
    mut environment: Endpoint<'run, TEST_ENV>,
) -> ShellResult<Endpoint<'run, TEST_ENV>> {
    loop {
        let request = owner.offer().await?;
        let suite = request.label();
        match suite {
            <Exit as g::Message>::LOGICAL_LABEL => {
                let exit = request.recv::<Exit>().await?;
                if exit != 0 {
                    return Err(ShellError::message("testing shell exited unsuccessfully"));
                }
                break;
            }
            TESTER_A => {
                let arguments = request.recv::<RunTests<TESTER_A>>().await?;
                environment
                    .send::<EnvironmentRequest<TESTER_A>>(&TestRequest(arguments))
                    .await?;
            }
            TESTER_B => {
                let arguments = request.recv::<RunTests<TESTER_B>>().await?;
                environment
                    .send::<EnvironmentRequest<TESTER_B>>(&TestRequest(arguments))
                    .await?;
            }
            TESTER_C => {
                let arguments = request.recv::<RunTests<TESTER_C>>().await?;
                environment
                    .send::<EnvironmentRequest<TESTER_C>>(&TestRequest(arguments))
                    .await?;
            }
            _ => return Err(ShellError::message("unknown migration test suite")),
        }
        owner.send::<Loan<TEST_ENV>>(environment).await?;
        let result = owner.recv::<TestReport>().await?.to_vec();
        environment = owner.recv::<Loan<TEST_ENV>>().await?;
        match suite {
            TESTER_A => owner.send::<Tested<TESTER_A>>(&result.as_slice()).await?,
            TESTER_B => owner.send::<Tested<TESTER_B>>(&result.as_slice()).await?,
            TESTER_C => owner.send::<Tested<TESTER_C>>(&result.as_slice()).await?,
            _ => unreachable!(),
        }
    }
    Ok(environment)
}

#[cfg(test)]
#[path = "preparation_tests.rs"]
mod tests;
