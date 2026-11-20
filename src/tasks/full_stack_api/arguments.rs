use hibana::runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload};
use serde_json::Value;

pub const MIGRATION: u8 = 1;
pub const DEFINITION: u8 = 2;
pub const IMPLEMENTATION: u8 = 3;
pub const FRONTEND: u8 = 4;
pub const BACKEND_REVIEW: u8 = 5;
pub const FRONTEND_REVIEW: u8 = 6;
pub const INTEGRATION: u8 = 7;
pub const REPAIR: u8 = 8;
pub const SCHEMA_REQUEST: u8 = 9;

/// The argument bytes of an admitted operation, including an optional HTTP method.
pub struct Arguments<const OPERATION: u8>(Vec<u8>);

impl<const OPERATION: u8> Arguments<OPERATION> {
    pub fn into_value(self) -> Value {
        serde_json::from_slice(&self.0).expect("validated API arguments")
    }
}

impl<const OPERATION: u8> WireEncode for Arguments<OPERATION> {
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, CodecError> {
        output
            .get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(&self.0);
        Ok(self.0.len())
    }
}

impl<const OPERATION: u8> WirePayload for Arguments<OPERATION> {
    const SCHEMA_ID: u32 = 0xac08_4d00 | OPERATION as u32;
    type Decoded<'a> = Self;

    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        let args = value.as_object().ok_or(CodecError::Malformed)?;
        let valid = match OPERATION {
            MIGRATION => {
                args.len() == 1
                    && args.get("migration_name").and_then(Value::as_str) == Some("feature_schema")
            }
            DEFINITION => {
                args.get("endpoint").and_then(Value::as_str) == Some("feature_api_spec")
                    && args.keys().all(|key| key == "endpoint" || key == "method")
                    && args.get("method").is_none_or(|method| {
                        method.as_str().is_some_and(|text| !text.trim().is_empty())
                    })
            }
            IMPLEMENTATION => {
                args.len() == 1
                    && args.get("component").and_then(Value::as_str) == Some("api_endpoints")
            }
            FRONTEND => {
                args.len() == 1
                    && args.get("component").and_then(Value::as_str) == Some("user_interface")
            }
            BACKEND_REVIEW | FRONTEND_REVIEW => {
                let target = if OPERATION == BACKEND_REVIEW {
                    "backend_api"
                } else {
                    "frontend_ui"
                };
                args.len() == 1 && args.get("pr_id").and_then(Value::as_str) == Some(target)
            }
            INTEGRATION => {
                args.len() == 1
                    && args.get("suite").and_then(Value::as_str) == Some("full_stack_integration")
            }
            REPAIR => {
                args.len() == 1
                    && matches!(
                        args.get("component").and_then(Value::as_str),
                        Some("api" | "backend" | "frontend" | "schema")
                    )
            }
            SCHEMA_REQUEST => {
                args.len() == 1
                    && args
                        .get("reason")
                        .and_then(Value::as_str)
                        .is_some_and(|reason| !reason.trim().is_empty())
            }
            _ => false,
        };
        if !valid {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }

    fn decode_validated_payload(input: ReceivedPayload<'_, Self>) -> Self {
        Self(input.as_bytes().to_vec())
    }
}
