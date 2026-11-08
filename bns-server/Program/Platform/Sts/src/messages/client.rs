//! XML bodies from the 2020 client spec, section 07.
//! ❓ Reply root `Reply` and success status `200 OK` need a real-client check.
//! IDs 3101..3114 are library reply IDs. They are not status codes.
use crate::{key_data, Frame, StartLine, Transaction};
use quick_xml::{events::Event, Reader};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Xml,
    Field,
    Guid,
    KeyData,
    Frame,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "STS XML error: {self:?}")
    }
}
impl std::error::Error for Error {}

/// Bytes use Windows GUID memory order. Text uses registry order.
pub fn guid_text(bytes: [u8; 16]) -> String {
    let b = [
        bytes[3], bytes[2], bytes[1], bytes[0], bytes[5], bytes[4], bytes[7], bytes[6], bytes[8],
        bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
    ];
    format!("{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}", b[0],b[1],b[2],b[3],b[4],b[5],b[6],b[7],b[8],b[9],b[10],b[11],b[12],b[13],b[14],b[15])
}
pub fn parse_guid(text: &str) -> Result<[u8; 16], Error> {
    if text.len() != 36 {
        return Err(Error::Guid);
    }
    let mut b = [0; 16];
    let mut pos = 0;
    for (i, byte) in b.iter_mut().enumerate() {
        if [4, 6, 8, 10].contains(&i) {
            if text.as_bytes()[pos] != b'-' {
                return Err(Error::Guid);
            }
            pos += 1;
        }
        let pair = text.as_bytes().get(pos..pos + 2).ok_or(Error::Guid)?;
        let pair = std::str::from_utf8(pair).map_err(|_| Error::Guid)?;
        if !pair.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(Error::Guid);
        }
        *byte = u8::from_str_radix(pair, 16).map_err(|_| Error::Guid)?;
        pos += 2;
    }
    b[..4].reverse();
    b[4..6].reverse();
    b[6..8].reverse();
    Ok(b)
}
fn xml_char(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\r' | '\u{20}'..='\u{d7ff}' | '\u{e000}'..='\u{fffd}' | '\u{10000}'..='\u{10ffff}')
}
fn tag(name: &str) -> bool {
    !name.is_empty()
        && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        && name.as_bytes()[0].is_ascii_alphabetic()
}
pub(crate) fn field(out: &mut String, name: &str, value: &str) -> Result<(), Error> {
    if !tag(name) || !value.chars().all(xml_char) {
        return Err(Error::Xml);
    }
    out.push('<');
    out.push_str(name);
    out.push('>');
    out.push_str(&quick_xml::escape::escape(value));
    out.push_str("</");
    out.push_str(name);
    out.push_str(">\n");
    Ok(())
}
#[derive(Default)]
pub(crate) struct Node {
    name: String,
    text: String,
    children: Vec<Node>,
}
impl Node {
    pub(crate) fn only(&self, names: &[&str]) -> Result<(), Error> {
        if !self.text.trim().is_empty()
            || self
                .children
                .iter()
                .any(|n| !names.contains(&n.name.as_str()))
        {
            return Err(Error::Field);
        }
        for name in names {
            if self.children.iter().filter(|n| n.name == *name).count() > 1 {
                return Err(Error::Field);
            }
        }
        Ok(())
    }
    fn child(&self, name: &str) -> Option<&Node> {
        self.children.iter().find(|n| n.name == name)
    }
    fn optional(&self, name: &str) -> Result<Option<&str>, Error> {
        self.child(name)
            .map(|n| {
                if n.children.is_empty() {
                    Ok(n.text.as_str())
                } else {
                    Err(Error::Field)
                }
            })
            .transpose()
    }
    pub(crate) fn value(&self, name: &str) -> Result<&str, Error> {
        self.optional(name)?.ok_or(Error::Field)
    }
}
pub(crate) fn parse_xml(bytes: &[u8], root_name: &str) -> Result<Node, Error> {
    if bytes.len() > 65_536 {
        return Err(Error::Xml);
    }
    let text = std::str::from_utf8(bytes).map_err(|_| Error::Xml)?;
    if !text.chars().all(xml_char) {
        return Err(Error::Xml);
    }
    let mut reader = Reader::from_str(text);
    reader.config_mut().expand_empty_elements = true;
    let mut stack: Vec<Node> = Vec::new();
    let mut root = None;
    let mut nodes = 0;
    loop {
        match reader.read_event().map_err(|_| Error::Xml)? {
            Event::Start(e) | Event::Empty(e) => {
                // Empty events are expanded below through Reader configuration.
                let name = std::str::from_utf8(e.name().as_ref())
                    .map_err(|_| Error::Xml)?
                    .to_owned();
                if !tag(&name)
                    || e.attributes().next().is_some()
                    || stack.len() >= 8
                    || nodes >= 128
                    || (stack.is_empty() && root.is_some())
                {
                    return Err(Error::Xml);
                }
                nodes += 1;
                stack.push(Node {
                    name,
                    ..Node::default()
                });
            }
            Event::End(_) => {
                let n = stack.pop().ok_or(Error::Xml)?;
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(n);
                } else if root.replace(n).is_some() {
                    return Err(Error::Xml);
                }
            }
            Event::Text(e) => {
                let value = e.xml_content().map_err(|_| Error::Xml)?;
                if let Some(n) = stack.last_mut() {
                    n.text.push_str(&value);
                } else if !value.trim().is_empty() {
                    return Err(Error::Xml);
                }
            }
            Event::CData(e) => {
                stack
                    .last_mut()
                    .ok_or(Error::Xml)?
                    .text
                    .push_str(&e.xml_content().map_err(|_| Error::Xml)?);
            }
            Event::GeneralRef(e) => {
                let name = e.decode().map_err(|_| Error::Xml)?;
                let value = if let Some(c) = e.resolve_char_ref().map_err(|_| Error::Xml)? {
                    c.to_string()
                } else {
                    match name.as_ref() {
                        "amp" => "&",
                        "lt" => "<",
                        "gt" => ">",
                        "quot" => "\"",
                        "apos" => "'",
                        _ => return Err(Error::Xml),
                    }
                    .into()
                };
                if !value.chars().all(xml_char) {
                    return Err(Error::Xml);
                }
                stack.last_mut().ok_or(Error::Xml)?.text.push_str(&value);
            }
            Event::Decl(_) if root.is_none() && stack.is_empty() => {}
            Event::Eof => break,
            _ => return Err(Error::Xml),
        }
    }
    if !stack.is_empty() {
        return Err(Error::Xml);
    }
    let root = root.ok_or(Error::Xml)?;
    if root.name != root_name {
        return Err(Error::Xml);
    }
    Ok(root)
}
fn unsigned(text: &str) -> Result<u32, Error> {
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err(Error::Field);
    }
    text.parse().map_err(|_| Error::Field)
}
fn signed(text: &str) -> Result<i32, Error> {
    text.parse().map_err(|_| Error::Field)
}

