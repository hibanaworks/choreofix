//! 14H: actual material, finite samples, repeated review and affine study custody.
use super::imp;
use hibana::{Endpoint, g, runtime::program::Projectable};

pub const CHEMIST: u8 = 0;
pub const BIOLOGIST: u8 = 1;
pub const TOXICOLOGIST: u8 = 2;
pub const FORMULATOR: u8 = 3;
pub const CLINICIAN: u8 = 4;
pub const REGULATOR: u8 = 5;
pub const DIRECTOR: u8 = 6;
pub const HPLC_SERVICE: u8 = 14;
pub const MASS_SERVICE: u8 = 15;
pub const CELL_SERVICE: u8 = 16;
pub const FORM_SERVICE: u8 = 17;
pub const STUDY_SERVICE: u8 = 18;
pub const HPLC: u8 = 19;
pub const MASS: u8 = 20;
pub const CELL: u8 = 21;
pub const FORM: u8 = 22;
pub const SAMPLES: u8 = 23;
pub const QC: u8 = 24;
pub const CULTURE: u8 = 25;
pub const FORMULATION: u8 = 26;
pub const BIOASSAY: u8 = 27;
pub const STABILITY: u8 = 28;
pub const PURITY: u8 = 29;
pub const TOXICITY: u8 = 30;
pub const STUDY: u8 = 31;

pub type Intent<'a, const STEP: u8> = g::Msg<1, imp::AnalysisArguments<'a, STEP>>;
pub type Request<'a, const STEP: u8> = g::Msg<2, imp::AnalysisArguments<'a, STEP>>;
pub type Passed<'a, const STEP: u8> = g::Msg<3, imp::AnalysisReply<'a, STEP, true>>;
pub type Failed<'a, const STEP: u8> = g::Msg<4, imp::AnalysisReply<'a, STEP, false>>;
pub type Instrument<'r, const ROLE: u8> = g::Msg<5, Endpoint<'r, ROLE>>;
pub type Specimen<'r, const ROLE: u8> = g::Msg<6, Endpoint<'r, ROLE>>;
pub type Archived<'r, const ROLE: u8> = g::Msg<7, Endpoint<'r, ROLE>>;
pub type SampleRequest<'a, const STEP: u8> = g::Msg<8, imp::AnalysisArguments<'a, STEP>>;
pub type DeepIntent<'a, const TOOL: u8> = g::Msg<9, imp::JsonTo<'a, TOOL>>;
pub type Identity<'a, const DEST: u8> = g::Msg<60, imp::JsonTo<'a, DEST>>;
pub type Formulation<'a, const DEST: u8> = g::Msg<61, imp::JsonTo<'a, DEST>>;
pub type Cells<'a, const DEST: u8> = g::Msg<62, imp::JsonTo<'a, DEST>>;
pub type Baseline<'a, const DEST: u8> = g::Msg<63, imp::JsonTo<'a, DEST>>;
pub type BioEvidence<'a, const DEST: u8> = g::Msg<64, imp::JsonTo<'a, DEST>>;
pub type PurityEvidence<'a, const DEST: u8> = g::Msg<65, imp::JsonTo<'a, DEST>>;
pub type StabilityEvidence<'a, const DEST: u8> = g::Msg<66, imp::JsonTo<'a, DEST>>;
pub type ToxEvidence<'a, const DEST: u8> = g::Msg<67, imp::JsonTo<'a, DEST>>;
pub type ClinicalEvidence<'a, const DEST: u8> = g::Msg<68, imp::JsonTo<'a, DEST>>;
pub type RegulatoryEvidence<'a, const DEST: u8> = g::Msg<69, imp::JsonTo<'a, DEST>>;
pub type DecisionEvidence<'a, const DEST: u8> = g::Msg<70, imp::JsonTo<'a, DEST>>;
pub type QcEvidence<'a> = g::Msg<71, imp::JsonTo<'a, MASS_SERVICE>>;
pub type DecisionRequest<'a> = g::Msg<72, &'a [u8]>;
pub type Final<'a, const DEST: u8> = g::Msg<181, imp::FinalOutcome<'a, DEST>>;
pub type RoutineReview<'a> = g::Msg<73, imp::AnalysisArguments<'a, 10>>;
pub type DeepReview<'a> = g::Msg<74, imp::AnalysisArguments<'a, 11>>;

pub(super) fn trial<
    const WHO: u8,
    const SERVICE: u8,
    const TOOL: u8,
    const CAP: u8,
    const STEP: u8,
>() -> impl Projectable {
    g::seq(
        g::send::<CAP, SERVICE, Intent<STEP>>(),
        requested::<WHO, SERVICE, TOOL, CAP, STEP>(),
    )
}
fn requested<const WHO: u8, const SERVICE: u8, const TOOL: u8, const CAP: u8, const STEP: u8>()
-> impl Projectable {
    g::seq(
        g::send::<SERVICE, WHO, Instrument<TOOL>>(),
        g::seq(
            g::send::<TOOL, SERVICE, Request<STEP>>(),
            g::seq(
                g::send::<WHO, SERVICE, Instrument<TOOL>>(),
                g::send::<WHO, SERVICE, Specimen<CAP>>(),
            ),
        ),
    )
}
pub(super) fn passed<
    const WHO: u8,
    const SERVICE: u8,
    const CAP: u8,
    const STEP: u8,
    const DEST: u8,
