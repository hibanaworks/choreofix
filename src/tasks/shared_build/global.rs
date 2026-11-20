//! File operations, two affine build inputs and current object handoff.
use super::wire::Arguments;
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
pub const A_FILE: u8 = 16;
pub const B_FILE: u8 = 17;
pub const INTEGRATOR_FILE: u8 = 18;
pub const A_VM: u8 = 32;
pub const B_VM: u8 = 33;
pub const INTEGRATOR_VM: u8 = 34;
pub const A_DATA: u8 = 48;
pub const B_DATA: u8 = 49;
pub const INTEGRATOR_DATA: u8 = 50;
pub const SESSION: u32 = 101;
pub use crate::shell::global::{FileExit, Loan};
pub type CoreLoan<'r> = g::Msg<1, Endpoint<'r, CORE_LIB>>;
pub type TypesLoan<'r> = g::Msg<2, Endpoint<'r, SHARED_TYPES>>;
pub type CompileA = g::Msg<3, Arguments<BUILDER_A>>;
pub type CompileB = g::Msg<4, Arguments<BUILDER_B>>;
pub type CompileCore<const ROLE: u8> = g::Msg<5, Arguments<ROLE, true>>;
pub type CompileTypes<const ROLE: u8> = g::Msg<6, Arguments<ROLE, true>>;
pub type CoreCompiled<'a> = g::Msg<7, &'a [u8]>;
pub type TypesCompiled<'a> = g::Msg<8, &'a [u8]>;
pub type ReturnCore<'r> = g::Msg<9, Endpoint<'r, CORE_LIB>>;
pub type ReturnTypes<'r> = g::Msg<10, Endpoint<'r, SHARED_TYPES>>;
pub type BuiltA<'a> = g::Msg<11, &'a [u8]>;
pub type BuiltB<'a> = g::Msg<12, &'a [u8]>;
pub type Link = g::Msg<13, Arguments<INTEGRATOR>>;
pub type Linked<'a> = g::Msg<14, &'a [u8]>;
pub type TypesUpdated<'a> = g::Msg<15, &'a [u8]>;
pub type Compiled<'a> = g::Msg<16, &'a [u8]>;
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
pub const FILES: [hibana_wasip1_runtime::choreofs::ChoreoFsObject<'static>; 3] = [
    object(b"service/compile_module", 4),
    object(b"service/link_modules", 4),
    object(b"service/updates", 5),
];
fn build_a() -> impl Projectable {
    g::seq(
        g::par(
            g::send::<CORE_LIB, BUILDER_A, CompileCore<BUILDER_A>>(),
            g::send::<SHARED_TYPES, BUILDER_A, CompileTypes<BUILDER_A>>(),
        ),
        g::seq(
            g::par(
                g::send::<BUILDER_A, CORE_LIB, CoreCompiled>(),
                g::send::<BUILDER_A, SHARED_TYPES, TypesCompiled>(),
            ),
            g::send::<BUILDER_A, A_FILE, Compiled>(),
        ),
    )
}
fn build_b() -> impl Projectable {
    g::seq(
        g::par(
            g::send::<CORE_LIB, BUILDER_B, CompileCore<BUILDER_B>>(),
            g::send::<SHARED_TYPES, BUILDER_B, CompileTypes<BUILDER_B>>(),
        ),
        g::seq(
            g::par(
                g::send::<BUILDER_B, CORE_LIB, CoreCompiled>(),
                g::send::<BUILDER_B, SHARED_TYPES, TypesCompiled>(),
            ),
            g::send::<BUILDER_B, B_FILE, Compiled>(),
        ),
    )
}
fn final_b() -> impl Projectable {
    g::seq(
        build_b(),
        g::par(
            g::send::<BUILDER_B, INTEGRATOR, BuiltB>(),
            g::par(
                g::send::<BUILDER_B, BUILD_SERVICE, ReturnCore>(),
                g::send::<BUILDER_B, BUILD_SERVICE, ReturnTypes>(),
            ),
        ),
    )
}
pub fn source() -> impl Projectable {
    let a_then_b = g::seq(
        build_a(),
        g::par(
            g::send::<BUILDER_A, INTEGRATOR, BuiltA>(),
            g::seq(
                g::send::<BUILDER_A, B_FILE, TypesUpdated>(),
                g::seq(
                    g::send::<B_FILE, BUILDER_B, CompileB>(),
                    g::seq(
                        g::par(
                            g::send::<BUILDER_A, BUILDER_B, CoreLoan>(),
                            g::send::<BUILDER_A, BUILDER_B, TypesLoan>(),
                        ),
                        final_b(),
                    ),
                ),
            ),
        ),
    );
    let a_first = g::seq(
        g::send::<A_FILE, BUILDER_A, CompileA>(),
        g::seq(
            g::par(
                g::send::<BUILD_SERVICE, BUILDER_A, CoreLoan>(),
                g::send::<BUILD_SERVICE, BUILDER_A, TypesLoan>(),
            ),
            a_then_b,
        ),
    );
    let b_first = g::seq(
        g::send::<B_FILE, BUILDER_B, CompileB>(),
        g::seq(
            g::par(
                g::send::<BUILD_SERVICE, BUILDER_B, CoreLoan>(),
                g::send::<BUILD_SERVICE, BUILDER_B, TypesLoan>(),
            ),
            g::seq(
                build_b(),
                g::seq(
                    g::send::<A_FILE, BUILDER_A, CompileA>(),
                    g::seq(
                        g::par(
                            g::send::<BUILDER_B, BUILDER_A, CoreLoan>(),
                            g::send::<BUILDER_B, BUILDER_A, TypesLoan>(),
                        ),
                        a_then_b,
                    ),
                ),
            ),
        ),
    );
    let support = g::par(
        g::par(
            g::par(
                g::seq(
                    crate::shell::global::private_io::<A_VM, A_IO>(),
                    g::send::<A_VM, A_IO, crate::shell::global::Exit>(),
                ),
                g::seq(
                    crate::shell::global::private_io::<B_VM, B_IO>(),
                    g::send::<B_VM, B_IO, crate::shell::global::Exit>(),
                ),
            ),
            g::seq(
                crate::shell::global::private_io::<INTEGRATOR_VM, INTEGRATOR_IO>(),
                g::send::<INTEGRATOR_VM, INTEGRATOR_IO, crate::shell::global::Exit>(),
            ),
        ),
        g::par(
            g::par(
                g::seq(
                    crate::shell::global::file_metadata::<A_FILE, A_DATA>().roll(),
                    g::send::<A_FILE, A_DATA, FileExit>(),
                ),
                g::seq(
                    crate::shell::global::file_metadata::<B_FILE, B_DATA>().roll(),
                    g::send::<B_FILE, B_DATA, FileExit>(),
                ),
            ),
            g::seq(
                crate::shell::global::file_metadata::<INTEGRATOR_FILE, INTEGRATOR_DATA>().roll(),
                g::send::<INTEGRATOR_FILE, INTEGRATOR_DATA, FileExit>(),
            ),
        ),
    );
    let link = g::seq(
        g::send::<INTEGRATOR_FILE, INTEGRATOR, Link>(),
        g::send::<INTEGRATOR, INTEGRATOR_FILE, Linked>(),
    );
    g::seq(
        g::par(
            g::par(
                g::send::<A_DATA, BUILD_SERVICE, Loan<A_FILE>>(),
                g::send::<B_DATA, BUILD_SERVICE, Loan<B_FILE>>(),
            ),
            g::send::<INTEGRATOR_DATA, BUILD_SERVICE, Loan<INTEGRATOR_FILE>>(),
        ),
        g::par(g::seq(g::route(a_first, b_first), link), support),
    )
}
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
    pub a_file: RoleProgram<A_FILE>,
    pub b_file: RoleProgram<B_FILE>,
    pub integrator_file: RoleProgram<INTEGRATOR_FILE>,
    pub a_vm: RoleProgram<A_VM>,
    pub b_vm: RoleProgram<B_VM>,
    pub integrator_vm: RoleProgram<INTEGRATOR_VM>,
    pub a_data: RoleProgram<A_DATA>,
    pub b_data: RoleProgram<B_DATA>,
    pub integrator_data: RoleProgram<INTEGRATOR_DATA>,
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
            ("CompileCore", CompileCore::<BUILDER_A>::LOGICAL_LABEL),
            ("CompileTypes", CompileTypes::<BUILDER_A>::LOGICAL_LABEL),
            ("CoreCompiled", CoreCompiled::LOGICAL_LABEL),
            ("TypesCompiled", TypesCompiled::LOGICAL_LABEL),
            ("ReturnCore", ReturnCore::LOGICAL_LABEL),
            ("ReturnTypes", ReturnTypes::LOGICAL_LABEL),
            ("BuiltA", BuiltA::LOGICAL_LABEL),
            ("BuiltB", BuiltB::LOGICAL_LABEL),
            ("Link", Link::LOGICAL_LABEL),
            ("Linked", Linked::LOGICAL_LABEL),
            ("TypesUpdated", TypesUpdated::LOGICAL_LABEL),
            ("Compiled", Compiled::LOGICAL_LABEL),
        ]
        .into_iter()
        .map(|(name, label)| (name.to_owned(), label.into()))
        .collect();
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"10E","session":SESSION,
            "roles":{"BUILDER_A":BUILDER_A,"BUILDER_B":BUILDER_B,"INTEGRATOR":INTEGRATOR,"CORE_LIB":CORE_LIB,"SHARED_TYPES":SHARED_TYPES,"BUILD_SERVICE":BUILD_SERVICE,"A_IO":A_IO,"B_IO":B_IO,"INTEGRATOR_IO":INTEGRATOR_IO,"A_FILE":A_FILE,"B_FILE":B_FILE,"INTEGRATOR_FILE":INTEGRATOR_FILE,"A_VM":A_VM,"B_VM":B_VM,"INTEGRATOR_VM":INTEGRATOR_VM,"A_DATA":A_DATA,"B_DATA":B_DATA,"INTEGRATOR_DATA":INTEGRATOR_DATA},
            "labels":labels,"programs":[rows(&self.a),rows(&self.b),rows(&self.integrator),rows(&self.core),rows(&self.types),rows(&self.service),rows(&self.a_io),rows(&self.b_io),rows(&self.integrator_io),rows(&self.a_file),rows(&self.b_file),rows(&self.integrator_file),rows(&self.a_vm),rows(&self.b_vm),rows(&self.integrator_vm),rows(&self.a_data),rows(&self.b_data),rows(&self.integrator_data)]})
    }
}
pub fn programs() -> Programs {
    let source = source();
    Programs {
        a: project(&source),
        b: project(&source),
        integrator: project(&source),
        core: project(&source),
        types: project(&source),
        service: project(&source),
        a_io: project(&source),
        b_io: project(&source),
        integrator_io: project(&source),
        a_file: project(&source),
        b_file: project(&source),
        integrator_file: project(&source),
        a_vm: project(&source),
        b_vm: project(&source),
        integrator_vm: project(&source),
        a_data: project(&source),
        b_data: project(&source),
        integrator_data: project(&source),
    }
}
