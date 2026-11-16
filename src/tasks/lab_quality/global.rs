//! The QC operation used by the developing 13M choreography.
//! Both physical resource requests precede its one actual verification effect.
use hibana::{Endpoint, g, runtime::program::Projectable};

pub const INIT: u8 = 0;
pub const QC: u8 = 3;
pub const QC_SERVICE: u8 = 10;
pub const SPECTROMETER: u8 = 11;
pub const FUME_HOOD: u8 = 13;
pub const SESSION: u32 = 131;
pub type SpecLoan<'r> = g::Msg<1, Endpoint<'r, SPECTROMETER>>;
pub type FumeLoan<'r> = g::Msg<2, Endpoint<'r, FUME_HOOD>>;
pub type QcAdmission<'a> = g::Msg<3, &'a [u8]>;
pub type QcSpecRequest<'a> = g::Msg<4, &'a [u8]>;
pub type QcFumeRequest<'a> = g::Msg<5, &'a [u8]>;
pub type QcSpecResult<'a> = g::Msg<6, &'a [u8]>;
pub type QcFumeResult<'a> = g::Msg<7, &'a [u8]>;
pub type QcResult<'a> = g::Msg<8, &'a [u8]>;

pub fn qc_operation() -> impl Projectable {
    g::seq(
        g::send::<QC, QC_SERVICE, QcAdmission>(),
        g::seq(
            g::par(
                g::send::<SPECTROMETER, QC_SERVICE, QcSpecRequest>(),
                g::send::<FUME_HOOD, QC_SERVICE, QcFumeRequest>(),
            ),
            g::seq(
                g::par(
                    g::send::<QC_SERVICE, SPECTROMETER, QcSpecResult>(),
                    g::send::<QC_SERVICE, FUME_HOOD, QcFumeResult>(),
                ),
                g::send::<QC_SERVICE, QC, QcResult>(),
            ),
        ),
    )
}