>() -> impl Projectable {
    g::seq(
        g::send::<CAP, WHO, Passed<STEP>>(),
        g::send::<SERVICE, DEST, Specimen<CAP>>(),
    )
}
pub(super) fn positive<
    const WHO: u8,
    const SERVICE: u8,
    const TOOL: u8,
    const CAP: u8,
    const STEP: u8,
    const DEST: u8,
>() -> impl Projectable {
    g::seq(
        trial::<WHO, SERVICE, TOOL, CAP, STEP>(),
        positive_result::<WHO, SERVICE, TOOL, CAP, STEP, DEST>(),
    )
}
fn positive_result<
    const WHO: u8,
    const SERVICE: u8,
    const TOOL: u8,
    const CAP: u8,
    const STEP: u8,
    const DEST: u8,
>() -> impl Projectable {
    let retry = g::seq(
        g::send::<CAP, WHO, Failed<STEP>>(),
        g::seq(
            g::send::<SERVICE, WHO, Specimen<CAP>>(),
            trial::<WHO, SERVICE, TOOL, CAP, STEP>(),
        ),
    );
    g::route(
        passed::<WHO, SERVICE, CAP, STEP, DEST>(),
        g::seq(retry.roll(), passed::<WHO, SERVICE, CAP, STEP, DEST>()),
    )
}

pub(super) fn mass<const WHO: u8, const STEP: u8>() -> impl Projectable {
    g::seq(
        g::send::<WHO, MASS_SERVICE, Intent<STEP>>(),
        g::seq(
            g::send::<MASS_SERVICE, WHO, Instrument<MASS>>(),
            g::seq(
                g::send::<MASS, MASS_SERVICE, Request<STEP>>(),
                g::seq(
                    g::send::<MASS_SERVICE, MASS, Passed<STEP>>(),
                    g::send::<WHO, MASS_SERVICE, Instrument<MASS>>(),
                ),
            ),
        ),
    )
}

/// Reconsider the same material without manufacturing another sample. The
/// scientist retains its specimen between attempts and returns it to finish;
/// only that final transfer releases the result to clinical assessment.
pub(super) fn assay<const WHO: u8, const CAP: u8, const STEP: u8>() -> impl Projectable {
    let attempt = g::seq(
        g::send::<CAP, CELL_SERVICE, Intent<STEP>>(),
        g::seq(
            g::par(
                g::send::<CELL_SERVICE, WHO, Instrument<CELL>>(),
                g::send::<CELL_SERVICE, WHO, Instrument<SAMPLES>>(),
            ),
            g::seq(
                g::par(
                    g::send::<CELL, CELL_SERVICE, Request<STEP>>(),
                    g::send::<SAMPLES, CELL_SERVICE, SampleRequest<STEP>>(),
                ),
                g::seq(
                    g::par(
                        g::send::<WHO, CELL_SERVICE, Instrument<SAMPLES>>(),
                        g::send::<WHO, CELL_SERVICE, Instrument<CELL>>(),
                    ),
                    g::seq(
                        g::send::<WHO, CELL_SERVICE, Specimen<CAP>>(),
                        g::seq(
                            g::route(
                                g::send::<CAP, WHO, Passed<STEP>>(),
                                g::send::<CAP, WHO, Failed<STEP>>(),
                            ),
                            g::send::<CELL_SERVICE, WHO, Specimen<CAP>>(),
                        ),
                    ),
                ),
            ),
        ),
    );
    g::seq(
        attempt.roll(),
        g::seq(
            g::send::<WHO, CELL_SERVICE, Specimen<CAP>>(),
            g::send::<CELL_SERVICE, CLINICIAN, Specimen<CAP>>(),
        ),
    )
}

pub(super) fn formulation() -> impl Projectable {
    g::seq(
        g::seq(
            requested::<FORMULATOR, FORM_SERVICE, FORM, FORMULATION, 4>(),
            positive_result::<FORMULATOR, FORM_SERVICE, FORM, FORMULATION, 4, CLINICIAN>(),
        ),
        g::par(
            g::send::<FORM_SERVICE, HPLC_SERVICE, Formulation<HPLC_SERVICE>>(),
            g::par(
                g::send::<FORM_SERVICE, CELL_SERVICE, Formulation<CELL_SERVICE>>(),
                g::par(
                    g::send::<FORM_SERVICE, MASS_SERVICE, Formulation<MASS_SERVICE>>(),
                    g::send::<FORM_SERVICE, STUDY_SERVICE, Formulation<STUDY_SERVICE>>(),
                ),
            ),
        ),
    )
}

