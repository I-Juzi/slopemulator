//! Auth command ids (spec §2). The IDA table has 110 pairs. The spec names 18 of them.

macro_rules! auth_cmds {
    ($($name:ident = $id:expr),* $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        #[repr(u32)]
        pub enum AuthCmd { $($name = $id),* }

        impl AuthCmd {
            pub fn from_name(s: &str) -> Option<Self> {
                match s { $(stringify!($name) => Some(Self::$name),)* _ => None }
            }
            pub fn id(self) -> u32 { self as u32 }
            pub fn name(self) -> &'static str {
                match self { $(Self::$name => stringify!($name)),* }
            }
            pub fn from_id(id: u32) -> Option<Self> {
                match id { $($id => Some(Self::$name),)* _ => None }
            }
        }
    };
}

auth_cmds! {
    AccessTest = 1,
    GetMyUserInfo = 18,
    KeyData = 21,
    LoginStart = 22,
    LoginFinish = 23,
    RequestGameToken = 30,
    RequestToken = 32,
    VerifyMobiOtp = 35,
    VerifySecondPwd = 36,
    GetSecurityCardCodeIndex = 37,
    VerifySecondaryAuth = 38,
    CreateAccount = 46,
    TokenKeyData = 58,
    LoginTokenStart = 96,
    LogoutMyClient = 97,
    RequestCryptKey = 98,
    WaitForAllowedLogin = 99,
    Flush = 110,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_cmd_ids() {
        assert_eq!(AuthCmd::from_name("LogoutMyClient").unwrap().id(), 97);
        assert_eq!(AuthCmd::from_name("KeyData").unwrap().id(), 21);
        assert_eq!(AuthCmd::from_name("Nope"), None);
        assert_eq!(AuthCmd::from_id(23), Some(AuthCmd::LoginFinish));
        assert_eq!(AuthCmd::LoginStart.name(), "LoginStart");
    }
}
