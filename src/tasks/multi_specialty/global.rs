//! Joint investigation data, two independent image pipelines and conference custody.
use hibana::{
    Endpoint, g,
    runtime::program::{RoleProgram, project},
};
pub const ER_DOCTOR: u8 = 0;
pub const CARDIOLOGIST: u8 = 1;
pub const NEUROLOGIST: u8 = 2;
pub const RADIOLOGIST: u8 = 3;
pub const PHARMACIST: u8 = 4;
pub const ER_IO: u8 = 5;
pub const CARDIO_IO: u8 = 6;
pub const NEURO_IO: u8 = 7;
pub const RADIOLOGY_IO: u8 = 8;
pub const PHARMACY_IO: u8 = 9;
pub const GENERAL_SERVICE: u8 = 10;
pub const CARDIAC_SERVICE: u8 = 11;
pub const NEURO_SERVICE: u8 = 12;
pub const CT_SERVICE: u8 = 13;
pub const MRI_SERVICE: u8 = 14;
pub const PHARMACY_SERVICE: u8 = 15;
pub const CONFERENCE_SERVICE: u8 = 16;
pub const GENERAL_PANEL: u8 = 17;
pub const CARDIAC_PANEL: u8 = 18;
pub const NEURO_PANEL: u8 = 19;
pub const CT_SCANNER: u8 = 20;
pub const MRI: u8 = 21;
pub const CONSULTATION_ROOM: u8 = 22;
pub const GENERAL_COMPLETE: u16 = 601;
pub const CARDIAC_COMPLETE: u16 = 602;
pub const NEURO_COMPLETE: u16 = 603;
pub const SESSION: u32 = 51;
pub type PanelLoan<'r, const RESOURCE: u8> = g::Msg<1, Endpoint<'r, RESOURCE>>;
pub type PanelReturn<'r, const RESOURCE: u8> = g::Msg<2, Endpoint<'r, RESOURCE>>;
pub type PanelCompleted<'r, const RESOURCE: u8> = g::Msg<3, Endpoint<'r, RESOURCE>>;
pub type TestRequest<'a> = g::Msg<4, &'a [u8]>;
pub type TestResult<'a> = g::Msg<5, &'a [u8]>;
pub type PanelPartial<'a> = g::Msg<6, &'a [u8]>;
pub type PanelComplete<'a> = g::Msg<7, &'a [u8]>;
pub type CardReferral<'a> = g::Msg<8, &'a [u8]>;
pub type NeuroReferral<'a> = g::Msg<69, &'a [u8]>;
pub type ReferralForOwner<'a> = g::Msg<9, &'a [u8]>;
pub type CtOrder<'a> = g::Msg<10, &'a [u8]>;
pub type MriOrder<'a> = g::Msg<11, &'a [u8]>;
pub type CtOrderForOwner<'a> = g::Msg<12, &'a [u8]>;
pub type MriOrderForOwner<'a> = g::Msg<13, &'a [u8]>;
pub type CtPerformIntent<'a> = g::Msg<14, &'a [u8]>;
pub type MriPerformIntent<'a> = g::Msg<15, &'a [u8]>;
pub type CtLoan<'r> = g::Msg<16, Endpoint<'r, CT_SCANNER>>;
pub type MriLoan<'r> = g::Msg<17, Endpoint<'r, MRI>>;
pub type CtPerformRequest<'a> = g::Msg<18, &'a [u8]>;
pub type MriPerformRequest<'a> = g::Msg<19, &'a [u8]>;
pub type CtImageResult<'a> = g::Msg<20, &'a [u8]>;
pub type MriImageResult<'a> = g::Msg<21, &'a [u8]>;
pub type CtInterpretIntent<'a> = g::Msg<22, &'a [u8]>;
pub type MriInterpretIntent<'a> = g::Msg<23, &'a [u8]>;
pub type CtInterpretRequest<'a> = g::Msg<24, &'a [u8]>;
pub type MriInterpretRequest<'a> = g::Msg<25, &'a [u8]>;
pub type CtReportResult<'a> = g::Msg<26, &'a [u8]>;
pub type MriReportResult<'a> = g::Msg<27, &'a [u8]>;
pub type GeneralForCard<'a> = g::Msg<28, &'a [u8]>;
pub type GeneralForNeuro<'a> = g::Msg<29, &'a [u8]>;
pub type GeneralForCardOwner<'a> = g::Msg<30, &'a [u8]>;
pub type GeneralForNeuroOwner<'a> = g::Msg<31, &'a [u8]>;
pub type GeneralForConference<'a> = g::Msg<32, &'a [u8]>;
pub type CardPanelForNeuro<'a> = g::Msg<33, &'a [u8]>;
pub type NeuroPanelForCard<'a> = g::Msg<34, &'a [u8]>;
pub type CtForCard<'a> = g::Msg<35, &'a [u8]>;
pub type CtForNeuro<'a> = g::Msg<36, &'a [u8]>;
pub type CtForCardOwner<'a> = g::Msg<37, &'a [u8]>;
pub type CtForNeuroOwner<'a> = g::Msg<38, &'a [u8]>;
pub type CtForER<'a> = g::Msg<39, &'a [u8]>;
pub type CtForConference<'a> = g::Msg<40, &'a [u8]>;
pub type MriForCard<'a> = g::Msg<41, &'a [u8]>;
pub type MriForNeuro<'a> = g::Msg<42, &'a [u8]>;
pub type MriForCardOwner<'a> = g::Msg<43, &'a [u8]>;
pub type MriForNeuroOwner<'a> = g::Msg<44, &'a [u8]>;
pub type MriForER<'a> = g::Msg<45, &'a [u8]>;
pub type MriForConference<'a> = g::Msg<46, &'a [u8]>;
pub type CardDiagnosisRequest<'a> = g::Msg<47, &'a [u8]>;
pub type CardDiagnosisResult<'a> = g::Msg<48, &'a [u8]>;
pub type NeuroDiagnosisRequest<'a> = g::Msg<49, &'a [u8]>;
pub type NeuroDiagnosisResult<'a> = g::Msg<50, &'a [u8]>;
pub type CardDiagnosisForER<'a> = g::Msg<51, &'a [u8]>;
pub type NeuroDiagnosisForER<'a> = g::Msg<52, &'a [u8]>;
pub type CardDiagnosisForConference<'a> = g::Msg<53, &'a [u8]>;
pub type NeuroDiagnosisForConference<'a> = g::Msg<54, &'a [u8]>;
pub type AssessmentRequest<'a> = g::Msg<55, &'a [u8]>;
pub type AssessmentResult<'a> = g::Msg<56, &'a [u8]>;
pub type ReviewRequest<'a> = g::Msg<57, &'a [u8]>;
pub type ReviewResult<'a> = g::Msg<58, &'a [u8]>;
pub type ReviewForER<'a> = g::Msg<59, &'a [u8]>;
pub type ReviewForConference<'a> = g::Msg<60, &'a [u8]>;
pub type ConferenceIntent<'a> = g::Msg<61, &'a [u8]>;
pub type RoomLoan<'r> = g::Msg<62, Endpoint<'r, CONSULTATION_ROOM>>;
pub type RoomRequest<'a> = g::Msg<63, &'a [u8]>;
pub type RoomResult<'a> = g::Msg<64, &'a [u8]>;
pub type FinalForER<'a> = g::Msg<65, &'a [u8]>;
pub type FinalForCard<'a> = g::Msg<66, &'a [u8]>;
pub type FinalForNeuro<'a> = g::Msg<67, &'a [u8]>;
pub type FinalForRad<'a> = g::Msg<68, &'a [u8]>;

