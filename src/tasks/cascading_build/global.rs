//! One source for all initial builder orders and unbounded data-driven repairs.
use super::wire::{Arguments, BuildRequest};
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
pub const A_FILE: u8 = 16;
pub const A_VM: u8 = 32;
pub const A_DATA: u8 = 48;
pub const B_FILE: u8 = 17;
pub const B_VM: u8 = 33;
pub const B_DATA: u8 = 49;
pub const C_FILE: u8 = 18;
pub const C_VM: u8 = 34;
pub const C_DATA: u8 = 50;
pub const INTEGRATOR_FILE: u8 = 19;
pub const INTEGRATOR_VM: u8 = 35;
pub const INTEGRATOR_DATA: u8 = 51;
pub const TESTER_FILE: u8 = 20;
pub const TESTER_VM: u8 = 36;
pub const TESTER_DATA: u8 = 52;
pub use crate::shell::global::{FileExit, Loan};
pub const SESSION: u32 = 102;
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
pub type CompileCore<'a, const ROLE: u8> = g::Msg<13, BuildRequest<'a, ROLE>>;
pub type CompileConfig<'a, const ROLE: u8> = g::Msg<14, BuildRequest<'a, ROLE>>;
pub type CompileModels<'a, const ROLE: u8> = g::Msg<15, BuildRequest<'a, ROLE>>;
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
pub type TestA<'a> = g::Msg<28, BuildRequest<'a, TESTER>>;
pub type TestB<'a> = g::Msg<29, BuildRequest<'a, TESTER>>;
pub type TestC<'a> = g::Msg<30, BuildRequest<'a, TESTER>>;
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
pub type Link<'a> = g::Msg<45, BuildRequest<'a, INTEGRATOR>>;
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

pub type CompileFileA = g::Msg<60, Arguments<A>>;
pub type CompileFileB = g::Msg<61, Arguments<B>>;
pub type CompileFileC = g::Msg<62, Arguments<C>>;
pub type FileCompiled<'a> = g::Msg<63, &'a [u8]>;
pub type FileUpdated<'a> = g::Msg<64, &'a [u8]>;
pub type ObjectForTesting<'a> = g::Msg<65, &'a [u8]>;
pub type TestFileA = g::Msg<66, Arguments<A>>;
pub type TestFileB = g::Msg<67, Arguments<B>>;
pub type TestFileC = g::Msg<68, Arguments<C>>;
pub type FileTested<'a> = g::Msg<69, &'a [u8]>;
pub type LinkFile = g::Msg<70, Arguments<INTEGRATOR>>;
pub type FileLinked<'a> = g::Msg<71, &'a [u8]>;
pub type FileFeedback<'a> = g::Msg<72, &'a [u8]>;
pub type FileReconsidered<'a> = g::Msg<73, &'a [u8]>;
pub type FileFinished<'a> = g::Msg<74, &'a [u8]>;
pub type FileReady<'a> = g::Msg<75, ReviewReply<'a, true>>;
const fn object(
    path: &'static [u8],
    fd: u8,
) -> hibana_wasip1_runtime::choreofs::ChoreoFsObject<'static> {
    use hibana_wasip1_runtime::{
        choreofs::{ChoreoFsObject, FdSpec, ObjectId},
        protocol,
    };
    ChoreoFsObject::read_write(
        path,
        ObjectId(fd as u32),
        FdSpec::new(fd, 66),
        b"",
        protocol::FdBinding {
            read: Some(protocol::FdReadRow::Base),
            write: Some(protocol::FdWriteRow::Object),
            readdir: None,
        },
    )
}
pub const BUILDER_FILES: [hibana_wasip1_runtime::choreofs::ChoreoFsObject<'static>; 5] = [
    object(b"service/compile_module", 4),
    object(b"service/updates", 5),
    object(b"service/test_result", 6),
    object(b"service/reconsideration", 7),
    object(b"service/outcome", 8),
];
pub const TESTER_FILES: [hibana_wasip1_runtime::choreofs::ChoreoFsObject<'static>; 4] = [
    object(b"service/run_build_tests_a", 4),
    object(b"service/run_build_tests_b", 5),
    object(b"service/run_build_tests_c", 6),
    object(b"service/next_object", 7),
];
pub const INTEGRATOR_FILES: [hibana_wasip1_runtime::choreofs::ChoreoFsObject<'static>; 2] = [
    object(b"service/link_modules", 4),
    object(b"service/candidate", 5),
];

