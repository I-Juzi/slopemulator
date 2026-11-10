//! Additive build-381 request codecs for docs 10 and 11.
//! This does not change `game::GamePacket` or its strict trailing-byte policy.
//! Handler prefix decoders permit extra bytes. Unused bodies stay opaque.
//! `Q` decodes to I8; c to U8. Repeats use two-byte unsigned counts.
// The command consts keep the official names (for example `G_HelloReqCmd`).
#![allow(non_upper_case_globals)]

use bloodlib::packet::{self, DecodeLimits, PacketError, Value};
use packet_ids::GameCmd;

/// Command IDs as `u16` consts. Each const has the official name of its `GameCmd` variant,
/// so that a name in the code, the docs and IDA is the same text.
macro_rules! u16_commands {
    ($($name:ident),* $(,)?) => {
        $(
            const $name: u16 = GameCmd::$name.id() as u16;
        )*
    };
}

u16_commands!(
    G_HelloReqCmd,
    G_MobileHelloReqCmd,
    G_PingReqCmd,
    G_ServerPingReqCmd,
    G_GameKeyAuthReqCmd,
    G_NPTokenAuthReqCmd,
    G_LobbyTokenAuthReqCmd,
    G_ReturnToLobbyReqCmd,
    G_CleanUserConnectionInfoInCligateReqCmd,
    G_EnterWorldReqCmd,
    G_QueryWaitingNumberReqCmd,
    G_LeaveWorldReqCmd,
    G_EnterZoneReqCmd,
    G_MoveReqCmd,
    G_TurnReqCmd,
    G_ActivatedTeleportListReqCmd,
    G_DetectTeleportSwitchReqCmd,
    G_ReturnToBaseCampReqCmd,
    G_ClearLogoutPartyMemberReqCmd,
    G_ClearLogoutTeamMemberReqCmd,
    G_EnterPortalReqCmd,
    G_TurnBackFromPortalReqCmd,
    G_ChannelListReqCmd,
    G_ChangeChannelReqCmd,
    G_SaveClientOptionReqCmd,
    G_ClientConfigReqCmd,
    G_AutoLogoutCheckReqCmd,
    G_CheckRoundTripTimeReqCmd,
    G_WriteLatencyLogReqCmd,
    G_CurrentUTCTimeReqCmd,
    G_ReturnToGameReqCmd,
    G_ExtendJumpingCharacterCreationQuotaWithWarehouseItemReqCmd,
    G_UpgradeJumpingCharacterStateWithCreationQuotaReqCmd,
    G_QQLogoutReqCmd,
    G_ContentsRuleListReqCmd,
    G_EnterZoneDirectReqCmd,
    G_LeaveZoneDirectReqCmd,
    G_UseJumpingCharacterTicketReqCmd,
    G_ClientRunReqCmd,
    G_AirDashReqCmd,
    G_WarpToPosReqCmd,
    G_UseTeleportReqCmd,
    G_UseCaveEscapeReqCmd,
);

pub const COMMANDS: [u16; 43] = [
    G_HelloReqCmd,
    G_MobileHelloReqCmd,
    G_PingReqCmd,
    G_ServerPingReqCmd,
    G_GameKeyAuthReqCmd,
    G_NPTokenAuthReqCmd,
    G_LobbyTokenAuthReqCmd,
    G_ReturnToLobbyReqCmd,
    G_CleanUserConnectionInfoInCligateReqCmd,
    G_EnterWorldReqCmd,
    G_QueryWaitingNumberReqCmd,
    G_LeaveWorldReqCmd,
    G_EnterZoneReqCmd,
    G_MoveReqCmd,
    G_TurnReqCmd,
    G_ActivatedTeleportListReqCmd,
    G_DetectTeleportSwitchReqCmd,
    G_ReturnToBaseCampReqCmd,
    G_ClearLogoutPartyMemberReqCmd,
    G_ClearLogoutTeamMemberReqCmd,
    G_EnterPortalReqCmd,
    G_TurnBackFromPortalReqCmd,
    G_ChannelListReqCmd,
    G_ChangeChannelReqCmd,
    G_SaveClientOptionReqCmd,
    G_ClientConfigReqCmd,
    G_AutoLogoutCheckReqCmd,
    G_CheckRoundTripTimeReqCmd,
    G_WriteLatencyLogReqCmd,
    G_CurrentUTCTimeReqCmd,
    G_ReturnToGameReqCmd,
    G_ExtendJumpingCharacterCreationQuotaWithWarehouseItemReqCmd,
    G_UpgradeJumpingCharacterStateWithCreationQuotaReqCmd,
    G_QQLogoutReqCmd,
    G_ContentsRuleListReqCmd,
    G_EnterZoneDirectReqCmd,
    G_LeaveZoneDirectReqCmd,
    G_UseJumpingCharacterTicketReqCmd,
    G_ClientRunReqCmd,
    G_AirDashReqCmd,
    G_WarpToPosReqCmd,
    G_UseTeleportReqCmd,
    G_UseCaveEscapeReqCmd,
];

