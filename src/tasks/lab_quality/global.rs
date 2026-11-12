//! Actual scientific candidates, instruments and result-driven QC.
use super::imp::QcReply;
use crate::shell::imp::review_reply::ReviewReply;
use hibana::{
    Endpoint, g,
    runtime::program::{Projectable, RoleProgram, project},
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
pub const FUME_SERVICE: u8 = 10;
pub const SPECTROMETER: u8 = 11;
pub const CENTRIFUGE: u8 = 12;
pub const FUME_HOOD: u8 = 13;
// The scientist uses this view to create measurements. Submission moves the
// same continuation to QC, so the scientist cannot edit the pending candidate.
pub const CHEMISTRY: u8 = 14;
pub const BIOLOGY: u8 = 15;
pub const SESSION: u32 = 131;
pub const SELECT_QC: u16 = 1310;
pub const QC_SUCCESS: u16 = 1311;
pub const QC_SUBJECT: u16 = 1312;
pub const BIO_SUCCESS: u16 = 1313;

pub type SpecLoan<'r> = g::Msg<1, Endpoint<'r, SPECTROMETER>>;
pub type CentLoan<'r> = g::Msg<2, Endpoint<'r, CENTRIFUGE>>;
pub type FumeLoan<'r> = g::Msg<3, Endpoint<'r, FUME_HOOD>>;
pub type ChemAccess<'r> = g::Msg<4, Endpoint<'r, CHEMISTRY>>;
pub type ApprovedResults<'a> = g::Msg<5, &'a [u8]>;
pub type SynthesisRequest<'a> = g::Msg<6, &'a [u8]>;
pub type SpecRequest<'a> = g::Msg<7, &'a [u8]>;
pub type CentRequest<'a> = g::Msg<8, &'a [u8]>;
pub type InstrumentResult<'a> = g::Msg<9, &'a [u8]>;
pub type Synthesis<'a> = g::Msg<10, &'a [u8]>;
pub type InitialAnalysis<'a> = g::Msg<11, &'a [u8]>;
pub type Purification<'a> = g::Msg<12, &'a [u8]>;
pub type Preparation<'a> = g::Msg<13, &'a [u8]>;
pub type Calibration<'a> = g::Msg<14, &'a [u8]>;
pub type Reanalysis<'a> = g::Msg<15, &'a [u8]>;
pub type Molecular<'a> = g::Msg<16, &'a [u8]>;
pub type Separation<'a> = g::Msg<17, &'a [u8]>;
pub type FinalValidation<'a> = g::Msg<18, &'a [u8]>;
pub type BioAccess<'r> = g::Msg<19, Endpoint<'r, BIOLOGY>>;
pub type Sample<'a> = g::Msg<20, &'a [u8]>;
pub type ChemistReport<'a> = g::Msg<22, &'a [u8]>;
pub type BiologyReport<'a> = g::Msg<23, &'a [u8]>;
pub type BioRequest<'a> = g::Msg<24, &'a [u8]>;
pub type BioPassed<'a> = g::Msg<25, ReviewReply<'a, true>>;
pub type BioFailed<'a> = g::Msg<26, ReviewReply<'a, false>>;
pub type AnalysisProof<'a> = g::Msg<27, &'a [u8]>;
pub type PurificationProof<'a> = g::Msg<28, &'a [u8]>;
pub type CalibrationProof<'a> = g::Msg<29, &'a [u8]>;
pub type ApprovedBiology<'a> = g::Msg<30, &'a [u8]>;
pub type Quality<'a> = g::Msg<31, &'a [u8]>;
pub type QualityForOwner<'a> = g::Msg<32, &'a [u8]>;
pub type VerificationProof<'a> = g::Msg<33, &'a [u8]>;
pub type BioFumePassed<'a> = g::Msg<35, ReviewReply<'a, true>>;
pub type BioFumeFailed<'a> = g::Msg<36, ReviewReply<'a, false>>;
pub type BioSpecPassed<'a> = g::Msg<37, ReviewReply<'a, true>>;
pub type BioSpecFailed<'a> = g::Msg<38, ReviewReply<'a, false>>;
pub type QcAdmission<'a> = g::Msg<40, &'a [u8]>;
pub type QcSpecRequest<'a> = g::Msg<41, &'a [u8]>;
pub type QcFumeRequest<'a> = g::Msg<42, &'a [u8]>;
pub type QcSpecResult<'a> = g::Msg<43, &'a [u8]>;
pub type QcFumeResult<'a> = g::Msg<44, &'a [u8]>;
pub type FirstChemist<'a> = g::Msg<50, &'a [u8]>;
pub type FirstBiologist<'a> = g::Msg<51, &'a [u8]>;
pub type NextChemist<'a> = g::Msg<52, &'a [u8]>;
pub type NextBiologist<'a> = g::Msg<53, &'a [u8]>;
pub type LastChemist<'a> = g::Msg<54, &'a [u8]>;
pub type LastBiologist<'a> = g::Msg<55, &'a [u8]>;
pub type ChemistApproved<'a> = g::Msg<80, QcReply<'a, CHEMIST, true>>;
pub type ChemistRejected<'a> = g::Msg<81, QcReply<'a, CHEMIST, false>>;
pub type BiologistApproved<'a> = g::Msg<82, QcReply<'a, BIOLOGIST, true>>;
pub type BiologistRejected<'a> = g::Msg<83, QcReply<'a, BIOLOGIST, false>>;
pub type LastChemistApproved<'a> = g::Msg<84, QcReply<'a, CHEMIST, true>>;
pub type LastBiologistApproved<'a> = g::Msg<85, QcReply<'a, BIOLOGIST, true>>;
pub type LastChemistRejected<'a> = g::Msg<86, QcReply<'a, CHEMIST, false>>;
pub type LastBiologistRejected<'a> = g::Msg<87, QcReply<'a, BIOLOGIST, false>>;
pub type Retest<'a> = g::Msg<100, &'a [u8]>;
pub type RetestPassed<'a> = g::Msg<101, ReviewReply<'a, true>>;
pub type RetestFailed<'a> = g::Msg<102, ReviewReply<'a, false>>;
pub type RetestFumePassed<'a> = g::Msg<103, ReviewReply<'a, true>>;
pub type RetestFumeFailed<'a> = g::Msg<104, ReviewReply<'a, false>>;
pub type RetestSpecPassed<'a> = g::Msg<105, ReviewReply<'a, true>>;
pub type RetestSpecFailed<'a> = g::Msg<106, ReviewReply<'a, false>>;
pub type LastRetest<'a> = g::Msg<110, &'a [u8]>;
pub type LastRetestPassed<'a> = g::Msg<111, ReviewReply<'a, true>>;
pub type LastRetestFailed<'a> = g::Msg<112, ReviewReply<'a, false>>;
pub type LastRetestFumePassed<'a> = g::Msg<113, ReviewReply<'a, true>>;
pub type LastRetestFumeFailed<'a> = g::Msg<114, ReviewReply<'a, false>>;
pub type LastRetestSpecPassed<'a> = g::Msg<115, ReviewReply<'a, true>>;
pub type LastRetestSpecFailed<'a> = g::Msg<116, ReviewReply<'a, false>>;
pub type LastReanalysis<'a> = g::Msg<117, &'a [u8]>;

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
        analysis::<CHEMISTRY, LABEL>(),
        g::send::<SPEC_SERVICE, FUME_SERVICE, AnalysisProof>(),
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
        g::send::<CHEMISTRY, FUME_SERVICE, Synthesis>(),
        g::seq(
            g::send::<FUME_SERVICE, CHEMISTRY, FumeLoan>(),
            g::seq(
                g::send::<FUME_HOOD, FUME_SERVICE, SynthesisRequest>(),
                g::seq(
                    g::send::<FUME_SERVICE, FUME_HOOD, InstrumentResult>(),
                    g::send::<CHEMISTRY, FUME_SERVICE, FumeLoan>(),
                ),
            ),
        ),
    )
}
fn bio_reply<M: g::Message, F: g::Message, S: g::Message>() -> impl Projectable {
    g::seq(
        g::send::<CENT_SERVICE, BIOLOGIST, M>(),
        g::par(
            g::send::<CENT_SERVICE, FUME_SERVICE, F>(),
            g::send::<CENT_SERVICE, SPEC_SERVICE, S>(),
        ),
    )
}
// Distinct repeatable occurrences have distinct public operation selectors.
fn bio_until_success<Q: Projectable, P: Projectable, N: Projectable>(
    request: impl Fn() -> Q,
    passed: impl Fn() -> P,
    failed: impl Fn() -> N,
) -> impl Projectable {
    // Each actual trial submits the specimen continuation to the service.
    // A failed result returns it for a new trial; success sends it to QC.
    // Thus the service cannot restart an old trial after giving up custody.
    let first = g::seq(request(), g::send::<BIOLOGIST, CENT_SERVICE, BioAccess>());
    let retry = g::seq(
        failed(),
        g::seq(
            g::send::<CENT_SERVICE, BIOLOGIST, BioAccess>(),
            g::seq(request(), g::send::<BIOLOGIST, CENT_SERVICE, BioAccess>()),
        ),
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
        g::send::<BIOLOGY, CENT_SERVICE, BioRequest>,
        bio_reply::<BioPassed, BioFumePassed, BioSpecPassed>,
        bio_reply::<BioFailed, BioFumeFailed, BioSpecFailed>,
    )
}
fn repeated_biology() -> impl Projectable {
    bio_until_success(
        g::send::<BIOLOGY, CENT_SERVICE, Retest>,
        bio_reply::<RetestPassed, RetestFumePassed, RetestSpecPassed>,
        bio_reply::<RetestFailed, RetestFumeFailed, RetestSpecFailed>,
    )
}
fn last_biology() -> impl Projectable {
    bio_until_success(
        g::send::<BIOLOGY, CENT_SERVICE, LastRetest>,
        bio_reply::<LastRetestPassed, LastRetestFumePassed, LastRetestSpecPassed>,
        bio_reply::<LastRetestFailed, LastRetestFumeFailed, LastRetestSpecFailed>,
    )
}
fn select<const WHO: u8, const LABEL: u8>() -> impl Projectable {
    g::seq(
        g::send::<QC, SPEC_SERVICE, g::Msg<LABEL, &[u8]>>(),
        g::seq(
            g::send::<QC, FUME_SERVICE, g::Msg<LABEL, &[u8]>>(),
            g::send::<QC, WHO, g::Msg<LABEL, &[u8]>>(),
        ),
    )
}
fn select_both<const LABEL: u8>() -> impl Projectable {
    g::seq(
        g::send::<QC, SPEC_SERVICE, g::Msg<LABEL, &[u8]>>(),
        g::seq(
            g::send::<QC, FUME_SERVICE, g::Msg<LABEL, &[u8]>>(),
            g::seq(
                g::send::<QC, CHEMIST, g::Msg<LABEL, &[u8]>>(),
                g::send::<QC, BIOLOGIST, g::Msg<LABEL, &[u8]>>(),
            ),
        ),
    )
}
fn qc_resources() -> impl Projectable {
    let requests = g::seq(
        g::par(
            g::send::<SPEC_SERVICE, QC, SpecLoan>(),
            g::send::<FUME_SERVICE, QC, FumeLoan>(),
        ),
        g::par(
            g::send::<SPECTROMETER, FUME_SERVICE, QcSpecRequest>(),
            g::send::<FUME_HOOD, FUME_SERVICE, QcFumeRequest>(),
        ),
    );
    g::seq(
        requests,
        g::seq(
            g::par(
                g::send::<FUME_SERVICE, SPECTROMETER, QcSpecResult>(),
                g::send::<FUME_SERVICE, FUME_HOOD, QcFumeResult>(),
            ),
            g::par(
                g::send::<QC, SPEC_SERVICE, SpecLoan>(),
                g::send::<QC, FUME_SERVICE, FumeLoan>(),
            ),
        ),
    )
}
fn qc_chemist() -> impl Projectable {
    g::seq(
        g::send::<CHEMIST, QC, ChemistReport>(),
        g::seq(
            g::send::<CHEMISTRY, FUME_SERVICE, QcAdmission>(),
            qc_resources(),
        ),
    )
}
fn qc_biologist() -> impl Projectable {
    g::seq(
        g::send::<BIOLOGIST, QC, BiologyReport>(),
        g::seq(
            g::send::<BIOLOGY, FUME_SERVICE, QcAdmission>(),
            qc_resources(),
        ),
    )
}
// Both scientists have submitted immutable candidates. The receiver of the
// actual verification effect owns those continuations while deciding its result.
fn submit_candidates() -> impl Projectable {
    g::par(
        g::send::<QC, FUME_SERVICE, ChemAccess>(),
        g::send::<QC, FUME_SERVICE, BioAccess>(),
    )
}
fn return_candidates() -> impl Projectable {
    g::par(
        g::send::<FUME_SERVICE, QC, ChemAccess>(),
        g::send::<FUME_SERVICE, QC, BioAccess>(),
    )
}
// Share the actual subject-bound verdict, not recipient-specific control tokens.
// Sends queue the result; they do not wait for recipients to finish their work.
fn announce<M: g::Message>(held: impl Projectable) -> impl Projectable {
    g::seq(
        g::send::<FUME_SERVICE, QC, M>(),
        g::seq(
            held,
            g::seq(
                g::send::<QC, CHEMIST, M>(),
                g::seq(
                    g::send::<QC, BIOLOGIST, M>(),
                    g::seq(
                        g::send::<QC, SPEC_SERVICE, M>(),
                        g::send::<QC, CENT_SERVICE, M>(),
                    ),
                ),
            ),
        ),
    )
}
fn reject_chemist() -> impl Projectable {
    g::seq(
        announce::<ChemistRejected>(return_candidates()),
        g::send::<QC, CHEMIST, ChemAccess>(),
    )
}
fn reject_biologist() -> impl Projectable {
    g::seq(
        announce::<BiologistRejected>(return_candidates()),
        g::send::<QC, BIOLOGIST, BioAccess>(),
    )
}
fn last_chemist() -> impl Projectable {
    let verify = || {
        g::seq(
            g::seq(select::<CHEMIST, 54>(), qc_chemist()),
            g::send::<QC, FUME_SERVICE, ChemAccess>(),
        )
    };
    let approved = || announce::<LastChemistApproved>(g::send::<FUME_SERVICE, QC, ChemAccess>());
    let rejected = g::seq(
        announce::<LastChemistRejected>(g::send::<FUME_SERVICE, QC, ChemAccess>()),
        g::send::<QC, CHEMIST, ChemAccess>(),
    );
    g::seq(
        verify(),
        g::route(
            approved(),
            g::seq(
                g::seq(rejected, g::seq(remeasure::<117>(), verify())).roll(),
                approved(),
            ),
        )
        .resolve::<QC_SUCCESS>(),
    )
}
fn last_biologist() -> impl Projectable {
    let verify = || {
        g::seq(
            g::seq(select::<BIOLOGIST, 55>(), qc_biologist()),
            g::send::<QC, FUME_SERVICE, BioAccess>(),
        )
    };
    let approved = || announce::<LastBiologistApproved>(g::send::<FUME_SERVICE, QC, BioAccess>());
    let rejected = g::seq(
        announce::<LastBiologistRejected>(g::send::<FUME_SERVICE, QC, BioAccess>()),
        g::send::<QC, BIOLOGIST, BioAccess>(),
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
            announce::<ChemistApproved>(return_candidates()),
            last_biologist(),
        ),
        g::seq(
            announce::<BiologistApproved>(return_candidates()),
            last_chemist(),
        ),
    )
    .resolve::<QC_SUBJECT>()
}
fn next_qc() -> impl Projectable {
    g::seq(
        g::route(
            g::seq(select_both::<52>(), qc_chemist()),
            g::seq(select_both::<53>(), qc_biologist()),
        )
        .resolve::<SELECT_QC>(),
        submit_candidates(),
    )
}
fn next_qc_during_biology() -> impl Projectable {
    g::seq(
        g::route(
            g::seq(
                select_both::<52>(),
                g::par(repeated_biology(), qc_chemist()),
            ),
            g::seq(
                select_both::<53>(),
                g::seq(repeated_biology(), qc_biologist()),
            ),
        )
        .resolve::<SELECT_QC>(),
        submit_candidates(),
    )
}
fn results_until_approved() -> impl Projectable {
    let retry = g::route(
        g::seq(reject_chemist(), g::seq(remeasure::<15>(), next_qc())),
        g::seq(reject_biologist(), next_qc_during_biology()),
    )
    .resolve::<QC_SUBJECT>();
    g::route(
        positive_then_remaining(),
        g::seq(retry.roll(), positive_then_remaining()),
    )
    .resolve::<QC_SUCCESS>()
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
    pub fume_service: RoleProgram<FUME_SERVICE>,
    pub spectrometer: RoleProgram<SPECTROMETER>,
    pub centrifuge: RoleProgram<CENTRIFUGE>,
    pub fume_hood: RoleProgram<FUME_HOOD>,
    pub chemistry: RoleProgram<CHEMISTRY>,
    pub biology: RoleProgram<BIOLOGY>,
}
impl Programs {
    pub fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        use hibana::g::Message;
        let labels: serde_json::Map<String, serde_json::Value> = [
            ("Synthesis", Synthesis::LOGICAL_LABEL),
            ("InitialAnalysis", InitialAnalysis::LOGICAL_LABEL),
            ("Purification", Purification::LOGICAL_LABEL),
            ("Preparation", Preparation::LOGICAL_LABEL),
            ("Calibration", Calibration::LOGICAL_LABEL),
            ("Reanalysis", Reanalysis::LOGICAL_LABEL),
            ("LastReanalysis", LastReanalysis::LOGICAL_LABEL),
            ("Molecular", Molecular::LOGICAL_LABEL),
            ("Separation", Separation::LOGICAL_LABEL),
            ("FinalValidation", FinalValidation::LOGICAL_LABEL),
            ("SynthesisRequest", SynthesisRequest::LOGICAL_LABEL),
            ("SpecRequest", SpecRequest::LOGICAL_LABEL),
            ("CentRequest", CentRequest::LOGICAL_LABEL),
            ("InstrumentResult", InstrumentResult::LOGICAL_LABEL),
            ("Sample", Sample::LOGICAL_LABEL),
            ("BioRequest", BioRequest::LOGICAL_LABEL),
            ("BioPassed", BioPassed::LOGICAL_LABEL),
            ("BioFailed", BioFailed::LOGICAL_LABEL),
            ("Retest", Retest::LOGICAL_LABEL),
            ("RetestPassed", RetestPassed::LOGICAL_LABEL),
            ("RetestFailed", RetestFailed::LOGICAL_LABEL),
            ("LastRetest", LastRetest::LOGICAL_LABEL),
            ("LastRetestPassed", LastRetestPassed::LOGICAL_LABEL),
            ("LastRetestFailed", LastRetestFailed::LOGICAL_LABEL),
            ("AnalysisProof", AnalysisProof::LOGICAL_LABEL),
            ("PurificationProof", PurificationProof::LOGICAL_LABEL),
            ("CalibrationProof", CalibrationProof::LOGICAL_LABEL),
            ("ApprovedBiology", ApprovedBiology::LOGICAL_LABEL),
            ("ApprovedResults", ApprovedResults::LOGICAL_LABEL),
            ("Quality", Quality::LOGICAL_LABEL),
            ("QualityForOwner", QualityForOwner::LOGICAL_LABEL),
            ("VerificationProof", VerificationProof::LOGICAL_LABEL),
            ("ChemistReport", ChemistReport::LOGICAL_LABEL),
            ("BiologyReport", BiologyReport::LOGICAL_LABEL),
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
            ("LastChemistRejected", LastChemistRejected::LOGICAL_LABEL),
            (
                "LastBiologistApproved",
                LastBiologistApproved::LOGICAL_LABEL,
            ),
            (
                "LastBiologistRejected",
                LastBiologistRejected::LOGICAL_LABEL,
            ),
            ("BioFumePassed", BioFumePassed::LOGICAL_LABEL),
            ("BioFumeFailed", BioFumeFailed::LOGICAL_LABEL),
            ("BioSpecPassed", BioSpecPassed::LOGICAL_LABEL),
            ("BioSpecFailed", BioSpecFailed::LOGICAL_LABEL),
            ("RetestFumePassed", RetestFumePassed::LOGICAL_LABEL),
            ("RetestFumeFailed", RetestFumeFailed::LOGICAL_LABEL),
            ("RetestSpecPassed", RetestSpecPassed::LOGICAL_LABEL),
            ("RetestSpecFailed", RetestSpecFailed::LOGICAL_LABEL),
            ("LastRetestFumePassed", LastRetestFumePassed::LOGICAL_LABEL),
            ("LastRetestFumeFailed", LastRetestFumeFailed::LOGICAL_LABEL),
            ("LastRetestSpecPassed", LastRetestSpecPassed::LOGICAL_LABEL),
            ("LastRetestSpecFailed", LastRetestSpecFailed::LOGICAL_LABEL),
        ]
        .into_iter()
        .map(|(name, label)| (name.into(), label.into()))
        .collect();
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"13M","session":SESSION,
            "roles":{"CHEMIST":CHEMIST,"BIOLOGIST":BIOLOGIST,"ANALYST":ANALYST,"QC_CONTROLLER":QC,
              "CHEMIST_IO":CHEMIST_IO,"BIOLOGIST_IO":BIOLOGIST_IO,"ANALYST_IO":ANALYST_IO,"QC_IO":QC_IO,
              "SPEC_SERVICE":SPEC_SERVICE,"CENT_SERVICE":CENT_SERVICE,"FUME_SERVICE":FUME_SERVICE,
              "SPECTROMETER":SPECTROMETER,"CENTRIFUGE":CENTRIFUGE,"FUME_HOOD":FUME_HOOD,
              "CHEMISTRY":CHEMISTRY,"BIOLOGY":BIOLOGY},
            "labels":labels,
            "programs":[rows(&self.chemist),rows(&self.biologist),rows(&self.analyst),rows(&self.qc),
              rows(&self.chemist_io),rows(&self.biologist_io),rows(&self.analyst_io),rows(&self.qc_io),
              rows(&self.spec_service),rows(&self.cent_service),rows(&self.fume_service),
              rows(&self.spectrometer),rows(&self.centrifuge),rows(&self.fume_hood),rows(&self.chemistry),rows(&self.biology)]})
    }
}
pub fn programs() -> Programs {
    let initial = g::par(
        g::seq(
            g::send::<FUME_SERVICE, CHEMIST, ChemAccess>(),
            g::seq(synthesis(), measurement::<11>()),
        ),
        separation::<BIOLOGIST, 13>(),
    );
    let preparation = g::par(
        g::seq(
            separation::<CHEMISTRY, 12>(),
            g::send::<CENT_SERVICE, FUME_SERVICE, PurificationProof>(),
        ),
        g::seq(
            analysis::<BIOLOGIST, 14>(),
            g::send::<SPEC_SERVICE, CENT_SERVICE, CalibrationProof>(),
        ),
    );
    let first = g::route(
        g::seq(select_both::<50>(), g::par(initial_biology(), qc_chemist())),
        g::seq(
            select_both::<51>(),
            g::seq(initial_biology(), qc_biologist()),
        ),
    )
    .resolve::<SELECT_QC>();
    let finish = g::seq(
        g::send::<QC, BIOLOGIST, ApprovedResults>(),
        g::seq(
            g::send::<BIOLOGIST, ANALYST, ApprovedBiology>(),
            g::seq(
                analysis::<ANALYST, 16>(),
                g::seq(
                    separation::<ANALYST, 17>(),
                    g::seq(
                        g::par(
                            g::send::<CENT_SERVICE, SPEC_SERVICE, VerificationProof>(),
                            g::par(
                                g::send::<ANALYST, CHEMIST, Quality>(),
                                g::send::<ANALYST, SPEC_SERVICE, QualityForOwner>(),
                            ),
                        ),
                        analysis::<CHEMIST, 18>(),
                    ),
                ),
            ),
        ),
    );
    let business = g::seq(
        initial,
        g::seq(
            preparation,
            g::seq(
                g::send::<CHEMIST, BIOLOGIST, Sample>(),
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
    );
    let source = g::par(
        business,
        g::par(
            crate::shell::global::private_io::<CHEMIST, CHEMIST_IO>(),
            g::par(
                crate::shell::global::private_io::<BIOLOGIST, BIOLOGIST_IO>(),
                g::par(
                    crate::shell::global::private_io::<ANALYST, ANALYST_IO>(),
                    crate::shell::global::private_io::<QC, QC_IO>(),
                ),
            ),
        ),
    );
    Programs {
        chemist: project(&source),
        biologist: project(&source),
        analyst: project(&source),
        qc: project(&source),
        chemist_io: project(&source),
        biologist_io: project(&source),
        analyst_io: project(&source),
        qc_io: project(&source),
        spec_service: project(&source),
        cent_service: project(&source),
        fume_service: project(&source),
        spectrometer: project(&source),
        centrifuge: project(&source),
        fume_hood: project(&source),
        chemistry: project(&source),
        biology: project(&source),
    }
}