fn evaluation() -> impl Projectable {
    let bio = g::seq(
        assay::<BIOLOGIST, BIOASSAY, 5>(),
        g::send::<CELL_SERVICE, MASS_SERVICE, BioEvidence<MASS_SERVICE>>(),
    );
    let purity = g::seq(
        positive::<TOXICOLOGIST, HPLC_SERVICE, HPLC, PURITY, 7, CLINICIAN>(),
        g::send::<HPLC_SERVICE, MASS_SERVICE, PurityEvidence<MASS_SERVICE>>(),
    );
    let stability = g::seq(
        positive::<BIOLOGIST, HPLC_SERVICE, HPLC, STABILITY, 6, CLINICIAN>(),
        g::send::<HPLC_SERVICE, MASS_SERVICE, StabilityEvidence<MASS_SERVICE>>(),
    );
    let tox = g::seq(
        g::send::<TOXICOLOGIST, CELL_SERVICE, PurityEvidence<CELL_SERVICE>>(),
        g::seq(
            assay::<TOXICOLOGIST, TOXICITY, 8>(),
            g::send::<CELL_SERVICE, MASS_SERVICE, ToxEvidence<MASS_SERVICE>>(),
        ),
    );
    // The toxicologist forwards its actual purity result before toxicity.
    // Actors absent from the purity prefix may start their independent work:
    // BIOLOGIST's assay does not wait for purity, and stability does not join it.
    g::seq(purity, g::par(g::seq(bio, tox), stability))
}

fn clinical() -> impl Projectable {
    let inputs = g::par(
        g::send::<CLINICIAN, DIRECTOR, Specimen<FORMULATION>>(),
        g::par(
            g::send::<CLINICIAN, DIRECTOR, Specimen<BIOASSAY>>(),
            g::par(
                g::send::<CLINICIAN, DIRECTOR, Specimen<STABILITY>>(),
                g::par(
                    g::send::<CLINICIAN, DIRECTOR, Specimen<PURITY>>(),
                    g::send::<CLINICIAN, DIRECTOR, Specimen<TOXICITY>>(),
                ),
            ),
        ),
    );
    g::seq(
        trial::<CLINICIAN, MASS_SERVICE, MASS, STUDY, 9>(),
        g::seq(
            passed::<CLINICIAN, MASS_SERVICE, STUDY, 9, REGULATOR>(),
            g::par(
                inputs,
                g::par(
                    g::send::<MASS_SERVICE, REGULATOR, ClinicalEvidence<REGULATOR>>(),
                    g::send::<MASS_SERVICE, STUDY_SERVICE, ClinicalEvidence<STUDY_SERVICE>>(),
                ),
            ),
        ),
    )
}

pub(super) fn review() -> impl Projectable {
    let routine = g::seq(
        g::send::<STUDY, STUDY_SERVICE, RoutineReview>(),
        g::seq(
            g::send::<REGULATOR, STUDY_SERVICE, Specimen<STUDY>>(),
            g::seq(
                g::route(
                    g::send::<STUDY, REGULATOR, Passed<10>>(),
                    g::send::<STUDY, REGULATOR, Failed<10>>(),
                ),
                g::send::<STUDY_SERVICE, REGULATOR, Specimen<STUDY>>(),
            ),
        ),
    );
    // Both physical instrument owners receive the actual review plan. Routine
    // review leaves their instruments in place; deep review below borrows them.
    // Their next work must not depend on guessing which plan the regulator ran.
    let initial = g::seq(
        g::send::<STUDY, HPLC_SERVICE, RoutineReview>(),
        g::seq(g::send::<STUDY, MASS_SERVICE, RoutineReview>(), routine),
    );
    let acquired = g::seq(
        g::send::<STUDY, HPLC_SERVICE, DeepIntent<HPLC>>(),
        g::seq(
            g::send::<HPLC_SERVICE, REGULATOR, Instrument<HPLC>>(),
            g::seq(
                g::send::<STUDY, MASS_SERVICE, DeepIntent<MASS>>(),
                g::send::<MASS_SERVICE, REGULATOR, Instrument<MASS>>(),
            ),
        ),
    );
    let released = g::seq(
        g::send::<REGULATOR, MASS_SERVICE, Instrument<MASS>>(),
        g::send::<REGULATOR, HPLC_SERVICE, Instrument<HPLC>>(),
    );
    // The effect receiver owns both physical instruments while reviewing.
    // A received request alone does not retain an instrument after its return.
    let reviewing = g::seq(
        g::par(
            g::send::<REGULATOR, STUDY_SERVICE, Instrument<HPLC>>(),
            g::send::<REGULATOR, STUDY_SERVICE, Instrument<MASS>>(),
        ),
        g::seq(
            g::send::<REGULATOR, STUDY_SERVICE, Specimen<STUDY>>(),
            g::seq(
                g::route(
                    g::send::<STUDY, REGULATOR, Passed<11>>(),
                    g::send::<STUDY, REGULATOR, Failed<11>>(),
                ),
                g::seq(
                    g::send::<STUDY_SERVICE, REGULATOR, Specimen<STUDY>>(),
                    g::seq(
                        g::par(
                            g::send::<STUDY_SERVICE, REGULATOR, Instrument<HPLC>>(),
                            g::send::<STUDY_SERVICE, REGULATOR, Instrument<MASS>>(),
                        ),
                        released,
                    ),
                ),
            ),
        ),
    );
    let deep = g::seq(
        acquired,
        g::seq(
            g::send::<STUDY, STUDY_SERVICE, DeepReview>(),
            g::seq(
                g::par(
                    g::send::<HPLC, STUDY_SERVICE, Request<11>>(),
                    g::send::<MASS, STUDY_SERVICE, Request<11>>(),
                ),
                reviewing,
            ),
        ),
    );
    g::route(initial, deep)
}

