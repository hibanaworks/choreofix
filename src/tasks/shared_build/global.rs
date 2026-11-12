//! Either builder starts; a build against the old types is rebuilt after A.
use hibana::{
    Endpoint, g,
    runtime::program::{Projectable, RoleProgram, project},
};

pub const BUILDER_A: u8 = 0;
pub const BUILDER_B: u8 = 1;
pub const INTEGRATOR: u8 = 2;
pub const CORE_LIB: u8 = 3;
pub const SHARED_TYPES: u8 = 4;
pub const BUILD_SERVICE: u8 = 5;
pub const A_IO: u8 = 6;
pub const B_IO: u8 = 7;
pub const INTEGRATOR_IO: u8 = 8;
pub const FIRST_BUILDER: u16 = 901;
pub const SESSION: u32 = 101;

pub type CoreLoan<'r> = g::Msg<1, Endpoint<'r, CORE_LIB>>;
pub type TypesLoan<'r> = g::Msg<2, Endpoint<'r, SHARED_TYPES>>;
pub type CompileA<'a> = g::Msg<3, &'a [u8]>;
pub type CompileB<'a> = g::Msg<4, &'a [u8]>;
pub type CompileCore<'a> = g::Msg<5, &'a [u8]>;
pub type CompileTypes<'a> = g::Msg<6, &'a [u8]>;
pub type CoreCompiled<'a> = g::Msg<7, &'a [u8]>;
pub type TypesCompiled<'a> = g::Msg<8, &'a [u8]>;
pub type ReturnCore<'r> = g::Msg<9, Endpoint<'r, CORE_LIB>>;
pub type ReturnTypes<'r> = g::Msg<10, Endpoint<'r, SHARED_TYPES>>;
pub type BuiltA<'a> = g::Msg<11, &'a [u8]>;
pub type BuiltB<'a> = g::Msg<12, &'a [u8]>;
pub type Link<'a> = g::Msg<13, &'a [u8]>;
pub type Linked<'a> = g::Msg<14, &'a [u8]>;
pub type TypesUpdated<'a> = g::Msg<15, &'a [u8]>;

fn compile_a() -> impl Projectable {
    g::seq(
        g::send::<BUILDER_A, BUILD_SERVICE, CompileA>(),
        g::seq(
            g::par(
                g::send::<CORE_LIB, BUILD_SERVICE, CompileCore>(),
                g::send::<SHARED_TYPES, BUILD_SERVICE, CompileTypes>(),
            ),
            g::par(
                g::send::<BUILD_SERVICE, CORE_LIB, CoreCompiled>(),
                g::send::<BUILD_SERVICE, SHARED_TYPES, TypesCompiled>(),
            ),
        ),
    )
}
fn compile_b() -> impl Projectable {
    g::seq(
        g::send::<BUILDER_B, BUILD_SERVICE, CompileB>(),
        g::seq(
            g::par(
                g::send::<CORE_LIB, BUILD_SERVICE, CompileCore>(),
                g::send::<SHARED_TYPES, BUILD_SERVICE, CompileTypes>(),
            ),
            g::par(
                g::send::<BUILD_SERVICE, CORE_LIB, CoreCompiled>(),
                g::send::<BUILD_SERVICE, SHARED_TYPES, TypesCompiled>(),
            ),
        ),
    )
}
fn final_b() -> impl Projectable {
    g::seq(
        compile_b(),
        g::par(
            g::send::<BUILDER_B, INTEGRATOR, BuiltB>(),
            g::par(
                g::send::<BUILDER_B, BUILD_SERVICE, ReturnCore>(),
                g::send::<BUILDER_B, BUILD_SERVICE, ReturnTypes>(),
            ),
        ),
    )
}
pub fn business() -> impl Projectable {
    let a_then_b = g::seq(
        compile_a(),
        g::par(
            g::send::<BUILDER_A, INTEGRATOR, BuiltA>(),
            g::seq(
                g::send::<BUILDER_A, BUILDER_B, TypesUpdated>(),
                g::seq(
                    g::par(
                        g::send::<BUILDER_A, BUILDER_B, CoreLoan>(),
                        g::send::<BUILDER_A, BUILDER_B, TypesLoan>(),
                    ),
                    final_b(),
                ),
            ),
        ),
    );
    // B's first result uses the provided types. It remains real data, but is
    // not sent to the linker as a current build after A changes those types.
    let b_first = g::seq(
        g::par(
            g::send::<BUILD_SERVICE, BUILDER_B, CoreLoan>(),
            g::send::<BUILD_SERVICE, BUILDER_B, TypesLoan>(),
        ),
        g::seq(
            compile_b(),
            g::seq(
                g::par(
                    g::send::<BUILDER_B, BUILDER_A, CoreLoan>(),
                    g::send::<BUILDER_B, BUILDER_A, TypesLoan>(),
                ),
                a_then_b,
            ),
        ),
    );
    let a_first = g::seq(
        g::par(
            g::send::<BUILD_SERVICE, BUILDER_A, CoreLoan>(),
            g::send::<BUILD_SERVICE, BUILDER_A, TypesLoan>(),
        ),
        a_then_b,
    );
    g::seq(
        g::route(a_first, b_first).resolve::<FIRST_BUILDER>(),
        g::seq(
            g::send::<INTEGRATOR, BUILD_SERVICE, Link>(),
            g::send::<BUILD_SERVICE, INTEGRATOR, Linked>(),
        ),
    )
}
pub fn source() -> impl Projectable {
    g::par(
        business(),
        g::par(
            crate::shell::global::private_io::<BUILDER_A, A_IO>(),
            g::par(
                crate::shell::global::private_io::<BUILDER_B, B_IO>(),
                crate::shell::global::private_io::<INTEGRATOR, INTEGRATOR_IO>(),
            ),
        ),
    )
}