pub struct Programs {
    pub er: RoleProgram<ER_DOCTOR>,
    pub cardio: RoleProgram<CARDIOLOGIST>,
    pub neuro: RoleProgram<NEUROLOGIST>,
    pub radiologist: RoleProgram<RADIOLOGIST>,
    pub pharmacist: RoleProgram<PHARMACIST>,
    pub er_io: RoleProgram<ER_IO>,
    pub cardio_io: RoleProgram<CARDIO_IO>,
    pub neuro_io: RoleProgram<NEURO_IO>,
    pub radiology_io: RoleProgram<RADIOLOGY_IO>,
    pub pharmacy_io: RoleProgram<PHARMACY_IO>,
    pub general: RoleProgram<GENERAL_SERVICE>,
    pub cardiac: RoleProgram<CARDIAC_SERVICE>,
    pub neurology: RoleProgram<NEURO_SERVICE>,
    pub ct_service: RoleProgram<CT_SERVICE>,
    pub mri_service: RoleProgram<MRI_SERVICE>,
    pub pharmacy: RoleProgram<PHARMACY_SERVICE>,
    pub conference: RoleProgram<CONFERENCE_SERVICE>,
    pub general_panel: RoleProgram<GENERAL_PANEL>,
    pub cardiac_panel: RoleProgram<CARDIAC_PANEL>,
    pub neuro_panel: RoleProgram<NEURO_PANEL>,
    pub ct: RoleProgram<CT_SCANNER>,
    pub mri: RoleProgram<MRI>,
    pub room: RoleProgram<CONSULTATION_ROOM>,
}
impl Programs {
    pub fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        use hibana::g::Message;
        let labels: serde_json::Map<String, serde_json::Value> = [
            ("PanelLoan", PanelLoan::<GENERAL_PANEL>::LOGICAL_LABEL),
            ("PanelReturn", PanelReturn::<GENERAL_PANEL>::LOGICAL_LABEL),
            (
                "PanelCompleted",
                PanelCompleted::<GENERAL_PANEL>::LOGICAL_LABEL,
            ),
            ("TestRequest", TestRequest::LOGICAL_LABEL),
            ("TestResult", TestResult::LOGICAL_LABEL),
            ("PanelPartial", PanelPartial::LOGICAL_LABEL),
            ("PanelComplete", PanelComplete::LOGICAL_LABEL),
            ("CardReferral", CardReferral::LOGICAL_LABEL),
            ("NeuroReferral", NeuroReferral::LOGICAL_LABEL),
            ("ReferralForOwner", ReferralForOwner::LOGICAL_LABEL),
            ("CtOrder", CtOrder::LOGICAL_LABEL),
            ("MriOrder", MriOrder::LOGICAL_LABEL),
            ("CtOrderForOwner", CtOrderForOwner::LOGICAL_LABEL),
            ("MriOrderForOwner", MriOrderForOwner::LOGICAL_LABEL),
            ("CtPerformIntent", CtPerformIntent::LOGICAL_LABEL),
            ("MriPerformIntent", MriPerformIntent::LOGICAL_LABEL),
            ("CtLoan", CtLoan::LOGICAL_LABEL),
            ("MriLoan", MriLoan::LOGICAL_LABEL),
            ("CtPerformRequest", CtPerformRequest::LOGICAL_LABEL),
            ("MriPerformRequest", MriPerformRequest::LOGICAL_LABEL),
            ("CtImageResult", CtImageResult::LOGICAL_LABEL),
            ("MriImageResult", MriImageResult::LOGICAL_LABEL),
            ("CtInterpretIntent", CtInterpretIntent::LOGICAL_LABEL),
            ("MriInterpretIntent", MriInterpretIntent::LOGICAL_LABEL),
            ("CtInterpretRequest", CtInterpretRequest::LOGICAL_LABEL),
            ("MriInterpretRequest", MriInterpretRequest::LOGICAL_LABEL),
            ("CtReportResult", CtReportResult::LOGICAL_LABEL),
            ("MriReportResult", MriReportResult::LOGICAL_LABEL),
            ("GeneralForCard", GeneralForCard::LOGICAL_LABEL),
            ("GeneralForNeuro", GeneralForNeuro::LOGICAL_LABEL),
            ("GeneralForCardOwner", GeneralForCardOwner::LOGICAL_LABEL),
            ("GeneralForNeuroOwner", GeneralForNeuroOwner::LOGICAL_LABEL),
            ("GeneralForConference", GeneralForConference::LOGICAL_LABEL),
            ("CardPanelForNeuro", CardPanelForNeuro::LOGICAL_LABEL),
            ("NeuroPanelForCard", NeuroPanelForCard::LOGICAL_LABEL),
            ("CtForCard", CtForCard::LOGICAL_LABEL),
            ("CtForNeuro", CtForNeuro::LOGICAL_LABEL),
            ("CtForCardOwner", CtForCardOwner::LOGICAL_LABEL),
            ("CtForNeuroOwner", CtForNeuroOwner::LOGICAL_LABEL),
            ("CtForER", CtForER::LOGICAL_LABEL),
            ("CtForConference", CtForConference::LOGICAL_LABEL),
            ("MriForCard", MriForCard::LOGICAL_LABEL),
            ("MriForNeuro", MriForNeuro::LOGICAL_LABEL),
            ("MriForCardOwner", MriForCardOwner::LOGICAL_LABEL),
            ("MriForNeuroOwner", MriForNeuroOwner::LOGICAL_LABEL),
            ("MriForER", MriForER::LOGICAL_LABEL),
            ("MriForConference", MriForConference::LOGICAL_LABEL),
            ("CardDiagnosisRequest", CardDiagnosisRequest::LOGICAL_LABEL),
            ("CardDiagnosisResult", CardDiagnosisResult::LOGICAL_LABEL),
            (
                "NeuroDiagnosisRequest",
                NeuroDiagnosisRequest::LOGICAL_LABEL,
            ),
            ("NeuroDiagnosisResult", NeuroDiagnosisResult::LOGICAL_LABEL),
            ("CardDiagnosisForER", CardDiagnosisForER::LOGICAL_LABEL),
            ("NeuroDiagnosisForER", NeuroDiagnosisForER::LOGICAL_LABEL),
            (
                "CardDiagnosisForConference",
                CardDiagnosisForConference::LOGICAL_LABEL,
            ),
            (
                "NeuroDiagnosisForConference",
                NeuroDiagnosisForConference::LOGICAL_LABEL,
            ),
            ("AssessmentRequest", AssessmentRequest::LOGICAL_LABEL),
            ("AssessmentResult", AssessmentResult::LOGICAL_LABEL),
            ("ReviewRequest", ReviewRequest::LOGICAL_LABEL),
            ("ReviewResult", ReviewResult::LOGICAL_LABEL),
            ("ReviewForER", ReviewForER::LOGICAL_LABEL),
            ("ReviewForConference", ReviewForConference::LOGICAL_LABEL),
            ("ConferenceIntent", ConferenceIntent::LOGICAL_LABEL),
            ("RoomLoan", RoomLoan::LOGICAL_LABEL),
            ("RoomRequest", RoomRequest::LOGICAL_LABEL),
            ("RoomResult", RoomResult::LOGICAL_LABEL),
            ("FinalForER", FinalForER::LOGICAL_LABEL),
            ("FinalForCard", FinalForCard::LOGICAL_LABEL),
            ("FinalForNeuro", FinalForNeuro::LOGICAL_LABEL),
            ("FinalForRad", FinalForRad::LOGICAL_LABEL),
        ]
        .into_iter()
        .map(|(name, label)| (name.into(), label.into()))
        .collect();
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"5M","session":SESSION,
          "roles":{"ER_DOCTOR":ER_DOCTOR,"CARDIOLOGIST":CARDIOLOGIST,"NEUROLOGIST":NEUROLOGIST,"RADIOLOGIST":RADIOLOGIST,"PHARMACIST":PHARMACIST,"ER_IO":ER_IO,"CARDIO_IO":CARDIO_IO,"NEURO_IO":NEURO_IO,"RADIOLOGY_IO":RADIOLOGY_IO,"PHARMACY_IO":PHARMACY_IO,"GENERAL_SERVICE":GENERAL_SERVICE,"CARDIAC_SERVICE":CARDIAC_SERVICE,"NEURO_SERVICE":NEURO_SERVICE,"CT_SERVICE":CT_SERVICE,"MRI_SERVICE":MRI_SERVICE,"PHARMACY_SERVICE":PHARMACY_SERVICE,"CONFERENCE_SERVICE":CONFERENCE_SERVICE,"GENERAL_PANEL":GENERAL_PANEL,"CARDIAC_PANEL":CARDIAC_PANEL,"NEURO_PANEL":NEURO_PANEL,"CT_SCANNER":CT_SCANNER,"MRI":MRI,"CONSULTATION_ROOM":CONSULTATION_ROOM},
          "labels":labels,"programs":[rows(&self.er),rows(&self.cardio),rows(&self.neuro),rows(&self.radiologist),rows(&self.pharmacist),rows(&self.er_io),rows(&self.cardio_io),rows(&self.neuro_io),rows(&self.radiology_io),rows(&self.pharmacy_io),rows(&self.general),rows(&self.cardiac),rows(&self.neurology),rows(&self.ct_service),rows(&self.mri_service),rows(&self.pharmacy),rows(&self.conference),rows(&self.general_panel),rows(&self.cardiac_panel),rows(&self.neuro_panel),rows(&self.ct),rows(&self.mri),rows(&self.room)]})
    }
}
pub fn programs() -> Programs {
    let general_attempt = || {
        g::seq(
            g::send::<GENERAL_SERVICE, ER_DOCTOR, PanelLoan<GENERAL_PANEL>>(),
            g::seq(
                g::send::<GENERAL_PANEL, GENERAL_SERVICE, TestRequest>(),
                g::seq(
                    g::send::<GENERAL_SERVICE, GENERAL_PANEL, TestResult>(),
                    g::send::<ER_DOCTOR, GENERAL_SERVICE, PanelReturn<GENERAL_PANEL>>(),
                ),
            ),
        )
    };
    let general_complete = || {
        g::seq(
            g::send::<GENERAL_SERVICE, ER_DOCTOR, PanelCompleted<GENERAL_PANEL>>(),
            g::send::<GENERAL_SERVICE, ER_DOCTOR, PanelComplete>(),
        )
    };
    let general_panel = g::seq(
        general_attempt(),
        g::route(
            general_complete(),
            g::seq(
                g::seq(
                    g::send::<GENERAL_SERVICE, ER_DOCTOR, PanelPartial>(),
                    general_attempt(),
                )
                .roll(),
                general_complete(),
            ),
        )
        .resolve::<GENERAL_COMPLETE>(),
    );
    let cardiac_attempt = || {
        g::seq(
            g::send::<CARDIAC_SERVICE, CARDIOLOGIST, PanelLoan<CARDIAC_PANEL>>(),
            g::seq(
                g::send::<CARDIAC_PANEL, CARDIAC_SERVICE, TestRequest>(),
                g::seq(
                    g::send::<CARDIAC_SERVICE, CARDIAC_PANEL, TestResult>(),
                    g::send::<CARDIOLOGIST, CARDIAC_SERVICE, PanelReturn<CARDIAC_PANEL>>(),
                ),
            ),
        )
    };
    let cardiac_complete = || {
        g::seq(
            g::send::<CARDIAC_SERVICE, ER_DOCTOR, PanelCompleted<CARDIAC_PANEL>>(),
            g::send::<CARDIAC_SERVICE, CARDIOLOGIST, PanelComplete>(),
        )
    };
    let cardiac_panel = g::seq(
        cardiac_attempt(),
        g::route(
            cardiac_complete(),
            g::seq(
                g::seq(
                    g::send::<CARDIAC_SERVICE, CARDIOLOGIST, PanelPartial>(),
                    cardiac_attempt(),
                )
                .roll(),
                cardiac_complete(),
            ),
        )
        .resolve::<CARDIAC_COMPLETE>(),
    );
    let neuro_attempt = || {
        g::seq(
            g::send::<NEURO_SERVICE, NEUROLOGIST, PanelLoan<NEURO_PANEL>>(),
            g::seq(
                g::send::<NEURO_PANEL, NEURO_SERVICE, TestRequest>(),
                g::seq(
                    g::send::<NEURO_SERVICE, NEURO_PANEL, TestResult>(),
                    g::send::<NEUROLOGIST, NEURO_SERVICE, PanelReturn<NEURO_PANEL>>(),
                ),
            ),
        )
    };
    let neuro_complete = || {
        g::seq(
            g::send::<NEURO_SERVICE, ER_DOCTOR, PanelCompleted<NEURO_PANEL>>(),
            g::send::<NEURO_SERVICE, NEUROLOGIST, PanelComplete>(),
        )
    };
    let neuro_panel = g::seq(
        neuro_attempt(),
        g::route(
            neuro_complete(),
            g::seq(
                g::seq(
                    g::send::<NEURO_SERVICE, NEUROLOGIST, PanelPartial>(),
                    neuro_attempt(),
                )
                .roll(),
                neuro_complete(),
            ),
        )
        .resolve::<NEURO_COMPLETE>(),
    );
    let assessment = g::seq(
        g::send::<ER_DOCTOR, GENERAL_SERVICE, AssessmentRequest>(),
        g::send::<GENERAL_SERVICE, ER_DOCTOR, AssessmentResult>(),
    );
    let ct = g::seq(
        g::send::<RADIOLOGIST, CT_SERVICE, CtPerformIntent>(),
        g::seq(
            g::send::<CT_SERVICE, RADIOLOGIST, CtLoan>(),
            g::seq(
                g::send::<CT_SCANNER, CT_SERVICE, CtPerformRequest>(),
                g::seq(
                    g::send::<CT_SERVICE, CT_SCANNER, CtImageResult>(),
                    g::seq(
                        g::send::<RADIOLOGIST, CT_SERVICE, CtLoan>(),
                        g::seq(
                            g::send::<RADIOLOGIST, CT_SERVICE, CtInterpretIntent>(),
                            g::seq(
                                g::send::<CT_SERVICE, RADIOLOGIST, CtLoan>(),
                                g::seq(
                                    g::send::<CT_SCANNER, CT_SERVICE, CtInterpretRequest>(),
                                    g::seq(
                                        g::send::<CT_SERVICE, CT_SCANNER, CtReportResult>(),
                                        g::send::<RADIOLOGIST, CT_SERVICE, CtLoan>(),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );
    let mri = g::seq(
        g::send::<RADIOLOGIST, MRI_SERVICE, MriPerformIntent>(),
        g::seq(
            g::send::<MRI_SERVICE, RADIOLOGIST, MriLoan>(),
            g::seq(
                g::send::<MRI, MRI_SERVICE, MriPerformRequest>(),
                g::seq(
                    g::send::<MRI_SERVICE, MRI, MriImageResult>(),
                    g::seq(
                        g::send::<RADIOLOGIST, MRI_SERVICE, MriLoan>(),
                        g::seq(
                            g::send::<RADIOLOGIST, MRI_SERVICE, MriInterpretIntent>(),
                            g::seq(
                                g::send::<MRI_SERVICE, RADIOLOGIST, MriLoan>(),
                                g::seq(
                                    g::send::<MRI, MRI_SERVICE, MriInterpretRequest>(),
                                    g::seq(
                                        g::send::<MRI_SERVICE, MRI, MriReportResult>(),
                                        g::send::<RADIOLOGIST, MRI_SERVICE, MriLoan>(),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );
    let workup = g::par(
        general_panel,
        g::par(
            g::seq(
                g::seq(
                    g::send::<ER_DOCTOR, CARDIOLOGIST, CardReferral>(),
                    g::par(
                        g::send::<CARDIOLOGIST, CARDIAC_SERVICE, ReferralForOwner>(),
                        g::par(
                            g::send::<CARDIOLOGIST, RADIOLOGIST, CtOrder>(),
                            g::send::<CARDIOLOGIST, CT_SERVICE, CtOrderForOwner>(),
                        ),
                    ),
                ),
                g::par(cardiac_panel, ct),
            ),
            g::seq(
                g::seq(
                    g::send::<ER_DOCTOR, NEUROLOGIST, NeuroReferral>(),
                    g::par(
                        g::send::<NEUROLOGIST, NEURO_SERVICE, ReferralForOwner>(),
                        g::par(
                            g::send::<NEUROLOGIST, RADIOLOGIST, MriOrder>(),
                            g::send::<NEUROLOGIST, MRI_SERVICE, MriOrderForOwner>(),
                        ),
                    ),
                ),
                g::par(neuro_panel, mri),
            ),
        ),
    );
    // The joint case uses every completed investigation as shared input.
    // The common scorer independently permits its other data-valid orders.
    let findings = g::par(
        g::send::<GENERAL_SERVICE, CARDIOLOGIST, GeneralForCard>(),
        g::par(
            g::send::<GENERAL_SERVICE, NEUROLOGIST, GeneralForNeuro>(),
            g::par(
                g::send::<GENERAL_SERVICE, CARDIAC_SERVICE, GeneralForCardOwner>(),
                g::par(
                    g::send::<GENERAL_SERVICE, NEURO_SERVICE, GeneralForNeuroOwner>(),
                    g::par(
                        g::send::<GENERAL_SERVICE, CONFERENCE_SERVICE, GeneralForConference>(),
                        g::par(
                            g::send::<CARDIAC_SERVICE, NEUROLOGIST, CardPanelForNeuro>(),
                            g::par(
                                g::send::<NEURO_SERVICE, CARDIOLOGIST, NeuroPanelForCard>(),
                                g::par(
                                    g::send::<CT_SERVICE, CARDIOLOGIST, CtForCard>(),
                                    g::par(
                                        g::send::<CT_SERVICE, NEUROLOGIST, CtForNeuro>(),
                                        g::par(
                                            g::send::<CT_SERVICE, CARDIAC_SERVICE, CtForCardOwner>(
                                            ),
                                            g::par(
                                                g::send::<CT_SERVICE, NEURO_SERVICE, CtForNeuroOwner>(
                                                ),
                                                g::par(
                                                    g::send::<CT_SERVICE, ER_DOCTOR, CtForER>(),
                                                    g::par(
                                                        g::send::<
                                                            CT_SERVICE,
                                                            CONFERENCE_SERVICE,
                                                            CtForConference,
                                                        >(
                                                        ),
                                                        g::par(
                                                            g::send::<
                                                                MRI_SERVICE,
                                                                CARDIOLOGIST,
                                                                MriForCard,
                                                            >(
                                                            ),
                                                            g::par(
                                                                g::send::<
                                                                    MRI_SERVICE,
                                                                    NEUROLOGIST,
                                                                    MriForNeuro,
                                                                >(
                                                                ),
                                                                g::par(
                                                                    g::send::<
                                                                        MRI_SERVICE,
                                                                        CARDIAC_SERVICE,
                                                                        MriForCardOwner,
                                                                    >(
                                                                    ),
                                                                    g::par(
                                                                        g::send::<
                                                                            MRI_SERVICE,
                                                                            NEURO_SERVICE,
                                                                            MriForNeuroOwner,
                                                                        >(
                                                                        ),
                                                                        g::par(
                                                                            g::send::<
                                                                                MRI_SERVICE,
                                                                                ER_DOCTOR,
                                                                                MriForER,
                                                                            >(
                                                                            ),
                                                                            g::send::<
                                                                                MRI_SERVICE,
                                                                                CONFERENCE_SERVICE,
                                                                                MriForConference,
                                                                            >(
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
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );
    let diagnoses = g::par(
        g::seq(
            g::send::<CARDIOLOGIST, CARDIAC_SERVICE, CardDiagnosisRequest>(),
            g::seq(
                g::send::<CARDIAC_SERVICE, CARDIOLOGIST, CardDiagnosisResult>(),
                g::par(
                    g::send::<CARDIAC_SERVICE, ER_DOCTOR, CardDiagnosisForER>(),
                    g::send::<CARDIAC_SERVICE, CONFERENCE_SERVICE, CardDiagnosisForConference>(),
                ),
            ),
        ),
        g::seq(
            g::send::<NEUROLOGIST, NEURO_SERVICE, NeuroDiagnosisRequest>(),
            g::seq(
                g::send::<NEURO_SERVICE, NEUROLOGIST, NeuroDiagnosisResult>(),
                g::par(
                    g::send::<NEURO_SERVICE, ER_DOCTOR, NeuroDiagnosisForER>(),
                    g::send::<NEURO_SERVICE, CONFERENCE_SERVICE, NeuroDiagnosisForConference>(),
                ),
            ),
        ),
    );
    let clinical = g::seq(assessment, g::seq(workup, g::seq(findings, diagnoses)));
    let review = g::seq(
        g::send::<PHARMACIST, PHARMACY_SERVICE, ReviewRequest>(),
        g::seq(
            g::send::<PHARMACY_SERVICE, PHARMACIST, ReviewResult>(),
            g::par(
                g::send::<PHARMACY_SERVICE, ER_DOCTOR, ReviewForER>(),
                g::send::<PHARMACY_SERVICE, CONFERENCE_SERVICE, ReviewForConference>(),
            ),
        ),
    );
    let conference = g::seq(
        g::send::<ER_DOCTOR, CONFERENCE_SERVICE, ConferenceIntent>(),
        g::seq(
            g::send::<CONFERENCE_SERVICE, ER_DOCTOR, RoomLoan>(),
            g::seq(
                g::send::<CONSULTATION_ROOM, CONFERENCE_SERVICE, RoomRequest>(),
                g::seq(
                    g::send::<CONFERENCE_SERVICE, CONSULTATION_ROOM, RoomResult>(),
                    g::seq(
                        g::send::<ER_DOCTOR, CONFERENCE_SERVICE, RoomLoan>(),
                        g::par(
                            g::send::<CONFERENCE_SERVICE, ER_DOCTOR, FinalForER>(),
                            g::par(
                                g::send::<CONFERENCE_SERVICE, CARDIOLOGIST, FinalForCard>(),
                                g::par(
                                    g::send::<CONFERENCE_SERVICE, NEUROLOGIST, FinalForNeuro>(),
                                    g::send::<CONFERENCE_SERVICE, RADIOLOGIST, FinalForRad>(),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );
    let business = g::seq(g::par(clinical, review), conference);
    let global = g::par(
        business,
        g::par(
            crate::shell::global::private_io::<ER_DOCTOR, ER_IO>(),
            g::par(
                crate::shell::global::private_io::<CARDIOLOGIST, CARDIO_IO>(),
                g::par(
                    crate::shell::global::private_io::<NEUROLOGIST, NEURO_IO>(),
                    g::par(
                        crate::shell::global::private_io::<RADIOLOGIST, RADIOLOGY_IO>(),
                        crate::shell::global::private_io::<PHARMACIST, PHARMACY_IO>(),
                    ),
                ),
            ),
        ),
    );
    Programs {
        er: project::<ER_DOCTOR>(&global),
        cardio: project::<CARDIOLOGIST>(&global),
        neuro: project::<NEUROLOGIST>(&global),
        radiologist: project::<RADIOLOGIST>(&global),
        pharmacist: project::<PHARMACIST>(&global),
        er_io: project::<ER_IO>(&global),
        cardio_io: project::<CARDIO_IO>(&global),
        neuro_io: project::<NEURO_IO>(&global),
        radiology_io: project::<RADIOLOGY_IO>(&global),
        pharmacy_io: project::<PHARMACY_IO>(&global),
        general: project::<GENERAL_SERVICE>(&global),
        cardiac: project::<CARDIAC_SERVICE>(&global),
        neurology: project::<NEURO_SERVICE>(&global),
        ct_service: project::<CT_SERVICE>(&global),
        mri_service: project::<MRI_SERVICE>(&global),
        pharmacy: project::<PHARMACY_SERVICE>(&global),
        conference: project::<CONFERENCE_SERVICE>(&global),
        general_panel: project::<GENERAL_PANEL>(&global),
        cardiac_panel: project::<CARDIAC_PANEL>(&global),
        neuro_panel: project::<NEURO_PANEL>(&global),
        ct: project::<CT_SCANNER>(&global),
        mri: project::<MRI>(&global),
        room: project::<CONSULTATION_ROOM>(&global),
    }
}
