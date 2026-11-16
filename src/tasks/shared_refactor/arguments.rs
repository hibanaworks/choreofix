//! Original operation arguments for one assigned module or shared library.
use hibana::runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload};
use serde_json::Value;

pub struct Arguments<const OP: u8>(Vec<u8>);
impl<const OP: u8> Arguments<OP> {
    pub fn into_value(self) -> Value {
        serde_json::from_slice(&self.0).expect("validated shared-refactor arguments")
    }
}
impl<const OP: u8> TryFrom<&[u8]> for Arguments<OP> {
    type Error = CodecError;
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::validate_payload(Payload::new(bytes))?;
        Ok(Self(bytes.to_vec()))
    }
}
impl<const OP: u8> WireEncode for Arguments<OP> {
    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        out.get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(&self.0);
        Ok(self.0.len())
    }
}
impl<const OP: u8> WirePayload for Arguments<OP> {
    const SCHEMA_ID: u32 = 0xac04_4000 | OP as u32;
    type Decoded<'a> = Self;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        let (key, target) = match OP {
            0 => ("title", "module_a_refactor"),
            1 => ("title", "module_b_refactor"),
            2 => ("target", "module_a"),
            3 | 5 => ("target", "SHARED_LIBRARY"),
            4 => ("target", "module_b"),
            6 => ("pr_id", "module_a"),
            7 => ("pr_id", "module_b"),
            8 => ("suite", "module_a_tests"),
            9 => ("suite", "module_b_tests"),
            10 => ("branch", "module_a"),
            11 => ("branch", "module_b"),
            _ => return Err(CodecError::Malformed),
        };
        if value[key] != target || value.as_object().is_none_or(|v| v.len() != 1) {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self {
        Self(input.as_bytes().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn check<const OP: u8>(key: &str, target: &str) {
        for (value, valid) in [
            (json!({key:target}), true),
            (json!({key:"other"}), false),
            (json!({key:target,"next":true}), false),
            (json!({}), false),
        ] {
            let bytes = serde_json::to_vec(&value).unwrap();
            assert_eq!(
                Arguments::<OP>::validate_payload(Payload::new(&bytes)).is_ok(),
                valid
            );
        }
    }
    #[test]
    fn every_original_argument_contract_rejects_target_substitution() {
        check::<0>("title", "module_a_refactor");
        check::<1>("title", "module_b_refactor");
        check::<2>("target", "module_a");
        check::<3>("target", "SHARED_LIBRARY");
        check::<4>("target", "module_b");
        check::<5>("target", "SHARED_LIBRARY");
        check::<6>("pr_id", "module_a");
        check::<7>("pr_id", "module_b");
        check::<8>("suite", "module_a_tests");
        check::<9>("suite", "module_b_tests");
        check::<10>("branch", "module_a");
        check::<11>("branch", "module_b");
    }
}
