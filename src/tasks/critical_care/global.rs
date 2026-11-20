//! Direct patient operation files, one CT authority and independent treatment.
use super::imp::{Arguments, EffectRequest};
pub use crate::shell::global::{FileExit, Loan};
use hibana::{
    Endpoint, g,
    runtime::program::{Projectable, RoleProgram, project},
};
use hibana_wasip1_runtime::{
    choreofs::{ChoreoFsObject, FdSpec, ObjectId},
    protocol,
};
pub const ER_A: u8 = 0;
pub const ER_B: u8 = 1;
pub const RADIOLOGIST: u8 = 2;
pub const SURGEON: u8 = 3;
pub const ANESTHESIOLOGIST: u8 = 4;
pub const ICU_COORDINATOR: u8 = 5;
pub const PHARMACIST: u8 = 6;
pub const ER_A_IO: u8 = 7;
pub const ER_B_IO: u8 = 8;
pub const RADIOLOGY_IO: u8 = 9;
pub const SURGERY_IO: u8 = 10;
pub const ANESTHESIA_IO: u8 = 11;
pub const ICU_IO: u8 = 12;
pub const PHARMACY_IO: u8 = 13;
pub const PATIENT_A_SERVICE: u8 = 14;
pub const PATIENT_B_SERVICE: u8 = 15;
pub const CT_SERVICE: u8 = 16;
pub const MRI_SERVICE: u8 = 17;
pub const ANESTHESIA_SERVICE: u8 = 18;
pub const SURGERY_SERVICE: u8 = 19;
pub const ICU_SERVICE: u8 = 20;
pub const PHARMACY_SERVICE: u8 = 21;
pub const PANEL_A: u8 = 22;
pub const PANEL_B: u8 = 23;
pub const CT_SCANNER: u8 = 24;
pub const MRI: u8 = 25;
pub const OPERATING_ROOM: u8 = 26;
pub const ICU_BEDS: u8 = 27;
pub const VENTILATOR: u8 = 28;
pub const PANEL_A_COMPLETE: u16 = 701;
pub const PANEL_B_COMPLETE: u16 = 702;
pub const SESSION: u32 = 52;
pub type PanelLoan<'r, const RESOURCE: u8> = g::Msg<1, Endpoint<'r, RESOURCE>>;
pub type PanelReturn<'r, const RESOURCE: u8> = g::Msg<2, Endpoint<'r, RESOURCE>>;
pub type PanelCompleted<'r, const RESOURCE: u8> = g::Msg<3, Endpoint<'r, RESOURCE>>;
pub type TestRequest<const OPERATION: u8> = g::Msg<4, EffectRequest<OPERATION>>;
pub type TestResult<'a> = g::Msg<5, &'a [u8]>;
pub type PanelPartial<'a> = g::Msg<6, &'a [u8]>;
pub type PanelComplete<'a> = g::Msg<7, &'a [u8]>;
pub type AssessmentRequestA = g::Msg<8, EffectRequest<0>>;
pub type AssessmentResultA<'a> = g::Msg<9, &'a [u8]>;
pub type AssessmentRequestB = g::Msg<10, EffectRequest<1>>;
pub type AssessmentResultB<'a> = g::Msg<11, &'a [u8]>;
pub type CtPerformIntentA = g::Msg<14, Arguments<4>>;
pub type CtInterpretIntentA = g::Msg<15, Arguments<5>>;
pub type CtPerformIntentB = g::Msg<16, Arguments<6>>;
pub type CtInterpretIntentB = g::Msg<17, Arguments<7>>;
pub type CtLoan<'r> = g::Msg<18, Endpoint<'r, CT_SCANNER>>;
pub type CtPerformRequestA = g::Msg<19, EffectRequest<4>>;
pub type CtInterpretRequestA = g::Msg<20, EffectRequest<5>>;
pub type CtPerformRequestB = g::Msg<21, EffectRequest<6>>;
pub type CtInterpretRequestB = g::Msg<22, EffectRequest<7>>;
pub type CtImageA<'a> = g::Msg<23, &'a [u8]>;
pub type CtReportA<'a> = g::Msg<24, &'a [u8]>;
pub type CtImageB<'a> = g::Msg<25, &'a [u8]>;
pub type CtReportB<'a> = g::Msg<26, &'a [u8]>;
pub type CtAssessmentQueryA = g::Msg<27, Arguments<4>>;
pub type CtAssessmentQueryB = g::Msg<28, Arguments<6>>;
pub type CtAssessmentA<'a> = g::Msg<29, &'a [u8]>;
pub type CtAssessmentB<'a> = g::Msg<30, &'a [u8]>;
pub type CtFindingsA<'a> = g::Msg<31, &'a [u8]>;
pub type CtFindingsB<'a> = g::Msg<32, &'a [u8]>;
pub type MriPerformIntent = g::Msg<33, Arguments<8>>;
pub type MriInterpretIntent = g::Msg<34, Arguments<9>>;
pub type MriLoan<'r> = g::Msg<35, Endpoint<'r, MRI>>;
pub type MriPerformRequest = g::Msg<36, EffectRequest<8>>;
pub type MriInterpretRequest = g::Msg<37, EffectRequest<9>>;
pub type MriImage<'a> = g::Msg<38, &'a [u8]>;
pub type MriReport<'a> = g::Msg<39, &'a [u8]>;
pub type MriAssessmentQuery = g::Msg<40, Arguments<8>>;
pub type MriAssessment<'a> = g::Msg<41, &'a [u8]>;
pub type MriFindingsB<'a> = g::Msg<42, &'a [u8]>;
pub type FindingsA<'a> = g::Msg<43, &'a [u8]>;
pub type FindingsB<'a> = g::Msg<44, &'a [u8]>;
pub type DiagnosisRequestA = g::Msg<45, EffectRequest<14>>;
pub type DiagnosisResultA<'a> = g::Msg<46, &'a [u8]>;
pub type DiagnosisRequestB = g::Msg<47, EffectRequest<15>>;
pub type DiagnosisResultB<'a> = g::Msg<48, &'a [u8]>;
pub type DiagnosisForAnesthesia<'a> = g::Msg<49, &'a [u8]>;
pub type DiagnosisForSurgery<'a> = g::Msg<50, &'a [u8]>;
pub type DiagnosisForIcu<'a> = g::Msg<51, &'a [u8]>;
pub type AnesthesiaIntent = g::Msg<52, Arguments<10>>;
pub type AnesthesiaLoan<'r> = g::Msg<53, Endpoint<'r, OPERATING_ROOM>>;
pub type AnesthesiaRequest = g::Msg<54, EffectRequest<10>>;
pub type AnesthesiaResult<'a> = g::Msg<55, &'a [u8]>;
pub type PreparationForSurgery<'a> = g::Msg<56, &'a [u8]>;
pub type PreparedRoom<'r> = g::Msg<57, Endpoint<'r, OPERATING_ROOM>>;
pub type SurgeryIntent = g::Msg<58, Arguments<11>>;
pub type SurgeryLoan<'r> = g::Msg<59, Endpoint<'r, OPERATING_ROOM>>;
pub type SurgeryRequest = g::Msg<60, EffectRequest<11>>;
pub type SurgeryResult<'a> = g::Msg<61, &'a [u8]>;
pub type SurgeryForER<'a> = g::Msg<62, &'a [u8]>;
pub type IcuIntent = g::Msg<63, Arguments<12>>;
pub type ResourceLoan<'r, const RESOURCE: u8> = g::Msg<64, Endpoint<'r, RESOURCE>>;
pub type ResourceReturn<'r, const RESOURCE: u8> = g::Msg<65, Endpoint<'r, RESOURCE>>;
pub type BedRequest = g::Msg<66, EffectRequest<12>>;
pub type VentilatorRequest = g::Msg<67, EffectRequest<12>>;
pub type BedResult<'a> = g::Msg<68, &'a [u8]>;
pub type VentilatorResult<'a> = g::Msg<69, &'a [u8]>;
pub type IcuForER<'a> = g::Msg<70, &'a [u8]>;
pub type ReviewRequest = g::Msg<71, EffectRequest<13>>;
pub type ReviewResult<'a> = g::Msg<72, &'a [u8]>;
pub type ReviewForA<'a> = g::Msg<73, &'a [u8]>;
pub type ReviewForB<'a> = g::Msg<74, &'a [u8]>;

