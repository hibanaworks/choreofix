//! Instrument custody, the actual specimen and quality data for task 13E.
use hibana::{
    Endpoint, g,
    runtime::program::{Projectable, RoleProgram, project},
};
pub const CHEMIST: u8 = 0;
pub const BIOLOGIST: u8 = 1;
pub const ANALYST: u8 = 2;
pub const CHEMIST_IO: u8 = 3;
pub const BIOLOGIST_IO: u8 = 4;
pub const ANALYST_IO: u8 = 5;
pub const SPEC_SERVICE: u8 = 6;
pub const CENT_SERVICE: u8 = 7;
pub const SPECTROMETER: u8 = 8;
pub const CENTRIFUGE: u8 = 9;
pub const SESSION: u32 = 130;
pub type SpecIntent<'a> = g::Msg<1, &'a [u8]>;
pub type SpecRequest<'a> = g::Msg<2, &'a [u8]>;
pub type SpecResult<'a> = g::Msg<3, &'a [u8]>;
pub type SpecLoan<'r> = g::Msg<4, Endpoint<'r, SPECTROMETER>>;
pub type CentIntent<'a> = g::Msg<11, &'a [u8]>;
pub type CentRequest<'a> = g::Msg<12, &'a [u8]>;
pub type CentResult<'a> = g::Msg<13, &'a [u8]>;
pub type CentLoan<'r> = g::Msg<14, Endpoint<'r, CENTRIFUGE>>;
pub type Sample<'a> = g::Msg<20, &'a [u8]>;
pub type BioRequest<'a> = g::Msg<21, &'a [u8]>;
pub type BioResult<'a> = g::Msg<22, &'a [u8]>;
pub type BioProof<'a> = g::Msg<23, &'a [u8]>;
pub type BioReport<'a> = g::Msg<24, &'a [u8]>;
pub type Verification<'a> = g::Msg<25, &'a [u8]>;
pub type Quality<'a> = g::Msg<26, &'a [u8]>;
pub type QualityForOwner<'a> = g::Msg<27, &'a [u8]>;

fn analysis<const WHO: u8>() -> impl Projectable {
    g::seq(
        g::send::<WHO, SPEC_SERVICE, SpecIntent>(),
        g::seq(
            g::send::<SPEC_SERVICE, WHO, SpecLoan>(),
            g::seq(
                g::send::<SPECTROMETER, SPEC_SERVICE, SpecRequest>(),
                g::seq(
                    g::send::<SPEC_SERVICE, SPECTROMETER, SpecResult>(),
                    g::send::<WHO, SPEC_SERVICE, SpecLoan>(),
                ),
            ),
        ),
    )
}
fn separation<const WHO: u8>() -> impl Projectable {
    g::seq(
        g::send::<WHO, CENT_SERVICE, CentIntent>(),
        g::seq(
            g::send::<CENT_SERVICE, WHO, CentLoan>(),
            g::seq(
                g::send::<CENTRIFUGE, CENT_SERVICE, CentRequest>(),
                g::seq(
                    g::send::<CENT_SERVICE, CENTRIFUGE, CentResult>(),
                    g::send::<WHO, CENT_SERVICE, CentLoan>(),
                ),
            ),
        ),
    )
}
pub struct Programs {
    pub chemist: RoleProgram<CHEMIST>,
    pub biologist: RoleProgram<BIOLOGIST>,
    pub analyst: RoleProgram<ANALYST>,
    pub chemist_io: RoleProgram<CHEMIST_IO>,
    pub biologist_io: RoleProgram<BIOLOGIST_IO>,
    pub analyst_io: RoleProgram<ANALYST_IO>,
    pub spec_service: RoleProgram<SPEC_SERVICE>,
    pub cent_service: RoleProgram<CENT_SERVICE>,
    pub spectrometer: RoleProgram<SPECTROMETER>,
    pub centrifuge: RoleProgram<CENTRIFUGE>,
}
impl Programs {
    pub(crate) fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        use hibana::g::Message;
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"13E","session":SESSION,
  "roles":{"CHEMIST":CHEMIST,"BIOLOGIST":BIOLOGIST,"ANALYST":ANALYST,"CHEMIST_IO":CHEMIST_IO,"BIOLOGIST_IO":BIOLOGIST_IO,"ANALYST_IO":ANALYST_IO,
   "SPEC_SERVICE":SPEC_SERVICE,"CENT_SERVICE":CENT_SERVICE,"SPECTROMETER":SPECTROMETER,"CENTRIFUGE":CENTRIFUGE},
  "labels":{"SpecIntent":SpecIntent::LOGICAL_LABEL,"SpecRequest":SpecRequest::LOGICAL_LABEL,"SpecResult":SpecResult::LOGICAL_LABEL,"SpecLoan":SpecLoan::LOGICAL_LABEL,
   "CentIntent":CentIntent::LOGICAL_LABEL,"CentRequest":CentRequest::LOGICAL_LABEL,"CentResult":CentResult::LOGICAL_LABEL,"CentLoan":CentLoan::LOGICAL_LABEL,
   "Sample":Sample::LOGICAL_LABEL,"BioRequest":BioRequest::LOGICAL_LABEL,"BioResult":BioResult::LOGICAL_LABEL,"BioProof":BioProof::LOGICAL_LABEL,
   "BioReport":BioReport::LOGICAL_LABEL,"Verification":Verification::LOGICAL_LABEL,"Quality":Quality::LOGICAL_LABEL,"QualityForOwner":QualityForOwner::LOGICAL_LABEL},
  "programs":[rows(&self.chemist),rows(&self.biologist),rows(&self.analyst),rows(&self.chemist_io),rows(&self.biologist_io),rows(&self.analyst_io),
   rows(&self.spec_service),rows(&self.cent_service),rows(&self.spectrometer),rows(&self.centrifuge)]})
    }
}
pub fn programs() -> Programs {
    let initial = g::par(analysis::<CHEMIST>(), separation::<BIOLOGIST>());
    let preparation = g::par(separation::<CHEMIST>(), analysis::<BIOLOGIST>());
    let assay = g::seq(
        g::send::<BIOLOGIST, CENT_SERVICE, BioRequest>(),
        g::par(
            g::send::<CENT_SERVICE, BIOLOGIST, BioResult>(),
            g::send::<CENT_SERVICE, SPEC_SERVICE, BioProof>(),
        ),
    );
    let quality = g::par(
        g::send::<CENT_SERVICE, SPEC_SERVICE, Verification>(),
        g::par(
            g::send::<ANALYST, CHEMIST, Quality>(),
            g::send::<ANALYST, SPEC_SERVICE, QualityForOwner>(),
        ),
    );
    let business = g::seq(
        initial,
        g::seq(
            preparation,
            g::seq(
                g::send::<CHEMIST, BIOLOGIST, Sample>(),
                g::seq(
                    assay,
                    g::seq(
                        g::send::<BIOLOGIST, ANALYST, BioReport>(),
                        g::seq(
                            analysis::<ANALYST>(),
                            g::seq(
                                separation::<ANALYST>(),
                                g::seq(quality, analysis::<CHEMIST>()),
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
                crate::shell::global::private_io::<ANALYST, ANALYST_IO>(),
            ),
        ),
    );
    Programs {
        chemist: project(&source),
        biologist: project(&source),
        analyst: project(&source),
        chemist_io: project(&source),
        biologist_io: project(&source),
        analyst_io: project(&source),
        spec_service: project(&source),
        cent_service: project(&source),
        spectrometer: project(&source),
        centrifuge: project(&source),
    }
}
