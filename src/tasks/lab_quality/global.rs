//! 13M direct operation files, scientific custody and result-driven reconsideration.
use super::{arguments::Arguments, imp::QcReply};
pub use crate::shell::global::{Exit, FileExit, Loan};
use crate::shell::imp::review_reply::ReviewReply;
use hibana::{
    Endpoint, g,
    runtime::program::{Projectable, RoleProgram, project},
};
use hibana_wasip1_runtime::{
    choreofs::{ChoreoFsObject, FdSpec, ObjectId},
    protocol,
};
pub const CHEMIST: u8 = 0;
pub const BIOLOGIST: u8 = 1;
pub const ANALYST: u8 = 2;
pub const QC: u8 = 3;
pub const CHEMIST_IO: u8 = 4;
pub const BIOLOGIST_IO: u8 = 5;
pub const ANALYST_IO: u8 = 6;
pub const QC_IO: u8 = 7;
pub const SPEC_SERVICE: u8 = 8;
pub const CENT_SERVICE: u8 = 9;
pub const QC_SERVICE: u8 = 10;
pub const SPECTROMETER: u8 = 11;
pub const CENTRIFUGE: u8 = 12;
pub const FUME_HOOD: u8 = 13;
pub const CHEMISTRY: u8 = 14;
pub const BIOLOGY: u8 = 15;
pub const INIT: u8 = 16;
pub const CHEM_FILE: u8 = 32;
pub const BIO_FILE: u8 = 33;
pub const ANALYST_FILE: u8 = 34;
pub const QC_FILE: u8 = 35;
pub const CHEM_VM: u8 = 64;
pub const BIO_VM: u8 = 65;
pub const ANALYST_VM: u8 = 66;
pub const QC_VM: u8 = 67;
pub const CHEM_DATA: u8 = 96;
pub const BIO_DATA: u8 = 97;
pub const ANALYST_DATA: u8 = 98;
pub const QC_DATA: u8 = 99;
pub const SESSION: u32 = 131;
pub const QC_SUCCESS: u16 = 1311;
pub const QC_SUBJECT: u16 = 1312;
pub const BIO_SUCCESS: u16 = 1313;

pub type SpecLoan<'r> = g::Msg<1, Endpoint<'r, SPECTROMETER>>;
pub type CentLoan<'r> = g::Msg<10, Endpoint<'r, CENTRIFUGE>>;
pub type FumeLoan<'r> = g::Msg<2, Endpoint<'r, FUME_HOOD>>;
pub type ChemAccess<'r> = g::Msg<24, Endpoint<'r, CHEMISTRY>>;
pub type ApprovedResults<'a> = g::Msg<25, &'a [u8]>;
pub type SynthesisRequest<'a> = g::Msg<26, &'a [u8]>;
pub type SpecRequest<'a> = g::Msg<27, &'a [u8]>;
pub type CentRequest<'a> = g::Msg<28, &'a [u8]>;
pub type InstrumentResult<'a> = g::Msg<29, &'a [u8]>;
pub type Synthesis<'a> = g::Msg<30, &'a [u8]>;
pub type InitialAnalysis<'a> = g::Msg<31, &'a [u8]>;
pub type Purification<'a> = g::Msg<32, &'a [u8]>;
pub type Preparation<'a> = g::Msg<33, &'a [u8]>;
pub type Calibration<'a> = g::Msg<34, &'a [u8]>;
pub type Reanalysis<'a> = g::Msg<35, &'a [u8]>;
pub type Molecular<'a> = g::Msg<36, &'a [u8]>;
pub type Separation<'a> = g::Msg<37, &'a [u8]>;
pub type FinalValidation<'a> = g::Msg<38, &'a [u8]>;
pub type BioAccess<'r> = g::Msg<39, Endpoint<'r, BIOLOGY>>;
pub type Sample<'a> = g::Msg<40, &'a [u8]>;
pub type ChemistReport<'a> = g::Msg<42, &'a [u8]>;
pub type BiologyReport<'a> = g::Msg<43, &'a [u8]>;
pub type BioRequest<'a> = g::Msg<44, &'a [u8]>;
pub type BioPassed<'a> = g::Msg<45, ReviewReply<'a, true>>;
pub type BioFailed<'a> = g::Msg<46, ReviewReply<'a, false>>;
pub type AnalysisProof<'a> = g::Msg<47, &'a [u8]>;
pub type PurificationProof<'a> = g::Msg<48, &'a [u8]>;
pub type CalibrationProof<'a> = g::Msg<49, &'a [u8]>;
pub type ApprovedBiology<'a> = g::Msg<50, &'a [u8]>;
pub type Quality<'a> = g::Msg<51, &'a [u8]>;
pub type QualityForOwner<'a> = g::Msg<52, &'a [u8]>;
pub type VerificationProof<'a> = g::Msg<53, &'a [u8]>;
pub type BioFumePassed<'a> = g::Msg<55, ReviewReply<'a, true>>;
pub type BioFumeFailed<'a> = g::Msg<56, ReviewReply<'a, false>>;
pub type BioSpecPassed<'a> = g::Msg<57, ReviewReply<'a, true>>;
pub type BioSpecFailed<'a> = g::Msg<58, ReviewReply<'a, false>>;
pub type QcAdmission<'a> = g::Msg<3, &'a [u8]>;
pub type QcSpecRequest<'a> = g::Msg<4, &'a [u8]>;
pub type QcFumeRequest<'a> = g::Msg<5, &'a [u8]>;
pub type QcSpecResult<'a> = g::Msg<6, &'a [u8]>;
pub type QcFumeResult<'a> = g::Msg<7, &'a [u8]>;
pub type FirstChemist<'a> = g::Msg<70, &'a [u8]>;
pub type FirstBiologist<'a> = g::Msg<71, &'a [u8]>;
pub type NextChemist<'a> = g::Msg<72, &'a [u8]>;
pub type NextBiologist<'a> = g::Msg<73, &'a [u8]>;
pub type LastChemist<'a> = g::Msg<74, &'a [u8]>;
pub type LastBiologist<'a> = g::Msg<75, &'a [u8]>;
pub type ChemistApproved<'a> = g::Msg<100, QcReply<'a, CHEMIST, true>>;
pub type ChemistRejected<'a> = g::Msg<101, QcReply<'a, CHEMIST, false>>;
pub type BiologistApproved<'a> = g::Msg<102, QcReply<'a, BIOLOGIST, true>>;
pub type BiologistRejected<'a> = g::Msg<103, QcReply<'a, BIOLOGIST, false>>;
pub type LastChemistApproved<'a> = g::Msg<104, QcReply<'a, CHEMIST, true>>;
pub type LastBiologistApproved<'a> = g::Msg<105, QcReply<'a, BIOLOGIST, true>>;
pub type LastChemistRejected<'a> = g::Msg<106, QcReply<'a, CHEMIST, false>>;
pub type LastBiologistRejected<'a> = g::Msg<107, QcReply<'a, BIOLOGIST, false>>;
pub type Retest<'a> = g::Msg<120, &'a [u8]>;
pub type RetestPassed<'a> = g::Msg<121, ReviewReply<'a, true>>;
pub type RetestFailed<'a> = g::Msg<122, ReviewReply<'a, false>>;
pub type RetestFumePassed<'a> = g::Msg<123, ReviewReply<'a, true>>;
pub type RetestFumeFailed<'a> = g::Msg<124, ReviewReply<'a, false>>;
pub type RetestSpecPassed<'a> = g::Msg<125, ReviewReply<'a, true>>;
pub type RetestSpecFailed<'a> = g::Msg<126, ReviewReply<'a, false>>;
pub type LastRetest<'a> = g::Msg<130, &'a [u8]>;
pub type LastRetestPassed<'a> = g::Msg<131, ReviewReply<'a, true>>;
pub type LastRetestFailed<'a> = g::Msg<132, ReviewReply<'a, false>>;
pub type LastRetestFumePassed<'a> = g::Msg<133, ReviewReply<'a, true>>;
pub type LastRetestFumeFailed<'a> = g::Msg<134, ReviewReply<'a, false>>;
pub type LastRetestSpecPassed<'a> = g::Msg<135, ReviewReply<'a, true>>;
pub type LastRetestSpecFailed<'a> = g::Msg<136, ReviewReply<'a, false>>;
pub type LastReanalysis<'a> = g::Msg<137, &'a [u8]>;

