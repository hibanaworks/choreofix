//! Inspection consumes four actual resource requests and returns all four Endpoints.
use super::arguments::InspectionArguments;
use hibana::{g, runtime::program::Projectable};

pub use crate::shell::global::Loan;

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
pub const SAFETY_FILE: u8 = 36;

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
