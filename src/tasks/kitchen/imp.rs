//! Exact recipe inputs and actual results; no protocol decisions or progress data.
use crate::shell::{ShellError, ShellResult};
use hibana::runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload};
use serde_json::{Value, json};
pub const CHEFS: [&str; 3] = ["CHEF_A", "CHEF_B", "CHEF_C"];
pub const DISHES: [&str; 3] = ["appetizer", "main_course", "dessert"];
pub const INGREDIENTS: [&str; 3] = ["sauce", "garnish", "glaze"];
pub const RECIPIENTS: [&str; 3] = ["CHEF_C", "CHEF_A", "CHEF_B"];
pub const RECEIVED: [&str; 3] = ["garnish", "glaze", "sauce"];
pub fn expected(index: u8) -> Option<Value> {
    let chef = usize::from(index % 3);
    match index / 3 {
        0 => Some(json!({"dish":DISHES[chef]})),
        1 => Some(json!({"ingredient":INGREDIENTS[chef],"for_chef":RECIPIENTS[chef]})),
        2 => Some(json!({"dish":DISHES[chef],"ingredient":RECEIVED[chef]})),
        _ => None,
    }
}
#[derive(Debug)]
pub struct Arguments<const INDEX: u8>(Vec<u8>);
impl<const INDEX: u8> Arguments<INDEX> {
    #[cfg(test)]
    pub fn from_bytes(bytes: &[u8]) -> ShellResult<Self> {
        Self::validate_payload(Payload::new(bytes))
            .map_err(|_| ShellError::message("request differs from the assigned recipe"))?;
        Ok(Self(bytes.to_vec()))
    }
    pub fn value(&self) -> Value {
        serde_json::from_slice(&self.0).expect("validated recipe request")
    }
}
impl<const INDEX: u8> WireEncode for Arguments<INDEX> {
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, CodecError> {
        output
            .get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(&self.0);
        Ok(self.0.len())
    }
}
impl<const INDEX: u8> WirePayload for Arguments<INDEX> {
    const SCHEMA_ID: u32 = 0xac12_0000 | INDEX as u32;
    type Decoded<'a> = Self;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        if expected(INDEX).is_none_or(|expected| expected != value) {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self {
        Self(input.as_bytes().to_vec())
    }
}
pub fn encode(value: &Value) -> ShellResult<Vec<u8>> {
    serde_json::to_vec(value).map_err(|e| ShellError::message(e.to_string()))
}
pub fn decode(bytes: &[u8]) -> ShellResult<Value> {
    serde_json::from_slice(bytes).map_err(|e| ShellError::message(e.to_string()))
}
pub fn completed(tool: &str, chef: &str, arguments: &Value, result: &Value) -> ShellResult<()> {
    let status = match tool {
        "prepare_base_dish" => "base dish prepared",
        "prepare_ingredient" => "ingredient prepared",
        "combine_dish" => "dish completed",
        _ => return Err(ShellError::message("unknown kitchen operation")),
    };
    if result["tool_name"] != tool
        || result["success"] != true
        || result["data"]["chef"] != chef
        || result["data"]["status"] != status
        || arguments.as_object().is_none_or(|args| {
            args.iter()
                .any(|(key, value)| result["data"][key] != *value)
        })
        || (tool == "combine_dish"
            && (result["data"]["base_event"].as_u64().is_none()
                || result["data"]["ingredient_event"].as_u64().is_none()
                || result["data"]["base_event"] == result["data"]["ingredient_event"]))
    {
        return Err(ShellError::message(
            "kitchen result differs from the actual admitted operation",
        ));
    }
    Ok(())
}
