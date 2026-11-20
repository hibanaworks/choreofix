//! Scientific material, exclusive instruments and cascading reconsideration.
use super::{
    arguments::{Arguments, InspectionArguments},
    imp::SynthesisTerminal,
};
use hibana::{g, runtime::program::Projectable};
use hibana_wasip1_runtime::{
    choreofs::{ChoreoFsObject, FdSpec, ObjectId},
    protocol,
};

pub use crate::shell::global::{Exit, FileExit, Loan};
use crate::shell::global::{file_metadata, private_io};

pub const CHEMIST: u8 = 0;
pub const BIOLOGIST: u8 = 1;
pub const ANALYST: u8 = 2;
pub const QC: u8 = 3;
pub const SAFETY_OFFICER: u8 = 4;
pub const CHEM_CONTROL: u8 = 5;
pub const BIO_CONTROL: u8 = 6;
pub const INIT: u8 = 7;
pub const SPEC_OWNER: u8 = 8;
pub const CENT_OWNER: u8 = 9;
pub const FUME_OWNER: u8 = 10;
pub const CHROMA_OWNER: u8 = 11;
pub const INSPECTION_SERVICE: u8 = 12;
pub const SPECTROMETER: u8 = 16;
pub const CENTRIFUGE: u8 = 17;
pub const FUME_HOOD: u8 = 18;
pub const CHROMATOGRAPH: u8 = 19;
pub const REAGENT_SUPPLY: u8 = 20;
pub const CHEM_FILE: u8 = 32;
pub const BIO_FILE: u8 = 33;
pub const ANALYST_FILE: u8 = 34;
pub const QC_FILE: u8 = 35;
pub const SAFETY_FILE: u8 = 36;
pub const CHEM_VM: u8 = 64;
pub const CHEMIST_IO: u8 = 80;
pub const BIO_VM: u8 = 65;
pub const BIOLOGIST_IO: u8 = 81;
pub const ANALYST_VM: u8 = 66;
pub const ANALYST_IO: u8 = 82;
pub const QC_VM: u8 = 67;
pub const QC_IO: u8 = 83;
pub const SAFETY_VM: u8 = 68;
pub const SAFETY_IO: u8 = 84;
pub const CHEM_DATA: u8 = 96;
pub const BIO_DATA: u8 = 97;
pub const ANALYST_DATA: u8 = 98;
pub const QC_DATA: u8 = 99;
pub const SAFETY_DATA: u8 = 100;
pub const QUALITY_OUTCOME: u16 = 0x1303;

pub const SESSION: u32 = 132;

/// Immutable images of the one scientific source, with no runtime progress.
pub struct Programs {
    pub chemist: hibana::runtime::program::RoleProgram<CHEMIST>,
    pub biologist: hibana::runtime::program::RoleProgram<BIOLOGIST>,
    pub analyst: hibana::runtime::program::RoleProgram<ANALYST>,
    pub qc: hibana::runtime::program::RoleProgram<QC>,
    pub safety_officer: hibana::runtime::program::RoleProgram<SAFETY_OFFICER>,
    pub chem_control: hibana::runtime::program::RoleProgram<CHEM_CONTROL>,
    pub bio_control: hibana::runtime::program::RoleProgram<BIO_CONTROL>,
    pub init: hibana::runtime::program::RoleProgram<INIT>,
    pub spec_owner: hibana::runtime::program::RoleProgram<SPEC_OWNER>,
    pub cent_owner: hibana::runtime::program::RoleProgram<CENT_OWNER>,
    pub fume_owner: hibana::runtime::program::RoleProgram<FUME_OWNER>,
    pub chroma_owner: hibana::runtime::program::RoleProgram<CHROMA_OWNER>,
    pub inspection_service: hibana::runtime::program::RoleProgram<INSPECTION_SERVICE>,
    pub spectrometer: hibana::runtime::program::RoleProgram<SPECTROMETER>,
    pub centrifuge: hibana::runtime::program::RoleProgram<CENTRIFUGE>,
    pub fume_hood: hibana::runtime::program::RoleProgram<FUME_HOOD>,
    pub chromatograph: hibana::runtime::program::RoleProgram<CHROMATOGRAPH>,
    pub reagent_supply: hibana::runtime::program::RoleProgram<REAGENT_SUPPLY>,
    pub chem_file: hibana::runtime::program::RoleProgram<CHEM_FILE>,
    pub bio_file: hibana::runtime::program::RoleProgram<BIO_FILE>,
    pub analyst_file: hibana::runtime::program::RoleProgram<ANALYST_FILE>,
    pub qc_file: hibana::runtime::program::RoleProgram<QC_FILE>,
    pub safety_file: hibana::runtime::program::RoleProgram<SAFETY_FILE>,
    pub chem_data: hibana::runtime::program::RoleProgram<CHEM_DATA>,
    pub bio_data: hibana::runtime::program::RoleProgram<BIO_DATA>,
    pub analyst_data: hibana::runtime::program::RoleProgram<ANALYST_DATA>,
    pub qc_data: hibana::runtime::program::RoleProgram<QC_DATA>,
    pub safety_data: hibana::runtime::program::RoleProgram<SAFETY_DATA>,
    pub chem_vm: hibana::runtime::program::RoleProgram<CHEM_VM>,
    pub chem_io: hibana::runtime::program::RoleProgram<CHEMIST_IO>,
    pub bio_vm: hibana::runtime::program::RoleProgram<BIO_VM>,
    pub bio_io: hibana::runtime::program::RoleProgram<BIOLOGIST_IO>,
    pub analyst_vm: hibana::runtime::program::RoleProgram<ANALYST_VM>,
    pub analyst_io: hibana::runtime::program::RoleProgram<ANALYST_IO>,
    pub qc_vm: hibana::runtime::program::RoleProgram<QC_VM>,
    pub qc_io: hibana::runtime::program::RoleProgram<QC_IO>,
    pub safety_vm: hibana::runtime::program::RoleProgram<SAFETY_VM>,
    pub safety_io: hibana::runtime::program::RoleProgram<SAFETY_IO>,
}

pub fn programs() -> Programs {
    let source = science();
    Programs {
        chemist: hibana::runtime::program::project(&source),
        biologist: hibana::runtime::program::project(&source),
        analyst: hibana::runtime::program::project(&source),
        qc: hibana::runtime::program::project(&source),
        safety_officer: hibana::runtime::program::project(&source),
        chem_control: hibana::runtime::program::project(&source),
        bio_control: hibana::runtime::program::project(&source),
        init: hibana::runtime::program::project(&source),
        spec_owner: hibana::runtime::program::project(&source),
        cent_owner: hibana::runtime::program::project(&source),
        fume_owner: hibana::runtime::program::project(&source),
        chroma_owner: hibana::runtime::program::project(&source),
        inspection_service: hibana::runtime::program::project(&source),
        spectrometer: hibana::runtime::program::project(&source),
        centrifuge: hibana::runtime::program::project(&source),
        fume_hood: hibana::runtime::program::project(&source),
        chromatograph: hibana::runtime::program::project(&source),
        reagent_supply: hibana::runtime::program::project(&source),
        chem_file: hibana::runtime::program::project(&source),
        bio_file: hibana::runtime::program::project(&source),
        analyst_file: hibana::runtime::program::project(&source),
        qc_file: hibana::runtime::program::project(&source),
        safety_file: hibana::runtime::program::project(&source),
        chem_data: hibana::runtime::program::project(&source),
        bio_data: hibana::runtime::program::project(&source),
        analyst_data: hibana::runtime::program::project(&source),
        qc_data: hibana::runtime::program::project(&source),
        safety_data: hibana::runtime::program::project(&source),
        chem_vm: hibana::runtime::program::project(&source),
        chem_io: hibana::runtime::program::project(&source),
        bio_vm: hibana::runtime::program::project(&source),
        bio_io: hibana::runtime::program::project(&source),
        analyst_vm: hibana::runtime::program::project(&source),
        analyst_io: hibana::runtime::program::project(&source),
        qc_vm: hibana::runtime::program::project(&source),
        qc_io: hibana::runtime::program::project(&source),
        safety_vm: hibana::runtime::program::project(&source),
        safety_io: hibana::runtime::program::project(&source),
    }
}