pub const ER_A_VM: u8 = 64;
pub const ER_A_FILE: u8 = 32;
pub const ER_A_DATA: u8 = 96;
pub const ER_B_VM: u8 = 65;
pub const ER_B_FILE: u8 = 33;
pub const ER_B_DATA: u8 = 97;
pub const RADIOLOGY_VM: u8 = 66;
pub const RADIOLOGY_FILE: u8 = 34;
pub const RADIOLOGY_DATA: u8 = 98;
pub const SURGERY_VM: u8 = 67;
pub const SURGERY_FILE: u8 = 35;
pub const SURGERY_DATA: u8 = 99;
pub const ANESTHESIA_VM: u8 = 68;
pub const ANESTHESIA_FILE: u8 = 36;
pub const ANESTHESIA_DATA: u8 = 100;
pub const ICU_VM: u8 = 69;
pub const ICU_FILE: u8 = 37;
pub const ICU_DATA: u8 = 101;
pub const PHARMACY_VM: u8 = 70;
pub const PHARMACY_FILE: u8 = 38;
pub const PHARMACY_DATA: u8 = 102;
pub type FileAssessmentA<'a> = g::Msg<75, &'a [u8]>;
pub type FileAssessmentB<'a> = g::Msg<76, &'a [u8]>;
pub type PanelFileRequestA = g::Msg<77, Arguments<2>>;
pub type PanelAdmittedA = g::Msg<78, Arguments<2>>;
pub type FilePanelResultA<'a> = g::Msg<79, &'a [u8]>;
pub type PanelFileRequestB = g::Msg<80, Arguments<3>>;
pub type PanelAdmittedB = g::Msg<81, Arguments<3>>;
pub type FilePanelResultB<'a> = g::Msg<82, &'a [u8]>;
pub type FileDiagnosisResultA<'a> = g::Msg<83, &'a [u8]>;
pub type FileDiagnosisResultB<'a> = g::Msg<84, &'a [u8]>;
pub type FileFindingsA<'a> = g::Msg<85, &'a [u8]>;
pub type FileFindingsB<'a> = g::Msg<86, &'a [u8]>;
pub type SummaryA<'a> = g::Msg<87, &'a [u8]>;
pub type SummaryB<'a> = g::Msg<88, &'a [u8]>;
pub type CtPerformAdmittedA = g::Msg<89, Arguments<4>>;
pub type CtInterpretAdmittedA = g::Msg<90, Arguments<5>>;
pub type CtPerformAdmittedB = g::Msg<91, Arguments<6>>;
pub type CtInterpretAdmittedB = g::Msg<92, Arguments<7>>;
pub type MriPerformAdmitted = g::Msg<93, Arguments<8>>;
pub type MriInterpretAdmitted = g::Msg<94, Arguments<9>>;
pub type FileCtImageA<'a> = g::Msg<95, &'a [u8]>;
pub type FileCtReportA<'a> = g::Msg<96, &'a [u8]>;
pub type FileCtImageB<'a> = g::Msg<97, &'a [u8]>;
pub type FileCtReportB<'a> = g::Msg<98, &'a [u8]>;
pub type FileMriImage<'a> = g::Msg<99, &'a [u8]>;
pub type FileMriReport<'a> = g::Msg<100, &'a [u8]>;
pub type AnesthesiaAdmitted = g::Msg<101, Arguments<10>>;
pub type SurgeryAdmitted = g::Msg<102, Arguments<11>>;
pub type IcuAdmitted = g::Msg<103, Arguments<12>>;
pub type FileAnesthesiaResult<'a> = g::Msg<104, &'a [u8]>;
pub type FileSurgeryResult<'a> = g::Msg<105, &'a [u8]>;
pub type FileIcuResult<'a> = g::Msg<106, &'a [u8]>;
pub type FileReviewResult<'a> = g::Msg<107, &'a [u8]>;
pub type FileDiagnosisAnesthesia<'a> = g::Msg<110, &'a [u8]>;
pub type FileDiagnosisSurgery<'a> = g::Msg<111, &'a [u8]>;
pub type FileDiagnosisIcu<'a> = g::Msg<112, &'a [u8]>;

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
pub const ER_A_FILES: [ChoreoFsObject<'static>; 5] = [
    operation(b"service/assess_patient", 4),
    operation(b"service/order_tests", 5),
    result(b"service/findings", 6),
    operation(b"service/diagnose", 7),
    result(b"service/summary", 8),
];
pub const ER_B_FILES: [ChoreoFsObject<'static>; 5] = [
    operation(b"service/assess_patient", 4),
    operation(b"service/order_tests", 5),
    result(b"service/findings", 6),
    operation(b"service/diagnose", 7),
    result(b"service/summary", 8),
];
pub const RADIOLOGY_FILES: [ChoreoFsObject<'static>; 6] = [
    operation(b"service/perform_imaging_ct_a", 4),
    operation(b"service/interpret_imaging_ct_a", 5),
    operation(b"service/perform_imaging_ct_b", 6),
    operation(b"service/interpret_imaging_ct_b", 7),
    operation(b"service/perform_imaging_mri_b", 8),
    operation(b"service/interpret_imaging_mri_b", 9),
];

