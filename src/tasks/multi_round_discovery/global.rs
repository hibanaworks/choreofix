//! 14H: actual material, finite samples, repeated review and affine study custody.
use super::imp;
pub use crate::shell::global::{Exit, FileExit, Loan, file_metadata, private_io};
use hibana::{Endpoint, g, runtime::program::Projectable};
use hibana_wasip1_runtime::{
    choreofs::{ChoreoFsObject, FdSpec, ObjectId},
    protocol,
};

pub const CHEMIST: u8 = 0;
pub const BIOLOGIST: u8 = 1;
pub const TOXICOLOGIST: u8 = 2;
pub const FORMULATOR: u8 = 3;
pub const CLINICIAN: u8 = 4;
pub const REGULATOR: u8 = 5;
pub const DIRECTOR: u8 = 6;
pub const HPLC_SERVICE: u8 = 14;
pub const MASS_SERVICE: u8 = 15;
pub const CELL_SERVICE: u8 = 16;
pub const FORM_SERVICE: u8 = 17;
pub const STUDY_SERVICE: u8 = 18;
pub const HPLC: u8 = 19;
pub const MASS: u8 = 20;
pub const CELL: u8 = 21;
pub const FORM: u8 = 22;
pub const SAMPLES: u8 = 23;
pub const QC: u8 = 24;
pub const CULTURE: u8 = 25;
pub const FORMULATION: u8 = 26;
pub const BIOASSAY: u8 = 27;
pub const STABILITY: u8 = 28;
pub const PURITY: u8 = 29;
pub const TOXICITY: u8 = 30;
pub const STUDY: u8 = 31;

pub type FileRequest<'a, const STEP: u8> = g::Msg<STEP, imp::AnalysisArguments<'a, STEP>>;
pub type FileResult<'a, const STEP: u8> = g::Msg<STEP, &'a [u8]>;
pub type Use<const STEP: u8> = g::Msg<STEP, ()>;
pub type Publish<'a, const STEP: u8> = g::Msg<81, imp::EvidenceSubmission<'a, STEP>>;
pub type StudyFile<'a, const FILE: u8> = g::Msg<82, imp::JsonTo<'a, FILE>>;
pub type Passed<'a, const STEP: u8> = g::Msg<3, imp::AnalysisReply<'a, STEP, true>>;
pub type Failed<'a, const STEP: u8> = g::Msg<4, imp::AnalysisReply<'a, STEP, false>>;
pub type Instrument<'r, const ROLE: u8> = g::Msg<5, Endpoint<'r, ROLE>>;
pub type Specimen<'r, const ROLE: u8> = g::Msg<6, Endpoint<'r, ROLE>>;
pub type Archived<'r, const ROLE: u8> = g::Msg<7, Endpoint<'r, ROLE>>;
pub type SampleUse = g::Msg<83, ()>;
pub type EvidenceCheck<'a, const DEST: u8> = g::Msg<85, imp::JsonTo<'a, DEST>>;
pub type RoundBrief<'a> = g::Msg<84, &'a [u8]>;
pub type DeepIntent<'a, const TOOL: u8> = g::Msg<9, imp::JsonTo<'a, TOOL>>;
pub type Identity<'a, const DEST: u8> = g::Msg<60, imp::JsonTo<'a, DEST>>;
pub type Formulation<'a, const DEST: u8> = g::Msg<61, imp::JsonTo<'a, DEST>>;
pub type Cells<'a, const DEST: u8> = g::Msg<62, imp::JsonTo<'a, DEST>>;
pub type Baseline<'a, const DEST: u8> = g::Msg<63, imp::JsonTo<'a, DEST>>;
pub type BioEvidence<'a, const DEST: u8> = g::Msg<64, imp::JsonTo<'a, DEST>>;
pub type PurityEvidence<'a, const DEST: u8> = g::Msg<65, imp::JsonTo<'a, DEST>>;
pub type StabilityEvidence<'a, const DEST: u8> = g::Msg<66, imp::JsonTo<'a, DEST>>;
pub type ToxEvidence<'a, const DEST: u8> = g::Msg<67, imp::JsonTo<'a, DEST>>;
pub type ClinicalEvidence<'a, const DEST: u8> = g::Msg<68, imp::JsonTo<'a, DEST>>;
pub type RegulatoryEvidence<'a, const DEST: u8> = g::Msg<69, imp::JsonTo<'a, DEST>>;
pub type DecisionEvidence<'a, const DEST: u8> = g::Msg<70, imp::JsonTo<'a, DEST>>;
pub type QcEvidence<'a> = g::Msg<71, imp::JsonTo<'a, MASS_SERVICE>>;
pub type DecisionRequest<'a> = g::Msg<72, imp::DecisionArguments<'a>>;
pub type DecisionResult<'a> = g::Msg<72, &'a [u8]>;
pub type Final<'a, const DEST: u8> = g::Msg<181, imp::FinalOutcome<'a, DEST>>;

pub(super) fn trial<
    const FILE: u8,
    const WHO: u8,
    const SERVICE: u8,
    const TOOL: u8,
    const CAP: u8,
    const STEP: u8,