/// No Debug: login names, proofs and token data must not enter logs.
#[derive(Clone, PartialEq, Eq)]
pub enum Request {
    LoginStart {
        login_name: String,
    },
    KeyData {
        public: Vec<u8>,
        proof: Vec<u8>,
    },
    LoginFinish {
        language: u32,
    },
    RequestGameToken {
        game_code: String,
        account_alias: String,
    },
    RequestToken {
        app_id: String,
    },
    LogoutMyClient,
    GetMyUserInfo,
    /// `GameAccount/ListMyAccounts` (library ID 3106). Builder
    /// StsMsgCli32.dll 0.122 `sub_10023F10`: one `GameCode` element.
    ListMyAccounts {
        game_code: String,
    },
}
impl Request {
    pub fn protocol(&self) -> &'static str {
        match self {
            Self::ListMyAccounts { .. } => "GameAccount",
            _ => "Auth",
        }
    }
    pub fn command(&self) -> &'static str {
        match self {
            Self::LoginStart { .. } => "LoginStart",
            Self::KeyData { .. } => "KeyData",
            Self::LoginFinish { .. } => "LoginFinish",
            Self::RequestGameToken { .. } => "RequestGameToken",
            Self::RequestToken { .. } => "RequestToken",
            Self::LogoutMyClient => "LogoutMyClient",
            Self::GetMyUserInfo => "GetMyUserInfo",
            Self::ListMyAccounts { .. } => "ListMyAccounts",
        }
    }
    pub fn body(&self) -> Result<Vec<u8>, Error> {
        let mut out = String::from("<Request>\n");
        match self {
            Self::LoginStart { login_name } => field(&mut out, "LoginName", login_name)?,
            Self::KeyData { public, proof } => field(
                &mut out,
                "KeyData",
                &key_data::encode_client(public, proof).map_err(|_| Error::KeyData)?,
            )?,
            Self::LoginFinish { language } => field(&mut out, "Language", &language.to_string())?,
            Self::RequestGameToken {
                game_code,
                account_alias,
            } => {
                field(&mut out, "GameCode", game_code)?;
                field(&mut out, "AccountAlias", account_alias)?;
            }
            Self::RequestToken { app_id } => field(&mut out, "AppId", app_id)?,
            Self::ListMyAccounts { game_code } => field(&mut out, "GameCode", game_code)?,
            Self::LogoutMyClient | Self::GetMyUserInfo => return Ok(b"<Request/>".to_vec()),
        }
        out.push_str("</Request>\n");
        Ok(out.into_bytes())
    }
    pub fn parse(command: &str, body: &[u8]) -> Result<Self, Error> {
        let n = parse_xml(body, "Request")?;
        Ok(match command {
            "LoginStart" => {
                n.only(&["LoginName"])?;
                Self::LoginStart {
                    login_name: n.value("LoginName")?.into(),
                }
            }
            "KeyData" => {
                n.only(&["KeyData"])?;
                let (public, proof) =
                    key_data::decode_client(n.value("KeyData")?).map_err(|_| Error::KeyData)?;
                Self::KeyData { public, proof }
            }
            "LoginFinish" => {
                n.only(&["Language"])?;
                Self::LoginFinish {
                    language: unsigned(n.value("Language")?)?,
                }
            }
            "RequestGameToken" => {
                n.only(&["GameCode", "AccountAlias"])?;
                Self::RequestGameToken {
                    game_code: n.value("GameCode")?.into(),
                    account_alias: n.value("AccountAlias")?.into(),
                }
            }
            "RequestToken" => {
                n.only(&["AppId"])?;
                Self::RequestToken {
                    app_id: n.value("AppId")?.into(),
                }
            }
            "LogoutMyClient" => {
                n.only(&[])?;
                Self::LogoutMyClient
            }
            "GetMyUserInfo" => {
                n.only(&[])?;
                Self::GetMyUserInfo
            }
            "ListMyAccounts" => {
                n.only(&["GameCode"])?;
                Self::ListMyAccounts {
                    game_code: n.value("GameCode")?.into(),
                }
            }
            _ => return Err(Error::Field),
        })
    }
    pub fn frame(&self, transaction: Option<u32>) -> Result<Frame, Error> {
        Ok(Frame {
            start: StartLine::Request {
                protocol: self.protocol().into(),
                command: self.command().into(),
            },
            headers: transaction
                .map(|id| {
                    Transaction {
                        id,
                        complete: None,
                        sequence: 0,
                    }
                    .to_header()
                    .map_err(|_| Error::Frame)
                })
                .transpose()?
                .into_iter()
                .collect(),
            body: self.body()?,
        })
    }
    pub fn from_frame(frame: &Frame) -> Result<Self, Error> {
        match &frame.start {
            StartLine::Request { protocol, command }
                if (protocol == "GameAccount") == (command == "ListMyAccounts")
                    && (protocol == "Auth" || protocol == "GameAccount") =>
            {
                Self::parse(command, &frame.body)
            }
            _ => Err(Error::Frame),
        }
    }
}
#[derive(Clone, Default, PartialEq, Eq)]
pub struct LoginInfo {
    pub user_id: Option<[u8; 16]>,
    pub user_center: Option<String>,
    pub roles: Vec<i32>,
    pub auth_type: Option<i32>,
    pub status: Option<i32>,
    pub pc_cafe_code: Option<String>,
}
#[derive(Clone, PartialEq, Eq)]
pub struct UserInfo {
    pub user_id: [u8; 16],
    pub user_center: String,
    pub user_name: String,
    pub login_name: String,
    /// ❓ Names and meanings of these optional values are not established.
    pub external_account: Vec<(String, String)>,
}
#[derive(Clone, PartialEq, Eq)]
pub enum Reply {
    LoginStart {
        salt: Vec<u8>,
        public: Vec<u8>,
    },
    KeyData {
        proof: [u8; 32],
    },
    LoginFinish(LoginInfo),
    GameToken {
        token: [u8; 16],
    },
    Token {
        authentication_token: String,
    },
    UserInfo(UserInfo),
    /// Reply 3106. Parser StsMsgCli32.dll 0.122 `sub_10004360` reads only
    /// `GameAccount/Alias` for each repeated `GameAccount` element.
    GameAccountList {
        aliases: Vec<String>,
    },
    /// ❓ Logout reply fields and its library ID are not established.
    LoggedOut,
}
impl Reply {
    pub fn body(&self) -> Result<Vec<u8>, Error> {
        let mut out = String::from("<Reply>\n");
        match self {
            Self::LoginStart { salt, public } => field(
                &mut out,
                "KeyData",
                &key_data::encode_challenge(salt, public).map_err(|_| Error::KeyData)?,
            )?,
            Self::KeyData { proof } => field(
                &mut out,
                "KeyData",
                &key_data::encode_server(proof).map_err(|_| Error::KeyData)?,
            )?,
            Self::LoginFinish(info) => {
                if let Some(id) = info.user_id {
                    field(&mut out, "UserId", &guid_text(id))?;
                }
                if let Some(center) = &info.user_center {
                    field(&mut out, "UserCenter", center)?;
                }
                if !info.roles.is_empty() {
                    out.push_str("<Roles>\n");
                    for role in &info.roles {
                        field(&mut out, "RoleId", &role.to_string())?;
                    }
                    out.push_str("</Roles>\n");
                }
                if let Some(v) = info.auth_type {
                    field(&mut out, "AuthType", &v.to_string())?;
                }
                if let Some(v) = info.status {
                    field(&mut out, "Status", &v.to_string())?;
                }
                if let Some(v) = &info.pc_cafe_code {
                    out.push_str("<PcCafe>\n");
                    field(&mut out, "PcCafeCode", v)?;
                    out.push_str("</PcCafe>\n");
                }
            }
            Self::GameToken { token } => field(&mut out, "Token", &guid_text(*token))?,
            Self::Token {
                authentication_token,
            } => field(&mut out, "AuthnToken", authentication_token)?,
            Self::UserInfo(info) => {
                field(&mut out, "UserId", &guid_text(info.user_id))?;
                field(&mut out, "UserCenter", &info.user_center)?;
                field(&mut out, "UserName", &info.user_name)?;
                field(&mut out, "LoginName", &info.login_name)?;
                if !info.external_account.is_empty() {
                    if info.external_account.len() > 4 {
                        return Err(Error::Field);
                    }
                    out.push_str("<ExternalAccount>\n");
                    for (name, value) in &info.external_account {
                        field(&mut out, name, value)?;
                    }
                    out.push_str("</ExternalAccount>\n");
                }
            }
            Self::GameAccountList { aliases } => {
                if aliases.len() > 32 {
                    return Err(Error::Field);
                }
                for alias in aliases {
                    out.push_str("<GameAccount>\n");
                    field(&mut out, "Alias", alias)?;
                    out.push_str("</GameAccount>\n");
                }
            }
            Self::LoggedOut => return Ok(b"<Reply/>".to_vec()),
        }
        out.push_str("</Reply>\n");
        Ok(out.into_bytes())
    }
    pub fn parse(id: u32, body: &[u8]) -> Result<Self, Error> {
        let n = parse_xml(body, "Reply")?;
        Ok(match id {
            3101 => {
                n.only(&["KeyData"])?;
                let (salt, public) =
                    key_data::decode_challenge(n.value("KeyData")?).map_err(|_| Error::KeyData)?;
                Self::LoginStart { salt, public }
            }
            3102 => {
                n.only(&["KeyData"])?;
                Self::KeyData {
                    proof: key_data::decode_server(n.value("KeyData")?)
                        .map_err(|_| Error::KeyData)?,
                }
            }
            3103 => {
                n.only(&[
                    "UserId",
                    "UserCenter",
                    "Roles",
                    "AuthType",
                    "Status",
                    "PcCafe",
                ])?;
                let mut roles = Vec::new();
                if let Some(r) = n.child("Roles") {
                    if !r.text.trim().is_empty() {
                        return Err(Error::Field);
                    }
                    for role in &r.children {
                        if role.name != "RoleId" || !role.children.is_empty() {
                            return Err(Error::Field);
                        }
                        roles.push(signed(&role.text)?);
                    }
                }
                let pc_cafe_code = if let Some(p) = n.child("PcCafe") {
                    p.only(&["PcCafeCode"])?;
                    p.optional("PcCafeCode")?.map(str::to_owned)
                } else {
                    None
                };
                Self::LoginFinish(LoginInfo {
                    user_id: n.optional("UserId")?.map(parse_guid).transpose()?,
                    user_center: n.optional("UserCenter")?.map(str::to_owned),
                    roles,
                    auth_type: n.optional("AuthType")?.map(signed).transpose()?,
                    status: n.optional("Status")?.map(signed).transpose()?,
                    pc_cafe_code,
                })
            }
            3106 => {
                if !n.text.trim().is_empty() {
                    return Err(Error::Field);
                }
                let mut aliases = Vec::new();
                for account in &n.children {
                    if account.name != "GameAccount" {
                        return Err(Error::Field);
                    }
                    account.only(&["Alias"])?;
                    aliases.push(account.value("Alias")?.into());
                }
                Self::GameAccountList { aliases }
            }
            3107 => {
                n.only(&["Token"])?;
                Self::GameToken {
                    token: parse_guid(n.value("Token")?)?,
                }
            }
            3110 => {
                n.only(&["AuthnToken"])?;
                Self::Token {
                    authentication_token: n.value("AuthnToken")?.into(),
                }
            }
            3114 => {
                n.only(&[
                    "UserId",
                    "UserCenter",
                    "UserName",
                    "LoginName",
                    "ExternalAccount",
                ])?;
                let mut external_account = Vec::new();
                if let Some(e) = n.child("ExternalAccount") {
                    if e.children.len() > 4 || !e.text.trim().is_empty() {
                        return Err(Error::Field);
                    }
                    for value in &e.children {
                        if !value.children.is_empty() {
                            return Err(Error::Field);
                        }
                        external_account.push((value.name.clone(), value.text.clone()));
                    }
                }
                Self::UserInfo(UserInfo {
                    user_id: parse_guid(n.value("UserId")?)?,
                    user_center: n.value("UserCenter")?.into(),
                    user_name: n.value("UserName")?.into(),
                    login_name: n.value("LoginName")?.into(),
                    external_account,
                })
            }
            _ => return Err(Error::Field),
        })
    }
    pub fn frame(&self, transaction: Option<u32>) -> Result<Frame, Error> {
        Ok(Frame {
            start: StartLine::Response {
                status: 200,
                reason: "OK".into(),
            },
            headers: transaction
                .map(|id| {
                    Transaction {
                        id,
                        complete: Some(true),
                        sequence: 0,
                    }
                    .to_header()
                    .map_err(|_| Error::Frame)
                })
                .transpose()?
                .into_iter()
                .collect(),
            body: self.body()?,
        })
    }
}
