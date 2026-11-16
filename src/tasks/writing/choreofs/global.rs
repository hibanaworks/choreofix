//! Direct file requests, actual section Endpoints and original effect results.
use super::imp::{Arguments, WriteArguments};
use crate::shell::global::FileExit;
pub use crate::shell::global::Loan;
use hibana::{
    g,
    runtime::program::{Projectable, RoleProgram, project},
};

pub const SESSION: u32 = 18;
pub type ReadRequest<const ROLE: u8> = g::Msg<1, Arguments<ROLE, 0>>;
pub type DraftRequest<const ROLE: u8> = g::Msg<2, Arguments<ROLE, 1>>;
pub type WriteRequest<const ROLE: u8> = g::Msg<3, Arguments<ROLE, 2>>;
pub type ReviewRequest<const ROLE: u8> = g::Msg<4, Arguments<ROLE, 3>>;
pub type ReadResult<'a> = g::Msg<5, &'a [u8]>;
pub type DraftResult<'a> = g::Msg<6, &'a [u8]>;
pub type WriteResult<'a> = g::Msg<7, &'a [u8]>;
pub type ReviewResult<'a> = g::Msg<8, &'a [u8]>;
pub type SectionRequest<const RESOURCE: u8> = g::Msg<RESOURCE, WriteArguments>;
pub type SectionWritten<'a, const RESOURCE: u8> = g::Msg<RESOURCE, &'a [u8]>;

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
pub const FILES: [hibana_wasip1_runtime::choreofs::ChoreoFsObject<'static>; 4] = [
    object(b"service/read_literature", 4),
    object(b"service/draft_content", 5),
    object(b"service/write_to_document", 17),
    object(b"service/review_own_work", 18),
];

fn prepare<const OWNER: u8, const FILE: u8>() -> impl Projectable {
    g::seq(
        g::send::<FILE, OWNER, ReadRequest<OWNER>>(),
        g::seq(
            g::send::<OWNER, FILE, ReadResult>(),
            g::seq(
                g::send::<FILE, OWNER, DraftRequest<OWNER>>(),
                g::send::<OWNER, FILE, DraftResult>(),
            ),
        ),
    )
}

// Reused by all fifteen researchers. The argument edges remain explicit at
// each call site; this is the same two-resource choreography, not a dispatcher.
fn writing<const OWNER: u8, const FIRST: u8, const SECOND: u8>(
    (request, reply): (impl Projectable, impl Projectable),
    (first_loan, second_loan): (impl Projectable, impl Projectable),
    (first_return, second_return): (impl Projectable, impl Projectable),
) -> impl Projectable {
    g::seq(
        request,
        g::seq(
            g::par(first_loan, second_loan),
            g::seq(
                g::par(
                    g::send::<FIRST, OWNER, SectionRequest<FIRST>>(),
                    g::send::<SECOND, OWNER, SectionRequest<SECOND>>(),
                ),
                g::seq(
                    g::par(
                        g::send::<OWNER, FIRST, SectionWritten<FIRST>>(),
                        g::send::<OWNER, SECOND, SectionWritten<SECOND>>(),
                    ),
                    g::seq(g::par(first_return, second_return), reply),
                ),
            ),
        ),
    )
}
fn review<const OWNER: u8, const FILE: u8>() -> impl Projectable {
    g::seq(
        g::send::<FILE, OWNER, ReviewRequest<OWNER>>(),
        g::send::<OWNER, FILE, ReviewResult>(),
    )
}

