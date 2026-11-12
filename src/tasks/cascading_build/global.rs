//! One source for all initial builder orders and unbounded data-driven repairs.
use crate::shell::imp::review_reply::ReviewReply;
use hibana::{
    Endpoint, g,
    runtime::program::{Projectable, RoleProgram, project},
};

pub const A: u8 = 0;
pub const B: u8 = 1;
pub const C: u8 = 2;
pub const INTEGRATOR: u8 = 3;
pub const TESTER: u8 = 4;
pub const COMPILER: u8 = 5;
pub const TEST_SERVICE: u8 = 6;
pub const CONFIG: u8 = 7;
pub const CORE: u8 = 8;
pub const MODELS: u8 = 9;
pub const A_IO: u8 = 10;
pub const B_IO: u8 = 11;
pub const C_IO: u8 = 12;
pub const INTEGRATOR_IO: u8 = 13;
pub const TESTER_IO: u8 = 14;
pub const SESSION: u32 = 102;
pub const FIRST_A: u16 = 910;
pub const FIRST_B: u16 = 911;
pub const SECOND_A: u16 = 912;
pub const SECOND_B: u16 = 913;
pub const SECOND_C: u16 = 914;
pub const REBUILD_A: u16 = 915;
pub const REBUILD_B: u16 = 916;

pub type LoanCoreA<'r> = g::Msg<1, Endpoint<'r, CORE>>;
pub type LoanConfigA<'r> = g::Msg<2, Endpoint<'r, CONFIG>>;
pub type LoanCoreB<'r> = g::Msg<3, Endpoint<'r, CORE>>;
pub type LoanModelsB<'r> = g::Msg<4, Endpoint<'r, MODELS>>;
pub type LoanConfigC<'r> = g::Msg<5, Endpoint<'r, CONFIG>>;
pub type LoanModelsC<'r> = g::Msg<6, Endpoint<'r, MODELS>>;
pub type ReturnCore<'r> = g::Msg<7, Endpoint<'r, CORE>>;
pub type ReturnConfig<'r> = g::Msg<8, Endpoint<'r, CONFIG>>;
pub type ReturnModels<'r> = g::Msg<9, Endpoint<'r, MODELS>>;
pub type CompileA<'a> = g::Msg<10, &'a [u8]>;
pub type CompileB<'a> = g::Msg<11, &'a [u8]>;
pub type CompileC<'a> = g::Msg<12, &'a [u8]>;
pub type CompileCore<'a> = g::Msg<13, &'a [u8]>;
pub type CompileConfig<'a> = g::Msg<14, &'a [u8]>;
pub type CompileModels<'a> = g::Msg<15, &'a [u8]>;
pub type CoreBuilt<'a> = g::Msg<16, &'a [u8]>;
pub type ConfigBuilt<'a> = g::Msg<17, &'a [u8]>;
pub type ModelsBuilt<'a> = g::Msg<18, &'a [u8]>;
pub type RepairCandidate<'a> = g::Msg<19, &'a [u8]>;
pub type ReadyCandidate<'a> = g::Msg<20, ReviewReply<'a, true>>;
pub type TestObjectA<'a> = g::Msg<22, &'a [u8]>;
pub type TestObjectB<'a> = g::Msg<23, &'a [u8]>;
pub type TestObjectC<'a> = g::Msg<24, &'a [u8]>;
pub type UpdateA<'a> = g::Msg<25, &'a [u8]>;
pub type UpdateB<'a> = g::Msg<26, &'a [u8]>;
pub type UpdateC<'a> = g::Msg<27, &'a [u8]>;
pub type TestA<'a> = g::Msg<28, &'a [u8]>;
pub type TestB<'a> = g::Msg<29, &'a [u8]>;
pub type TestC<'a> = g::Msg<30, &'a [u8]>;
pub type Tested<'a> = g::Msg<31, &'a [u8]>;
pub type Feedback<'a> = g::Msg<32, &'a [u8]>;
pub type AllForCompiler<'a> = g::Msg<34, &'a [u8]>;
pub type RetryA<'a> = g::Msg<35, &'a [u8]>;
pub type RetryB<'a> = g::Msg<36, &'a [u8]>;
pub type RetryC<'a> = g::Msg<37, &'a [u8]>;
pub type RetryAForCompiler<'a> = g::Msg<38, &'a [u8]>;
pub type RetryBForCompiler<'a> = g::Msg<39, &'a [u8]>;
pub type RetryCForCompiler<'a> = g::Msg<40, &'a [u8]>;
pub type FinishChecks<'a> = g::Msg<41, ReviewReply<'a, true>>;
pub type FrozenCore<'r> = g::Msg<42, Endpoint<'r, CORE>>;
pub type FrozenConfig<'r> = g::Msg<43, Endpoint<'r, CONFIG>>;
pub type FrozenModels<'r> = g::Msg<44, Endpoint<'r, MODELS>>;
pub type Link<'a> = g::Msg<45, &'a [u8]>;
pub type Linked<'a> = g::Msg<46, &'a [u8]>;
pub type FinishedA<'a> = g::Msg<47, &'a [u8]>;
pub type FinishedB<'a> = g::Msg<48, &'a [u8]>;
pub type FinishedC<'a> = g::Msg<49, &'a [u8]>;
pub type FinishedTester<'a> = g::Msg<50, &'a [u8]>;
pub type FinishedService<'a> = g::Msg<51, &'a [u8]>;
pub type RetryAToB<'a> = g::Msg<52, &'a [u8]>;
pub type RetryAToC<'a> = g::Msg<53, &'a [u8]>;
pub type RetryBToA<'a> = g::Msg<54, &'a [u8]>;
pub type RetryBToC<'a> = g::Msg<55, &'a [u8]>;
pub type RetryCToA<'a> = g::Msg<56, &'a [u8]>;
pub type RetryCToB<'a> = g::Msg<57, &'a [u8]>;

