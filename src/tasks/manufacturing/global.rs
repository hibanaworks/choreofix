//! Actual job requests choose each machine's order; replies do not hold a machine.
use super::imp::Arguments;
use hibana::{
    Endpoint, g,
    runtime::program::{Projectable, RoleProgram, project},
};

pub const TECHNICIAN_A: u8 = 0;
pub const TECHNICIAN_B: u8 = 1;
pub const SUPERVISOR: u8 = 2;
pub const LATHE: u8 = 3;
pub const MILL: u8 = 4;
pub const LATHE_SERVICE: u8 = 5;
pub const MILL_SERVICE: u8 = 6;
pub const A_IO: u8 = 8;
pub const B_IO: u8 = 9;
pub const SUPERVISOR_IO: u8 = 10;
pub const LATHE_FILE_A: u8 = 16;
pub const LATHE_FILE_B: u8 = 17;
pub const MILL_FILE_A: u8 = 18;
pub const MILL_FILE_B: u8 = 19;
pub const SUPERVISOR_FILE: u8 = 20;
pub const A_VM: u8 = 32;
pub const B_VM: u8 = 33;
pub const SUPERVISOR_VM: u8 = 34;
pub const LATHE_DATA_A: u8 = 48;
pub const LATHE_DATA_B: u8 = 49;
pub const MILL_DATA_A: u8 = 50;
pub const MILL_DATA_B: u8 = 51;
pub const SUPERVISOR_DATA: u8 = 52;
pub const SESSION: u32 = 111;
pub use crate::shell::global::{FileExit, Loan};

pub type RequestLatheA = g::Msg<1, Arguments<0>>;
pub type RequestLatheB = g::Msg<2, Arguments<1>>;
pub type RequestMillA = g::Msg<3, Arguments<2>>;
pub type RequestMillB = g::Msg<4, Arguments<3>>;
pub type LatheLoan<'r> = g::Msg<5, Endpoint<'r, LATHE>>;
pub type MillLoan<'r> = g::Msg<6, Endpoint<'r, MILL>>;
pub type LatheJobA = g::Msg<7, Arguments<0>>;
pub type LatheJobB = g::Msg<8, Arguments<1>>;
pub type MillJobA = g::Msg<9, Arguments<2>>;
pub type MillJobB = g::Msg<10, Arguments<3>>;
pub type LatheResult<'a> = g::Msg<11, &'a [u8]>;
pub type MillResult<'a> = g::Msg<12, &'a [u8]>;
pub type ReturnLathe<'r> = g::Msg<13, Endpoint<'r, LATHE>>;
pub type ReturnMill<'r> = g::Msg<14, Endpoint<'r, MILL>>;
pub type LatheDoneA<'a> = g::Msg<15, &'a [u8]>;
pub type LatheDoneB<'a> = g::Msg<16, &'a [u8]>;
pub type MillDoneA<'a> = g::Msg<17, &'a [u8]>;
pub type MillDoneB<'a> = g::Msg<18, &'a [u8]>;
pub type FileLatheResultA<'a> = g::Msg<19, &'a [u8]>;
pub type FileLatheResultB<'a> = g::Msg<20, &'a [u8]>;
pub type FileMillResultA<'a> = g::Msg<21, &'a [u8]>;
pub type FileMillResultB<'a> = g::Msg<22, &'a [u8]>;
pub type CollectReports = g::Msg<23, Arguments<4>>;
pub type Reports<'a> = g::Msg<24, &'a [u8]>;

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
pub const JOB_FILES: [hibana_wasip1_runtime::choreofs::ChoreoFsObject<'static>; 2] = [
    object(b"service/do_lathe_job", 4),
    object(b"service/do_mill_job", 5),
];
pub const REPORT_FILE: hibana_wasip1_runtime::choreofs::ChoreoFsObject<'static> =
    object(b"service/collect_reports", 4);

