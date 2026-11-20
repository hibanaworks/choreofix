//! Four scientists, three affine instrument views, and one decision receipt.
pub use super::arguments::*;
pub use crate::shell::global::{Exit, FileExit, Loan, file_metadata, private_io};
use hibana::{g, runtime::program::Projectable};
use hibana_wasip1_runtime::{
    choreofs::{ChoreoFsObject, FdSpec, ObjectId},
    protocol,
};

pub const CHEMIST: u8 = 0;
pub const BIOLOGIST: u8 = 1;
pub const TOXICOLOGIST: u8 = 2;
pub const LEAD: u8 = 3;
pub const HPLC_OWNER: u8 = 4;
pub const MASS_OWNER: u8 = 5;
pub const CELL_OWNER: u8 = 6;
pub const HPLC: u8 = 7;
pub const MASS: u8 = 8;
pub const CELL: u8 = 9;
pub const INIT: u8 = 10;

pub const CHEM_FILE: u8 = 32;
pub const BIO_FILE: u8 = 33;
pub const TOX_FILE: u8 = 34;
pub const LEAD_FILE: u8 = 35;
pub const CHEM_VM: u8 = 64;
pub const BIO_VM: u8 = 65;
pub const TOX_VM: u8 = 66;
pub const LEAD_VM: u8 = 67;
pub const CHEM_IO: u8 = 80;
pub const BIO_IO: u8 = 81;
pub const TOX_IO: u8 = 82;
pub const LEAD_IO: u8 = 83;
pub const CHEM_DATA: u8 = 96;
pub const BIO_DATA: u8 = 97;
pub const TOX_DATA: u8 = 98;
pub const LEAD_DATA: u8 = 99;
pub const SESSION: u32 = 140;

pub type Request<const OP: u8> = g::Msg<OP, Arguments<OP>>;
pub type Use<const OP: u8> = g::Msg<OP, ()>;
pub type Reply<'a, const OP: u8> = g::Msg<OP, &'a [u8]>;
pub type BiologicalSample<'a> = g::Msg<50, &'a [u8]>;
pub type ToxicologySample<'a> = g::Msg<54, &'a [u8]>;
pub type BioReport<'a> = g::Msg<51, &'a [u8]>;
pub type ToxReport<'a> = g::Msg<52, &'a [u8]>;
pub type DecisionReceipt<'a> = g::Msg<53, &'a [u8]>;

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
    incoming(b"service/decision", 6),
    operation(b"service/run_hplc_analysis/documentation", 7),
];
pub const BIO_FILES: [ChoreoFsObject<'static>; 4] = [
    operation(b"service/run_cell_assay/culture_prep", 4),
    incoming(b"service/compound", 5),
    operation(b"service/run_cell_assay/bioassay", 6),
    operation(b"service/run_hplc_analysis/stability", 7),
];
pub const TOX_FILES: [ChoreoFsObject<'static>; 4] = [
    operation(b"service/run_mass_spec_analysis/baseline_markers", 4),
    incoming(b"service/compound", 5),
    operation(b"service/run_hplc_analysis/purity_check", 6),
    operation(b"service/run_cell_assay/cytotoxicity", 7),
];
pub const LEAD_FILES: [ChoreoFsObject<'static>; 6] = [
    operation(b"service/run_mass_spec_analysis/reference_methods", 4),
    operation(b"service/run_hplc_analysis/structural_confirmation", 5),
    operation(b"service/run_mass_spec_analysis/characterization", 6),
    operation(b"service/submit_decision", 7),
    incoming(b"service/bioactivity", 8),
    incoming(b"service/toxicity", 9),
];

fn instrument<
    const OP: u8,
    const FILE: u8,
    const SCIENTIST: u8,
    const OWNER: u8,
    const RESOURCE: u8,
>() -> impl Projectable {
    g::seq(
        g::send::<FILE, OWNER, Request<OP>>(),
        use_instrument::<OP, FILE, SCIENTIST, OWNER, RESOURCE>(),
    )
}

fn use_instrument<
    const OP: u8,
    const FILE: u8,
    const SCIENTIST: u8,
    const OWNER: u8,
    const RESOURCE: u8,
