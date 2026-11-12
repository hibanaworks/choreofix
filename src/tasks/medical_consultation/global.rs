//! Patient findings and actual CT custody for the 5E consultation.
use hibana::{
    Endpoint, g,
    runtime::program::{RoleProgram, project},
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
pub const GENERAL_COMPLETE: u16 = 501;
pub const CARDIAC_COMPLETE: u16 = 502;
pub const SESSION: u32 = 50;

pub type AssessmentRequest<'a> = g::Msg<1, &'a [u8]>;
pub type AssessmentResult<'a> = g::Msg<2, &'a [u8]>;
pub type GeneralTests<'a> = g::Msg<3, &'a [u8]>;
pub type GeneralTestResults<'a> = g::Msg<4, &'a [u8]>;
pub type CardiacTests<'a> = g::Msg<5, &'a [u8]>;
pub type CardiacTestResults<'a> = g::Msg<6, &'a [u8]>;
pub type ImagingIntent<'a> = g::Msg<7, &'a [u8]>;
pub type CtLoan<'r> = g::Msg<8, Endpoint<'r, CT_SCANNER>>;
pub type ImagingRequest<'a> = g::Msg<9, &'a [u8]>;
pub type ImagingResult<'a> = g::Msg<10, &'a [u8]>;
pub type Referral<'a> = g::Msg<11, &'a [u8]>;
pub type ReferralForOwner<'a> = g::Msg<12, &'a [u8]>;
pub type GeneralFindings<'a> = g::Msg<13, &'a [u8]>;
pub type FindingsForOwner<'a> = g::Msg<14, &'a [u8]>;
pub type DiagnosisRequest<'a> = g::Msg<15, &'a [u8]>;
pub type DiagnosisResult<'a> = g::Msg<16, &'a [u8]>;
pub type CardiacFindings<'a> = g::Msg<17, &'a [u8]>;
pub type ReviewRequest<'a> = g::Msg<18, &'a [u8]>;
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

/// Immutable sealed images, not another clinical-progress or permission table.
pub struct Programs {
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
        ]
        .into_iter()
        .map(|(name, label)| (name.into(), label.into()))
        .collect();
        serde_json::json!({
            "type":"compiled_projection","format_version":1,"task":"5E","session":SESSION,
            "roles":{"ER_DOCTOR":ER_DOCTOR,"CARDIOLOGIST":CARDIOLOGIST,"PHARMACIST":PHARMACIST,"ER_IO":ER_IO,"CARDIO_IO":CARDIO_IO,"PHARMACY_IO":PHARMACY_IO,"GENERAL_SERVICE":GENERAL_SERVICE,"CARDIAC_SERVICE":CARDIAC_SERVICE,"PHARMACY_SERVICE":PHARMACY_SERVICE,"IMAGING_SERVICE":IMAGING_SERVICE,"CT_SCANNER":CT_SCANNER,"GENERAL_PANEL":GENERAL_PANEL,"CARDIAC_PANEL":CARDIAC_PANEL},
            "labels":labels,
            "programs":[rows(&self.er),rows(&self.cardiologist),rows(&self.pharmacist),rows(&self.er_io),rows(&self.cardio_io),rows(&self.pharmacy_io),rows(&self.general_service),rows(&self.cardiac_service),rows(&self.pharmacy_service),rows(&self.imaging_service),rows(&self.ct),rows(&self.general_panel),rows(&self.cardiac_panel)]
        })
    }
}

