//! Real instrument custody and compound/report data; no application scheduler.
use hibana::{
    Endpoint, g,
    runtime::program::{Projectable, RoleProgram, project},
};
pub const CHEMIST: u8 = 0;
pub const BIOLOGIST: u8 = 1;
pub const TOXICOLOGIST: u8 = 2;
pub const LEAD: u8 = 3;
pub const CHEMIST_IO: u8 = 4;
pub const BIOLOGIST_IO: u8 = 5;
pub const TOXICOLOGIST_IO: u8 = 6;
pub const LEAD_IO: u8 = 7;
pub const HPLC_SERVICE: u8 = 8;
pub const MASS_SERVICE: u8 = 9;
pub const CELL_SERVICE: u8 = 10;
pub const HPLC: u8 = 13;
pub const MASS_SPEC: u8 = 14;
pub const CELL_LAB: u8 = 15;
pub const SESSION: u32 = 140;
pub type HplcIntent<'a> = g::Msg<1, &'a [u8]>;
pub type HplcRequest<'a> = g::Msg<2, &'a [u8]>;
pub type HplcResult<'a> = g::Msg<3, &'a [u8]>;
pub type HplcLoan<'r> = g::Msg<4, Endpoint<'r, HPLC>>;
pub type MassIntent<'a> = g::Msg<11, &'a [u8]>;
pub type MassRequest<'a> = g::Msg<12, &'a [u8]>;
pub type MassResult<'a> = g::Msg<13, &'a [u8]>;
pub type MassLoan<'r> = g::Msg<14, Endpoint<'r, MASS_SPEC>>;
pub type CellIntent<'a> = g::Msg<21, &'a [u8]>;
pub type CellRequest<'a> = g::Msg<22, &'a [u8]>;
pub type CellResult<'a> = g::Msg<23, &'a [u8]>;
pub type CellLoan<'r> = g::Msg<24, Endpoint<'r, CELL_LAB>>;
pub type CompoundBio<'a> = g::Msg<30, &'a [u8]>;
pub type CompoundTox<'a> = g::Msg<31, &'a [u8]>;
pub type BioReport<'a> = g::Msg<32, &'a [u8]>;
pub type ToxReport<'a> = g::Msg<33, &'a [u8]>;
pub type Decision<'a> = g::Msg<34, &'a [u8]>;
pub type DecisionForOwner<'a> = g::Msg<35, &'a [u8]>;
pub type DecisionSubmitted<'a> = g::Msg<36, &'a [u8]>;

fn hplc<const WHO: u8>() -> impl Projectable {
    g::seq(
        g::send::<WHO, HPLC_SERVICE, HplcIntent>(),
        g::seq(
            g::send::<HPLC_SERVICE, WHO, HplcLoan>(),
            g::seq(
                g::send::<HPLC, HPLC_SERVICE, HplcRequest>(),
                g::seq(
                    g::send::<HPLC_SERVICE, HPLC, HplcResult>(),
                    g::send::<WHO, HPLC_SERVICE, HplcLoan>(),
                ),
            ),
        ),
    )
}
fn mass<const WHO: u8>() -> impl Projectable {
    g::seq(
        g::send::<WHO, MASS_SERVICE, MassIntent>(),
        g::seq(
            g::send::<MASS_SERVICE, WHO, MassLoan>(),
            g::seq(
                g::send::<MASS_SPEC, MASS_SERVICE, MassRequest>(),
                g::seq(
                    g::send::<MASS_SERVICE, MASS_SPEC, MassResult>(),
                    g::send::<WHO, MASS_SERVICE, MassLoan>(),
                ),
            ),
        ),
    )
}
fn cell_assay<const WHO: u8>() -> impl Projectable {
    g::seq(
        g::send::<WHO, CELL_SERVICE, CellIntent>(),
        g::seq(
            g::send::<CELL_LAB, CELL_SERVICE, CellRequest>(),
            g::send::<CELL_SERVICE, CELL_LAB, CellResult>(),
        ),
    )
}

