//! 5H: two patient histories, one CT authority and independent surgery/ICU care.
use hibana::{
    Endpoint, g,
    runtime::program::{RoleProgram, project},
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
pub type TestRequest<'a> = g::Msg<4, &'a [u8]>;
pub type TestResult<'a> = g::Msg<5, &'a [u8]>;
pub type PanelPartial<'a> = g::Msg<6, &'a [u8]>;
pub type PanelComplete<'a> = g::Msg<7, &'a [u8]>;
pub type AssessmentRequestA<'a> = g::Msg<8, &'a [u8]>;
pub type AssessmentResultA<'a> = g::Msg<9, &'a [u8]>;
pub type AssessmentRequestB<'a> = g::Msg<10, &'a [u8]>;
pub type AssessmentResultB<'a> = g::Msg<11, &'a [u8]>;
pub type CtPerformIntentA<'a> = g::Msg<14, &'a [u8]>;
pub type CtInterpretIntentA<'a> = g::Msg<15, &'a [u8]>;
pub type CtPerformIntentB<'a> = g::Msg<16, &'a [u8]>;
pub type CtInterpretIntentB<'a> = g::Msg<17, &'a [u8]>;
pub type CtLoan<'r> = g::Msg<18, Endpoint<'r, CT_SCANNER>>;
pub type CtPerformRequestA<'a> = g::Msg<19, &'a [u8]>;
pub type CtInterpretRequestA<'a> = g::Msg<20, &'a [u8]>;
pub type CtPerformRequestB<'a> = g::Msg<21, &'a [u8]>;
pub type CtInterpretRequestB<'a> = g::Msg<22, &'a [u8]>;
pub type CtImageA<'a> = g::Msg<23, &'a [u8]>;
pub type CtReportA<'a> = g::Msg<24, &'a [u8]>;
pub type CtImageB<'a> = g::Msg<25, &'a [u8]>;
pub type CtReportB<'a> = g::Msg<26, &'a [u8]>;
pub type CtAssessmentQueryA<'a> = g::Msg<27, &'a [u8]>;
pub type CtAssessmentQueryB<'a> = g::Msg<28, &'a [u8]>;
pub type CtAssessmentA<'a> = g::Msg<29, &'a [u8]>;
pub type CtAssessmentB<'a> = g::Msg<30, &'a [u8]>;
pub type CtFindingsA<'a> = g::Msg<31, &'a [u8]>;
pub type CtFindingsB<'a> = g::Msg<32, &'a [u8]>;
pub type MriPerformIntent<'a> = g::Msg<33, &'a [u8]>;
pub type MriInterpretIntent<'a> = g::Msg<34, &'a [u8]>;
pub type MriLoan<'r> = g::Msg<35, Endpoint<'r, MRI>>;
pub type MriPerformRequest<'a> = g::Msg<36, &'a [u8]>;
pub type MriInterpretRequest<'a> = g::Msg<37, &'a [u8]>;
pub type MriImage<'a> = g::Msg<38, &'a [u8]>;
pub type MriReport<'a> = g::Msg<39, &'a [u8]>;
pub type MriAssessmentQuery<'a> = g::Msg<40, &'a [u8]>;
pub type MriAssessment<'a> = g::Msg<41, &'a [u8]>;
pub type MriFindingsB<'a> = g::Msg<42, &'a [u8]>;
pub type FindingsA<'a> = g::Msg<43, &'a [u8]>;
pub type FindingsB<'a> = g::Msg<44, &'a [u8]>;
pub type DiagnosisRequestA<'a> = g::Msg<45, &'a [u8]>;
pub type DiagnosisResultA<'a> = g::Msg<46, &'a [u8]>;
pub type DiagnosisRequestB<'a> = g::Msg<47, &'a [u8]>;
pub type DiagnosisResultB<'a> = g::Msg<48, &'a [u8]>;
pub type DiagnosisForAnesthesia<'a> = g::Msg<49, &'a [u8]>;
pub type DiagnosisForSurgery<'a> = g::Msg<50, &'a [u8]>;
pub type DiagnosisForIcu<'a> = g::Msg<51, &'a [u8]>;
pub type AnesthesiaIntent<'a> = g::Msg<52, &'a [u8]>;
pub type AnesthesiaLoan<'r> = g::Msg<53, Endpoint<'r, OPERATING_ROOM>>;
pub type AnesthesiaRequest<'a> = g::Msg<54, &'a [u8]>;
pub type AnesthesiaResult<'a> = g::Msg<55, &'a [u8]>;
pub type PreparationForSurgery<'a> = g::Msg<56, &'a [u8]>;
pub type PreparedRoom<'r> = g::Msg<57, Endpoint<'r, OPERATING_ROOM>>;
pub type SurgeryIntent<'a> = g::Msg<58, &'a [u8]>;
pub type SurgeryLoan<'r> = g::Msg<59, Endpoint<'r, OPERATING_ROOM>>;
pub type SurgeryRequest<'a> = g::Msg<60, &'a [u8]>;
pub type SurgeryResult<'a> = g::Msg<61, &'a [u8]>;
pub type SurgeryForER<'a> = g::Msg<62, &'a [u8]>;
pub type IcuIntent<'a> = g::Msg<63, &'a [u8]>;
pub type ResourceLoan<'r, const RESOURCE: u8> = g::Msg<64, Endpoint<'r, RESOURCE>>;
pub type ResourceReturn<'r, const RESOURCE: u8> = g::Msg<65, Endpoint<'r, RESOURCE>>;
pub type BedRequest<'a> = g::Msg<66, &'a [u8]>;
pub type VentilatorRequest<'a> = g::Msg<67, &'a [u8]>;
pub type BedResult<'a> = g::Msg<68, &'a [u8]>;
pub type VentilatorResult<'a> = g::Msg<69, &'a [u8]>;
pub type IcuForER<'a> = g::Msg<70, &'a [u8]>;
pub type ReviewRequest<'a> = g::Msg<71, &'a [u8]>;
pub type ReviewResult<'a> = g::Msg<72, &'a [u8]>;
pub type ReviewForA<'a> = g::Msg<73, &'a [u8]>;
pub type ReviewForB<'a> = g::Msg<74, &'a [u8]>;

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
}
impl Programs {
    pub fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        use hibana::g::Message;
        let labels: serde_json::Map<String, serde_json::Value> = [
            ("PanelLoan", PanelLoan::<PANEL_A>::LOGICAL_LABEL),
            ("PanelReturn", PanelReturn::<PANEL_A>::LOGICAL_LABEL),
            ("PanelCompleted", PanelCompleted::<PANEL_A>::LOGICAL_LABEL),
            ("TestRequest", TestRequest::LOGICAL_LABEL),
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
        ]
        .into_iter()
        .map(|(name, label)| (name.into(), label.into()))
        .collect();
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"5H","session":SESSION,
            "roles":{"ER_DOCTOR_A":ER_A,"ER_DOCTOR_B":ER_B,"RADIOLOGIST":RADIOLOGIST,"SURGEON":SURGEON,"ANESTHESIOLOGIST":ANESTHESIOLOGIST,"ICU_COORDINATOR":ICU_COORDINATOR,"PHARMACIST":PHARMACIST,"ER_A_IO":ER_A_IO,"ER_B_IO":ER_B_IO,"RADIOLOGY_IO":RADIOLOGY_IO,"SURGERY_IO":SURGERY_IO,"ANESTHESIA_IO":ANESTHESIA_IO,"ICU_IO":ICU_IO,"PHARMACY_IO":PHARMACY_IO,"PATIENT_A_SERVICE":PATIENT_A_SERVICE,"PATIENT_B_SERVICE":PATIENT_B_SERVICE,"CT_SERVICE":CT_SERVICE,"MRI_SERVICE":MRI_SERVICE,"ANESTHESIA_SERVICE":ANESTHESIA_SERVICE,"SURGERY_SERVICE":SURGERY_SERVICE,"ICU_SERVICE":ICU_SERVICE,"PHARMACY_SERVICE":PHARMACY_SERVICE,"PANEL_A":PANEL_A,"PANEL_B":PANEL_B,"CT_SCANNER":CT_SCANNER,"MRI":MRI,"OPERATING_ROOM":OPERATING_ROOM,"ICU_BEDS":ICU_BEDS,"VENTILATOR":VENTILATOR},
            "labels":labels,"programs":[rows(&self.er_a),rows(&self.er_b),rows(&self.radiologist),rows(&self.surgeon),rows(&self.anesthesiologist),rows(&self.icu_coordinator),rows(&self.pharmacist),rows(&self.er_a_io),rows(&self.er_b_io),rows(&self.radiology_io),rows(&self.surgery_io),rows(&self.anesthesia_io),rows(&self.icu_io),rows(&self.pharmacy_io),rows(&self.patient_a),rows(&self.patient_b),rows(&self.ct_service),rows(&self.mri_service),rows(&self.anesthesia),rows(&self.surgery),rows(&self.icu),rows(&self.pharmacy),rows(&self.panel_a),rows(&self.panel_b),rows(&self.ct),rows(&self.mri),rows(&self.room),rows(&self.bed),rows(&self.ventilator)]})
    }
}