impl Programs {
    pub fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        use hibana::g::Message;
        let labels: serde_json::Map<String, serde_json::Value> = [
            ("FileInspection", FileInspection::LOGICAL_LABEL),
            ("InspectionAdmission", InspectionAdmission::LOGICAL_LABEL),
            ("SpecInspection", SpecInspection::LOGICAL_LABEL),
            ("CentInspection", CentInspection::LOGICAL_LABEL),
            ("FumeInspection", FumeInspection::LOGICAL_LABEL),
            ("ChromaInspection", ChromaInspection::LOGICAL_LABEL),
            ("SpecResult", SpecResult::LOGICAL_LABEL),
            ("CentResult", CentResult::LOGICAL_LABEL),
            ("FumeResult", FumeResult::LOGICAL_LABEL),
            ("ChromaResult", ChromaResult::LOGICAL_LABEL),
            ("FileInspectionResult", FileInspectionResult::LOGICAL_LABEL),
            ("FileSynthesis", FileSynthesis::LOGICAL_LABEL),
            ("SynthesisAdmission", SynthesisAdmission::LOGICAL_LABEL),
            ("FumeSynthesis", FumeSynthesis::LOGICAL_LABEL),
            ("ReagentSynthesis", ReagentSynthesis::LOGICAL_LABEL),
            ("FumeSynthesisResult", FumeSynthesisResult::LOGICAL_LABEL),
            (
                "ReagentSynthesisResult",
                ReagentSynthesisResult::LOGICAL_LABEL,
            ),
            ("FileSynthesisResult", FileSynthesisResult::LOGICAL_LABEL),
            ("SynthesisFinished", SynthesisFinished::LOGICAL_LABEL),
            ("ReagentsExhausted", ReagentsExhausted::LOGICAL_LABEL),
            ("BiologyReport", BiologyReport::LOGICAL_LABEL),
            ("FileQuality", FileQuality::LOGICAL_LABEL),
            ("QualityAdmission", QualityAdmission::LOGICAL_LABEL),
            ("FumeQuality", FumeQuality::LOGICAL_LABEL),
            ("SpecQuality", SpecQuality::LOGICAL_LABEL),
            ("FumeQualityApproved", FumeQualityApproved::LOGICAL_LABEL),
            ("FumeQualityRejected", FumeQualityRejected::LOGICAL_LABEL),
            ("SpecQualityApproved", SpecQualityApproved::LOGICAL_LABEL),
            ("SpecQualityRejected", SpecQualityRejected::LOGICAL_LABEL),
            ("BiologyApproved", BiologyApproved::LOGICAL_LABEL),
            ("BiologyRejected", BiologyRejected::LOGICAL_LABEL),
            ("FileQualityResult", FileQualityResult::LOGICAL_LABEL),
            ("FileAnalysis", FileAnalysis::LOGICAL_LABEL),
            ("Analysis", Analysis::LOGICAL_LABEL),
            ("AnalysisResult", AnalysisResult::LOGICAL_LABEL),
            ("FileAnalysisResult", FileAnalysisResult::LOGICAL_LABEL),
            ("FilePurification", FilePurification::LOGICAL_LABEL),
            ("Purification", Purification::LOGICAL_LABEL),
            ("PurificationResult", PurificationResult::LOGICAL_LABEL),
            (
                "FilePurificationResult",
                FilePurificationResult::LOGICAL_LABEL,
            ),
            ("FileCells", FileCells::LOGICAL_LABEL),
            ("Cells", Cells::LOGICAL_LABEL),
            ("CellsResult", CellsResult::LOGICAL_LABEL),
            ("FileCellsResult", FileCellsResult::LOGICAL_LABEL),
            ("FileCalibration", FileCalibration::LOGICAL_LABEL),
            ("Calibration", Calibration::LOGICAL_LABEL),
            ("CalibrationResult", CalibrationResult::LOGICAL_LABEL),
            (
                "FileCalibrationResult",
                FileCalibrationResult::LOGICAL_LABEL,
            ),
            ("PurifiedMaterial", PurifiedMaterial::LOGICAL_LABEL),
            ("FileBioassay", FileBioassay::LOGICAL_LABEL),
            ("Assay", Assay::LOGICAL_LABEL),
            ("AssayResult", AssayResult::LOGICAL_LABEL),
            ("FileBioassayResult", FileBioassayResult::LOGICAL_LABEL),
            ("AnalystMaterial", AnalystMaterial::LOGICAL_LABEL),
            ("FilePublishAnalyst", FilePublishAnalyst::LOGICAL_LABEL),
            ("FilePublishQc", FilePublishQc::LOGICAL_LABEL),
            ("FilePublishedAnalyst", FilePublishedAnalyst::LOGICAL_LABEL),
            ("FilePublishedQc", FilePublishedQc::LOGICAL_LABEL),
            ("FileChromatography", FileChromatography::LOGICAL_LABEL),
            ("Chromatography", Chromatography::LOGICAL_LABEL),
            ("ChromatographyResult", ChromatographyResult::LOGICAL_LABEL),
            (
                "FileChromatographyResult",
                FileChromatographyResult::LOGICAL_LABEL,
            ),
            ("FileMolecular", FileMolecular::LOGICAL_LABEL),
            ("Molecular", Molecular::LOGICAL_LABEL),
            ("MolecularResult", MolecularResult::LOGICAL_LABEL),
            ("FileMolecularResult", FileMolecularResult::LOGICAL_LABEL),
            ("FileVerification", FileVerification::LOGICAL_LABEL),
            ("Verification", Verification::LOGICAL_LABEL),
            ("VerificationResult", VerificationResult::LOGICAL_LABEL),
            (
                "FileVerificationResult",
                FileVerificationResult::LOGICAL_LABEL,
            ),
            ("AnalystReport", AnalystReport::LOGICAL_LABEL),
            ("AnalystApproved", AnalystApproved::LOGICAL_LABEL),
            ("AnalystRejected", AnalystRejected::LOGICAL_LABEL),
            ("ReSynthesize", ReSynthesize::LOGICAL_LABEL),
            ("FinalApproved", FinalApproved::LOGICAL_LABEL),
            ("FinalExhausted", FinalExhausted::LOGICAL_LABEL),
            ("FileFinal", FileFinal::LOGICAL_LABEL),
            ("FinalValidation", FinalValidation::LOGICAL_LABEL),
            ("FinalResult", FinalResult::LOGICAL_LABEL),
            ("FileFinalResult", FileFinalResult::LOGICAL_LABEL),
            ("FinalValidated", FinalValidated::LOGICAL_LABEL),
            ("FileBioBatch", FileBioBatch::LOGICAL_LABEL),
            ("FileAnalystBatch", FileAnalystBatch::LOGICAL_LABEL),
            ("BiologyFinalApproved", BiologyFinalApproved::LOGICAL_LABEL),
            ("AnalystFinalApproved", AnalystFinalApproved::LOGICAL_LABEL),
            ("FumeFinalApproved", FumeFinalApproved::LOGICAL_LABEL),
            ("SpecFinalApproved", SpecFinalApproved::LOGICAL_LABEL),
            ("CentFinalApproved", CentFinalApproved::LOGICAL_LABEL),
            ("ChromaFinalApproved", ChromaFinalApproved::LOGICAL_LABEL),
            ("BiologyReSynthesize", BiologyReSynthesize::LOGICAL_LABEL),
            ("AnalystReSynthesize", AnalystReSynthesize::LOGICAL_LABEL),
            ("FumeReSynthesize", FumeReSynthesize::LOGICAL_LABEL),
            ("SpecReSynthesize", SpecReSynthesize::LOGICAL_LABEL),
            ("CentReSynthesize", CentReSynthesize::LOGICAL_LABEL),
            ("ChromaReSynthesize", ChromaReSynthesize::LOGICAL_LABEL),
            (
                "BiologyFinalExhausted",
                BiologyFinalExhausted::LOGICAL_LABEL,
            ),
            (
                "AnalystFinalExhausted",
                AnalystFinalExhausted::LOGICAL_LABEL,
            ),
            ("FumeFinalExhausted", FumeFinalExhausted::LOGICAL_LABEL),
            ("SpecFinalExhausted", SpecFinalExhausted::LOGICAL_LABEL),
            ("CentFinalExhausted", CentFinalExhausted::LOGICAL_LABEL),
            ("ChromaFinalExhausted", ChromaFinalExhausted::LOGICAL_LABEL),
            ("AnalystReportToQc", AnalystReportToQc::LOGICAL_LABEL),
            ("FileChemQuality", FileChemQuality::LOGICAL_LABEL),
            ("FileQcQuality", FileQcQuality::LOGICAL_LABEL),
            ("FileQcInput", FileQcInput::LOGICAL_LABEL),
            ("SynthesisFailed", SynthesisFailed::LOGICAL_LABEL),
            ("BiologyBatch", BiologyBatch::LOGICAL_LABEL),
            ("AnalystBatch", AnalystBatch::LOGICAL_LABEL),
            ("SpecBatch", SpecBatch::LOGICAL_LABEL),
            ("CentBatch", CentBatch::LOGICAL_LABEL),
            ("ChromaBatch", ChromaBatch::LOGICAL_LABEL),
            (
                "BiologySynthesisFailed",
                BiologySynthesisFailed::LOGICAL_LABEL,
            ),
            (
                "AnalystSynthesisFailed",
                AnalystSynthesisFailed::LOGICAL_LABEL,
            ),
            ("SpecSynthesisFailed", SpecSynthesisFailed::LOGICAL_LABEL),
            ("CentSynthesisFailed", CentSynthesisFailed::LOGICAL_LABEL),
            (
                "ChromaSynthesisFailed",
                ChromaSynthesisFailed::LOGICAL_LABEL,
            ),
            (
                "BiologyReagentsExhausted",
                BiologyReagentsExhausted::LOGICAL_LABEL,
            ),
            (
                "AnalystReagentsExhausted",
                AnalystReagentsExhausted::LOGICAL_LABEL,
            ),
            (
                "SpecReagentsExhausted",
                SpecReagentsExhausted::LOGICAL_LABEL,
            ),
            (
                "CentReagentsExhausted",
                CentReagentsExhausted::LOGICAL_LABEL,
            ),
            (
                "ChromaReagentsExhausted",
                ChromaReagentsExhausted::LOGICAL_LABEL,
            ),
            ("SpecQualityAdmission", SpecQualityAdmission::LOGICAL_LABEL),
            (
                "ChromaQualityApproved",
                ChromaQualityApproved::LOGICAL_LABEL,
            ),
            (
                "ChromaQualityRejected",
                ChromaQualityRejected::LOGICAL_LABEL,
            ),
            ("ChromaQualityReceipt", ChromaQualityReceipt::LOGICAL_LABEL),
        ]
        .into_iter()
        .map(|(name, label)| (name.into(), label.into()))
        .collect();
        let roles = std::collections::BTreeMap::from([
            ("CHEMIST", CHEMIST),
            ("BIOLOGIST", BIOLOGIST),
            ("ANALYST", ANALYST),
            ("QC", QC),
            ("SAFETY_OFFICER", SAFETY_OFFICER),
            ("CHEM_CONTROL", CHEM_CONTROL),
            ("BIO_CONTROL", BIO_CONTROL),
            ("INIT", INIT),
            ("SPEC_OWNER", SPEC_OWNER),
            ("CENT_OWNER", CENT_OWNER),
            ("FUME_OWNER", FUME_OWNER),
            ("CHROMA_OWNER", CHROMA_OWNER),
            ("INSPECTION_SERVICE", INSPECTION_SERVICE),
            ("SPECTROMETER", SPECTROMETER),
            ("CENTRIFUGE", CENTRIFUGE),
            ("FUME_HOOD", FUME_HOOD),
            ("CHROMATOGRAPH", CHROMATOGRAPH),
            ("REAGENT_SUPPLY", REAGENT_SUPPLY),
            ("CHEM_FILE", CHEM_FILE),
            ("BIO_FILE", BIO_FILE),
            ("ANALYST_FILE", ANALYST_FILE),
            ("QC_FILE", QC_FILE),
            ("SAFETY_FILE", SAFETY_FILE),
            ("CHEM_DATA", CHEM_DATA),
            ("BIO_DATA", BIO_DATA),
            ("ANALYST_DATA", ANALYST_DATA),
            ("QC_DATA", QC_DATA),
            ("SAFETY_DATA", SAFETY_DATA),
            ("CHEM_VM", CHEM_VM),
            ("BIO_VM", BIO_VM),
            ("ANALYST_VM", ANALYST_VM),
            ("QC_VM", QC_VM),
            ("SAFETY_VM", SAFETY_VM),
            ("CHEMIST_IO", CHEMIST_IO),
            ("BIOLOGIST_IO", BIOLOGIST_IO),
            ("ANALYST_IO", ANALYST_IO),
            ("QC_IO", QC_IO),
            ("SAFETY_IO", SAFETY_IO),
        ]);
        let programs = [
            rows(&self.chemist),
            rows(&self.biologist),
            rows(&self.analyst),
            rows(&self.qc),
            rows(&self.safety_officer),
            rows(&self.chem_control),
            rows(&self.bio_control),
            rows(&self.init),
            rows(&self.spec_owner),
            rows(&self.cent_owner),
            rows(&self.fume_owner),
            rows(&self.chroma_owner),
            rows(&self.inspection_service),
            rows(&self.spectrometer),
            rows(&self.centrifuge),
            rows(&self.fume_hood),
            rows(&self.chromatograph),
            rows(&self.reagent_supply),
            rows(&self.chem_file),
            rows(&self.bio_file),
            rows(&self.analyst_file),
            rows(&self.qc_file),
            rows(&self.safety_file),
            rows(&self.chem_data),
            rows(&self.bio_data),
            rows(&self.analyst_data),
            rows(&self.qc_data),
            rows(&self.safety_data),
            rows(&self.chem_vm),
            rows(&self.bio_vm),
            rows(&self.analyst_vm),
            rows(&self.qc_vm),
            rows(&self.safety_vm),
            rows(&self.chem_io),
            rows(&self.bio_io),
            rows(&self.analyst_io),
            rows(&self.qc_io),
            rows(&self.safety_io),
        ];
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"13H",
            "session":SESSION,"roles":roles,"labels":labels,"programs":programs.as_slice()})
    }
}