pub struct Programs {
    pub chemist: RoleProgram<CHEMIST>,
    pub biologist: RoleProgram<BIOLOGIST>,
    pub toxicologist: RoleProgram<TOXICOLOGIST>,
    pub lead: RoleProgram<LEAD>,
    pub chemist_io: RoleProgram<CHEMIST_IO>,
    pub biologist_io: RoleProgram<BIOLOGIST_IO>,
    pub toxicologist_io: RoleProgram<TOXICOLOGIST_IO>,
    pub lead_io: RoleProgram<LEAD_IO>,
    pub hplc_service: RoleProgram<HPLC_SERVICE>,
    pub mass_service: RoleProgram<MASS_SERVICE>,
    pub cell_service: RoleProgram<CELL_SERVICE>,
    pub hplc: RoleProgram<HPLC>,
    pub mass: RoleProgram<MASS_SPEC>,
    pub cell: RoleProgram<CELL_LAB>,
}
impl Programs {
    pub(crate) fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        use hibana::g::Message;
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"14E","session":SESSION,
          "roles":{"CHEMIST":CHEMIST,"BIOLOGIST":BIOLOGIST,"TOXICOLOGIST":TOXICOLOGIST,"LEAD_SCIENTIST":LEAD,
          "CHEMIST_IO":CHEMIST_IO,"BIOLOGIST_IO":BIOLOGIST_IO,"TOXICOLOGIST_IO":TOXICOLOGIST_IO,"LEAD_IO":LEAD_IO,
          "HPLC_SERVICE":HPLC_SERVICE,"MASS_SERVICE":MASS_SERVICE,"CELL_SERVICE":CELL_SERVICE,
          "HPLC":HPLC,"MASS_SPEC":MASS_SPEC,"CELL_LAB":CELL_LAB},
          "labels":{"HplcIntent":HplcIntent::LOGICAL_LABEL,"HplcRequest":HplcRequest::LOGICAL_LABEL,
          "HplcResult":HplcResult::LOGICAL_LABEL,"HplcLoan":HplcLoan::LOGICAL_LABEL,
          "MassIntent":MassIntent::LOGICAL_LABEL,"MassRequest":MassRequest::LOGICAL_LABEL,
          "MassResult":MassResult::LOGICAL_LABEL,"MassLoan":MassLoan::LOGICAL_LABEL,
          "CellIntent":CellIntent::LOGICAL_LABEL,"CellRequest":CellRequest::LOGICAL_LABEL,
          "CellResult":CellResult::LOGICAL_LABEL,"CellLoan":CellLoan::LOGICAL_LABEL,
          "CompoundBio":CompoundBio::LOGICAL_LABEL,"CompoundTox":CompoundTox::LOGICAL_LABEL,
          "BioReport":BioReport::LOGICAL_LABEL,"ToxReport":ToxReport::LOGICAL_LABEL,
          "Decision":Decision::LOGICAL_LABEL,"DecisionForOwner":DecisionForOwner::LOGICAL_LABEL,"DecisionSubmitted":DecisionSubmitted::LOGICAL_LABEL},
          "programs":[rows(&self.chemist),rows(&self.biologist),rows(&self.toxicologist),rows(&self.lead),
          rows(&self.chemist_io),rows(&self.biologist_io),rows(&self.toxicologist_io),rows(&self.lead_io),
          rows(&self.hplc_service),rows(&self.mass_service),rows(&self.cell_service),rows(&self.hplc),rows(&self.mass),rows(&self.cell)]})
    }
}
pub fn programs() -> Programs {
    let prep = g::seq(
        g::send::<CELL_SERVICE, BIOLOGIST, CellLoan>(),
        cell_assay::<BIOLOGIST>(),
    );
    let initial = g::par(
        hplc::<CHEMIST>(),
        g::par(prep, g::seq(mass::<TOXICOLOGIST>(), mass::<LEAD>())),
    );
    let samples = g::par(
        g::send::<CHEMIST, BIOLOGIST, CompoundBio>(),
        g::send::<CHEMIST, TOXICOLOGIST, CompoundTox>(),
    );
    let evaluation = g::seq(
        g::par(hplc::<TOXICOLOGIST>(), cell_assay::<BIOLOGIST>()),
        g::seq(
            hplc::<BIOLOGIST>(),
            g::seq(
                g::send::<BIOLOGIST, CELL_SERVICE, CellLoan>(),
                g::par(
                    g::send::<BIOLOGIST, LEAD, BioReport>(),
                    g::seq(
                        g::send::<CELL_SERVICE, TOXICOLOGIST, CellLoan>(),
                        g::seq(
                            cell_assay::<TOXICOLOGIST>(),
                            g::seq(
                                g::send::<TOXICOLOGIST, CELL_SERVICE, CellLoan>(),
                                g::send::<TOXICOLOGIST, LEAD, ToxReport>(),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );
    let decision = g::seq(
        g::send::<LEAD, HPLC_SERVICE, DecisionForOwner>(),
        g::seq(
            g::send::<HPLC_SERVICE, LEAD, DecisionSubmitted>(),
            g::send::<LEAD, CHEMIST, Decision>(),
        ),
    );
    let business = g::seq(
        initial,
        g::seq(
            mass::<CHEMIST>(),
            g::seq(
                samples,
                g::seq(
                    evaluation,
                    g::seq(
                        hplc::<LEAD>(),
                        g::seq(mass::<LEAD>(), g::seq(decision, hplc::<CHEMIST>())),
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
                    crate::shell::global::private_io::<TOXICOLOGIST, TOXICOLOGIST_IO>(),
                    crate::shell::global::private_io::<LEAD, LEAD_IO>(),
                ),
            ),
        ),
    );
    Programs {
        chemist: project(&source),
        biologist: project(&source),
        toxicologist: project(&source),
        lead: project(&source),
        chemist_io: project(&source),
        biologist_io: project(&source),
        toxicologist_io: project(&source),
        lead_io: project(&source),
        hplc_service: project(&source),
        mass_service: project(&source),
        cell_service: project(&source),
        hplc: project(&source),
        mass: project(&source),
        cell: project(&source),
    }
}
