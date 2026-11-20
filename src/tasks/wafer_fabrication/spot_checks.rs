//! Final 15E spot checks on a single aligned reference wafer.
use choreofix::imp::json::encode;
use choreofix::shell::{
    ShellResult,
    global::Loan,
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

pub type Film = g::Msg<20, SpotCheck<DEPOSITION_CHAMBER>>;
pub type FilmResult<'a> = g::Msg<21, &'a [u8]>;
pub type Uniformity = g::Msg<22, SpotCheck<ETCH_CHAMBER>>;
pub type UniformityResult<'a> = g::Msg<23, &'a [u8]>;

pub fn source() -> impl Projectable {
    let film = g::seq(
        g::send::<WAFER, DEPOSITION_CHAMBER, Film>(),
        g::send::<DEPOSITION_CHAMBER, WAFER, FilmResult>(),
    );
    let etch = g::seq(
        g::send::<WAFER, ETCH_CHAMBER, Uniformity>(),
        g::send::<ETCH_CHAMBER, WAFER, UniformityResult>(),
    );
    g::seq(
        g::send::<ALIGNER, ENGINEER, Loan<WAFER>>(),
        g::seq(
            g::route(g::seq(film, etch), g::seq(etch, film)),
            g::send::<ENGINEER, ALIGNER, Loan<WAFER>>(),
        ),
    )
}

// The ChoreoFS codec preserves the actual file arguments sent to the instrument.
pub struct SpotCheck<const CHAMBER: u8>(Vec<u8>);

impl<const CHAMBER: u8> SpotCheck<CHAMBER> {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CodecError> {
        Self::validate_payload(Payload::new(bytes))?;
        Ok(Self(bytes.to_vec()))
    }
}

impl<const CHAMBER: u8> WireEncode for SpotCheck<CHAMBER> {
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, CodecError> {
        output
            .get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(&self.0);
        Ok(self.0.len())
    }
}

impl<const CHAMBER: u8> WirePayload for SpotCheck<CHAMBER> {
    const SCHEMA_ID: u32 = 0xac15_0000 | CHAMBER as u32;
    type Decoded<'a> = ReceivedPayload<'a, Self>;

    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let args: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        let name = match CHAMBER {
            DEPOSITION_CHAMBER => "film_integrity_check",
            ETCH_CHAMBER => "etch_uniformity_check",
            _ => return Err(CodecError::Malformed),
        };
        let process = args["process"].as_str().ok_or(CodecError::Malformed)?;
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

impl SimulatorOperation for SpotCheck<DEPOSITION_CHAMBER> {
    const RPC: (&'static str, &'static str) = ("INTEGRATION_ENGINEER", "run_deposition");
}

impl SimulatorOperation for SpotCheck<ETCH_CHAMBER> {
    const RPC: (&'static str, &'static str) = ("INTEGRATION_ENGINEER", "run_etch_process");
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
