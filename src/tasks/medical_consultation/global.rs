//! Actual operation files, independent clinical work and affine patient-result custody.
use super::imp::{Arguments, Submission};
pub use crate::shell::global::{FileExit, Loan};
use hibana::{
    Endpoint, g,
    runtime::program::{Projectable, RoleProgram, project},
};
use hibana_wasip1_runtime::{
    choreofs::{ChoreoFsObject, FdSpec, ObjectId},
    protocol,
};
pub const ER_DOCTOR: u8 = 0;
pub const CARDIOLOGIST: u8 = 1;
pub const PHARMACIST: u8 = 2;
pub const ER_IO: u8 = 3;
pub const CARDIO_IO: u8 = 4;
pub const PHARMACY_IO: u8 = 5;
pub const GENERAL_SERVICE: u8 = 6;
pub const CARDIAC_SERVICE: u8 = 7;
pub const PHARMACY_SERVICE: u8 = 8;
pub const IMAGING_SERVICE: u8 = 9;
pub const CT_SCANNER: u8 = 10;
pub const GENERAL_PANEL: u8 = 11;
pub const CARDIAC_PANEL: u8 = 12;
pub const RECORDS: u8 = 13;
pub const GENERAL_COMPLETE: u16 = 501;
pub const CARDIAC_COMPLETE: u16 = 502;
pub const SESSION: u32 = 50;
pub const ER_VM: u8 = 32;
pub const CARDIO_VM: u8 = 33;
pub const PHARMACY_VM: u8 = 34;
pub const ASSESS_FILE: u8 = 16;
pub const ASSESS_DATA: u8 = 48;
pub const GENERAL_FILE: u8 = 17;
pub const GENERAL_DATA: u8 = 49;
pub const IMAGING_FILE: u8 = 18;
pub const IMAGING_DATA: u8 = 50;
pub const REFERRAL_FILE: u8 = 19;
pub const REFERRAL_DATA: u8 = 51;
pub const CARDIAC_FILE: u8 = 20;
pub const CARDIAC_DATA: u8 = 52;
pub const FINDINGS_FILE: u8 = 21;
pub const FINDINGS_DATA: u8 = 53;
pub const DIAGNOSIS_FILE: u8 = 22;
pub const DIAGNOSIS_DATA: u8 = 54;
pub const REVIEW_FILE: u8 = 23;
pub const REVIEW_DATA: u8 = 55;
pub const SUMMARY_FILE: u8 = 24;
pub const SUMMARY_DATA: u8 = 56;
pub type AssessmentRequest = g::Msg<1, Arguments<0>>;
pub type AssessmentResult<'a> = g::Msg<2, &'a [u8]>;
pub type GeneralTests = g::Msg<3, Arguments<1>>;
pub type GeneralTestResults<'a> = g::Msg<4, &'a [u8]>;
pub type CardiacTests = g::Msg<5, Arguments<3>>;
pub type CardiacTestResults<'a> = g::Msg<6, &'a [u8]>;
pub type ImagingIntent = g::Msg<7, Submission<2>>;
pub type CtLoan<'r> = g::Msg<8, Endpoint<'r, CT_SCANNER>>;
pub type ImagingRequest = g::Msg<9, Arguments<2>>;
pub type ImagingResult<'a> = g::Msg<10, &'a [u8]>;
pub type Referral<'a> = g::Msg<11, &'a [u8]>;
pub type ReferralForOwner<'a> = g::Msg<12, &'a [u8]>;
pub type GeneralFindings<'a> = g::Msg<13, &'a [u8]>;
pub type FindingsForOwner<'a> = g::Msg<14, &'a [u8]>;
pub type DiagnosisRequest = g::Msg<15, Arguments<4>>;
pub type DiagnosisResult<'a> = g::Msg<16, &'a [u8]>;
pub type CardiacFindings<'a> = g::Msg<17, &'a [u8]>;
pub type ReviewRequest = g::Msg<18, Arguments<5>>;
pub type ReviewResult<'a> = g::Msg<19, &'a [u8]>;
pub type MedicationReview<'a> = g::Msg<20, &'a [u8]>;
pub type GeneralPanelLoan<'r> = g::Msg<21, Endpoint<'r, GENERAL_PANEL>>;
pub type GeneralTestsPartial<'a> = g::Msg<22, &'a [u8]>;
pub type GeneralPanelReturn<'r> = g::Msg<23, Endpoint<'r, GENERAL_PANEL>>;
pub type GeneralPanelCompleted<'r> = g::Msg<25, Endpoint<'r, GENERAL_PANEL>>;
pub type GeneralTestsComplete<'a> = g::Msg<26, &'a [u8]>;
pub type CardiacPanelLoan<'r> = g::Msg<28, Endpoint<'r, CARDIAC_PANEL>>;
pub type CardiacPanelReturn<'r> = g::Msg<29, Endpoint<'r, CARDIAC_PANEL>>;
pub type CardiacTestsPartial<'a> = g::Msg<30, &'a [u8]>;
pub type CardiacPanelCompleted<'r> = g::Msg<31, Endpoint<'r, CARDIAC_PANEL>>;
pub type CardiacTestsComplete<'a> = g::Msg<32, &'a [u8]>;
pub type FileAssessmentResult<'a> = g::Msg<33, &'a [u8]>;
pub type GeneralFileRequest = g::Msg<34, Submission<1>>;
pub type GeneralAdmitted = g::Msg<35, Submission<1>>;
pub type FileGeneralResult<'a> = g::Msg<36, &'a [u8]>;
pub type ImagingAdmitted = g::Msg<37, Submission<2>>;
pub type FileImagingResult<'a> = g::Msg<38, &'a [u8]>;
pub type FileReferral<'a> = g::Msg<39, &'a [u8]>;
pub type FileFindings<'a> = g::Msg<40, &'a [u8]>;
pub type FileDiagnosisResult<'a> = g::Msg<41, &'a [u8]>;
pub type FileReviewResult<'a> = g::Msg<42, &'a [u8]>;
pub type Summary<'a> = g::Msg<43, &'a [u8]>;
pub type CardiacFileRequest = g::Msg<44, Submission<3>>;
pub type CardiacAdmitted = g::Msg<45, Submission<3>>;
pub type FileCardiacResult<'a> = g::Msg<46, &'a [u8]>;
pub type GeneralCompletedData<'a> = g::Msg<48, &'a [u8]>;
pub type CardiacCompletedData<'a> = g::Msg<49, &'a [u8]>;
pub type CompletedConsultation<'a> = g::Msg<50, &'a [u8]>;
pub type CardiacPanelReport<'a> = g::Msg<51, &'a [u8]>;

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
const fn result(path: &'static [u8], fd: u8) -> ChoreoFsObject<'static> {
    ChoreoFsObject::readable(
        path,
        ObjectId(fd as u32),
        FdSpec::new(fd, 2),
        b"",
        protocol::FdBinding {
            read: Some(protocol::FdReadRow::Base),
            write: None,
            readdir: None,
        },
    )
}
pub const ER_FILES: [ChoreoFsObject<'static>; 4] = [
    operation(b"service/assess_patient", 4),
    operation(b"service/order_tests", 5),
    operation(b"service/perform_imaging", 6),
    result(b"service/summary", 7),
];
pub const CARDIO_FILES: [ChoreoFsObject<'static>; 4] = [
    result(b"service/referral", 4),
    operation(b"service/order_tests", 5),
    result(b"service/findings", 6),
    operation(b"service/diagnose", 7),
];
pub const PHARMACY_FILES: [ChoreoFsObject<'static>; 1] =
    [operation(b"service/review_prescriptions", 4)];

pub fn source() -> impl Projectable {
    let assessment = g::seq(
        g::send::<ASSESS_FILE, GENERAL_SERVICE, AssessmentRequest>(),
        g::par(
            g::send::<GENERAL_SERVICE, ER_DOCTOR, AssessmentResult>(),
            g::send::<GENERAL_SERVICE, ASSESS_FILE, FileAssessmentResult>(),
        ),
    );
    let general_attempt = || {
        g::seq(
            g::send::<GENERAL_SERVICE, ER_DOCTOR, GeneralPanelLoan>(),
            g::seq(
                g::send::<GENERAL_FILE, GENERAL_SERVICE, GeneralFileRequest>(),
                g::seq(
                    g::send::<GENERAL_SERVICE, ER_DOCTOR, GeneralAdmitted>(),
                    g::seq(
                        g::send::<GENERAL_PANEL, GENERAL_SERVICE, GeneralTests>(),
                        g::seq(
                            g::send::<GENERAL_SERVICE, GENERAL_PANEL, GeneralTestResults>(),
                            g::send::<ER_DOCTOR, GENERAL_SERVICE, GeneralPanelReturn>(),
                        ),
                    ),
                ),
            ),
        )
    };
    let general_ready = || {
        g::seq(
            g::send::<GENERAL_SERVICE, RECORDS, GeneralCompletedData>(),
            g::send::<GENERAL_SERVICE, ER_DOCTOR, GeneralTestsComplete>(),
        )
    };
    let general_tests = g::seq(
        general_attempt(),
        g::seq(
            g::route(
                general_ready(),
                g::seq(
                    g::seq(
                        g::send::<GENERAL_SERVICE, GENERAL_FILE, FileGeneralResult>(),
                        g::seq(
                            g::send::<GENERAL_SERVICE, ER_DOCTOR, GeneralTestsPartial>(),
                            general_attempt(),
                        ),
                    )
                    .roll(),
                    general_ready(),
                ),
            )
            .resolve::<GENERAL_COMPLETE>(),
            g::seq(
                g::send::<GENERAL_SERVICE, RECORDS, GeneralPanelCompleted>(),
                g::send::<RECORDS, GENERAL_FILE, FileGeneralResult>(),
            ),
        ),
    );
    let imaging = g::seq(
        g::send::<IMAGING_FILE, IMAGING_SERVICE, ImagingIntent>(),
        g::seq(
            g::send::<IMAGING_SERVICE, ER_DOCTOR, ImagingAdmitted>(),
            g::seq(
                g::send::<IMAGING_SERVICE, ER_DOCTOR, CtLoan>(),
                g::seq(
                    g::send::<CT_SCANNER, IMAGING_SERVICE, ImagingRequest>(),
                    g::seq(
                        g::par(
                            g::send::<IMAGING_SERVICE, CT_SCANNER, ImagingResult>(),
                            g::send::<IMAGING_SERVICE, IMAGING_FILE, FileImagingResult>(),
                        ),
                        g::send::<ER_DOCTOR, IMAGING_SERVICE, CtLoan>(),
                    ),
                ),
            ),
        ),
    );
    let referral = g::par(
        g::send::<ER_DOCTOR, CARDIOLOGIST, Referral>(),
        g::par(
            g::send::<ER_DOCTOR, CARDIAC_SERVICE, ReferralForOwner>(),
            g::send::<ER_DOCTOR, REFERRAL_FILE, FileReferral>(),
        ),
    );
    let cardiac_attempt = || {
        g::seq(
            g::send::<CARDIAC_SERVICE, CARDIOLOGIST, CardiacPanelLoan>(),
            g::seq(
                g::send::<CARDIAC_FILE, CARDIAC_SERVICE, CardiacFileRequest>(),
                g::seq(
                    g::send::<CARDIAC_SERVICE, CARDIOLOGIST, CardiacAdmitted>(),
                    g::seq(
                        g::send::<CARDIAC_PANEL, CARDIAC_SERVICE, CardiacTests>(),
                        g::seq(
                            g::send::<CARDIAC_SERVICE, CARDIAC_PANEL, CardiacTestResults>(),
                            g::send::<CARDIOLOGIST, CARDIAC_SERVICE, CardiacPanelReturn>(),
                        ),
                    ),
                ),
            ),
        )
    };
    let cardiac_ready = || {
        g::seq(
            g::send::<CARDIAC_SERVICE, RECORDS, CardiacCompletedData>(),
            g::par(
                g::send::<CARDIAC_SERVICE, CARDIOLOGIST, CardiacTestsComplete>(),
                g::send::<CARDIAC_SERVICE, ER_DOCTOR, CardiacPanelReport>(),
            ),
        )
    };
    let cardiology = g::seq(
        referral,
        g::seq(
            cardiac_attempt(),
            g::seq(
                g::route(
                    cardiac_ready(),
                    g::seq(
                        g::seq(
                            g::send::<CARDIAC_SERVICE, CARDIAC_FILE, FileCardiacResult>(),
                            g::seq(
                                g::send::<CARDIAC_SERVICE, CARDIOLOGIST, CardiacTestsPartial>(),
                                cardiac_attempt(),
                            ),
                        )
                        .roll(),
                        cardiac_ready(),
                    ),
                )
                .resolve::<CARDIAC_COMPLETE>(),
                g::seq(
                    g::send::<CARDIAC_SERVICE, RECORDS, CardiacPanelCompleted>(),
                    g::send::<RECORDS, CARDIAC_FILE, FileCardiacResult>(),
                ),
            ),
        ),
    );
    let findings = g::par(
        g::send::<ER_DOCTOR, CARDIOLOGIST, GeneralFindings>(),
        g::par(
            g::send::<ER_DOCTOR, CARDIAC_SERVICE, FindingsForOwner>(),
            g::send::<ER_DOCTOR, FINDINGS_FILE, FileFindings>(),
        ),
    );
    let diagnosis = g::seq(
        g::send::<DIAGNOSIS_FILE, CARDIAC_SERVICE, DiagnosisRequest>(),
        g::seq(
            g::par(
                g::send::<CARDIAC_SERVICE, CARDIOLOGIST, DiagnosisResult>(),
                g::send::<CARDIAC_SERVICE, DIAGNOSIS_FILE, FileDiagnosisResult>(),
            ),
            g::send::<CARDIOLOGIST, ER_DOCTOR, CardiacFindings>(),
        ),
    );
    let clinical = g::seq(
        assessment,
        g::seq(
            g::par(g::par(general_tests, imaging), cardiology),
            g::seq(findings, diagnosis),
        ),
    );
    // This operation reviews caller data; no drug-administration effect exists.
    let review = g::seq(
        g::send::<REVIEW_FILE, PHARMACY_SERVICE, ReviewRequest>(),
        g::seq(
            g::par(
                g::send::<PHARMACY_SERVICE, PHARMACIST, ReviewResult>(),
                g::send::<PHARMACY_SERVICE, REVIEW_FILE, FileReviewResult>(),
            ),
            g::send::<PHARMACIST, ER_DOCTOR, MedicationReview>(),
        ),
    );
    let task = g::seq(
        g::par(clinical, review),
        g::seq(
            g::send::<ER_DOCTOR, RECORDS, CompletedConsultation>(),
            g::send::<RECORDS, SUMMARY_FILE, Summary>(),
        ),
    );
    let bootstrap = g::par(
        g::par(
            g::par(
                g::send::<ASSESS_DATA, ER_DOCTOR, Loan<ASSESS_FILE>>(),
                g::send::<GENERAL_DATA, ER_DOCTOR, Loan<GENERAL_FILE>>(),
            ),
            g::par(
                g::send::<IMAGING_DATA, ER_DOCTOR, Loan<IMAGING_FILE>>(),
                g::send::<REFERRAL_DATA, CARDIOLOGIST, Loan<REFERRAL_FILE>>(),
            ),
        ),
        g::par(
            g::par(
                g::send::<CARDIAC_DATA, CARDIOLOGIST, Loan<CARDIAC_FILE>>(),
                g::send::<FINDINGS_DATA, CARDIOLOGIST, Loan<FINDINGS_FILE>>(),
            ),
            g::par(
                g::send::<DIAGNOSIS_DATA, CARDIOLOGIST, Loan<DIAGNOSIS_FILE>>(),
                g::par(
                    g::send::<REVIEW_DATA, PHARMACIST, Loan<REVIEW_FILE>>(),
                    g::send::<SUMMARY_DATA, ER_DOCTOR, Loan<SUMMARY_FILE>>(),
                ),
            ),
        ),
    );
    let metadata = g::par(
        g::par(
            g::par(
                g::seq(
                    crate::shell::global::file_metadata::<ASSESS_FILE, ASSESS_DATA>().roll(),
                    g::send::<ASSESS_FILE, ASSESS_DATA, FileExit>(),
                ),
                g::seq(
                    crate::shell::global::file_metadata::<GENERAL_FILE, GENERAL_DATA>().roll(),
                    g::send::<GENERAL_FILE, GENERAL_DATA, FileExit>(),
                ),
            ),
            g::par(
                g::seq(
                    crate::shell::global::file_metadata::<IMAGING_FILE, IMAGING_DATA>().roll(),
                    g::send::<IMAGING_FILE, IMAGING_DATA, FileExit>(),
                ),
                g::seq(
                    crate::shell::global::file_metadata::<REFERRAL_FILE, REFERRAL_DATA>().roll(),
                    g::send::<REFERRAL_FILE, REFERRAL_DATA, FileExit>(),
                ),
            ),
        ),
        g::par(
            g::par(
                g::seq(
                    crate::shell::global::file_metadata::<CARDIAC_FILE, CARDIAC_DATA>().roll(),
                    g::send::<CARDIAC_FILE, CARDIAC_DATA, FileExit>(),
                ),
                g::seq(
                    crate::shell::global::file_metadata::<FINDINGS_FILE, FINDINGS_DATA>().roll(),
                    g::send::<FINDINGS_FILE, FINDINGS_DATA, FileExit>(),
                ),
            ),
            g::par(
                g::seq(
                    crate::shell::global::file_metadata::<DIAGNOSIS_FILE, DIAGNOSIS_DATA>().roll(),
                    g::send::<DIAGNOSIS_FILE, DIAGNOSIS_DATA, FileExit>(),
                ),
                g::par(
                    g::seq(
                        crate::shell::global::file_metadata::<REVIEW_FILE, REVIEW_DATA>().roll(),
                        g::send::<REVIEW_FILE, REVIEW_DATA, FileExit>(),
                    ),
                    g::seq(
                        crate::shell::global::file_metadata::<SUMMARY_FILE, SUMMARY_DATA>().roll(),
                        g::send::<SUMMARY_FILE, SUMMARY_DATA, FileExit>(),
                    ),
                ),
            ),
        ),
    );
    let private = g::par(
        g::seq(
            crate::shell::global::private_io::<ER_VM, ER_IO>(),
            g::send::<ER_VM, ER_IO, crate::shell::global::Exit>(),
        ),
        g::par(
            g::seq(
                crate::shell::global::private_io::<CARDIO_VM, CARDIO_IO>(),
                g::send::<CARDIO_VM, CARDIO_IO, crate::shell::global::Exit>(),
            ),
            g::seq(
                crate::shell::global::private_io::<PHARMACY_VM, PHARMACY_IO>(),
                g::send::<PHARMACY_VM, PHARMACY_IO, crate::shell::global::Exit>(),
            ),
        ),
    );
    g::seq(bootstrap, g::par(task, g::par(metadata, private)))
}

pub struct Programs {
    pub records: RoleProgram<RECORDS>,
    pub er: RoleProgram<ER_DOCTOR>,
    pub cardiologist: RoleProgram<CARDIOLOGIST>,
    pub pharmacist: RoleProgram<PHARMACIST>,
    pub er_io: RoleProgram<ER_IO>,
    pub cardio_io: RoleProgram<CARDIO_IO>,
    pub pharmacy_io: RoleProgram<PHARMACY_IO>,
    pub general_service: RoleProgram<GENERAL_SERVICE>,
    pub cardiac_service: RoleProgram<CARDIAC_SERVICE>,
    pub pharmacy_service: RoleProgram<PHARMACY_SERVICE>,
    pub imaging_service: RoleProgram<IMAGING_SERVICE>,
    pub ct: RoleProgram<CT_SCANNER>,
    pub general_panel: RoleProgram<GENERAL_PANEL>,
    pub cardiac_panel: RoleProgram<CARDIAC_PANEL>,
    pub er_vm: RoleProgram<ER_VM>,
    pub cardio_vm: RoleProgram<CARDIO_VM>,
    pub pharmacy_vm: RoleProgram<PHARMACY_VM>,
    pub assess_file: RoleProgram<ASSESS_FILE>,
    pub general_file: RoleProgram<GENERAL_FILE>,
    pub imaging_file: RoleProgram<IMAGING_FILE>,
    pub referral_file: RoleProgram<REFERRAL_FILE>,
    pub cardiac_file: RoleProgram<CARDIAC_FILE>,
    pub findings_file: RoleProgram<FINDINGS_FILE>,
    pub diagnosis_file: RoleProgram<DIAGNOSIS_FILE>,
    pub review_file: RoleProgram<REVIEW_FILE>,
    pub summary_file: RoleProgram<SUMMARY_FILE>,
    pub assess_data: RoleProgram<ASSESS_DATA>,
    pub general_data: RoleProgram<GENERAL_DATA>,
    pub imaging_data: RoleProgram<IMAGING_DATA>,
    pub referral_data: RoleProgram<REFERRAL_DATA>,
    pub cardiac_data: RoleProgram<CARDIAC_DATA>,
    pub findings_data: RoleProgram<FINDINGS_DATA>,
    pub diagnosis_data: RoleProgram<DIAGNOSIS_DATA>,
    pub review_data: RoleProgram<REVIEW_DATA>,
    pub summary_data: RoleProgram<SUMMARY_DATA>,
}
impl Programs {
    pub fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        use hibana::g::Message;
        let labels: serde_json::Map<String, serde_json::Value> = [
            ("AssessmentRequest", AssessmentRequest::LOGICAL_LABEL),
            ("AssessmentResult", AssessmentResult::LOGICAL_LABEL),
            ("GeneralTests", GeneralTests::LOGICAL_LABEL),
            ("GeneralTestResults", GeneralTestResults::LOGICAL_LABEL),
            ("CardiacTests", CardiacTests::LOGICAL_LABEL),
            ("CardiacTestResults", CardiacTestResults::LOGICAL_LABEL),
            ("ImagingIntent", ImagingIntent::LOGICAL_LABEL),
            ("CtLoan", CtLoan::LOGICAL_LABEL),
            ("ImagingRequest", ImagingRequest::LOGICAL_LABEL),
            ("ImagingResult", ImagingResult::LOGICAL_LABEL),
            ("Referral", Referral::LOGICAL_LABEL),
            ("ReferralForOwner", ReferralForOwner::LOGICAL_LABEL),
            ("GeneralFindings", GeneralFindings::LOGICAL_LABEL),
            ("FindingsForOwner", FindingsForOwner::LOGICAL_LABEL),
            ("DiagnosisRequest", DiagnosisRequest::LOGICAL_LABEL),
            ("DiagnosisResult", DiagnosisResult::LOGICAL_LABEL),
            ("CardiacFindings", CardiacFindings::LOGICAL_LABEL),
            ("ReviewRequest", ReviewRequest::LOGICAL_LABEL),
            ("ReviewResult", ReviewResult::LOGICAL_LABEL),
            ("MedicationReview", MedicationReview::LOGICAL_LABEL),
            ("GeneralPanelLoan", GeneralPanelLoan::LOGICAL_LABEL),
            ("GeneralTestsPartial", GeneralTestsPartial::LOGICAL_LABEL),
            ("GeneralPanelReturn", GeneralPanelReturn::LOGICAL_LABEL),
            (
                "GeneralPanelCompleted",
                GeneralPanelCompleted::LOGICAL_LABEL,
            ),
            ("GeneralTestsComplete", GeneralTestsComplete::LOGICAL_LABEL),
            ("CardiacPanelLoan", CardiacPanelLoan::LOGICAL_LABEL),
            ("CardiacPanelReturn", CardiacPanelReturn::LOGICAL_LABEL),
            ("CardiacTestsPartial", CardiacTestsPartial::LOGICAL_LABEL),
            (
                "CardiacPanelCompleted",
                CardiacPanelCompleted::LOGICAL_LABEL,
            ),
            ("CardiacTestsComplete", CardiacTestsComplete::LOGICAL_LABEL),
            ("FileAssessmentResult", FileAssessmentResult::LOGICAL_LABEL),
            ("GeneralFileRequest", GeneralFileRequest::LOGICAL_LABEL),
            ("GeneralAdmitted", GeneralAdmitted::LOGICAL_LABEL),
            ("FileGeneralResult", FileGeneralResult::LOGICAL_LABEL),
            ("ImagingAdmitted", ImagingAdmitted::LOGICAL_LABEL),
            ("FileImagingResult", FileImagingResult::LOGICAL_LABEL),
            ("FileReferral", FileReferral::LOGICAL_LABEL),
            ("FileFindings", FileFindings::LOGICAL_LABEL),
            ("FileDiagnosisResult", FileDiagnosisResult::LOGICAL_LABEL),
            ("FileReviewResult", FileReviewResult::LOGICAL_LABEL),
            ("Summary", Summary::LOGICAL_LABEL),
            ("CardiacFileRequest", CardiacFileRequest::LOGICAL_LABEL),
            ("CardiacAdmitted", CardiacAdmitted::LOGICAL_LABEL),
            ("FileCardiacResult", FileCardiacResult::LOGICAL_LABEL),
            ("GeneralCompletedData", GeneralCompletedData::LOGICAL_LABEL),
            ("CardiacCompletedData", CardiacCompletedData::LOGICAL_LABEL),
            ("CardiacPanelReport", CardiacPanelReport::LOGICAL_LABEL),
            (
                "CompletedConsultation",
                CompletedConsultation::LOGICAL_LABEL,
            ),
        ]
        .into_iter()
        .map(|(name, label)| (name.to_owned(), label.into()))
        .collect();
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"5E","session":SESSION,
            "roles":{"RECORDS":RECORDS,"ER_DOCTOR":ER_DOCTOR,"CARDIOLOGIST":CARDIOLOGIST,"PHARMACIST":PHARMACIST,"ER_IO":ER_IO,"CARDIO_IO":CARDIO_IO,"PHARMACY_IO":PHARMACY_IO,"GENERAL_SERVICE":GENERAL_SERVICE,"CARDIAC_SERVICE":CARDIAC_SERVICE,"PHARMACY_SERVICE":PHARMACY_SERVICE,"IMAGING_SERVICE":IMAGING_SERVICE,"CT_SCANNER":CT_SCANNER,"GENERAL_PANEL":GENERAL_PANEL,"CARDIAC_PANEL":CARDIAC_PANEL,"ER_VM":ER_VM,"CARDIO_VM":CARDIO_VM,"PHARMACY_VM":PHARMACY_VM,"ASSESS_FILE":ASSESS_FILE,"GENERAL_FILE":GENERAL_FILE,"IMAGING_FILE":IMAGING_FILE,"REFERRAL_FILE":REFERRAL_FILE,"CARDIAC_FILE":CARDIAC_FILE,"FINDINGS_FILE":FINDINGS_FILE,"DIAGNOSIS_FILE":DIAGNOSIS_FILE,"REVIEW_FILE":REVIEW_FILE,"SUMMARY_FILE":SUMMARY_FILE,"ASSESS_DATA":ASSESS_DATA,"GENERAL_DATA":GENERAL_DATA,"IMAGING_DATA":IMAGING_DATA,"REFERRAL_DATA":REFERRAL_DATA,"CARDIAC_DATA":CARDIAC_DATA,"FINDINGS_DATA":FINDINGS_DATA,"DIAGNOSIS_DATA":DIAGNOSIS_DATA,"REVIEW_DATA":REVIEW_DATA,"SUMMARY_DATA":SUMMARY_DATA},
            "labels":labels,"programs":[rows(&self.records),rows(&self.er),rows(&self.cardiologist),rows(&self.pharmacist),rows(&self.er_io),rows(&self.cardio_io),rows(&self.pharmacy_io),rows(&self.general_service),rows(&self.cardiac_service),rows(&self.pharmacy_service),rows(&self.imaging_service),rows(&self.ct),rows(&self.general_panel),rows(&self.cardiac_panel),rows(&self.er_vm),rows(&self.cardio_vm),rows(&self.pharmacy_vm),rows(&self.assess_file),rows(&self.general_file),rows(&self.imaging_file),rows(&self.referral_file),rows(&self.cardiac_file),rows(&self.findings_file),rows(&self.diagnosis_file),rows(&self.review_file),rows(&self.summary_file),rows(&self.assess_data),rows(&self.general_data),rows(&self.imaging_data),rows(&self.referral_data),rows(&self.cardiac_data),rows(&self.findings_data),rows(&self.diagnosis_data),rows(&self.review_data),rows(&self.summary_data)]})
    }
}
pub fn programs() -> Programs {
    let source = source();
    Programs {
        records: project(&source),
        er: project(&source),
        cardiologist: project(&source),
        pharmacist: project(&source),
        er_io: project(&source),
        cardio_io: project(&source),
        pharmacy_io: project(&source),
        general_service: project(&source),
        cardiac_service: project(&source),
        pharmacy_service: project(&source),
        imaging_service: project(&source),
        ct: project(&source),
        general_panel: project(&source),
        cardiac_panel: project(&source),
        er_vm: project(&source),
        cardio_vm: project(&source),
        pharmacy_vm: project(&source),
        assess_file: project(&source),
        general_file: project(&source),
        imaging_file: project(&source),
        referral_file: project(&source),
        cardiac_file: project(&source),
        findings_file: project(&source),
        diagnosis_file: project(&source),
        review_file: project(&source),
        summary_file: project(&source),
        assess_data: project(&source),
        general_data: project(&source),
        imaging_data: project(&source),
        referral_data: project(&source),
        cardiac_data: project(&source),
        findings_data: project(&source),
        diagnosis_data: project(&source),
        review_data: project(&source),
        summary_data: project(&source),
    }
}
