//! Inspection consumes four actual resource requests and returns all four Endpoints.
use super::{
    arguments::{Arguments, InspectionArguments},
    imp::SynthesisTerminal,
};
use hibana::{g, runtime::program::Projectable};

pub use crate::shell::global::Loan;

pub const CHEMIST: u8 = 0;
pub const QC: u8 = 3;
pub const SAFETY_OFFICER: u8 = 4;
pub const SPEC_OWNER: u8 = 8;
pub const CENT_OWNER: u8 = 9;
pub const FUME_OWNER: u8 = 10;
pub const CHROMA_OWNER: u8 = 11;
pub const INSPECTION_SERVICE: u8 = 12;
pub const SPECTROMETER: u8 = 16;
pub const CENTRIFUGE: u8 = 17;
pub const FUME_HOOD: u8 = 18;
pub const CHROMATOGRAPH: u8 = 19;
pub const REAGENT_SUPPLY: u8 = 20;
pub const CHEM_FILE: u8 = 32;
pub const SAFETY_FILE: u8 = 36;
pub const CHEM_DATA: u8 = 96;

pub type FileInspection = g::Msg<1, InspectionArguments>;
pub type InspectionAdmission = g::Msg<2, InspectionArguments>;
pub type SpecInspection = g::Msg<3, InspectionArguments>;
pub type CentInspection = g::Msg<4, InspectionArguments>;
pub type FumeInspection = g::Msg<5, InspectionArguments>;
pub type ChromaInspection = g::Msg<6, InspectionArguments>;
pub type SpecResult<'a> = g::Msg<7, &'a [u8]>;
pub type CentResult<'a> = g::Msg<8, &'a [u8]>;
pub type FumeResult<'a> = g::Msg<9, &'a [u8]>;
pub type ChromaResult<'a> = g::Msg<10, &'a [u8]>;
pub type FileInspectionResult<'a> = g::Msg<11, &'a [u8]>;

pub type FileSynthesis = g::Msg<16, Arguments<0>>;
pub type SynthesisAdmission = g::Msg<17, Arguments<0>>;
pub type FumeSynthesis = g::Msg<18, Arguments<0>>;
pub type ReagentSynthesis = g::Msg<19, Arguments<0>>;
pub type FumeSynthesisResult<'a> = g::Msg<20, &'a [u8]>;
pub type ReagentSynthesisResult<'a> = g::Msg<21, &'a [u8]>;
pub type FileSynthesisResult<'a> = g::Msg<22, &'a [u8]>;
pub type SynthesisFinished = g::Msg<23, SynthesisTerminal<true>>;
pub type ReagentsExhausted = g::Msg<24, SynthesisTerminal<false>>;

/// Actual attempts repeat; finishing transfers the actual outcome and stock access to QC.
/// The world owns the material quantity. No application stock counter is introduced.
pub fn synthesis() -> impl Projectable {
    let attempt = g::seq(
        g::send::<CHEM_FILE, CHEMIST, FileSynthesis>(),
        g::seq(
            g::send::<CHEMIST, FUME_OWNER, SynthesisAdmission>(),
            g::seq(
                g::par(
                    g::send::<FUME_OWNER, CHEMIST, Loan<FUME_HOOD>>(),
                    g::send::<FUME_OWNER, CHEMIST, Loan<REAGENT_SUPPLY>>(),
                ),
                g::seq(
                    g::par(
                        g::send::<FUME_HOOD, FUME_OWNER, FumeSynthesis>(),
                        g::send::<REAGENT_SUPPLY, FUME_OWNER, ReagentSynthesis>(),
                    ),
                    g::seq(
                        g::par(
                            g::send::<FUME_OWNER, FUME_HOOD, FumeSynthesisResult>(),
                            g::send::<FUME_OWNER, REAGENT_SUPPLY, ReagentSynthesisResult>(),
                        ),
                        g::par(
                            g::par(
                                g::send::<CHEMIST, FUME_OWNER, Loan<FUME_HOOD>>(),
                                g::send::<CHEMIST, FUME_OWNER, Loan<REAGENT_SUPPLY>>(),
                            ),
                            g::send::<CHEMIST, CHEM_FILE, FileSynthesisResult>(),
                        ),
                    ),
                ),
            ),
        ),
    );
    g::seq(
        g::send::<CHEM_DATA, FUME_OWNER, Loan<CHEM_FILE>>(),
        g::seq(
            attempt.roll(),
            g::seq(
                g::route(
                    g::seq(
                        g::send::<FUME_OWNER, CHEMIST, SynthesisFinished>(),
                        g::send::<FUME_OWNER, QC, SynthesisFinished>(),
                    ),
                    g::seq(
                        g::send::<FUME_OWNER, CHEMIST, ReagentsExhausted>(),
                        g::send::<FUME_OWNER, QC, ReagentsExhausted>(),
                    ),
                ),
                g::send::<FUME_OWNER, QC, Loan<REAGENT_SUPPLY>>(),
            ),
        ),
    )
}

