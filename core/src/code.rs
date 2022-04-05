use crate::Message;
use bincode::{
    config::{Configuration, Fixint, LittleEndian, NoLimit, SkipFixedArrayLength},
    error::{DecodeError, EncodeError},
};

const CONFIG: Configuration<LittleEndian, Fixint, SkipFixedArrayLength, NoLimit> =
    bincode::config::standard()
        .with_fixed_int_encoding()
        .skip_fixed_array_length();

pub fn encode(message: &Message, buf: &mut Vec<u8>) -> Result<u32, EncodeError> {
    let len = bincode::encode_into_std_write(message, buf, CONFIG)? as u32;
    Ok(len)
}

pub fn decode(buf: &[u8]) -> Result<Message, DecodeError> {
    let (message, _) = bincode::decode_from_slice(buf, CONFIG)?;
    Ok(message)
}