pub fn source() -> impl Projectable {
    let lathe_a = g::seq(
        g::send::<LATHE_FILE_A, TECHNICIAN_A, RequestLatheA>(),
        g::seq(
            g::send::<LATHE_SERVICE, TECHNICIAN_A, LatheLoan>(),
            g::seq(
                g::send::<LATHE, TECHNICIAN_A, LatheJobA>(),
                g::send::<TECHNICIAN_A, LATHE, LatheResult>(),
            ),
        ),
    );
    let lathe_b = g::seq(
        g::send::<LATHE_FILE_B, TECHNICIAN_B, RequestLatheB>(),
        g::seq(
            g::send::<LATHE_SERVICE, TECHNICIAN_B, LatheLoan>(),
            g::seq(
                g::send::<LATHE, TECHNICIAN_B, LatheJobB>(),
                g::send::<TECHNICIAN_B, LATHE, LatheResult>(),
            ),
        ),
    );
    let mill_a = g::seq(
        g::send::<MILL_FILE_A, TECHNICIAN_A, RequestMillA>(),
        g::seq(
            g::send::<MILL_SERVICE, TECHNICIAN_A, MillLoan>(),
            g::seq(
                g::send::<MILL, TECHNICIAN_A, MillJobA>(),
                g::send::<TECHNICIAN_A, MILL, MillResult>(),
            ),
        ),
    );
    let mill_b = g::seq(
        g::send::<MILL_FILE_B, TECHNICIAN_B, RequestMillB>(),
        g::seq(
            g::send::<MILL_SERVICE, TECHNICIAN_B, MillLoan>(),
            g::seq(
                g::send::<MILL, TECHNICIAN_B, MillJobB>(),
                g::send::<TECHNICIAN_B, MILL, MillResult>(),
            ),
        ),
    );
    let lathe_a_reply = g::par(
        g::send::<TECHNICIAN_A, LATHE_FILE_A, FileLatheResultA>(),
        g::send::<TECHNICIAN_A, SUPERVISOR, LatheDoneA>(),
    );
    let lathe_b_reply = g::par(
        g::send::<TECHNICIAN_B, LATHE_FILE_B, FileLatheResultB>(),
        g::send::<TECHNICIAN_B, SUPERVISOR, LatheDoneB>(),
    );
    let mill_a_reply = g::par(
        g::send::<TECHNICIAN_A, MILL_FILE_A, FileMillResultA>(),
        g::send::<TECHNICIAN_A, SUPERVISOR, MillDoneA>(),
    );
    let mill_b_reply = g::par(
        g::send::<TECHNICIAN_B, MILL_FILE_B, FileMillResultB>(),
        g::send::<TECHNICIAN_B, SUPERVISOR, MillDoneB>(),
    );
    let lathe_a_last = g::seq(
        lathe_a,
        g::par(
            lathe_a_reply,
            g::send::<TECHNICIAN_A, LATHE_SERVICE, ReturnLathe>(),
        ),
    );
    let lathe_b_last = g::seq(
        lathe_b,
        g::par(
            lathe_b_reply,
            g::send::<TECHNICIAN_B, LATHE_SERVICE, ReturnLathe>(),
        ),
    );
    let mill_a_last = g::seq(
        mill_a,
        g::par(
            mill_a_reply,
            g::send::<TECHNICIAN_A, MILL_SERVICE, ReturnMill>(),
        ),
    );
    let mill_b_last = g::seq(
        mill_b,
        g::par(
            mill_b_reply,
            g::send::<TECHNICIAN_B, MILL_SERVICE, ReturnMill>(),
        ),
    );
    // A reply/report can remain in flight while the returned machine starts
    // its other job. Neither guest must consume a reply to release a machine.
    let lathe = g::route(
        g::seq(
            lathe_a,
            g::par(
                lathe_a_reply,
                g::seq(
                    g::send::<TECHNICIAN_A, LATHE_SERVICE, ReturnLathe>(),
                    lathe_b_last,
                ),
            ),
        ),
        g::seq(
            lathe_b,
            g::par(
                lathe_b_reply,
                g::seq(
                    g::send::<TECHNICIAN_B, LATHE_SERVICE, ReturnLathe>(),
                    lathe_a_last,
                ),
            ),
        ),
    );
    let mill = g::route(
        g::seq(
            mill_a,
            g::par(
                mill_a_reply,
                g::seq(
                    g::send::<TECHNICIAN_A, MILL_SERVICE, ReturnMill>(),
                    mill_b_last,
                ),
            ),
        ),
        g::seq(
            mill_b,
            g::par(
                mill_b_reply,
                g::seq(
                    g::send::<TECHNICIAN_B, MILL_SERVICE, ReturnMill>(),
                    mill_a_last,
                ),
            ),
        ),
    );
    let bootstrap = g::par(
        g::par(
            g::send::<LATHE_DATA_A, LATHE_SERVICE, Loan<LATHE_FILE_A>>(),
            g::send::<LATHE_DATA_B, LATHE_SERVICE, Loan<LATHE_FILE_B>>(),
        ),
        g::par(
            g::par(
                g::send::<MILL_DATA_A, MILL_SERVICE, Loan<MILL_FILE_A>>(),
                g::send::<MILL_DATA_B, MILL_SERVICE, Loan<MILL_FILE_B>>(),
            ),
            g::send::<SUPERVISOR_DATA, SUPERVISOR, Loan<SUPERVISOR_FILE>>(),
        ),
    );
    let private = g::par(
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
            crate::shell::global::private_io::<SUPERVISOR_VM, SUPERVISOR_IO>(),
            g::send::<SUPERVISOR_VM, SUPERVISOR_IO, crate::shell::global::Exit>(),
        ),
    );
    let metadata = g::par(
        g::par(
            g::seq(
                crate::shell::global::file_metadata::<LATHE_FILE_A, LATHE_DATA_A>().roll(),
                g::send::<LATHE_FILE_A, LATHE_DATA_A, FileExit>(),
            ),
            g::seq(
                crate::shell::global::file_metadata::<LATHE_FILE_B, LATHE_DATA_B>().roll(),
                g::send::<LATHE_FILE_B, LATHE_DATA_B, FileExit>(),
            ),
        ),
        g::par(
            g::par(
                g::seq(
                    crate::shell::global::file_metadata::<MILL_FILE_A, MILL_DATA_A>().roll(),
                    g::send::<MILL_FILE_A, MILL_DATA_A, FileExit>(),
                ),
                g::seq(
                    crate::shell::global::file_metadata::<MILL_FILE_B, MILL_DATA_B>().roll(),
                    g::send::<MILL_FILE_B, MILL_DATA_B, FileExit>(),
                ),
            ),
            g::seq(
                crate::shell::global::file_metadata::<SUPERVISOR_FILE, SUPERVISOR_DATA>().roll(),
                g::send::<SUPERVISOR_FILE, SUPERVISOR_DATA, FileExit>(),
            ),
        ),
    );
    g::seq(
        bootstrap,
        g::par(
            g::seq(
                g::par(lathe, mill),
                g::seq(
                    g::send::<SUPERVISOR_FILE, SUPERVISOR, CollectReports>(),
                    g::send::<SUPERVISOR, SUPERVISOR_FILE, Reports>(),
                ),
            ),
            g::par(private, metadata),
        ),
    )
}

