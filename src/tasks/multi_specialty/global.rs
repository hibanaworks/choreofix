//! Direct operation-file Endpoints, independent imaging and actual case records.
use super::imp::Arguments;
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
pub const ER_VM: u8 = 64;
pub const CARDIO_VM: u8 = 65;
pub const NEURO_VM: u8 = 66;
pub const RADIOLOGY_VM: u8 = 67;
pub const PHARMACY_VM: u8 = 68;
pub const ER_FILE: u8 = 32;
pub const ER_DATA: u8 = 96;
pub const CARDIO_FILE: u8 = 33;
pub const CARDIO_DATA: u8 = 97;
pub const NEURO_FILE: u8 = 34;
pub const NEURO_DATA: u8 = 98;
pub const RADIOLOGY_FILE: u8 = 35;
pub const RADIOLOGY_DATA: u8 = 99;
pub const PHARMACY_FILE: u8 = 36;
pub const PHARMACY_DATA: u8 = 100;
pub const GENERAL_COMPLETE: u16 = 601;
pub const CARDIAC_COMPLETE: u16 = 602;
pub const NEURO_COMPLETE: u16 = 603;
pub const SESSION: u32 = 51;
pub type PanelLoan<'r, const RESOURCE: u8> = g::Msg<1, Endpoint<'r, RESOURCE>>;
pub type PanelReturn<'r, const RESOURCE: u8> = g::Msg<2, Endpoint<'r, RESOURCE>>;
pub type PanelCompleted<'r, const RESOURCE: u8> = g::Msg<3, Endpoint<'r, RESOURCE>>;
pub type TestRequest<const OPERATION: u8> = g::Msg<4, Arguments<OPERATION>>;
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
pub type CtPerformIntent = g::Msg<14, Arguments<4>>;
pub type MriPerformIntent = g::Msg<15, Arguments<6>>;
pub type CtLoan<'r> = g::Msg<16, Endpoint<'r, CT_SCANNER>>;
pub type MriLoan<'r> = g::Msg<17, Endpoint<'r, MRI>>;
pub type CtPerformRequest = g::Msg<18, Arguments<4>>;
pub type MriPerformRequest = g::Msg<19, Arguments<6>>;
pub type CtImageResult<'a> = g::Msg<20, &'a [u8]>;
pub type MriImageResult<'a> = g::Msg<21, &'a [u8]>;
pub type CtInterpretIntent = g::Msg<22, Arguments<5>>;
pub type MriInterpretIntent = g::Msg<23, Arguments<7>>;
pub type CtInterpretRequest = g::Msg<24, Arguments<5>>;
pub type MriInterpretRequest = g::Msg<25, Arguments<7>>;
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
pub type CardDiagnosisRequest = g::Msg<47, Arguments<8>>;
pub type CardDiagnosisResult<'a> = g::Msg<48, &'a [u8]>;
pub type NeuroDiagnosisRequest = g::Msg<49, Arguments<9>>;
pub type NeuroDiagnosisResult<'a> = g::Msg<50, &'a [u8]>;
pub type CardDiagnosisForER<'a> = g::Msg<51, &'a [u8]>;
pub type NeuroDiagnosisForER<'a> = g::Msg<52, &'a [u8]>;
pub type CardDiagnosisForConference<'a> = g::Msg<53, &'a [u8]>;
pub type NeuroDiagnosisForConference<'a> = g::Msg<54, &'a [u8]>;
pub type AssessmentRequest = g::Msg<55, Arguments<0>>;
pub type AssessmentResult<'a> = g::Msg<56, &'a [u8]>;
pub type ReviewRequest = g::Msg<57, Arguments<10>>;
pub type ReviewResult<'a> = g::Msg<58, &'a [u8]>;
pub type ReviewForER<'a> = g::Msg<59, &'a [u8]>;
pub type ReviewForConference<'a> = g::Msg<60, &'a [u8]>;
pub type ConferenceIntent = g::Msg<61, Arguments<11>>;
pub type RoomLoan<'r> = g::Msg<62, Endpoint<'r, CONSULTATION_ROOM>>;
pub type RoomRequest = g::Msg<63, Arguments<12>>;
pub type RoomResult<'a> = g::Msg<64, &'a [u8]>;
pub type FinalForER<'a> = g::Msg<65, &'a [u8]>;
pub type FinalForCard<'a> = g::Msg<66, &'a [u8]>;
pub type FinalForNeuro<'a> = g::Msg<67, &'a [u8]>;
pub type FinalForRad<'a> = g::Msg<68, &'a [u8]>;
pub type GeneralFileRequest = g::Msg<70, Arguments<1>>;
pub type GeneralAdmitted = g::Msg<71, Arguments<1>>;
pub type FileGeneralResult<'a> = g::Msg<72, &'a [u8]>;
pub type CardiacFileRequest = g::Msg<73, Arguments<2>>;
pub type CardiacAdmitted = g::Msg<74, Arguments<2>>;
pub type FileCardiacResult<'a> = g::Msg<75, &'a [u8]>;
pub type NeuroFileRequest = g::Msg<76, Arguments<3>>;
pub type NeuroAdmitted = g::Msg<77, Arguments<3>>;
pub type FileNeuroResult<'a> = g::Msg<78, &'a [u8]>;
pub type GeneralCompletedData<'a> = g::Msg<79, &'a [u8]>;
pub type CardiacCompletedData<'a> = g::Msg<80, &'a [u8]>;
pub type NeuroCompletedData<'a> = g::Msg<81, &'a [u8]>;
pub type FileAssessmentResult<'a> = g::Msg<82, &'a [u8]>;
pub type CtPerformAdmitted = g::Msg<83, Arguments<4>>;
pub type CtInterpretAdmitted = g::Msg<84, Arguments<5>>;
pub type FileCtImageResult<'a> = g::Msg<85, &'a [u8]>;
pub type FileCtReportResult<'a> = g::Msg<86, &'a [u8]>;
pub type MriPerformAdmitted = g::Msg<87, Arguments<6>>;
pub type MriInterpretAdmitted = g::Msg<88, Arguments<7>>;
pub type FileMriImageResult<'a> = g::Msg<89, &'a [u8]>;
pub type FileMriReportResult<'a> = g::Msg<90, &'a [u8]>;
pub type FileCardDiagnosisResult<'a> = g::Msg<91, &'a [u8]>;
pub type FileNeuroDiagnosisResult<'a> = g::Msg<92, &'a [u8]>;
pub type FileReviewResult<'a> = g::Msg<93, &'a [u8]>;
pub type FileConferenceResult<'a> = g::Msg<94, &'a [u8]>;
pub type ConferenceAdmitted = g::Msg<95, Arguments<11>>;
pub type FileCardReferral<'a> = g::Msg<96, &'a [u8]>;
pub type FileNeuroReferral<'a> = g::Msg<97, &'a [u8]>;
pub type FileCtOrder<'a> = g::Msg<98, &'a [u8]>;
pub type FileMriOrder<'a> = g::Msg<99, &'a [u8]>;
pub type FileCardFindings<'a> = g::Msg<100, &'a [u8]>;
pub type FileNeuroFindings<'a> = g::Msg<101, &'a [u8]>;
pub type Candidate<'a> = g::Msg<102, &'a [u8]>;

