# Card auth-1b-rust: AuthSrv crate (login, SRP, tokens)

<!-- Max 3,000 tokens. ASD-STE100. The worker has no other context. -->

## Goal
Make a new crate `authsrv` (lib + binary `AuthSrv`) at `Program/Platform/StServers/AuthSrv`. It serves the STS `Auth` commands LoginStart, KeyData, LoginFinish, RequestToken and RequestGameToken. `cargo test -p authsrv` passes.

## Context
- Spec: `Docs/Platform/StServers/15-authsrv.md`. Read only the line ranges in Inputs.
- Use `sts-core` (path `../StsCore`): `conn::{Peer, Incoming}`, `error::StsError` (`StsError::new(code, module, line).with_text(..)`), `xml::Element`, `ini::Ini`.
- AuthSrv error module = 6. The `t` header checks use module 5 (spec §3.3).
- Crates: tokio 1, sha2 0.10, num-bigint 0.4, base64 0.22, rand 0.8 (all are in the workspace). Add uuid 1 (feature `v4`) only if you need it.
- Big numbers use the little-endian format of spec §4.1. Do not use a big-endian SRP crate.
- The UserId comes from the STS `t` header in GUID form (§3.3). AuthSrv does not resolve names. The handlers get the UserId as an argument.
- DB access goes through the trait `AcctDb`. Game accounts go through the trait `GameAccounts`. Tests use in-memory fakes. A real DbSrv or GameAccount client is out of scope.
- Use a clock trait or a `now_ms: u64` argument for token expiry, so tests can move time.
- Never put a key or password in a file. Write `keyhere`.

## Inputs
| What | Path | Lines or section |
| --- | --- | --- |
| Dispatch, ids, txn | Docs/Platform/StServers/15-authsrv.md | §1, lines 7–43 |
| Session record | same | §2, lines 44–59 |
| LoginStart, `t` header | same | §3, lines 60–94 |
| SRP-6a | same | §4, lines 95–156 |
| KeyData | same | §5, lines 157–171 |
| LoginFinish | same | §6, lines 172–181 |
| Tokens | same | §7, lines 182–276 |
| DB calls | same | §8, lines 277–303 |
| STS core | Program/Platform/StServers/StsCore | `Peer`, `Incoming`, `StsError`, `Element`, `Ini` |

## Work
1. Add `Program/Platform/StServers/AuthSrv` to `members` in the root `Cargo.toml`.
2. `srp.rs`: `le`, `from_le`, `hnum`, k, HNG, `interleave`, `SrpServer`, test client `SrpClient`. Copy §4.3 step 1 exactly (it removes non-zero first bytes).
3. `db.rs`: `AcctDb`, `AcctLoginRow` (§8 columns), `MemAcctDb`, `guid_text` (§8.1 format, upper case).
4. `session.rs`: login sessions keyed by SessionId (§2).
5. `token.rs`: app tokens (§7.2, §7.3) and game tokens (§7.4, §7.5). Lifetimes from INI `[TokenTimeout] App` (default 1800 s) and `Game` (default 300 s), stored in ms (§7.1).
6. `handlers.rs`: one fn per command, with the error codes and reply elements of §3–§7.
7. `main.rs`: read the INI, listen, `Peer::spawn` per connection, dispatch on the command name. Parse `t` as §3.3.
8. Tests in `tests/`.

