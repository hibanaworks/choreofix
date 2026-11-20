//! Payloads of actual review branches and agent-selected scientific rework.
use crate::shell::ShellError;
use hibana::runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload};
use serde_json::Value;

/// The same selection has distinct receiver contracts, so branches can notify concurrently.
pub struct Reanalysis<'a, const CHOICE: u8, const RECIPIENT: u8>(&'a [u8]);

impl<'a, const CHOICE: u8, const RECIPIENT: u8> Reanalysis<'a, CHOICE, RECIPIENT> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, ShellError> {
        Self::validate_payload(Payload::new(bytes)).map_err(|_| {
            ShellError::message("selected branch differs from the admitted reanalysis")
        })?;
        Ok(Self(bytes))
    }
}

impl<const CHOICE: u8, const RECIPIENT: u8> WireEncode for Reanalysis<'_, CHOICE, RECIPIENT> {
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, CodecError> {
        output
            .get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(self.0);
        Ok(self.0.len())
    }
}

impl<const CHOICE: u8, const RECIPIENT: u8> WirePayload for Reanalysis<'_, CHOICE, RECIPIENT> {
    const SCHEMA_ID: u32 = 0xac14_6000 | ((CHOICE as u32) << 8) | (RECIPIENT as u32);
    type Decoded<'a> = &'a [u8];
    fn validate_payload(payload: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(payload.as_bytes()).map_err(|_| CodecError::Malformed)?;
        if CHOICE > 3
            || value.as_object().is_none_or(|fields| fields.len() != 2)
            || value["biology"].as_bool() != Some(CHOICE & 1 != 0)
            || value["toxicology"].as_bool() != Some(CHOICE & 2 != 0)
        {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }
    fn decode_validated_payload(payload: ReceivedPayload<'_, Self>) -> &[u8] {
        payload.as_bytes()
    }
}
