mod code;

pub use crate::code::{decode, encode};
use bincode::{Decode, Encode};

#[derive(Decode, Encode)]
pub enum Message {
    Auth { name: String },
    Text(String),
}