## Interfaces
```rust
// srp.rs
pub fn le(x: &BigUint) -> Vec<u8>;
pub fn from_le(b: &[u8]) -> BigUint;
pub fn hnum(d: &[u8; 32]) -> BigUint;
pub fn interleave(t: &[u8]) -> [u8; 64];
pub struct SrpServer { /* state, I, s, v, b, B, K */ }
impl SrpServer {
    pub fn new(b: [u8; 32]) -> Self;
    pub fn start(&mut self, login_name: &str, salt: &[u8], verifier: &[u8]) -> Option<Vec<u8>>;
    pub fn step(&mut self, client: &[u8]) -> Option<Vec<u8>>;
    pub fn session_key(&self) -> Option<[u8; 64]>;
}
pub struct SrpClient; // test helper; x formula is not known (spec §4.4), use x = hnum(H(salt, H(I ":" password)))
impl SrpClient {
    pub fn verifier(login_name: &str, password: &str, salt: &[u8; 8]) -> Vec<u8>;
    /// Returns (client key data `u32le lenA ‖ A ‖ u32le 32 ‖ M1`, expected M2, K).
    pub fn step(login_name: &str, password: &str, a: [u8; 32], server_key: &[u8]) -> Option<(Vec<u8>, [u8; 32], [u8; 64])>;
}

// db.rs
pub type Guid = [u8; 16];
#[derive(Clone, Debug)]
pub struct AcctLoginRow {
    pub user_id: Guid, pub user_center: u32, pub user_name: String,
    pub salt: Vec<u8>, pub verifier: Vec<u8>, pub password_until: Option<i64>,
    pub validate_type: u32, pub validated: bool, pub roles: Vec<u32>,
}
pub trait AcctDb: Send + Sync {
    fn acct_login_info_by_id(&self, user_id: Guid)
        -> Pin<Box<dyn Future<Output = Result<Option<AcctLoginRow>, StsError>> + Send + '_>>;
}
#[derive(Default)] pub struct MemAcctDb { /* HashMap<Guid, AcctLoginRow> */ }
impl MemAcctDb { pub fn insert(&mut self, row: AcctLoginRow); }
pub fn guid_text(id: &Guid) -> String;

// token.rs
#[derive(Clone, Debug)] pub struct GameAccount { pub alias: String, pub game_account_id: String }
pub trait GameAccounts: Send + Sync {
    fn list_accounts(&self, user_id: Guid, game_code: &str)
        -> Pin<Box<dyn Future<Output = Result<Vec<GameAccount>, StsError>> + Send + '_>>;
}
#[derive(Default)] pub struct MemGameAccounts { /* HashMap<(Guid, String), Vec<GameAccount>> */ }
impl MemGameAccounts { pub fn insert(&mut self, user: Guid, game_code: &str, acct: GameAccount); }
pub struct TokenConfig { pub app_ms: u32, pub game_ms: u32 } // from INI, defaults 1_800_000 / 300_000
pub struct AppToken { pub user_id: Guid, pub token: Guid, pub app_id: Guid, pub issuer_address: String, pub expire_ms: u64 }
pub struct GameToken { pub user_id: Guid, pub token: Guid, pub game_code: String, pub game_account_id: String, pub address: String, pub expire_ms: u64 }
pub struct TokenStore { /* app: HashMap<(Guid user, Guid app), AppToken>, game: Vec<GameToken> */ }
impl TokenStore {
    pub fn new(cfg: TokenConfig) -> Self;
    /// Replaces an old token of the same (user, app). Returns the `AuthnToken` text `<user GUID>:<token GUID>`.
    pub fn create_app(&mut self, user_id: Guid, app_id: Guid, address: &str, now_ms: u64) -> String;
    pub fn create_game(&mut self, user_id: Guid, game_code: &str, game_account_id: &str, address: &str, now_ms: u64) -> Guid;
    pub fn expire(&mut self, now_ms: u64); // removes tokens with expire_ms <= now_ms
    pub fn app_count(&self, user_id: Guid) -> usize;
    pub fn game_count(&self, user_id: Guid) -> usize;
}

// handlers.rs
pub struct User { pub user_id: Guid, pub user_status: u32 } // §7.2 step 1: status must be 1
pub struct AuthService { /* Arc<dyn AcctDb>, Arc<dyn GameAccounts>, sessions, Mutex<TokenStore> */ }
impl AuthService {
    pub fn new(db: Arc<dyn AcctDb>, accts: Arc<dyn GameAccounts>, cfg: TokenConfig) -> Self;
    pub async fn login_start(&self, user_id: Guid, req: &Element) -> Result<Element, StsError>;
    pub async fn key_data(&self, req: &Element) -> Result<Element, StsError>;
    pub async fn login_finish(&self, req: &Element) -> Result<Element, StsError>;
    pub async fn request_token(&self, user: &User, caller_app_id: Guid, address: &str, req: &Element, now_ms: u64) -> Result<Element, StsError>;
    pub async fn request_game_token(&self, user: &User, address: &str, req: &Element, now_ms: u64) -> Result<Element, StsError>;
}
```

