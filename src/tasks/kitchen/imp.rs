//! Exact recipe inputs and actual results; no protocol decisions or progress data.
use crate::shell::{ShellError, ShellResult, imp::service::SimulatorOperation};
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
pub struct Arguments<const INDEX: u8, const SUBMISSION: bool = false>(Vec<u8>);
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
impl<const INDEX: u8, const SUBMISSION: bool> WireEncode for Arguments<INDEX, SUBMISSION> {
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
    type Decoded<'a> = ReceivedPayload<'a, Self>;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        if expected(INDEX).is_none_or(|expected| expected != value) {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self::Decoded<'a> {
        input
    }
}
// The initial file receive provides data for the equipment requests.
impl<const INDEX: u8> WirePayload for Arguments<INDEX, true> {
    const SCHEMA_ID: u32 = 0xac12_0100 | INDEX as u32;
    type Decoded<'a> = Arguments<INDEX>;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        Arguments::<INDEX>::validate_payload(input)
    }
    fn decode_validated_payload(input: ReceivedPayload<'_, Self>) -> Arguments<INDEX> {
        Arguments(input.as_bytes().to_vec())
    }
}
impl SimulatorOperation<2> for Arguments<0> {
    const RPC: (&'static str, &'static str) = (CHEFS[0], "prepare_base_dish");
}
impl SimulatorOperation<2> for Arguments<1> {
    const RPC: (&'static str, &'static str) = (CHEFS[1], "prepare_base_dish");
}
impl SimulatorOperation for Arguments<2> {
    const RPC: (&'static str, &'static str) = (CHEFS[2], "prepare_base_dish");
}
impl SimulatorOperation for Arguments<3> {
    const RPC: (&'static str, &'static str) = (CHEFS[0], "prepare_ingredient");
}
impl SimulatorOperation for Arguments<4> {
    const RPC: (&'static str, &'static str) = (CHEFS[1], "prepare_ingredient");
}
impl SimulatorOperation for Arguments<5> {
    const RPC: (&'static str, &'static str) = (CHEFS[2], "prepare_ingredient");
}
impl SimulatorOperation for Arguments<6> {
    const RPC: (&'static str, &'static str) = (CHEFS[0], "combine_dish");
}
impl SimulatorOperation for Arguments<7> {
    const RPC: (&'static str, &'static str) = (CHEFS[1], "combine_dish");
}
impl SimulatorOperation for Arguments<8> {
    const RPC: (&'static str, &'static str) = (CHEFS[2], "combine_dish");
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
