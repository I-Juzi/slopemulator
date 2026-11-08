//! Names and records from the STS DLL/PDB. Records are not XML serializers.
//! Client-auth API fields do not establish their wire fields or state order.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthCommand {
    ConsumeGameToken,
    KeyData,
    LoginCancel,
    LoginFinish,
    LoginStart,
    LoginWebStart,
    LoginWebFinish,
    RequestCryptKey,
    RequestGameToken,
    RequestSession,
    RequestToken,
    LoginTokenStart,
    TokenKeyData,
}
impl AuthCommand {
    /// Wire names from `Cli2Auth::CommandToString`, STS DLL 0x180028020.
    pub fn wire_name(self) -> &'static str {
        match self {
            Self::ConsumeGameToken => "ConsumeGameToken",
            Self::KeyData => "KeyData",
            Self::LoginCancel => "LoginCancel",
            Self::LoginFinish => "LoginFinish",
            Self::LoginStart => "LoginStart",
            Self::LoginWebStart => "LoginWebStart",
            Self::LoginWebFinish => "LoginWebFinish",
            Self::RequestCryptKey => "RequestCryptKey",
            Self::RequestGameToken => "RequestGameToken",
            Self::RequestSession => "RequestSession",
            Self::RequestToken => "RequestToken",
            Self::LoginTokenStart => "LoginTokenStart",
            Self::TokenKeyData => "TokenKeyData",
        }
    }

    pub fn from_wire_name(name: &str) -> Option<Self> {
        const ALL: [AuthCommand; 13] = [
            AuthCommand::ConsumeGameToken,
            AuthCommand::KeyData,
            AuthCommand::LoginCancel,
            AuthCommand::LoginFinish,
            AuthCommand::LoginStart,
            AuthCommand::LoginWebStart,
            AuthCommand::LoginWebFinish,
            AuthCommand::RequestCryptKey,
            AuthCommand::RequestGameToken,
            AuthCommand::RequestSession,
            AuthCommand::RequestToken,
            AuthCommand::LoginTokenStart,
            AuthCommand::TokenKeyData,
        ];
        ALL.into_iter().find(|command| command.wire_name() == name)
    }
}

/// Checked XML request fields plus the routed destination header.
/// GUIDs remain text here; native GUID byte-order conversion is not specified.
/// No Debug implementation: records contain bearer-token data.
#[derive(Clone, PartialEq, Eq)]
pub struct ConsumeGameTokenRequest {
    pub portal_id_text: String,
    pub token_text: String,
    pub game_code: String,
    pub client_net_address: String,
}

/// Fields read by STS DLL 0x18000DD10. The parser can leave fields absent.
/// LobbyDaemon requires more than the library parser does.
#[derive(Clone, PartialEq, Eq)]
pub struct ConsumeGameTokenResponse {
    pub game_account_id_text: Option<String>,
    pub login_name: Option<String>,
    pub user_id_text: Option<String>,
    pub user_name: Option<String>,
    pub user_center: Option<String>,
    pub role_ids: Vec<u32>,
}
