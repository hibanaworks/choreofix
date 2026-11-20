//! Affine study custody and direct ChoreoFS requests for regulatory discovery.
pub use super::arguments::*;
use super::reanalysis::Reanalysis;
pub use crate::shell::global::{Exit, FileExit, Loan, file_metadata, private_io};
use crate::shell::imp::review_reply::ReviewReply;
use hibana::{g, runtime::program::Projectable};
use hibana_wasip1_runtime::{
    choreofs::{ChoreoFsObject, FdSpec, ObjectId},
    protocol,
};

pub const CHEMIST: u8 = 0;
pub const BIOLOGIST: u8 = 1;
pub const TOXICOLOGIST: u8 = 2;
pub const LEAD: u8 = 3;
pub const REGULATOR: u8 = 4;
pub const HPLC_OWNER: u8 = 5;
pub const MASS_OWNER: u8 = 6;
pub const CELL_OWNER: u8 = 7;
pub const REVIEW_OWNER: u8 = 8;
pub const HPLC: u8 = 9;
pub const MASS: u8 = 10;
pub const CELL: u8 = 11;
pub const INIT: u8 = 12;
pub const SYNTHESIS_VIEW: u8 = 20;
pub const CULTURE_VIEW: u8 = 21;
pub const ASSAY_VIEW: u8 = 22;
pub const STABILITY_VIEW: u8 = 23;
pub const PURITY_VIEW: u8 = 24;
pub const TOXICITY_VIEW: u8 = 25;
pub const STRUCTURE_VIEW: u8 = 26;
pub const STUDY: u8 = 27;
pub const CHEM_FILE: u8 = 32;
pub const CHEM_VM: u8 = 64;
pub const CHEM_IO: u8 = 80;
pub const CHEM_DATA: u8 = 96;
pub const BIO_FILE: u8 = 33;
pub const BIO_VM: u8 = 65;
pub const BIO_IO: u8 = 81;
pub const BIO_DATA: u8 = 97;
pub const TOX_FILE: u8 = 34;
pub const TOX_VM: u8 = 66;
pub const TOX_IO: u8 = 82;
pub const TOX_DATA: u8 = 98;
pub const LEAD_FILE: u8 = 35;
pub const LEAD_VM: u8 = 67;
pub const LEAD_IO: u8 = 83;
pub const LEAD_DATA: u8 = 99;
pub const REG_FILE: u8 = 36;
pub const REG_VM: u8 = 68;
pub const REG_IO: u8 = 84;
pub const REG_DATA: u8 = 100;
pub const SESSION: u32 = 141;
pub const ANALYSIS_FAILED: u8 = 71;
pub const REANALYSIS_FAILED: u8 = 72;
pub const SELECT_BIOLOGY: u16 = 0x1411;
pub const SELECT_TOXICOLOGY: u16 = 0x1412;

pub type Request<const OP: u8> = g::Msg<OP, Arguments<OP>>;
pub type Use<const OP: u8> = g::Msg<OP, ()>;
pub type Reply<'a, const OP: u8> = g::Msg<OP, &'a [u8]>;
pub type Passed<'a> = g::Msg<70, ReviewReply<'a, true>>;
pub type Failed<'a, const LABEL: u8 = ANALYSIS_FAILED> = g::Msg<LABEL, ReviewReply<'a, false>>;
pub type ComplianceRejected<'a> = Failed<'a, 73>;
pub type DocumentationFailed<'a> = Failed<'a, 74>;
pub type BiologicalSample<'a> = g::Msg<50, &'a [u8]>;
pub type ToxicologySample<'a> = g::Msg<54, &'a [u8]>;
pub type BioReport<'a> = g::Msg<51, &'a [u8]>;
pub type ToxReport<'a> = g::Msg<52, &'a [u8]>;
pub type Decision<'a> = g::Msg<53, &'a [u8]>;
pub type ReviewInput<'a> = g::Msg<55, &'a [u8]>;
pub type ReviewResult<'a> = g::Msg<56, &'a [u8]>;
pub type ApprovedForLead<'a> = g::Msg<90, ReviewReply<'a, true>>;
pub type ApprovedForBio<'a> = g::Msg<91, ReviewReply<'a, true>>;
pub type ApprovedForTox<'a> = g::Msg<92, ReviewReply<'a, true>>;
pub type ApprovedForHplc<'a> = g::Msg<93, ReviewReply<'a, true>>;
pub type ApprovedForCells<'a> = g::Msg<94, ReviewReply<'a, true>>;
pub type ApprovedForChemist<'a> = g::Msg<95, ReviewReply<'a, true>>;
pub type Plan<'a, const MASK: u8, const TO: u8> = g::Msg<MASK, Reanalysis<'a, MASK, TO>>;
pub type StudyNotice<'a> = g::Msg<96, &'a [u8]>;
pub type ApprovalNotice<'a> = g::Msg<97, &'a [u8]>;