pub const SYNTHESIS_OUTCOME: u16 = 0x1301;
pub const SYNTHESIS_FAILURE: u16 = 0x1302;

const fn writable(path: &'static [u8], fd: u8) -> ChoreoFsObject<'static> {
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

pub const CHEM_FILES: [ChoreoFsObject<'static>; 5] = [
    writable(b"service/run_synthesis", 4),
    writable(b"service/run_analysis/synthesized_compound", 5),
    writable(b"service/run_separation/compound_purification", 6),
    incoming(b"service/qc", 7),
    writable(b"service/run_analysis/final_validation", 8),
];
pub const BIO_FILES: [ChoreoFsObject<'static>; 6] = [
    writable(b"service/run_separation/cell_cultures", 4),
    writable(b"service/run_analysis/fluorescence", 5),
    incoming(b"service/batch", 6),
    writable(b"service/run_bioassay", 7),
    writable(b"service/publish/analyst", 8),
    writable(b"service/publish/qc", 9),
];
pub const ANALYST_FILES: [ChoreoFsObject<'static>; 4] = [
    incoming(b"service/batch", 4),
    writable(b"service/run_chromatography", 5),
    writable(b"service/run_analysis/molecular_detail", 6),
    writable(b"service/run_separation/verification", 7),
];
pub const QC_FILES: [ChoreoFsObject<'static>; 3] = [
    incoming(b"service/bioassay", 4),
    writable(b"service/verify_result/BIOLOGIST", 5),
    incoming(b"service/qc", 6),
];
pub const SAFETY_FILES: [ChoreoFsObject<'static>; 1] = [writable(b"service/perform_inspection", 4)];

pub type FileInspection = g::Msg<1, InspectionArguments>;
pub type InspectionAdmission = g::Msg<2, InspectionArguments>;
pub type SpecInspection = g::Msg<3, InspectionArguments>;
pub type CentInspection = g::Msg<4, InspectionArguments>;
pub type FumeInspection = g::Msg<5, InspectionArguments>;
pub type ChromaInspection = g::Msg<6, InspectionArguments>;
pub type SpecResult<'a> = g::Msg<7, &'a [u8]>;
pub type CentResult<'a> = g::Msg<8, &'a [u8]>;
pub type FumeResult<'a> = g::Msg<9, &'a [u8]>;
pub type ChromaResult<'a> = g::Msg<10, &'a [u8]>;
pub type FileInspectionResult<'a> = g::Msg<11, &'a [u8]>;

pub type FileSynthesis = g::Msg<16, Arguments<0>>;
pub type SynthesisAdmission = g::Msg<17, Arguments<0>>;
pub type FumeSynthesis = g::Msg<18, Arguments<0>>;
pub type ReagentSynthesis = g::Msg<19, Arguments<0>>;
pub type FumeSynthesisResult<'a> = g::Msg<20, &'a [u8]>;
pub type ReagentSynthesisResult<'a> = g::Msg<21, &'a [u8]>;
pub type FileSynthesisResult<'a> = g::Msg<22, &'a [u8]>;
pub type SynthesisFinished = g::Msg<23, SynthesisTerminal<true>>;
pub type ReagentsExhausted = g::Msg<24, SynthesisTerminal<false>>;

pub type BiologyReport<'a> = g::Msg<29, &'a [u8]>;
pub type FileQuality = g::Msg<30, Arguments<9>>;
pub type QualityAdmission<'a> = g::Msg<31, &'a [u8]>;
pub type FumeQuality<'a> = g::Msg<32, &'a [u8]>;
pub type SpecQuality<'a> = g::Msg<33, &'a [u8]>;
pub type FumeQualityApproved<'a> =
    g::Msg<34, crate::shell::imp::review_reply::ReviewReply<'a, true>>;
pub type FumeQualityRejected<'a> =
    g::Msg<35, crate::shell::imp::review_reply::ReviewReply<'a, false>>;
pub type SpecQualityApproved<'a> =
    g::Msg<36, crate::shell::imp::review_reply::ReviewReply<'a, true>>;
pub type SpecQualityRejected<'a> =
    g::Msg<37, crate::shell::imp::review_reply::ReviewReply<'a, false>>;
pub type BiologyApproved<'a> = g::Msg<38, crate::shell::imp::review_reply::ReviewReply<'a, true>>;
pub type BiologyRejected<'a> = g::Msg<39, crate::shell::imp::review_reply::ReviewReply<'a, false>>;
pub type FileQualityResult<'a> = g::Msg<40, &'a [u8]>;

pub type FileAnalysis = g::Msg<50, Arguments<1>>;
pub type Analysis<'a> = g::Msg<51, &'a [u8]>;
pub type AnalysisResult<'a> = g::Msg<52, &'a [u8]>;
pub type FileAnalysisResult<'a> = g::Msg<53, &'a [u8]>;
pub type FilePurification = g::Msg<54, Arguments<2>>;
pub type Purification<'a> = g::Msg<55, &'a [u8]>;
pub type PurificationResult<'a> = g::Msg<56, &'a [u8]>;
pub type FilePurificationResult<'a> = g::Msg<57, &'a [u8]>;
pub type FileCells = g::Msg<58, Arguments<3>>;
pub type Cells<'a> = g::Msg<59, &'a [u8]>;
pub type CellsResult<'a> = g::Msg<60, &'a [u8]>;
pub type FileCellsResult<'a> = g::Msg<61, &'a [u8]>;
pub type FileCalibration = g::Msg<62, Arguments<4>>;
pub type Calibration<'a> = g::Msg<63, &'a [u8]>;
pub type CalibrationResult<'a> = g::Msg<64, &'a [u8]>;
pub type FileCalibrationResult<'a> = g::Msg<65, &'a [u8]>;
pub type PurifiedMaterial<'a> = g::Msg<66, &'a [u8]>;
pub type FileBioassay = g::Msg<67, Arguments<5>>;
pub type Assay<'a> = g::Msg<68, &'a [u8]>;
pub type AssayResult<'a> = g::Msg<69, &'a [u8]>;
pub type FileBioassayResult<'a> = g::Msg<70, &'a [u8]>;
pub type AnalystMaterial<'a> = g::Msg<71, &'a [u8]>;
pub type FilePublishAnalyst = g::Msg<72, Arguments<11>>;
pub type FilePublishQc = g::Msg<73, Arguments<11>>;
pub type FilePublishedAnalyst<'a> = g::Msg<74, &'a [u8]>;
pub type FilePublishedQc<'a> = g::Msg<75, &'a [u8]>;
pub type FileChromatography = g::Msg<76, Arguments<6>>;
pub type Chromatography<'a> = g::Msg<77, &'a [u8]>;
pub type ChromatographyResult<'a> = g::Msg<78, &'a [u8]>;
pub type FileChromatographyResult<'a> = g::Msg<79, &'a [u8]>;
pub type FileMolecular = g::Msg<80, Arguments<7>>;
pub type Molecular<'a> = g::Msg<81, &'a [u8]>;
pub type MolecularResult<'a> = g::Msg<82, &'a [u8]>;
pub type FileMolecularResult<'a> = g::Msg<83, &'a [u8]>;
pub type FileVerification = g::Msg<84, Arguments<8>>;
pub type Verification<'a> = g::Msg<85, &'a [u8]>;
pub type VerificationResult<'a> = g::Msg<86, &'a [u8]>;
pub type FileVerificationResult<'a> = g::Msg<87, &'a [u8]>;
pub type AnalystReport<'a> = g::Msg<88, &'a [u8]>;
pub type AnalystApproved<'a> = g::Msg<89, crate::shell::imp::review_reply::ReviewReply<'a, true>>;
pub type AnalystRejected<'a> = g::Msg<90, crate::shell::imp::review_reply::ReviewReply<'a, false>>;
pub type ReSynthesize<'a> = g::Msg<91, crate::shell::imp::review_reply::ReviewReply<'a, false>>;
pub type FinalApproved<'a> = g::Msg<92, crate::shell::imp::review_reply::ReviewReply<'a, true>>;
pub type FinalExhausted = g::Msg<93, SynthesisTerminal<false>>;
pub type FileFinal = g::Msg<94, Arguments<10>>;
pub type FinalValidation<'a> = g::Msg<95, &'a [u8]>;
pub type FinalResult<'a> = g::Msg<96, &'a [u8]>;
pub type FileFinalResult<'a> = g::Msg<97, &'a [u8]>;
pub type FinalValidated<'a> = g::Msg<151, &'a [u8]>;
pub type FileBioBatch<'a> = g::Msg<99, &'a [u8]>;
pub type FileAnalystBatch<'a> = g::Msg<100, &'a [u8]>;
pub type BiologyFinalApproved<'a> =
    g::Msg<102, crate::shell::imp::review_reply::ReviewReply<'a, true>>;
pub type AnalystFinalApproved<'a> =
    g::Msg<103, crate::shell::imp::review_reply::ReviewReply<'a, true>>;
pub type FumeFinalApproved<'a> =
    g::Msg<104, crate::shell::imp::review_reply::ReviewReply<'a, true>>;
pub type SpecFinalApproved<'a> =
    g::Msg<105, crate::shell::imp::review_reply::ReviewReply<'a, true>>;
pub type CentFinalApproved<'a> =
    g::Msg<106, crate::shell::imp::review_reply::ReviewReply<'a, true>>;
pub type ChromaFinalApproved<'a> =
    g::Msg<107, crate::shell::imp::review_reply::ReviewReply<'a, true>>;
pub type BiologyReSynthesize<'a> =
    g::Msg<108, crate::shell::imp::review_reply::ReviewReply<'a, false>>;
pub type AnalystReSynthesize<'a> =
    g::Msg<109, crate::shell::imp::review_reply::ReviewReply<'a, false>>;
pub type FumeReSynthesize<'a> =
    g::Msg<110, crate::shell::imp::review_reply::ReviewReply<'a, false>>;
pub type SpecReSynthesize<'a> =
    g::Msg<111, crate::shell::imp::review_reply::ReviewReply<'a, false>>;
pub type CentReSynthesize<'a> =
    g::Msg<112, crate::shell::imp::review_reply::ReviewReply<'a, false>>;
pub type ChromaReSynthesize<'a> =
    g::Msg<113, crate::shell::imp::review_reply::ReviewReply<'a, false>>;
pub type BiologyFinalExhausted = g::Msg<114, SynthesisTerminal<false>>;
pub type AnalystFinalExhausted = g::Msg<115, SynthesisTerminal<false>>;
pub type FumeFinalExhausted = g::Msg<116, SynthesisTerminal<false>>;
pub type SpecFinalExhausted = g::Msg<117, SynthesisTerminal<false>>;
pub type CentFinalExhausted = g::Msg<118, SynthesisTerminal<false>>;
pub type ChromaFinalExhausted = g::Msg<119, SynthesisTerminal<false>>;
pub type AnalystReportToQc<'a> = g::Msg<120, &'a [u8]>;
pub type FileChemQuality<'a> = g::Msg<98, &'a [u8]>;
pub type FileQcQuality<'a> = g::Msg<122, &'a [u8]>;
pub type FileQcInput<'a> = g::Msg<123, &'a [u8]>;
pub type SynthesisFailed<'a> = g::Msg<124, crate::shell::imp::review_reply::ReviewReply<'a, false>>;
pub type BiologyBatch = g::Msg<125, SynthesisTerminal<true>>;
pub type AnalystBatch = g::Msg<126, SynthesisTerminal<true>>;
pub type SpecBatch = g::Msg<127, SynthesisTerminal<true>>;
pub type CentBatch = g::Msg<128, SynthesisTerminal<true>>;
pub type ChromaBatch = g::Msg<129, SynthesisTerminal<true>>;
pub type BiologySynthesisFailed<'a> =
    g::Msg<130, crate::shell::imp::review_reply::ReviewReply<'a, false>>;
pub type AnalystSynthesisFailed<'a> =
    g::Msg<131, crate::shell::imp::review_reply::ReviewReply<'a, false>>;
pub type SpecSynthesisFailed<'a> =
    g::Msg<132, crate::shell::imp::review_reply::ReviewReply<'a, false>>;
pub type CentSynthesisFailed<'a> =
    g::Msg<133, crate::shell::imp::review_reply::ReviewReply<'a, false>>;
pub type ChromaSynthesisFailed<'a> =
    g::Msg<134, crate::shell::imp::review_reply::ReviewReply<'a, false>>;
pub type BiologyReagentsExhausted = g::Msg<135, SynthesisTerminal<false>>;
pub type AnalystReagentsExhausted = g::Msg<136, SynthesisTerminal<false>>;
pub type SpecReagentsExhausted = g::Msg<137, SynthesisTerminal<false>>;
pub type CentReagentsExhausted = g::Msg<138, SynthesisTerminal<false>>;
pub type ChromaReagentsExhausted = g::Msg<139, SynthesisTerminal<false>>;
pub type SpecQualityAdmission<'a> = g::Msg<140, &'a [u8]>;
pub type ChromaQualityApproved<'a> =
    g::Msg<141, crate::shell::imp::review_reply::ReviewReply<'a, true>>;
pub type ChromaQualityRejected<'a> =
    g::Msg<142, crate::shell::imp::review_reply::ReviewReply<'a, false>>;
pub type ChromaQualityReceipt<'a> = g::Msg<143, &'a [u8]>;

/// QC consumes the current material; rejection returns actual reagent authority.
/// This fragment has no second batch counter or approval table.
pub(crate) fn quality_exchange(
    approved: impl Projectable,
    rejected: impl Projectable,
) -> impl Projectable {
    let approved = g::seq(
        g::par(
            g::send::<FUME_OWNER, FUME_HOOD, FumeQualityApproved>(),
            g::send::<FUME_OWNER, SPECTROMETER, SpecQualityApproved>(),
        ),
        g::seq(
            g::par(
                g::send::<QC, FUME_OWNER, Loan<FUME_HOOD>>(),
                g::send::<QC, SPEC_OWNER, Loan<SPECTROMETER>>(),
            ),
            approved,
        ),
    );
    let rejected = g::seq(
        g::par(
            g::send::<FUME_OWNER, FUME_HOOD, FumeQualityRejected>(),
            g::send::<FUME_OWNER, SPECTROMETER, SpecQualityRejected>(),
        ),
        g::seq(
            g::par(
                g::send::<QC, FUME_OWNER, Loan<FUME_HOOD>>(),
                g::send::<QC, SPEC_OWNER, Loan<SPECTROMETER>>(),
            ),
            rejected,
        ),
    );
    g::seq(
        g::par(
            g::send::<QC, FUME_OWNER, QualityAdmission>(),
            g::send::<QC, SPEC_OWNER, SpecQualityAdmission>(),
        ),
        g::seq(
            g::par(
                g::send::<FUME_OWNER, QC, Loan<FUME_HOOD>>(),
                g::send::<SPEC_OWNER, QC, Loan<SPECTROMETER>>(),
            ),
            g::seq(
                g::par(
                    g::send::<FUME_HOOD, FUME_OWNER, FumeQuality>(),
                    g::send::<SPECTROMETER, FUME_OWNER, SpecQuality>(),
                ),
                g::route(approved, rejected).resolve::<QUALITY_OUTCOME>(),
            ),
        ),
    )
}

/// Actual attempts repeat; finishing transfers the actual outcome and stock access to QC.
/// The world owns the material quantity. No application stock counter is introduced.
pub(crate) fn synthesis_attempt() -> impl Projectable {
    g::seq(
        g::send::<CHEM_FILE, CHEMIST, FileSynthesis>(),
        g::seq(
            g::send::<CHEMIST, FUME_OWNER, SynthesisAdmission>(),
            g::seq(
                g::par(
                    g::send::<FUME_OWNER, CHEMIST, Loan<FUME_HOOD>>(),
                    g::send::<FUME_OWNER, CHEMIST, Loan<REAGENT_SUPPLY>>(),
                ),
                g::seq(
                    g::par(
                        g::send::<FUME_HOOD, FUME_OWNER, FumeSynthesis>(),
                        g::send::<REAGENT_SUPPLY, FUME_OWNER, ReagentSynthesis>(),
                    ),
                    g::seq(
                        g::par(
                            g::send::<FUME_OWNER, FUME_HOOD, FumeSynthesisResult>(),
                            g::send::<FUME_OWNER, REAGENT_SUPPLY, ReagentSynthesisResult>(),
                        ),
                        g::par(
                            g::par(
                                g::send::<CHEMIST, FUME_OWNER, Loan<FUME_HOOD>>(),
                                g::send::<CHEMIST, FUME_OWNER, Loan<REAGENT_SUPPLY>>(),
                            ),
                            g::send::<CHEMIST, CHEM_FILE, FileSynthesisResult>(),
                        ),
                    ),
                ),
            ),
        ),
    )
}

/// The file request, exclusive instrument loans, effect requests and actual replies.
/// This is a component of 13H, not the complete scientific task choreography.
pub fn inspection() -> impl Projectable {
    let loans = g::par(
        g::par(
            g::seq(
                g::send::<SAFETY_OFFICER, SPEC_OWNER, SpecInspection>(),
                g::send::<SPEC_OWNER, SAFETY_OFFICER, Loan<SPECTROMETER>>(),
            ),
            g::seq(
                g::send::<SAFETY_OFFICER, CENT_OWNER, CentInspection>(),
                g::send::<CENT_OWNER, SAFETY_OFFICER, Loan<CENTRIFUGE>>(),
            ),
        ),
        g::par(
            g::seq(
                g::send::<SAFETY_OFFICER, FUME_OWNER, FumeInspection>(),
                g::send::<FUME_OWNER, SAFETY_OFFICER, Loan<FUME_HOOD>>(),
            ),
            g::seq(
                g::send::<SAFETY_OFFICER, CHROMA_OWNER, ChromaInspection>(),
                g::send::<CHROMA_OWNER, SAFETY_OFFICER, Loan<CHROMATOGRAPH>>(),
            ),
        ),
    );
    let requests = g::par(
        g::par(
            g::send::<SPECTROMETER, INSPECTION_SERVICE, SpecInspection>(),
            g::send::<CENTRIFUGE, INSPECTION_SERVICE, CentInspection>(),
        ),
        g::par(
            g::send::<FUME_HOOD, INSPECTION_SERVICE, FumeInspection>(),
            g::send::<CHROMATOGRAPH, INSPECTION_SERVICE, ChromaInspection>(),
        ),
    );
    let replies = g::par(
        g::par(
            g::send::<INSPECTION_SERVICE, SPECTROMETER, SpecResult>(),
            g::send::<INSPECTION_SERVICE, CENTRIFUGE, CentResult>(),
        ),
        g::par(
            g::send::<INSPECTION_SERVICE, FUME_HOOD, FumeResult>(),
            g::send::<INSPECTION_SERVICE, CHROMATOGRAPH, ChromaResult>(),
        ),
    );
    let returns = g::par(
        g::par(
            g::send::<SAFETY_OFFICER, SPEC_OWNER, Loan<SPECTROMETER>>(),
            g::send::<SAFETY_OFFICER, CENT_OWNER, Loan<CENTRIFUGE>>(),
        ),
        g::par(
            g::send::<SAFETY_OFFICER, FUME_OWNER, Loan<FUME_HOOD>>(),
            g::send::<SAFETY_OFFICER, CHROMA_OWNER, Loan<CHROMATOGRAPH>>(),
        ),
    );
    g::seq(
        g::send::<SAFETY_FILE, SAFETY_OFFICER, FileInspection>(),
        g::seq(
            g::send::<SAFETY_OFFICER, INSPECTION_SERVICE, InspectionAdmission>(),
            g::seq(
                loans,
                g::seq(
                    requests,
                    g::seq(
                        replies,
                        g::par(
                            returns,
                            g::send::<SAFETY_OFFICER, SAFETY_FILE, FileInspectionResult>(),
                        ),
                    ),
                ),
            ),
        ),
    )
}

// These are reused source fragments. Every application localside still writes
// its complete send/receive/loan/effect/return sequence directly.
fn spectrometry<const WORKER: u8, M: g::Message, R: g::Message>() -> impl Projectable {
    g::seq(
        g::send::<WORKER, SPEC_OWNER, M>(),
        g::seq(
            g::send::<SPEC_OWNER, WORKER, Loan<SPECTROMETER>>(),
            g::seq(
                g::send::<SPECTROMETER, SPEC_OWNER, M>(),
                g::seq(
                    g::send::<SPEC_OWNER, SPECTROMETER, R>(),
                    g::send::<WORKER, SPEC_OWNER, Loan<SPECTROMETER>>(),
                ),
            ),
        ),
    )
}

fn centrifugation<const WORKER: u8, M: g::Message, R: g::Message>() -> impl Projectable {
    g::seq(
        g::send::<WORKER, CENT_OWNER, M>(),
        g::seq(
            g::send::<CENT_OWNER, WORKER, Loan<CENTRIFUGE>>(),
            g::seq(
                g::send::<CENTRIFUGE, CENT_OWNER, M>(),
                g::seq(
                    g::send::<CENT_OWNER, CENTRIFUGE, R>(),
                    g::send::<WORKER, CENT_OWNER, Loan<CENTRIFUGE>>(),
                ),
            ),
        ),
    )
}

fn biology_preparation() -> impl Projectable {
    g::seq(
        g::send::<BIO_FILE, BIO_CONTROL, FileCells>(),
        g::seq(
            centrifugation::<BIO_CONTROL, Cells, CellsResult>(),
            g::seq(
                g::send::<BIO_CONTROL, BIO_FILE, FileCellsResult>(),
                g::seq(
                    g::send::<BIO_FILE, BIO_CONTROL, FileCalibration>(),
                    g::seq(
                        spectrometry::<BIO_CONTROL, Calibration, CalibrationResult>(),
                        g::send::<BIO_CONTROL, BIO_FILE, FileCalibrationResult>(),
                    ),
                ),
            ),
        ),
    )
}

fn compound_measurements() -> impl Projectable {
    g::seq(
        g::send::<CHEM_FILE, CHEM_CONTROL, FileAnalysis>(),
        g::seq(
            spectrometry::<CHEM_CONTROL, Analysis, AnalysisResult>(),
            g::seq(
                g::send::<CHEM_CONTROL, CHEM_FILE, FileAnalysisResult>(),
                g::seq(
                    g::send::<CHEM_FILE, CHEM_CONTROL, FilePurification>(),
                    g::seq(
                        centrifugation::<CHEM_CONTROL, Purification, PurificationResult>(),
                        g::seq(
                            g::send::<CHEM_CONTROL, CHEM_FILE, FilePurificationResult>(),
                            g::seq(
                                g::send::<CHEM_CONTROL, BIO_CONTROL, PurifiedMaterial>(),
                                g::send::<BIO_CONTROL, BIO_FILE, FileBioBatch>(),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    )
}

fn bioassay() -> impl Projectable {
    g::seq(
        g::send::<BIO_FILE, BIOLOGIST, FileBioassay>(),
        g::seq(
            g::send::<BIOLOGIST, CENT_OWNER, Assay>(),
            g::seq(
                g::send::<CENT_OWNER, BIOLOGIST, AssayResult>(),
                g::seq(
                    g::par(
                        g::seq(
                            g::send::<BIOLOGIST, ANALYST, AnalystMaterial>(),
                            g::send::<ANALYST, ANALYST_FILE, FileAnalystBatch>(),
                        ),
                        g::par(
                            g::send::<BIOLOGIST, QC, BiologyReport>(),
                            g::send::<BIOLOGIST, BIO_FILE, FileBioassayResult>(),
                        ),
                    ),
                    // The shell no longer has the assay authority during QC.
                    g::send::<BIO_CONTROL, QC, Loan<BIOLOGIST>>(),
                ),
            ),
        ),
    )
}

fn chromatography_and_molecular() -> impl Projectable {
    let chromatography = g::seq(
        g::send::<ANALYST_FILE, ANALYST, FileChromatography>(),
        g::seq(
            g::send::<ANALYST, CHROMA_OWNER, Chromatography>(),
            g::seq(
                g::send::<CHROMA_OWNER, ANALYST, Loan<CHROMATOGRAPH>>(),
                g::seq(
                    g::send::<CHROMATOGRAPH, CHROMA_OWNER, Chromatography>(),
                    g::seq(
                        g::send::<CHROMA_OWNER, CHROMATOGRAPH, ChromatographyResult>(),
                        g::seq(
                            g::send::<ANALYST, CHROMA_OWNER, Loan<CHROMATOGRAPH>>(),
                            g::send::<ANALYST, ANALYST_FILE, FileChromatographyResult>(),
                        ),
                    ),
                ),
            ),
        ),
    );
    g::seq(
        chromatography,
        g::seq(
            g::send::<ANALYST_FILE, ANALYST, FileMolecular>(),
            g::seq(
                spectrometry::<ANALYST, Molecular, MolecularResult>(),
                g::send::<ANALYST, ANALYST_FILE, FileMolecularResult>(),
            ),
        ),
    )
}

fn analyst_verification() -> impl Projectable {
    g::seq(
        g::send::<ANALYST_FILE, ANALYST, FileVerification>(),
        g::seq(
            centrifugation::<ANALYST, Verification, VerificationResult>(),
            g::send::<ANALYST, ANALYST_FILE, FileVerificationResult>(),
        ),
    )
}

fn publish_analyst() -> impl Projectable {
    g::seq(
        g::send::<BIO_FILE, ANALYST, FilePublishAnalyst>(),
        g::send::<ANALYST, BIO_FILE, FilePublishedAnalyst>(),
    )
}

fn publish_qc() -> impl Projectable {
    g::seq(
        g::send::<BIO_FILE, QC, FilePublishQc>(),
        g::send::<QC, BIO_FILE, FilePublishedQc>(),
    )
}

fn analyst_report() -> impl Projectable {
    // The actual report also fences old readers before another synthesis.
    g::par(
        g::send::<ANALYST, CHEM_CONTROL, AnalystReport>(),
        g::send::<ANALYST, QC, AnalystReportToQc>(),
    )
}

fn checked_quality(approved: impl Projectable, rejected: impl Projectable) -> impl Projectable {
    g::seq(
        g::send::<QC, QC_FILE, FileQcInput>(),
        g::seq(
            g::send::<QC_FILE, QC, FileQuality>(),
            quality_exchange(
                g::seq(
                    g::par(
                        g::seq(
                            g::send::<QC, BIO_CONTROL, BiologyApproved>(),
                            g::send::<BIO_CONTROL, BIO_FILE, FileBioBatch>(),
                        ),
                        g::seq(
                            g::send::<QC, CHROMA_OWNER, ChromaQualityApproved>(),
                            g::send::<CHROMA_OWNER, QC, ChromaQualityReceipt>(),
                        ),
                    ),
                    g::seq(g::send::<QC, QC_FILE, FileQualityResult>(), approved),
                ),
                g::seq(
                    g::par(
                        g::send::<QC, BIO_CONTROL, BiologyRejected>(),
                        g::seq(
                            g::send::<QC, CHROMA_OWNER, ChromaQualityRejected>(),
                            g::send::<CHROMA_OWNER, QC, ChromaQualityReceipt>(),
                        ),
                    ),
                    g::seq(g::send::<QC, QC_FILE, FileQualityResult>(), rejected),
                ),
            ),
        ),
    )
}

fn analytics_and_quality() -> impl Projectable {
    // The biologist's real publication chooses who starts first. Both arms
    // start at BIO_FILE; independent actors do not guess each other's choice.
    let analyst_first = g::seq(
        publish_analyst(),
        g::seq(
            chromatography_and_molecular(),
            g::seq(
                publish_qc(),
                checked_quality(
                    g::send::<QC, ANALYST, AnalystApproved>(),
                    g::send::<QC, ANALYST, AnalystRejected>(),
                ),
            ),
        ),
    );
    let qc_first = g::seq(
        publish_qc(),
        checked_quality(
            g::seq(
                g::send::<QC, ANALYST, AnalystApproved>(),
                g::seq(publish_analyst(), chromatography_and_molecular()),
            ),
            // A rejected old assay cannot start new chromatography.
            g::send::<QC, ANALYST, AnalystRejected>(),
        ),
    );
    g::seq(
        g::par(analyst_verification(), g::route(analyst_first, qc_first)),
        analyst_report(),
    )
}

fn scientific_round() -> impl Projectable {
    let batch = g::par(
        g::send::<FUME_OWNER, BIO_CONTROL, BiologyBatch>(),
        g::par(
            g::send::<FUME_OWNER, ANALYST, AnalystBatch>(),
            g::par(
                g::send::<FUME_OWNER, SPEC_OWNER, SpecBatch>(),
                g::par(
                    g::send::<FUME_OWNER, CENT_OWNER, CentBatch>(),
                    g::send::<FUME_OWNER, CHROMA_OWNER, ChromaBatch>(),
                ),
            ),
        ),
    );
    let failed_batch = g::par(
        g::send::<FUME_OWNER, BIO_CONTROL, BiologySynthesisFailed>(),
        g::par(
            g::send::<FUME_OWNER, ANALYST, AnalystSynthesisFailed>(),
            g::par(
                g::send::<FUME_OWNER, SPEC_OWNER, SpecSynthesisFailed>(),
                g::par(
                    g::send::<FUME_OWNER, CENT_OWNER, CentSynthesisFailed>(),
                    g::send::<FUME_OWNER, CHROMA_OWNER, ChromaSynthesisFailed>(),
                ),
            ),
        ),
    );
    let exhausted_batch = g::par(
        g::send::<FUME_OWNER, BIO_CONTROL, BiologyReagentsExhausted>(),
        g::par(
            g::send::<FUME_OWNER, ANALYST, AnalystReagentsExhausted>(),
            g::par(
                g::send::<FUME_OWNER, SPEC_OWNER, SpecReagentsExhausted>(),
                g::par(
                    g::send::<FUME_OWNER, CENT_OWNER, CentReagentsExhausted>(),
                    g::send::<FUME_OWNER, CHROMA_OWNER, ChromaReagentsExhausted>(),
                ),
            ),
        ),
    );
    let finished = g::seq(
        g::send::<FUME_OWNER, CHEMIST, SynthesisFinished>(),
        g::seq(
            g::send::<FUME_OWNER, QC, SynthesisFinished>(),
            g::seq(
                batch,
                g::seq(
                    // Custody closes edits while measurements and QC read this batch.
                    g::send::<CHEM_CONTROL, QC, Loan<CHEMIST>>(),
                    g::seq(
                        g::send::<FUME_OWNER, QC, Loan<REAGENT_SUPPLY>>(),
                        g::seq(
                            compound_measurements(),
                            g::seq(bioassay(), analytics_and_quality()),
                        ),
                    ),
                ),
            ),
        ),
    );
    let exhausted = g::seq(
        g::send::<FUME_OWNER, CHEMIST, ReagentsExhausted>(),
        g::seq(
            g::send::<FUME_OWNER, QC, ReagentsExhausted>(),
            g::seq(
                exhausted_batch,
                g::seq(
                    g::par(
                        g::send::<CHEM_CONTROL, QC, Loan<CHEMIST>>(),
                        g::send::<BIO_CONTROL, QC, Loan<BIOLOGIST>>(),
                    ),
                    g::send::<FUME_OWNER, QC, Loan<REAGENT_SUPPLY>>(),
                ),
            ),
        ),
    );
    let failed = g::seq(
        g::send::<FUME_OWNER, CHEMIST, SynthesisFailed>(),
        g::seq(
            g::send::<FUME_OWNER, QC, SynthesisFailed>(),
            g::seq(
                failed_batch,
                g::seq(
                    g::par(
                        g::send::<CHEM_CONTROL, QC, Loan<CHEMIST>>(),
                        g::send::<BIO_CONTROL, QC, Loan<BIOLOGIST>>(),
                    ),
                    g::send::<FUME_OWNER, QC, Loan<REAGENT_SUPPLY>>(),
                ),
            ),
        ),
    );
    g::route(
        finished,
        g::route(failed, exhausted).resolve::<SYNTHESIS_FAILURE>(),
    )
    .resolve::<SYNTHESIS_OUTCOME>()
}

fn approved_notice() -> impl Projectable {
    g::seq(
        g::send::<QC, CHEM_CONTROL, FinalApproved>(),
        g::seq(
            g::send::<CHEM_CONTROL, CHEM_FILE, FileChemQuality>(),
            g::par(
                g::send::<QC, BIO_CONTROL, BiologyFinalApproved>(),
                g::par(
                    g::seq(
                        g::send::<QC, ANALYST, AnalystFinalApproved>(),
                        g::send::<ANALYST, ANALYST_FILE, FileAnalystBatch>(),
                    ),
                    g::par(
                        g::send::<QC, FUME_OWNER, FumeFinalApproved>(),
                        g::par(
                            g::send::<QC, SPEC_OWNER, SpecFinalApproved>(),
                            g::par(
                                g::send::<QC, CENT_OWNER, CentFinalApproved>(),
                                g::send::<QC, CHROMA_OWNER, ChromaFinalApproved>(),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    )
}

fn rejected_notice() -> impl Projectable {
    g::seq(
        g::route(
            g::send::<QC, CHEM_CONTROL, SynthesisFailed>(),
            g::seq(
                g::send::<QC, CHEM_CONTROL, ReSynthesize>(),
                g::send::<CHEM_CONTROL, CHEM_FILE, FileChemQuality>(),
            ),
        ),
        g::par(
            g::seq(
                g::send::<QC, BIO_CONTROL, BiologyReSynthesize>(),
                g::send::<BIO_CONTROL, BIO_FILE, FileBioBatch>(),
            ),
            g::par(
                g::seq(
                    g::send::<QC, ANALYST, AnalystReSynthesize>(),
                    g::send::<ANALYST, ANALYST_FILE, FileAnalystBatch>(),
                ),
                g::par(
                    g::send::<QC, FUME_OWNER, FumeReSynthesize>(),
                    g::par(
                        g::send::<QC, SPEC_OWNER, SpecReSynthesize>(),
                        g::par(
                            g::send::<QC, CENT_OWNER, CentReSynthesize>(),
                            g::send::<QC, CHROMA_OWNER, ChromaReSynthesize>(),
                        ),
                    ),
                ),
            ),
        ),
    )
}

fn exhausted_notice() -> impl Projectable {
    g::seq(
        g::send::<QC, CHEM_CONTROL, FinalExhausted>(),
        g::par(
            g::send::<QC, BIO_CONTROL, BiologyFinalExhausted>(),
            g::par(
                g::send::<QC, ANALYST, AnalystFinalExhausted>(),
                g::par(
                    g::send::<QC, FUME_OWNER, FumeFinalExhausted>(),
                    g::par(
                        g::send::<QC, SPEC_OWNER, SpecFinalExhausted>(),
                        g::par(
                            g::send::<QC, CENT_OWNER, CentFinalExhausted>(),
                            g::send::<QC, CHROMA_OWNER, ChromaFinalExhausted>(),
                        ),
                    ),
                ),
            ),
        ),
    )
}

fn retire_scientific_authority() -> impl Projectable {
    g::par(
        g::send::<QC, INIT, Loan<CHEMIST>>(),
        g::par(
            g::send::<QC, INIT, Loan<BIOLOGIST>>(),
            g::send::<QC, INIT, Loan<REAGENT_SUPPLY>>(),
        ),
    )
}

fn finish_science() -> impl Projectable {
    let approved = g::seq(
        approved_notice(),
        g::seq(
            g::send::<CHEM_FILE, CHEM_CONTROL, FileFinal>(),
            g::seq(
                spectrometry::<CHEM_CONTROL, FinalValidation, FinalResult>(),
                g::par(
                    g::send::<CHEM_CONTROL, CHEM_FILE, FileFinalResult>(),
                    g::send::<CHEM_CONTROL, QC, FinalValidated>(),
                ),
            ),
        ),
    );
    let exhausted = g::seq(
        exhausted_notice(),
        g::par(
            g::send::<QC, BIO_FILE, FileBioBatch>(),
            g::par(
                g::send::<QC, ANALYST_FILE, FileAnalystBatch>(),
                g::send::<QC, QC_FILE, FileQcQuality>(),
            ),
        ),
    );
    g::route(approved, exhausted)
}

/// The scientific workflow and the five actual WASI participants.
pub fn science() -> impl Projectable {
    let resources = g::par(
        g::send::<INIT, SPEC_OWNER, Loan<SPECTROMETER>>(),
        g::par(
            g::send::<INIT, CENT_OWNER, Loan<CENTRIFUGE>>(),
            g::par(
                g::send::<INIT, FUME_OWNER, Loan<FUME_HOOD>>(),
                g::par(
                    g::send::<INIT, CHROMA_OWNER, Loan<CHROMATOGRAPH>>(),
                    g::send::<INIT, FUME_OWNER, Loan<REAGENT_SUPPLY>>(),
                ),
            ),
        ),
    );
    let files = g::par(
        g::send::<CHEM_DATA, CHEM_CONTROL, Loan<CHEM_FILE>>(),
        g::par(
            g::send::<BIO_DATA, BIO_CONTROL, Loan<BIO_FILE>>(),
            g::par(
                g::send::<ANALYST_DATA, ANALYST, Loan<ANALYST_FILE>>(),
                g::par(
                    g::send::<QC_DATA, QC, Loan<QC_FILE>>(),
                    g::send::<SAFETY_DATA, SAFETY_OFFICER, Loan<SAFETY_FILE>>(),
                ),
            ),
        ),
    );
    let access = g::par(
        g::send::<INIT, CHEM_CONTROL, Loan<CHEMIST>>(),
        g::send::<INIT, BIO_CONTROL, Loan<BIOLOGIST>>(),
    );
    let guests = g::par(
        g::send::<INIT, CHEM_CONTROL, Loan<CHEM_VM>>(),
        g::par(
            g::send::<INIT, BIO_CONTROL, Loan<BIO_VM>>(),
            g::par(
                g::send::<INIT, ANALYST, Loan<ANALYST_VM>>(),
                g::par(
                    g::send::<INIT, QC, Loan<QC_VM>>(),
                    g::send::<INIT, SAFETY_OFFICER, Loan<SAFETY_VM>>(),
                ),
            ),
        ),
    );
    let revise = || {
        g::seq(
            rejected_notice(),
            g::seq(
                g::par(
                    g::send::<QC, CHEM_CONTROL, Loan<CHEMIST>>(),
                    g::par(
                        g::send::<QC, BIO_CONTROL, Loan<BIOLOGIST>>(),
                        g::send::<QC, FUME_OWNER, Loan<REAGENT_SUPPLY>>(),
                    ),
                ),
                g::seq(synthesis_attempt(), scientific_round()),
            ),
        )
    };
    // The task has three consumable units. When all three attempts need
    // rework, the fourth request returns the actual exhaustion result.
    let after_third = g::route(finish_science(), g::seq(revise(), finish_science()));
    let after_second = g::route(finish_science(), g::seq(revise(), after_third));
    let after_first = g::route(finish_science(), g::seq(revise(), after_second));
    let scientific_task = g::seq(
        scientific_round(),
        g::seq(after_first, retire_scientific_authority()),
    );
    let task = g::seq(
        // Fume synthesis is independent of cell preparation/calibration.
        g::par(synthesis_attempt(), biology_preparation()),
        g::seq(inspection(), scientific_task),
    );
    let metadata = g::par(
        g::seq(
            file_metadata::<CHEM_FILE, CHEM_DATA>().roll(),
            g::send::<CHEM_FILE, CHEM_DATA, FileExit>(),
        ),
        g::par(
            file_metadata::<BIO_FILE, BIO_DATA>().roll(),
            g::par(
                file_metadata::<ANALYST_FILE, ANALYST_DATA>().roll(),
                g::par(
                    file_metadata::<QC_FILE, QC_DATA>().roll(),
                    g::seq(
                        file_metadata::<SAFETY_FILE, SAFETY_DATA>().roll(),
                        g::send::<SAFETY_FILE, SAFETY_DATA, FileExit>(),
                    ),
                ),
            ),
        ),
    );
    let file_exits = g::par(
        g::send::<BIO_FILE, BIO_DATA, FileExit>(),
        g::par(
            g::send::<ANALYST_FILE, ANALYST_DATA, FileExit>(),
            g::send::<QC_FILE, QC_DATA, FileExit>(),
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
                    private_io::<ANALYST_VM, ANALYST_IO>(),
                    g::send::<ANALYST_VM, ANALYST_IO, Exit>(),
                ),
                g::par(
                    g::seq(
                        private_io::<QC_VM, QC_IO>(),
                        g::send::<QC_VM, QC_IO, Exit>(),
                    ),
                    g::seq(
                        private_io::<SAFETY_VM, SAFETY_IO>(),
                        g::send::<SAFETY_VM, SAFETY_IO, Exit>(),
                    ),
                ),
            ),
        ),
    );
    g::seq(
        g::par(resources, g::par(files, g::par(access, guests))),
        g::par(g::seq(g::par(task, metadata), file_exits), io),
    )
}