pub mod roles2e {
    pub const A: u8 = 0;
    pub const A_FILE: u8 = 80;
    pub const A_VM: u8 = 96;
    pub const A_IO: u8 = 112;
    pub const A_DATA: u8 = 128;
    pub const B: u8 = 1;
    pub const B_FILE: u8 = 81;
    pub const B_VM: u8 = 97;
    pub const B_IO: u8 = 113;
    pub const B_DATA: u8 = 129;
    pub const C: u8 = 2;
    pub const C_FILE: u8 = 82;
    pub const C_VM: u8 = 98;
    pub const C_IO: u8 = 114;
    pub const C_DATA: u8 = 130;
    pub const INTRO: u8 = 32;
    pub const INTRO_RESOURCE: u8 = 64;
    pub const METHODS: u8 = 33;
    pub const METHODS_RESOURCE: u8 = 65;
    pub const RESULTS: u8 = 34;
    pub const RESULTS_RESOURCE: u8 = 66;
}
pub struct Programs2E {
    pub a: RoleProgram<{ roles2e::A }>,
    pub a_file: RoleProgram<{ roles2e::A_FILE }>,
    pub a_vm: RoleProgram<{ roles2e::A_VM }>,
    pub a_io: RoleProgram<{ roles2e::A_IO }>,
    pub a_data: RoleProgram<{ roles2e::A_DATA }>,
    pub b: RoleProgram<{ roles2e::B }>,
    pub b_file: RoleProgram<{ roles2e::B_FILE }>,
    pub b_vm: RoleProgram<{ roles2e::B_VM }>,
    pub b_io: RoleProgram<{ roles2e::B_IO }>,
    pub b_data: RoleProgram<{ roles2e::B_DATA }>,
    pub c: RoleProgram<{ roles2e::C }>,
    pub c_file: RoleProgram<{ roles2e::C_FILE }>,
    pub c_vm: RoleProgram<{ roles2e::C_VM }>,
    pub c_io: RoleProgram<{ roles2e::C_IO }>,
    pub c_data: RoleProgram<{ roles2e::C_DATA }>,
    pub intro: RoleProgram<{ roles2e::INTRO }>,
    pub intro_resource: RoleProgram<{ roles2e::INTRO_RESOURCE }>,
    pub methods: RoleProgram<{ roles2e::METHODS }>,
    pub methods_resource: RoleProgram<{ roles2e::METHODS_RESOURCE }>,
    pub results: RoleProgram<{ roles2e::RESULTS }>,
    pub results_resource: RoleProgram<{ roles2e::RESULTS_RESOURCE }>,
}
pub fn project_2e() -> Programs2E {
    use roles2e::*;
    let writing_a = writing::<A, INTRO_RESOURCE, METHODS_RESOURCE>(
        (
            g::send::<A_FILE, A, WriteRequest<A>>(),
            g::send::<A, A_FILE, WriteResult>(),
        ),
        (
            g::send::<INTRO, A, Loan<INTRO_RESOURCE>>(),
            g::send::<METHODS, A, Loan<METHODS_RESOURCE>>(),
        ),
        (
            g::send::<A, INTRO, Loan<INTRO_RESOURCE>>(),
            g::send::<A, METHODS, Loan<METHODS_RESOURCE>>(),
        ),
    );
    let writing_b = writing::<B, METHODS_RESOURCE, RESULTS_RESOURCE>(
        (
            g::send::<B_FILE, B, WriteRequest<B>>(),
            g::send::<B, B_FILE, WriteResult>(),
        ),
        (
            g::send::<METHODS, B, Loan<METHODS_RESOURCE>>(),
            g::send::<RESULTS, B, Loan<RESULTS_RESOURCE>>(),
        ),
        (
            g::send::<B, METHODS, Loan<METHODS_RESOURCE>>(),
            g::send::<B, RESULTS, Loan<RESULTS_RESOURCE>>(),
        ),
    );
    let writing_c = writing::<C, INTRO_RESOURCE, RESULTS_RESOURCE>(
        (
            g::send::<C_FILE, C, WriteRequest<C>>(),
            g::send::<C, C_FILE, WriteResult>(),
        ),
        (
            g::send::<INTRO, C, Loan<INTRO_RESOURCE>>(),
            g::send::<RESULTS, C, Loan<RESULTS_RESOURCE>>(),
        ),
        (
            g::send::<C, INTRO, Loan<INTRO_RESOURCE>>(),
            g::send::<C, RESULTS, Loan<RESULTS_RESOURCE>>(),
        ),
    );
    let preparations = g::par(
        prepare::<A, A_FILE>(),
        g::par(prepare::<B, B_FILE>(), prepare::<C, C_FILE>()),
    );
    let writes = g::seq(writing_a, g::seq(writing_b, writing_c));
    let reviews = g::par(
        review::<A, A_FILE>(),
        g::par(review::<B, B_FILE>(), review::<C, C_FILE>()),
    );
    let task = g::seq(preparations, g::seq(writes, reviews));
    let initial = g::par(
        g::send::<A_DATA, A_VM, Loan<A_FILE>>(),
        g::par(
            g::send::<B_DATA, B_VM, Loan<B_FILE>>(),
            g::send::<C_DATA, C_VM, Loan<C_FILE>>(),
        ),
    );
    let private = g::par(
        g::seq(
            crate::shell::global::private_io::<A_VM, A_IO>(),
            g::send::<A_VM, A_IO, crate::shell::global::Exit>(),
        ),
        g::par(
            g::seq(
                crate::shell::global::private_io::<B_VM, B_IO>(),
                g::send::<B_VM, B_IO, crate::shell::global::Exit>(),
            ),
            g::seq(
                crate::shell::global::private_io::<C_VM, C_IO>(),
                g::send::<C_VM, C_IO, crate::shell::global::Exit>(),
            ),
        ),
    );
    let metadata = g::par(
        g::seq(
            crate::shell::global::file_metadata::<A_FILE, A_DATA>().roll(),
            g::send::<A_FILE, A_DATA, FileExit>(),
        ),
        g::par(
            g::seq(
                crate::shell::global::file_metadata::<B_FILE, B_DATA>().roll(),
                g::send::<B_FILE, B_DATA, FileExit>(),
            ),
            g::seq(
                crate::shell::global::file_metadata::<C_FILE, C_DATA>().roll(),
                g::send::<C_FILE, C_DATA, FileExit>(),
            ),
        ),
    );
    let protocol = g::seq(initial, g::par(task, g::par(private, metadata)));
    Programs2E {
        a: project(&protocol),
        a_file: project(&protocol),
        a_vm: project(&protocol),
        a_io: project(&protocol),
        a_data: project(&protocol),
        b: project(&protocol),
        b_file: project(&protocol),
        b_vm: project(&protocol),
        b_io: project(&protocol),
        b_data: project(&protocol),
        c: project(&protocol),
        c_file: project(&protocol),
        c_vm: project(&protocol),
        c_io: project(&protocol),
        c_data: project(&protocol),
        intro: project(&protocol),
        intro_resource: project(&protocol),
        methods: project(&protocol),
        methods_resource: project(&protocol),
        results: project(&protocol),
        results_resource: project(&protocol),
    }
}
impl Programs2E {
    pub fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        use roles2e::*;
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"2E","session":SESSION,"participants":{
"RESEARCHER_A":{"role":A,"file":A_FILE,"vm":A_VM,"io":A_IO,"data":A_DATA,"resources":[INTRO_RESOURCE,METHODS_RESOURCE]},
"RESEARCHER_B":{"role":B,"file":B_FILE,"vm":B_VM,"io":B_IO,"data":B_DATA,"resources":[METHODS_RESOURCE,RESULTS_RESOURCE]},
"RESEARCHER_C":{"role":C,"file":C_FILE,"vm":C_VM,"io":C_IO,"data":C_DATA,"resources":[INTRO_RESOURCE,RESULTS_RESOURCE]}},"resources":{"INTRO_SECTION":{"role":INTRO_RESOURCE,"service":INTRO},"METHODS_SECTION":{"role":METHODS_RESOURCE,"service":METHODS},"RESULTS_SECTION":{"role":RESULTS_RESOURCE,"service":RESULTS}},"labels":{"read":1,"draft":2,"write":3,"review":4,"read_result":5,"draft_result":6,"write_result":7,"review_result":8},"programs":[rows(&self.a),rows(&self.a_file),rows(&self.a_vm),rows(&self.a_io),rows(&self.a_data),rows(&self.b),rows(&self.b_file),rows(&self.b_vm),rows(&self.b_io),rows(&self.b_data),rows(&self.c),rows(&self.c_file),rows(&self.c_vm),rows(&self.c_io),rows(&self.c_data),rows(&self.intro),rows(&self.intro_resource),rows(&self.methods),rows(&self.methods_resource),rows(&self.results),rows(&self.results_resource)]})
    }
}