pub fn programs() -> Programs {
    let panel_a_attempt = || {
        g::seq(
            g::send::<PATIENT_A_SERVICE, ER_A, PanelLoan<PANEL_A>>(),
            g::seq(
                g::send::<PANEL_A, PATIENT_A_SERVICE, TestRequest>(),
                g::seq(
                    g::send::<PATIENT_A_SERVICE, PANEL_A, TestResult>(),
                    g::send::<ER_A, PATIENT_A_SERVICE, PanelReturn<PANEL_A>>(),
                ),
            ),
        )
    };
    let panel_a_complete = || {
        g::seq(
            g::send::<PATIENT_A_SERVICE, ER_A, PanelCompleted<PANEL_A>>(),
            g::send::<PATIENT_A_SERVICE, ER_A, PanelComplete>(),
        )
    };
    let panel_a = g::seq(
        panel_a_attempt(),
        g::route(
            panel_a_complete(),
            g::seq(
                g::seq(
                    g::send::<PATIENT_A_SERVICE, ER_A, PanelPartial>(),
                    panel_a_attempt(),
                )
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
                g::send::<PANEL_B, PATIENT_B_SERVICE, TestRequest>(),
                g::seq(
                    g::send::<PATIENT_B_SERVICE, PANEL_B, TestResult>(),
                    g::send::<ER_B, PATIENT_B_SERVICE, PanelReturn<PANEL_B>>(),
                ),
            ),
        )
    };
    let panel_b_complete = || {
        g::seq(
            g::send::<PATIENT_B_SERVICE, ER_B, PanelCompleted<PANEL_B>>(),
            g::send::<PATIENT_B_SERVICE, ER_B, PanelComplete>(),
        )
    };
    let panel_b = g::seq(
        panel_b_attempt(),
        g::route(
            panel_b_complete(),
            g::seq(
                g::seq(
                    g::send::<PATIENT_B_SERVICE, ER_B, PanelPartial>(),
                    panel_b_attempt(),
                )
                .roll(),
                panel_b_complete(),
            ),
        )
        .resolve::<PANEL_B_COMPLETE>(),
    );
    let assessment = g::par(
        g::seq(
            g::send::<ER_A, PATIENT_A_SERVICE, AssessmentRequestA>(),
            g::send::<PATIENT_A_SERVICE, ER_A, AssessmentResultA>(),
        ),
        g::seq(
            g::send::<ER_B, PATIENT_B_SERVICE, AssessmentRequestB>(),
            g::send::<PATIENT_B_SERVICE, ER_B, AssessmentResultB>(),
        ),
    );

    // Each choice starts with the actual caller's request, not an extra grant.
    // The query retrieves the current assessment only for that patient.
    let ct_a_perform = || {
        g::seq(
            g::send::<RADIOLOGIST, CT_SERVICE, CtPerformIntentA>(),
            g::seq(
                g::send::<CT_SERVICE, PATIENT_A_SERVICE, CtAssessmentQueryA>(),
                g::seq(
                    g::send::<PATIENT_A_SERVICE, CT_SERVICE, CtAssessmentA>(),
                    g::seq(
                        g::send::<CT_SERVICE, RADIOLOGIST, CtLoan>(),
                        g::seq(
                            g::send::<CT_SCANNER, CT_SERVICE, CtPerformRequestA>(),
                            g::seq(
                                g::send::<CT_SERVICE, CT_SCANNER, CtImageA>(),
                                g::send::<RADIOLOGIST, CT_SERVICE, CtLoan>(),
                            ),
                        ),
                    ),
                ),
            ),
        )
    };
    let ct_a_interpret = || {
        g::seq(
            g::send::<RADIOLOGIST, CT_SERVICE, CtInterpretIntentA>(),
            g::seq(
                g::send::<CT_SERVICE, RADIOLOGIST, CtLoan>(),
                g::seq(
                    g::send::<CT_SCANNER, CT_SERVICE, CtInterpretRequestA>(),
                    g::seq(
                        g::send::<CT_SERVICE, CT_SCANNER, CtReportA>(),
                        g::seq(
                            g::send::<RADIOLOGIST, CT_SERVICE, CtLoan>(),
                            g::send::<CT_SERVICE, PATIENT_A_SERVICE, CtFindingsA>(),
                        ),
                    ),
                ),
            ),
        )
    };
    let ct_b_perform = || {
        g::seq(
            g::send::<RADIOLOGIST, CT_SERVICE, CtPerformIntentB>(),
            g::seq(
                g::send::<CT_SERVICE, PATIENT_B_SERVICE, CtAssessmentQueryB>(),
                g::seq(
                    g::send::<PATIENT_B_SERVICE, CT_SERVICE, CtAssessmentB>(),
                    g::seq(
                        g::send::<CT_SERVICE, RADIOLOGIST, CtLoan>(),
                        g::seq(
                            g::send::<CT_SCANNER, CT_SERVICE, CtPerformRequestB>(),
                            g::seq(
                                g::send::<CT_SERVICE, CT_SCANNER, CtImageB>(),
                                g::send::<RADIOLOGIST, CT_SERVICE, CtLoan>(),
                            ),
                        ),
                    ),
                ),
            ),
        )
    };
    let ct_b_interpret = || {
        g::seq(
            g::send::<RADIOLOGIST, CT_SERVICE, CtInterpretIntentB>(),
            g::seq(
                g::send::<CT_SERVICE, RADIOLOGIST, CtLoan>(),
                g::seq(
                    g::send::<CT_SCANNER, CT_SERVICE, CtInterpretRequestB>(),
                    g::seq(
                        g::send::<CT_SERVICE, CT_SCANNER, CtReportB>(),
                        g::seq(
                            g::send::<RADIOLOGIST, CT_SERVICE, CtLoan>(),
                            g::send::<CT_SERVICE, PATIENT_B_SERVICE, CtFindingsB>(),
                        ),
                    ),
                ),
            ),
        )
    };
    // All six interleavings of A's and B's perform -> interpret pairs.
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
    let mri_perform = g::seq(
        g::send::<RADIOLOGIST, MRI_SERVICE, MriPerformIntent>(),
        g::seq(
            g::send::<MRI_SERVICE, PATIENT_B_SERVICE, MriAssessmentQuery>(),
            g::seq(
                g::send::<PATIENT_B_SERVICE, MRI_SERVICE, MriAssessment>(),
                g::seq(
                    g::send::<MRI_SERVICE, RADIOLOGIST, MriLoan>(),
                    g::seq(
                        g::send::<MRI, MRI_SERVICE, MriPerformRequest>(),
                        g::seq(
                            g::send::<MRI_SERVICE, MRI, MriImage>(),
                            g::send::<RADIOLOGIST, MRI_SERVICE, MriLoan>(),
                        ),
                    ),
                ),
            ),
        ),
    );
    let mri_interpret = g::seq(
        g::send::<RADIOLOGIST, MRI_SERVICE, MriInterpretIntent>(),
        g::seq(
            g::send::<MRI_SERVICE, RADIOLOGIST, MriLoan>(),
            g::seq(
                g::send::<MRI, MRI_SERVICE, MriInterpretRequest>(),
                g::seq(
                    g::send::<MRI_SERVICE, MRI, MriReport>(),
                    g::seq(
                        g::send::<RADIOLOGIST, MRI_SERVICE, MriLoan>(),
                        g::send::<MRI_SERVICE, PATIENT_B_SERVICE, MriFindingsB>(),
                    ),
                ),
            ),
        ),
    );
    let mri = g::seq(mri_perform, mri_interpret);
    let workup = g::par(panel_a, g::par(panel_b, g::par(ct, mri)));
    let diagnosis = g::par(
        g::seq(
            g::send::<PATIENT_A_SERVICE, ER_A, FindingsA>(),
            g::seq(
                g::send::<ER_A, PATIENT_A_SERVICE, DiagnosisRequestA>(),
                g::seq(
                    g::send::<PATIENT_A_SERVICE, ER_A, DiagnosisResultA>(),
                    g::par(
                        g::send::<PATIENT_A_SERVICE, ANESTHESIA_SERVICE, DiagnosisForAnesthesia>(),
                        g::send::<PATIENT_A_SERVICE, SURGERY_SERVICE, DiagnosisForSurgery>(),
                    ),
                ),
            ),
        ),
        g::seq(
            g::send::<PATIENT_B_SERVICE, ER_B, FindingsB>(),
            g::seq(
                g::send::<ER_B, PATIENT_B_SERVICE, DiagnosisRequestB>(),
                g::seq(
                    g::send::<PATIENT_B_SERVICE, ER_B, DiagnosisResultB>(),
                    g::send::<PATIENT_B_SERVICE, ICU_SERVICE, DiagnosisForIcu>(),
                ),
            ),
        ),
    );
    let anesthesia = g::seq(
        g::send::<ANESTHESIOLOGIST, ANESTHESIA_SERVICE, AnesthesiaIntent>(),
        g::seq(
            g::send::<ANESTHESIA_SERVICE, ANESTHESIOLOGIST, AnesthesiaLoan>(),
            g::seq(
                g::send::<OPERATING_ROOM, ANESTHESIA_SERVICE, AnesthesiaRequest>(),
                g::seq(
                    g::send::<ANESTHESIA_SERVICE, OPERATING_ROOM, AnesthesiaResult>(),
                    g::seq(
                        g::send::<ANESTHESIOLOGIST, ANESTHESIA_SERVICE, AnesthesiaLoan>(),
                        g::par(
                            g::send::<ANESTHESIA_SERVICE, SURGERY_SERVICE, PreparedRoom>(),
                            g::send::<ANESTHESIA_SERVICE, SURGERY_SERVICE, PreparationForSurgery>(),
                        ),
                    ),
                ),
            ),
        ),
    );
    let surgery = g::seq(
        g::send::<SURGEON, SURGERY_SERVICE, SurgeryIntent>(),
        g::seq(
            g::send::<SURGERY_SERVICE, SURGEON, SurgeryLoan>(),
            g::seq(
                g::send::<OPERATING_ROOM, SURGERY_SERVICE, SurgeryRequest>(),
                g::seq(
                    g::send::<SURGERY_SERVICE, OPERATING_ROOM, SurgeryResult>(),
                    g::seq(
                        g::send::<SURGEON, SURGERY_SERVICE, SurgeryLoan>(),
                        g::send::<SURGERY_SERVICE, ER_A, SurgeryForER>(),
                    ),
                ),
            ),
        ),
    );
    let icu = g::seq(
        g::send::<ICU_COORDINATOR, ICU_SERVICE, IcuIntent>(),
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
                        g::send::<ICU_SERVICE, VENTILATOR, VentilatorResult>(),
                    ),
                    g::seq(
                        g::par(
                            g::send::<ICU_COORDINATOR, ICU_SERVICE, ResourceReturn<ICU_BEDS>>(),
                            g::send::<ICU_COORDINATOR, ICU_SERVICE, ResourceReturn<VENTILATOR>>(),
                        ),
                        g::send::<ICU_SERVICE, ER_B, IcuForER>(),
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
        g::send::<PHARMACIST, PHARMACY_SERVICE, ReviewRequest>(),
        g::seq(
            g::send::<PHARMACY_SERVICE, PHARMACIST, ReviewResult>(),
            g::par(
                g::send::<PHARMACY_SERVICE, ER_A, ReviewForA>(),
                g::send::<PHARMACY_SERVICE, ER_B, ReviewForB>(),
            ),
        ),
    );
    let business = g::par(clinical, review);
    let private_io = g::par(
        crate::shell::global::private_io::<ER_A, ER_A_IO>(),
        g::par(
            crate::shell::global::private_io::<ER_B, ER_B_IO>(),
            g::par(
                crate::shell::global::private_io::<RADIOLOGIST, RADIOLOGY_IO>(),
                g::par(
                    crate::shell::global::private_io::<SURGEON, SURGERY_IO>(),
                    g::par(
                        crate::shell::global::private_io::<ANESTHESIOLOGIST, ANESTHESIA_IO>(),
                        g::par(
                            crate::shell::global::private_io::<ICU_COORDINATOR, ICU_IO>(),
                            crate::shell::global::private_io::<PHARMACIST, PHARMACY_IO>(),
                        ),
                    ),
                ),
            ),
        ),
    );
    let global = g::par(business, private_io);
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
    }
}