fn round_entry() -> impl Projectable {
    g::seq(
        g::send::<STUDY, FORMULATOR, Specimen<FORMULATION>>(),
        g::send::<FORMULATION, FORM_SERVICE, Intent<4>>(),
    )
}

/// The effect owner receives its actual formulation before releasing the
/// material's Endpoint. Regaining a material must not authorize an old inner
/// assay while its next formulation is still being prepared.
pub(super) fn material<const WHO: u8, const SERVICE: u8, const CAP: u8>() -> impl Projectable {
    g::seq(
        g::send::<DIRECTOR, SERVICE, Specimen<CAP>>(),
        g::send::<SERVICE, WHO, Specimen<CAP>>(),
    )
}

fn round() -> impl Projectable {
    let reviewed = g::seq(
        review().roll(),
        g::seq(
            g::send::<REGULATOR, STUDY_SERVICE, Specimen<STUDY>>(),
            g::send::<STUDY, DIRECTOR, RegulatoryEvidence<DIRECTOR>>(),
        ),
    );
    // The same study effect owner holds clinical evidence and its real review
    // result. No second owner or copied authority is needed for the decision.
    let decision = g::seq(
        g::send::<DIRECTOR, STUDY_SERVICE, DecisionRequest>(),
        g::seq(
            g::send::<STUDY, DIRECTOR, DecisionEvidence<DIRECTOR>>(),
            g::send::<STUDY_SERVICE, DIRECTOR, Specimen<STUDY>>(),
        ),
    );
    let materials = g::par(
        material::<BIOLOGIST, CELL_SERVICE, BIOASSAY>(),
        g::par(
            material::<BIOLOGIST, HPLC_SERVICE, STABILITY>(),
            g::par(
                material::<TOXICOLOGIST, HPLC_SERVICE, PURITY>(),
                material::<TOXICOLOGIST, CELL_SERVICE, TOXICITY>(),
            ),
        ),
    );
    g::seq(
        formulation(),
        g::seq(
            materials,
            g::seq(
                g::send::<DIRECTOR, CLINICIAN, Specimen<STUDY>>(),
                g::seq(evaluation(), g::seq(clinical(), g::seq(reviewed, decision))),
            ),
        ),
    )
}