pub mod roles2m {
    pub const A: u8 = 3;
    pub const A_FILE: u8 = 80;
    pub const A_VM: u8 = 96;
    pub const A_IO: u8 = 112;
    pub const A_DATA: u8 = 128;
    pub const B: u8 = 4;
    pub const B_FILE: u8 = 81;
    pub const B_VM: u8 = 97;
    pub const B_IO: u8 = 113;
    pub const B_DATA: u8 = 129;
    pub const C: u8 = 5;
    pub const C_FILE: u8 = 82;
    pub const C_VM: u8 = 98;
    pub const C_IO: u8 = 114;
    pub const C_DATA: u8 = 130;
    pub const D: u8 = 6;
    pub const D_FILE: u8 = 83;
    pub const D_VM: u8 = 99;
    pub const D_IO: u8 = 115;
    pub const D_DATA: u8 = 131;
    pub const E: u8 = 7;
    pub const E_FILE: u8 = 84;
    pub const E_VM: u8 = 100;
    pub const E_IO: u8 = 116;
    pub const E_DATA: u8 = 132;
    pub const BACKGROUND: u8 = 32;
    pub const BACKGROUND_RESOURCE: u8 = 64;
    pub const INTRO: u8 = 33;
    pub const INTRO_RESOURCE: u8 = 65;
    pub const METHODS: u8 = 34;
    pub const METHODS_RESOURCE: u8 = 66;
    pub const EXPERIMENTS: u8 = 35;
    pub const EXPERIMENTS_RESOURCE: u8 = 67;
    pub const RESULTS: u8 = 36;
    pub const RESULTS_RESOURCE: u8 = 68;
}
pub struct Programs2M {
    pub a: RoleProgram<{ roles2m::A }>,
    pub a_file: RoleProgram<{ roles2m::A_FILE }>,
    pub a_vm: RoleProgram<{ roles2m::A_VM }>,
    pub a_io: RoleProgram<{ roles2m::A_IO }>,
    pub a_data: RoleProgram<{ roles2m::A_DATA }>,
    pub b: RoleProgram<{ roles2m::B }>,
    pub b_file: RoleProgram<{ roles2m::B_FILE }>,
    pub b_vm: RoleProgram<{ roles2m::B_VM }>,
    pub b_io: RoleProgram<{ roles2m::B_IO }>,
    pub b_data: RoleProgram<{ roles2m::B_DATA }>,
    pub c: RoleProgram<{ roles2m::C }>,
    pub c_file: RoleProgram<{ roles2m::C_FILE }>,
    pub c_vm: RoleProgram<{ roles2m::C_VM }>,
    pub c_io: RoleProgram<{ roles2m::C_IO }>,
    pub c_data: RoleProgram<{ roles2m::C_DATA }>,
    pub d: RoleProgram<{ roles2m::D }>,
    pub d_file: RoleProgram<{ roles2m::D_FILE }>,
    pub d_vm: RoleProgram<{ roles2m::D_VM }>,
    pub d_io: RoleProgram<{ roles2m::D_IO }>,
    pub d_data: RoleProgram<{ roles2m::D_DATA }>,
    pub e: RoleProgram<{ roles2m::E }>,
    pub e_file: RoleProgram<{ roles2m::E_FILE }>,
    pub e_vm: RoleProgram<{ roles2m::E_VM }>,
    pub e_io: RoleProgram<{ roles2m::E_IO }>,
    pub e_data: RoleProgram<{ roles2m::E_DATA }>,
    pub background: RoleProgram<{ roles2m::BACKGROUND }>,
    pub background_resource: RoleProgram<{ roles2m::BACKGROUND_RESOURCE }>,
    pub intro: RoleProgram<{ roles2m::INTRO }>,
    pub intro_resource: RoleProgram<{ roles2m::INTRO_RESOURCE }>,
    pub methods: RoleProgram<{ roles2m::METHODS }>,
    pub methods_resource: RoleProgram<{ roles2m::METHODS_RESOURCE }>,
    pub experiments: RoleProgram<{ roles2m::EXPERIMENTS }>,
    pub experiments_resource: RoleProgram<{ roles2m::EXPERIMENTS_RESOURCE }>,
    pub results: RoleProgram<{ roles2m::RESULTS }>,
    pub results_resource: RoleProgram<{ roles2m::RESULTS_RESOURCE }>,
}
pub fn project_2m() -> Programs2M {
    use roles2m::*;
    let writing_a = writing::<A, BACKGROUND_RESOURCE, INTRO_RESOURCE>(
        (
            g::send::<A_FILE, A, WriteRequest<A>>(),
            g::send::<A, A_FILE, WriteResult>(),
        ),
        (
            g::send::<BACKGROUND, A, Loan<BACKGROUND_RESOURCE>>(),
            g::send::<INTRO, A, Loan<INTRO_RESOURCE>>(),
        ),
        (
            g::send::<A, BACKGROUND, Loan<BACKGROUND_RESOURCE>>(),
            g::send::<A, INTRO, Loan<INTRO_RESOURCE>>(),
        ),
    );
    let writing_b = writing::<B, BACKGROUND_RESOURCE, METHODS_RESOURCE>(
        (
            g::send::<B_FILE, B, WriteRequest<B>>(),
            g::send::<B, B_FILE, WriteResult>(),
        ),
        (
            g::send::<BACKGROUND, B, Loan<BACKGROUND_RESOURCE>>(),
            g::send::<METHODS, B, Loan<METHODS_RESOURCE>>(),
        ),
        (
            g::send::<B, BACKGROUND, Loan<BACKGROUND_RESOURCE>>(),
            g::send::<B, METHODS, Loan<METHODS_RESOURCE>>(),
        ),
    );
    let writing_c = writing::<C, EXPERIMENTS_RESOURCE, METHODS_RESOURCE>(
        (
            g::send::<C_FILE, C, WriteRequest<C>>(),
            g::send::<C, C_FILE, WriteResult>(),
        ),
        (
            g::send::<EXPERIMENTS, C, Loan<EXPERIMENTS_RESOURCE>>(),
            g::send::<METHODS, C, Loan<METHODS_RESOURCE>>(),
        ),
        (
            g::send::<C, EXPERIMENTS, Loan<EXPERIMENTS_RESOURCE>>(),
            g::send::<C, METHODS, Loan<METHODS_RESOURCE>>(),
        ),
    );
    let writing_d = writing::<D, EXPERIMENTS_RESOURCE, RESULTS_RESOURCE>(
        (
            g::send::<D_FILE, D, WriteRequest<D>>(),
            g::send::<D, D_FILE, WriteResult>(),
        ),
        (
            g::send::<EXPERIMENTS, D, Loan<EXPERIMENTS_RESOURCE>>(),
            g::send::<RESULTS, D, Loan<RESULTS_RESOURCE>>(),
        ),
        (
            g::send::<D, EXPERIMENTS, Loan<EXPERIMENTS_RESOURCE>>(),
            g::send::<D, RESULTS, Loan<RESULTS_RESOURCE>>(),
        ),
    );
    let writing_e = writing::<E, INTRO_RESOURCE, RESULTS_RESOURCE>(
        (
            g::send::<E_FILE, E, WriteRequest<E>>(),
            g::send::<E, E_FILE, WriteResult>(),
        ),
        (
            g::send::<INTRO, E, Loan<INTRO_RESOURCE>>(),
            g::send::<RESULTS, E, Loan<RESULTS_RESOURCE>>(),
        ),
        (
            g::send::<E, INTRO, Loan<INTRO_RESOURCE>>(),
            g::send::<E, RESULTS, Loan<RESULTS_RESOURCE>>(),
        ),
    );
    let preparations = g::par(
        prepare::<A, A_FILE>(),
        g::par(
            prepare::<B, B_FILE>(),
            g::par(
                prepare::<C, C_FILE>(),
                g::par(prepare::<D, D_FILE>(), prepare::<E, E_FILE>()),
            ),
        ),
    );
    let writes = g::seq(
        g::par(writing_a, writing_c),
        g::seq(g::par(writing_b, writing_d), writing_e),
    );
    let reviews = g::par(
        review::<A, A_FILE>(),
        g::par(
            review::<B, B_FILE>(),
            g::par(
                review::<C, C_FILE>(),
                g::par(review::<D, D_FILE>(), review::<E, E_FILE>()),
            ),
        ),
    );
    let task = g::seq(preparations, g::seq(writes, reviews));
    let initial = g::par(
        g::send::<A_DATA, A_VM, Loan<A_FILE>>(),
        g::par(
            g::send::<B_DATA, B_VM, Loan<B_FILE>>(),
            g::par(
                g::send::<C_DATA, C_VM, Loan<C_FILE>>(),
                g::par(
                    g::send::<D_DATA, D_VM, Loan<D_FILE>>(),
                    g::send::<E_DATA, E_VM, Loan<E_FILE>>(),
                ),
            ),
        ),
    );
    let private = g::par(
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
                        crate::shell::global::private_io::<D_VM, D_IO>(),
                        g::send::<D_VM, D_IO, crate::shell::global::Exit>(),
                    ),
                    g::seq(
                        crate::shell::global::private_io::<E_VM, E_IO>(),
                        g::send::<E_VM, E_IO, crate::shell::global::Exit>(),
                    ),
                ),
            ),
        ),
    );
    let metadata = g::par(
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
                        crate::shell::global::file_metadata::<D_FILE, D_DATA>().roll(),
                        g::send::<D_FILE, D_DATA, FileExit>(),
                    ),
                    g::seq(
                        crate::shell::global::file_metadata::<E_FILE, E_DATA>().roll(),
                        g::send::<E_FILE, E_DATA, FileExit>(),
                    ),
                ),
            ),
        ),
    );
    let protocol = g::seq(initial, g::par(task, g::par(private, metadata)));
    Programs2M {
        a: project(&protocol),
        a_file: project(&protocol),
        a_vm: project(&protocol),
        a_io: project(&protocol),
        a_data: project(&protocol),
        b: project(&protocol),
        b_file: project(&protocol),
        b_vm: project(&protocol),
        b_io: project(&protocol),
        b_data: project(&protocol),
        c: project(&protocol),
        c_file: project(&protocol),
        c_vm: project(&protocol),
        c_io: project(&protocol),
        c_data: project(&protocol),
        d: project(&protocol),
        d_file: project(&protocol),
        d_vm: project(&protocol),
        d_io: project(&protocol),
        d_data: project(&protocol),
        e: project(&protocol),
        e_file: project(&protocol),
        e_vm: project(&protocol),
        e_io: project(&protocol),
        e_data: project(&protocol),
        background: project(&protocol),
        background_resource: project(&protocol),
        intro: project(&protocol),
        intro_resource: project(&protocol),
        methods: project(&protocol),
        methods_resource: project(&protocol),
        experiments: project(&protocol),
        experiments_resource: project(&protocol),
        results: project(&protocol),
        results_resource: project(&protocol),
    }
}
impl Programs2M {
    pub fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        use roles2m::*;
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"2M","session":SESSION,"participants":{
"RESEARCHER_A":{"role":A,"file":A_FILE,"vm":A_VM,"io":A_IO,"data":A_DATA,"resources":[BACKGROUND_RESOURCE,INTRO_RESOURCE]},
"RESEARCHER_B":{"role":B,"file":B_FILE,"vm":B_VM,"io":B_IO,"data":B_DATA,"resources":[BACKGROUND_RESOURCE,METHODS_RESOURCE]},
"RESEARCHER_C":{"role":C,"file":C_FILE,"vm":C_VM,"io":C_IO,"data":C_DATA,"resources":[EXPERIMENTS_RESOURCE,METHODS_RESOURCE]},
"RESEARCHER_D":{"role":D,"file":D_FILE,"vm":D_VM,"io":D_IO,"data":D_DATA,"resources":[EXPERIMENTS_RESOURCE,RESULTS_RESOURCE]},
"RESEARCHER_E":{"role":E,"file":E_FILE,"vm":E_VM,"io":E_IO,"data":E_DATA,"resources":[INTRO_RESOURCE,RESULTS_RESOURCE]}},"resources":{"BACKGROUND_SECTION":{"role":BACKGROUND_RESOURCE,"service":BACKGROUND},"INTRO_SECTION":{"role":INTRO_RESOURCE,"service":INTRO},"METHODS_SECTION":{"role":METHODS_RESOURCE,"service":METHODS},"EXPERIMENTS_SECTION":{"role":EXPERIMENTS_RESOURCE,"service":EXPERIMENTS},"RESULTS_SECTION":{"role":RESULTS_RESOURCE,"service":RESULTS}},"labels":{"read":1,"draft":2,"write":3,"review":4,"read_result":5,"draft_result":6,"write_result":7,"review_result":8},"programs":[rows(&self.a),rows(&self.a_file),rows(&self.a_vm),rows(&self.a_io),rows(&self.a_data),rows(&self.b),rows(&self.b_file),rows(&self.b_vm),rows(&self.b_io),rows(&self.b_data),rows(&self.c),rows(&self.c_file),rows(&self.c_vm),rows(&self.c_io),rows(&self.c_data),rows(&self.d),rows(&self.d_file),rows(&self.d_vm),rows(&self.d_io),rows(&self.d_data),rows(&self.e),rows(&self.e_file),rows(&self.e_vm),rows(&self.e_io),rows(&self.e_data),rows(&self.background),rows(&self.background_resource),rows(&self.intro),rows(&self.intro_resource),rows(&self.methods),rows(&self.methods_resource),rows(&self.experiments),rows(&self.experiments_resource),rows(&self.results),rows(&self.results_resource)]})
    }
}