/// The file request, exclusive instrument loans, effect requests and actual replies.
/// This is a component of 13H, not the complete scientific task choreography.
pub fn inspection() -> impl Projectable {
    let loans = g::par(
        g::par(
            g::seq(
                g::send::<SAFETY_OFFICER, SPEC_OWNER, SpecInspection>(),
                g::send::<SPEC_OWNER, SAFETY_OFFICER, Loan<SPECTROMETER>>(),
            ),
            g::seq(
                g::send::<SAFETY_OFFICER, CENT_OWNER, CentInspection>(),
                g::send::<CENT_OWNER, SAFETY_OFFICER, Loan<CENTRIFUGE>>(),
            ),
        ),
        g::par(
            g::seq(
                g::send::<SAFETY_OFFICER, FUME_OWNER, FumeInspection>(),
                g::send::<FUME_OWNER, SAFETY_OFFICER, Loan<FUME_HOOD>>(),
            ),
            g::seq(
                g::send::<SAFETY_OFFICER, CHROMA_OWNER, ChromaInspection>(),
                g::send::<CHROMA_OWNER, SAFETY_OFFICER, Loan<CHROMATOGRAPH>>(),
            ),
        ),
    );
    let requests = g::par(
        g::par(
            g::send::<SPECTROMETER, INSPECTION_SERVICE, SpecInspection>(),
            g::send::<CENTRIFUGE, INSPECTION_SERVICE, CentInspection>(),
        ),
        g::par(
            g::send::<FUME_HOOD, INSPECTION_SERVICE, FumeInspection>(),
            g::send::<CHROMATOGRAPH, INSPECTION_SERVICE, ChromaInspection>(),
        ),
    );
    let replies = g::par(
        g::par(
            g::send::<INSPECTION_SERVICE, SPECTROMETER, SpecResult>(),
            g::send::<INSPECTION_SERVICE, CENTRIFUGE, CentResult>(),
        ),
        g::par(
            g::send::<INSPECTION_SERVICE, FUME_HOOD, FumeResult>(),
            g::send::<INSPECTION_SERVICE, CHROMATOGRAPH, ChromaResult>(),
        ),
    );
    let returns = g::par(
        g::par(
            g::send::<SAFETY_OFFICER, SPEC_OWNER, Loan<SPECTROMETER>>(),
            g::send::<SAFETY_OFFICER, CENT_OWNER, Loan<CENTRIFUGE>>(),
        ),
        g::par(
            g::send::<SAFETY_OFFICER, FUME_OWNER, Loan<FUME_HOOD>>(),
            g::send::<SAFETY_OFFICER, CHROMA_OWNER, Loan<CHROMATOGRAPH>>(),
        ),
    );
    g::seq(
        g::send::<SAFETY_FILE, SAFETY_OFFICER, FileInspection>(),
        g::seq(
            g::send::<SAFETY_OFFICER, INSPECTION_SERVICE, InspectionAdmission>(),
            g::seq(
                loans,
                g::seq(
                    requests,
                    g::seq(
                        replies,
                        g::par(
                            returns,
                            g::send::<SAFETY_OFFICER, SAFETY_FILE, FileInspectionResult>(),
                        ),
                    ),
                ),
            ),
        ),
    )
}
