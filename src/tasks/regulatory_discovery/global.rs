//! 14M: actual instruments, result-dependent retries and affine study custody.
use hibana::{Endpoint, g, runtime::program::Projectable};

pub const CHEMIST: u8 = 0;
pub const BIOLOGIST: u8 = 1;
pub const TOXICOLOGIST: u8 = 2;
pub const LEAD: u8 = 3;
pub const REGULATOR: u8 = 4;
pub const HPLC_SERVICE: u8 = 10;
pub const MASS_SERVICE: u8 = 11;
pub const CELL_SERVICE: u8 = 12;
pub const HPLC: u8 = 13;
pub const MASS: u8 = 14;
pub const CELL: u8 = 15;
pub const REVIEW_SERVICE: u8 = 16;
pub const QC: u8 = 20;
pub const CULTURE: u8 = 21;
pub const BIOASSAY: u8 = 22;
pub const STABILITY: u8 = 23;
pub const PURITY: u8 = 24;
pub const TOXICITY: u8 = 25;
pub const STRUCTURAL: u8 = 26;
pub const STUDY: u8 = 27;
pub const SESSION: u32 = 141;
pub const HPLC_RESULT: u16 = 970;
pub const CELL_RESULT: u16 = 974;
pub const REGULATORY_RESULT: u16 = 971;
pub const SELECT_TOX: u16 = 972;
pub const SELECT_BIO: u16 = 973;

pub type Intent<'a, const STEP: u8> = g::Msg<1, super::imp::AnalysisArguments<'a, STEP>>;
pub type Request<'a, const STEP: u8> = g::Msg<2, super::imp::AnalysisArguments<'a, STEP>>;
pub type Passed<'a, const STEP: u8> = g::Msg<3, super::imp::AnalysisReply<'a, STEP, true>>;
pub type Failed<'a, const STEP: u8> = g::Msg<4, super::imp::AnalysisReply<'a, STEP, false>>;
pub type Instrument<'r, const ROLE: u8> = g::Msg<5, Endpoint<'r, ROLE>>;
pub type Specimen<'r, const ROLE: u8> = g::Msg<6, Endpoint<'r, ROLE>>;
pub type MassIntent<'a, const STEP: u8> = g::Msg<10, super::imp::AnalysisArguments<'a, STEP>>;
pub type MassRequest<'a, const STEP: u8> = g::Msg<11, super::imp::AnalysisArguments<'a, STEP>>;
pub type MassResult<'a> = g::Msg<12, &'a [u8]>;
pub type CompoundBio<'a> = g::Msg<20, &'a [u8]>;
pub type CompoundTox<'a> = g::Msg<31, &'a [u8]>;
pub type BioReport<'a> = g::Msg<21, &'a [u8]>;
pub type ToxReport<'a> = g::Msg<22, &'a [u8]>;
pub type Decision<'a> = g::Msg<23, &'a [u8]>;
pub type DecisionRecorded<'a> = g::Msg<24, &'a [u8]>;
pub type Review<'a> = g::Msg<25, &'a [u8]>;
pub type Approved<'a> = g::Msg<26, crate::shell::imp::review_reply::ReviewReply<'a, true>>;
pub type ApprovedForLead<'a> = g::Msg<40, crate::shell::imp::review_reply::ReviewReply<'a, true>>;
pub type ApprovedForBio<'a> = g::Msg<41, crate::shell::imp::review_reply::ReviewReply<'a, true>>;
pub type ApprovedForTox<'a> = g::Msg<42, crate::shell::imp::review_reply::ReviewReply<'a, true>>;
pub type ApprovedForHplc<'a> = g::Msg<43, crate::shell::imp::review_reply::ReviewReply<'a, true>>;
pub type ApprovedForCells<'a> = g::Msg<44, crate::shell::imp::review_reply::ReviewReply<'a, true>>;
pub type ApprovedForChemist<'a> =
    g::Msg<45, crate::shell::imp::review_reply::ReviewReply<'a, true>>;
pub type Rejected<'a> = g::Msg<27, crate::shell::imp::review_reply::ReviewReply<'a, false>>;
pub type Plan<'a, const MASK: u8, const DEST: u8> =
    g::Msg<MASK, super::imp::ReanalysisPlan<'a, MASK, DEST>>;
pub type Document<'a> = g::Msg<29, &'a [u8]>;