fn compile_a() -> impl Projectable {
    g::seq(
        g::par(
            g::send::<COMPILER, A, LoanCoreA>(),
            g::send::<COMPILER, A, LoanConfigA>(),
        ),
        g::seq(
            g::send::<A, COMPILER, CompileA>(),
            g::seq(
                g::par(
                    g::send::<CORE, COMPILER, CompileCore>(),
                    g::send::<CONFIG, COMPILER, CompileConfig>(),
                ),
                g::seq(
                    g::par(
                        g::send::<COMPILER, CORE, CoreBuilt>(),
                        g::send::<COMPILER, CONFIG, ConfigBuilt>(),
                    ),
                    g::seq(
                        g::par(
                            g::send::<A, COMPILER, ReturnCore>(),
                            g::send::<A, COMPILER, ReturnConfig>(),
                        ),
                        g::send::<COMPILER, TESTER, TestObjectA>(),
                    ),
                ),
            ),
        ),
    )
}
fn compile_b() -> impl Projectable {
    g::seq(
        g::par(
            g::send::<COMPILER, B, LoanCoreB>(),
            g::send::<COMPILER, B, LoanModelsB>(),
        ),
        g::seq(
            g::send::<B, COMPILER, CompileB>(),
            g::seq(
                g::par(
                    g::send::<CORE, COMPILER, CompileCore>(),
                    g::send::<MODELS, COMPILER, CompileModels>(),
                ),
                g::seq(
                    g::par(
                        g::send::<COMPILER, CORE, CoreBuilt>(),
                        g::send::<COMPILER, MODELS, ModelsBuilt>(),
                    ),
                    g::seq(
                        g::par(
                            g::send::<B, COMPILER, ReturnCore>(),
                            g::send::<B, COMPILER, ReturnModels>(),
                        ),
                        g::send::<COMPILER, TESTER, TestObjectB>(),
                    ),
                ),
            ),
        ),
    )
}
fn compile_c() -> impl Projectable {
    g::seq(
        g::par(
            g::send::<COMPILER, C, LoanConfigC>(),
            g::send::<COMPILER, C, LoanModelsC>(),
        ),
        g::seq(
            g::send::<C, COMPILER, CompileC>(),
            g::seq(
                g::par(
                    g::send::<CONFIG, COMPILER, CompileConfig>(),
                    g::send::<MODELS, COMPILER, CompileModels>(),
                ),
                g::seq(
                    g::par(
                        g::send::<COMPILER, CONFIG, ConfigBuilt>(),
                        g::send::<COMPILER, MODELS, ModelsBuilt>(),
                    ),
                    g::seq(
                        g::par(
                            g::send::<C, COMPILER, ReturnConfig>(),
                            g::send::<C, COMPILER, ReturnModels>(),
                        ),
                        g::send::<COMPILER, TESTER, TestObjectC>(),
                    ),
                ),
            ),
        ),
    )
}
fn initial_a() -> impl Projectable {
    g::seq(
        compile_a(),
        g::par(
            g::send::<COMPILER, B, UpdateB>(),
            g::send::<COMPILER, C, UpdateC>(),
        ),
    )
}
fn initial_b() -> impl Projectable {
    g::seq(
        compile_b(),
        g::par(
            g::send::<COMPILER, A, UpdateA>(),
            g::send::<COMPILER, C, UpdateC>(),
        ),
    )
}
fn initial_c() -> impl Projectable {
    g::seq(
        compile_c(),
        g::par(
            g::send::<COMPILER, A, UpdateA>(),
            g::send::<COMPILER, B, UpdateB>(),
        ),
    )
}
fn test_a() -> impl Projectable {
    g::seq(
        g::send::<TESTER, TEST_SERVICE, TestA>(),
        g::par(
            g::send::<TEST_SERVICE, TESTER, Tested>(),
            g::send::<TEST_SERVICE, A, Feedback>(),
        ),
    )
}
fn test_b() -> impl Projectable {
    g::seq(
        g::send::<TESTER, TEST_SERVICE, TestB>(),
        g::par(
            g::send::<TEST_SERVICE, TESTER, Tested>(),
            g::send::<TEST_SERVICE, B, Feedback>(),
        ),
    )
}
fn test_c() -> impl Projectable {
    g::seq(
        g::send::<TESTER, TEST_SERVICE, TestC>(),
        g::par(
            g::send::<TEST_SERVICE, TESTER, Tested>(),
            g::send::<TEST_SERVICE, C, Feedback>(),
        ),
    )
}
// All builds contend on an actual shared file. Tests use a distinct worker;
// the source overlaps each completed-object test with the next compilation,
// while requiring one test to finish before another starts.
fn pipeline(
    (x, tx): (impl Projectable, impl Projectable),
    (y, ty): (impl Projectable, impl Projectable),
    (z, tz): (impl Projectable, impl Projectable),
) -> impl Projectable {
    g::seq(x, g::seq(g::par(tx, y), g::seq(g::par(ty, z), tz)))
}
fn all_results() -> impl Projectable {
    g::send::<TEST_SERVICE, COMPILER, AllForCompiler>()
}
fn repair() -> impl Projectable {
    g::seq(
        g::send::<COMPILER, INTEGRATOR, RepairCandidate>(),
        g::seq(
            g::route(
                g::seq(
                    g::par(
                        g::send::<INTEGRATOR, A, RetryA>(),
                        g::par(
                            g::send::<INTEGRATOR, COMPILER, RetryAForCompiler>(),
                            g::par(
                                g::send::<INTEGRATOR, B, RetryAToB>(),
                                g::send::<INTEGRATOR, C, RetryAToC>(),
                            ),
                        ),
                    ),
                    g::seq(compile_a(), test_a()),
                ),
                g::route(
                    g::seq(
                        g::par(
                            g::send::<INTEGRATOR, B, RetryB>(),
                            g::par(
                                g::send::<INTEGRATOR, COMPILER, RetryBForCompiler>(),
                                g::par(
                                    g::send::<INTEGRATOR, A, RetryBToA>(),
                                    g::send::<INTEGRATOR, C, RetryBToC>(),
                                ),
                            ),
                        ),
                        g::seq(compile_b(), test_b()),
                    ),
                    g::seq(
                        g::par(
                            g::send::<INTEGRATOR, C, RetryC>(),
                            g::par(
                                g::send::<INTEGRATOR, COMPILER, RetryCForCompiler>(),
                                g::par(
                                    g::send::<INTEGRATOR, A, RetryCToA>(),
                                    g::send::<INTEGRATOR, B, RetryCToB>(),
                                ),
                            ),
                        ),
                        g::seq(compile_c(), test_c()),
                    ),
                )
                .resolve::<REBUILD_B>(),
            )
            .resolve::<REBUILD_A>(),
            all_results(),
        ),
    )
}
fn finish() -> impl Projectable {
    g::seq(
        g::send::<COMPILER, INTEGRATOR, ReadyCandidate>(),
        g::seq(
            g::send::<INTEGRATOR, COMPILER, FinishChecks>(),
            g::seq(
                // The real editing authorities leave the old repair continuation
                // before publication. A later scalar notification is not a fence.
                g::par(
                    g::send::<COMPILER, INTEGRATOR, FrozenCore>(),
                    g::par(
                        g::send::<COMPILER, INTEGRATOR, FrozenConfig>(),
                        g::send::<COMPILER, INTEGRATOR, FrozenModels>(),
                    ),
                ),
                g::seq(
                    g::send::<INTEGRATOR, COMPILER, Link>(),
                    g::seq(
                        g::send::<COMPILER, INTEGRATOR, Linked>(),
                        g::par(
                            g::send::<INTEGRATOR, A, FinishedA>(),
                            g::par(
                                g::send::<INTEGRATOR, B, FinishedB>(),
                                g::par(
                                    g::send::<INTEGRATOR, C, FinishedC>(),
                                    g::par(
                                        g::send::<INTEGRATOR, TESTER, FinishedTester>(),
                                        g::send::<INTEGRATOR, TEST_SERVICE, FinishedService>(),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    )
}
pub fn business() -> impl Projectable {
    let initial = g::route(
        g::route(
            pipeline(
                (initial_a(), test_a()),
                (initial_b(), test_b()),
                (initial_c(), test_c()),
            ),
            pipeline(
                (initial_a(), test_a()),
                (initial_c(), test_c()),
                (initial_b(), test_b()),
            ),
        )
        .resolve::<SECOND_A>(),
        g::route(
            g::route(
                pipeline(
                    (initial_b(), test_b()),
                    (initial_a(), test_a()),
                    (initial_c(), test_c()),
                ),
                pipeline(
                    (initial_b(), test_b()),
                    (initial_c(), test_c()),
                    (initial_a(), test_a()),
                ),
            )
            .resolve::<SECOND_B>(),
            g::route(
                pipeline(
                    (initial_c(), test_c()),
                    (initial_a(), test_a()),
                    (initial_b(), test_b()),
                ),
                pipeline(
                    (initial_c(), test_c()),
                    (initial_b(), test_b()),
                    (initial_a(), test_a()),
                ),
            )
            .resolve::<SECOND_C>(),
        )
        .resolve::<FIRST_B>(),
    )
    .resolve::<FIRST_A>();
    g::seq(
        initial,
        g::seq(
            all_results(),
            g::route(g::seq(repair().roll(), finish()), finish()),
        ),
    )
}
pub fn source() -> impl Projectable {
    g::par(
        business(),
        g::par(
            crate::shell::global::private_io::<A, A_IO>(),
            g::par(
                crate::shell::global::private_io::<B, B_IO>(),
                g::par(
                    crate::shell::global::private_io::<C, C_IO>(),
                    g::par(
                        crate::shell::global::private_io::<INTEGRATOR, INTEGRATOR_IO>(),
                        crate::shell::global::private_io::<TESTER, TESTER_IO>(),
                    ),
                ),
            ),
        ),
    )
}

pub struct Programs {
    pub a: RoleProgram<A>,
    pub b: RoleProgram<B>,
    pub c: RoleProgram<C>,
    pub integrator: RoleProgram<INTEGRATOR>,
    pub tester: RoleProgram<TESTER>,
    pub compiler: RoleProgram<COMPILER>,
    pub testing: RoleProgram<TEST_SERVICE>,
    pub config: RoleProgram<CONFIG>,
    pub core: RoleProgram<CORE>,
    pub models: RoleProgram<MODELS>,
    pub a_io: RoleProgram<A_IO>,
    pub b_io: RoleProgram<B_IO>,
    pub c_io: RoleProgram<C_IO>,
    pub integrator_io: RoleProgram<INTEGRATOR_IO>,
    pub tester_io: RoleProgram<TESTER_IO>,
}
impl Programs {
    pub fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        use hibana::g::Message;
        let labels: serde_json::Map<String, serde_json::Value> = [
            ("LoanCoreA", LoanCoreA::LOGICAL_LABEL),
            ("LoanConfigA", LoanConfigA::LOGICAL_LABEL),
            ("LoanCoreB", LoanCoreB::LOGICAL_LABEL),
            ("LoanModelsB", LoanModelsB::LOGICAL_LABEL),
            ("LoanConfigC", LoanConfigC::LOGICAL_LABEL),
            ("LoanModelsC", LoanModelsC::LOGICAL_LABEL),
            ("ReturnCore", ReturnCore::LOGICAL_LABEL),
            ("ReturnConfig", ReturnConfig::LOGICAL_LABEL),
            ("ReturnModels", ReturnModels::LOGICAL_LABEL),
            ("CompileA", CompileA::LOGICAL_LABEL),
            ("CompileB", CompileB::LOGICAL_LABEL),
            ("CompileC", CompileC::LOGICAL_LABEL),
            ("CompileCore", CompileCore::LOGICAL_LABEL),
            ("CompileConfig", CompileConfig::LOGICAL_LABEL),
            ("CompileModels", CompileModels::LOGICAL_LABEL),
            ("CoreBuilt", CoreBuilt::LOGICAL_LABEL),
            ("ConfigBuilt", ConfigBuilt::LOGICAL_LABEL),
            ("ModelsBuilt", ModelsBuilt::LOGICAL_LABEL),
            ("RepairCandidate", RepairCandidate::LOGICAL_LABEL),
            ("ReadyCandidate", ReadyCandidate::LOGICAL_LABEL),
            ("TestObjectA", TestObjectA::LOGICAL_LABEL),
            ("TestObjectB", TestObjectB::LOGICAL_LABEL),
            ("TestObjectC", TestObjectC::LOGICAL_LABEL),
            ("UpdateA", UpdateA::LOGICAL_LABEL),
            ("UpdateB", UpdateB::LOGICAL_LABEL),
            ("UpdateC", UpdateC::LOGICAL_LABEL),
            ("TestA", TestA::LOGICAL_LABEL),
            ("TestB", TestB::LOGICAL_LABEL),
            ("TestC", TestC::LOGICAL_LABEL),
            ("Tested", Tested::LOGICAL_LABEL),
            ("Feedback", Feedback::LOGICAL_LABEL),
            ("AllForCompiler", AllForCompiler::LOGICAL_LABEL),
            ("RetryA", RetryA::LOGICAL_LABEL),
            ("RetryB", RetryB::LOGICAL_LABEL),
            ("RetryC", RetryC::LOGICAL_LABEL),
            ("RetryAForCompiler", RetryAForCompiler::LOGICAL_LABEL),
            ("RetryBForCompiler", RetryBForCompiler::LOGICAL_LABEL),
            ("RetryCForCompiler", RetryCForCompiler::LOGICAL_LABEL),
            ("FinishChecks", FinishChecks::LOGICAL_LABEL),
            ("FrozenCore", FrozenCore::LOGICAL_LABEL),
            ("FrozenConfig", FrozenConfig::LOGICAL_LABEL),
            ("FrozenModels", FrozenModels::LOGICAL_LABEL),
            ("Link", Link::LOGICAL_LABEL),
            ("Linked", Linked::LOGICAL_LABEL),
            ("FinishedA", FinishedA::LOGICAL_LABEL),
            ("FinishedB", FinishedB::LOGICAL_LABEL),
            ("FinishedC", FinishedC::LOGICAL_LABEL),
            ("FinishedTester", FinishedTester::LOGICAL_LABEL),
            ("FinishedService", FinishedService::LOGICAL_LABEL),
            ("RetryAToB", RetryAToB::LOGICAL_LABEL),
            ("RetryAToC", RetryAToC::LOGICAL_LABEL),
            ("RetryBToA", RetryBToA::LOGICAL_LABEL),
            ("RetryBToC", RetryBToC::LOGICAL_LABEL),
            ("RetryCToA", RetryCToA::LOGICAL_LABEL),
            ("RetryCToB", RetryCToB::LOGICAL_LABEL),
        ]
        .into_iter()
        .map(|(name, label)| (name.into(), label.into()))
        .collect();
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"10M","session":SESSION,"labels":labels,
            "roles":{"BUILDER_A":A,"BUILDER_B":B,"BUILDER_C":C,"INTEGRATOR":INTEGRATOR,"TEST_RUNNER":TESTER,
                "COMPILER":COMPILER,"TEST_SERVICE":TEST_SERVICE,"CONFIG":CONFIG,"CORE_LIB":CORE,"DATA_MODELS":MODELS,
                "A_IO":A_IO,"B_IO":B_IO,"C_IO":C_IO,"INTEGRATOR_IO":INTEGRATOR_IO,"TESTER_IO":TESTER_IO},
            "programs":[rows(&self.a),rows(&self.b),rows(&self.c),rows(&self.integrator),rows(&self.tester),
                rows(&self.compiler),rows(&self.testing),rows(&self.config),rows(&self.core),rows(&self.models),
                rows(&self.a_io),rows(&self.b_io),rows(&self.c_io),rows(&self.integrator_io),rows(&self.tester_io)]})
    }
}
pub fn programs() -> Programs {
    let source = source();
    Programs {
        a: project::<A>(&source),
        b: project::<B>(&source),
        c: project::<C>(&source),
        integrator: project::<INTEGRATOR>(&source),
        tester: project::<TESTER>(&source),
        compiler: project::<COMPILER>(&source),
        testing: project::<TEST_SERVICE>(&source),
        config: project::<CONFIG>(&source),
        core: project::<CORE>(&source),
        models: project::<MODELS>(&source),
        a_io: project::<A_IO>(&source),
        b_io: project::<B_IO>(&source),
        c_io: project::<C_IO>(&source),
        integrator_io: project::<INTEGRATOR_IO>(&source),
        tester_io: project::<TESTER_IO>(&source),
    }
}