/// Immutable sealed images; no owner table or application progress.
pub struct Programs {
    pub a: RoleProgram<BUILDER_A>,
    pub b: RoleProgram<BUILDER_B>,
    pub integrator: RoleProgram<INTEGRATOR>,
    pub core: RoleProgram<CORE_LIB>,
    pub types: RoleProgram<SHARED_TYPES>,
    pub service: RoleProgram<BUILD_SERVICE>,
    pub a_io: RoleProgram<A_IO>,
    pub b_io: RoleProgram<B_IO>,
    pub integrator_io: RoleProgram<INTEGRATOR_IO>,
}
impl Programs {
    pub fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        use hibana::g::Message;
        let labels: serde_json::Map<String, serde_json::Value> = [
            ("CoreLoan", CoreLoan::LOGICAL_LABEL),
            ("TypesLoan", TypesLoan::LOGICAL_LABEL),
            ("CompileA", CompileA::LOGICAL_LABEL),
            ("CompileB", CompileB::LOGICAL_LABEL),
            ("CompileCore", CompileCore::LOGICAL_LABEL),
            ("CompileTypes", CompileTypes::LOGICAL_LABEL),
            ("CoreCompiled", CoreCompiled::LOGICAL_LABEL),
            ("TypesCompiled", TypesCompiled::LOGICAL_LABEL),
            ("ReturnCore", ReturnCore::LOGICAL_LABEL),
            ("ReturnTypes", ReturnTypes::LOGICAL_LABEL),
            ("BuiltA", BuiltA::LOGICAL_LABEL),
            ("BuiltB", BuiltB::LOGICAL_LABEL),
            ("Link", Link::LOGICAL_LABEL),
            ("Linked", Linked::LOGICAL_LABEL),
            ("TypesUpdated", TypesUpdated::LOGICAL_LABEL),
        ]
        .into_iter()
        .map(|(name, label)| (name.to_owned(), label.into()))
        .collect();
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"10E",
            "session":SESSION,"labels":labels,
            "roles":{"BUILDER_A":BUILDER_A,"BUILDER_B":BUILDER_B,"INTEGRATOR":INTEGRATOR,
                "CORE_LIB":CORE_LIB,"SHARED_TYPES":SHARED_TYPES,"BUILD_SERVICE":BUILD_SERVICE,
                "A_IO":A_IO,"B_IO":B_IO,"INTEGRATOR_IO":INTEGRATOR_IO},
            "programs":[rows(&self.a), rows(&self.b), rows(&self.integrator), rows(&self.core),
                rows(&self.types), rows(&self.service), rows(&self.a_io), rows(&self.b_io), rows(&self.integrator_io)]})
    }
}
pub fn programs() -> Programs {
    let source = source();
    Programs {
        a: project::<BUILDER_A>(&source),
        b: project::<BUILDER_B>(&source),
        integrator: project::<INTEGRATOR>(&source),
        core: project::<CORE_LIB>(&source),
        types: project::<SHARED_TYPES>(&source),
        service: project::<BUILD_SERVICE>(&source),
        a_io: project::<A_IO>(&source),
        b_io: project::<B_IO>(&source),
        integrator_io: project::<INTEGRATOR_IO>(&source),
    }
}
