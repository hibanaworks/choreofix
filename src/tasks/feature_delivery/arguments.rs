//! Canonical bytes for one operation on one assigned development target.
use hibana::runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload};
use serde_json::Value;

pub struct Arguments<const OPERATION: u8>(Vec<u8>);
impl<const OPERATION: u8> Arguments<OPERATION> {
    pub fn into_value(self) -> Value {
        serde_json::from_slice(&self.0).expect("validated feature-delivery arguments")
    }
}
impl<const OPERATION: u8> WireEncode for Arguments<OPERATION> {
    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        out.get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(&self.0);
        Ok(self.0.len())
    }
}
impl<const OPERATION: u8> WirePayload for Arguments<OPERATION> {
    const SCHEMA_ID: u32 = 0xac06_2000 | OPERATION as u32;
    type Decoded<'a> = Self;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        let (key, target) = match OPERATION {
            0 => ("feature_name", "user_authentication"),
            1 => ("feature_name", "payment_processing"),
            2 => ("branch", "user_authentication"),
            3 => ("branch", "payment_processing"),
            4 => ("pr_id", "user_authentication"),
            5 => ("pr_id", "payment_processing"),
            6 => ("target", "user_authentication"),
            7 => ("target", "payment_processing"),
            8 => ("suite", "full_test_suite_after_auth"),
            9 => ("suite", "full_test_suite_after_payment"),
            _ => return Err(CodecError::Malformed),
        };
        let object = value.as_object().ok_or(CodecError::Malformed)?;
        if value[key] != target
            || object
                .keys()
                .any(|k| k != key && !(OPERATION <= 1 && k == "source"))
            || object
                .get("source")
                .is_some_and(|v| v.as_str().is_none_or(|v| v.trim().is_empty()))
        {
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
    use super::Arguments;
    use hibana::runtime::wire::{Payload, WirePayload};
    use serde_json::json;
    fn check<const OP: u8>(key: &str, target: &str) {
        let valid = serde_json::to_vec(&json!({key:target})).unwrap();
        assert!(Arguments::<OP>::validate_payload(Payload::new(&valid)).is_ok());
        for invalid in [
            json!({key:"another target"}),
            json!({key:target,"unexpected":true}),
            json!({key:12}),
            json!({}),
        ] {
            let bytes = serde_json::to_vec(&invalid).unwrap();
            assert!(Arguments::<OP>::validate_payload(Payload::new(&bytes)).is_err());
        }
        let source = serde_json::to_vec(&json!({key:target,"source":"actual revision"})).unwrap();
        assert_eq!(
            Arguments::<OP>::validate_payload(Payload::new(&source)).is_ok(),
            OP < 2
        );
        for source in [json!(" "), json!(null), json!([])] {
            let bytes = serde_json::to_vec(&json!({key:target,"source":source})).unwrap();
            assert!(Arguments::<OP>::validate_payload(Payload::new(&bytes)).is_err());
        }
    }
    #[test]
    fn feature_delivery_assigned_operations_reject_target_substitution_before_sending() {
        check::<0>("feature_name", "user_authentication");
        check::<1>("feature_name", "payment_processing");
        check::<2>("branch", "user_authentication");
        check::<3>("branch", "payment_processing");
        check::<4>("pr_id", "user_authentication");
        check::<5>("pr_id", "payment_processing");
        check::<6>("target", "user_authentication");
        check::<7>("target", "payment_processing");
        check::<8>("suite", "full_test_suite_after_auth");
        check::<9>("suite", "full_test_suite_after_payment");
    }
}