fn build_a() -> impl Projectable {
    g::seq(
        g::par(
            g::send::<COMPILER, A, LoanCoreA>(),
            g::send::<COMPILER, A, LoanConfigA>(),
        ),
        g::seq(
            g::send::<A, COMPILER, CompileA>(),
            g::seq(
                g::par(
                    g::send::<CORE, COMPILER, CompileCore<A>>(),
                    g::send::<CONFIG, COMPILER, CompileConfig<A>>(),
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
                        g::send::<A, A_FILE, FileCompiled>(),
                    ),
                ),
            ),
        ),
    )
}
fn build_b() -> impl Projectable {
    g::seq(
        g::par(
            g::send::<COMPILER, B, LoanCoreB>(),
            g::send::<COMPILER, B, LoanModelsB>(),
        ),
        g::seq(
            g::send::<B, COMPILER, CompileB>(),
            g::seq(
                g::par(
                    g::send::<CORE, COMPILER, CompileCore<B>>(),
                    g::send::<MODELS, COMPILER, CompileModels<B>>(),
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
                        g::send::<B, B_FILE, FileCompiled>(),
                    ),
                ),
            ),
        ),
    )
}
fn build_c() -> impl Projectable {
    g::seq(
        g::par(
            g::send::<COMPILER, C, LoanConfigC>(),
            g::send::<COMPILER, C, LoanModelsC>(),
        ),
        g::seq(
            g::send::<C, COMPILER, CompileC>(),
            g::seq(
                g::par(
                    g::send::<CONFIG, COMPILER, CompileConfig<C>>(),
                    g::send::<MODELS, COMPILER, CompileModels<C>>(),
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
                        g::send::<C, C_FILE, FileCompiled>(),
                    ),
                ),
            ),
        ),
    )
}
fn initial_a() -> impl Projectable {
    g::seq(
        build_a(),
        g::par(
            g::seq(
                g::send::<COMPILER, B, UpdateB>(),
                g::send::<B, B_FILE, FileUpdated>(),
            ),
            g::seq(
                g::send::<COMPILER, C, UpdateC>(),
                g::send::<C, C_FILE, FileUpdated>(),
            ),
        ),
    )
}
fn initial_b() -> impl Projectable {
    g::seq(
        build_b(),
        g::par(
            g::seq(
                g::send::<COMPILER, A, UpdateA>(),
                g::send::<A, A_FILE, FileUpdated>(),
            ),
            g::seq(
                g::send::<COMPILER, C, UpdateC>(),
                g::send::<C, C_FILE, FileUpdated>(),
            ),
        ),
    )
}
fn initial_c() -> impl Projectable {
    g::seq(
        build_c(),
        g::par(
            g::seq(
                g::send::<COMPILER, A, UpdateA>(),
                g::send::<A, A_FILE, FileUpdated>(),
            ),
            g::seq(
                g::send::<COMPILER, B, UpdateB>(),
                g::send::<B, B_FILE, FileUpdated>(),
            ),
        ),
    )
}
fn test_a() -> impl Projectable {
    g::seq(
        g::send::<TESTER, TESTER_FILE, ObjectForTesting>(),
        g::seq(
            g::send::<TESTER_FILE, TESTER, TestFileA>(),
            g::seq(
                g::send::<TESTER, TEST_SERVICE, TestA>(),
                g::par(
                    g::seq(
                        g::send::<TEST_SERVICE, TESTER, Tested>(),
                        g::send::<TESTER, TESTER_FILE, FileTested>(),
                    ),
                    g::seq(
                        g::send::<TEST_SERVICE, A, Feedback>(),
                        g::send::<A, A_FILE, FileFeedback>(),
                    ),
                ),
            ),
        ),
    )
}
fn test_b() -> impl Projectable {
    g::seq(
        g::send::<TESTER, TESTER_FILE, ObjectForTesting>(),
        g::seq(
            g::send::<TESTER_FILE, TESTER, TestFileB>(),
            g::seq(
                g::send::<TESTER, TEST_SERVICE, TestB>(),
                g::par(
                    g::seq(
                        g::send::<TEST_SERVICE, TESTER, Tested>(),
                        g::send::<TESTER, TESTER_FILE, FileTested>(),
                    ),
                    g::seq(
                        g::send::<TEST_SERVICE, B, Feedback>(),
                        g::send::<B, B_FILE, FileFeedback>(),
                    ),
                ),
            ),
        ),
    )
}
fn test_c() -> impl Projectable {
    g::seq(
        g::send::<TESTER, TESTER_FILE, ObjectForTesting>(),
        g::seq(
            g::send::<TESTER_FILE, TESTER, TestFileC>(),
            g::seq(
                g::send::<TESTER, TEST_SERVICE, TestC>(),
                g::par(
                    g::seq(
                        g::send::<TEST_SERVICE, TESTER, Tested>(),
                        g::send::<TESTER, TESTER_FILE, FileTested>(),
                    ),
                    g::seq(
                        g::send::<TEST_SERVICE, C, Feedback>(),
                        g::send::<C, C_FILE, FileFeedback>(),
                    ),
                ),
            ),
        ),
    )
}
// Object delivery gives the tester its choice knowledge before the fork.
// Testing the previous object overlaps the next build; tests remain sequential.
fn pipeline(
    (first_object, first_test): (impl Projectable, impl Projectable),
    (second_build, second_object, second_test): (
        impl Projectable,
        impl Projectable,
        impl Projectable,
    ),
    (third_build, third_object, third_test): (impl Projectable, impl Projectable, impl Projectable),
) -> impl Projectable {
    g::seq(
        first_object,
        g::seq(
            g::par(first_test, second_build),
            g::seq(
                second_object,
                g::seq(
                    g::par(second_test, third_build),
                    g::seq(third_object, third_test),
                ),
            ),
        ),
    )
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
                        g::seq(
                            g::send::<INTEGRATOR, A, RetryA>(),
                            g::send::<A, A_FILE, FileReconsidered>(),
                        ),
                        g::par(
                            g::send::<INTEGRATOR, COMPILER, RetryAForCompiler>(),
                            g::par(
                                g::seq(
                                    g::send::<INTEGRATOR, B, RetryAToB>(),
                                    g::send::<B, B_FILE, FileUpdated>(),
                                ),
                                g::seq(
                                    g::send::<INTEGRATOR, C, RetryAToC>(),
                                    g::send::<C, C_FILE, FileUpdated>(),
                                ),
                            ),
                        ),
                    ),
                    g::seq(
                        g::seq(g::send::<A_FILE, A, CompileFileA>(), build_a()),
                        g::seq(g::send::<COMPILER, TESTER, TestObjectA>(), test_a()),
                    ),
                ),
                g::route(
                    g::seq(
                        g::par(
                            g::seq(
                                g::send::<INTEGRATOR, B, RetryB>(),
                                g::send::<B, B_FILE, FileReconsidered>(),
                            ),
                            g::par(
                                g::send::<INTEGRATOR, COMPILER, RetryBForCompiler>(),
                                g::par(
                                    g::seq(
                                        g::send::<INTEGRATOR, A, RetryBToA>(),
                                        g::send::<A, A_FILE, FileUpdated>(),
                                    ),
                                    g::seq(
                                        g::send::<INTEGRATOR, C, RetryBToC>(),
                                        g::send::<C, C_FILE, FileUpdated>(),
                                    ),
                                ),
                            ),
                        ),
                        g::seq(
                            g::seq(g::send::<B_FILE, B, CompileFileB>(), build_b()),
                            g::seq(g::send::<COMPILER, TESTER, TestObjectB>(), test_b()),
                        ),
                    ),
                    g::seq(
                        g::par(
                            g::seq(
                                g::send::<INTEGRATOR, C, RetryC>(),
                                g::send::<C, C_FILE, FileReconsidered>(),
                            ),
                            g::par(
                                g::send::<INTEGRATOR, COMPILER, RetryCForCompiler>(),
                                g::par(
                                    g::seq(
                                        g::send::<INTEGRATOR, A, RetryCToA>(),
                                        g::send::<A, A_FILE, FileUpdated>(),
                                    ),
                                    g::seq(
                                        g::send::<INTEGRATOR, B, RetryCToB>(),
                                        g::send::<B, B_FILE, FileUpdated>(),
                                    ),
                                ),
                            ),
                        ),
                        g::seq(
                            g::seq(g::send::<C_FILE, C, CompileFileC>(), build_c()),
                            g::seq(g::send::<COMPILER, TESTER, TestObjectC>(), test_c()),
                        ),
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
                    g::seq(
                        g::seq(
                            g::send::<INTEGRATOR, INTEGRATOR_FILE, FileReady>(),
                            g::send::<INTEGRATOR_FILE, INTEGRATOR, LinkFile>(),
                        ),
                        g::send::<INTEGRATOR, COMPILER, Link>(),
                    ),
                    g::seq(
                        g::seq(
                            g::send::<COMPILER, INTEGRATOR, Linked>(),
                            g::send::<INTEGRATOR, INTEGRATOR_FILE, FileLinked>(),
                        ),
                        g::par(
                            g::seq(
                                g::send::<INTEGRATOR, A, FinishedA>(),
                                g::send::<A, A_FILE, FileFinished>(),
                            ),
                            g::par(
                                g::seq(
                                    g::send::<INTEGRATOR, B, FinishedB>(),
                                    g::send::<B, B_FILE, FileFinished>(),
                                ),
                                g::par(
                                    g::seq(
                                        g::send::<INTEGRATOR, C, FinishedC>(),
                                        g::send::<C, C_FILE, FileFinished>(),
                                    ),
                                    g::par(
                                        g::seq(
                                            g::send::<INTEGRATOR, TESTER, FinishedTester>(),
                                            g::send::<TESTER, TESTER_FILE, ObjectForTesting>(),
                                        ),
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
pub fn source() -> impl Projectable {
    g::seq(
        g::par(
            g::send::<A_DATA, COMPILER, Loan<A_FILE>>(),
            g::par(
                g::send::<B_DATA, COMPILER, Loan<B_FILE>>(),
                g::par(
                    g::send::<C_DATA, COMPILER, Loan<C_FILE>>(),
                    g::par(
                        g::send::<INTEGRATOR_DATA, COMPILER, Loan<INTEGRATOR_FILE>>(),
                        g::send::<TESTER_DATA, COMPILER, Loan<TESTER_FILE>>(),
                    ),
                ),
            ),
        ),
        g::par(
            g::seq(
                g::route(
                    g::seq(
                        g::send::<A_FILE, A, CompileFileA>(),
                        g::seq(
                            initial_a(),
                            g::route(
                                g::seq(
                                    g::send::<B_FILE, B, CompileFileB>(),
                                    pipeline(
                                        (g::send::<COMPILER, TESTER, TestObjectA>(), test_a()),
                                        (initial_b(), g::send::<COMPILER, TESTER, TestObjectB>(), test_b()),
                                        (g::seq(
                                            g::send::<C_FILE, C, CompileFileC>(),
                                            initial_c(),
                                        ), g::send::<COMPILER, TESTER, TestObjectC>(), test_c()),
                                    ),
                                ),
                                g::seq(
                                    g::send::<C_FILE, C, CompileFileC>(),
                                    pipeline(
                                        (g::send::<COMPILER, TESTER, TestObjectA>(), test_a()),
                                        (initial_c(), g::send::<COMPILER, TESTER, TestObjectC>(), test_c()),
                                        (g::seq(
                                            g::send::<B_FILE, B, CompileFileB>(),
                                            initial_b(),
                                        ), g::send::<COMPILER, TESTER, TestObjectB>(), test_b()),
                                    ),
                                ),
                            ),
                        ),
                    ),
                    g::route(
                        g::seq(
                            g::send::<B_FILE, B, CompileFileB>(),
                            g::seq(
                                initial_b(),
                                g::route(
                                    g::seq(
                                        g::send::<A_FILE, A, CompileFileA>(),
                                        pipeline(
                                            (g::send::<COMPILER, TESTER, TestObjectB>(), test_b()),
                                            (initial_a(), g::send::<COMPILER, TESTER, TestObjectA>(), test_a()),
                                            (g::seq(
                                                g::send::<C_FILE, C, CompileFileC>(),
                                                initial_c(),
                                            ), g::send::<COMPILER, TESTER, TestObjectC>(), test_c()),
                                        ),
                                    ),
                                    g::seq(
                                        g::send::<C_FILE, C, CompileFileC>(),
                                        pipeline(
                                            (g::send::<COMPILER, TESTER, TestObjectB>(), test_b()),
                                            (initial_c(), g::send::<COMPILER, TESTER, TestObjectC>(), test_c()),
                                            (g::seq(
                                                g::send::<A_FILE, A, CompileFileA>(),
                                                initial_a(),
                                            ), g::send::<COMPILER, TESTER, TestObjectA>(), test_a()),
                                        ),
                                    ),
                                ),
                            ),
                        ),
                        g::seq(
                            g::send::<C_FILE, C, CompileFileC>(),
                            g::seq(
                                initial_c(),
                                g::route(
                                    g::seq(
                                        g::send::<A_FILE, A, CompileFileA>(),
                                        pipeline(
                                            (g::send::<COMPILER, TESTER, TestObjectC>(), test_c()),
                                            (initial_a(), g::send::<COMPILER, TESTER, TestObjectA>(), test_a()),
                                            (g::seq(
                                                g::send::<B_FILE, B, CompileFileB>(),
                                                initial_b(),
                                            ), g::send::<COMPILER, TESTER, TestObjectB>(), test_b()),
                                        ),
                                    ),
                                    g::seq(
                                        g::send::<B_FILE, B, CompileFileB>(),
                                        pipeline(
                                            (g::send::<COMPILER, TESTER, TestObjectC>(), test_c()),
                                            (initial_b(), g::send::<COMPILER, TESTER, TestObjectB>(), test_b()),
                                            (g::seq(
                                                g::send::<A_FILE, A, CompileFileA>(),
                                                initial_a(),
                                            ), g::send::<COMPILER, TESTER, TestObjectA>(), test_a()),
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
                g::seq(
                    all_results(),
                    g::route(
                        g::seq(
                            repair().roll(),
                            finish(),
                        ),
                        finish(),
                    ),
                ),
            ),
            g::par(
                g::par(
                    g::seq(
                        crate::shell::global::file_metadata::<A_FILE, A_DATA>().roll(),
                        g::send::<A_FILE, A_DATA, FileExit>(),
                    ),
                    g::par(
                        g::seq(
                            crate::shell::global::file_metadata::<B_FILE, B_DATA>().roll(),
                            g::send::<B_FILE, B_DATA, FileExit>(),
                        ),
                        g::par(
                            g::seq(
                                crate::shell::global::file_metadata::<C_FILE, C_DATA>().roll(),
                                g::send::<C_FILE, C_DATA, FileExit>(),
                            ),
                            g::par(
                                g::seq(
                                    crate::shell::global::file_metadata::<INTEGRATOR_FILE, INTEGRATOR_DATA>().roll(),
                                    g::send::<INTEGRATOR_FILE, INTEGRATOR_DATA, FileExit>(),
                                ),
                                g::seq(
                                    crate::shell::global::file_metadata::<TESTER_FILE, TESTER_DATA>().roll(),
                                    g::send::<TESTER_FILE, TESTER_DATA, FileExit>(),
                                ),
                            ),
                        ),
                    ),
                ),
                g::par(
                    g::seq(
                        crate::shell::global::private_io::<A_VM, A_IO>(),
                        g::send::<A_VM, A_IO, crate::shell::global::Exit>(),
                    ),
                    g::par(
                        g::seq(
                            crate::shell::global::private_io::<B_VM, B_IO>(),
                            g::send::<B_VM, B_IO, crate::shell::global::Exit>(),
                        ),
                        g::par(
                            g::seq(
                                crate::shell::global::private_io::<C_VM, C_IO>(),
                                g::send::<C_VM, C_IO, crate::shell::global::Exit>(),
                            ),
                            g::par(
                                g::seq(
                                    crate::shell::global::private_io::<INTEGRATOR_VM, INTEGRATOR_IO>(),
                                    g::send::<INTEGRATOR_VM, INTEGRATOR_IO, crate::shell::global::Exit>(),
                                ),
                                g::seq(
                                    crate::shell::global::private_io::<TESTER_VM, TESTER_IO>(),
                                    g::send::<TESTER_VM, TESTER_IO, crate::shell::global::Exit>(),
                                ),
                            ),
                        ),
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
    pub a_file: RoleProgram<A_FILE>,
    pub a_vm: RoleProgram<A_VM>,
    pub a_data: RoleProgram<A_DATA>,
    pub b_file: RoleProgram<B_FILE>,
    pub b_vm: RoleProgram<B_VM>,
    pub b_data: RoleProgram<B_DATA>,
    pub c_file: RoleProgram<C_FILE>,
    pub c_vm: RoleProgram<C_VM>,
    pub c_data: RoleProgram<C_DATA>,
    pub integrator_file: RoleProgram<INTEGRATOR_FILE>,
    pub integrator_vm: RoleProgram<INTEGRATOR_VM>,
    pub integrator_data: RoleProgram<INTEGRATOR_DATA>,
    pub tester_file: RoleProgram<TESTER_FILE>,
    pub tester_vm: RoleProgram<TESTER_VM>,
    pub tester_data: RoleProgram<TESTER_DATA>,
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
            ("CompileCore", CompileCore::<A>::LOGICAL_LABEL),
            ("CompileConfig", CompileConfig::<A>::LOGICAL_LABEL),
            ("CompileModels", CompileModels::<B>::LOGICAL_LABEL),
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
            ("CompileFileA", CompileFileA::LOGICAL_LABEL),
            ("CompileFileB", CompileFileB::LOGICAL_LABEL),
            ("CompileFileC", CompileFileC::LOGICAL_LABEL),
            ("FileCompiled", FileCompiled::LOGICAL_LABEL),
            ("FileUpdated", FileUpdated::LOGICAL_LABEL),
            ("ObjectForTesting", ObjectForTesting::LOGICAL_LABEL),
            ("TestFileA", TestFileA::LOGICAL_LABEL),
            ("TestFileB", TestFileB::LOGICAL_LABEL),
            ("TestFileC", TestFileC::LOGICAL_LABEL),
            ("FileTested", FileTested::LOGICAL_LABEL),
            ("LinkFile", LinkFile::LOGICAL_LABEL),
            ("FileLinked", FileLinked::LOGICAL_LABEL),
            ("FileFeedback", FileFeedback::LOGICAL_LABEL),
            ("FileReconsidered", FileReconsidered::LOGICAL_LABEL),
            ("FileFinished", FileFinished::LOGICAL_LABEL),
            ("FileReady", FileReady::LOGICAL_LABEL),
        ]
        .into_iter()
        .map(|(name, label)| (name.into(), label.into()))
        .collect();
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"10M","session":SESSION,"labels":labels,
            "roles":{"BUILDER_A":A,"BUILDER_B":B,"BUILDER_C":C,"INTEGRATOR":INTEGRATOR,"TEST_RUNNER":TESTER,
                "COMPILER":COMPILER,"TEST_SERVICE":TEST_SERVICE,"CONFIG":CONFIG,"CORE_LIB":CORE,"DATA_MODELS":MODELS,
                "A_IO":A_IO,"B_IO":B_IO,"C_IO":C_IO,"INTEGRATOR_IO":INTEGRATOR_IO,"TESTER_IO":TESTER_IO,"A_FILE":A_FILE,"A_VM":A_VM,"A_DATA":A_DATA,"B_FILE":B_FILE,"B_VM":B_VM,"B_DATA":B_DATA,"C_FILE":C_FILE,"C_VM":C_VM,"C_DATA":C_DATA,"INTEGRATOR_FILE":INTEGRATOR_FILE,"INTEGRATOR_VM":INTEGRATOR_VM,"INTEGRATOR_DATA":INTEGRATOR_DATA,"TESTER_FILE":TESTER_FILE,"TESTER_VM":TESTER_VM,"TESTER_DATA":TESTER_DATA},
            "programs":[rows(&self.a),rows(&self.b),rows(&self.c),rows(&self.integrator),rows(&self.tester),
                rows(&self.compiler),rows(&self.testing),rows(&self.config),rows(&self.core),rows(&self.models),
                rows(&self.a_io),rows(&self.b_io),rows(&self.c_io),rows(&self.integrator_io),rows(&self.tester_io),rows(&self.a_file),rows(&self.a_vm),rows(&self.a_data),rows(&self.b_file),rows(&self.b_vm),rows(&self.b_data),rows(&self.c_file),rows(&self.c_vm),rows(&self.c_data),rows(&self.integrator_file),rows(&self.integrator_vm),rows(&self.integrator_data),rows(&self.tester_file),rows(&self.tester_vm),rows(&self.tester_data)]})
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
        a_file: project::<A_FILE>(&source),
        a_vm: project::<A_VM>(&source),
        a_data: project::<A_DATA>(&source),
        b_file: project::<B_FILE>(&source),
        b_vm: project::<B_VM>(&source),
        b_data: project::<B_DATA>(&source),
        c_file: project::<C_FILE>(&source),
        c_vm: project::<C_VM>(&source),
        c_data: project::<C_DATA>(&source),
        integrator_file: project::<INTEGRATOR_FILE>(&source),
        integrator_vm: project::<INTEGRATOR_VM>(&source),
        integrator_data: project::<INTEGRATOR_DATA>(&source),
        tester_file: project::<TESTER_FILE>(&source),
        tester_vm: project::<TESTER_VM>(&source),
        tester_data: project::<TESTER_DATA>(&source),
    }
}