pub type FileSynthesis = g::Msg<170, Arguments<0>>;
pub type FileAnalysis = g::Msg<171, Arguments<1>>;
pub type FilePurification = g::Msg<172, Arguments<2>>;
pub type FileCells = g::Msg<173, Arguments<3>>;
pub type FileCalibration = g::Msg<174, Arguments<4>>;
pub type FileBioassay = g::Msg<175, Arguments<5>>;
pub type FileMolecular = g::Msg<176, Arguments<6>>;
pub type FileVerification = g::Msg<177, Arguments<7>>;
pub type FileFinal = g::Msg<178, Arguments<8>>;
pub type FileQcChemist = g::Msg<179, Arguments<9>>;
pub type FileQcBiologist = g::Msg<180, Arguments<10>>;
pub type FileSynthesisResult<'a> = g::Msg<181, &'a [u8]>;
pub type FileAnalysisResult<'a> = g::Msg<182, &'a [u8]>;
pub type FilePurificationResult<'a> = g::Msg<183, &'a [u8]>;
pub type FileCellsResult<'a> = g::Msg<184, &'a [u8]>;
pub type FileCalibrationResult<'a> = g::Msg<185, &'a [u8]>;
pub type FileBioassayResult<'a> = g::Msg<186, &'a [u8]>;
pub type FileMolecularResult<'a> = g::Msg<187, &'a [u8]>;
pub type FileVerificationResult<'a> = g::Msg<188, &'a [u8]>;
pub type FileFinalResult<'a> = g::Msg<189, &'a [u8]>;
pub type FileQcChemistResult<'a> = g::Msg<190, &'a [u8]>;
pub type FileQcBiologistResult<'a> = g::Msg<191, &'a [u8]>;
pub type FileSample<'a> = g::Msg<192, &'a [u8]>;
pub type FileFeedback<'a> = g::Msg<193, &'a [u8]>;
pub type FileAssay<'a> = g::Msg<194, &'a [u8]>;
pub type FileQuality<'a> = g::Msg<195, &'a [u8]>;
fn qc_requests_and_replies() -> impl Projectable {
    g::seq(
        g::par(
            g::send::<SPECTROMETER, QC_SERVICE, QcSpecRequest>(),
            g::send::<FUME_HOOD, QC_SERVICE, QcFumeRequest>(),
        ),
        g::par(
            g::send::<QC_SERVICE, SPECTROMETER, QcSpecResult>(),
            g::send::<QC_SERVICE, FUME_HOOD, QcFumeResult>(),
        ),
    )
}
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
pub const CHEM_FILES: [ChoreoFsObject<'static>; 6] = [
    writable(b"service/run_synthesis", 4),
    writable(b"service/run_analysis/synthesized_compound", 5),
    writable(b"service/run_separation/compound_purification", 6),
    incoming(b"service/qc", 7),
    incoming(b"service/quality", 8),
    writable(b"service/run_analysis/final_validation", 9),
];
pub const BIO_FILES: [ChoreoFsObject<'static>; 5] = [
    writable(b"service/run_separation/cell_cultures", 4),
    writable(b"service/run_analysis/fluorescence", 5),
    incoming(b"service/sample", 6),
    writable(b"service/run_bioassay", 7),
    incoming(b"service/qc", 8),
];
pub const ANALYST_FILES: [ChoreoFsObject<'static>; 3] = [
    incoming(b"service/bioassay", 4),
    writable(b"service/run_analysis/molecular_detail", 5),
    writable(b"service/run_separation/verification", 6),
];
pub const QC_FILES: [ChoreoFsObject<'static>; 2] = [
    writable(b"service/verify_result/CHEMIST", 4),
    writable(b"service/verify_result/BIOLOGIST", 5),
];

