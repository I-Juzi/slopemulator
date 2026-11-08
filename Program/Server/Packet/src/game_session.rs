//! Additive build-381 session codecs. No framing encryption or transaction policy.
//! Sources: docs 10, 0x14014CCB0 / 0x14014ED65..0x14014EDA7.
use bloodlib::packet::{self, DecodeLimits, PacketError, Value};

/// Late state-16 result. This codec does not authorize or complete token auth.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LobbyTokenAuthOk {
    pub charge_apply_type: u8,
    pub remaining_play_time: i32,
    pub charge_base_type: u8,
    pub charge_item_id: i32,
    pub reservations: u16,
}
impl LobbyTokenAuthOk {
    pub fn encode(&self) -> Result<Vec<u8>, PacketError> {
        use Value::{I32, I8, U16};
        packet::compose(
            "hQiQih",
            &[
                U16(24),
                I8(self.charge_apply_type as i8),
                I32(self.remaining_play_time),
                I8(self.charge_base_type as i8),
                I32(self.charge_item_id),
                U16(self.reservations),
            ],
        )
    }
    pub fn decode(input: &[u8]) -> Result<Self, PacketError> {
        packet::packet_size(input, 16)?;
        // This fixed result has 12 body bytes plus size/command. No trailing fields.
        if input.len() != 16 {
            return Err(PacketError::PacketLength);
        }
        let fields = packet::decompose("hQiQih", input, DecodeLimits::default())?;
        match fields.as_slice() {
            [Value::I16(24), Value::I8(apply), Value::U32(time), Value::I8(base), Value::U32(item), Value::I16(reservations)] => {
                Ok(Self {
                    charge_apply_type: *apply as u8,
                    remaining_play_time: *time as i32,
                    charge_base_type: *base as u8,
                    charge_item_id: *item as i32,
                    reservations: *reservations as u16,
                })
            }
            _ => Err(PacketError::ValueType('h')),
        }
    }
}

/// Lobby command 5. Routing world and accepted-connection ownership stay external.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LobbyConsumeToken {
    pub transaction_id: u64,
    pub client_key: u64,
    pub lobby_token: u64,
    pub accepted_address_bits: u32,
}
impl LobbyConsumeToken {
    pub fn encode(&self) -> Result<Vec<u8>, PacketError> {
        use Value::{U16, U32, U64};
        packet::compose(
            "hqqqi",
            &[
                U16(5),
                U64(self.transaction_id),
                U64(self.client_key),
                U64(self.lobby_token),
                U32(self.accepted_address_bits),
            ],
        )
    }
}