pub struct Programs {
    pub a: RoleProgram<TECHNICIAN_A>,
    pub b: RoleProgram<TECHNICIAN_B>,
    pub supervisor: RoleProgram<SUPERVISOR>,
    pub lathe: RoleProgram<LATHE>,
    pub mill: RoleProgram<MILL>,
    pub lathe_service: RoleProgram<LATHE_SERVICE>,
    pub mill_service: RoleProgram<MILL_SERVICE>,
    pub a_io: RoleProgram<A_IO>,
    pub b_io: RoleProgram<B_IO>,
    pub supervisor_io: RoleProgram<SUPERVISOR_IO>,
    pub lathe_file_a: RoleProgram<LATHE_FILE_A>,
    pub lathe_file_b: RoleProgram<LATHE_FILE_B>,
    pub mill_file_a: RoleProgram<MILL_FILE_A>,
    pub mill_file_b: RoleProgram<MILL_FILE_B>,
    pub supervisor_file: RoleProgram<SUPERVISOR_FILE>,
    pub a_vm: RoleProgram<A_VM>,
    pub b_vm: RoleProgram<B_VM>,
    pub supervisor_vm: RoleProgram<SUPERVISOR_VM>,
    pub lathe_data_a: RoleProgram<LATHE_DATA_A>,
    pub lathe_data_b: RoleProgram<LATHE_DATA_B>,
    pub mill_data_a: RoleProgram<MILL_DATA_A>,
    pub mill_data_b: RoleProgram<MILL_DATA_B>,
    pub supervisor_data: RoleProgram<SUPERVISOR_DATA>,
}
impl Programs {
    pub fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        use hibana::g::Message;
        let labels: serde_json::Map<String, serde_json::Value> = [
            ("RequestLatheA", RequestLatheA::LOGICAL_LABEL),
            ("RequestLatheB", RequestLatheB::LOGICAL_LABEL),
            ("RequestMillA", RequestMillA::LOGICAL_LABEL),
            ("RequestMillB", RequestMillB::LOGICAL_LABEL),
            ("LatheLoan", LatheLoan::LOGICAL_LABEL),
            ("MillLoan", MillLoan::LOGICAL_LABEL),
            ("LatheJobA", LatheJobA::LOGICAL_LABEL),
            ("LatheJobB", LatheJobB::LOGICAL_LABEL),
            ("MillJobA", MillJobA::LOGICAL_LABEL),
            ("MillJobB", MillJobB::LOGICAL_LABEL),
            ("LatheResult", LatheResult::LOGICAL_LABEL),
            ("MillResult", MillResult::LOGICAL_LABEL),
            ("ReturnLathe", ReturnLathe::LOGICAL_LABEL),
            ("ReturnMill", ReturnMill::LOGICAL_LABEL),
            ("LatheDoneA", LatheDoneA::LOGICAL_LABEL),
            ("LatheDoneB", LatheDoneB::LOGICAL_LABEL),
            ("MillDoneA", MillDoneA::LOGICAL_LABEL),
            ("MillDoneB", MillDoneB::LOGICAL_LABEL),
            ("FileLatheResultA", FileLatheResultA::LOGICAL_LABEL),
            ("FileLatheResultB", FileLatheResultB::LOGICAL_LABEL),
            ("FileMillResultA", FileMillResultA::LOGICAL_LABEL),
            ("FileMillResultB", FileMillResultB::LOGICAL_LABEL),
            ("CollectReports", CollectReports::LOGICAL_LABEL),
            ("Reports", Reports::LOGICAL_LABEL),
        ]
        .into_iter()
        .map(|(name, label)| (name.to_owned(), label.into()))
        .collect();
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"11E","session":SESSION,"labels":labels,
            "roles":{"TECHNICIAN_A":TECHNICIAN_A,"TECHNICIAN_B":TECHNICIAN_B,"SUPERVISOR":SUPERVISOR,
            "LATHE":LATHE,"MILL":MILL,"LATHE_SERVICE":LATHE_SERVICE,"MILL_SERVICE":MILL_SERVICE,
            "A_IO":A_IO,"B_IO":B_IO,"SUPERVISOR_IO":SUPERVISOR_IO,
            "LATHE_FILE_A":LATHE_FILE_A,"LATHE_FILE_B":LATHE_FILE_B,"MILL_FILE_A":MILL_FILE_A,"MILL_FILE_B":MILL_FILE_B,
            "SUPERVISOR_FILE":SUPERVISOR_FILE,"A_VM":A_VM,"B_VM":B_VM,"SUPERVISOR_VM":SUPERVISOR_VM,
            "LATHE_DATA_A":LATHE_DATA_A,"LATHE_DATA_B":LATHE_DATA_B,"MILL_DATA_A":MILL_DATA_A,"MILL_DATA_B":MILL_DATA_B,
            "SUPERVISOR_DATA":SUPERVISOR_DATA},
            "programs":[rows(&self.a),rows(&self.b),rows(&self.supervisor),rows(&self.lathe),rows(&self.mill),
            rows(&self.lathe_service),rows(&self.mill_service),rows(&self.a_io),rows(&self.b_io),rows(&self.supervisor_io),
            rows(&self.lathe_file_a),rows(&self.lathe_file_b),rows(&self.mill_file_a),rows(&self.mill_file_b),rows(&self.supervisor_file),
            rows(&self.a_vm),rows(&self.b_vm),rows(&self.supervisor_vm),rows(&self.lathe_data_a),rows(&self.lathe_data_b),
            rows(&self.mill_data_a),rows(&self.mill_data_b),rows(&self.supervisor_data)]})
    }
}
pub fn programs() -> Programs {
    let source = source();
    Programs {
        a: project(&source),
        b: project(&source),
        supervisor: project(&source),
        lathe: project(&source),
        mill: project(&source),
        lathe_service: project(&source),
        mill_service: project(&source),
        a_io: project(&source),
        b_io: project(&source),
        supervisor_io: project(&source),
        lathe_file_a: project(&source),
        lathe_file_b: project(&source),
        mill_file_a: project(&source),
        mill_file_b: project(&source),
        supervisor_file: project(&source),
        a_vm: project(&source),
        b_vm: project(&source),
        supervisor_vm: project(&source),
        lathe_data_a: project(&source),
        lathe_data_b: project(&source),
        mill_data_a: project(&source),
        mill_data_b: project(&source),
        supervisor_data: project(&source),
    }
}