pub mod roles2h {
    pub const A: u8 = 8;
    pub const A_FILE: u8 = 80;
    pub const A_VM: u8 = 96;
    pub const A_IO: u8 = 112;
    pub const A_DATA: u8 = 128;
    pub const B: u8 = 9;
    pub const B_FILE: u8 = 81;
    pub const B_VM: u8 = 97;
    pub const B_IO: u8 = 113;
    pub const B_DATA: u8 = 129;
    pub const C: u8 = 10;
    pub const C_FILE: u8 = 82;
    pub const C_VM: u8 = 98;
    pub const C_IO: u8 = 114;
    pub const C_DATA: u8 = 130;
    pub const D: u8 = 11;
    pub const D_FILE: u8 = 83;
    pub const D_VM: u8 = 99;
    pub const D_IO: u8 = 115;
    pub const D_DATA: u8 = 131;
    pub const E: u8 = 12;
    pub const E_FILE: u8 = 84;
    pub const E_VM: u8 = 100;
    pub const E_IO: u8 = 116;
    pub const E_DATA: u8 = 132;
    pub const F: u8 = 13;
    pub const F_FILE: u8 = 85;
    pub const F_VM: u8 = 101;
    pub const F_IO: u8 = 117;
    pub const F_DATA: u8 = 133;
    pub const G: u8 = 14;
    pub const G_FILE: u8 = 86;
    pub const G_VM: u8 = 102;
    pub const G_IO: u8 = 118;
    pub const G_DATA: u8 = 134;
    pub const BACKGROUND: u8 = 32;
    pub const BACKGROUND_RESOURCE: u8 = 64;
    pub const INTRO: u8 = 33;
    pub const INTRO_RESOURCE: u8 = 65;
    pub const RELATED: u8 = 34;
    pub const RELATED_RESOURCE: u8 = 66;
    pub const METHODS: u8 = 35;
    pub const METHODS_RESOURCE: u8 = 67;
    pub const EXPERIMENTS: u8 = 36;
    pub const EXPERIMENTS_RESOURCE: u8 = 68;
    pub const RESULTS: u8 = 37;
    pub const RESULTS_RESOURCE: u8 = 69;
    pub const DISCUSSION: u8 = 38;
    pub const DISCUSSION_RESOURCE: u8 = 70;
}
pub struct Programs2H {
    pub a: RoleProgram<{ roles2h::A }>,
    pub a_file: RoleProgram<{ roles2h::A_FILE }>,
    pub a_vm: RoleProgram<{ roles2h::A_VM }>,
    pub a_io: RoleProgram<{ roles2h::A_IO }>,
    pub a_data: RoleProgram<{ roles2h::A_DATA }>,
    pub b: RoleProgram<{ roles2h::B }>,
    pub b_file: RoleProgram<{ roles2h::B_FILE }>,
    pub b_vm: RoleProgram<{ roles2h::B_VM }>,
    pub b_io: RoleProgram<{ roles2h::B_IO }>,
    pub b_data: RoleProgram<{ roles2h::B_DATA }>,
    pub c: RoleProgram<{ roles2h::C }>,
    pub c_file: RoleProgram<{ roles2h::C_FILE }>,
    pub c_vm: RoleProgram<{ roles2h::C_VM }>,
    pub c_io: RoleProgram<{ roles2h::C_IO }>,
    pub c_data: RoleProgram<{ roles2h::C_DATA }>,
    pub d: RoleProgram<{ roles2h::D }>,
    pub d_file: RoleProgram<{ roles2h::D_FILE }>,
    pub d_vm: RoleProgram<{ roles2h::D_VM }>,
    pub d_io: RoleProgram<{ roles2h::D_IO }>,
    pub d_data: RoleProgram<{ roles2h::D_DATA }>,
    pub e: RoleProgram<{ roles2h::E }>,
    pub e_file: RoleProgram<{ roles2h::E_FILE }>,
    pub e_vm: RoleProgram<{ roles2h::E_VM }>,
    pub e_io: RoleProgram<{ roles2h::E_IO }>,
    pub e_data: RoleProgram<{ roles2h::E_DATA }>,
    pub f: RoleProgram<{ roles2h::F }>,
    pub f_file: RoleProgram<{ roles2h::F_FILE }>,
    pub f_vm: RoleProgram<{ roles2h::F_VM }>,
    pub f_io: RoleProgram<{ roles2h::F_IO }>,
    pub f_data: RoleProgram<{ roles2h::F_DATA }>,
    pub g: RoleProgram<{ roles2h::G }>,
    pub g_file: RoleProgram<{ roles2h::G_FILE }>,
    pub g_vm: RoleProgram<{ roles2h::G_VM }>,
    pub g_io: RoleProgram<{ roles2h::G_IO }>,
    pub g_data: RoleProgram<{ roles2h::G_DATA }>,
    pub background: RoleProgram<{ roles2h::BACKGROUND }>,
    pub background_resource: RoleProgram<{ roles2h::BACKGROUND_RESOURCE }>,
    pub intro: RoleProgram<{ roles2h::INTRO }>,
    pub intro_resource: RoleProgram<{ roles2h::INTRO_RESOURCE }>,
    pub related: RoleProgram<{ roles2h::RELATED }>,
    pub related_resource: RoleProgram<{ roles2h::RELATED_RESOURCE }>,
    pub methods: RoleProgram<{ roles2h::METHODS }>,
    pub methods_resource: RoleProgram<{ roles2h::METHODS_RESOURCE }>,
    pub experiments: RoleProgram<{ roles2h::EXPERIMENTS }>,
    pub experiments_resource: RoleProgram<{ roles2h::EXPERIMENTS_RESOURCE }>,
    pub results: RoleProgram<{ roles2h::RESULTS }>,
    pub results_resource: RoleProgram<{ roles2h::RESULTS_RESOURCE }>,
    pub discussion: RoleProgram<{ roles2h::DISCUSSION }>,
    pub discussion_resource: RoleProgram<{ roles2h::DISCUSSION_RESOURCE }>,
}
pub fn project_2h() -> Programs2H {
    use roles2h::*;
    let writing_a = writing::<A, BACKGROUND_RESOURCE, INTRO_RESOURCE>(
        (
            g::send::<A_FILE, A, WriteRequest<A>>(),
            g::send::<A, A_FILE, WriteResult>(),
        ),
        (
            g::send::<BACKGROUND, A, Loan<BACKGROUND_RESOURCE>>(),
            g::send::<INTRO, A, Loan<INTRO_RESOURCE>>(),
        ),
        (
            g::send::<A, BACKGROUND, Loan<BACKGROUND_RESOURCE>>(),
            g::send::<A, INTRO, Loan<INTRO_RESOURCE>>(),
        ),
    );
    let writing_b = writing::<B, BACKGROUND_RESOURCE, RELATED_RESOURCE>(
        (
            g::send::<B_FILE, B, WriteRequest<B>>(),
            g::send::<B, B_FILE, WriteResult>(),
        ),
        (
            g::send::<BACKGROUND, B, Loan<BACKGROUND_RESOURCE>>(),
            g::send::<RELATED, B, Loan<RELATED_RESOURCE>>(),
        ),
        (
            g::send::<B, BACKGROUND, Loan<BACKGROUND_RESOURCE>>(),
            g::send::<B, RELATED, Loan<RELATED_RESOURCE>>(),
        ),
    );
    let writing_c = writing::<C, METHODS_RESOURCE, RELATED_RESOURCE>(
        (
            g::send::<C_FILE, C, WriteRequest<C>>(),
            g::send::<C, C_FILE, WriteResult>(),
        ),
        (
            g::send::<METHODS, C, Loan<METHODS_RESOURCE>>(),
            g::send::<RELATED, C, Loan<RELATED_RESOURCE>>(),
        ),
        (
            g::send::<C, METHODS, Loan<METHODS_RESOURCE>>(),
            g::send::<C, RELATED, Loan<RELATED_RESOURCE>>(),
        ),
    );
    let writing_d = writing::<D, EXPERIMENTS_RESOURCE, METHODS_RESOURCE>(
        (
            g::send::<D_FILE, D, WriteRequest<D>>(),
            g::send::<D, D_FILE, WriteResult>(),
        ),
        (
            g::send::<EXPERIMENTS, D, Loan<EXPERIMENTS_RESOURCE>>(),
            g::send::<METHODS, D, Loan<METHODS_RESOURCE>>(),
        ),
        (
            g::send::<D, EXPERIMENTS, Loan<EXPERIMENTS_RESOURCE>>(),
            g::send::<D, METHODS, Loan<METHODS_RESOURCE>>(),
        ),
    );
    let writing_e = writing::<E, EXPERIMENTS_RESOURCE, RESULTS_RESOURCE>(
        (
            g::send::<E_FILE, E, WriteRequest<E>>(),
            g::send::<E, E_FILE, WriteResult>(),
        ),
        (
            g::send::<EXPERIMENTS, E, Loan<EXPERIMENTS_RESOURCE>>(),
            g::send::<RESULTS, E, Loan<RESULTS_RESOURCE>>(),
        ),
        (
            g::send::<E, EXPERIMENTS, Loan<EXPERIMENTS_RESOURCE>>(),
            g::send::<E, RESULTS, Loan<RESULTS_RESOURCE>>(),
        ),
    );
    let writing_f = writing::<F, DISCUSSION_RESOURCE, RESULTS_RESOURCE>(
        (
            g::send::<F_FILE, F, WriteRequest<F>>(),
            g::send::<F, F_FILE, WriteResult>(),
        ),
        (
            g::send::<DISCUSSION, F, Loan<DISCUSSION_RESOURCE>>(),
            g::send::<RESULTS, F, Loan<RESULTS_RESOURCE>>(),
        ),
        (
            g::send::<F, DISCUSSION, Loan<DISCUSSION_RESOURCE>>(),
            g::send::<F, RESULTS, Loan<RESULTS_RESOURCE>>(),
        ),
    );
    let writing_g = writing::<G, DISCUSSION_RESOURCE, INTRO_RESOURCE>(
        (
            g::send::<G_FILE, G, WriteRequest<G>>(),
            g::send::<G, G_FILE, WriteResult>(),
        ),
        (
            g::send::<DISCUSSION, G, Loan<DISCUSSION_RESOURCE>>(),
            g::send::<INTRO, G, Loan<INTRO_RESOURCE>>(),
        ),
        (
            g::send::<G, DISCUSSION, Loan<DISCUSSION_RESOURCE>>(),
            g::send::<G, INTRO, Loan<INTRO_RESOURCE>>(),
        ),
    );
    let preparations = g::par(
        prepare::<A, A_FILE>(),
        g::par(
            prepare::<B, B_FILE>(),
            g::par(
                prepare::<C, C_FILE>(),
                g::par(
                    prepare::<D, D_FILE>(),
                    g::par(
                        prepare::<E, E_FILE>(),
                        g::par(prepare::<F, F_FILE>(), prepare::<G, G_FILE>()),
                    ),
                ),
            ),
        ),
    );
    let writes = g::seq(
        g::par(writing_a, g::par(writing_c, writing_e)),
        g::seq(g::par(writing_b, g::par(writing_d, writing_f)), writing_g),
    );
    let reviews = g::par(
        review::<A, A_FILE>(),
        g::par(
            review::<B, B_FILE>(),
            g::par(
                review::<C, C_FILE>(),
                g::par(
                    review::<D, D_FILE>(),
                    g::par(
                        review::<E, E_FILE>(),
                        g::par(review::<F, F_FILE>(), review::<G, G_FILE>()),
                    ),
                ),
            ),
        ),
    );
    let task = g::seq(preparations, g::seq(writes, reviews));
    let initial = g::par(
        g::send::<A_DATA, A_VM, Loan<A_FILE>>(),
        g::par(
            g::send::<B_DATA, B_VM, Loan<B_FILE>>(),
            g::par(
                g::send::<C_DATA, C_VM, Loan<C_FILE>>(),
                g::par(
                    g::send::<D_DATA, D_VM, Loan<D_FILE>>(),
                    g::par(
                        g::send::<E_DATA, E_VM, Loan<E_FILE>>(),
                        g::par(
                            g::send::<F_DATA, F_VM, Loan<F_FILE>>(),
                            g::send::<G_DATA, G_VM, Loan<G_FILE>>(),
                        ),
                    ),
                ),
            ),
        ),
    );
    let private = g::par(
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
                        crate::shell::global::private_io::<D_VM, D_IO>(),
                        g::send::<D_VM, D_IO, crate::shell::global::Exit>(),
                    ),
                    g::par(
                        g::seq(
                            crate::shell::global::private_io::<E_VM, E_IO>(),
                            g::send::<E_VM, E_IO, crate::shell::global::Exit>(),
                        ),
                        g::par(
                            g::seq(
                                crate::shell::global::private_io::<F_VM, F_IO>(),
                                g::send::<F_VM, F_IO, crate::shell::global::Exit>(),
                            ),
                            g::seq(
                                crate::shell::global::private_io::<G_VM, G_IO>(),
                                g::send::<G_VM, G_IO, crate::shell::global::Exit>(),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );
    let metadata = g::par(
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
                        crate::shell::global::file_metadata::<D_FILE, D_DATA>().roll(),
                        g::send::<D_FILE, D_DATA, FileExit>(),
                    ),
                    g::par(
                        g::seq(
                            crate::shell::global::file_metadata::<E_FILE, E_DATA>().roll(),
                            g::send::<E_FILE, E_DATA, FileExit>(),
                        ),
                        g::par(
                            g::seq(
                                crate::shell::global::file_metadata::<F_FILE, F_DATA>().roll(),
                                g::send::<F_FILE, F_DATA, FileExit>(),
                            ),
                            g::seq(
                                crate::shell::global::file_metadata::<G_FILE, G_DATA>().roll(),
                                g::send::<G_FILE, G_DATA, FileExit>(),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );
    let protocol = g::seq(initial, g::par(task, g::par(private, metadata)));
    Programs2H {
        a: project(&protocol),
        a_file: project(&protocol),
        a_vm: project(&protocol),
        a_io: project(&protocol),
        a_data: project(&protocol),
        b: project(&protocol),
        b_file: project(&protocol),
        b_vm: project(&protocol),
        b_io: project(&protocol),
        b_data: project(&protocol),
        c: project(&protocol),
        c_file: project(&protocol),
        c_vm: project(&protocol),
        c_io: project(&protocol),
        c_data: project(&protocol),
        d: project(&protocol),
        d_file: project(&protocol),
        d_vm: project(&protocol),
        d_io: project(&protocol),
        d_data: project(&protocol),
        e: project(&protocol),
        e_file: project(&protocol),
        e_vm: project(&protocol),
        e_io: project(&protocol),
        e_data: project(&protocol),
        f: project(&protocol),
        f_file: project(&protocol),
        f_vm: project(&protocol),
        f_io: project(&protocol),
        f_data: project(&protocol),
        g: project(&protocol),
        g_file: project(&protocol),
        g_vm: project(&protocol),
        g_io: project(&protocol),
        g_data: project(&protocol),
        background: project(&protocol),
        background_resource: project(&protocol),
        intro: project(&protocol),
        intro_resource: project(&protocol),
        related: project(&protocol),
        related_resource: project(&protocol),
        methods: project(&protocol),
        methods_resource: project(&protocol),
        experiments: project(&protocol),
        experiments_resource: project(&protocol),
        results: project(&protocol),
        results_resource: project(&protocol),
        discussion: project(&protocol),
        discussion_resource: project(&protocol),
    }
}
impl Programs2H {
    pub fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        use roles2h::*;
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"2H","session":SESSION,"participants":{
"RESEARCHER_A":{"role":A,"file":A_FILE,"vm":A_VM,"io":A_IO,"data":A_DATA,"resources":[BACKGROUND_RESOURCE,INTRO_RESOURCE]},
"RESEARCHER_B":{"role":B,"file":B_FILE,"vm":B_VM,"io":B_IO,"data":B_DATA,"resources":[BACKGROUND_RESOURCE,RELATED_RESOURCE]},
"RESEARCHER_C":{"role":C,"file":C_FILE,"vm":C_VM,"io":C_IO,"data":C_DATA,"resources":[METHODS_RESOURCE,RELATED_RESOURCE]},
"RESEARCHER_D":{"role":D,"file":D_FILE,"vm":D_VM,"io":D_IO,"data":D_DATA,"resources":[EXPERIMENTS_RESOURCE,METHODS_RESOURCE]},
"RESEARCHER_E":{"role":E,"file":E_FILE,"vm":E_VM,"io":E_IO,"data":E_DATA,"resources":[EXPERIMENTS_RESOURCE,RESULTS_RESOURCE]},
"RESEARCHER_F":{"role":F,"file":F_FILE,"vm":F_VM,"io":F_IO,"data":F_DATA,"resources":[DISCUSSION_RESOURCE,RESULTS_RESOURCE]},
"RESEARCHER_G":{"role":G,"file":G_FILE,"vm":G_VM,"io":G_IO,"data":G_DATA,"resources":[DISCUSSION_RESOURCE,INTRO_RESOURCE]}},"resources":{"BACKGROUND_SECTION":{"role":BACKGROUND_RESOURCE,"service":BACKGROUND},"INTRO_SECTION":{"role":INTRO_RESOURCE,"service":INTRO},"RELATED_SECTION":{"role":RELATED_RESOURCE,"service":RELATED},"METHODS_SECTION":{"role":METHODS_RESOURCE,"service":METHODS},"EXPERIMENTS_SECTION":{"role":EXPERIMENTS_RESOURCE,"service":EXPERIMENTS},"RESULTS_SECTION":{"role":RESULTS_RESOURCE,"service":RESULTS},"DISCUSSION_SECTION":{"role":DISCUSSION_RESOURCE,"service":DISCUSSION}},"labels":{"read":1,"draft":2,"write":3,"review":4,"read_result":5,"draft_result":6,"write_result":7,"review_result":8},"programs":[rows(&self.a),rows(&self.a_file),rows(&self.a_vm),rows(&self.a_io),rows(&self.a_data),rows(&self.b),rows(&self.b_file),rows(&self.b_vm),rows(&self.b_io),rows(&self.b_data),rows(&self.c),rows(&self.c_file),rows(&self.c_vm),rows(&self.c_io),rows(&self.c_data),rows(&self.d),rows(&self.d_file),rows(&self.d_vm),rows(&self.d_io),rows(&self.d_data),rows(&self.e),rows(&self.e_file),rows(&self.e_vm),rows(&self.e_io),rows(&self.e_data),rows(&self.f),rows(&self.f_file),rows(&self.f_vm),rows(&self.f_io),rows(&self.f_data),rows(&self.g),rows(&self.g_file),rows(&self.g_vm),rows(&self.g_io),rows(&self.g_data),rows(&self.background),rows(&self.background_resource),rows(&self.intro),rows(&self.intro_resource),rows(&self.related),rows(&self.related_resource),rows(&self.methods),rows(&self.methods_resource),rows(&self.experiments),rows(&self.experiments_resource),rows(&self.results),rows(&self.results_resource),rows(&self.discussion),rows(&self.discussion_resource)]})
    }
}