pub(super) fn terminal() -> impl Projectable {
    let archived = g::par(
        g::send::<DIRECTOR, HPLC_SERVICE, Archived<FORMULATION>>(),
        g::par(
            g::send::<DIRECTOR, HPLC_SERVICE, Archived<BIOASSAY>>(),
            g::par(
                g::send::<DIRECTOR, HPLC_SERVICE, Archived<STABILITY>>(),
                g::par(
                    g::send::<DIRECTOR, HPLC_SERVICE, Archived<PURITY>>(),
                    g::send::<DIRECTOR, HPLC_SERVICE, Archived<TOXICITY>>(),
                ),
            ),
        ),
    );
    let outcome = g::par(
        g::send::<DIRECTOR, CHEMIST, Final<CHEMIST>>(),
        g::par(
            g::send::<DIRECTOR, BIOLOGIST, Final<BIOLOGIST>>(),
            g::par(
                g::send::<DIRECTOR, TOXICOLOGIST, Final<TOXICOLOGIST>>(),
                g::par(
                    g::send::<DIRECTOR, FORMULATOR, Final<FORMULATOR>>(),
                    g::par(
                        g::send::<DIRECTOR, CLINICIAN, Final<CLINICIAN>>(),
                        g::par(
                            g::send::<DIRECTOR, REGULATOR, Final<REGULATOR>>(),
                            g::par(
                                g::send::<DIRECTOR, HPLC_SERVICE, Final<HPLC_SERVICE>>(),
                                g::par(
                                    g::send::<DIRECTOR, MASS_SERVICE, Final<MASS_SERVICE>>(),
                                    g::par(
                                        g::send::<DIRECTOR, CELL_SERVICE, Final<CELL_SERVICE>>(),
                                        g::par(
                                            g::send::<DIRECTOR, FORM_SERVICE, Final<FORM_SERVICE>>(
                                            ),
                                            g::send::<DIRECTOR, STUDY_SERVICE, Final<STUDY_SERVICE>>(
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
    // Retire the study's entry authority before announcing a terminal result.
    // The material archives remain independent operations in parallel.
    g::seq(
        g::send::<DIRECTOR, CHEMIST, Specimen<STUDY>>(),
        g::seq(
            archived,
            g::seq(
                outcome,
                g::seq(
                    g::send::<STUDY_SERVICE, HPLC_SERVICE, DecisionEvidence<HPLC_SERVICE>>(),
                    positive::<CHEMIST, HPLC_SERVICE, HPLC, STUDY, 15, MASS_SERVICE>(),
                ),
            ),
        ),
    )
}

pub fn source() -> impl Projectable {
    let qc = g::seq(
        positive::<CHEMIST, HPLC_SERVICE, HPLC, QC, 0, FORMULATOR>(),
        g::send::<HPLC_SERVICE, MASS_SERVICE, QcEvidence>(),
    );
    let baseline = g::seq(
        mass::<TOXICOLOGIST, 2>(),
        g::send::<MASS_SERVICE, CELL_SERVICE, Baseline<CELL_SERVICE>>(),
    );
    let identity = g::seq(
        mass::<CHEMIST, 3>(),
        g::par(
            g::send::<MASS_SERVICE, FORMULATOR, Identity<FORMULATOR>>(),
            g::par(
                g::send::<MASS_SERVICE, FORM_SERVICE, Identity<FORM_SERVICE>>(),
                g::send::<MASS_SERVICE, STUDY_SERVICE, Identity<STUDY_SERVICE>>(),
            ),
        ),
    );
    let culture = g::seq(
        positive::<BIOLOGIST, CELL_SERVICE, CELL, CULTURE, 1, CLINICIAN>(),
        g::send::<CELL_SERVICE, HPLC_SERVICE, Cells<HPLC_SERVICE>>(),
    );
    let initial = g::par(g::seq(g::par(qc, baseline), identity), culture);
    g::seq(
        initial,
        g::seq(g::seq(round_entry(), round()).roll(), terminal()),
    )
}

pub const SESSION: u32 = 142;
pub const CHEMIST_IO: u8 = 7;
pub const BIOLOGIST_IO: u8 = 8;
pub const TOXICOLOGIST_IO: u8 = 9;
pub const FORMULATOR_IO: u8 = 10;
pub const CLINICIAN_IO: u8 = 11;
pub const REGULATOR_IO: u8 = 12;
pub const DIRECTOR_IO: u8 = 13;

/// Immutable projections of one global, including each guest's ChoreoFS I/O.
pub struct Programs {
    pub chemist: hibana::runtime::program::RoleProgram<CHEMIST>,
    pub biologist: hibana::runtime::program::RoleProgram<BIOLOGIST>,
    pub toxicologist: hibana::runtime::program::RoleProgram<TOXICOLOGIST>,
    pub formulator: hibana::runtime::program::RoleProgram<FORMULATOR>,
    pub clinician: hibana::runtime::program::RoleProgram<CLINICIAN>,
    pub regulator: hibana::runtime::program::RoleProgram<REGULATOR>,
    pub director: hibana::runtime::program::RoleProgram<DIRECTOR>,
    pub chemist_io: hibana::runtime::program::RoleProgram<CHEMIST_IO>,
    pub biologist_io: hibana::runtime::program::RoleProgram<BIOLOGIST_IO>,
    pub toxicologist_io: hibana::runtime::program::RoleProgram<TOXICOLOGIST_IO>,
    pub formulator_io: hibana::runtime::program::RoleProgram<FORMULATOR_IO>,
    pub clinician_io: hibana::runtime::program::RoleProgram<CLINICIAN_IO>,
    pub regulator_io: hibana::runtime::program::RoleProgram<REGULATOR_IO>,
    pub director_io: hibana::runtime::program::RoleProgram<DIRECTOR_IO>,
    pub hplc_service: hibana::runtime::program::RoleProgram<HPLC_SERVICE>,
    pub mass_service: hibana::runtime::program::RoleProgram<MASS_SERVICE>,
    pub cell_service: hibana::runtime::program::RoleProgram<CELL_SERVICE>,
    pub form_service: hibana::runtime::program::RoleProgram<FORM_SERVICE>,
    pub study_service: hibana::runtime::program::RoleProgram<STUDY_SERVICE>,
    pub hplc: hibana::runtime::program::RoleProgram<HPLC>,
    pub mass: hibana::runtime::program::RoleProgram<MASS>,
    pub cell: hibana::runtime::program::RoleProgram<CELL>,
    pub form: hibana::runtime::program::RoleProgram<FORM>,
    pub samples: hibana::runtime::program::RoleProgram<SAMPLES>,
    pub qc: hibana::runtime::program::RoleProgram<QC>,
    pub culture: hibana::runtime::program::RoleProgram<CULTURE>,
    pub formulation: hibana::runtime::program::RoleProgram<FORMULATION>,
    pub bioassay: hibana::runtime::program::RoleProgram<BIOASSAY>,
    pub stability: hibana::runtime::program::RoleProgram<STABILITY>,
    pub purity: hibana::runtime::program::RoleProgram<PURITY>,
    pub toxicity: hibana::runtime::program::RoleProgram<TOXICITY>,
    pub study: hibana::runtime::program::RoleProgram<STUDY>,
}
impl Programs {
    pub fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        serde_json::json!({
            "type":"compiled_projection", "format_version":1,
            "task":"14H", "session":SESSION,
            "roles":{"CHEMIST":CHEMIST,"BIOLOGIST":BIOLOGIST,"TOXICOLOGIST":TOXICOLOGIST,"FORMULATION_SCIENTIST":FORMULATOR,"CLINICAL_LEAD":CLINICIAN,"REGULATORY_SPECIALIST":REGULATOR,"PROJECT_DIRECTOR":DIRECTOR,"CHEMIST_IO":CHEMIST_IO,"BIOLOGIST_IO":BIOLOGIST_IO,"TOXICOLOGIST_IO":TOXICOLOGIST_IO,"FORMULATOR_IO":FORMULATOR_IO,"CLINICIAN_IO":CLINICIAN_IO,"REGULATOR_IO":REGULATOR_IO,"DIRECTOR_IO":DIRECTOR_IO,"HPLC_SERVICE":HPLC_SERVICE,"MASS_SERVICE":MASS_SERVICE,"CELL_SERVICE":CELL_SERVICE,"FORM_SERVICE":FORM_SERVICE,"STUDY_SERVICE":STUDY_SERVICE,"HPLC":HPLC,"MASS_SPEC":MASS,"CELL_LAB":CELL,"FORMULATION_SUITE":FORM,"BIOLOGICAL_SAMPLES":SAMPLES,"QC":QC,"CULTURE":CULTURE,"FORMULATION":FORMULATION,"BIOASSAY":BIOASSAY,"STABILITY":STABILITY,"PURITY":PURITY,"TOXICITY":TOXICITY,"STUDY":STUDY},
            "programs":[rows(&self.chemist),rows(&self.biologist),rows(&self.toxicologist),rows(&self.formulator),rows(&self.clinician),rows(&self.regulator),rows(&self.director),rows(&self.chemist_io),rows(&self.biologist_io),rows(&self.toxicologist_io),rows(&self.formulator_io),rows(&self.clinician_io),rows(&self.regulator_io),rows(&self.director_io),rows(&self.hplc_service),rows(&self.mass_service),rows(&self.cell_service),rows(&self.form_service),rows(&self.study_service),rows(&self.hplc),rows(&self.mass),rows(&self.cell),rows(&self.form),rows(&self.samples),rows(&self.qc),rows(&self.culture),rows(&self.formulation),rows(&self.bioassay),rows(&self.stability),rows(&self.purity),rows(&self.toxicity),rows(&self.study)]
        })
    }
}
pub fn programs() -> Programs {
    let source = g::par(
        source(),
        g::par(
            crate::shell::global::private_io::<CHEMIST, CHEMIST_IO>(),
            g::par(
                crate::shell::global::private_io::<BIOLOGIST, BIOLOGIST_IO>(),
                g::par(
                    crate::shell::global::private_io::<TOXICOLOGIST, TOXICOLOGIST_IO>(),
                    g::par(
                        crate::shell::global::private_io::<FORMULATOR, FORMULATOR_IO>(),
                        g::par(
                            crate::shell::global::private_io::<CLINICIAN, CLINICIAN_IO>(),
                            g::par(
                                crate::shell::global::private_io::<REGULATOR, REGULATOR_IO>(),
                                crate::shell::global::private_io::<DIRECTOR, DIRECTOR_IO>(),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );
    Programs {
        chemist: hibana::runtime::program::project(&source),
        biologist: hibana::runtime::program::project(&source),
        toxicologist: hibana::runtime::program::project(&source),
        formulator: hibana::runtime::program::project(&source),
        clinician: hibana::runtime::program::project(&source),
        regulator: hibana::runtime::program::project(&source),
        director: hibana::runtime::program::project(&source),
        chemist_io: hibana::runtime::program::project(&source),
        biologist_io: hibana::runtime::program::project(&source),
        toxicologist_io: hibana::runtime::program::project(&source),
        formulator_io: hibana::runtime::program::project(&source),
        clinician_io: hibana::runtime::program::project(&source),
        regulator_io: hibana::runtime::program::project(&source),
        director_io: hibana::runtime::program::project(&source),
        hplc_service: hibana::runtime::program::project(&source),
        mass_service: hibana::runtime::program::project(&source),
        cell_service: hibana::runtime::program::project(&source),
        form_service: hibana::runtime::program::project(&source),
        study_service: hibana::runtime::program::project(&source),
        hplc: hibana::runtime::program::project(&source),
        mass: hibana::runtime::program::project(&source),
        cell: hibana::runtime::program::project(&source),
        form: hibana::runtime::program::project(&source),
        samples: hibana::runtime::program::project(&source),
        qc: hibana::runtime::program::project(&source),
        culture: hibana::runtime::program::project(&source),
        formulation: hibana::runtime::program::project(&source),
        bioassay: hibana::runtime::program::project(&source),
        stability: hibana::runtime::program::project(&source),
        purity: hibana::runtime::program::project(&source),
        toxicity: hibana::runtime::program::project(&source),
        study: hibana::runtime::program::project(&source),
    }
}

#[cfg(test)]
mod tests {
    #[hibana_runtime::main]
    #[test]
    async fn scientific_views_keep_independent_work_and_gate_toxicity_on_purity()
    -> crate::shell::ShellResult<()> {
        use crate::shell::{ShellError, imp::bus::Bus};
        use hibana::runtime::{SessionKitStorage, ids::SessionId, program::project};
        for probe in 0..3 {
            let source = super::evaluation();
            assert_eq!(g::diagnose(&source), None);
            let bp = project::<BIOLOGIST>(&source);
            let tp = project::<TOXICOLOGIST>(&source);
            let bap = project::<BIOASSAY>(&source);
            let sap = project::<STABILITY>(&source);
            let pp = project::<PURITY>(&source);
            let txp = project::<TOXICITY>(&source);
            let sid = SessionId::new(SESSION);
            let (bus, _, _) = Bus::open(
                ["BIOLOGIST", "TOXICOLOGIST"],
                [
                    (sid, BIOASSAY),
                    (sid, STABILITY),
                    (sid, PURITY),
                    (sid, TOXICITY),
                ],
            );
            let mut slab = vec![0; 1024 * 1024];
            let mut storage = SessionKitStorage::uninit();
            let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
            let _biologist = rv.enter(sid, &bp)?;
            let _toxicologist = rv.enter(sid, &tp)?;
            let mut bio = rv.enter(sid, &bap)?;
            let mut stability = rv.enter(sid, &sap)?;
            let _purity = rv.enter(sid, &pp)?;
            let mut toxicity = rv.enter(sid, &txp)?;
            let bio_bytes = imp::encode(&serde_json::json!({"assay_type":"bioassay"}))?;
            let stable_bytes = imp::encode(&serde_json::json!({"sample":"stability"}))?;
            let tox_bytes = imp::encode(&serde_json::json!({"assay_type":"cytotoxicity"}))?;
            let actual = match probe {
                0 => {
                    hibana_runtime::race(
                        async {
                            Ok::<_, ShellError>(Some(
                                bio.send::<Intent<5>>(&imp::AnalysisArguments::<5>::from_bytes(
                                    &bio_bytes,
                                )?)
                                .await,
                            ))
                        },
                        async { Ok(None) },
                    )
                    .await?
                }
                1 => {
                    hibana_runtime::race(
                        async {
                            Ok::<_, ShellError>(Some(
                                stability
                                    .send::<Intent<6>>(&imp::AnalysisArguments::<6>::from_bytes(
                                        &stable_bytes,
                                    )?)
                                    .await,
                            ))
                        },
                        async { Ok(None) },
                    )
                    .await?
                }
                _ => {
                    hibana_runtime::race(
                        async {
                            Ok::<_, ShellError>(Some(
                                toxicity
                                    .send::<Intent<8>>(&imp::AnalysisArguments::<8>::from_bytes(
                                        &tox_bytes,
                                    )?)
                                    .await,
                            ))
                        },
                        async { Ok(None) },
                    )
                    .await?
                }
            };
            if probe == 2 {
                assert!(
                    !matches!(actual, Some(Ok(()))),
                    "toxicity must not send before its purity predecessor: {actual:?}"
                );
                assert!(
                    bus.transport
                        .take_observations()
                        .iter()
                        .all(|row| row["action"] != "send")
                );
            } else {
                assert!(
                    matches!(actual, Some(Ok(()))),
                    "independent analysis was blocked: {actual:?}"
                );
            }
        }
        Ok(())
    }
    use super::*;
    #[hibana_runtime::main]
    #[test]
    async fn stability_borrows_the_real_chromatograph_without_starting_bioassay()
    -> crate::shell::ShellResult<()> {
        use crate::shell::imp::bus::Bus;
        use hibana::runtime::{SessionKitStorage, ids::SessionId, program::project};
        let source = evaluation();
        assert_eq!(g::diagnose(&source), None);
        let hp = project::<HPLC_SERVICE>(&source);
        let bp = project::<BIOLOGIST>(&source);
        let tp = project::<TOXICOLOGIST>(&source);
        let mp = project::<MASS_SERVICE>(&source);
        let lp = project::<CLINICIAN>(&source);
        let pp = project::<PURITY>(&source);
        let sp = project::<STABILITY>(&source);
        let bap = project::<BIOASSAY>(&source);
        let ip = project::<HPLC>(&source);
        let sid = SessionId::new(SESSION);
        let (bus, _, _) = Bus::open(
            ["BIOLOGIST", "TOXICOLOGIST"],
            [
                (sid, HPLC),
                (sid, PURITY),
                (sid, STABILITY),
                (sid, BIOASSAY),
            ],
        );
        let mut slab = vec![0; 1024 * 1024];
        let mut storage = SessionKitStorage::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        let mut owner = rv.enter(sid, &hp)?;
        let mut biologist = rv.enter(sid, &bp)?;
        let mut toxicologist = rv.enter(sid, &tp)?;
        let mut mass = rv.enter(sid, &mp)?;
        let mut clinician = rv.enter(sid, &lp)?;
        let mut purity = rv.enter(sid, &pp)?;
        let mut stability = rv.enter(sid, &sp)?;
        let bioassay = rv.enter(sid, &bap)?;
        let instrument = rv.enter(sid, &ip)?;
        let outcome = serde_json::json!({
            "tool_name":"run_hplc_analysis", "success":true,
            "data":{"scientist":"TOXICOLOGIST","sample":"purity_check",
                "event_id":7,"artifact_id":7,"status":"complete",
                "inputs":{"formulation":4}}
        });
        let reply = imp::encode(&outcome)?;
        let purity_bytes = imp::encode(&serde_json::json!({"sample":"purity_check"}))?;
        let stability_bytes = imp::encode(&serde_json::json!({"sample":"stability"}))?;
        async {
            async {
                let input = imp::AnalysisArguments::<7>::from_bytes(&purity_bytes)?;
                purity.send::<Intent<7>>(&input).await?;
                let mut tool = toxicologist.recv::<Instrument<HPLC>>().await?;
                tool.send::<Request<7>>(&input).await?;
                toxicologist.send::<Instrument<HPLC>>(tool).await?;
                toxicologist.send::<Specimen<PURITY>>(purity).await?;
                let actual = imp::decode(toxicologist.recv::<Passed<7>>().await?)?;
                imp::same(&actual, &outcome)?;
                Ok(())
            };
            async {
                let input = imp::AnalysisArguments::<6>::from_bytes(&stability_bytes)?;
                stability.send::<Intent<6>>(&input).await?;
                let _instrument = biologist.recv::<Instrument<HPLC>>().await?;
                // The actual loan arrived although this scientist still owns
                // the untouched BIOASSAY Endpoint: no biological join exists.
                drop(bioassay);
                Ok(())
            };
            async {
                owner.recv::<Intent<7>>().await?;
                owner.send::<Instrument<HPLC>>(instrument).await?;
                owner.recv::<Request<7>>().await?;
                let instrument = owner.recv::<Instrument<HPLC>>().await?;
                let mut purity = owner.recv::<Specimen<PURITY>>().await?;
                purity
                    .send::<Passed<7>>(&imp::AnalysisReply::<7, true>::from_bytes(&reply)?)
                    .await?;
                owner.send::<Specimen<PURITY>>(purity).await?;
                owner
                    .send::<PurityEvidence<MASS_SERVICE>>(&imp::JsonTo::<MASS_SERVICE>::from_bytes(
                        &reply,
                    )?)
                    .await?;
                owner.recv::<Intent<6>>().await?;
                owner.send::<Instrument<HPLC>>(instrument).await?;
                Ok(())
            };
            async {
                let _purity = clinician.recv::<Specimen<PURITY>>().await?;
                Ok(())
            };
            async {
                let actual = imp::decode(mass.recv::<PurityEvidence<MASS_SERVICE>>().await?)?;
                imp::same(&actual, &outcome)?;
                Ok(())
            };
        }
        .await?;
        Ok(())
    }
    #[test]
    fn routine_and_deep_review_are_distinguishable_to_the_instrument_owners() {
        assert_eq!(g::diagnose(&review()), None);
    }
    #[test]
    fn all_guest_io_and_resource_views_use_the_same_complete_source() {
        use hibana::runtime::tap;
        let observation = programs().observation();
        let programs = observation["programs"].as_array().unwrap();
        assert_eq!(programs.len(), 32);
        let mut roles = programs
            .iter()
            .map(|p| p["role"].as_u64().unwrap())
            .collect::<Vec<_>>();
        roles.sort_unstable();
        assert_eq!(roles, (0..32).collect::<Vec<_>>());
        let sources = |program: &serde_json::Value| {
            program["rows"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|row| row[1].as_u64() == Some(u64::from(tap::PROJECTED_SOURCE)))
                .cloned()
                .collect::<Vec<_>>()
        };
        let expected = sources(&programs[0]);
        assert!(
            expected.len() > 206,
            "all seven WASI I/O branches must be included"
        );
        for program in programs {
            assert_eq!(sources(program), expected);
        }
        println!(
            "14H full source events={} programs={}",
            expected.len(),
            programs.len()
        );
    }
    #[test]
    fn all_material_and_review_branches_form_one_valid_global() {
        assert_eq!(g::diagnose(&source()), None);
    }
    #[test]
    fn director_projection_keeps_both_physical_sample_requests_and_deep_review() {
        use hibana::runtime::{program::project, tap};
        let program = project::<DIRECTOR>(&source());
        let rows = tap::projected(&program).collect::<Vec<_>>();
        let sources = rows
            .iter()
            .filter(|row| row.id() == tap::PROJECTED_SOURCE)
            .collect::<Vec<_>>();
        assert!(sources.iter().any(|row| row.arg0() >> 24 == SAMPLES as u32));
        assert!(sources.iter().any(
            |row| row.arg0() >> 24 == HPLC as u32 && (row.arg0() >> 16) as u8 == STUDY_SERVICE
        ));
        assert!(sources.iter().any(
            |row| row.arg0() >> 24 == MASS as u32 && (row.arg0() >> 16) as u8 == STUDY_SERVICE
        ));
        println!("14H business source events={}", sources.len());
    }
}