const fn operation(path: &'static [u8], fd: u8) -> ChoreoFsObject<'static> {
    ChoreoFsObject::read_write(
        path,
        ObjectId(fd as u32),
        FdSpec::new(fd, 66, 1),
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
        FdSpec::new(fd, 2, 1),
        b"",
        protocol::FdBinding {
            read: Some(protocol::FdReadRow::Base),
            write: None,
            readdir: None,
        },
    )
}
pub const ER_FILES: [ChoreoFsObject<'static>; 5] = [
    operation(b"service/assess_patient", 4),
    operation(b"service/order_tests", 5),
    result(b"service/candidate", 6),
    operation(b"service/conduct_case_conference", 7),
    result(b"service/summary", 8),
];
pub const CARDIO_FILES: [ChoreoFsObject<'static>; 5] = [
    result(b"service/referral", 4),
    operation(b"service/order_tests", 5),
    result(b"service/findings", 6),
    operation(b"service/diagnose", 7),
    result(b"service/summary", 8),
];
pub const NEURO_FILES: [ChoreoFsObject<'static>; 5] = [
    result(b"service/referral", 4),
    operation(b"service/order_tests", 5),
    result(b"service/findings", 6),
    operation(b"service/diagnose", 7),
    result(b"service/summary", 8),
];
pub const RADIOLOGY_FILES: [ChoreoFsObject<'static>; 7] = [
    result(b"service/ct_order", 4),
    operation(b"service/perform_imaging_ct", 5),
    operation(b"service/interpret_imaging_ct", 6),
    result(b"service/mri_order", 7),
    operation(b"service/perform_imaging_mri", 8),
    operation(b"service/interpret_imaging_mri", 9),
    result(b"service/summary", 10),
];
pub const PHARMACY_FILES: [ChoreoFsObject<'static>; 1] =
    [operation(b"service/review_prescriptions", 4)];

pub fn source() -> impl Projectable {
    let general_attempt = || {
        g::seq(
            g::send::<GENERAL_SERVICE, ER_DOCTOR, PanelLoan<GENERAL_PANEL>>(),
            g::seq(
                g::send::<ER_FILE, GENERAL_SERVICE, GeneralFileRequest>(),
                g::seq(
                    g::send::<GENERAL_SERVICE, ER_DOCTOR, GeneralAdmitted>(),
                    g::seq(
                        g::send::<GENERAL_PANEL, GENERAL_SERVICE, TestRequest<1>>(),
                        g::seq(
                            g::send::<GENERAL_SERVICE, GENERAL_PANEL, TestResult>(),
                            g::send::<ER_DOCTOR, GENERAL_SERVICE, PanelReturn<GENERAL_PANEL>>(),
                        ),
                    ),
                ),
            ),
        )
    };
    let general_ready = || {
        g::seq(
            g::send::<GENERAL_SERVICE, CONFERENCE_SERVICE, GeneralCompletedData>(),
            g::seq(
                g::send::<GENERAL_SERVICE, ER_DOCTOR, PanelComplete>(),
                g::seq(
                    g::send::<GENERAL_SERVICE, CONFERENCE_SERVICE, PanelCompleted<GENERAL_PANEL>>(),
                    g::send::<CONFERENCE_SERVICE, ER_FILE, FileGeneralResult>(),
                ),
            ),
        )
    };
    let general_panel = g::seq(
        general_attempt(),
        g::route(
            general_ready(),
            g::seq(
                g::seq(
                    g::send::<GENERAL_SERVICE, ER_FILE, FileGeneralResult>(),
                    g::seq(
                        g::send::<GENERAL_SERVICE, ER_DOCTOR, PanelPartial>(),
                        general_attempt(),
                    ),
                )
                .roll(),
                general_ready(),
            ),
        )
        .resolve::<GENERAL_COMPLETE>(),
    );
    let cardiac_attempt = || {
        g::seq(
            g::send::<CARDIAC_SERVICE, CARDIOLOGIST, PanelLoan<CARDIAC_PANEL>>(),
            g::seq(
                g::send::<CARDIO_FILE, CARDIAC_SERVICE, CardiacFileRequest>(),
                g::seq(
                    g::send::<CARDIAC_SERVICE, CARDIOLOGIST, CardiacAdmitted>(),
                    g::seq(
                        g::send::<CARDIAC_PANEL, CARDIAC_SERVICE, TestRequest<2>>(),
                        g::seq(
                            g::send::<CARDIAC_SERVICE, CARDIAC_PANEL, TestResult>(),
                            g::send::<CARDIOLOGIST, CARDIAC_SERVICE, PanelReturn<CARDIAC_PANEL>>(),
                        ),
                    ),
                ),
            ),
        )
    };
    let cardiac_ready = || {
        g::seq(
            g::send::<CARDIAC_SERVICE, CONFERENCE_SERVICE, CardiacCompletedData>(),
            g::seq(
                g::send::<CARDIAC_SERVICE, CARDIOLOGIST, PanelComplete>(),
                g::seq(
                    g::send::<CARDIAC_SERVICE, CONFERENCE_SERVICE, PanelCompleted<CARDIAC_PANEL>>(),
                    g::send::<CONFERENCE_SERVICE, CARDIO_FILE, FileCardiacResult>(),
                ),
            ),
        )
    };
    let cardiac_panel = g::seq(
        cardiac_attempt(),
        g::route(
            cardiac_ready(),
            g::seq(
                g::seq(
                    g::send::<CARDIAC_SERVICE, CARDIO_FILE, FileCardiacResult>(),
                    g::seq(
                        g::send::<CARDIAC_SERVICE, CARDIOLOGIST, PanelPartial>(),
                        cardiac_attempt(),
                    ),
                )
                .roll(),
                cardiac_ready(),
            ),
        )
        .resolve::<CARDIAC_COMPLETE>(),
    );
    let neuro_attempt = || {
        g::seq(
            g::send::<NEURO_SERVICE, NEUROLOGIST, PanelLoan<NEURO_PANEL>>(),
            g::seq(
                g::send::<NEURO_FILE, NEURO_SERVICE, NeuroFileRequest>(),
                g::seq(
                    g::send::<NEURO_SERVICE, NEUROLOGIST, NeuroAdmitted>(),
                    g::seq(
                        g::send::<NEURO_PANEL, NEURO_SERVICE, TestRequest<3>>(),
                        g::seq(
                            g::send::<NEURO_SERVICE, NEURO_PANEL, TestResult>(),
                            g::send::<NEUROLOGIST, NEURO_SERVICE, PanelReturn<NEURO_PANEL>>(),
                        ),
                    ),
                ),
            ),
        )
    };
    let neuro_ready = || {
        g::seq(
            g::send::<NEURO_SERVICE, CONFERENCE_SERVICE, NeuroCompletedData>(),
            g::seq(
                g::send::<NEURO_SERVICE, NEUROLOGIST, PanelComplete>(),
                g::seq(
                    g::send::<NEURO_SERVICE, CONFERENCE_SERVICE, PanelCompleted<NEURO_PANEL>>(),
                    g::send::<CONFERENCE_SERVICE, NEURO_FILE, FileNeuroResult>(),
                ),
            ),
        )
    };
    let neuro_panel = g::seq(
        neuro_attempt(),
        g::route(
            neuro_ready(),
            g::seq(
                g::seq(
                    g::send::<NEURO_SERVICE, NEURO_FILE, FileNeuroResult>(),
                    g::seq(
                        g::send::<NEURO_SERVICE, NEUROLOGIST, PanelPartial>(),
                        neuro_attempt(),
                    ),
                )
                .roll(),
                neuro_ready(),
            ),
        )
        .resolve::<NEURO_COMPLETE>(),
    );
    let assessment = g::seq(
        g::send::<ER_FILE, GENERAL_SERVICE, AssessmentRequest>(),
        g::par(
            g::send::<GENERAL_SERVICE, ER_DOCTOR, AssessmentResult>(),
            g::send::<GENERAL_SERVICE, ER_FILE, FileAssessmentResult>(),
        ),
    );
    let ct = g::seq(
        g::send::<RADIOLOGY_FILE, CT_SERVICE, CtPerformIntent>(),
        g::seq(
            g::send::<CT_SERVICE, RADIOLOGIST, CtPerformAdmitted>(),
            g::seq(
                g::send::<CT_SERVICE, RADIOLOGIST, CtLoan>(),
                g::seq(
                    g::send::<CT_SCANNER, CT_SERVICE, CtPerformRequest>(),
                    g::seq(
                        g::par(
                            g::send::<CT_SERVICE, CT_SCANNER, CtImageResult>(),
                            g::send::<CT_SERVICE, RADIOLOGY_FILE, FileCtImageResult>(),
                        ),
                        g::seq(
                            g::send::<RADIOLOGIST, CT_SERVICE, CtLoan>(),
                            g::seq(
                                g::send::<RADIOLOGY_FILE, CT_SERVICE, CtInterpretIntent>(),
                                g::seq(
                                    g::send::<CT_SERVICE, RADIOLOGIST, CtInterpretAdmitted>(),
                                    g::seq(
                                        g::send::<CT_SERVICE, RADIOLOGIST, CtLoan>(),
                                        g::seq(
                                            g::send::<CT_SCANNER, CT_SERVICE, CtInterpretRequest>(),
                                            g::seq(
                                                g::par(
                                                    g::send::<CT_SERVICE, CT_SCANNER, CtReportResult>(
                                                    ),
                                                    g::send::<
                                                        CT_SERVICE,
                                                        RADIOLOGY_FILE,
                                                        FileCtReportResult,
                                                    >(
                                                    ),
                                                ),
                                                g::send::<RADIOLOGIST, CT_SERVICE, CtLoan>(),
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
    let mri = g::seq(
        g::send::<RADIOLOGY_FILE, MRI_SERVICE, MriPerformIntent>(),
        g::seq(
            g::send::<MRI_SERVICE, RADIOLOGIST, MriPerformAdmitted>(),
            g::seq(
                g::send::<MRI_SERVICE, RADIOLOGIST, MriLoan>(),
                g::seq(
                    g::send::<MRI, MRI_SERVICE, MriPerformRequest>(),
                    g::seq(
                        g::par(
                            g::send::<MRI_SERVICE, MRI, MriImageResult>(),
                            g::send::<MRI_SERVICE, RADIOLOGY_FILE, FileMriImageResult>(),
                        ),
                        g::seq(
                            g::send::<RADIOLOGIST, MRI_SERVICE, MriLoan>(),
                            g::seq(
                                g::send::<RADIOLOGY_FILE, MRI_SERVICE, MriInterpretIntent>(),
                                g::seq(
                                    g::send::<MRI_SERVICE, RADIOLOGIST, MriInterpretAdmitted>(),
                                    g::seq(
                                        g::send::<MRI_SERVICE, RADIOLOGIST, MriLoan>(),
                                        g::seq(
                                            g::send::<MRI, MRI_SERVICE, MriInterpretRequest>(),
                                            g::seq(
                                                g::par(
                                                    g::send::<MRI_SERVICE, MRI, MriReportResult>(),
                                                    g::send::<
                                                        MRI_SERVICE,
                                                        RADIOLOGY_FILE,
                                                        FileMriReportResult,
                                                    >(
                                                    ),
                                                ),
                                                g::send::<RADIOLOGIST, MRI_SERVICE, MriLoan>(),
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
                            g::par(
                                g::send::<CARDIOLOGIST, CT_SERVICE, CtOrderForOwner>(),
                                g::par(
                                    g::send::<CARDIOLOGIST, RADIOLOGY_FILE, FileCtOrder>(),
                                    g::send::<CARDIOLOGIST, CARDIO_FILE, FileCardReferral>(),
                                ),
                            ),
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
                            g::par(
                                g::send::<NEUROLOGIST, MRI_SERVICE, MriOrderForOwner>(),
                                g::par(
                                    g::send::<NEUROLOGIST, RADIOLOGY_FILE, FileMriOrder>(),
                                    g::send::<NEUROLOGIST, NEURO_FILE, FileNeuroReferral>(),
                                ),
                            ),
                        ),
                    ),
                ),
                g::par(neuro_panel, mri),
            ),
        ),
    );
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
    let readable_findings = g::par(
        g::send::<CARDIOLOGIST, CARDIO_FILE, FileCardFindings>(),
        g::send::<NEUROLOGIST, NEURO_FILE, FileNeuroFindings>(),
    );
    let diagnoses = g::par(
        g::seq(
            g::send::<CARDIO_FILE, CARDIAC_SERVICE, CardDiagnosisRequest>(),
            g::seq(
                g::send::<CARDIAC_SERVICE, CARDIOLOGIST, CardDiagnosisResult>(),
                g::par(
                    g::send::<CARDIAC_SERVICE, CARDIO_FILE, FileCardDiagnosisResult>(),
                    g::par(
                        g::send::<CARDIAC_SERVICE, ER_DOCTOR, CardDiagnosisForER>(),
                        g::send::<CARDIAC_SERVICE, CONFERENCE_SERVICE, CardDiagnosisForConference>(
                        ),
                    ),
                ),
            ),
        ),
        g::seq(
            g::send::<NEURO_FILE, NEURO_SERVICE, NeuroDiagnosisRequest>(),
            g::seq(
                g::send::<NEURO_SERVICE, NEUROLOGIST, NeuroDiagnosisResult>(),
                g::par(
                    g::send::<NEURO_SERVICE, NEURO_FILE, FileNeuroDiagnosisResult>(),
                    g::par(
                        g::send::<NEURO_SERVICE, ER_DOCTOR, NeuroDiagnosisForER>(),
                        g::send::<NEURO_SERVICE, CONFERENCE_SERVICE, NeuroDiagnosisForConference>(),
                    ),
                ),
            ),
        ),
    );
    let clinical = g::seq(
        assessment,
        g::seq(
            workup,
            g::seq(findings, g::seq(readable_findings, diagnoses)),
        ),
    );
    let review = g::seq(
        g::send::<PHARMACY_FILE, PHARMACY_SERVICE, ReviewRequest>(),
        g::par(
            g::send::<PHARMACY_SERVICE, PHARMACIST, ReviewResult>(),
            g::par(
                g::send::<PHARMACY_SERVICE, ER_DOCTOR, ReviewForER>(),
                g::par(
                    g::send::<PHARMACY_SERVICE, CONFERENCE_SERVICE, ReviewForConference>(),
                    g::send::<PHARMACY_SERVICE, PHARMACY_FILE, FileReviewResult>(),
                ),
            ),
        ),
    );
    let business = g::seq(
        g::par(clinical, review),
        g::seq(
            g::send::<ER_DOCTOR, ER_FILE, Candidate>(),
            g::seq(
                g::send::<ER_FILE, CONFERENCE_SERVICE, ConferenceIntent>(),
                g::seq(
                    g::send::<CONFERENCE_SERVICE, ER_DOCTOR, ConferenceAdmitted>(),
                    g::seq(
                        g::send::<CONFERENCE_SERVICE, ER_DOCTOR, RoomLoan>(),
                        g::seq(
                            g::send::<CONSULTATION_ROOM, CONFERENCE_SERVICE, RoomRequest>(),
                            g::seq(
                                g::send::<CONFERENCE_SERVICE, CONSULTATION_ROOM, RoomResult>(),
                                g::seq(
                                    g::send::<ER_DOCTOR, CONFERENCE_SERVICE, RoomLoan>(),
                                    g::par(
                                        g::send::<CONFERENCE_SERVICE, ER_FILE, FileConferenceResult>(
                                        ),
                                        g::par(
                                            g::send::<CONFERENCE_SERVICE, ER_FILE, FinalForER>(),
                                            g::par(
                                                g::send::<
                                                    CONFERENCE_SERVICE,
                                                    CARDIO_FILE,
                                                    FinalForCard,
                                                >(),
                                                g::par(
                                                    g::send::<
                                                        CONFERENCE_SERVICE,
                                                        NEURO_FILE,
                                                        FinalForNeuro,
                                                    >(
                                                    ),
                                                    g::send::<
                                                        CONFERENCE_SERVICE,
                                                        RADIOLOGY_FILE,
                                                        FinalForRad,
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
    );
    let bootstrap = g::par(
        g::send::<ER_DATA, ER_VM, Loan<ER_FILE>>(),
        g::par(
            g::send::<CARDIO_DATA, CARDIO_VM, Loan<CARDIO_FILE>>(),
            g::par(
                g::send::<NEURO_DATA, NEURO_VM, Loan<NEURO_FILE>>(),
                g::par(
                    g::send::<RADIOLOGY_DATA, RADIOLOGY_VM, Loan<RADIOLOGY_FILE>>(),
                    g::send::<PHARMACY_DATA, PHARMACY_VM, Loan<PHARMACY_FILE>>(),
                ),
            ),
        ),
    );
    let metadata = g::par(
        g::seq(
            crate::shell::global::file_metadata::<ER_FILE, ER_DATA>().roll(),
            g::send::<ER_FILE, ER_DATA, FileExit>(),
        ),
        g::par(
            g::seq(
                crate::shell::global::file_metadata::<CARDIO_FILE, CARDIO_DATA>().roll(),
                g::send::<CARDIO_FILE, CARDIO_DATA, FileExit>(),
            ),
            g::par(
                g::seq(
                    crate::shell::global::file_metadata::<NEURO_FILE, NEURO_DATA>().roll(),
                    g::send::<NEURO_FILE, NEURO_DATA, FileExit>(),
                ),
                g::par(
                    g::seq(
                        crate::shell::global::file_metadata::<RADIOLOGY_FILE, RADIOLOGY_DATA>()
                            .roll(),
                        g::send::<RADIOLOGY_FILE, RADIOLOGY_DATA, FileExit>(),
                    ),
                    g::seq(
                        crate::shell::global::file_metadata::<PHARMACY_FILE, PHARMACY_DATA>()
                            .roll(),
                        g::send::<PHARMACY_FILE, PHARMACY_DATA, FileExit>(),
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
            g::par(
                g::seq(
                    crate::shell::global::private_io::<NEURO_VM, NEURO_IO>(),
                    g::send::<NEURO_VM, NEURO_IO, crate::shell::global::Exit>(),
                ),
                g::par(
                    g::seq(
                        crate::shell::global::private_io::<RADIOLOGY_VM, RADIOLOGY_IO>(),
                        g::send::<RADIOLOGY_VM, RADIOLOGY_IO, crate::shell::global::Exit>(),
                    ),
                    g::seq(
                        crate::shell::global::private_io::<PHARMACY_VM, PHARMACY_IO>(),
                        g::send::<PHARMACY_VM, PHARMACY_IO, crate::shell::global::Exit>(),
                    ),
                ),
            ),
        ),
    );
    g::seq(bootstrap, g::par(business, g::par(metadata, private)))
}

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
    pub er_vm: RoleProgram<ER_VM>,
    pub cardio_vm: RoleProgram<CARDIO_VM>,
    pub neuro_vm: RoleProgram<NEURO_VM>,
    pub radiology_vm: RoleProgram<RADIOLOGY_VM>,
    pub pharmacy_vm: RoleProgram<PHARMACY_VM>,
    pub er_file: RoleProgram<ER_FILE>,
    pub er_data: RoleProgram<ER_DATA>,
    pub cardio_file: RoleProgram<CARDIO_FILE>,
    pub cardio_data: RoleProgram<CARDIO_DATA>,
    pub neuro_file: RoleProgram<NEURO_FILE>,
    pub neuro_data: RoleProgram<NEURO_DATA>,
    pub radiology_file: RoleProgram<RADIOLOGY_FILE>,
    pub radiology_data: RoleProgram<RADIOLOGY_DATA>,
    pub pharmacy_file: RoleProgram<PHARMACY_FILE>,
    pub pharmacy_data: RoleProgram<PHARMACY_DATA>,
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
            ("TestRequest", TestRequest::<1>::LOGICAL_LABEL),
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
            ("GeneralFileRequest", GeneralFileRequest::LOGICAL_LABEL),
            ("GeneralAdmitted", GeneralAdmitted::LOGICAL_LABEL),
            ("FileGeneralResult", FileGeneralResult::LOGICAL_LABEL),
            ("CardiacFileRequest", CardiacFileRequest::LOGICAL_LABEL),
            ("CardiacAdmitted", CardiacAdmitted::LOGICAL_LABEL),
            ("FileCardiacResult", FileCardiacResult::LOGICAL_LABEL),
            ("NeuroFileRequest", NeuroFileRequest::LOGICAL_LABEL),
            ("NeuroAdmitted", NeuroAdmitted::LOGICAL_LABEL),
            ("FileNeuroResult", FileNeuroResult::LOGICAL_LABEL),
            ("GeneralCompletedData", GeneralCompletedData::LOGICAL_LABEL),
            ("CardiacCompletedData", CardiacCompletedData::LOGICAL_LABEL),
            ("NeuroCompletedData", NeuroCompletedData::LOGICAL_LABEL),
            ("FileAssessmentResult", FileAssessmentResult::LOGICAL_LABEL),
            ("CtPerformAdmitted", CtPerformAdmitted::LOGICAL_LABEL),
            ("CtInterpretAdmitted", CtInterpretAdmitted::LOGICAL_LABEL),
            ("FileCtImageResult", FileCtImageResult::LOGICAL_LABEL),
            ("FileCtReportResult", FileCtReportResult::LOGICAL_LABEL),
            ("MriPerformAdmitted", MriPerformAdmitted::LOGICAL_LABEL),
            ("MriInterpretAdmitted", MriInterpretAdmitted::LOGICAL_LABEL),
            ("FileMriImageResult", FileMriImageResult::LOGICAL_LABEL),
            ("FileMriReportResult", FileMriReportResult::LOGICAL_LABEL),
            (
                "FileCardDiagnosisResult",
                FileCardDiagnosisResult::LOGICAL_LABEL,
            ),
            (
                "FileNeuroDiagnosisResult",
                FileNeuroDiagnosisResult::LOGICAL_LABEL,
            ),
            ("FileReviewResult", FileReviewResult::LOGICAL_LABEL),
            ("FileConferenceResult", FileConferenceResult::LOGICAL_LABEL),
            ("ConferenceAdmitted", ConferenceAdmitted::LOGICAL_LABEL),
            ("FileCardReferral", FileCardReferral::LOGICAL_LABEL),
            ("FileNeuroReferral", FileNeuroReferral::LOGICAL_LABEL),
            ("FileCtOrder", FileCtOrder::LOGICAL_LABEL),
            ("FileMriOrder", FileMriOrder::LOGICAL_LABEL),
            ("FileCardFindings", FileCardFindings::LOGICAL_LABEL),
            ("FileNeuroFindings", FileNeuroFindings::LOGICAL_LABEL),
            ("Candidate", Candidate::LOGICAL_LABEL),
        ]
        .into_iter()
        .map(|(name, label)| (name.into(), label.into()))
        .collect();
        let roles: serde_json::Map<String, serde_json::Value> = [
            ("ER_DOCTOR", ER_DOCTOR),
            ("CARDIOLOGIST", CARDIOLOGIST),
            ("NEUROLOGIST", NEUROLOGIST),
            ("RADIOLOGIST", RADIOLOGIST),
            ("PHARMACIST", PHARMACIST),
            ("ER_IO", ER_IO),
            ("CARDIO_IO", CARDIO_IO),
            ("NEURO_IO", NEURO_IO),
            ("RADIOLOGY_IO", RADIOLOGY_IO),
            ("PHARMACY_IO", PHARMACY_IO),
            ("GENERAL_SERVICE", GENERAL_SERVICE),
            ("CARDIAC_SERVICE", CARDIAC_SERVICE),
            ("NEURO_SERVICE", NEURO_SERVICE),
            ("CT_SERVICE", CT_SERVICE),
            ("MRI_SERVICE", MRI_SERVICE),
            ("PHARMACY_SERVICE", PHARMACY_SERVICE),
            ("CONFERENCE_SERVICE", CONFERENCE_SERVICE),
            ("GENERAL_PANEL", GENERAL_PANEL),
            ("CARDIAC_PANEL", CARDIAC_PANEL),
            ("NEURO_PANEL", NEURO_PANEL),
            ("CT_SCANNER", CT_SCANNER),
            ("MRI", MRI),
            ("CONSULTATION_ROOM", CONSULTATION_ROOM),
            ("ER_VM", ER_VM),
            ("CARDIO_VM", CARDIO_VM),
            ("NEURO_VM", NEURO_VM),
            ("RADIOLOGY_VM", RADIOLOGY_VM),
            ("PHARMACY_VM", PHARMACY_VM),
            ("ER_FILE", ER_FILE),
            ("ER_DATA", ER_DATA),
            ("CARDIO_FILE", CARDIO_FILE),
            ("CARDIO_DATA", CARDIO_DATA),
            ("NEURO_FILE", NEURO_FILE),
            ("NEURO_DATA", NEURO_DATA),
            ("RADIOLOGY_FILE", RADIOLOGY_FILE),
            ("RADIOLOGY_DATA", RADIOLOGY_DATA),
            ("PHARMACY_FILE", PHARMACY_FILE),
            ("PHARMACY_DATA", PHARMACY_DATA),
        ]
        .into_iter()
        .map(|(name, role)| (name.into(), role.into()))
        .collect();
        let programs = [
            rows(&self.er),
            rows(&self.cardio),
            rows(&self.neuro),
            rows(&self.radiologist),
            rows(&self.pharmacist),
            rows(&self.er_io),
            rows(&self.cardio_io),
            rows(&self.neuro_io),
            rows(&self.radiology_io),
            rows(&self.pharmacy_io),
            rows(&self.general),
            rows(&self.cardiac),
            rows(&self.neurology),
            rows(&self.ct_service),
            rows(&self.mri_service),
            rows(&self.pharmacy),
            rows(&self.conference),
            rows(&self.general_panel),
            rows(&self.cardiac_panel),
            rows(&self.neuro_panel),
            rows(&self.ct),
            rows(&self.mri),
            rows(&self.room),
            rows(&self.er_vm),
            rows(&self.cardio_vm),
            rows(&self.neuro_vm),
            rows(&self.radiology_vm),
            rows(&self.pharmacy_vm),
            rows(&self.er_file),
            rows(&self.er_data),
            rows(&self.cardio_file),
            rows(&self.cardio_data),
            rows(&self.neuro_file),
            rows(&self.neuro_data),
            rows(&self.radiology_file),
            rows(&self.radiology_data),
            rows(&self.pharmacy_file),
            rows(&self.pharmacy_data),
        ];
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"5M","session":SESSION,"roles":roles,"labels":labels,"programs":programs.as_slice()})
    }
}
pub fn programs() -> Programs {
    let global = source();
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
        er_vm: project::<ER_VM>(&global),
        cardio_vm: project::<CARDIO_VM>(&global),
        neuro_vm: project::<NEURO_VM>(&global),
        radiology_vm: project::<RADIOLOGY_VM>(&global),
        pharmacy_vm: project::<PHARMACY_VM>(&global),
        er_file: project::<ER_FILE>(&global),
        er_data: project::<ER_DATA>(&global),
        cardio_file: project::<CARDIO_FILE>(&global),
        cardio_data: project::<CARDIO_DATA>(&global),
        neuro_file: project::<NEURO_FILE>(&global),
        neuro_data: project::<NEURO_DATA>(&global),
        radiology_file: project::<RADIOLOGY_FILE>(&global),
        radiology_data: project::<RADIOLOGY_DATA>(&global),
        pharmacy_file: project::<PHARMACY_FILE>(&global),
        pharmacy_data: project::<PHARMACY_DATA>(&global),
    }
}