>() -> impl Projectable {
    g::seq(
        g::send::<FILE, SERVICE, FileRequest<STEP>>(),
        requested::<WHO, SERVICE, TOOL, CAP, STEP>(),
    )
}
pub const INIT: u8 = 46;
pub const SESSION: u32 = 142;
pub const CHEMIST_IO: u8 = 7;
pub const CHEM_FILE: u8 = 32;
pub const CHEM_VM: u8 = 48;
pub const CHEM_DATA: u8 = 64;
pub const BIOLOGIST_IO: u8 = 8;
pub const BIO_FILE: u8 = 33;
pub const BIO_VM: u8 = 49;
pub const BIO_DATA: u8 = 65;
pub const TOXICOLOGIST_IO: u8 = 9;
pub const TOX_FILE: u8 = 34;
pub const TOX_VM: u8 = 50;
pub const TOX_DATA: u8 = 66;
pub const FORMULATOR_IO: u8 = 10;
pub const FORM_FILE: u8 = 35;
pub const FORM_VM: u8 = 51;
pub const FORM_DATA: u8 = 67;
pub const CLINICIAN_IO: u8 = 11;
pub const CLIN_FILE: u8 = 36;
pub const CLIN_VM: u8 = 52;
pub const CLIN_DATA: u8 = 68;
pub const REGULATOR_IO: u8 = 12;
pub const REG_FILE: u8 = 37;
pub const REG_VM: u8 = 53;
pub const REG_DATA: u8 = 69;
pub const DIRECTOR_IO: u8 = 13;
pub const DIR_FILE: u8 = 38;
pub const DIR_VM: u8 = 54;
pub const DIR_DATA: u8 = 70;

