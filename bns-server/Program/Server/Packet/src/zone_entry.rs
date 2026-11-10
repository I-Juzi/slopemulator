//! Build-381 zone-entry output codecs. Doc 10, 0x1404BEED0 / 0x1402304A0.
//! These encode authoritative action data. They do not authorize or finish spawn.
use bloodlib::packet::{self, PacketError, Value};
use packet_ids::GameCmd;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrepareZone {
    pub runtime_zone_id: u64,
    pub channel: u16,
    pub geo_zone_id: u32,
    pub correlation: u32,
    pub config_byte: u8,
    pub action_flag: u8,
    pub zone_block: Vec<u8>,
    pub position: [i16; 3],
    pub yaw: u16,
    pub pc_transition: u16,
    pub summon_transition: u16,
}
impl PrepareZone {
    /// Full packet, including size and command. No encryption or appearance data.
    pub fn encode(&self) -> Result<Vec<u8>, PacketError> {
        use Value::{Bytes, U16, U32, U64, U8};
        packet::compose(
            "hqhiiccKuuuhhh",
            &[
                U16(GameCmd::G_PrepareEnterZoneCmd.id() as u16),
                U64(self.runtime_zone_id),
                U16(self.channel),
                U32(self.geo_zone_id),
                U32(self.correlation),
                U8(self.config_byte),
                U8(self.action_flag),
                Bytes(self.zone_block.clone()),
                U16(self.position[0] as u16),
                U16(self.position[1] as u16),
                U16(self.position[2] as u16),
                U16(self.yaw),
                U16(self.pc_transition),
                U16(self.summon_transition),
            ],
        )
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnterZoneOk {
    pub context: u64,
}
impl EnterZoneOk {
    /// Native code sends this after appearance queueing but before later updates.
    /// This packet alone does not prove that the full spawn operation succeeded.
    pub fn encode(&self) -> Result<Vec<u8>, PacketError> {
        packet::compose(
            "hq",
            &[
                Value::U16(GameCmd::G_EnterZoneOkCmd.id() as u16),
                Value::U64(self.context),
            ],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn prepare() -> PrepareZone {
        PrepareZone {
            runtime_zone_id: 0x0807060504030201,
            channel: 9,
            geo_zone_id: 0x14131211,
            correlation: 0x24232221,
            config_byte: 0x81,
            action_flag: 0x82,
            zone_block: vec![0x31, 0x32, 0x33],
            position: [-1, -32768, 32767],
            yaw: 359,
            pc_transition: 0xABCD,
            summon_transition: 0xDCBA,
        }
    }
    #[test]
    fn prepare_zone_all_fields_in_native_order() {
        assert_eq!(
            prepare().encode().unwrap(),
            vec![
                41, 0, 41, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 0x11, 0x12, 0x13, 0x14, 0x21, 0x22,
                0x23, 0x24, 0x81, 0x82, 3, 0, 0x31, 0x32, 0x33, 255, 255, 0, 128, 255, 127, 103, 1,
                0xCD, 0xAB, 0xBA, 0xDC
            ]
        );
    }
    #[test]
    fn prepare_zone_capacity_and_missing_summon_counter() {
        let mut p = prepare();
        p.summon_transition = 0;
        p.zone_block = vec![0; 3962];
        let bytes = p.encode().unwrap();
        assert_eq!(bytes.len(), 4000);
        assert_eq!(&bytes[3998..], &[0, 0]);
        p.zone_block.push(0);
        assert!(p.encode().is_err());
        p.zone_block = vec![0; 32768];
        assert!(p.encode().is_err());
    }
    #[test]
    fn enter_zone_ok_is_only_a_context_qword() {
        assert_eq!(
            EnterZoneOk {
                context: 0x0807060504030201
            }
            .encode()
            .unwrap(),
            vec![12, 0, 43, 0, 1, 2, 3, 4, 5, 6, 7, 8]
        );
    }
}
