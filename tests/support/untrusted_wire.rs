//! Raw wire input for exercising validation at the actual receiver.
pub(super) struct Untrusted<const SCHEMA: u32>(pub(super) Vec<u8>);
impl<const SCHEMA: u32> hibana::runtime::wire::WireEncode for Untrusted<SCHEMA> {
    fn encode_into(&self, out: &mut [u8]) -> Result<usize, hibana::runtime::wire::CodecError> {
        out.get_mut(..self.0.len())
            .ok_or(hibana::runtime::wire::CodecError::Truncated)?
            .copy_from_slice(&self.0);
        Ok(self.0.len())
    }
}
impl<const SCHEMA: u32> hibana::runtime::wire::WirePayload for Untrusted<SCHEMA> {
    const SCHEMA_ID: u32 = SCHEMA;
    type Decoded<'a> = &'a [u8];
    fn validate_payload(
        _: hibana::runtime::wire::Payload,
    ) -> Result<(), hibana::runtime::wire::CodecError> {
        Ok(())
    }
    fn decode_validated_payload<'a>(
        payload: hibana::runtime::wire::ReceivedPayload<'a, Self>,
    ) -> &'a [u8] {
        payload.as_bytes()
    }
}