fn analysis<const WHO: u8, const LABEL: u8>() -> impl Projectable {
    g::seq(
        g::send::<WHO, SPEC_SERVICE, g::Msg<LABEL, &[u8]>>(),
        g::seq(
            g::send::<SPEC_SERVICE, WHO, SpecLoan>(),
            g::seq(
                g::send::<SPECTROMETER, SPEC_SERVICE, SpecRequest>(),
                g::seq(
                    g::send::<SPEC_SERVICE, SPECTROMETER, InstrumentResult>(),
                    g::send::<WHO, SPEC_SERVICE, SpecLoan>(),
                ),
            ),
        ),
    )
}
fn measurement<const LABEL: u8>() -> impl Projectable {
    g::seq(
        g::send::<CHEM_FILE, CHEMISTRY, FileAnalysis>(),
        g::seq(
            analysis::<CHEMISTRY, LABEL>(),
            g::seq(
                g::send::<CHEMISTRY, CHEM_FILE, FileAnalysisResult>(),
                g::send::<SPEC_SERVICE, QC_SERVICE, AnalysisProof>(),
            ),
        ),
    )
}
fn remeasure<const LABEL: u8>() -> impl Projectable {
    g::seq(measurement::<LABEL>(), g::send::<CHEMIST, QC, ChemAccess>())
}
pub(super) fn separation<const WHO: u8, const LABEL: u8>() -> impl Projectable {
    g::seq(
        g::send::<WHO, CENT_SERVICE, g::Msg<LABEL, &[u8]>>(),
        g::seq(
            g::send::<CENT_SERVICE, WHO, CentLoan>(),
            g::seq(
                g::send::<CENTRIFUGE, CENT_SERVICE, CentRequest>(),
                g::seq(
                    g::send::<CENT_SERVICE, CENTRIFUGE, InstrumentResult>(),
                    g::send::<WHO, CENT_SERVICE, CentLoan>(),
                ),
            ),
        ),
    )
}
fn synthesis() -> impl Projectable {
    g::seq(
        g::send::<CHEM_FILE, CHEMISTRY, FileSynthesis>(),
        g::seq(
            g::send::<CHEMISTRY, QC_SERVICE, Synthesis>(),
            g::seq(
                g::send::<QC_SERVICE, CHEMISTRY, FumeLoan>(),
                g::seq(
                    g::send::<FUME_HOOD, QC_SERVICE, SynthesisRequest>(),
                    g::seq(
                        g::send::<QC_SERVICE, FUME_HOOD, InstrumentResult>(),
                        g::seq(
                            g::send::<CHEMISTRY, QC_SERVICE, FumeLoan>(),
                            g::send::<CHEMISTRY, CHEM_FILE, FileSynthesisResult>(),
                        ),
                    ),
                ),
            ),
        ),
    )
}
fn bio_reply<M: g::Message, F: g::Message, S: g::Message>() -> impl Projectable {
    g::seq(
        g::send::<BIOLOGY, BIOLOGIST, M>(),
        g::seq(
            g::send::<BIOLOGIST, BIO_FILE, FileBioassayResult>(),
            g::par(
                g::send::<CENT_SERVICE, QC_SERVICE, F>(),
                g::send::<CENT_SERVICE, SPEC_SERVICE, S>(),
            ),
        ),
    )
}
fn bio_until_success<Q: Projectable, P: Projectable, N: Projectable>(
    request: impl Fn() -> Q,
    passed: impl Fn() -> P,
    failed: impl Fn() -> N,
) -> impl Projectable {
    // Each actual trial submits the specimen continuation to the service.
    // A failed result returns it for a new trial; success sends it to QC.
    // Thus the service cannot restart an old trial after giving up custody.
    let request = || {
        g::seq(
            g::send::<BIO_FILE, BIOLOGY, FileBioassay>(),
            g::seq(request(), g::send::<BIOLOGIST, CENT_SERVICE, BioAccess>()),
        )
    };
    let first = request();
    let retry = g::seq(
        failed(),
        g::seq(g::send::<CENT_SERVICE, BIOLOGIST, BioAccess>(), request()),
    );
    g::seq(
        first,
        g::seq(
            g::route(passed(), g::seq(retry.roll(), passed())).resolve::<BIO_SUCCESS>(),
            g::send::<CENT_SERVICE, QC, BioAccess>(),
        ),
    )
}
pub(super) fn initial_biology() -> impl Projectable {
    bio_until_success(
        g::send::<BIOLOGIST, CENT_SERVICE, BioRequest>,
        bio_reply::<BioPassed, BioFumePassed, BioSpecPassed>,
        bio_reply::<BioFailed, BioFumeFailed, BioSpecFailed>,
    )
}
fn repeated_biology() -> impl Projectable {
    bio_until_success(
        g::send::<BIOLOGIST, CENT_SERVICE, Retest>,
        bio_reply::<RetestPassed, RetestFumePassed, RetestSpecPassed>,
        bio_reply::<RetestFailed, RetestFumeFailed, RetestSpecFailed>,
    )
}
fn last_biology() -> impl Projectable {
    bio_until_success(
        g::send::<BIOLOGIST, CENT_SERVICE, LastRetest>,
        bio_reply::<LastRetestPassed, LastRetestFumePassed, LastRetestSpecPassed>,
        bio_reply::<LastRetestFailed, LastRetestFumeFailed, LastRetestSpecFailed>,
    )
}
fn select<const WHO: u8, const LABEL: u8, M: g::Message>() -> impl Projectable {
    g::seq(
        g::send::<QC_FILE, QC, M>(),
        g::seq(
            g::send::<QC, SPEC_SERVICE, g::Msg<LABEL, &[u8]>>(),
            g::seq(
                g::send::<QC, QC_SERVICE, g::Msg<LABEL, &[u8]>>(),
                g::send::<QC, WHO, g::Msg<LABEL, &[u8]>>(),
            ),
        ),
    )
}
fn select_both<const LABEL: u8, M: g::Message>() -> impl Projectable {
    g::seq(
        g::send::<QC_FILE, QC, M>(),
        g::seq(
            g::send::<QC, SPEC_SERVICE, g::Msg<LABEL, &[u8]>>(),
            g::seq(
                g::send::<QC, QC_SERVICE, g::Msg<LABEL, &[u8]>>(),
                g::seq(
                    g::send::<QC, CHEMIST, g::Msg<LABEL, &[u8]>>(),
                    g::send::<QC, BIOLOGIST, g::Msg<LABEL, &[u8]>>(),
                ),
            ),
        ),
    )
}
fn qc_resources() -> impl Projectable {
    g::seq(
        g::par(
            g::send::<SPEC_SERVICE, QC, SpecLoan>(),
            g::send::<QC_SERVICE, QC, FumeLoan>(),
        ),
        g::seq(
            qc_requests_and_replies(),
            g::par(
                g::send::<QC, SPEC_SERVICE, SpecLoan>(),
                g::send::<QC, QC_SERVICE, FumeLoan>(),
            ),
        ),
    )
}
fn qc_chemist() -> impl Projectable {
    g::seq(
        g::send::<CHEMIST, QC, ChemistReport>(),
        g::seq(
            g::send::<CHEMISTRY, QC_SERVICE, QcAdmission>(),
            qc_resources(),
        ),
    )
}
fn qc_biologist() -> impl Projectable {
    g::seq(
        g::send::<BIOLOGIST, QC, BiologyReport>(),
        g::seq(
            g::send::<BIOLOGY, QC_SERVICE, QcAdmission>(),
            qc_resources(),
        ),
    )
}
// Both scientists have submitted immutable candidates. The receiver of the
// actual verification effect owns those continuations while deciding its result.
fn submit_candidates() -> impl Projectable {
    g::par(
        g::send::<QC, QC_SERVICE, ChemAccess>(),
        g::send::<QC, QC_SERVICE, BioAccess>(),
    )
}
fn return_candidates() -> impl Projectable {
    g::par(
        g::send::<QC_SERVICE, QC, ChemAccess>(),
        g::send::<QC_SERVICE, QC, BioAccess>(),
    )
}
// Share the actual subject-bound verdict, not recipient-specific control tokens.
// Sends queue the result; they do not wait for recipients to finish their work.
fn announce<M: g::Message, R: g::Message>(held: impl Projectable) -> impl Projectable {
    g::seq(
        g::send::<QC_SERVICE, QC, M>(),
        g::seq(
            held,
            g::seq(
                g::send::<QC, CHEMIST, M>(),
                g::seq(
                    g::send::<QC, BIOLOGIST, M>(),
                    g::seq(
                        g::send::<QC, SPEC_SERVICE, M>(),
                        g::seq(
                            g::send::<QC, CENT_SERVICE, M>(),
                            g::send::<QC, QC_FILE, R>(),
                        ),
                    ),
                ),
            ),
        ),
    )
}
fn reject_chemist() -> impl Projectable {
    g::seq(
        announce::<ChemistRejected, FileQcChemistResult>(return_candidates()),
        g::seq(
            g::send::<QC, CHEMIST, ChemAccess>(),
            g::send::<CHEMIST, CHEM_FILE, FileFeedback>(),
        ),
    )
}
fn reject_biologist() -> impl Projectable {
    g::seq(
        announce::<BiologistRejected, FileQcBiologistResult>(return_candidates()),
        g::seq(
            g::send::<QC, BIOLOGIST, BioAccess>(),
            g::send::<BIOLOGIST, BIO_FILE, FileFeedback>(),
        ),
    )
}
fn last_chemist() -> impl Projectable {
    let verify = || {
        g::seq(
            g::seq(select::<CHEMIST, 74, FileQcChemist>(), qc_chemist()),
            g::send::<QC, QC_SERVICE, ChemAccess>(),
        )
    };
    let approved = || {
        g::seq(
            announce::<LastChemistApproved, FileQcChemistResult>(g::send::<
                QC_SERVICE,
                QC,
                ChemAccess,
            >()),
            g::seq(
                g::send::<QC, INIT, ChemAccess>(),
                g::send::<CHEMIST, CHEM_FILE, FileFeedback>(),
            ),
        )
    };
    let rejected = g::seq(
        announce::<LastChemistRejected, FileQcChemistResult>(
            g::send::<QC_SERVICE, QC, ChemAccess>(),
        ),
        g::seq(
            g::send::<QC, CHEMIST, ChemAccess>(),
            g::send::<CHEMIST, CHEM_FILE, FileFeedback>(),
        ),
    );
    g::seq(
        verify(),
        g::route(
            approved(),
            g::seq(
                g::seq(rejected, g::seq(remeasure::<137>(), verify())).roll(),
                approved(),
            ),
        )
        .resolve::<QC_SUCCESS>(),
    )
}
fn last_biologist() -> impl Projectable {
    let verify = || {
        g::seq(
            g::seq(select::<BIOLOGIST, 75, FileQcBiologist>(), qc_biologist()),
            g::send::<QC, QC_SERVICE, BioAccess>(),
        )
    };
    let approved = || {
        g::seq(
            announce::<LastBiologistApproved, FileQcBiologistResult>(g::send::<
                QC_SERVICE,
                QC,
                BioAccess,
            >()),
            g::seq(
                g::send::<QC, INIT, BioAccess>(),
                g::send::<BIOLOGIST, BIO_FILE, FileFeedback>(),
            ),
        )
    };
    let rejected = g::seq(
        announce::<LastBiologistRejected, FileQcBiologistResult>(g::send::<
            QC_SERVICE,
            QC,
            BioAccess,
        >()),
        g::seq(
            g::send::<QC, BIOLOGIST, BioAccess>(),
            g::send::<BIOLOGIST, BIO_FILE, FileFeedback>(),
        ),
    );
    g::seq(
        verify(),
        g::route(
            approved(),
            g::seq(
                g::seq(rejected, g::seq(last_biology(), verify())).roll(),
                approved(),
            ),
        )
        .resolve::<QC_SUCCESS>(),
    )
}
fn positive_then_remaining() -> impl Projectable {
    g::route(
        g::seq(
            announce::<ChemistApproved, FileQcChemistResult>(return_candidates()),
            g::seq(
                g::send::<QC, INIT, ChemAccess>(),
                g::seq(
                    g::send::<CHEMIST, CHEM_FILE, FileFeedback>(),
                    last_biologist(),
                ),
            ),
        ),
        g::seq(
            announce::<BiologistApproved, FileQcBiologistResult>(return_candidates()),
            g::seq(
                g::send::<QC, INIT, BioAccess>(),
                g::seq(
                    g::send::<BIOLOGIST, BIO_FILE, FileFeedback>(),
                    last_chemist(),
                ),
            ),
        ),
    )
    .resolve::<QC_SUBJECT>()
}
fn next_qc() -> impl Projectable {
    g::seq(
        g::route(
            g::seq(select_both::<72, FileQcChemist>(), qc_chemist()),
            g::seq(select_both::<73, FileQcBiologist>(), qc_biologist()),
        ),
        submit_candidates(),
    )
}
fn next_qc_during_biology() -> impl Projectable {
    g::seq(
        g::route(
            g::seq(
                select_both::<72, FileQcChemist>(),
                g::par(repeated_biology(), qc_chemist()),
            ),
            g::seq(
                select_both::<73, FileQcBiologist>(),
                g::seq(repeated_biology(), qc_biologist()),
            ),
        ),
        submit_candidates(),
    )
}
fn results_until_approved() -> impl Projectable {
    let retry = g::route(
        g::seq(reject_chemist(), g::seq(remeasure::<35>(), next_qc())),
        g::seq(reject_biologist(), next_qc_during_biology()),
    )
    .resolve::<QC_SUBJECT>();
    g::route(
        positive_then_remaining(),
        g::seq(retry.roll(), positive_then_remaining()),
    )
    .resolve::<QC_SUCCESS>()
}