>() -> impl Projectable {
    g::seq(
        g::send::<OWNER, SCIENTIST, Loan<RESOURCE>>(),
        g::seq(
            g::send::<RESOURCE, OWNER, Use<OP>>(),
            g::seq(
                g::send::<OWNER, RESOURCE, Reply<OP>>(),
                g::seq(
                    g::send::<SCIENTIST, OWNER, Loan<RESOURCE>>(),
                    g::send::<SCIENTIST, FILE, Reply<OP>>(),
                ),
            ),
        ),
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
                g::send::<LEAD_DATA, LEAD, Loan<LEAD_FILE>>(),
            ),
        ),
    );
    let guests = g::par(
        g::send::<INIT, CHEMIST, Loan<CHEM_VM>>(),
        g::par(
            g::send::<INIT, BIOLOGIST, Loan<BIO_VM>>(),
            g::par(
                g::send::<INIT, TOXICOLOGIST, Loan<TOX_VM>>(),
                g::send::<INIT, LEAD, Loan<LEAD_VM>>(),
            ),
        ),
    );

    let mass_setup = g::seq(
        g::par(
            instrument::<BASELINE, TOX_FILE, TOXICOLOGIST, MASS_OWNER, MASS>(),
            g::send::<LEAD_FILE, MASS_OWNER, Request<REFERENCE>>(),
        ),
        use_instrument::<REFERENCE, LEAD_FILE, LEAD, MASS_OWNER, MASS>(),
    );
    let prepare = g::par(
        instrument::<SYNTHESIS_QC, CHEM_FILE, CHEMIST, HPLC_OWNER, HPLC>(),
        g::par(
            g::seq(
                g::send::<BIO_FILE, CELL_OWNER, Request<CULTURE_PREP>>(),
                g::seq(
                    g::send::<CELL_OWNER, BIOLOGIST, Loan<CELL>>(),
                    g::seq(
                        g::send::<CELL, CELL_OWNER, Use<CULTURE_PREP>>(),
                        g::seq(
                            g::send::<CELL_OWNER, CELL, Reply<CULTURE_PREP>>(),
                            g::send::<BIOLOGIST, BIO_FILE, Reply<CULTURE_PREP>>(),
                        ),
                    ),
                ),
            ),
            mass_setup,
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
    let assay = g::seq(
        g::send::<BIO_FILE, CELL_OWNER, Request<BIOASSAY>>(),
        g::seq(
            g::send::<CELL, CELL_OWNER, Use<BIOASSAY>>(),
            g::seq(
                g::send::<CELL_OWNER, CELL, Reply<BIOASSAY>>(),
                g::send::<BIOLOGIST, BIO_FILE, Reply<BIOASSAY>>(),
            ),
        ),
    );
    let evaluations = g::seq(
        g::par(
            assay,
            instrument::<PURITY, TOX_FILE, TOXICOLOGIST, HPLC_OWNER, HPLC>(),
        ),
        g::seq(
            instrument::<STABILITY, BIO_FILE, BIOLOGIST, HPLC_OWNER, HPLC>(),
            g::seq(
                g::send::<BIOLOGIST, CELL_OWNER, Loan<CELL>>(),
                g::par(
                    g::seq(
                        g::send::<BIOLOGIST, LEAD, BioReport>(),
                        g::send::<LEAD, LEAD_FILE, BioReport>(),
                    ),
                    g::seq(
                        instrument::<CYTOTOXICITY, TOX_FILE, TOXICOLOGIST, CELL_OWNER, CELL>(),
                        g::seq(
                            g::send::<TOXICOLOGIST, LEAD, ToxReport>(),
                            g::send::<LEAD, LEAD_FILE, ToxReport>(),
                        ),
                    ),
                ),
            ),
        ),
    );
    let decide = g::seq(
        g::par(
            instrument::<STRUCTURE, LEAD_FILE, LEAD, HPLC_OWNER, HPLC>(),
            instrument::<CHARACTERIZATION, LEAD_FILE, LEAD, MASS_OWNER, MASS>(),
        ),
        g::seq(
            g::send::<LEAD_FILE, LEAD, Request<DECISION>>(),
            g::par(
                g::send::<LEAD, LEAD_FILE, Reply<DECISION>>(),
                g::seq(
                    g::send::<LEAD, CHEMIST, DecisionReceipt>(),
                    g::send::<CHEMIST, CHEM_FILE, DecisionReceipt>(),
                ),
            ),
        ),
    );
    let work = g::seq(
        prepare,
        g::seq(
            instrument::<MOLECULAR_ID, CHEM_FILE, CHEMIST, MASS_OWNER, MASS>(),
            g::seq(
                distribute,
                g::seq(
                    evaluations,
                    g::seq(
                        decide,
                        instrument::<DOCUMENTATION, CHEM_FILE, CHEMIST, HPLC_OWNER, HPLC>(),
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
                file_metadata::<LEAD_FILE, LEAD_DATA>().roll(),
            ),
        ),
    );
    let file_exits = g::par(
        g::send::<CHEM_FILE, CHEM_DATA, FileExit>(),
        g::par(
            g::send::<BIO_FILE, BIO_DATA, FileExit>(),
            g::par(
                g::send::<TOX_FILE, TOX_DATA, FileExit>(),
                g::send::<LEAD_FILE, LEAD_DATA, FileExit>(),
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
                g::seq(
                    private_io::<LEAD_VM, LEAD_IO>(),
                    g::send::<LEAD_VM, LEAD_IO, Exit>(),
                ),
            ),
        ),
    );
    g::seq(
        g::par(instruments, g::par(files, guests)),
        g::par(g::seq(g::par(work, metadata), file_exits), io),
    )
}

/// The images reported by --projection are the same images passed to attach.
pub struct Programs {
    pub chemist: hibana::runtime::program::RoleProgram<CHEMIST>,
    pub biologist: hibana::runtime::program::RoleProgram<BIOLOGIST>,
    pub toxicologist: hibana::runtime::program::RoleProgram<TOXICOLOGIST>,
    pub lead: hibana::runtime::program::RoleProgram<LEAD>,
    pub hplc_owner: hibana::runtime::program::RoleProgram<HPLC_OWNER>,
    pub mass_owner: hibana::runtime::program::RoleProgram<MASS_OWNER>,
    pub cell_owner: hibana::runtime::program::RoleProgram<CELL_OWNER>,
    pub hplc: hibana::runtime::program::RoleProgram<HPLC>,
    pub mass: hibana::runtime::program::RoleProgram<MASS>,
    pub cell: hibana::runtime::program::RoleProgram<CELL>,
    pub init: hibana::runtime::program::RoleProgram<INIT>,
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
}

pub fn programs() -> Programs {
    let source = discovery();
    use hibana::runtime::program::project;
    Programs {
        chemist: project(&source),
        biologist: project(&source),
        toxicologist: project(&source),
        lead: project(&source),
        hplc_owner: project(&source),
        mass_owner: project(&source),
        cell_owner: project(&source),
        hplc: project(&source),
        mass: project(&source),
        cell: project(&source),
        init: project(&source),
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
    }
}

impl Programs {
    pub fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        serde_json::json!({
            "type": "compiled_projection", "format_version": 1, "task": "14E",
            "session": SESSION,
            "roles": {
                "CHEMIST": CHEMIST,
                "BIOLOGIST": BIOLOGIST,
                "TOXICOLOGIST": TOXICOLOGIST,
                "LEAD_SCIENTIST": LEAD,
                "HPLC_OWNER": HPLC_OWNER,
                "MASS_OWNER": MASS_OWNER,
                "CELL_OWNER": CELL_OWNER,
                "HPLC": HPLC,
                "MASS_SPEC": MASS,
                "CELL_LAB": CELL,
                "INIT": INIT,
                "CHEM_FILE": CHEM_FILE,
                "CHEM_VM": CHEM_VM,
                "CHEM_IO": CHEM_IO,
                "CHEM_DATA": CHEM_DATA,
                "BIO_FILE": BIO_FILE,
                "BIO_VM": BIO_VM,
                "BIO_IO": BIO_IO,
                "BIO_DATA": BIO_DATA,
                "TOX_FILE": TOX_FILE,
                "TOX_VM": TOX_VM,
                "TOX_IO": TOX_IO,
                "TOX_DATA": TOX_DATA,
                "LEAD_FILE": LEAD_FILE,
                "LEAD_VM": LEAD_VM,
                "LEAD_IO": LEAD_IO,
                "LEAD_DATA": LEAD_DATA,
            },
            "labels": {
                "SynthesisQc": SYNTHESIS_QC, "MolecularId": MOLECULAR_ID,
                "CulturePrep": CULTURE_PREP, "Bioassay": BIOASSAY, "Stability": STABILITY,
                "Baseline": BASELINE, "Purity": PURITY, "Cytotoxicity": CYTOTOXICITY,
                "Reference": REFERENCE, "Structure": STRUCTURE,
                "Characterization": CHARACTERIZATION, "Decision": DECISION,
                "Documentation": DOCUMENTATION, "BiologicalSample": 50, "ToxicologySample": 54,
                "BioReport": 51, "ToxReport": 52, "DecisionReceipt": 53,
            },
            "programs": [
                rows(&self.chemist),
                rows(&self.biologist),
                rows(&self.toxicologist),
                rows(&self.lead),
                rows(&self.hplc_owner),
                rows(&self.mass_owner),
                rows(&self.cell_owner),
                rows(&self.hplc),
                rows(&self.mass),
                rows(&self.cell),
                rows(&self.init),
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
            ],
        })
    }
}