const fn operation(path: &'static [u8], fd: u8) -> ChoreoFsObject<'static> {
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

const fn incoming(path: &'static [u8], fd: u8) -> ChoreoFsObject<'static> {
    ChoreoFsObject::readable(
        path,
        ObjectId(fd as u32),
        FdSpec::new(fd, 2),
        b"",
        protocol::FdBinding::read(protocol::FdReadRow::Base),
    )
}

pub const CHEM_FILES: [ChoreoFsObject<'static>; 4] = [
    operation(b"service/run_hplc_analysis/synthesis_qc", 4),
    operation(b"service/run_mass_spec_analysis/molecular_id", 5),
    incoming(b"service/approval", 6),
    operation(b"service/run_hplc_analysis/documentation", 7),
];
pub const BIO_FILES: [ChoreoFsObject<'static>; 5] = [
    operation(b"service/run_cell_assay/culture_prep", 4),
    incoming(b"service/compound", 5),
    operation(b"service/run_cell_assay/bioassay", 6),
    operation(b"service/run_hplc_analysis/stability", 7),
    incoming(b"service/study", 8),
];
pub const TOX_FILES: [ChoreoFsObject<'static>; 5] = [
    operation(b"service/run_mass_spec_analysis/baseline_markers", 4),
    incoming(b"service/compound", 5),
    operation(b"service/run_hplc_analysis/purity_check", 6),
    operation(b"service/run_cell_assay/cytotoxicity", 7),
    incoming(b"service/study", 8),
];
pub const LEAD_FILES: [ChoreoFsObject<'static>; 8] = [
    operation(b"service/run_mass_spec_analysis/reference_methods", 4),
    operation(b"service/run_hplc_analysis/structural_confirmation", 5),
    operation(b"service/run_mass_spec_analysis/characterization", 6),
    operation(b"service/submit_decision", 7),
    incoming(b"service/bioactivity", 8),
    incoming(b"service/toxicity", 9),
    incoming(b"service/review", 10),
    ChoreoFsObject::writable(
        b"service/reanalysis",
        ObjectId(11),
        FdSpec::new(11, 64),
        protocol::FdBinding::write(protocol::FdWriteRow::Object),
    ),
];
pub const REG_FILES: [ChoreoFsObject<'static>; 2] = [
    incoming(b"service/decision", 4),
    operation(b"service/verify_regulatory_compliance", 5),
];

fn trial<
    const OP: u8,
    const FILE: u8,
    const WHO: u8,
    const OWNER: u8,
    const TOOL: u8,
    const MATERIAL: u8,
>() -> impl Projectable {
    g::seq(
        g::send::<FILE, OWNER, Request<OP>>(),
        g::seq(
            g::send::<OWNER, WHO, Loan<TOOL>>(),
            g::seq(
                g::send::<TOOL, OWNER, Use<OP>>(),
                g::seq(
                    g::send::<WHO, OWNER, Loan<TOOL>>(),
                    g::send::<WHO, OWNER, Loan<MATERIAL>>(),
                ),
            ),
        ),
    )
}

fn analysis<
    const OP: u8,
    const FILE: u8,
    const WHO: u8,
    const OWNER: u8,
    const TOOL: u8,
    const MATERIAL: u8,
    const REJECTION: u8,
>() -> impl Projectable {
    let succeeded = || {
        g::seq(
            g::send::<MATERIAL, WHO, Passed>(),
            g::seq(
                g::send::<WHO, FILE, Reply<OP>>(),
                g::send::<OWNER, LEAD, Loan<MATERIAL>>(),
            ),
        )
    };
    let retry = g::seq(
        g::send::<MATERIAL, WHO, Failed<REJECTION>>(),
        g::seq(
            g::send::<WHO, FILE, Reply<OP>>(),
            g::seq(
                g::send::<OWNER, WHO, Loan<MATERIAL>>(),
                trial::<OP, FILE, WHO, OWNER, TOOL, MATERIAL>(),
            ),
        ),
    );
    g::seq(
        trial::<OP, FILE, WHO, OWNER, TOOL, MATERIAL>(),
        g::route(succeeded(), g::seq(retry.roll(), succeeded())),
    )
}

fn mass<const OP: u8, const FILE: u8, const WHO: u8>() -> impl Projectable {
    g::seq(
        g::send::<FILE, MASS_OWNER, Request<OP>>(),
        g::seq(
            g::send::<MASS_OWNER, WHO, Loan<MASS>>(),
            g::seq(
                g::send::<MASS, MASS_OWNER, Use<OP>>(),
                g::seq(
                    g::send::<MASS_OWNER, MASS, Reply<OP>>(),
                    g::seq(
                        g::send::<WHO, MASS_OWNER, Loan<MASS>>(),
                        g::send::<WHO, FILE, Reply<OP>>(),
                    ),
                ),
            ),
        ),
    )
}

fn biology_report() -> impl Projectable {
    g::seq(
        g::send::<BIOLOGIST, LEAD, BioReport>(),
        g::send::<LEAD, LEAD_FILE, BioReport>(),
    )
}

fn toxicology_report() -> impl Projectable {
    g::seq(
        g::send::<TOXICOLOGIST, LEAD, ToxReport>(),
        g::send::<LEAD, LEAD_FILE, ToxReport>(),
    )
}

fn reports() -> impl Projectable {
    g::par(biology_report(), toxicology_report())
}

fn biology() -> impl Projectable {
    g::seq(
        analysis::<BIOASSAY, BIO_FILE, BIOLOGIST, CELL_OWNER, CELL, ASSAY_VIEW, REANALYSIS_FAILED>(
        ),
        analysis::<
            STABILITY,
            BIO_FILE,
            BIOLOGIST,
            HPLC_OWNER,
            HPLC,
            STABILITY_VIEW,
            REANALYSIS_FAILED,
        >(),
    )
}
fn toxicology() -> impl Projectable {
    g::seq(
        analysis::<PURITY, TOX_FILE, TOXICOLOGIST, HPLC_OWNER, HPLC, PURITY_VIEW, REANALYSIS_FAILED>(
        ),
        analysis::<
            CYTOTOXICITY,
            TOX_FILE,
            TOXICOLOGIST,
            CELL_OWNER,
            CELL,
            TOXICITY_VIEW,
            REANALYSIS_FAILED,
        >(),
    )
}
fn reissue_biology() -> impl Projectable {
    g::par(
        g::send::<LEAD, BIOLOGIST, Loan<ASSAY_VIEW>>(),
        g::send::<LEAD, BIOLOGIST, Loan<STABILITY_VIEW>>(),
    )
}
fn reissue_toxicology() -> impl Projectable {
    g::par(
        g::send::<LEAD, TOXICOLOGIST, Loan<PURITY_VIEW>>(),
        g::send::<LEAD, TOXICOLOGIST, Loan<TOXICITY_VIEW>>(),
    )
}
fn parallel_evaluation<const REJECTION: u8>() -> impl Projectable {
    g::seq(
        g::par(
            analysis::<BIOASSAY, BIO_FILE, BIOLOGIST, CELL_OWNER, CELL, ASSAY_VIEW, REJECTION>(),
            analysis::<PURITY, TOX_FILE, TOXICOLOGIST, HPLC_OWNER, HPLC, PURITY_VIEW, REJECTION>(),
        ),
        g::par(
            analysis::<STABILITY, BIO_FILE, BIOLOGIST, HPLC_OWNER, HPLC, STABILITY_VIEW, REJECTION>(
            ),
            analysis::<
                CYTOTOXICITY,
                TOX_FILE,
                TOXICOLOGIST,
                CELL_OWNER,
                CELL,
                TOXICITY_VIEW,
                REJECTION,
            >(),
        ),
    )
}
fn plan<const MASK: u8>() -> impl Projectable {
    g::par(
        g::send::<STUDY, BIOLOGIST, Plan<MASK, BIOLOGIST>>(),
        g::par(
            g::send::<STUDY, TOXICOLOGIST, Plan<MASK, TOXICOLOGIST>>(),
            g::par(
                g::send::<STUDY, HPLC_OWNER, Plan<MASK, HPLC_OWNER>>(),
                g::send::<STUDY, CELL_OWNER, Plan<MASK, CELL_OWNER>>(),
            ),
        ),
    )
}
fn reconsider() -> impl Projectable {
    g::seq(
        g::send::<LEAD_FILE, LEAD, Request<REANALYSIS>>(),
        g::seq(
            g::route(
                g::route(
                    plan::<0>(),
                    g::seq(
                        plan::<1>(),
                        g::seq(
                            g::send::<BIOLOGIST, BIO_FILE, StudyNotice>(),
                            g::seq(reissue_biology(), g::seq(biology(), biology_report())),
                        ),
                    ),
                )
                .resolve::<SELECT_BIOLOGY>(),
                g::route(
                    g::seq(
                        plan::<2>(),
                        g::seq(
                            g::send::<TOXICOLOGIST, TOX_FILE, StudyNotice>(),
                            g::seq(
                                reissue_toxicology(),
                                g::seq(toxicology(), toxicology_report()),
                            ),
                        ),
                    ),
                    g::seq(
                        plan::<3>(),
                        g::seq(
                            g::par(
                                g::send::<BIOLOGIST, BIO_FILE, StudyNotice>(),
                                g::send::<TOXICOLOGIST, TOX_FILE, StudyNotice>(),
                            ),
                            g::seq(
                                g::par(reissue_biology(), reissue_toxicology()),
                                g::seq(parallel_evaluation::<REANALYSIS_FAILED>(), reports()),
                            ),
                        ),
                    ),
                )
                .resolve::<SELECT_BIOLOGY>(),
            )
            .resolve::<SELECT_TOXICOLOGY>(),
            analysis::<
                STRUCTURE,
                LEAD_FILE,
                LEAD,
                HPLC_OWNER,
                HPLC,
                STRUCTURE_VIEW,
                REANALYSIS_FAILED,
            >(),
        ),
    )
}
fn review() -> impl Projectable {
    g::seq(
        g::send::<LEAD_FILE, LEAD, Request<DECISION>>(),
        g::seq(
            g::send::<LEAD, LEAD_FILE, Reply<DECISION>>(),
            g::seq(
                g::send::<STUDY, REGULATOR, Decision>(),
                g::seq(
                    g::send::<REGULATOR, REG_FILE, Decision>(),
                    g::seq(
                        g::send::<LEAD, REGULATOR, Loan<STUDY>>(),
                        g::seq(
                            g::send::<REG_FILE, REVIEW_OWNER, Request<COMPLIANCE>>(),
                            g::seq(
                                g::send::<STUDY, REVIEW_OWNER, ReviewInput>(),
                                g::send::<REGULATOR, REVIEW_OWNER, Loan<STUDY>>(),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    )
}
fn approved() -> impl Projectable {
    g::seq(
        g::send::<STUDY, REGULATOR, Passed>(),
        g::seq(
            g::send::<REGULATOR, REG_FILE, Reply<COMPLIANCE>>(),
            g::seq(
                g::send::<STUDY, LEAD, ApprovedForLead>(),
                g::seq(
                    g::send::<LEAD, LEAD_FILE, ReviewResult>(),
                    g::seq(
                        g::par(
                            g::seq(
                                g::send::<STUDY, BIOLOGIST, ApprovedForBio>(),
                                g::send::<BIOLOGIST, BIO_FILE, StudyNotice>(),
                            ),
                            g::par(
                                g::seq(
                                    g::send::<STUDY, TOXICOLOGIST, ApprovedForTox>(),
                                    g::send::<TOXICOLOGIST, TOX_FILE, StudyNotice>(),
                                ),
                                g::par(
                                    g::send::<STUDY, HPLC_OWNER, ApprovedForHplc>(),
                                    g::send::<STUDY, CELL_OWNER, ApprovedForCells>(),
                                ),
                            ),
                        ),
                        g::seq(
                            g::send::<STUDY, CHEMIST, ApprovedForChemist>(),
                            g::seq(
                                g::send::<CHEMIST, CHEM_FILE, ApprovalNotice>(),
                                g::send::<REVIEW_OWNER, CHEMIST, Loan<STUDY>>(),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    )
}
fn decision_cycle() -> impl Projectable {
    let retry = g::seq(
        g::send::<STUDY, REGULATOR, ComplianceRejected>(),
        g::seq(
            g::send::<REGULATOR, REG_FILE, Reply<COMPLIANCE>>(),
            g::seq(
                g::send::<STUDY, LEAD, ComplianceRejected>(),
                g::seq(
                    g::send::<LEAD, LEAD_FILE, ReviewResult>(),
                    g::seq(
                        g::send::<REVIEW_OWNER, LEAD, Loan<STUDY>>(),
                        g::seq(reconsider(), review()),
                    ),
                ),
            ),
        ),
    );
    g::seq(
        review(),
        g::route(approved(), g::seq(retry.roll(), approved())),
    )
}
fn document() -> impl Projectable {
    let trial = || {
        g::seq(
            g::send::<CHEM_FILE, HPLC_OWNER, Request<DOCUMENTATION>>(),
            g::seq(
                g::send::<STUDY, HPLC_OWNER, ApprovalNotice>(),
                g::seq(
                    g::send::<HPLC_OWNER, CHEMIST, Loan<HPLC>>(),
                    g::seq(
                        g::send::<HPLC, HPLC_OWNER, Use<DOCUMENTATION>>(),
                        g::seq(
                            g::send::<CHEMIST, HPLC_OWNER, Loan<HPLC>>(),
                            g::send::<CHEMIST, HPLC_OWNER, Loan<STUDY>>(),
                        ),
                    ),
                ),
            ),
        )
    };
    let finished = || {
        g::seq(
            g::send::<STUDY, CHEMIST, Passed>(),
            g::seq(
                g::send::<CHEMIST, CHEM_FILE, Reply<DOCUMENTATION>>(),
                g::send::<HPLC_OWNER, MASS_OWNER, Loan<STUDY>>(),
            ),
        )
    };
    let retry = g::seq(
        g::send::<STUDY, CHEMIST, DocumentationFailed>(),
        g::seq(
            g::send::<CHEMIST, CHEM_FILE, Reply<DOCUMENTATION>>(),
            g::seq(g::send::<HPLC_OWNER, CHEMIST, Loan<STUDY>>(), trial()),
        ),
    );
    g::seq(
        trial(),
        g::route(finished(), g::seq(retry.roll(), finished())),
    )
}

pub fn discovery() -> impl Projectable {
    let instruments = g::par(
        g::send::<INIT, HPLC_OWNER, Loan<HPLC>>(),
        g::par(
            g::send::<INIT, MASS_OWNER, Loan<MASS>>(),
            g::send::<INIT, CELL_OWNER, Loan<CELL>>(),
        ),
    );
    let files = g::par(
        g::send::<CHEM_DATA, CHEMIST, Loan<CHEM_FILE>>(),
        g::par(
            g::send::<BIO_DATA, BIOLOGIST, Loan<BIO_FILE>>(),
            g::par(
                g::send::<TOX_DATA, TOXICOLOGIST, Loan<TOX_FILE>>(),
                g::par(
                    g::send::<LEAD_DATA, LEAD, Loan<LEAD_FILE>>(),
                    g::send::<REG_DATA, REGULATOR, Loan<REG_FILE>>(),
                ),
            ),
        ),
    );
    let guests = g::par(
        g::send::<INIT, CHEMIST, Loan<CHEM_VM>>(),
        g::par(
            g::send::<INIT, BIOLOGIST, Loan<BIO_VM>>(),
            g::par(
                g::send::<INIT, TOXICOLOGIST, Loan<TOX_VM>>(),
                g::par(
                    g::send::<INIT, LEAD, Loan<LEAD_VM>>(),
                    g::send::<INIT, REGULATOR, Loan<REG_VM>>(),
                ),
            ),
        ),
    );
    let prepare = g::par(
        analysis::<
            SYNTHESIS_QC,
            CHEM_FILE,
            CHEMIST,
            HPLC_OWNER,
            HPLC,
            SYNTHESIS_VIEW,
            ANALYSIS_FAILED,
        >(),
        g::par(
            analysis::<
                CULTURE_PREP,
                BIO_FILE,
                BIOLOGIST,
                CELL_OWNER,
                CELL,
                CULTURE_VIEW,
                ANALYSIS_FAILED,
            >(),
            g::seq(
                mass::<BASELINE, TOX_FILE, TOXICOLOGIST>(),
                mass::<REFERENCE, LEAD_FILE, LEAD>(),
            ),
        ),
    );
    let distribute = g::par(
        g::seq(
            g::send::<CHEMIST, BIOLOGIST, BiologicalSample>(),
            g::send::<BIOLOGIST, BIO_FILE, BiologicalSample>(),
        ),
        g::seq(
            g::send::<CHEMIST, TOXICOLOGIST, ToxicologySample>(),
            g::send::<TOXICOLOGIST, TOX_FILE, ToxicologySample>(),
        ),
    );
    let work = g::seq(
        prepare,
        g::seq(
            mass::<MOLECULAR_ID, CHEM_FILE, CHEMIST>(),
            g::seq(
                distribute,
                g::seq(
                    parallel_evaluation::<ANALYSIS_FAILED>(),
                    g::seq(
                        reports(),
                        g::seq(
                            g::par(
                                analysis::<
                                    STRUCTURE,
                                    LEAD_FILE,
                                    LEAD,
                                    HPLC_OWNER,
                                    HPLC,
                                    STRUCTURE_VIEW,
                                    ANALYSIS_FAILED,
                                >(),
                                mass::<CHARACTERIZATION, LEAD_FILE, LEAD>(),
                            ),
                            g::seq(decision_cycle(), document()),
                        ),
                    ),
                ),
            ),
        ),
    );
    let metadata = g::par(
        file_metadata::<CHEM_FILE, CHEM_DATA>().roll(),
        g::par(
            file_metadata::<BIO_FILE, BIO_DATA>().roll(),
            g::par(
                file_metadata::<TOX_FILE, TOX_DATA>().roll(),
                g::par(
                    file_metadata::<LEAD_FILE, LEAD_DATA>().roll(),
                    file_metadata::<REG_FILE, REG_DATA>().roll(),
                ),
            ),
        ),
    );
    let exits = g::par(
        g::send::<CHEM_FILE, CHEM_DATA, FileExit>(),
        g::par(
            g::send::<BIO_FILE, BIO_DATA, FileExit>(),
            g::par(
                g::send::<TOX_FILE, TOX_DATA, FileExit>(),
                g::par(
                    g::send::<LEAD_FILE, LEAD_DATA, FileExit>(),
                    g::send::<REG_FILE, REG_DATA, FileExit>(),
                ),
            ),
        ),
    );
    let io = g::par(
        g::seq(
            private_io::<CHEM_VM, CHEM_IO>(),
            g::send::<CHEM_VM, CHEM_IO, Exit>(),
        ),
        g::par(
            g::seq(
                private_io::<BIO_VM, BIO_IO>(),
                g::send::<BIO_VM, BIO_IO, Exit>(),
            ),
            g::par(
                g::seq(
                    private_io::<TOX_VM, TOX_IO>(),
                    g::send::<TOX_VM, TOX_IO, Exit>(),
                ),
                g::par(
                    g::seq(
                        private_io::<LEAD_VM, LEAD_IO>(),
                        g::send::<LEAD_VM, LEAD_IO, Exit>(),
                    ),
                    g::seq(
                        private_io::<REG_VM, REG_IO>(),
                        g::send::<REG_VM, REG_IO, Exit>(),
                    ),
                ),
            ),
        ),
    );
    g::seq(
        g::par(instruments, g::par(files, guests)),
        g::par(g::seq(g::par(work, metadata), exits), io),
    )
}

#[allow(dead_code)]
pub struct Programs {
    pub chemist: hibana::runtime::program::RoleProgram<CHEMIST>,
    pub biologist: hibana::runtime::program::RoleProgram<BIOLOGIST>,
    pub toxicologist: hibana::runtime::program::RoleProgram<TOXICOLOGIST>,
    pub lead: hibana::runtime::program::RoleProgram<LEAD>,
    pub regulator: hibana::runtime::program::RoleProgram<REGULATOR>,
    pub hplc_owner: hibana::runtime::program::RoleProgram<HPLC_OWNER>,
    pub mass_owner: hibana::runtime::program::RoleProgram<MASS_OWNER>,
    pub cell_owner: hibana::runtime::program::RoleProgram<CELL_OWNER>,
    pub review_owner: hibana::runtime::program::RoleProgram<REVIEW_OWNER>,
    pub hplc: hibana::runtime::program::RoleProgram<HPLC>,
    pub mass: hibana::runtime::program::RoleProgram<MASS>,
    pub cell: hibana::runtime::program::RoleProgram<CELL>,
    pub init: hibana::runtime::program::RoleProgram<INIT>,
    pub synthesis: hibana::runtime::program::RoleProgram<SYNTHESIS_VIEW>,
    pub culture: hibana::runtime::program::RoleProgram<CULTURE_VIEW>,
    pub assay: hibana::runtime::program::RoleProgram<ASSAY_VIEW>,
    pub stability: hibana::runtime::program::RoleProgram<STABILITY_VIEW>,
    pub purity: hibana::runtime::program::RoleProgram<PURITY_VIEW>,
    pub toxicity: hibana::runtime::program::RoleProgram<TOXICITY_VIEW>,
    pub structure: hibana::runtime::program::RoleProgram<STRUCTURE_VIEW>,
    pub study: hibana::runtime::program::RoleProgram<STUDY>,
    pub chem_file: hibana::runtime::program::RoleProgram<CHEM_FILE>,
    pub chem_vm: hibana::runtime::program::RoleProgram<CHEM_VM>,
    pub chem_io: hibana::runtime::program::RoleProgram<CHEM_IO>,
    pub chem_data: hibana::runtime::program::RoleProgram<CHEM_DATA>,
    pub bio_file: hibana::runtime::program::RoleProgram<BIO_FILE>,
    pub bio_vm: hibana::runtime::program::RoleProgram<BIO_VM>,
    pub bio_io: hibana::runtime::program::RoleProgram<BIO_IO>,
    pub bio_data: hibana::runtime::program::RoleProgram<BIO_DATA>,
    pub tox_file: hibana::runtime::program::RoleProgram<TOX_FILE>,
    pub tox_vm: hibana::runtime::program::RoleProgram<TOX_VM>,
    pub tox_io: hibana::runtime::program::RoleProgram<TOX_IO>,
    pub tox_data: hibana::runtime::program::RoleProgram<TOX_DATA>,
    pub lead_file: hibana::runtime::program::RoleProgram<LEAD_FILE>,
    pub lead_vm: hibana::runtime::program::RoleProgram<LEAD_VM>,
    pub lead_io: hibana::runtime::program::RoleProgram<LEAD_IO>,
    pub lead_data: hibana::runtime::program::RoleProgram<LEAD_DATA>,
    pub reg_file: hibana::runtime::program::RoleProgram<REG_FILE>,
    pub reg_vm: hibana::runtime::program::RoleProgram<REG_VM>,
    pub reg_io: hibana::runtime::program::RoleProgram<REG_IO>,
    pub reg_data: hibana::runtime::program::RoleProgram<REG_DATA>,
}
pub fn programs() -> Programs {
    let source = discovery();
    use hibana::runtime::program::project;
    Programs {
        chemist: project(&source),
        biologist: project(&source),
        toxicologist: project(&source),
        lead: project(&source),
        regulator: project(&source),
        hplc_owner: project(&source),
        mass_owner: project(&source),
        cell_owner: project(&source),
        review_owner: project(&source),
        hplc: project(&source),
        mass: project(&source),
        cell: project(&source),
        init: project(&source),
        synthesis: project(&source),
        culture: project(&source),
        assay: project(&source),
        stability: project(&source),
        purity: project(&source),
        toxicity: project(&source),
        structure: project(&source),
        study: project(&source),
        chem_file: project(&source),
        chem_vm: project(&source),
        chem_io: project(&source),
        chem_data: project(&source),
        bio_file: project(&source),
        bio_vm: project(&source),
        bio_io: project(&source),
        bio_data: project(&source),
        tox_file: project(&source),
        tox_vm: project(&source),
        tox_io: project(&source),
        tox_data: project(&source),
        lead_file: project(&source),
        lead_vm: project(&source),
        lead_io: project(&source),
        lead_data: project(&source),
        reg_file: project(&source),
        reg_vm: project(&source),
        reg_io: project(&source),
        reg_data: project(&source),
    }
}

impl Programs {
    pub fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        let roles = [
            ("CHEMIST", CHEMIST),
            ("BIOLOGIST", BIOLOGIST),
            ("TOXICOLOGIST", TOXICOLOGIST),
            ("LEAD_SCIENTIST", LEAD),
            ("REGULATORY_SPECIALIST", REGULATOR),
            ("HPLC_OWNER", HPLC_OWNER),
            ("MASS_OWNER", MASS_OWNER),
            ("CELL_OWNER", CELL_OWNER),
            ("REVIEW_OWNER", REVIEW_OWNER),
            ("HPLC", HPLC),
            ("MASS_SPEC", MASS),
            ("CELL_LAB", CELL),
            ("INIT", INIT),
            ("SYNTHESIS_VIEW", SYNTHESIS_VIEW),
            ("CULTURE_VIEW", CULTURE_VIEW),
            ("ASSAY_VIEW", ASSAY_VIEW),
            ("STABILITY_VIEW", STABILITY_VIEW),
            ("PURITY_VIEW", PURITY_VIEW),
            ("TOXICITY_VIEW", TOXICITY_VIEW),
            ("STRUCTURE_VIEW", STRUCTURE_VIEW),
            ("STUDY", STUDY),
            ("CHEM_FILE", CHEM_FILE),
            ("CHEM_VM", CHEM_VM),
            ("CHEM_IO", CHEM_IO),
            ("CHEM_DATA", CHEM_DATA),
            ("BIO_FILE", BIO_FILE),
            ("BIO_VM", BIO_VM),
            ("BIO_IO", BIO_IO),
            ("BIO_DATA", BIO_DATA),
            ("TOX_FILE", TOX_FILE),
            ("TOX_VM", TOX_VM),
            ("TOX_IO", TOX_IO),
            ("TOX_DATA", TOX_DATA),
            ("LEAD_FILE", LEAD_FILE),
            ("LEAD_VM", LEAD_VM),
            ("LEAD_IO", LEAD_IO),
            ("LEAD_DATA", LEAD_DATA),
            ("REG_FILE", REG_FILE),
            ("REG_VM", REG_VM),
            ("REG_IO", REG_IO),
            ("REG_DATA", REG_DATA),
        ]
        .into_iter()
        .collect::<std::collections::BTreeMap<_, _>>();
        serde_json::json!({
            "type": "compiled_projection", "format_version":1, "task":"14M", "session":SESSION,
            "roles": roles,
            "labels": {
                "SynthesisQc": SYNTHESIS_QC,"MolecularId":MOLECULAR_ID,"CulturePrep":CULTURE_PREP,
                "Bioassay":BIOASSAY,"Stability":STABILITY,"Baseline":BASELINE,"Purity":PURITY,
                "Cytotoxicity":CYTOTOXICITY,"Reference":REFERENCE,"Structure":STRUCTURE,
                "Characterization":CHARACTERIZATION,"Decision":DECISION,"Documentation":DOCUMENTATION,
                "Compliance":COMPLIANCE,"Reanalysis":REANALYSIS,
                "BiologicalSample":50,"ToxicologySample":54,"BioReport":51,"ToxReport":52,
                "DecisionReceipt":53,"ReviewInput":55,"ReviewResult":56,"Passed":70,
                "Failed":ANALYSIS_FAILED,"ReanalysisFailed":REANALYSIS_FAILED,
                "ComplianceRejected":73,"DocumentationFailed":74,
                "ApprovedForLead":90,"ApprovedForBio":91,"ApprovedForTox":92,
                "ApprovedForHplc":93,"ApprovedForCells":94,"ApprovedForChemist":95,
                "StudyNotice":96,"ApprovalNotice":97,
            },
            "programs": [
                rows(&self.chemist),
                rows(&self.biologist),
                rows(&self.toxicologist),
                rows(&self.lead),
                rows(&self.regulator),
                rows(&self.hplc_owner),
                rows(&self.mass_owner),
                rows(&self.cell_owner),
                rows(&self.review_owner),
                rows(&self.hplc),
                rows(&self.mass),
                rows(&self.cell),
                rows(&self.init),
                rows(&self.synthesis),
                rows(&self.culture),
                rows(&self.assay),
                rows(&self.stability),
                rows(&self.purity),
                rows(&self.toxicity),
                rows(&self.structure),
                rows(&self.study),
                rows(&self.chem_file),
                rows(&self.chem_vm),
                rows(&self.chem_io),
                rows(&self.chem_data),
                rows(&self.bio_file),
                rows(&self.bio_vm),
                rows(&self.bio_io),
                rows(&self.bio_data),
                rows(&self.tox_file),
                rows(&self.tox_vm),
                rows(&self.tox_io),
                rows(&self.tox_data),
                rows(&self.lead_file),
                rows(&self.lead_vm),
                rows(&self.lead_io),
                rows(&self.lead_data),
                rows(&self.reg_file),
                rows(&self.reg_vm),
                rows(&self.reg_io),
                rows(&self.reg_data),
            ],
        })
    }
}