pub fn programs() -> Programs {
    let assessment = g::seq(
        g::send::<ER_DOCTOR, GENERAL_SERVICE, AssessmentRequest>(),
        g::send::<GENERAL_SERVICE, ER_DOCTOR, AssessmentResult>(),
    );
    let general_attempt = || {
        g::seq(
            g::send::<GENERAL_SERVICE, ER_DOCTOR, GeneralPanelLoan>(),
            g::seq(
                g::send::<GENERAL_PANEL, GENERAL_SERVICE, GeneralTests>(),
                g::seq(
                    g::send::<GENERAL_SERVICE, GENERAL_PANEL, GeneralTestResults>(),
                    g::send::<ER_DOCTOR, GENERAL_SERVICE, GeneralPanelReturn>(),
                ),
            ),
        )
    };
    let general_complete = || {
        g::seq(
            g::send::<GENERAL_SERVICE, ER_DOCTOR, GeneralPanelCompleted>(),
            g::send::<GENERAL_SERVICE, ER_DOCTOR, GeneralTestsComplete>(),
        )
    };
    let general_tests = g::seq(
        general_attempt(),
        g::route(
            general_complete(),
            g::seq(
                g::seq(
                    g::send::<GENERAL_SERVICE, ER_DOCTOR, GeneralTestsPartial>(),
                    general_attempt(),
                )
                .roll(),
                general_complete(),
            ),
        )
        .resolve::<GENERAL_COMPLETE>(),
    );
    let imaging = g::seq(
        g::send::<ER_DOCTOR, IMAGING_SERVICE, ImagingIntent>(),
        g::seq(
            g::send::<IMAGING_SERVICE, ER_DOCTOR, CtLoan>(),
            g::seq(
                g::send::<CT_SCANNER, IMAGING_SERVICE, ImagingRequest>(),
                g::seq(
                    g::send::<IMAGING_SERVICE, CT_SCANNER, ImagingResult>(),
                    g::send::<ER_DOCTOR, IMAGING_SERVICE, CtLoan>(),
                ),
            ),
        ),
    );
    let cardiac_attempt = || {
        g::seq(
            g::send::<CARDIAC_SERVICE, CARDIOLOGIST, CardiacPanelLoan>(),
            g::seq(
                g::send::<CARDIAC_PANEL, CARDIAC_SERVICE, CardiacTests>(),
                g::seq(
                    g::send::<CARDIAC_SERVICE, CARDIAC_PANEL, CardiacTestResults>(),
                    g::send::<CARDIOLOGIST, CARDIAC_SERVICE, CardiacPanelReturn>(),
                ),
            ),
        )
    };
    let cardiac_complete = || {
        g::seq(
            g::send::<CARDIAC_SERVICE, ER_DOCTOR, CardiacPanelCompleted>(),
            g::send::<CARDIAC_SERVICE, CARDIOLOGIST, CardiacTestsComplete>(),
        )
    };
    let cardiology = g::seq(
        g::par(
            g::send::<ER_DOCTOR, CARDIOLOGIST, Referral>(),
            g::send::<ER_DOCTOR, CARDIAC_SERVICE, ReferralForOwner>(),
        ),
        g::seq(
            cardiac_attempt(),
            g::route(
                cardiac_complete(),
                g::seq(
                    g::seq(
                        g::send::<CARDIAC_SERVICE, CARDIOLOGIST, CardiacTestsPartial>(),
                        cardiac_attempt(),
                    )
                    .roll(),
                    cardiac_complete(),
                ),
            )
            .resolve::<CARDIAC_COMPLETE>(),
        ),
    );
    let findings = g::par(
        g::send::<ER_DOCTOR, CARDIOLOGIST, GeneralFindings>(),
        g::send::<ER_DOCTOR, CARDIAC_SERVICE, FindingsForOwner>(),
    );
    let diagnosis = g::seq(
        g::send::<CARDIOLOGIST, CARDIAC_SERVICE, DiagnosisRequest>(),
        g::seq(
            g::send::<CARDIAC_SERVICE, CARDIOLOGIST, DiagnosisResult>(),
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
    // Review uses caller-supplied medication data. No prescribing or
    // administration effect exists in this task's public domain interface.
    let review = g::seq(
        g::send::<PHARMACIST, PHARMACY_SERVICE, ReviewRequest>(),
        g::seq(
            g::send::<PHARMACY_SERVICE, PHARMACIST, ReviewResult>(),
            g::send::<PHARMACIST, ER_DOCTOR, MedicationReview>(),
        ),
    );
    let global = g::par(
        g::par(clinical, review),
        g::par(
            crate::shell::global::private_io::<ER_DOCTOR, ER_IO>(),
            g::par(
                crate::shell::global::private_io::<CARDIOLOGIST, CARDIO_IO>(),
                crate::shell::global::private_io::<PHARMACIST, PHARMACY_IO>(),
            ),
        ),
    );
    Programs {
        er: project::<ER_DOCTOR>(&global),
        cardiologist: project::<CARDIOLOGIST>(&global),
        pharmacist: project::<PHARMACIST>(&global),
        er_io: project::<ER_IO>(&global),
        cardio_io: project::<CARDIO_IO>(&global),
        pharmacy_io: project::<PHARMACY_IO>(&global),
        general_service: project::<GENERAL_SERVICE>(&global),
        cardiac_service: project::<CARDIAC_SERVICE>(&global),
        pharmacy_service: project::<PHARMACY_SERVICE>(&global),
        imaging_service: project::<IMAGING_SERVICE>(&global),
        ct: project::<CT_SCANNER>(&global),
        general_panel: project::<GENERAL_PANEL>(&global),
        cardiac_panel: project::<CARDIAC_PANEL>(&global),
    }
}
