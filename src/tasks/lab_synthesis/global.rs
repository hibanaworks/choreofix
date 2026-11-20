//! One scientific dataflow; two affine instrument Endpoints have actual I/O owners.
use super::arguments::{Arguments, InstrumentRequest};
pub use crate::shell::global::{Exit, FileExit, Loan};
use hibana::{
    Endpoint, g,
    runtime::program::{Projectable, RoleProgram, project},
};
use hibana_wasip1_runtime::{
    choreofs::{ChoreoFsObject, FdSpec, ObjectId},
    protocol,
};
pub const SESSION: u32 = 53;
pub const CHEMIST: u8 = 0;
pub const BIOLOGIST: u8 = 1;
pub const ANALYST: u8 = 2;
pub const SPEC_OWNER: u8 = 3;
pub const CENT_OWNER: u8 = 4;
pub const INIT: u8 = 5;
pub const SPECTROMETER: u8 = 6;
pub const CENTRIFUGE: u8 = 7;
pub const CHEM_FILE: u8 = 16;
pub const BIO_FILE: u8 = 17;
pub const ANALYST_FILE: u8 = 18;
pub const CHEM_VM: u8 = 32;
pub const BIO_VM: u8 = 33;
pub const ANALYST_VM: u8 = 34;
pub const CHEM_DATA: u8 = 48;
pub const BIO_DATA: u8 = 49;
pub const ANALYST_DATA: u8 = 50;
pub const CHEM_IO: u8 = 64;
pub const BIO_IO: u8 = 65;
pub const ANALYST_IO: u8 = 66;
pub type SpecLoan<'r> = g::Msg<70, Endpoint<'r, SPECTROMETER>>;
pub type CentLoan<'r> = g::Msg<71, Endpoint<'r, CENTRIFUGE>>;
pub type Raw = g::Msg<100, Arguments<0>>;
pub type RawResult<'a> = g::Msg<101, &'a [u8]>;
pub type Purify = g::Msg<102, Arguments<1>>;
pub type PurifiedResult<'a> = g::Msg<103, &'a [u8]>;
pub type Cells = g::Msg<104, Arguments<2>>;
pub type CellsResult<'a> = g::Msg<105, &'a [u8]>;
pub type Calibrate = g::Msg<106, Arguments<3>>;
pub type CalibrationResult<'a> = g::Msg<107, &'a [u8]>;
pub type Bioassay = g::Msg<108, Arguments<4>>;
pub type BioassayResult<'a> = g::Msg<109, &'a [u8]>;
pub type Molecular = g::Msg<110, Arguments<5>>;
pub type MolecularResult<'a> = g::Msg<111, &'a [u8]>;
pub type Verify = g::Msg<112, Arguments<6>>;
pub type VerificationResult<'a> = g::Msg<113, &'a [u8]>;
pub type Final = g::Msg<114, Arguments<7>>;
pub type FinalResult<'a> = g::Msg<115, &'a [u8]>;
pub type Purified<'a> = g::Msg<116, &'a [u8]>;
pub type Sample<'a> = g::Msg<117, &'a [u8]>;
pub type AssayNotice<'a> = g::Msg<118, &'a [u8]>;
pub type FileAssay<'a> = g::Msg<119, &'a [u8]>;
pub type Quality<'a> = g::Msg<120, &'a [u8]>;
pub type FileQuality<'a> = g::Msg<121, &'a [u8]>;
pub type FileRawResult<'a> = g::Msg<122, &'a [u8]>;
pub type FilePurifiedResult<'a> = g::Msg<123, &'a [u8]>;
pub type FileCellsResult<'a> = g::Msg<124, &'a [u8]>;
pub type FileCalibrationResult<'a> = g::Msg<125, &'a [u8]>;
pub type FileMolecularResult<'a> = g::Msg<126, &'a [u8]>;
pub type FileVerificationResult<'a> = g::Msg<127, &'a [u8]>;
pub type FileFinalResult<'a> = g::Msg<128, &'a [u8]>;
pub type RawOperation<'a> = g::Msg<129, InstrumentRequest<'a, 0>>;
pub type PurifyOperation<'a> = g::Msg<130, InstrumentRequest<'a, 1>>;
pub type CellsOperation<'a> = g::Msg<131, InstrumentRequest<'a, 2>>;
pub type CalibrationOperation<'a> = g::Msg<132, InstrumentRequest<'a, 3>>;
pub type MolecularOperation<'a> = g::Msg<133, InstrumentRequest<'a, 5>>;
pub type VerificationOperation<'a> = g::Msg<134, InstrumentRequest<'a, 6>>;
pub type FinalOperation<'a> = g::Msg<135, InstrumentRequest<'a, 7>>;

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
    operation(b"service/run_analysis/raw_materials", 4),
    operation(b"service/run_separation/compound", 5),
    incoming(b"service/quality", 6),
    operation(b"service/run_analysis/final_validation", 7),
];
pub const BIO_FILES: [ChoreoFsObject<'static>; 4] = [
    operation(b"service/run_separation/cell_cultures", 4),
    operation(b"service/run_analysis/fluorescence", 5),
    incoming(b"service/sample", 6),
    operation(b"service/run_bioassay", 7),
];
pub const ANALYST_FILES: [ChoreoFsObject<'static>; 3] = [
    incoming(b"service/bioassay", 4),
    operation(b"service/run_analysis/molecular_detail", 5),
    operation(b"service/run_separation/verification", 6),
];
pub fn source() -> impl Projectable {
    let instruments = g::par(
        g::send::<INIT, SPEC_OWNER, SpecLoan>(),
        g::send::<INIT, CENT_OWNER, CentLoan>(),
    );
    let files = g::par(
        g::send::<CHEM_DATA, CHEM_VM, Loan<CHEM_FILE>>(),
        g::par(
            g::send::<BIO_DATA, BIO_VM, Loan<BIO_FILE>>(),
            g::send::<ANALYST_DATA, ANALYST_VM, Loan<ANALYST_FILE>>(),
        ),
    );
    let raw = g::seq(
        g::send::<CHEM_FILE, CHEMIST, Raw>(),
        g::seq(
            g::send::<CHEMIST, SPECTROMETER, RawOperation>(),
            g::seq(
                g::send::<SPECTROMETER, CHEMIST, RawResult>(),
                g::send::<CHEMIST, CHEM_FILE, FileRawResult>(),
            ),
        ),
    );
    let purify = g::seq(
        g::send::<CHEM_FILE, CHEMIST, Purify>(),
        g::seq(
            g::send::<CHEMIST, CENTRIFUGE, PurifyOperation>(),
            g::seq(
                g::send::<CENTRIFUGE, CHEMIST, PurifiedResult>(),
                g::send::<CHEMIST, CHEM_FILE, FilePurifiedResult>(),
            ),
        ),
    );
    let cells = g::seq(
        g::send::<BIO_FILE, BIOLOGIST, Cells>(),
        g::seq(
            g::send::<BIOLOGIST, CENTRIFUGE, CellsOperation>(),
            g::seq(
                g::send::<CENTRIFUGE, BIOLOGIST, CellsResult>(),
                g::send::<BIOLOGIST, BIO_FILE, FileCellsResult>(),
            ),
        ),
    );
    let calibrate = g::seq(
        g::send::<BIO_FILE, BIOLOGIST, Calibrate>(),
        g::seq(
            g::send::<BIOLOGIST, SPECTROMETER, CalibrationOperation>(),
            g::seq(
                g::send::<SPECTROMETER, BIOLOGIST, CalibrationResult>(),
                g::send::<BIOLOGIST, BIO_FILE, FileCalibrationResult>(),
            ),
        ),
    );
    let molecular = g::seq(
        g::send::<ANALYST_FILE, ANALYST, Molecular>(),
        g::seq(
            g::send::<ANALYST, SPECTROMETER, MolecularOperation>(),
            g::seq(
                g::send::<SPECTROMETER, ANALYST, MolecularResult>(),
                g::send::<ANALYST, ANALYST_FILE, FileMolecularResult>(),
            ),
        ),
    );
    let verify = g::seq(
        g::send::<ANALYST_FILE, ANALYST, Verify>(),
        g::seq(
            g::send::<ANALYST, CENTRIFUGE, VerificationOperation>(),
            g::seq(
                g::send::<CENTRIFUGE, ANALYST, VerificationResult>(),
                g::par(
                    g::send::<ANALYST, ANALYST_FILE, FileVerificationResult>(),
                    g::send::<CENT_OWNER, INIT, CentLoan>(),
                ),
            ),
        ),
    );
    let final_validation = g::seq(
        g::send::<CHEM_FILE, CHEMIST, Final>(),
        g::seq(
            g::send::<CHEMIST, SPECTROMETER, FinalOperation>(),
            g::seq(
                g::send::<SPECTROMETER, CHEMIST, FinalResult>(),
                g::par(
                    g::send::<CHEMIST, CHEM_FILE, FileFinalResult>(),
                    g::send::<SPEC_OWNER, INIT, SpecLoan>(),
                ),
            ),
        ),
    );
    let prepare = g::par(
        g::seq(
            raw,
            g::seq(purify, g::send::<CHEMIST, BIOLOGIST, Purified>()),
        ),
        g::seq(cells, calibrate),
    );
    let assay = g::seq(
        g::send::<BIOLOGIST, BIO_FILE, Sample>(),
        g::seq(
            g::send::<BIO_FILE, BIOLOGIST, Bioassay>(),
            g::seq(
                g::send::<BIOLOGIST, BIO_FILE, BioassayResult>(),
                g::send::<BIOLOGIST, ANALYST, AssayNotice>(),
            ),
        ),
    );
    let quality = g::seq(
        g::send::<ANALYST, ANALYST_FILE, FileAssay>(),
        g::seq(
            g::par(molecular, verify),
            g::send::<ANALYST, CHEMIST, Quality>(),
        ),
    );
    let task = g::seq(
        prepare,
        g::seq(
            assay,
            g::seq(
                quality,
                g::seq(
                    g::send::<CHEMIST, CHEM_FILE, FileQuality>(),
                    final_validation,
                ),
            ),
        ),
    );
    let metadata = g::par(
        g::seq(
            crate::shell::global::file_metadata::<CHEM_FILE, CHEM_DATA>().roll(),
            g::send::<CHEM_FILE, CHEM_DATA, FileExit>(),
        ),
        g::par(
            g::seq(
                crate::shell::global::file_metadata::<BIO_FILE, BIO_DATA>().roll(),
                g::send::<BIO_FILE, BIO_DATA, FileExit>(),
            ),
            g::seq(
                crate::shell::global::file_metadata::<ANALYST_FILE, ANALYST_DATA>().roll(),
                g::send::<ANALYST_FILE, ANALYST_DATA, FileExit>(),
            ),
        ),
    );
    let io = g::par(
        g::seq(
            crate::shell::global::private_io::<CHEM_VM, CHEM_IO>(),
            g::send::<CHEM_VM, CHEM_IO, Exit>(),
        ),
        g::par(
            g::seq(
                crate::shell::global::private_io::<BIO_VM, BIO_IO>(),
                g::send::<BIO_VM, BIO_IO, Exit>(),
            ),
            g::seq(
                crate::shell::global::private_io::<ANALYST_VM, ANALYST_IO>(),
                g::send::<ANALYST_VM, ANALYST_IO, Exit>(),
            ),
        ),
    );
    g::seq(
        g::par(instruments, files),
        g::par(task, g::par(metadata, io)),
    )
}
pub struct Programs {
    pub chemist: RoleProgram<CHEMIST>,
    pub biologist: RoleProgram<BIOLOGIST>,
    pub analyst: RoleProgram<ANALYST>,
    pub spec_owner: RoleProgram<SPEC_OWNER>,
    pub cent_owner: RoleProgram<CENT_OWNER>,
    pub init: RoleProgram<INIT>,
    pub spec: RoleProgram<SPECTROMETER>,
    pub cent: RoleProgram<CENTRIFUGE>,
    pub chem_file: RoleProgram<CHEM_FILE>,
    pub bio_file: RoleProgram<BIO_FILE>,
    pub analyst_file: RoleProgram<ANALYST_FILE>,
    pub chem_vm: RoleProgram<CHEM_VM>,
    pub bio_vm: RoleProgram<BIO_VM>,
    pub analyst_vm: RoleProgram<ANALYST_VM>,
    pub chem_data: RoleProgram<CHEM_DATA>,
    pub bio_data: RoleProgram<BIO_DATA>,
    pub analyst_data: RoleProgram<ANALYST_DATA>,
    pub chem_io: RoleProgram<CHEM_IO>,
    pub bio_io: RoleProgram<BIO_IO>,
    pub analyst_io: RoleProgram<ANALYST_IO>,
}
pub fn programs() -> Programs {
    let p = source();
    Programs {
        chemist: project(&p),
        biologist: project(&p),
        analyst: project(&p),
        spec_owner: project(&p),
        cent_owner: project(&p),
        init: project(&p),
        spec: project(&p),
        cent: project(&p),
        chem_file: project(&p),
        bio_file: project(&p),
        analyst_file: project(&p),
        chem_vm: project(&p),
        bio_vm: project(&p),
        analyst_vm: project(&p),
        chem_data: project(&p),
        bio_data: project(&p),
        analyst_data: project(&p),
        chem_io: project(&p),
        bio_io: project(&p),
        analyst_io: project(&p),
    }
}
impl Programs {
    pub fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        use hibana::g::Message;
        let labels: serde_json::Map<String, serde_json::Value> = [
            ("SpecLoan", SpecLoan::LOGICAL_LABEL),
            ("CentLoan", CentLoan::LOGICAL_LABEL),
            ("Raw", Raw::LOGICAL_LABEL),
            ("RawResult", RawResult::LOGICAL_LABEL),
            ("Purify", Purify::LOGICAL_LABEL),
            ("PurifiedResult", PurifiedResult::LOGICAL_LABEL),
            ("Cells", Cells::LOGICAL_LABEL),
            ("CellsResult", CellsResult::LOGICAL_LABEL),
            ("Calibrate", Calibrate::LOGICAL_LABEL),
            ("CalibrationResult", CalibrationResult::LOGICAL_LABEL),
            ("Bioassay", Bioassay::LOGICAL_LABEL),
            ("BioassayResult", BioassayResult::LOGICAL_LABEL),
            ("Molecular", Molecular::LOGICAL_LABEL),
            ("MolecularResult", MolecularResult::LOGICAL_LABEL),
            ("Verify", Verify::LOGICAL_LABEL),
            ("VerificationResult", VerificationResult::LOGICAL_LABEL),
            ("Final", Final::LOGICAL_LABEL),
            ("FinalResult", FinalResult::LOGICAL_LABEL),
            ("Purified", Purified::LOGICAL_LABEL),
            ("Sample", Sample::LOGICAL_LABEL),
            ("AssayNotice", AssayNotice::LOGICAL_LABEL),
            ("FileAssay", FileAssay::LOGICAL_LABEL),
            ("Quality", Quality::LOGICAL_LABEL),
            ("FileQuality", FileQuality::LOGICAL_LABEL),
            ("FileRawResult", FileRawResult::LOGICAL_LABEL),
            ("FilePurifiedResult", FilePurifiedResult::LOGICAL_LABEL),
            ("FileCellsResult", FileCellsResult::LOGICAL_LABEL),
            (
                "FileCalibrationResult",
                FileCalibrationResult::LOGICAL_LABEL,
            ),
            ("FileMolecularResult", FileMolecularResult::LOGICAL_LABEL),
            (
                "FileVerificationResult",
                FileVerificationResult::LOGICAL_LABEL,
            ),
            ("FileFinalResult", FileFinalResult::LOGICAL_LABEL),
            ("RawOperation", RawOperation::LOGICAL_LABEL),
            ("PurifyOperation", PurifyOperation::LOGICAL_LABEL),
            ("CellsOperation", CellsOperation::LOGICAL_LABEL),
            ("CalibrationOperation", CalibrationOperation::LOGICAL_LABEL),
            ("MolecularOperation", MolecularOperation::LOGICAL_LABEL),
            (
                "VerificationOperation",
                VerificationOperation::LOGICAL_LABEL,
            ),
            ("FinalOperation", FinalOperation::LOGICAL_LABEL),
        ]
        .into_iter()
        .map(|(n, l)| (n.into(), l.into()))
        .collect();
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"13E","session":SESSION,"labels":labels,"roles":{
"CHEMIST":CHEMIST,"BIOLOGIST":BIOLOGIST,"ANALYST":ANALYST,"SPEC_OWNER":SPEC_OWNER,"CENT_OWNER":CENT_OWNER,"INIT":INIT,"SPECTROMETER":SPECTROMETER,"CENTRIFUGE":CENTRIFUGE,"CHEM_FILE":CHEM_FILE,"BIO_FILE":BIO_FILE,"ANALYST_FILE":ANALYST_FILE,"CHEM_VM":CHEM_VM,"BIO_VM":BIO_VM,"ANALYST_VM":ANALYST_VM,"CHEM_DATA":CHEM_DATA,"BIO_DATA":BIO_DATA,"ANALYST_DATA":ANALYST_DATA,"CHEM_IO":CHEM_IO,"BIO_IO":BIO_IO,"ANALYST_IO":ANALYST_IO},"programs":[rows(&self.chemist),rows(&self.biologist),rows(&self.analyst),rows(&self.spec_owner),rows(&self.cent_owner),rows(&self.init),rows(&self.spec),rows(&self.cent),rows(&self.chem_file),rows(&self.bio_file),rows(&self.analyst_file),rows(&self.chem_vm),rows(&self.bio_vm),rows(&self.analyst_vm),rows(&self.chem_data),rows(&self.bio_data),rows(&self.analyst_data),rows(&self.chem_io),rows(&self.bio_io),rows(&self.analyst_io)]})
    }
}
