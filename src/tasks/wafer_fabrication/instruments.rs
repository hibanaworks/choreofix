//! The integration engineer's typed requests to the four 15E instruments.
use choreofix::imp::json::encode;
use choreofix::shell::imp::review_reply::ReviewReply;
use choreofix::shell::{
    ShellResult,
    imp::service::{ReceivedEffects, SimulatorOperation},
};
use hibana::{
    Endpoint, g,
    runtime::{
        program::Projectable,
        wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload},
    },
};
use serde_json::Value;

pub const ALIGNER: u8 = 0;
pub const ENGINEER: u8 = 1;
pub const WAFER: u8 = 2;
pub const DEPOSITION_CHAMBER: u8 = 3;
pub const ETCH_CHAMBER: u8 = 4;
pub const STEPPER: u8 = 6;
pub const METROLOGY: u8 = 7;
pub const STAGE: u8 = 8;

pub type Film = g::Msg<20, InstrumentRequest<DEPOSITION_CHAMBER>>;
pub type FilmResult<'a> = g::Msg<21, &'a [u8]>;
pub type Uniformity = g::Msg<22, InstrumentRequest<ETCH_CHAMBER>>;
pub type UniformityResult<'a> = g::Msg<23, &'a [u8]>;

pub type Mount = g::Msg<24, InstrumentRequest<STEPPER>>;
pub type Mounted<'a> = g::Msg<25, &'a [u8]>;
pub type Measure = g::Msg<26, InstrumentRequest<METROLOGY>>;
pub type Measured<'a> = g::Msg<27, &'a [u8]>;
pub type OverlayPassed<'a> = g::Msg<28, ReviewReply<'a, true>>;
pub type OverlayFailed<'a> = g::Msg<29, ReviewReply<'a, false>>;

pub fn checks() -> impl Projectable {
    let film = g::seq(
        g::send::<WAFER, DEPOSITION_CHAMBER, Film>(),
        g::send::<DEPOSITION_CHAMBER, WAFER, FilmResult>(),
    );
    let etch = g::seq(
        g::send::<WAFER, ETCH_CHAMBER, Uniformity>(),
        g::send::<ETCH_CHAMBER, WAFER, UniformityResult>(),
    );
    g::route(g::seq(film, etch), g::seq(etch, film))
}

// The ChoreoFS codec preserves the actual file arguments sent to the instrument.
pub struct InstrumentRequest<const CHAMBER: u8>(Vec<u8>);

impl<const CHAMBER: u8> TryFrom<&[u8]> for InstrumentRequest<CHAMBER> {
    type Error = CodecError;

    fn try_from(bytes: &[u8]) -> Result<Self, CodecError> {
        Self::validate_payload(Payload::new(bytes))?;
        Ok(Self(bytes.to_vec()))
    }
}

impl<const CHAMBER: u8> WireEncode for InstrumentRequest<CHAMBER> {
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, CodecError> {
        output
            .get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(&self.0);
        Ok(self.0.len())
    }
}

impl<const CHAMBER: u8> WirePayload for InstrumentRequest<CHAMBER> {
    const SCHEMA_ID: u32 = 0xac15_0000 | CHAMBER as u32;
    type Decoded<'a> = ReceivedPayload<'a, Self>;

    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let args: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        let (field, name) = match CHAMBER {
            DEPOSITION_CHAMBER => ("process", "film_integrity_check"),
            ETCH_CHAMBER => ("process", "etch_uniformity_check"),
            STEPPER => ("process", "overlay_alignment"),
            METROLOGY => ("measurement", "overlay_alignment"),
            _ => return Err(CodecError::Malformed),
        };
        let process = args[field].as_str().ok_or(CodecError::Malformed)?;
        if args.as_object().is_some_and(|args| args.len() == 1)
            && process
                .to_lowercase()
                .replace(['-', ' '], "_")
                .trim_matches('_')
                == name
        {
            Ok(())
        } else {
            Err(CodecError::Malformed)
        }
    }

    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self::Decoded<'a> {
        input
    }
}

impl SimulatorOperation for InstrumentRequest<DEPOSITION_CHAMBER> {
    const RPC: (&'static str, &'static str) = ("INTEGRATION_ENGINEER", "run_deposition");
}

impl SimulatorOperation for InstrumentRequest<ETCH_CHAMBER> {
    const RPC: (&'static str, &'static str) = ("INTEGRATION_ENGINEER", "run_etch_process");
}

impl SimulatorOperation for InstrumentRequest<STEPPER> {
    const RPC: (&'static str, &'static str) = ("INTEGRATION_ENGINEER", "run_stepper_exposure");
}

impl SimulatorOperation for InstrumentRequest<METROLOGY> {
    const RPC: (&'static str, &'static str) = ("INTEGRATION_ENGINEER", "run_metrology_measurement");
}

pub async fn deposition<'run>(
    endpoint: &mut Endpoint<'run, DEPOSITION_CHAMBER>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    let result = effects.apply(endpoint.recv::<Film>().await?).await?;
    endpoint
        .send::<FilmResult>(&encode(&result)?.as_slice())
        .await?;
    Ok(())
}

pub async fn etching<'run>(
    endpoint: &mut Endpoint<'run, ETCH_CHAMBER>,
    effects: &impl ReceivedEffects,
) -> ShellResult<()> {
    let result = effects.apply(endpoint.recv::<Uniformity>().await?).await?;
    endpoint
        .send::<UniformityResult>(&encode(&result)?.as_slice())
        .await?;
    Ok(())
}