// A specimen goes to the effect receiver before an actual result is known.
// Failure returns it for a new attempt; success transfers it to the lead.
// These are scientific continuations, not another instrument owner table.
fn trial<const WHO: u8, const SERVICE: u8, const TOOL: u8, const CAP: u8, const STEP: u8>()
-> impl Projectable {
    g::seq(
        g::send::<CAP, SERVICE, Intent<STEP>>(),
        g::seq(
            g::send::<SERVICE, WHO, Instrument<TOOL>>(),
            g::seq(
                g::send::<TOOL, SERVICE, Request<STEP>>(),
                g::seq(
                    g::send::<WHO, SERVICE, Instrument<TOOL>>(),
                    g::send::<WHO, SERVICE, Specimen<CAP>>(),
                ),
            ),
        ),
    )
}
fn passed<const WHO: u8, const SERVICE: u8, const CAP: u8, const STEP: u8>() -> impl Projectable {
    g::seq(
        g::send::<CAP, WHO, Passed<STEP>>(),
        g::send::<SERVICE, LEAD, Specimen<CAP>>(),
    )
}
fn analysis<
    const WHO: u8,
    const SERVICE: u8,
    const TOOL: u8,
    const CAP: u8,
    const STEP: u8,
    const RESOLVER: u16,
>() -> impl Projectable {
    let retry = g::seq(
        g::send::<CAP, WHO, Failed<STEP>>(),
        g::seq(
            g::send::<SERVICE, WHO, Specimen<CAP>>(),
            trial::<WHO, SERVICE, TOOL, CAP, STEP>(),
        ),
    );
    g::seq(
        trial::<WHO, SERVICE, TOOL, CAP, STEP>(),
        g::route(
            passed::<WHO, SERVICE, CAP, STEP>(),
            g::seq(retry.roll(), passed::<WHO, SERVICE, CAP, STEP>()),
        )
        .resolve::<RESOLVER>(),
    )
}
fn mass<const WHO: u8, const STEP: u8>() -> impl Projectable {
    g::seq(
        g::send::<WHO, MASS_SERVICE, MassIntent<STEP>>(),
        g::seq(
            g::send::<MASS_SERVICE, WHO, Instrument<MASS>>(),
            g::seq(
                g::send::<MASS, MASS_SERVICE, MassRequest<STEP>>(),
                g::seq(
                    g::send::<MASS_SERVICE, MASS, MassResult>(),
                    g::send::<WHO, MASS_SERVICE, Instrument<MASS>>(),
                ),
            ),
        ),
    )
}
fn reports() -> impl Projectable {
    g::par(
        g::send::<BIOLOGIST, LEAD, BioReport>(),
        g::send::<TOXICOLOGIST, LEAD, ToxReport>(),
    )
}
fn plan<const MASK: u8>() -> impl Projectable {
    g::par(
        g::send::<STUDY, BIOLOGIST, Plan<MASK, BIOLOGIST>>(),
        g::par(
            g::send::<STUDY, TOXICOLOGIST, Plan<MASK, TOXICOLOGIST>>(),
            g::par(
                g::send::<STUDY, HPLC_SERVICE, Plan<MASK, HPLC_SERVICE>>(),
                g::send::<STUDY, CELL_SERVICE, Plan<MASK, CELL_SERVICE>>(),
            ),
        ),
    )
}
fn reanalyze_bio() -> impl Projectable {
    g::seq(
        g::par(
            g::send::<LEAD, BIOLOGIST, Specimen<BIOASSAY>>(),
            g::send::<LEAD, BIOLOGIST, Specimen<STABILITY>>(),
        ),
        g::seq(
            analysis::<BIOLOGIST, CELL_SERVICE, CELL, BIOASSAY, 11, CELL_RESULT>(),
            analysis::<BIOLOGIST, HPLC_SERVICE, HPLC, STABILITY, 12, HPLC_RESULT>(),
        ),
    )
}
fn reanalyze_tox() -> impl Projectable {
    g::seq(
        g::par(
            g::send::<LEAD, TOXICOLOGIST, Specimen<PURITY>>(),
            g::send::<LEAD, TOXICOLOGIST, Specimen<TOXICITY>>(),
        ),
        g::seq(
            analysis::<TOXICOLOGIST, HPLC_SERVICE, HPLC, PURITY, 13, HPLC_RESULT>(),
            analysis::<TOXICOLOGIST, CELL_SERVICE, CELL, TOXICITY, 14, CELL_RESULT>(),
        ),
    )
}
fn reanalyze_both() -> impl Projectable {
    let loans = g::par(
        g::par(
            g::send::<LEAD, BIOLOGIST, Specimen<BIOASSAY>>(),
            g::send::<LEAD, BIOLOGIST, Specimen<STABILITY>>(),
        ),
        g::par(
            g::send::<LEAD, TOXICOLOGIST, Specimen<PURITY>>(),
            g::send::<LEAD, TOXICOLOGIST, Specimen<TOXICITY>>(),
        ),
    );
    g::seq(
        loans,
        g::seq(
            g::par(
                analysis::<BIOLOGIST, CELL_SERVICE, CELL, BIOASSAY, 11, CELL_RESULT>(),
                analysis::<TOXICOLOGIST, HPLC_SERVICE, HPLC, PURITY, 13, HPLC_RESULT>(),
            ),
            g::par(
                analysis::<BIOLOGIST, HPLC_SERVICE, HPLC, STABILITY, 12, HPLC_RESULT>(),
                analysis::<TOXICOLOGIST, CELL_SERVICE, CELL, TOXICITY, 14, CELL_RESULT>(),
            ),
        ),
    )
}
fn reconsider() -> impl Projectable {
    g::seq(
        g::route(
            g::route(plan::<0>(), g::seq(plan::<1>(), reanalyze_bio())).resolve::<SELECT_BIO>(),
            g::route(
                g::seq(plan::<2>(), reanalyze_tox()),
                g::seq(plan::<3>(), reanalyze_both()),
            )
            .resolve::<SELECT_BIO>(),
        )
        .resolve::<SELECT_TOX>(),
        reports(),
    )
}
fn review() -> impl Projectable {
    g::seq(
        g::send::<STUDY, HPLC_SERVICE, Decision>(),
        g::seq(
            g::send::<HPLC_SERVICE, STUDY, DecisionRecorded>(),
            g::seq(
                g::send::<STUDY, REGULATOR, Decision>(),
                g::seq(
                    g::send::<LEAD, REGULATOR, Specimen<STUDY>>(),
                    g::seq(
                        g::send::<STUDY, REVIEW_SERVICE, Review>(),
                        g::send::<REGULATOR, REVIEW_SERVICE, Specimen<STUDY>>(),
                    ),
                ),
            ),
        ),
    )
}
fn approved() -> impl Projectable {
    // Every participant awaiting possible reanalysis receives the actual approval.
    // This also gives their localsides a protocol-defined termination boundary.
    g::seq(
        g::send::<STUDY, REGULATOR, Approved>(),
        g::seq(
            g::send::<STUDY, LEAD, ApprovedForLead>(),
            g::seq(
                g::send::<STUDY, BIOLOGIST, ApprovedForBio>(),
                g::seq(
                    g::send::<STUDY, TOXICOLOGIST, ApprovedForTox>(),
                    g::seq(
                        g::send::<STUDY, HPLC_SERVICE, ApprovedForHplc>(),
                        g::seq(
                            g::send::<STUDY, CELL_SERVICE, ApprovedForCells>(),
                            g::seq(
                                g::send::<STUDY, CHEMIST, ApprovedForChemist>(),
                                g::send::<REVIEW_SERVICE, CHEMIST, Specimen<STUDY>>(),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    )
}
fn decision_cycle() -> impl Projectable {
    let rejected = g::seq(
        g::send::<STUDY, REGULATOR, Rejected>(),
        g::seq(
            g::send::<STUDY, LEAD, Rejected>(),
            g::seq(
                g::send::<REVIEW_SERVICE, LEAD, Specimen<STUDY>>(),
                g::seq(reconsider(), review()),
            ),
        ),
    );
    g::seq(
        review(),
        g::route(approved(), g::seq(rejected.roll(), approved())).resolve::<REGULATORY_RESULT>(),
    )
}

/// One complete study, including all eleven analyses, optional selective
/// reanalysis, a fresh decision after each rejection and approved documentation.
pub fn source() -> impl Projectable {
    let initial = g::par(
        analysis::<CHEMIST, HPLC_SERVICE, HPLC, QC, 0, HPLC_RESULT>(),
        g::par(
            analysis::<BIOLOGIST, CELL_SERVICE, CELL, CULTURE, 1, CELL_RESULT>(),
            g::seq(mass::<TOXICOLOGIST, 2>(), mass::<LEAD, 3>()),
        ),
    );
    let evaluation = g::seq(
        g::par(
            analysis::<BIOLOGIST, CELL_SERVICE, CELL, BIOASSAY, 4, CELL_RESULT>(),
            analysis::<TOXICOLOGIST, HPLC_SERVICE, HPLC, PURITY, 6, HPLC_RESULT>(),
        ),
        g::par(
            analysis::<BIOLOGIST, HPLC_SERVICE, HPLC, STABILITY, 5, HPLC_RESULT>(),
            analysis::<TOXICOLOGIST, CELL_SERVICE, CELL, TOXICITY, 7, CELL_RESULT>(),
        ),
    );
    let documentation = g::seq(
        g::send::<STUDY, HPLC_SERVICE, Document>(),
        g::seq(
            g::send::<HPLC_SERVICE, CHEMIST, Instrument<HPLC>>(),
            g::seq(
                g::send::<HPLC, HPLC_SERVICE, Document>(),
                g::seq(
                    g::send::<CHEMIST, HPLC_SERVICE, Instrument<HPLC>>(),
                    g::send::<CHEMIST, HPLC_SERVICE, Specimen<STUDY>>(),
                ),
            ),
        ),
    );
    let finished = g::seq(
        g::send::<STUDY, CHEMIST, Passed<15>>(),
        g::send::<HPLC_SERVICE, MASS_SERVICE, Specimen<STUDY>>(),
    );
    let retry = g::seq(
        g::send::<STUDY, CHEMIST, Failed<15>>(),
        g::seq(
            g::send::<HPLC_SERVICE, CHEMIST, Specimen<STUDY>>(),
            documentation,
        ),
    );
    let finish = g::seq(
        documentation,
        g::route(finished, g::seq(retry.roll(), finished)).resolve::<HPLC_RESULT>(),
    );
    g::seq(
        initial,
        g::seq(
            mass::<CHEMIST, 8>(),
            g::seq(
                g::par(
                    g::send::<CHEMIST, BIOLOGIST, CompoundBio>(),
                    g::send::<CHEMIST, TOXICOLOGIST, CompoundTox>(),
                ),
                g::seq(
                    evaluation,
                    g::seq(
                        reports(),
                        g::seq(
                            g::par(
                                analysis::<LEAD, HPLC_SERVICE, HPLC, STRUCTURAL, 9, HPLC_RESULT>(),
                                mass::<LEAD, 10>(),
                            ),
                            g::seq(decision_cycle(), finish),
                        ),
                    ),
                ),
            ),
        ),
    )
}

pub const CHEMIST_IO: u8 = 5;
pub const BIOLOGIST_IO: u8 = 6;
pub const TOXICOLOGIST_IO: u8 = 7;
pub const LEAD_IO: u8 = 8;
pub const REGULATOR_IO: u8 = 9;
pub struct Programs {
    pub chemist: hibana::runtime::program::RoleProgram<CHEMIST>,
    pub biologist: hibana::runtime::program::RoleProgram<BIOLOGIST>,
    pub toxicologist: hibana::runtime::program::RoleProgram<TOXICOLOGIST>,
    pub lead: hibana::runtime::program::RoleProgram<LEAD>,
    pub regulator: hibana::runtime::program::RoleProgram<REGULATOR>,
    pub chemist_io: hibana::runtime::program::RoleProgram<CHEMIST_IO>,
    pub biologist_io: hibana::runtime::program::RoleProgram<BIOLOGIST_IO>,
    pub toxicologist_io: hibana::runtime::program::RoleProgram<TOXICOLOGIST_IO>,
    pub lead_io: hibana::runtime::program::RoleProgram<LEAD_IO>,
    pub regulator_io: hibana::runtime::program::RoleProgram<REGULATOR_IO>,
    pub hplc_service: hibana::runtime::program::RoleProgram<HPLC_SERVICE>,
    pub mass_service: hibana::runtime::program::RoleProgram<MASS_SERVICE>,
    pub cell_service: hibana::runtime::program::RoleProgram<CELL_SERVICE>,
    pub review_service: hibana::runtime::program::RoleProgram<REVIEW_SERVICE>,
    pub hplc: hibana::runtime::program::RoleProgram<HPLC>,
    pub mass: hibana::runtime::program::RoleProgram<MASS>,
    pub cell: hibana::runtime::program::RoleProgram<CELL>,
    pub qc: hibana::runtime::program::RoleProgram<QC>,
    pub culture: hibana::runtime::program::RoleProgram<CULTURE>,
    pub bioassay: hibana::runtime::program::RoleProgram<BIOASSAY>,
    pub stability: hibana::runtime::program::RoleProgram<STABILITY>,
    pub purity: hibana::runtime::program::RoleProgram<PURITY>,
    pub toxicity: hibana::runtime::program::RoleProgram<TOXICITY>,
    pub structural: hibana::runtime::program::RoleProgram<STRUCTURAL>,
    pub study: hibana::runtime::program::RoleProgram<STUDY>,
}
impl Programs {
    pub fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"14M","session":SESSION,
   "roles":{"CHEMIST":CHEMIST,"BIOLOGIST":BIOLOGIST,"TOXICOLOGIST":TOXICOLOGIST,"LEAD_SCIENTIST":LEAD,"REGULATORY_SPECIALIST":REGULATOR,"CHEMIST_IO":CHEMIST_IO,"BIOLOGIST_IO":BIOLOGIST_IO,"TOXICOLOGIST_IO":TOXICOLOGIST_IO,"LEAD_IO":LEAD_IO,"REGULATOR_IO":REGULATOR_IO,"HPLC_SERVICE":HPLC_SERVICE,"MASS_SERVICE":MASS_SERVICE,"CELL_SERVICE":CELL_SERVICE,"REVIEW_SERVICE":REVIEW_SERVICE,"HPLC":HPLC,"MASS_SPEC":MASS,"CELL_LAB":CELL,"QC":QC,"CULTURE":CULTURE,"BIOASSAY":BIOASSAY,"STABILITY":STABILITY,"PURITY":PURITY,"TOXICITY":TOXICITY,"STRUCTURAL":STRUCTURAL,"STUDY":STUDY},
"programs":[
rows(&self.chemist),rows(&self.biologist),rows(&self.toxicologist),rows(&self.lead),rows(&self.regulator),rows(&self.chemist_io),rows(&self.biologist_io),rows(&self.toxicologist_io),rows(&self.lead_io),rows(&self.regulator_io),rows(&self.hplc_service),rows(&self.mass_service),rows(&self.cell_service),rows(&self.review_service),rows(&self.hplc),rows(&self.mass),rows(&self.cell),rows(&self.qc),rows(&self.culture),rows(&self.bioassay),rows(&self.stability),rows(&self.purity),rows(&self.toxicity),rows(&self.structural),rows(&self.study)]})
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
                        crate::shell::global::private_io::<LEAD, LEAD_IO>(),
                        crate::shell::global::private_io::<REGULATOR, REGULATOR_IO>(),
                    ),
                ),
            ),
        ),
    );
    Programs {
        chemist: hibana::runtime::program::project(&source),
        biologist: hibana::runtime::program::project(&source),
        toxicologist: hibana::runtime::program::project(&source),
        lead: hibana::runtime::program::project(&source),
        regulator: hibana::runtime::program::project(&source),
        chemist_io: hibana::runtime::program::project(&source),
        biologist_io: hibana::runtime::program::project(&source),
        toxicologist_io: hibana::runtime::program::project(&source),
        lead_io: hibana::runtime::program::project(&source),
        regulator_io: hibana::runtime::program::project(&source),
        hplc_service: hibana::runtime::program::project(&source),
        mass_service: hibana::runtime::program::project(&source),
        cell_service: hibana::runtime::program::project(&source),
        review_service: hibana::runtime::program::project(&source),
        hplc: hibana::runtime::program::project(&source),
        mass: hibana::runtime::program::project(&source),
        cell: hibana::runtime::program::project(&source),
        qc: hibana::runtime::program::project(&source),
        culture: hibana::runtime::program::project(&source),
        bioassay: hibana::runtime::program::project(&source),
        stability: hibana::runtime::program::project(&source),
        purity: hibana::runtime::program::project(&source),
        toxicity: hibana::runtime::program::project(&source),
        structural: hibana::runtime::program::project(&source),
        study: hibana::runtime::program::project(&source),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn one_failed_analysis_can_return_the_specimen_and_retry() {
        assert_eq!(
            g::diagnose(&analysis::<CHEMIST, HPLC_SERVICE, HPLC, QC, 0, HPLC_RESULT>()),
            None
        );
    }
}