fn business() -> impl Projectable {
    let initial = g::par(
        g::seq(
            g::send::<QC_SERVICE, CHEMIST, ChemAccess>(),
            g::seq(synthesis(), measurement::<31>()),
        ),
        g::seq(
            g::send::<BIO_FILE, BIOLOGIST, FileCells>(),
            g::seq(
                separation::<BIOLOGIST, 33>(),
                g::send::<BIOLOGIST, BIO_FILE, FileCellsResult>(),
            ),
        ),
    );
    let preparation = g::par(
        g::seq(
            g::send::<CHEM_FILE, CHEMISTRY, FilePurification>(),
            g::seq(
                separation::<CHEMISTRY, 32>(),
                g::seq(
                    g::send::<CHEMISTRY, CHEM_FILE, FilePurificationResult>(),
                    g::send::<CENT_SERVICE, QC_SERVICE, PurificationProof>(),
                ),
            ),
        ),
        g::seq(
            g::send::<BIO_FILE, BIOLOGIST, FileCalibration>(),
            g::seq(
                analysis::<BIOLOGIST, 34>(),
                g::seq(
                    g::send::<BIOLOGIST, BIO_FILE, FileCalibrationResult>(),
                    g::send::<SPEC_SERVICE, CENT_SERVICE, CalibrationProof>(),
                ),
            ),
        ),
    );
    let first = g::route(
        g::seq(
            select_both::<70, FileQcChemist>(),
            g::par(initial_biology(), qc_chemist()),
        ),
        g::seq(
            select_both::<71, FileQcBiologist>(),
            g::seq(initial_biology(), qc_biologist()),
        ),
    );
    let molecular = g::seq(
        g::send::<ANALYST_FILE, ANALYST, FileMolecular>(),
        g::seq(
            analysis::<ANALYST, 36>(),
            g::send::<ANALYST, ANALYST_FILE, FileMolecularResult>(),
        ),
    );
    let verification = g::seq(
        g::send::<ANALYST_FILE, ANALYST, FileVerification>(),
        g::seq(
            separation::<ANALYST, 37>(),
            g::send::<ANALYST, ANALYST_FILE, FileVerificationResult>(),
        ),
    );
    let final_validation = g::seq(
        g::send::<CHEM_FILE, CHEMIST, FileFinal>(),
        g::seq(
            analysis::<CHEMIST, 38>(),
            g::send::<CHEMIST, CHEM_FILE, FileFinalResult>(),
        ),
    );
    let finish = g::seq(
        g::send::<QC, BIOLOGIST, ApprovedResults>(),
        g::seq(
            g::send::<BIOLOGIST, ANALYST, ApprovedBiology>(),
            g::seq(
                g::send::<ANALYST, ANALYST_FILE, FileAssay>(),
                g::seq(
                    g::par(molecular, verification),
                    g::seq(
                        g::par(
                            g::send::<CENT_SERVICE, SPEC_SERVICE, VerificationProof>(),
                            g::par(
                                g::send::<ANALYST, CHEMIST, Quality>(),
                                g::send::<ANALYST, SPEC_SERVICE, QualityForOwner>(),
                            ),
                        ),
                        g::seq(
                            g::send::<CHEMIST, CHEM_FILE, FileQuality>(),
                            final_validation,
                        ),
                    ),
                ),
            ),
        ),
    );
    g::seq(
        initial,
        g::seq(
            preparation,
            g::seq(
                g::send::<CHEMIST, BIOLOGIST, Sample>(),
                g::seq(
                    g::send::<BIOLOGIST, BIO_FILE, FileSample>(),
                    g::seq(
                        g::send::<CHEMIST, QC, ChemAccess>(),
                        g::seq(
                            g::send::<CENT_SERVICE, BIOLOGIST, BioAccess>(),
                            g::seq(
                                first,
                                g::seq(
                                    submit_candidates(),
                                    g::seq(results_until_approved(), finish),
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
    let resources = g::par(
        g::send::<INIT, SPEC_SERVICE, SpecLoan>(),
        g::par(
            g::send::<INIT, CENT_SERVICE, CentLoan>(),
            g::send::<INIT, QC_SERVICE, FumeLoan>(),
        ),
    );
    let files = g::par(
        g::send::<CHEM_DATA, CHEMIST, Loan<CHEM_FILE>>(),
        g::par(
            g::send::<BIO_DATA, BIOLOGIST, Loan<BIO_FILE>>(),
            g::par(
                g::send::<ANALYST_DATA, ANALYST, Loan<ANALYST_FILE>>(),
                g::send::<QC_DATA, QC, Loan<QC_FILE>>(),
            ),
        ),
    );
    let guests = g::par(
        g::send::<INIT, CHEMIST, Loan<CHEM_VM>>(),
        g::par(
            g::send::<INIT, BIOLOGIST, Loan<BIO_VM>>(),
            g::par(
                g::send::<INIT, ANALYST, Loan<ANALYST_VM>>(),
                g::send::<INIT, QC, Loan<QC_VM>>(),
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
            g::par(
                g::seq(
                    crate::shell::global::file_metadata::<ANALYST_FILE, ANALYST_DATA>().roll(),
                    g::send::<ANALYST_FILE, ANALYST_DATA, FileExit>(),
                ),
                g::seq(
                    crate::shell::global::file_metadata::<QC_FILE, QC_DATA>().roll(),
                    g::send::<QC_FILE, QC_DATA, FileExit>(),
                ),
            ),
        ),
    );
    let io = g::par(
        g::seq(
            crate::shell::global::private_io::<CHEM_VM, CHEMIST_IO>(),
            g::send::<CHEM_VM, CHEMIST_IO, Exit>(),
        ),
        g::par(
            g::seq(
                crate::shell::global::private_io::<BIO_VM, BIOLOGIST_IO>(),
                g::send::<BIO_VM, BIOLOGIST_IO, Exit>(),
            ),
            g::par(
                g::seq(
                    crate::shell::global::private_io::<ANALYST_VM, ANALYST_IO>(),
                    g::send::<ANALYST_VM, ANALYST_IO, Exit>(),
                ),
                g::seq(
                    crate::shell::global::private_io::<QC_VM, QC_IO>(),
                    g::send::<QC_VM, QC_IO, Exit>(),
                ),
            ),
        ),
    );
    let returned = g::par(
        g::send::<SPEC_SERVICE, INIT, SpecLoan>(),
        g::par(
            g::send::<CENT_SERVICE, INIT, CentLoan>(),
            g::send::<QC_SERVICE, INIT, FumeLoan>(),
        ),
    );
    g::seq(
        g::par(resources, g::par(files, guests)),
        g::par(g::seq(business(), returned), g::par(metadata, io)),
    )
}
pub struct Programs {
    pub chemist: RoleProgram<CHEMIST>,
    pub biologist: RoleProgram<BIOLOGIST>,
    pub analyst: RoleProgram<ANALYST>,
    pub qc: RoleProgram<QC>,
    pub chemist_io: RoleProgram<CHEMIST_IO>,
    pub biologist_io: RoleProgram<BIOLOGIST_IO>,
    pub analyst_io: RoleProgram<ANALYST_IO>,
    pub qc_io: RoleProgram<QC_IO>,
    pub spec_service: RoleProgram<SPEC_SERVICE>,
    pub cent_service: RoleProgram<CENT_SERVICE>,
    pub qc_service: RoleProgram<QC_SERVICE>,
    pub spectrometer: RoleProgram<SPECTROMETER>,
    pub centrifuge: RoleProgram<CENTRIFUGE>,
    pub fume_hood: RoleProgram<FUME_HOOD>,
    pub chemistry: RoleProgram<CHEMISTRY>,
    pub biology: RoleProgram<BIOLOGY>,
    pub init: RoleProgram<INIT>,
    pub chem_file: RoleProgram<CHEM_FILE>,
    pub bio_file: RoleProgram<BIO_FILE>,
    pub analyst_file: RoleProgram<ANALYST_FILE>,
    pub qc_file: RoleProgram<QC_FILE>,
    pub chem_vm: RoleProgram<CHEM_VM>,
    pub bio_vm: RoleProgram<BIO_VM>,
    pub analyst_vm: RoleProgram<ANALYST_VM>,
    pub qc_vm: RoleProgram<QC_VM>,
    pub chem_data: RoleProgram<CHEM_DATA>,
    pub bio_data: RoleProgram<BIO_DATA>,
    pub analyst_data: RoleProgram<ANALYST_DATA>,
    pub qc_data: RoleProgram<QC_DATA>,
}
pub fn programs() -> Programs {
    let p = source();
    Programs {
        chemist: project(&p),
        biologist: project(&p),
        analyst: project(&p),
        qc: project(&p),
        chemist_io: project(&p),
        biologist_io: project(&p),
        analyst_io: project(&p),
        qc_io: project(&p),
        spec_service: project(&p),
        cent_service: project(&p),
        qc_service: project(&p),
        spectrometer: project(&p),
        centrifuge: project(&p),
        fume_hood: project(&p),
        chemistry: project(&p),
        biology: project(&p),
        init: project(&p),
        chem_file: project(&p),
        bio_file: project(&p),
        analyst_file: project(&p),
        qc_file: project(&p),
        chem_vm: project(&p),
        bio_vm: project(&p),
        analyst_vm: project(&p),
        qc_vm: project(&p),
        chem_data: project(&p),
        bio_data: project(&p),
        analyst_data: project(&p),
        qc_data: project(&p),
    }
}
impl Programs {
    pub fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        use hibana::g::Message;
        let labels: serde_json::Map<String, serde_json::Value> = [
            ("SpecLoan", SpecLoan::LOGICAL_LABEL),
            ("CentLoan", CentLoan::LOGICAL_LABEL),
            ("FumeLoan", FumeLoan::LOGICAL_LABEL),
            ("ChemAccess", ChemAccess::LOGICAL_LABEL),
            ("ApprovedResults", ApprovedResults::LOGICAL_LABEL),
            ("SynthesisRequest", SynthesisRequest::LOGICAL_LABEL),
            ("SpecRequest", SpecRequest::LOGICAL_LABEL),
            ("CentRequest", CentRequest::LOGICAL_LABEL),
            ("InstrumentResult", InstrumentResult::LOGICAL_LABEL),
            ("Synthesis", Synthesis::LOGICAL_LABEL),
            ("InitialAnalysis", InitialAnalysis::LOGICAL_LABEL),
            ("Purification", Purification::LOGICAL_LABEL),
            ("Preparation", Preparation::LOGICAL_LABEL),
            ("Calibration", Calibration::LOGICAL_LABEL),
            ("Reanalysis", Reanalysis::LOGICAL_LABEL),
            ("Molecular", Molecular::LOGICAL_LABEL),
            ("Separation", Separation::LOGICAL_LABEL),
            ("FinalValidation", FinalValidation::LOGICAL_LABEL),
            ("BioAccess", BioAccess::LOGICAL_LABEL),
            ("Sample", Sample::LOGICAL_LABEL),
            ("ChemistReport", ChemistReport::LOGICAL_LABEL),
            ("BiologyReport", BiologyReport::LOGICAL_LABEL),
            ("BioRequest", BioRequest::LOGICAL_LABEL),
            ("BioPassed", BioPassed::LOGICAL_LABEL),
            ("BioFailed", BioFailed::LOGICAL_LABEL),
            ("AnalysisProof", AnalysisProof::LOGICAL_LABEL),
            ("PurificationProof", PurificationProof::LOGICAL_LABEL),
            ("CalibrationProof", CalibrationProof::LOGICAL_LABEL),
            ("ApprovedBiology", ApprovedBiology::LOGICAL_LABEL),
            ("Quality", Quality::LOGICAL_LABEL),
            ("QualityForOwner", QualityForOwner::LOGICAL_LABEL),
            ("VerificationProof", VerificationProof::LOGICAL_LABEL),
            ("BioFumePassed", BioFumePassed::LOGICAL_LABEL),
            ("BioFumeFailed", BioFumeFailed::LOGICAL_LABEL),
            ("BioSpecPassed", BioSpecPassed::LOGICAL_LABEL),
            ("BioSpecFailed", BioSpecFailed::LOGICAL_LABEL),
            ("QcAdmission", QcAdmission::LOGICAL_LABEL),
            ("QcSpecRequest", QcSpecRequest::LOGICAL_LABEL),
            ("QcFumeRequest", QcFumeRequest::LOGICAL_LABEL),
            ("QcSpecResult", QcSpecResult::LOGICAL_LABEL),
            ("QcFumeResult", QcFumeResult::LOGICAL_LABEL),
            ("FirstChemist", FirstChemist::LOGICAL_LABEL),
            ("FirstBiologist", FirstBiologist::LOGICAL_LABEL),
            ("NextChemist", NextChemist::LOGICAL_LABEL),
            ("NextBiologist", NextBiologist::LOGICAL_LABEL),
            ("LastChemist", LastChemist::LOGICAL_LABEL),
            ("LastBiologist", LastBiologist::LOGICAL_LABEL),
            ("ChemistApproved", ChemistApproved::LOGICAL_LABEL),
            ("ChemistRejected", ChemistRejected::LOGICAL_LABEL),
            ("BiologistApproved", BiologistApproved::LOGICAL_LABEL),
            ("BiologistRejected", BiologistRejected::LOGICAL_LABEL),
            ("LastChemistApproved", LastChemistApproved::LOGICAL_LABEL),
            (
                "LastBiologistApproved",
                LastBiologistApproved::LOGICAL_LABEL,
            ),
            ("LastChemistRejected", LastChemistRejected::LOGICAL_LABEL),
            (
                "LastBiologistRejected",
                LastBiologistRejected::LOGICAL_LABEL,
            ),
            ("Retest", Retest::LOGICAL_LABEL),
            ("RetestPassed", RetestPassed::LOGICAL_LABEL),
            ("RetestFailed", RetestFailed::LOGICAL_LABEL),
            ("RetestFumePassed", RetestFumePassed::LOGICAL_LABEL),
            ("RetestFumeFailed", RetestFumeFailed::LOGICAL_LABEL),
            ("RetestSpecPassed", RetestSpecPassed::LOGICAL_LABEL),
            ("RetestSpecFailed", RetestSpecFailed::LOGICAL_LABEL),
            ("LastRetest", LastRetest::LOGICAL_LABEL),
            ("LastRetestPassed", LastRetestPassed::LOGICAL_LABEL),
            ("LastRetestFailed", LastRetestFailed::LOGICAL_LABEL),
            ("LastRetestFumePassed", LastRetestFumePassed::LOGICAL_LABEL),
            ("LastRetestFumeFailed", LastRetestFumeFailed::LOGICAL_LABEL),
            ("LastRetestSpecPassed", LastRetestSpecPassed::LOGICAL_LABEL),
            ("LastRetestSpecFailed", LastRetestSpecFailed::LOGICAL_LABEL),
            ("LastReanalysis", LastReanalysis::LOGICAL_LABEL),
            ("FileSynthesis", FileSynthesis::LOGICAL_LABEL),
            ("FileAnalysis", FileAnalysis::LOGICAL_LABEL),
            ("FilePurification", FilePurification::LOGICAL_LABEL),
            ("FileCells", FileCells::LOGICAL_LABEL),
            ("FileCalibration", FileCalibration::LOGICAL_LABEL),
            ("FileBioassay", FileBioassay::LOGICAL_LABEL),
            ("FileMolecular", FileMolecular::LOGICAL_LABEL),
            ("FileVerification", FileVerification::LOGICAL_LABEL),
            ("FileFinal", FileFinal::LOGICAL_LABEL),
            ("FileQcChemist", FileQcChemist::LOGICAL_LABEL),
            ("FileQcBiologist", FileQcBiologist::LOGICAL_LABEL),
            ("FileSynthesisResult", FileSynthesisResult::LOGICAL_LABEL),
            ("FileAnalysisResult", FileAnalysisResult::LOGICAL_LABEL),
            (
                "FilePurificationResult",
                FilePurificationResult::LOGICAL_LABEL,
            ),
            ("FileCellsResult", FileCellsResult::LOGICAL_LABEL),
            (
                "FileCalibrationResult",
                FileCalibrationResult::LOGICAL_LABEL,
            ),
            ("FileBioassayResult", FileBioassayResult::LOGICAL_LABEL),
            ("FileMolecularResult", FileMolecularResult::LOGICAL_LABEL),
            (
                "FileVerificationResult",
                FileVerificationResult::LOGICAL_LABEL,
            ),
            ("FileFinalResult", FileFinalResult::LOGICAL_LABEL),
            ("FileQcChemistResult", FileQcChemistResult::LOGICAL_LABEL),
            (
                "FileQcBiologistResult",
                FileQcBiologistResult::LOGICAL_LABEL,
            ),
            ("FileSample", FileSample::LOGICAL_LABEL),
            ("FileFeedback", FileFeedback::LOGICAL_LABEL),
            ("FileAssay", FileAssay::LOGICAL_LABEL),
            ("FileQuality", FileQuality::LOGICAL_LABEL),
        ]
        .into_iter()
        .map(|(n, l)| (n.into(), l.into()))
        .collect();
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"13M","session":SESSION,
             "roles":{"CHEMIST":CHEMIST,"BIOLOGIST":BIOLOGIST,"ANALYST":ANALYST,"QC_CONTROLLER":QC,"CHEMIST_IO":CHEMIST_IO,"BIOLOGIST_IO":BIOLOGIST_IO,"ANALYST_IO":ANALYST_IO,"QC_IO":QC_IO,"SPEC_SERVICE":SPEC_SERVICE,"CENT_SERVICE":CENT_SERVICE,"QC_SERVICE":QC_SERVICE,"SPECTROMETER":SPECTROMETER,"CENTRIFUGE":CENTRIFUGE,"FUME_HOOD":FUME_HOOD,"CHEMISTRY":CHEMISTRY,"BIOLOGY":BIOLOGY,"INIT":INIT,"CHEM_FILE":CHEM_FILE,"BIO_FILE":BIO_FILE,"ANALYST_FILE":ANALYST_FILE,"QC_FILE":QC_FILE,"CHEM_VM":CHEM_VM,"BIO_VM":BIO_VM,"ANALYST_VM":ANALYST_VM,"QC_VM":QC_VM,"CHEM_DATA":CHEM_DATA,"BIO_DATA":BIO_DATA,"ANALYST_DATA":ANALYST_DATA,"QC_DATA":QC_DATA},"labels":labels,"programs":[rows(&self.chemist),rows(&self.biologist),rows(&self.analyst),rows(&self.qc),rows(&self.chemist_io),rows(&self.biologist_io),rows(&self.analyst_io),rows(&self.qc_io),rows(&self.spec_service),rows(&self.cent_service),rows(&self.qc_service),rows(&self.spectrometer),rows(&self.centrifuge),rows(&self.fume_hood),rows(&self.chemistry),rows(&self.biology),rows(&self.init),rows(&self.chem_file),rows(&self.bio_file),rows(&self.analyst_file),rows(&self.qc_file),rows(&self.chem_vm),rows(&self.bio_vm),rows(&self.analyst_vm),rows(&self.qc_vm),rows(&self.chem_data),rows(&self.bio_data),rows(&self.analyst_data),rows(&self.qc_data)]})
    }
}