## Files
- `Cargo.toml`
- `Cargo.lock`
- `Program/Platform/StServers/AuthSrv/Cargo.toml`
- `Program/Platform/StServers/AuthSrv/src/lib.rs`
- `Program/Platform/StServers/AuthSrv/src/main.rs`
- `Program/Platform/StServers/AuthSrv/src/srp.rs`
- `Program/Platform/StServers/AuthSrv/src/db.rs`
- `Program/Platform/StServers/AuthSrv/src/session.rs`
- `Program/Platform/StServers/AuthSrv/src/token.rs`
- `Program/Platform/StServers/AuthSrv/src/handlers.rs`
- `Program/Platform/StServers/AuthSrv/tests/srp.rs`
- `Program/Platform/StServers/AuthSrv/tests/login.rs`
- `Program/Platform/StServers/AuthSrv/tests/token.rs`

## Acceptance
Run `cargo test -p authsrv`.

| Test name | Input | Expected result |
| --- | --- | --- |
| `le_len_multiple_of_4` | `le(2)` | `02 00 00 00` |
| `le_n_is_128_bytes` | `le(N)` | 128 bytes; last byte = `0xEE` (N starts `EEAF0AB9`) |
| `interleave_drops_nonzero_lead` | T = `01 02 00` ‖ 61 bytes | first 2 bytes removed; K = interleave of the other 62 bytes |
| `srp_round_trip` | fixed salt, password, a, b | server `step` is `Some`; client M2 = server M2; both K are equal |
| `srp_bad_m1` | M1 with one bit changed | `step` is `None` |
| `srp_start_bad_sizes` | salt 7 bytes, or verifier 127 bytes, or empty name | `start` is `None` |
| `guid_text_format` | bytes `00..0F` | `03020100-0504-0706-0809-0A0B0C0D0E0F` |
| `login_start_no_name` | no `<LoginName>` | error 10 |
| `login_start_no_salt` | row with empty salt | error 3317 |
| `key_data_bad_session` | unknown `<Session>` | error 3101 |
| `key_data_bad_base64` | `<KeyData>a*b</KeyData>` | error 10; session deleted |
| `key_data_wrong_password` | M1 of another password | error 11; session deleted |
| `key_data_not_validated` | `validated = false`, good password | error 3320; session deleted |
| `full_login` | LoginStart → KeyData → LoginFinish | KeyData reply has `<Complete/>` and `<SessionKey>` (64 bytes Base64); LoginFinish reply has `LoginName`, `UserId`, `UserName`, `UserCenter`, `Roles`; session deleted |
| `token_bad_status` | user_status 2 | error 49 (both token commands) |
| `token_missing_app_id` | no `<AppId>`, caller AppId zero | error 10 |
| `token_format` | RequestToken OK | `<AuthnToken>` = `<guid_text(user)>:<guid_text(token)>`, 73 chars |
| `token_replace_same_app` | RequestToken twice, same AppId | second text differs; `app_count` = 1 |
| `token_expire` | app token at t=0; `expire(1_799_999)` then `expire(1_800_000)` | count 1, then 0 |
| `game_token_missing_param` | no `<AccountAlias>` | error 10 |
| `game_token_no_alias` | alias not in `MemGameAccounts` | error 4 |
| `game_token_ok` | alias exists | `<Token>` is GUID text (36 chars); `game_count` = 1; expires at 300 000 ms |

## Out of scope
- ExAuth, OTP, PC-cafe, billing, SecondaryAuthSrv, ConsumeGameToken, forward to another datacenter.
- Name resolve of `t=login:<name>` (router side, spec §3.3).
- Real DbSrv or GameAccount clients. Only the traits and the fakes.
- Do not change files outside the Files list.