/// None is an unused body, not an exact-length check.
pub fn format(command: u16) -> Option<&'static str> {
    Some(match command {
        G_HelloReqCmd => "hhqsqb",
        G_MobileHelloReqCmd => "hhsChh;",
        G_ServerPingReqCmd | G_CheckRoundTripTimeReqCmd => "hqq",
        G_LobbyTokenAuthReqCmd => "hhqq",
        G_EnterWorldReqCmd => "hic",
        G_EnterZoneReqCmd => "hiuuuh",
        G_MoveReqCmd => "hcuuuuuuiiiiiihhhcCQcccccch;Chhh;",
        G_TurnReqCmd => "hch",
        G_DetectTeleportSwitchReqCmd | G_ChangeChannelReqCmd | G_UseCaveEscapeReqCmd => "hh",
        G_TurnBackFromPortalReqCmd => "huuuh",
        G_SaveClientOptionReqCmd => "hQb",
        G_WriteLatencyLogReqCmd => "hQiiiii",
        G_ExtendJumpingCharacterCreationQuotaWithWarehouseItemReqCmd => "hqqi",
        G_EnterZoneDirectReqCmd => "hQqcChh;",
        G_LeaveZoneDirectReqCmd => "h",
        G_UseJumpingCharacterTicketReqCmd => "hqqqc",
        G_ClientRunReqCmd => "hcc",
        G_AirDashReqCmd => "hq",
        G_UseTeleportReqCmd => "hhh",
        _ => return None,
    })
}

#[derive(Clone, Debug, PartialEq)]
pub enum Body {
    /// Values after the command. Signed bits follow BloodLib's format codes.
    Fields(Vec<Value>),
    /// A no-decoder handler must accept this without interpreting it.
    Unused(Vec<u8>),
}
#[derive(Clone, Debug, PartialEq)]
pub struct Request {
    pub command: u16,
    pub body: Body,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Packet(PacketError),
    Unsupported(u16),
    WrongBody,
}
impl From<PacketError> for Error {
    fn from(value: PacketError) -> Self {
        Self::Packet(value)
    }
}
impl Request {
    pub fn decode(input: &[u8]) -> Result<Self, Error> {
        packet::packet_size(input, 4)?;
        let command = u16::from_le_bytes([input[2], input[3]]);
        if !COMMANDS.contains(&command) {
            return Err(Error::Unsupported(command));
        }
        let Some(fmt) = format(command) else {
            return Ok(Self {
                command,
                body: Body::Unused(input[4..].to_vec()),
            });
        };
        let limits = DecodeLimits {
            blob_bytes: 4000,
            string_units: if matches!(command, G_HelloReqCmd | G_MobileHelloReqCmd) {
                40
            } else {
                usize::MAX
            },
            ..DecodeLimits::default()
        };
        let mut values = packet::decompose(fmt, input, limits)?;
        values.remove(0);
        Ok(Self {
            command,
            body: Body::Fields(values),
        })
    }
    pub fn encode(&self) -> Result<Vec<u8>, Error> {
        if !COMMANDS.contains(&self.command) {
            return Err(Error::Unsupported(self.command));
        }
        match (format(self.command), &self.body) {
            (Some(fmt), Body::Fields(body)) => {
                let mut values = vec![Value::U16(self.command)];
                values.extend(body.clone());
                Ok(packet::compose(fmt, &values)?)
            }
            (None, Body::Unused(body)) => {
                if body.len() > packet::MAX_PACKET - 4 {
                    return Err(PacketError::ComposeOverflow.into());
                }
                let mut out = ((body.len() + 4) as u16).to_le_bytes().to_vec();
                out.extend(self.command.to_le_bytes());
                out.extend(body);
                Ok(out)
            }
            _ => Err(Error::WrongBody),
        }
    }
}

/// Exact verified backend descriptors. No completion/persistence claim.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackendRequest {
    LobbyClean {
        db_id: u64,
    },
    ReturnToGame {
        db_id: u64,
        object_id: u64,
    },
    SaveClientOption {
        db_id: u64,
        option_type: u8,
        data: Vec<u8>,
    },
}
impl BackendRequest {
    pub fn encode(&self) -> Result<Vec<u8>, PacketError> {
        use Value::{Bytes, Repeat, I8, U16, U64, U8};
        match self {
            Self::LobbyClean { db_id } => {
                packet::compose("hqQQ", &[U16(17), U64(*db_id), I8(0), I8(1)])
            }
            Self::ReturnToGame { db_id, object_id } => {
                packet::compose("hQqq", &[U16(271), I8(2), U64(*db_id), U64(*object_id)])
            }
            Self::SaveClientOption {
                db_id,
                option_type,
                data,
            } => {
                let len = i16::try_from(data.len()).map_err(|_| PacketError::LengthRange)?;
                let mut nested = 2i16.to_le_bytes().to_vec();
                nested.extend(len.to_le_bytes());
                nested.extend(data);
                packet::compose(
                    "hQCqqchb;",
                    &[
                        U16(17),
                        I8(0),
                        Repeat(vec![vec![
                            U64(*db_id),
                            U64(*option_type as i8 as i64 as u64),
                            U8(14),
                            U16(2),
                            Bytes(nested),
                        ]]),
                    ],
                )
            }
        }
    }
}