const fn request_file(path: &'static [u8], fd: u8) -> ChoreoFsObject<'static> {
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

const fn outgoing(path: &'static [u8], fd: u8) -> ChoreoFsObject<'static> {
    ChoreoFsObject::writable(
        path,
        ObjectId(fd as u32),
        FdSpec::new(fd, 64),
        protocol::FdBinding::write(protocol::FdWriteRow::Object),
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
    request_file(b"service/run_hplc_analysis/synthesis_qc", 4),
    request_file(b"service/run_mass_spec_analysis/molecular_id", 5),
    incoming(b"service/outcome", 6),
    request_file(b"service/run_hplc_analysis/documentation", 7),
];
pub const BIO_FILES: [ChoreoFsObject<'static>; 5] = [
    request_file(b"service/run_cell_assay/culture_prep", 4),
    request_file(b"service/run_cell_assay/bioassay", 5),
    request_file(b"service/run_hplc_analysis/stability", 6),
    outgoing(b"service/submit_evidence", 7),
    incoming(b"service/study", 8),
];
pub const TOX_FILES: [ChoreoFsObject<'static>; 5] = [
    request_file(b"service/run_mass_spec_analysis/baseline_markers", 4),
    request_file(b"service/run_hplc_analysis/purity_check", 5),
    request_file(b"service/run_cell_assay/cytotoxicity", 6),
    outgoing(b"service/submit_evidence", 7),
    incoming(b"service/study", 8),
];
pub const FORM_FILES: [ChoreoFsObject<'static>; 2] = [
    request_file(b"service/prepare_formulation", 4),
    incoming(b"service/study", 5),
];
pub const CLIN_FILES: [ChoreoFsObject<'static>; 2] = [
    request_file(b"service/review_clinical_data", 4),
    incoming(b"service/study", 5),
];
pub const REG_FILES: [ChoreoFsObject<'static>; 4] = [
    request_file(b"service/verify_regulatory_compliance/initial_review", 4),
    request_file(b"service/verify_regulatory_compliance/hold_review", 5),
    outgoing(b"service/submit_evidence", 6),
    incoming(b"service/study", 7),
];
pub const DIR_FILES: [ChoreoFsObject<'static>; 2] = [
    request_file(b"service/submit_project_decision", 4),
    incoming(b"service/study", 5),
];

pub fn discovery() -> impl Projectable {
    let files = g::par(
        g::send::<CHEM_DATA, CHEMIST, Loan<CHEM_FILE>>(),
        g::par(
            g::send::<BIO_DATA, BIOLOGIST, Loan<BIO_FILE>>(),
            g::par(
                g::send::<TOX_DATA, TOXICOLOGIST, Loan<TOX_FILE>>(),
                g::par(
                    g::send::<FORM_DATA, FORMULATOR, Loan<FORM_FILE>>(),
                    g::par(
                        g::send::<CLIN_DATA, CLINICIAN, Loan<CLIN_FILE>>(),
                        g::par(
                            g::send::<REG_DATA, REGULATOR, Loan<REG_FILE>>(),
                            g::send::<DIR_DATA, DIRECTOR, Loan<DIR_FILE>>(),
                        ),
                    ),
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
                    g::send::<INIT, FORMULATOR, Loan<FORM_VM>>(),
                    g::par(
                        g::send::<INIT, CLINICIAN, Loan<CLIN_VM>>(),
                        g::par(
                            g::send::<INIT, REGULATOR, Loan<REG_VM>>(),
                            g::send::<INIT, DIRECTOR, Loan<DIR_VM>>(),
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
                    file_metadata::<FORM_FILE, FORM_DATA>().roll(),
                    g::par(
                        file_metadata::<CLIN_FILE, CLIN_DATA>().roll(),
                        g::par(
                            file_metadata::<REG_FILE, REG_DATA>().roll(),
                            file_metadata::<DIR_FILE, DIR_DATA>().roll(),
                        ),
                    ),
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
                    g::send::<FORM_FILE, FORM_DATA, FileExit>(),
                    g::par(
                        g::send::<CLIN_FILE, CLIN_DATA, FileExit>(),
                        g::par(
                            g::send::<REG_FILE, REG_DATA, FileExit>(),
                            g::send::<DIR_FILE, DIR_DATA, FileExit>(),
                        ),
                    ),
                ),
            ),
        ),
    );
    let io = g::par(
        g::seq(
            private_io::<CHEM_VM, CHEMIST_IO>(),
            g::send::<CHEM_VM, CHEMIST_IO, Exit>(),
        ),
        g::par(
            g::seq(
                private_io::<BIO_VM, BIOLOGIST_IO>(),
                g::send::<BIO_VM, BIOLOGIST_IO, Exit>(),
            ),
            g::par(
                g::seq(
                    private_io::<TOX_VM, TOXICOLOGIST_IO>(),
                    g::send::<TOX_VM, TOXICOLOGIST_IO, Exit>(),
                ),
                g::par(
                    g::seq(
                        private_io::<FORM_VM, FORMULATOR_IO>(),
                        g::send::<FORM_VM, FORMULATOR_IO, Exit>(),
                    ),
                    g::par(
                        g::seq(
                            private_io::<CLIN_VM, CLINICIAN_IO>(),
                            g::send::<CLIN_VM, CLINICIAN_IO, Exit>(),
                        ),
                        g::par(
                            g::seq(
                                private_io::<REG_VM, REGULATOR_IO>(),
                                g::send::<REG_VM, REGULATOR_IO, Exit>(),
                            ),
                            g::seq(
                                private_io::<DIR_VM, DIRECTOR_IO>(),
                                g::send::<DIR_VM, DIRECTOR_IO, Exit>(),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );
    g::seq(
        g::par(files, guests),
        g::par(g::seq(g::par(source(), metadata), exits), io),
    )
}

pub struct Programs {
    pub chemist: hibana::runtime::program::RoleProgram<CHEMIST>,
    pub biologist: hibana::runtime::program::RoleProgram<BIOLOGIST>,
    pub toxicologist: hibana::runtime::program::RoleProgram<TOXICOLOGIST>,
    pub formulator: hibana::runtime::program::RoleProgram<FORMULATOR>,
    pub clinician: hibana::runtime::program::RoleProgram<CLINICIAN>,
    pub regulator: hibana::runtime::program::RoleProgram<REGULATOR>,
    pub director: hibana::runtime::program::RoleProgram<DIRECTOR>,
    pub hplc_service: hibana::runtime::program::RoleProgram<HPLC_SERVICE>,
    pub mass_service: hibana::runtime::program::RoleProgram<MASS_SERVICE>,
    pub cell_service: hibana::runtime::program::RoleProgram<CELL_SERVICE>,
    pub form_service: hibana::runtime::program::RoleProgram<FORM_SERVICE>,
    pub study_service: hibana::runtime::program::RoleProgram<STUDY_SERVICE>,
    pub hplc: hibana::runtime::program::RoleProgram<HPLC>,
    pub mass: hibana::runtime::program::RoleProgram<MASS>,
    pub cell: hibana::runtime::program::RoleProgram<CELL>,
    pub form: hibana::runtime::program::RoleProgram<FORM>,
    pub samples: hibana::runtime::program::RoleProgram<SAMPLES>,
    pub qc: hibana::runtime::program::RoleProgram<QC>,
    pub culture: hibana::runtime::program::RoleProgram<CULTURE>,
    pub formulation: hibana::runtime::program::RoleProgram<FORMULATION>,
    pub bioassay: hibana::runtime::program::RoleProgram<BIOASSAY>,
    pub stability: hibana::runtime::program::RoleProgram<STABILITY>,
    pub purity: hibana::runtime::program::RoleProgram<PURITY>,
    pub toxicity: hibana::runtime::program::RoleProgram<TOXICITY>,
    pub study: hibana::runtime::program::RoleProgram<STUDY>,
    pub init: hibana::runtime::program::RoleProgram<INIT>,
    pub chem_file: hibana::runtime::program::RoleProgram<CHEM_FILE>,
    pub chem_vm: hibana::runtime::program::RoleProgram<CHEM_VM>,
    pub chem_data: hibana::runtime::program::RoleProgram<CHEM_DATA>,
    pub chem_io: hibana::runtime::program::RoleProgram<CHEMIST_IO>,
    pub bio_file: hibana::runtime::program::RoleProgram<BIO_FILE>,
    pub bio_vm: hibana::runtime::program::RoleProgram<BIO_VM>,
    pub bio_data: hibana::runtime::program::RoleProgram<BIO_DATA>,
    pub bio_io: hibana::runtime::program::RoleProgram<BIOLOGIST_IO>,
    pub tox_file: hibana::runtime::program::RoleProgram<TOX_FILE>,
    pub tox_vm: hibana::runtime::program::RoleProgram<TOX_VM>,
    pub tox_data: hibana::runtime::program::RoleProgram<TOX_DATA>,
    pub tox_io: hibana::runtime::program::RoleProgram<TOXICOLOGIST_IO>,
    pub form_file: hibana::runtime::program::RoleProgram<FORM_FILE>,
    pub form_vm: hibana::runtime::program::RoleProgram<FORM_VM>,
    pub form_data: hibana::runtime::program::RoleProgram<FORM_DATA>,
    pub form_io: hibana::runtime::program::RoleProgram<FORMULATOR_IO>,
    pub clin_file: hibana::runtime::program::RoleProgram<CLIN_FILE>,
    pub clin_vm: hibana::runtime::program::RoleProgram<CLIN_VM>,
    pub clin_data: hibana::runtime::program::RoleProgram<CLIN_DATA>,
    pub clin_io: hibana::runtime::program::RoleProgram<CLINICIAN_IO>,
    pub reg_file: hibana::runtime::program::RoleProgram<REG_FILE>,
    pub reg_vm: hibana::runtime::program::RoleProgram<REG_VM>,
    pub reg_data: hibana::runtime::program::RoleProgram<REG_DATA>,
    pub reg_io: hibana::runtime::program::RoleProgram<REGULATOR_IO>,
    pub dir_file: hibana::runtime::program::RoleProgram<DIR_FILE>,
    pub dir_vm: hibana::runtime::program::RoleProgram<DIR_VM>,
    pub dir_data: hibana::runtime::program::RoleProgram<DIR_DATA>,
    pub dir_io: hibana::runtime::program::RoleProgram<DIRECTOR_IO>,
}
pub fn programs() -> Programs {
    use hibana::runtime::program::project;
    let source = discovery();
    Programs {
        chemist: project(&source),
        biologist: project(&source),
        toxicologist: project(&source),
        formulator: project(&source),
        clinician: project(&source),
        regulator: project(&source),
        director: project(&source),
        hplc_service: project(&source),
        mass_service: project(&source),
        cell_service: project(&source),
        form_service: project(&source),
        study_service: project(&source),
        hplc: project(&source),
        mass: project(&source),
        cell: project(&source),
        form: project(&source),
        samples: project(&source),
        qc: project(&source),
        culture: project(&source),
        formulation: project(&source),
        bioassay: project(&source),
        stability: project(&source),
        purity: project(&source),
        toxicity: project(&source),
        study: project(&source),
        init: project(&source),
        chem_file: project(&source),
        chem_vm: project(&source),
        chem_data: project(&source),
        chem_io: project(&source),
        bio_file: project(&source),
        bio_vm: project(&source),
        bio_data: project(&source),
        bio_io: project(&source),
        tox_file: project(&source),
        tox_vm: project(&source),
        tox_data: project(&source),
        tox_io: project(&source),
        form_file: project(&source),
        form_vm: project(&source),
        form_data: project(&source),
        form_io: project(&source),
        clin_file: project(&source),
        clin_vm: project(&source),
        clin_data: project(&source),
        clin_io: project(&source),
        reg_file: project(&source),
        reg_vm: project(&source),
        reg_data: project(&source),
        reg_io: project(&source),
        dir_file: project(&source),
        dir_vm: project(&source),
        dir_data: project(&source),
        dir_io: project(&source),
    }
}

impl Programs {
    pub fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        let roles = [
            ("CHEMIST", CHEMIST),
            ("BIOLOGIST", BIOLOGIST),
            ("TOXICOLOGIST", TOXICOLOGIST),
            ("FORMULATION_SCIENTIST", FORMULATOR),
            ("CLINICAL_LEAD", CLINICIAN),
            ("REGULATORY_SPECIALIST", REGULATOR),
            ("PROJECT_DIRECTOR", DIRECTOR),
            ("HPLC_SERVICE", HPLC_SERVICE),
            ("MASS_SERVICE", MASS_SERVICE),
            ("CELL_SERVICE", CELL_SERVICE),
            ("FORM_SERVICE", FORM_SERVICE),
            ("STUDY_SERVICE", STUDY_SERVICE),
            ("HPLC", HPLC),
            ("MASS_SPEC", MASS),
            ("CELL_LAB", CELL),
            ("FORMULATION_SUITE", FORM),
            ("BIOLOGICAL_SAMPLES", SAMPLES),
            ("QC", QC),
            ("CULTURE", CULTURE),
            ("FORMULATION", FORMULATION),
            ("BIOASSAY", BIOASSAY),
            ("STABILITY", STABILITY),
            ("PURITY", PURITY),
            ("TOXICITY", TOXICITY),
            ("STUDY", STUDY),
            ("INIT", INIT),
            ("CHEM_FILE", CHEM_FILE),
            ("CHEM_VM", CHEM_VM),
            ("CHEM_DATA", CHEM_DATA),
            ("CHEMIST_IO", CHEMIST_IO),
            ("BIO_FILE", BIO_FILE),
            ("BIO_VM", BIO_VM),
            ("BIO_DATA", BIO_DATA),
            ("BIOLOGIST_IO", BIOLOGIST_IO),
            ("TOX_FILE", TOX_FILE),
            ("TOX_VM", TOX_VM),
            ("TOX_DATA", TOX_DATA),
            ("TOXICOLOGIST_IO", TOXICOLOGIST_IO),
            ("FORM_FILE", FORM_FILE),
            ("FORM_VM", FORM_VM),
            ("FORM_DATA", FORM_DATA),
            ("FORMULATOR_IO", FORMULATOR_IO),
            ("CLIN_FILE", CLIN_FILE),
            ("CLIN_VM", CLIN_VM),
            ("CLIN_DATA", CLIN_DATA),
            ("CLINICIAN_IO", CLINICIAN_IO),
            ("REG_FILE", REG_FILE),
            ("REG_VM", REG_VM),
            ("REG_DATA", REG_DATA),
            ("REGULATOR_IO", REGULATOR_IO),
            ("DIR_FILE", DIR_FILE),
            ("DIR_VM", DIR_VM),
            ("DIR_DATA", DIR_DATA),
            ("DIRECTOR_IO", DIRECTOR_IO),
        ]
        .into_iter()
        .collect::<std::collections::BTreeMap<_, _>>();
        let programs = [
            rows(&self.chemist),
            rows(&self.biologist),
            rows(&self.toxicologist),
            rows(&self.formulator),
            rows(&self.clinician),
            rows(&self.regulator),
            rows(&self.director),
            rows(&self.hplc_service),
            rows(&self.mass_service),
            rows(&self.cell_service),
            rows(&self.form_service),
            rows(&self.study_service),
            rows(&self.hplc),
            rows(&self.mass),
            rows(&self.cell),
            rows(&self.form),
            rows(&self.samples),
            rows(&self.qc),
            rows(&self.culture),
            rows(&self.formulation),
            rows(&self.bioassay),
            rows(&self.stability),
            rows(&self.purity),
            rows(&self.toxicity),
            rows(&self.study),
            rows(&self.init),
            rows(&self.chem_file),
            rows(&self.chem_vm),
            rows(&self.chem_data),
            rows(&self.chem_io),
            rows(&self.bio_file),
            rows(&self.bio_vm),
            rows(&self.bio_data),
            rows(&self.bio_io),
            rows(&self.tox_file),
            rows(&self.tox_vm),
            rows(&self.tox_data),
            rows(&self.tox_io),
            rows(&self.form_file),
            rows(&self.form_vm),
            rows(&self.form_data),
            rows(&self.form_io),
            rows(&self.clin_file),
            rows(&self.clin_vm),
            rows(&self.clin_data),
            rows(&self.clin_io),
            rows(&self.reg_file),
            rows(&self.reg_vm),
            rows(&self.reg_data),
            rows(&self.reg_io),
            rows(&self.dir_file),
            rows(&self.dir_vm),
            rows(&self.dir_data),
            rows(&self.dir_io),
        ];
        serde_json::json!({
            "type":"compiled_projection","format_version":1,"task":"14H","session":SESSION,
            "roles":roles, "programs":programs.as_slice(),
            "labels": {
                "SynthesisQc":0,"CulturePrep":1,"Baseline":2,"MolecularId":3,"Formulation":4,
                "Bioassay":5,"Stability":6,"Purity":7,"Cytotoxicity":8,"Clinical":9,
                "RoutineReview":10,"DeepReview":11,"Decision":72,"Documentation":15,
                "Passed":3,"Failed":4,"StudyFile":82,"Publish":81,"SampleUse":83,
            },
        })
    }
}
fn requested<const WHO: u8, const SERVICE: u8, const TOOL: u8, const CAP: u8, const STEP: u8>()
-> impl Projectable {
    g::seq(
        g::send::<SERVICE, WHO, Instrument<TOOL>>(),
        g::seq(
            g::send::<TOOL, SERVICE, Use<STEP>>(),
            g::seq(
                g::send::<WHO, SERVICE, Instrument<TOOL>>(),
                g::send::<WHO, SERVICE, Specimen<CAP>>(),
            ),
        ),
    )
}
pub(super) fn passed<
    const FILE: u8,
    const WHO: u8,
    const SERVICE: u8,
    const CAP: u8,
    const STEP: u8,
    const DEST: u8,
>() -> impl Projectable {
    g::seq(
        g::send::<CAP, WHO, Passed<STEP>>(),
        g::seq(
            g::send::<WHO, FILE, FileResult<STEP>>(),
            g::send::<SERVICE, DEST, Specimen<CAP>>(),
        ),
    )
}
pub(super) fn positive<
    const FILE: u8,
    const WHO: u8,
    const SERVICE: u8,
    const TOOL: u8,
    const CAP: u8,
    const STEP: u8,
    const DEST: u8,
>() -> impl Projectable {
    g::seq(
        trial::<FILE, WHO, SERVICE, TOOL, CAP, STEP>(),
        positive_result::<FILE, WHO, SERVICE, TOOL, CAP, STEP, DEST>(),
    )
}
fn positive_result<
    const FILE: u8,
    const WHO: u8,
    const SERVICE: u8,
    const TOOL: u8,
    const CAP: u8,
    const STEP: u8,
    const DEST: u8,
>() -> impl Projectable {
    let retry = g::seq(
        g::send::<CAP, WHO, Failed<STEP>>(),
        g::seq(
            g::send::<WHO, FILE, FileResult<STEP>>(),
            g::seq(
                g::send::<SERVICE, WHO, Specimen<CAP>>(),
                trial::<FILE, WHO, SERVICE, TOOL, CAP, STEP>(),
            ),
        ),
    );
    g::route(
        passed::<FILE, WHO, SERVICE, CAP, STEP, DEST>(),
        g::seq(
            retry.roll(),
            passed::<FILE, WHO, SERVICE, CAP, STEP, DEST>(),
        ),
    )
}

pub(super) fn mass<const FILE: u8, const WHO: u8, const STEP: u8>() -> impl Projectable {
    g::seq(
        g::send::<FILE, MASS_SERVICE, FileRequest<STEP>>(),
        g::seq(
            g::send::<MASS_SERVICE, WHO, Instrument<MASS>>(),
            g::seq(
                g::send::<MASS, MASS_SERVICE, Use<STEP>>(),
                g::seq(
                    g::send::<MASS_SERVICE, MASS, Passed<STEP>>(),
                    g::seq(
                        g::send::<WHO, MASS_SERVICE, Instrument<MASS>>(),
                        g::send::<WHO, FILE, FileResult<STEP>>(),
                    ),
                ),
            ),
        ),
    )
}

/// Reconsider the same material without manufacturing another sample. The
/// scientist retains its specimen between attempts and returns it to finish;
/// only that final transfer releases the result to clinical assessment.
pub(super) fn assay<const FILE: u8, const WHO: u8, const CAP: u8, const STEP: u8>()
-> impl Projectable {
    let attempt = g::seq(
        g::send::<FILE, CELL_SERVICE, FileRequest<STEP>>(),
        g::seq(
            g::par(
                g::send::<CELL_SERVICE, WHO, Instrument<CELL>>(),
                g::send::<CELL_SERVICE, WHO, Instrument<SAMPLES>>(),
            ),
            g::seq(
                g::par(
                    g::send::<CELL, CELL_SERVICE, Use<STEP>>(),
                    g::send::<SAMPLES, CELL_SERVICE, SampleUse>(),
                ),
                g::seq(
                    g::par(
                        g::send::<WHO, CELL_SERVICE, Instrument<SAMPLES>>(),
                        g::send::<WHO, CELL_SERVICE, Instrument<CELL>>(),
                    ),
                    g::seq(
                        g::send::<WHO, CELL_SERVICE, Specimen<CAP>>(),
                        g::seq(
                            g::route(
                                g::send::<CAP, WHO, Passed<STEP>>(),
                                g::send::<CAP, WHO, Failed<STEP>>(),
                            ),
                            g::seq(
                                g::send::<WHO, FILE, FileResult<STEP>>(),
                                g::send::<CELL_SERVICE, WHO, Specimen<CAP>>(),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );
    g::seq(
        attempt.roll(),
        g::seq(
            g::send::<FILE, WHO, Publish<STEP>>(),
            g::seq(
                g::send::<WHO, CELL_SERVICE, Specimen<CAP>>(),
                g::send::<CELL_SERVICE, CLINICIAN, Specimen<CAP>>(),
            ),
        ),
    )
}

pub(super) fn formulation() -> impl Projectable {
    g::seq(
        g::seq(
            requested::<FORMULATOR, FORM_SERVICE, FORM, FORMULATION, 4>(),
            positive_result::<FORM_FILE, FORMULATOR, FORM_SERVICE, FORM, FORMULATION, 4, CLINICIAN>(
            ),
        ),
        g::par(
            g::send::<FORM_SERVICE, HPLC_SERVICE, Formulation<HPLC_SERVICE>>(),
            g::par(
                g::send::<FORM_SERVICE, CELL_SERVICE, Formulation<CELL_SERVICE>>(),
                g::par(
                    g::send::<FORM_SERVICE, MASS_SERVICE, Formulation<MASS_SERVICE>>(),
                    g::par(
                        g::send::<FORM_SERVICE, STUDY_SERVICE, Formulation<STUDY_SERVICE>>(),
                        g::par(
                            g::send::<FORM_SERVICE, BIO_FILE, StudyFile<BIO_FILE>>(),
                            g::send::<FORM_SERVICE, TOX_FILE, StudyFile<TOX_FILE>>(),
                        ),
                    ),
                ),
            ),
        ),
    )
}

fn evaluation() -> impl Projectable {
    let bio = g::seq(
        assay::<BIO_FILE, BIOLOGIST, BIOASSAY, 5>(),
        g::send::<CELL_SERVICE, MASS_SERVICE, BioEvidence<MASS_SERVICE>>(),
    );
    let purity = g::seq(
        positive::<TOX_FILE, TOXICOLOGIST, HPLC_SERVICE, HPLC, PURITY, 7, CLINICIAN>(),
        g::send::<HPLC_SERVICE, MASS_SERVICE, PurityEvidence<MASS_SERVICE>>(),
    );
    let stability = g::seq(
        positive::<BIO_FILE, BIOLOGIST, HPLC_SERVICE, HPLC, STABILITY, 6, CLINICIAN>(),
        g::send::<HPLC_SERVICE, MASS_SERVICE, StabilityEvidence<MASS_SERVICE>>(),
    );
    let tox = g::seq(
        g::send::<TOXICOLOGIST, CELL_SERVICE, PurityEvidence<CELL_SERVICE>>(),
        g::seq(
            assay::<TOX_FILE, TOXICOLOGIST, TOXICITY, 8>(),
            g::send::<CELL_SERVICE, MASS_SERVICE, ToxEvidence<MASS_SERVICE>>(),
        ),
    );
    // The toxicologist forwards its actual purity result before toxicity.
    // Actors absent from the purity prefix may start their independent work:
    // BIOLOGIST's assay does not wait for purity, and stability does not join it.
    g::seq(purity, g::par(g::seq(bio, tox), stability))
}

fn clinical() -> impl Projectable {
    let inputs = g::par(
        g::send::<CLINICIAN, DIRECTOR, Specimen<FORMULATION>>(),
        g::par(
            g::send::<CLINICIAN, DIRECTOR, Specimen<BIOASSAY>>(),
            g::par(
                g::send::<CLINICIAN, DIRECTOR, Specimen<STABILITY>>(),
                g::par(
                    g::send::<CLINICIAN, DIRECTOR, Specimen<PURITY>>(),
                    g::send::<CLINICIAN, DIRECTOR, Specimen<TOXICITY>>(),
                ),
            ),
        ),
    );
    g::seq(
        g::seq(
            g::send::<MASS_SERVICE, CLIN_FILE, StudyFile<CLIN_FILE>>(),
            trial::<CLIN_FILE, CLINICIAN, MASS_SERVICE, MASS, STUDY, 9>(),
        ),
        g::seq(
            passed::<CLIN_FILE, CLINICIAN, MASS_SERVICE, STUDY, 9, REGULATOR>(),
            g::par(
                inputs,
                g::par(
                    g::seq(
                        g::send::<MASS_SERVICE, REGULATOR, ClinicalEvidence<REGULATOR>>(),
                        g::send::<REGULATOR, REG_FILE, StudyFile<REG_FILE>>(),
                    ),
                    g::send::<MASS_SERVICE, STUDY_SERVICE, ClinicalEvidence<STUDY_SERVICE>>(),
                ),
            ),
        ),
    )
}

pub(super) fn review() -> impl Projectable {
    let initial = g::seq(
        g::send::<REG_FILE, STUDY_SERVICE, FileRequest<10>>(),
        g::seq(
            g::par(
                g::seq(
                    g::send::<STUDY_SERVICE, HPLC_SERVICE, EvidenceCheck<HPLC_SERVICE>>(),
                    g::seq(
                        g::send::<HPLC_SERVICE, STUDY_SERVICE, PurityEvidence<STUDY_SERVICE>>(),
                        g::send::<HPLC_SERVICE, STUDY_SERVICE, StabilityEvidence<STUDY_SERVICE>>(),
                    ),
                ),
                g::seq(
                    g::send::<STUDY_SERVICE, MASS_SERVICE, EvidenceCheck<MASS_SERVICE>>(),
                    g::send::<MASS_SERVICE, STUDY_SERVICE, ClinicalEvidence<STUDY_SERVICE>>(),
                ),
            ),
            g::seq(
                g::send::<STUDY_SERVICE, REGULATOR, FileRequest<10>>(),
                g::seq(
                    g::send::<REGULATOR, STUDY_SERVICE, Specimen<STUDY>>(),
                    g::seq(
                        g::route(
                            g::send::<STUDY, REGULATOR, Passed<10>>(),
                            g::send::<STUDY, REGULATOR, Failed<10>>(),
                        ),
                        g::seq(
                            g::send::<REGULATOR, REG_FILE, FileResult<10>>(),
                            g::send::<STUDY_SERVICE, REGULATOR, Specimen<STUDY>>(),
                        ),
                    ),
                ),
            ),
        ),
    );
    let acquired = g::seq(
        g::send::<STUDY, HPLC_SERVICE, DeepIntent<HPLC>>(),
        g::seq(
            g::send::<HPLC_SERVICE, REGULATOR, Instrument<HPLC>>(),
            g::seq(
                g::send::<STUDY, MASS_SERVICE, DeepIntent<MASS>>(),
                g::send::<MASS_SERVICE, REGULATOR, Instrument<MASS>>(),
            ),
        ),
    );
    let released = g::seq(
        g::send::<REGULATOR, MASS_SERVICE, Instrument<MASS>>(),
        g::send::<REGULATOR, HPLC_SERVICE, Instrument<HPLC>>(),
    );
    // The effect receiver owns both physical instruments while reviewing.
    // A received request alone does not retain an instrument after its return.
    let reviewing = g::seq(
        g::par(
            g::send::<REGULATOR, STUDY_SERVICE, Instrument<HPLC>>(),
            g::send::<REGULATOR, STUDY_SERVICE, Instrument<MASS>>(),
        ),
        g::seq(
            g::send::<REGULATOR, STUDY_SERVICE, Specimen<STUDY>>(),
            g::seq(
                g::route(
                    g::send::<STUDY, REGULATOR, Passed<11>>(),
                    g::send::<STUDY, REGULATOR, Failed<11>>(),
                ),
                g::seq(
                    g::send::<STUDY_SERVICE, REGULATOR, Specimen<STUDY>>(),
                    g::seq(
                        g::par(
                            g::send::<STUDY_SERVICE, REGULATOR, Instrument<HPLC>>(),
                            g::send::<STUDY_SERVICE, REGULATOR, Instrument<MASS>>(),
                        ),
                        released,
                    ),
                ),
            ),
        ),
    );
    let deep = g::seq(
        g::send::<REG_FILE, STUDY_SERVICE, FileRequest<11>>(),
        g::seq(
            g::seq(
                g::send::<STUDY_SERVICE, REGULATOR, FileRequest<11>>(),
                acquired,
            ),
            g::seq(
                g::par(
                    g::send::<HPLC, STUDY_SERVICE, Use<11>>(),
                    g::send::<MASS, STUDY_SERVICE, Use<11>>(),
                ),
                g::seq(reviewing, g::send::<REGULATOR, REG_FILE, FileResult<11>>()),
            ),
        ),
    );
    g::route(initial, deep)
}

fn round_entry() -> impl Projectable {
    g::seq(
        g::send::<STUDY, FORMULATOR, RoundBrief>(),
        g::seq(
            g::send::<STUDY, FORMULATOR, Specimen<FORMULATION>>(),
            g::seq(
                g::send::<FORMULATOR, FORM_FILE, StudyFile<FORM_FILE>>(),
                g::send::<FORM_FILE, FORM_SERVICE, FileRequest<4>>(),
            ),
        ),
    )
}

/// The effect owner receives its actual formulation before releasing the
/// material's Endpoint. Regaining a material must not authorize an old inner
/// assay while its next formulation is still being prepared.
pub(super) fn material<const WHO: u8, const SERVICE: u8, const CAP: u8>() -> impl Projectable {
    g::seq(
        g::send::<DIRECTOR, SERVICE, Specimen<CAP>>(),
        g::send::<SERVICE, WHO, Specimen<CAP>>(),
    )
}

fn round() -> impl Projectable {
    let reviewed = g::seq(
        review().roll(),
        g::seq(
            g::send::<REG_FILE, REGULATOR, Publish<10>>(),
            g::seq(
                g::send::<REGULATOR, STUDY_SERVICE, Specimen<STUDY>>(),
                g::seq(
                    g::send::<STUDY, DIRECTOR, RegulatoryEvidence<DIRECTOR>>(),
                    g::send::<DIRECTOR, DIR_FILE, StudyFile<DIR_FILE>>(),
                ),
            ),
        ),
    );
    // The same study effect owner holds clinical evidence and its real review
    // result. No second owner or copied authority is needed for the decision.
    let decision = g::seq(
        g::send::<DIR_FILE, STUDY_SERVICE, DecisionRequest>(),
        g::seq(
            g::send::<STUDY, DIRECTOR, DecisionEvidence<DIRECTOR>>(),
            g::seq(
                g::send::<DIRECTOR, DIR_FILE, DecisionResult>(),
                g::send::<STUDY_SERVICE, DIRECTOR, Specimen<STUDY>>(),
            ),
        ),
    );
    let materials = g::par(
        material::<BIOLOGIST, CELL_SERVICE, BIOASSAY>(),
        g::par(
            material::<BIOLOGIST, HPLC_SERVICE, STABILITY>(),
            g::par(
                material::<TOXICOLOGIST, HPLC_SERVICE, PURITY>(),
                material::<TOXICOLOGIST, CELL_SERVICE, TOXICITY>(),
            ),
        ),
    );
    g::seq(
        formulation(),
        g::seq(
            materials,
            g::seq(
                g::send::<DIRECTOR, CLINICIAN, Specimen<STUDY>>(),
                g::seq(evaluation(), g::seq(clinical(), g::seq(reviewed, decision))),
            ),
        ),
    )
}

pub(super) fn terminal() -> impl Projectable {
    let archived = g::par(
        g::send::<DIRECTOR, HPLC_SERVICE, Archived<FORMULATION>>(),
        g::par(
            g::send::<DIRECTOR, HPLC_SERVICE, Archived<BIOASSAY>>(),
            g::par(
                g::send::<DIRECTOR, HPLC_SERVICE, Archived<STABILITY>>(),
                g::par(
                    g::send::<DIRECTOR, HPLC_SERVICE, Archived<PURITY>>(),
                    g::send::<DIRECTOR, HPLC_SERVICE, Archived<TOXICITY>>(),
                ),
            ),
        ),
    );
    let outcome = g::par(
        g::seq(
            g::send::<DIRECTOR, CHEMIST, Final<CHEMIST>>(),
            g::send::<CHEMIST, CHEM_FILE, StudyFile<CHEM_FILE>>(),
        ),
        g::par(
            g::seq(
                g::send::<DIRECTOR, BIOLOGIST, Final<BIOLOGIST>>(),
                g::send::<BIOLOGIST, BIO_FILE, StudyFile<BIO_FILE>>(),
            ),
            g::par(
                g::seq(
                    g::send::<DIRECTOR, TOXICOLOGIST, Final<TOXICOLOGIST>>(),
                    g::send::<TOXICOLOGIST, TOX_FILE, StudyFile<TOX_FILE>>(),
                ),
                g::par(
                    g::seq(
                        g::send::<DIRECTOR, FORMULATOR, Final<FORMULATOR>>(),
                        g::send::<FORMULATOR, FORM_FILE, StudyFile<FORM_FILE>>(),
                    ),
                    g::par(
                        g::seq(
                            g::send::<DIRECTOR, CLINICIAN, Final<CLINICIAN>>(),
                            g::send::<CLINICIAN, CLIN_FILE, StudyFile<CLIN_FILE>>(),
                        ),
                        g::par(
                            g::seq(
                                g::send::<DIRECTOR, REGULATOR, Final<REGULATOR>>(),
                                g::send::<REGULATOR, REG_FILE, StudyFile<REG_FILE>>(),
                            ),
                            g::par(
                                g::send::<DIRECTOR, HPLC_SERVICE, Final<HPLC_SERVICE>>(),
                                g::par(
                                    g::send::<DIRECTOR, MASS_SERVICE, Final<MASS_SERVICE>>(),
                                    g::par(
                                        g::send::<DIRECTOR, CELL_SERVICE, Final<CELL_SERVICE>>(),
                                        g::par(
                                            g::send::<DIRECTOR, FORM_SERVICE, Final<FORM_SERVICE>>(
                                            ),
                                            g::send::<DIRECTOR, STUDY_SERVICE, Final<STUDY_SERVICE>>(
                                            ),
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );
    // Retire the study's entry authority before announcing a terminal result.
    // The material archives remain independent operations in parallel.
    g::seq(
        g::send::<DIRECTOR, CHEMIST, Specimen<STUDY>>(),
        g::seq(
            archived,
            g::seq(
                outcome,
                g::seq(
                    g::send::<STUDY_SERVICE, HPLC_SERVICE, DecisionEvidence<HPLC_SERVICE>>(),
                    positive::<CHEM_FILE, CHEMIST, HPLC_SERVICE, HPLC, STUDY, 15, MASS_SERVICE>(),
                ),
            ),
        ),
    )
}

pub fn source() -> impl Projectable {
    let qc = g::seq(
        positive::<CHEM_FILE, CHEMIST, HPLC_SERVICE, HPLC, QC, 0, FORMULATOR>(),
        g::send::<HPLC_SERVICE, MASS_SERVICE, QcEvidence>(),
    );
    let baseline = g::seq(
        mass::<TOX_FILE, TOXICOLOGIST, 2>(),
        g::send::<MASS_SERVICE, CELL_SERVICE, Baseline<CELL_SERVICE>>(),
    );
    let identity = g::seq(
        mass::<CHEM_FILE, CHEMIST, 3>(),
        g::par(
            g::send::<MASS_SERVICE, DIRECTOR, Identity<DIRECTOR>>(),
            g::par(
                g::send::<MASS_SERVICE, FORM_SERVICE, Identity<FORM_SERVICE>>(),
                g::send::<MASS_SERVICE, STUDY_SERVICE, Identity<STUDY_SERVICE>>(),
            ),
        ),
    );
    let culture = g::seq(
        positive::<BIO_FILE, BIOLOGIST, CELL_SERVICE, CELL, CULTURE, 1, CLINICIAN>(),
        g::send::<CELL_SERVICE, HPLC_SERVICE, Cells<HPLC_SERVICE>>(),
    );
    let initial = g::par(g::seq(g::par(qc, baseline), identity), culture);
    g::seq(
        initial,
        g::seq(g::seq(round_entry(), round()).roll(), terminal()),
    )
}