pub const SURGERY_FILES: [ChoreoFsObject<'static>; 2] = [
    result(b"service/diagnosis", 4),
    operation(b"service/perform_surgery", 5),
];
pub const ANESTHESIA_FILES: [ChoreoFsObject<'static>; 2] = [
    result(b"service/diagnosis", 4),
    operation(b"service/prepare_anesthesia", 5),
];
pub const ICU_FILES: [ChoreoFsObject<'static>; 2] = [
    result(b"service/diagnosis", 4),
    operation(b"service/manage_icu", 5),
];
pub const PHARMACY_FILES: [ChoreoFsObject<'static>; 1] =
    [operation(b"service/review_prescriptions", 4)];
pub fn source() -> impl Projectable {
    let panel_a_attempt = || {
        g::seq(
            g::send::<PATIENT_A_SERVICE, ER_A, PanelLoan<PANEL_A>>(),
            g::seq(
                g::send::<ER_A_FILE, PATIENT_A_SERVICE, PanelFileRequestA>(),
                g::seq(
                    g::send::<PATIENT_A_SERVICE, ER_A, PanelAdmittedA>(),
                    g::seq(
                        g::send::<PANEL_A, PATIENT_A_SERVICE, TestRequest<2>>(),
                        g::seq(
                            g::send::<PATIENT_A_SERVICE, PANEL_A, TestResult>(),
                            g::send::<ER_A, PATIENT_A_SERVICE, PanelReturn<PANEL_A>>(),
                        ),
                    ),
                ),
            ),
        )
    };
    let panel_a_complete = || {
        g::seq(
            g::send::<PATIENT_A_SERVICE, ER_A, PanelCompleted<PANEL_A>>(),
            g::seq(
                g::send::<PATIENT_A_SERVICE, ER_A, PanelComplete>(),
                g::send::<ER_A, ER_A_FILE, FilePanelResultA>(),
            ),
        )
    };
    let panel_a = g::seq(
        panel_a_attempt(),
        g::route(
            panel_a_complete(),
            g::seq(
                (g::seq(
                    g::send::<PATIENT_A_SERVICE, ER_A_FILE, FilePanelResultA>(),
                    g::seq(
                        g::send::<PATIENT_A_SERVICE, ER_A, PanelPartial>(),
                        panel_a_attempt(),
                    ),
                ))
                .roll(),
                panel_a_complete(),
            ),
        )
        .resolve::<PANEL_A_COMPLETE>(),
    );
    let panel_b_attempt = || {
        g::seq(
            g::send::<PATIENT_B_SERVICE, ER_B, PanelLoan<PANEL_B>>(),
            g::seq(
                g::send::<ER_B_FILE, PATIENT_B_SERVICE, PanelFileRequestB>(),
                g::seq(
                    g::send::<PATIENT_B_SERVICE, ER_B, PanelAdmittedB>(),
                    g::seq(
                        g::send::<PANEL_B, PATIENT_B_SERVICE, TestRequest<3>>(),
                        g::seq(
                            g::send::<PATIENT_B_SERVICE, PANEL_B, TestResult>(),
                            g::send::<ER_B, PATIENT_B_SERVICE, PanelReturn<PANEL_B>>(),
                        ),
                    ),
                ),
            ),
        )
    };
    let panel_b_complete = || {
        g::seq(
            g::send::<PATIENT_B_SERVICE, ER_B, PanelCompleted<PANEL_B>>(),
            g::seq(
                g::send::<PATIENT_B_SERVICE, ER_B, PanelComplete>(),
                g::send::<ER_B, ER_B_FILE, FilePanelResultB>(),
            ),
        )
    };
    let panel_b = g::seq(
        panel_b_attempt(),
        g::route(
            panel_b_complete(),
            g::seq(
                (g::seq(
                    g::send::<PATIENT_B_SERVICE, ER_B_FILE, FilePanelResultB>(),
                    g::seq(
                        g::send::<PATIENT_B_SERVICE, ER_B, PanelPartial>(),
                        panel_b_attempt(),
                    ),
                ))
                .roll(),
                panel_b_complete(),
            ),
        )
        .resolve::<PANEL_B_COMPLETE>(),
    );
    let assessment = g::par(
        g::seq(
            g::send::<ER_A_FILE, PATIENT_A_SERVICE, AssessmentRequestA>(),
            g::par(
                g::send::<PATIENT_A_SERVICE, ER_A, AssessmentResultA>(),
                g::send::<PATIENT_A_SERVICE, ER_A_FILE, FileAssessmentA>(),
            ),
        ),
        g::seq(
            g::send::<ER_B_FILE, PATIENT_B_SERVICE, AssessmentRequestB>(),
            g::par(
                g::send::<PATIENT_B_SERVICE, ER_B, AssessmentResultB>(),
                g::send::<PATIENT_B_SERVICE, ER_B_FILE, FileAssessmentB>(),
            ),
        ),
    );
    let ct_a_perform = || {
        g::seq(
            g::send::<RADIOLOGY_FILE, CT_SERVICE, CtPerformIntentA>(),
            g::seq(
                g::send::<CT_SERVICE, PATIENT_A_SERVICE, CtAssessmentQueryA>(),
                g::seq(
                    g::send::<PATIENT_A_SERVICE, CT_SERVICE, CtAssessmentA>(),
                    g::seq(
                        g::send::<CT_SERVICE, RADIOLOGIST, CtPerformAdmittedA>(),
                        g::seq(
                            g::send::<CT_SERVICE, RADIOLOGIST, CtLoan>(),
                            g::seq(
                                g::send::<CT_SCANNER, CT_SERVICE, CtPerformRequestA>(),
                                g::seq(
                                    g::par(
                                        g::send::<CT_SERVICE, CT_SCANNER, CtImageA>(),
                                        g::send::<CT_SERVICE, RADIOLOGY_FILE, FileCtImageA>(),
                                    ),
                                    g::send::<RADIOLOGIST, CT_SERVICE, CtLoan>(),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        )
    };
    let ct_a_interpret = || {
        g::seq(
            g::send::<RADIOLOGY_FILE, CT_SERVICE, CtInterpretIntentA>(),
            g::seq(
                g::send::<CT_SERVICE, RADIOLOGIST, CtInterpretAdmittedA>(),
                g::seq(
                    g::send::<CT_SERVICE, RADIOLOGIST, CtLoan>(),
                    g::seq(
                        g::send::<CT_SCANNER, CT_SERVICE, CtInterpretRequestA>(),
                        g::seq(
                            g::par(
                                g::send::<CT_SERVICE, CT_SCANNER, CtReportA>(),
                                g::send::<CT_SERVICE, RADIOLOGY_FILE, FileCtReportA>(),
                            ),
                            g::seq(
                                g::send::<RADIOLOGIST, CT_SERVICE, CtLoan>(),
                                g::send::<CT_SERVICE, PATIENT_A_SERVICE, CtFindingsA>(),
                            ),
                        ),
                    ),
                ),
            ),
        )
    };
    let ct_b_perform = || {
        g::seq(
            g::send::<RADIOLOGY_FILE, CT_SERVICE, CtPerformIntentB>(),
            g::seq(
                g::send::<CT_SERVICE, PATIENT_B_SERVICE, CtAssessmentQueryB>(),
                g::seq(
                    g::send::<PATIENT_B_SERVICE, CT_SERVICE, CtAssessmentB>(),
                    g::seq(
                        g::send::<CT_SERVICE, RADIOLOGIST, CtPerformAdmittedB>(),
                        g::seq(
                            g::send::<CT_SERVICE, RADIOLOGIST, CtLoan>(),
                            g::seq(
                                g::send::<CT_SCANNER, CT_SERVICE, CtPerformRequestB>(),
                                g::seq(
                                    g::par(
                                        g::send::<CT_SERVICE, CT_SCANNER, CtImageB>(),
                                        g::send::<CT_SERVICE, RADIOLOGY_FILE, FileCtImageB>(),
                                    ),
                                    g::send::<RADIOLOGIST, CT_SERVICE, CtLoan>(),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        )
    };
    let ct_b_interpret = || {
        g::seq(
            g::send::<RADIOLOGY_FILE, CT_SERVICE, CtInterpretIntentB>(),
            g::seq(
                g::send::<CT_SERVICE, RADIOLOGIST, CtInterpretAdmittedB>(),
                g::seq(
                    g::send::<CT_SERVICE, RADIOLOGIST, CtLoan>(),
                    g::seq(
                        g::send::<CT_SCANNER, CT_SERVICE, CtInterpretRequestB>(),
                        g::seq(
                            g::par(
                                g::send::<CT_SERVICE, CT_SCANNER, CtReportB>(),
                                g::send::<CT_SERVICE, RADIOLOGY_FILE, FileCtReportB>(),
                            ),
                            g::seq(
                                g::send::<RADIOLOGIST, CT_SERVICE, CtLoan>(),
                                g::send::<CT_SERVICE, PATIENT_B_SERVICE, CtFindingsB>(),
                            ),
                        ),
                    ),
                ),
            ),
        )
    };
    let ct = g::route(
        g::seq(
            ct_a_perform(),
            g::route(
                g::seq(ct_a_interpret(), g::seq(ct_b_perform(), ct_b_interpret())),
                g::seq(
                    ct_b_perform(),
                    g::route(
                        g::seq(ct_a_interpret(), ct_b_interpret()),
                        g::seq(ct_b_interpret(), ct_a_interpret()),
                    ),
                ),
            ),
        ),
        g::seq(
            ct_b_perform(),
            g::route(
                g::seq(ct_b_interpret(), g::seq(ct_a_perform(), ct_a_interpret())),
                g::seq(
                    ct_a_perform(),
                    g::route(
                        g::seq(ct_b_interpret(), ct_a_interpret()),
                        g::seq(ct_a_interpret(), ct_b_interpret()),
                    ),
                ),
            ),
        ),
    );
    let mri = g::seq(
        g::seq(
            g::send::<RADIOLOGY_FILE, MRI_SERVICE, MriPerformIntent>(),
            g::seq(
                g::send::<MRI_SERVICE, PATIENT_B_SERVICE, MriAssessmentQuery>(),
                g::seq(
                    g::send::<PATIENT_B_SERVICE, MRI_SERVICE, MriAssessment>(),
                    g::seq(
                        g::send::<MRI_SERVICE, RADIOLOGIST, MriPerformAdmitted>(),
                        g::seq(
                            g::send::<MRI_SERVICE, RADIOLOGIST, MriLoan>(),
                            g::seq(
                                g::send::<MRI, MRI_SERVICE, MriPerformRequest>(),
                                g::seq(
                                    g::par(
                                        g::send::<MRI_SERVICE, MRI, MriImage>(),
                                        g::send::<MRI_SERVICE, RADIOLOGY_FILE, FileMriImage>(),
                                    ),
                                    g::send::<RADIOLOGIST, MRI_SERVICE, MriLoan>(),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
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
                                g::send::<MRI_SERVICE, MRI, MriReport>(),
                                g::send::<MRI_SERVICE, RADIOLOGY_FILE, FileMriReport>(),
                            ),
                            g::seq(
                                g::send::<RADIOLOGIST, MRI_SERVICE, MriLoan>(),
                                g::send::<MRI_SERVICE, PATIENT_B_SERVICE, MriFindingsB>(),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );
    let workup = g::par(panel_a, g::par(panel_b, g::par(ct, mri)));
    let diagnosis = g::par(
        g::seq(
            g::send::<PATIENT_A_SERVICE, ER_A, FindingsA>(),
            g::seq(
                g::send::<ER_A, ER_A_FILE, FileFindingsA>(),
                g::seq(
                    g::send::<ER_A_FILE, PATIENT_A_SERVICE, DiagnosisRequestA>(),
                    g::par(
                        g::send::<PATIENT_A_SERVICE, ER_A, DiagnosisResultA>(),
                        g::par(
                            g::send::<PATIENT_A_SERVICE, ER_A_FILE, FileDiagnosisResultA>(),
                            g::par(
                                g::send::<
                                    PATIENT_A_SERVICE,
                                    ANESTHESIA_SERVICE,
                                    DiagnosisForAnesthesia,
                                >(),
                                g::par(
                                    g::send::<
                                        PATIENT_A_SERVICE,
                                        SURGERY_SERVICE,
                                        DiagnosisForSurgery,
                                    >(),
                                    g::par(
                                        g::send::<
                                            PATIENT_A_SERVICE,
                                            ANESTHESIA_FILE,
                                            FileDiagnosisAnesthesia,
                                        >(),
                                        g::send::<
                                            PATIENT_A_SERVICE,
                                            SURGERY_FILE,
                                            FileDiagnosisSurgery,
                                        >(),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
        g::seq(
            g::send::<PATIENT_B_SERVICE, ER_B, FindingsB>(),
            g::seq(
                g::send::<ER_B, ER_B_FILE, FileFindingsB>(),
                g::seq(
                    g::send::<ER_B_FILE, PATIENT_B_SERVICE, DiagnosisRequestB>(),
                    g::par(
                        g::send::<PATIENT_B_SERVICE, ER_B, DiagnosisResultB>(),
                        g::par(
                            g::send::<PATIENT_B_SERVICE, ER_B_FILE, FileDiagnosisResultB>(),
                            g::par(
                                g::send::<PATIENT_B_SERVICE, ICU_SERVICE, DiagnosisForIcu>(),
                                g::send::<PATIENT_B_SERVICE, ICU_FILE, FileDiagnosisIcu>(),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );
    let anesthesia = g::seq(
        g::send::<ANESTHESIA_FILE, ANESTHESIA_SERVICE, AnesthesiaIntent>(),
        g::seq(
            g::send::<ANESTHESIA_SERVICE, ANESTHESIOLOGIST, AnesthesiaAdmitted>(),
            g::seq(
                g::send::<ANESTHESIA_SERVICE, ANESTHESIOLOGIST, AnesthesiaLoan>(),
                g::seq(
                    g::send::<OPERATING_ROOM, ANESTHESIA_SERVICE, AnesthesiaRequest>(),
                    g::seq(
                        g::par(
                            g::send::<ANESTHESIA_SERVICE, OPERATING_ROOM, AnesthesiaResult>(),
                            g::send::<ANESTHESIA_SERVICE, ANESTHESIA_FILE, FileAnesthesiaResult>(),
                        ),
                        g::seq(
                            g::send::<ANESTHESIOLOGIST, ANESTHESIA_SERVICE, AnesthesiaLoan>(),
                            g::par(
                                g::send::<ANESTHESIA_SERVICE, SURGERY_SERVICE, PreparedRoom>(),
                                g::send::<ANESTHESIA_SERVICE, SURGERY_SERVICE, PreparationForSurgery>(
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );
    let surgery = g::seq(
        g::send::<SURGERY_FILE, SURGERY_SERVICE, SurgeryIntent>(),
        g::seq(
            g::send::<SURGERY_SERVICE, SURGEON, SurgeryAdmitted>(),
            g::seq(
                g::send::<SURGERY_SERVICE, SURGEON, SurgeryLoan>(),
                g::seq(
                    g::send::<OPERATING_ROOM, SURGERY_SERVICE, SurgeryRequest>(),
                    g::seq(
                        g::par(
                            g::send::<SURGERY_SERVICE, OPERATING_ROOM, SurgeryResult>(),
                            g::send::<SURGERY_SERVICE, SURGERY_FILE, FileSurgeryResult>(),
                        ),
                        g::seq(
                            g::send::<SURGEON, SURGERY_SERVICE, SurgeryLoan>(),
                            g::send::<SURGERY_SERVICE, ER_A, SurgeryForER>(),
                        ),
                    ),
                ),
            ),
        ),
    );
    let icu = g::seq(
        g::send::<ICU_FILE, ICU_SERVICE, IcuIntent>(),
        g::seq(
            g::send::<ICU_SERVICE, ICU_COORDINATOR, IcuAdmitted>(),
            g::seq(
                g::par(
                    g::send::<ICU_SERVICE, ICU_COORDINATOR, ResourceLoan<ICU_BEDS>>(),
                    g::send::<ICU_SERVICE, ICU_COORDINATOR, ResourceLoan<VENTILATOR>>(),
                ),
                g::seq(
                    g::par(
                        g::send::<ICU_BEDS, ICU_SERVICE, BedRequest>(),
                        g::send::<VENTILATOR, ICU_SERVICE, VentilatorRequest>(),
                    ),
                    g::seq(
                        g::par(
                            g::send::<ICU_SERVICE, ICU_BEDS, BedResult>(),
                            g::par(
                                g::send::<ICU_SERVICE, VENTILATOR, VentilatorResult>(),
                                g::send::<ICU_SERVICE, ICU_FILE, FileIcuResult>(),
                            ),
                        ),
                        g::seq(
                            g::par(
                                g::send::<ICU_COORDINATOR, ICU_SERVICE, ResourceReturn<ICU_BEDS>>(),
                                g::send::<ICU_COORDINATOR, ICU_SERVICE, ResourceReturn<VENTILATOR>>(
                                ),
                            ),
                            g::send::<ICU_SERVICE, ER_B, IcuForER>(),
                        ),
                    ),
                ),
            ),
        ),
    );
    let clinical = g::seq(
        assessment,
        g::seq(
            workup,
            g::seq(diagnosis, g::par(g::seq(anesthesia, surgery), icu)),
        ),
    );
    let review = g::seq(
        g::send::<PHARMACY_FILE, PHARMACY_SERVICE, ReviewRequest>(),
        g::par(
            g::send::<PHARMACY_SERVICE, PHARMACIST, ReviewResult>(),
            g::par(
                g::send::<PHARMACY_SERVICE, PHARMACY_FILE, FileReviewResult>(),
                g::par(
                    g::send::<PHARMACY_SERVICE, ER_A, ReviewForA>(),
                    g::send::<PHARMACY_SERVICE, ER_B, ReviewForB>(),
                ),
            ),
        ),
    );
    let business = g::seq(
        g::par(clinical, review),
        g::par(
            g::send::<ER_A, ER_A_FILE, SummaryA>(),
            g::send::<ER_B, ER_B_FILE, SummaryB>(),
        ),
    );
    let bootstrap = g::par(
        g::send::<ER_A_DATA, ER_A_VM, Loan<ER_A_FILE>>(),
        g::par(
            g::send::<ER_B_DATA, ER_B_VM, Loan<ER_B_FILE>>(),
            g::par(
                g::send::<RADIOLOGY_DATA, RADIOLOGY_VM, Loan<RADIOLOGY_FILE>>(),
                g::par(
                    g::send::<SURGERY_DATA, SURGERY_VM, Loan<SURGERY_FILE>>(),
                    g::par(
                        g::send::<ANESTHESIA_DATA, ANESTHESIA_VM, Loan<ANESTHESIA_FILE>>(),
                        g::par(
                            g::send::<ICU_DATA, ICU_VM, Loan<ICU_FILE>>(),
                            g::send::<PHARMACY_DATA, PHARMACY_VM, Loan<PHARMACY_FILE>>(),
                        ),
                    ),
                ),
            ),
        ),
    );
    let metadata =
        g::par(
            g::seq(
                crate::shell::global::file_metadata::<ER_A_FILE, ER_A_DATA>().roll(),
                g::send::<ER_A_FILE, ER_A_DATA, FileExit>(),
            ),
            g::par(
                g::seq(
                    crate::shell::global::file_metadata::<ER_B_FILE, ER_B_DATA>().roll(),
                    g::send::<ER_B_FILE, ER_B_DATA, FileExit>(),
                ),
                g::par(
                    g::seq(
                        crate::shell::global::file_metadata::<RADIOLOGY_FILE, RADIOLOGY_DATA>()
                            .roll(),
                        g::send::<RADIOLOGY_FILE, RADIOLOGY_DATA, FileExit>(),
                    ),
                    g::par(
                        g::seq(
                            crate::shell::global::file_metadata::<SURGERY_FILE, SURGERY_DATA>()
                                .roll(),
                            g::send::<SURGERY_FILE, SURGERY_DATA, FileExit>(),
                        ),
                        g::par(
                            g::seq(
                                crate::shell::global::file_metadata::<
                                    ANESTHESIA_FILE,
                                    ANESTHESIA_DATA,
                                >()
                                .roll(),
                                g::send::<ANESTHESIA_FILE, ANESTHESIA_DATA, FileExit>(),
                            ),
                            g::par(
                                g::seq(
                                    crate::shell::global::file_metadata::<ICU_FILE, ICU_DATA>()
                                        .roll(),
                                    g::send::<ICU_FILE, ICU_DATA, FileExit>(),
                                ),
                                g::seq(
                                    crate::shell::global::file_metadata::<
                                        PHARMACY_FILE,
                                        PHARMACY_DATA,
                                    >()
                                    .roll(),
                                    g::send::<PHARMACY_FILE, PHARMACY_DATA, FileExit>(),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        );
    let private = g::par(
        g::seq(
            crate::shell::global::private_io::<ER_A_VM, ER_A_IO>(),
            g::send::<ER_A_VM, ER_A_IO, crate::shell::global::Exit>(),
        ),
        g::par(
            g::seq(
                crate::shell::global::private_io::<ER_B_VM, ER_B_IO>(),
                g::send::<ER_B_VM, ER_B_IO, crate::shell::global::Exit>(),
            ),
            g::par(
                g::seq(
                    crate::shell::global::private_io::<RADIOLOGY_VM, RADIOLOGY_IO>(),
                    g::send::<RADIOLOGY_VM, RADIOLOGY_IO, crate::shell::global::Exit>(),
                ),
                g::par(
                    g::seq(
                        crate::shell::global::private_io::<SURGERY_VM, SURGERY_IO>(),
                        g::send::<SURGERY_VM, SURGERY_IO, crate::shell::global::Exit>(),
                    ),
                    g::par(
                        g::seq(
                            crate::shell::global::private_io::<ANESTHESIA_VM, ANESTHESIA_IO>(),
                            g::send::<ANESTHESIA_VM, ANESTHESIA_IO, crate::shell::global::Exit>(),
                        ),
                        g::par(
                            g::seq(
                                crate::shell::global::private_io::<ICU_VM, ICU_IO>(),
                                g::send::<ICU_VM, ICU_IO, crate::shell::global::Exit>(),
                            ),
                            g::seq(
                                crate::shell::global::private_io::<PHARMACY_VM, PHARMACY_IO>(),
                                g::send::<PHARMACY_VM, PHARMACY_IO, crate::shell::global::Exit>(),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );
    g::seq(bootstrap, g::par(business, g::par(metadata, private)))
}
pub struct Programs {
    pub er_a: RoleProgram<ER_A>,
    pub er_b: RoleProgram<ER_B>,
    pub radiologist: RoleProgram<RADIOLOGIST>,
    pub surgeon: RoleProgram<SURGEON>,
    pub anesthesiologist: RoleProgram<ANESTHESIOLOGIST>,
    pub icu_coordinator: RoleProgram<ICU_COORDINATOR>,
    pub pharmacist: RoleProgram<PHARMACIST>,
    pub er_a_io: RoleProgram<ER_A_IO>,
    pub er_b_io: RoleProgram<ER_B_IO>,
    pub radiology_io: RoleProgram<RADIOLOGY_IO>,
    pub surgery_io: RoleProgram<SURGERY_IO>,
    pub anesthesia_io: RoleProgram<ANESTHESIA_IO>,
    pub icu_io: RoleProgram<ICU_IO>,
    pub pharmacy_io: RoleProgram<PHARMACY_IO>,
    pub patient_a: RoleProgram<PATIENT_A_SERVICE>,
    pub patient_b: RoleProgram<PATIENT_B_SERVICE>,
    pub ct_service: RoleProgram<CT_SERVICE>,
    pub mri_service: RoleProgram<MRI_SERVICE>,
    pub anesthesia: RoleProgram<ANESTHESIA_SERVICE>,
    pub surgery: RoleProgram<SURGERY_SERVICE>,
    pub icu: RoleProgram<ICU_SERVICE>,
    pub pharmacy: RoleProgram<PHARMACY_SERVICE>,
    pub panel_a: RoleProgram<PANEL_A>,
    pub panel_b: RoleProgram<PANEL_B>,
    pub ct: RoleProgram<CT_SCANNER>,
    pub mri: RoleProgram<MRI>,
    pub room: RoleProgram<OPERATING_ROOM>,
    pub bed: RoleProgram<ICU_BEDS>,
    pub ventilator: RoleProgram<VENTILATOR>,
    pub er_a_vm: RoleProgram<ER_A_VM>,
    pub er_a_file: RoleProgram<ER_A_FILE>,
    pub er_a_data: RoleProgram<ER_A_DATA>,
    pub er_b_vm: RoleProgram<ER_B_VM>,
    pub er_b_file: RoleProgram<ER_B_FILE>,
    pub er_b_data: RoleProgram<ER_B_DATA>,
    pub radiology_vm: RoleProgram<RADIOLOGY_VM>,
    pub radiology_file: RoleProgram<RADIOLOGY_FILE>,
    pub radiology_data: RoleProgram<RADIOLOGY_DATA>,
    pub surgery_vm: RoleProgram<SURGERY_VM>,
    pub surgery_file: RoleProgram<SURGERY_FILE>,
    pub surgery_data: RoleProgram<SURGERY_DATA>,
    pub anesthesia_vm: RoleProgram<ANESTHESIA_VM>,
    pub anesthesia_file: RoleProgram<ANESTHESIA_FILE>,
    pub anesthesia_data: RoleProgram<ANESTHESIA_DATA>,
    pub icu_vm: RoleProgram<ICU_VM>,
    pub icu_file: RoleProgram<ICU_FILE>,
    pub icu_data: RoleProgram<ICU_DATA>,
    pub pharmacy_vm: RoleProgram<PHARMACY_VM>,
    pub pharmacy_file: RoleProgram<PHARMACY_FILE>,
    pub pharmacy_data: RoleProgram<PHARMACY_DATA>,
}
impl Programs {
    pub fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        use hibana::g::Message;
        let labels: serde_json::Map<String, serde_json::Value> = [
            ("PanelLoan", PanelLoan::<PANEL_A>::LOGICAL_LABEL),
            ("PanelReturn", PanelReturn::<PANEL_A>::LOGICAL_LABEL),
            ("PanelCompleted", PanelCompleted::<PANEL_A>::LOGICAL_LABEL),
            ("TestRequest", TestRequest::<2>::LOGICAL_LABEL),
            ("TestResult", TestResult::LOGICAL_LABEL),
            ("PanelPartial", PanelPartial::LOGICAL_LABEL),
            ("PanelComplete", PanelComplete::LOGICAL_LABEL),
            ("AssessmentRequestA", AssessmentRequestA::LOGICAL_LABEL),
            ("AssessmentResultA", AssessmentResultA::LOGICAL_LABEL),
            ("AssessmentRequestB", AssessmentRequestB::LOGICAL_LABEL),
            ("AssessmentResultB", AssessmentResultB::LOGICAL_LABEL),
            ("CtPerformIntentA", CtPerformIntentA::LOGICAL_LABEL),
            ("CtInterpretIntentA", CtInterpretIntentA::LOGICAL_LABEL),
            ("CtPerformIntentB", CtPerformIntentB::LOGICAL_LABEL),
            ("CtInterpretIntentB", CtInterpretIntentB::LOGICAL_LABEL),
            ("CtLoan", CtLoan::LOGICAL_LABEL),
            ("CtPerformRequestA", CtPerformRequestA::LOGICAL_LABEL),
            ("CtInterpretRequestA", CtInterpretRequestA::LOGICAL_LABEL),
            ("CtPerformRequestB", CtPerformRequestB::LOGICAL_LABEL),
            ("CtInterpretRequestB", CtInterpretRequestB::LOGICAL_LABEL),
            ("CtImageA", CtImageA::LOGICAL_LABEL),
            ("CtReportA", CtReportA::LOGICAL_LABEL),
            ("CtImageB", CtImageB::LOGICAL_LABEL),
            ("CtReportB", CtReportB::LOGICAL_LABEL),
            ("CtAssessmentQueryA", CtAssessmentQueryA::LOGICAL_LABEL),
            ("CtAssessmentQueryB", CtAssessmentQueryB::LOGICAL_LABEL),
            ("CtAssessmentA", CtAssessmentA::LOGICAL_LABEL),
            ("CtAssessmentB", CtAssessmentB::LOGICAL_LABEL),
            ("CtFindingsA", CtFindingsA::LOGICAL_LABEL),
            ("CtFindingsB", CtFindingsB::LOGICAL_LABEL),
            ("MriPerformIntent", MriPerformIntent::LOGICAL_LABEL),
            ("MriInterpretIntent", MriInterpretIntent::LOGICAL_LABEL),
            ("MriLoan", MriLoan::LOGICAL_LABEL),
            ("MriPerformRequest", MriPerformRequest::LOGICAL_LABEL),
            ("MriInterpretRequest", MriInterpretRequest::LOGICAL_LABEL),
            ("MriImage", MriImage::LOGICAL_LABEL),
            ("MriReport", MriReport::LOGICAL_LABEL),
            ("MriAssessmentQuery", MriAssessmentQuery::LOGICAL_LABEL),
            ("MriAssessment", MriAssessment::LOGICAL_LABEL),
            ("MriFindingsB", MriFindingsB::LOGICAL_LABEL),
            ("FindingsA", FindingsA::LOGICAL_LABEL),
            ("FindingsB", FindingsB::LOGICAL_LABEL),
            ("DiagnosisRequestA", DiagnosisRequestA::LOGICAL_LABEL),
            ("DiagnosisResultA", DiagnosisResultA::LOGICAL_LABEL),
            ("DiagnosisRequestB", DiagnosisRequestB::LOGICAL_LABEL),
            ("DiagnosisResultB", DiagnosisResultB::LOGICAL_LABEL),
            (
                "DiagnosisForAnesthesia",
                DiagnosisForAnesthesia::LOGICAL_LABEL,
            ),
            ("DiagnosisForSurgery", DiagnosisForSurgery::LOGICAL_LABEL),
            ("DiagnosisForIcu", DiagnosisForIcu::LOGICAL_LABEL),
            ("AnesthesiaIntent", AnesthesiaIntent::LOGICAL_LABEL),
            ("AnesthesiaLoan", AnesthesiaLoan::LOGICAL_LABEL),
            ("AnesthesiaRequest", AnesthesiaRequest::LOGICAL_LABEL),
            ("AnesthesiaResult", AnesthesiaResult::LOGICAL_LABEL),
            (
                "PreparationForSurgery",
                PreparationForSurgery::LOGICAL_LABEL,
            ),
            ("PreparedRoom", PreparedRoom::LOGICAL_LABEL),
            ("SurgeryIntent", SurgeryIntent::LOGICAL_LABEL),
            ("SurgeryLoan", SurgeryLoan::LOGICAL_LABEL),
            ("SurgeryRequest", SurgeryRequest::LOGICAL_LABEL),
            ("SurgeryResult", SurgeryResult::LOGICAL_LABEL),
            ("SurgeryForER", SurgeryForER::LOGICAL_LABEL),
            ("IcuIntent", IcuIntent::LOGICAL_LABEL),
            ("ResourceLoan", ResourceLoan::<PANEL_A>::LOGICAL_LABEL),
            ("ResourceReturn", ResourceReturn::<PANEL_A>::LOGICAL_LABEL),
            ("BedRequest", BedRequest::LOGICAL_LABEL),
            ("VentilatorRequest", VentilatorRequest::LOGICAL_LABEL),
            ("BedResult", BedResult::LOGICAL_LABEL),
            ("VentilatorResult", VentilatorResult::LOGICAL_LABEL),
            ("IcuForER", IcuForER::LOGICAL_LABEL),
            ("ReviewRequest", ReviewRequest::LOGICAL_LABEL),
            ("ReviewResult", ReviewResult::LOGICAL_LABEL),
            ("ReviewForA", ReviewForA::LOGICAL_LABEL),
            ("ReviewForB", ReviewForB::LOGICAL_LABEL),
            ("FileAssessmentA", FileAssessmentA::LOGICAL_LABEL),
            ("FileAssessmentB", FileAssessmentB::LOGICAL_LABEL),
            ("PanelFileRequestA", PanelFileRequestA::LOGICAL_LABEL),
            ("PanelAdmittedA", PanelAdmittedA::LOGICAL_LABEL),
            ("FilePanelResultA", FilePanelResultA::LOGICAL_LABEL),
            ("PanelFileRequestB", PanelFileRequestB::LOGICAL_LABEL),
            ("PanelAdmittedB", PanelAdmittedB::LOGICAL_LABEL),
            ("FilePanelResultB", FilePanelResultB::LOGICAL_LABEL),
            ("FileDiagnosisResultA", FileDiagnosisResultA::LOGICAL_LABEL),
            ("FileDiagnosisResultB", FileDiagnosisResultB::LOGICAL_LABEL),
            ("FileFindingsA", FileFindingsA::LOGICAL_LABEL),
            ("FileFindingsB", FileFindingsB::LOGICAL_LABEL),
            ("SummaryA", SummaryA::LOGICAL_LABEL),
            ("SummaryB", SummaryB::LOGICAL_LABEL),
            ("CtPerformAdmittedA", CtPerformAdmittedA::LOGICAL_LABEL),
            ("CtInterpretAdmittedA", CtInterpretAdmittedA::LOGICAL_LABEL),
            ("CtPerformAdmittedB", CtPerformAdmittedB::LOGICAL_LABEL),
            ("CtInterpretAdmittedB", CtInterpretAdmittedB::LOGICAL_LABEL),
            ("MriPerformAdmitted", MriPerformAdmitted::LOGICAL_LABEL),
            ("MriInterpretAdmitted", MriInterpretAdmitted::LOGICAL_LABEL),
            ("FileCtImageA", FileCtImageA::LOGICAL_LABEL),
            ("FileCtReportA", FileCtReportA::LOGICAL_LABEL),
            ("FileCtImageB", FileCtImageB::LOGICAL_LABEL),
            ("FileCtReportB", FileCtReportB::LOGICAL_LABEL),
            ("FileMriImage", FileMriImage::LOGICAL_LABEL),
            ("FileMriReport", FileMriReport::LOGICAL_LABEL),
            ("AnesthesiaAdmitted", AnesthesiaAdmitted::LOGICAL_LABEL),
            ("SurgeryAdmitted", SurgeryAdmitted::LOGICAL_LABEL),
            ("IcuAdmitted", IcuAdmitted::LOGICAL_LABEL),
            ("FileAnesthesiaResult", FileAnesthesiaResult::LOGICAL_LABEL),
            ("FileSurgeryResult", FileSurgeryResult::LOGICAL_LABEL),
            ("FileIcuResult", FileIcuResult::LOGICAL_LABEL),
            ("FileReviewResult", FileReviewResult::LOGICAL_LABEL),
            (
                "FileDiagnosisAnesthesia",
                FileDiagnosisAnesthesia::LOGICAL_LABEL,
            ),
            ("FileDiagnosisSurgery", FileDiagnosisSurgery::LOGICAL_LABEL),
            ("FileDiagnosisIcu", FileDiagnosisIcu::LOGICAL_LABEL),
        ]
        .into_iter()
        .map(|(name, label)| (name.into(), serde_json::json!(label)))
        .collect();
        let roles: serde_json::Map<String, serde_json::Value> = [
            ("ER_DOCTOR_A", ER_A),
            ("ER_DOCTOR_B", ER_B),
            ("RADIOLOGIST", RADIOLOGIST),
            ("SURGEON", SURGEON),
            ("ANESTHESIOLOGIST", ANESTHESIOLOGIST),
            ("ICU_COORDINATOR", ICU_COORDINATOR),
            ("PHARMACIST", PHARMACIST),
            ("ER_A_IO", ER_A_IO),
            ("ER_B_IO", ER_B_IO),
            ("RADIOLOGY_IO", RADIOLOGY_IO),
            ("SURGERY_IO", SURGERY_IO),
            ("ANESTHESIA_IO", ANESTHESIA_IO),
            ("ICU_IO", ICU_IO),
            ("PHARMACY_IO", PHARMACY_IO),
            ("PATIENT_A_SERVICE", PATIENT_A_SERVICE),
            ("PATIENT_B_SERVICE", PATIENT_B_SERVICE),
            ("CT_SERVICE", CT_SERVICE),
            ("MRI_SERVICE", MRI_SERVICE),
            ("ANESTHESIA_SERVICE", ANESTHESIA_SERVICE),
            ("SURGERY_SERVICE", SURGERY_SERVICE),
            ("ICU_SERVICE", ICU_SERVICE),
            ("PHARMACY_SERVICE", PHARMACY_SERVICE),
            ("PANEL_A", PANEL_A),
            ("PANEL_B", PANEL_B),
            ("CT_SCANNER", CT_SCANNER),
            ("MRI", MRI),
            ("OPERATING_ROOM", OPERATING_ROOM),
            ("ICU_BEDS", ICU_BEDS),
            ("VENTILATOR", VENTILATOR),
            ("ER_A_VM", ER_A_VM),
            ("ER_A_FILE", ER_A_FILE),
            ("ER_A_DATA", ER_A_DATA),
            ("ER_B_VM", ER_B_VM),
            ("ER_B_FILE", ER_B_FILE),
            ("ER_B_DATA", ER_B_DATA),
            ("RADIOLOGY_VM", RADIOLOGY_VM),
            ("RADIOLOGY_FILE", RADIOLOGY_FILE),
            ("RADIOLOGY_DATA", RADIOLOGY_DATA),
            ("SURGERY_VM", SURGERY_VM),
            ("SURGERY_FILE", SURGERY_FILE),
            ("SURGERY_DATA", SURGERY_DATA),
            ("ANESTHESIA_VM", ANESTHESIA_VM),
            ("ANESTHESIA_FILE", ANESTHESIA_FILE),
            ("ANESTHESIA_DATA", ANESTHESIA_DATA),
            ("ICU_VM", ICU_VM),
            ("ICU_FILE", ICU_FILE),
            ("ICU_DATA", ICU_DATA),
            ("PHARMACY_VM", PHARMACY_VM),
            ("PHARMACY_FILE", PHARMACY_FILE),
            ("PHARMACY_DATA", PHARMACY_DATA),
        ]
        .into_iter()
        .map(|(name, role)| (name.into(), serde_json::json!(role)))
        .collect();
        let programs = [
            rows(&self.er_a),
            rows(&self.er_b),
            rows(&self.radiologist),
            rows(&self.surgeon),
            rows(&self.anesthesiologist),
            rows(&self.icu_coordinator),
            rows(&self.pharmacist),
            rows(&self.er_a_io),
            rows(&self.er_b_io),
            rows(&self.radiology_io),
            rows(&self.surgery_io),
            rows(&self.anesthesia_io),
            rows(&self.icu_io),
            rows(&self.pharmacy_io),
            rows(&self.patient_a),
            rows(&self.patient_b),
            rows(&self.ct_service),
            rows(&self.mri_service),
            rows(&self.anesthesia),
            rows(&self.surgery),
            rows(&self.icu),
            rows(&self.pharmacy),
            rows(&self.panel_a),
            rows(&self.panel_b),
            rows(&self.ct),
            rows(&self.mri),
            rows(&self.room),
            rows(&self.bed),
            rows(&self.ventilator),
            rows(&self.er_a_vm),
            rows(&self.er_a_file),
            rows(&self.er_a_data),
            rows(&self.er_b_vm),
            rows(&self.er_b_file),
            rows(&self.er_b_data),
            rows(&self.radiology_vm),
            rows(&self.radiology_file),
            rows(&self.radiology_data),
            rows(&self.surgery_vm),
            rows(&self.surgery_file),
            rows(&self.surgery_data),
            rows(&self.anesthesia_vm),
            rows(&self.anesthesia_file),
            rows(&self.anesthesia_data),
            rows(&self.icu_vm),
            rows(&self.icu_file),
            rows(&self.icu_data),
            rows(&self.pharmacy_vm),
            rows(&self.pharmacy_file),
            rows(&self.pharmacy_data),
        ];
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"5H","session":SESSION,"roles":roles,"labels":labels,"programs":programs.as_slice()})
    }
}
pub fn programs() -> Programs {
    let global = source();
    Programs {
        er_a: project::<ER_A>(&global),
        er_b: project::<ER_B>(&global),
        radiologist: project::<RADIOLOGIST>(&global),
        surgeon: project::<SURGEON>(&global),
        anesthesiologist: project::<ANESTHESIOLOGIST>(&global),
        icu_coordinator: project::<ICU_COORDINATOR>(&global),
        pharmacist: project::<PHARMACIST>(&global),
        er_a_io: project::<ER_A_IO>(&global),
        er_b_io: project::<ER_B_IO>(&global),
        radiology_io: project::<RADIOLOGY_IO>(&global),
        surgery_io: project::<SURGERY_IO>(&global),
        anesthesia_io: project::<ANESTHESIA_IO>(&global),
        icu_io: project::<ICU_IO>(&global),
        pharmacy_io: project::<PHARMACY_IO>(&global),
        patient_a: project::<PATIENT_A_SERVICE>(&global),
        patient_b: project::<PATIENT_B_SERVICE>(&global),
        ct_service: project::<CT_SERVICE>(&global),
        mri_service: project::<MRI_SERVICE>(&global),
        anesthesia: project::<ANESTHESIA_SERVICE>(&global),
        surgery: project::<SURGERY_SERVICE>(&global),
        icu: project::<ICU_SERVICE>(&global),
        pharmacy: project::<PHARMACY_SERVICE>(&global),
        panel_a: project::<PANEL_A>(&global),
        panel_b: project::<PANEL_B>(&global),
        ct: project::<CT_SCANNER>(&global),
        mri: project::<MRI>(&global),
        room: project::<OPERATING_ROOM>(&global),
        bed: project::<ICU_BEDS>(&global),
        ventilator: project::<VENTILATOR>(&global),
        er_a_vm: project::<ER_A_VM>(&global),
        er_a_file: project::<ER_A_FILE>(&global),
        er_a_data: project::<ER_A_DATA>(&global),
        er_b_vm: project::<ER_B_VM>(&global),
        er_b_file: project::<ER_B_FILE>(&global),
        er_b_data: project::<ER_B_DATA>(&global),
        radiology_vm: project::<RADIOLOGY_VM>(&global),
        radiology_file: project::<RADIOLOGY_FILE>(&global),
        radiology_data: project::<RADIOLOGY_DATA>(&global),
        surgery_vm: project::<SURGERY_VM>(&global),
        surgery_file: project::<SURGERY_FILE>(&global),
        surgery_data: project::<SURGERY_DATA>(&global),
        anesthesia_vm: project::<ANESTHESIA_VM>(&global),
        anesthesia_file: project::<ANESTHESIA_FILE>(&global),
        anesthesia_data: project::<ANESTHESIA_DATA>(&global),
        icu_vm: project::<ICU_VM>(&global),
        icu_file: project::<ICU_FILE>(&global),
        icu_data: project::<ICU_DATA>(&global),
        pharmacy_vm: project::<PHARMACY_VM>(&global),
        pharmacy_file: project::<PHARMACY_FILE>(&global),
        pharmacy_data: project::<PHARMACY_DATA>(&global),
    }
}
