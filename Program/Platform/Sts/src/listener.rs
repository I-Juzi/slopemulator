//! Blocking local TCP service. The binary is gated by feature `listener`.
//! This module stays compiled so the required default workspace checks test it.
//! No addresses, account names, passwords or verifier records are built in.
use crate::{
    messages::client::parse_guid,
    password::{
        Account, Binding, Grant, Identity, MemoryAccounts, Request, Session, State, VerifierRecord,
    },
    srp6,
    tokens::TokenAuthority,
    wire::{self},
    Codec, Frame, StartLine, Transaction,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use rand::rngs::OsRng;
use std::{
    collections::HashMap,
    fs,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Configuration,
    AccountFile,
    Io,
    Protocol,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "local STS error: {self:?}")
    }
}
impl std::error::Error for Error {}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectMode {
    /// The caller already handles Connect. The first wire command is LoginStart.
    LoginStart,
    /// ❓ Explicit local test policy. Accept Sts/Connect without body semantics.
    /// Reply with an empty Reply and status 200. This is not a recovered schema.
    UnverifiedConnect,
}
pub struct Config {
    pub bind: SocketAddr,
    pub binding: Binding,
    pub user_center: String,
    pub accounts: PathBuf,
    pub token_lifetime: Duration,
    pub token_capacity: usize,
    pub session_lifetime: Duration,
    pub min_accept_interval: Duration,
    pub connect_mode: ConnectMode,
}
fn positive(value: String) -> Result<u64, Error> {
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return Err(Error::Configuration);
    }
    let number = value.parse().map_err(|_| Error::Configuration)?;
    if number == 0 {
        return Err(Error::Configuration);
    }
    Ok(number)
}
impl Config {
    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, Error> {
        let mut options = HashMap::new();
        let mut args = args.into_iter();
        while let Some(name) = args.next() {
            let value = args.next().ok_or(Error::Configuration)?;
            if options.insert(name, value).is_some() {
                return Err(Error::Configuration);
            }
        }
        let mut get = |name: &str| options.remove(name).ok_or(Error::Configuration);
        let bind: SocketAddr = get("--bind")?.parse().map_err(|_| Error::Configuration)?;
        if !bind.ip().is_loopback() {
            return Err(Error::Configuration);
        }
        let program_id =
            u32::try_from(positive(get("--program")?)?).map_err(|_| Error::Configuration)?;
        let binding = Binding {
            program_id,
            game_code: get("--game-code")?,
        };
        if !binding.valid() {
            return Err(Error::Configuration);
        }
        let user_center = get("--user-center")?;
        if user_center.is_empty() || user_center.len() > 256 {
            return Err(Error::Configuration);
        }
        let accounts = PathBuf::from(get("--accounts")?);
        if accounts.as_os_str().is_empty() {
            return Err(Error::Configuration);
        }
        let token_lifetime = Duration::from_secs(positive(get("--token-seconds")?)?);
        let token_capacity = usize::try_from(positive(get("--token-capacity")?)?)
            .map_err(|_| Error::Configuration)?;
        let session_lifetime = Duration::from_secs(positive(get("--session-seconds")?)?);
        let min_accept_interval = Duration::from_millis(positive(get("--min-accept-ms")?)?);
        let connect_mode = match get("--connect-mode")?.as_str() {
            "login-start" => ConnectMode::LoginStart,
            "unverified-connect" => ConnectMode::UnverifiedConnect,
            _ => return Err(Error::Configuration),
        };
        if !options.is_empty() {
            return Err(Error::Configuration);
        }
        let now = Instant::now();
        for duration in [token_lifetime, session_lifetime, min_accept_interval] {
            now.checked_add(duration).ok_or(Error::Configuration)?;
        }
        Ok(Self {
            bind,
            binding,
            user_center,
            accounts,
            token_lifetime,
            token_capacity,
            session_lifetime,
            min_accept_interval,
            connect_mode,
        })
    }
}
/// UTF-8 TSV, nine fields per record:
/// login, user name, portal GUID, account GUID, slot, alias, salt Base64,
/// verifier Base64, scheme. No password column. One grant per account.
/// Use password::VerifierRecord::enroll to make records outside the service.
pub fn parse_accounts(text: &str, binding: &Binding) -> Result<MemoryAccounts, Error> {
    if text.len() > 1_048_576 {
        return Err(Error::AccountFile);
    }
    let mut store = MemoryAccounts::default();
    let mut count = 0usize;
    for line in text.lines().filter(|l| !l.is_empty()) {
        count += 1;
        if count > 4096 {
            return Err(Error::AccountFile);
        }
        let f: Vec<_> = line.split('\t').collect();
        if f.len() != 9 || f[8] != srp6::SCHEME {
            return Err(Error::AccountFile);
        }
        let salt = STANDARD.decode(f[6]).map_err(|_| Error::AccountFile)?;
        let verifier = STANDARD.decode(f[7]).map_err(|_| Error::AccountFile)?;
        // Validate the imported record, including canonical word encoding.
        srp6::ServerExchange::start(f[0], &salt, &verifier, &[1; 32])
            .map_err(|_| Error::AccountFile)?;
        let account = Account::from_verifier(
            Identity {
                portal_id: parse_guid(f[2]).map_err(|_| Error::AccountFile)?,
                account_id: parse_guid(f[3]).map_err(|_| Error::AccountFile)?,
                login_name: f[0].into(),
                user_name: f[1].into(),
                slot_id: positive(f[4].into()).map_err(|_| Error::AccountFile)?,
            },
            VerifierRecord {
                scheme: f[8].into(),
                salt,
                verifier,
            },
            vec![Grant {
                binding: binding.clone(),
                game_alias: f[5].into(),
            }],
        )
        .map_err(|_| Error::AccountFile)?;
        store.insert(account).map_err(|_| Error::AccountFile)?;
    }
    if count == 0 {
        return Err(Error::AccountFile);
    }
    Ok(store)
}
pub fn main_from_args(args: impl IntoIterator<Item = String>) -> Result<(), Error> {
    let config = Config::parse(args)?;
    // Bound the read before allocating. Do not trust metadata from a mutable file.
    let mut contents = String::new();
    fs::File::open(&config.accounts)
        .map_err(|_| Error::AccountFile)?
        .take(1_048_577)
        .read_to_string(&mut contents)
        .map_err(|_| Error::AccountFile)?;
    let store = parse_accounts(&contents, &config.binding)?;
    let authority = Arc::new(
        TokenAuthority::new(config.token_lifetime, config.token_capacity)
            .map_err(|_| Error::Configuration)?,
    );
    let listener = TcpListener::bind(config.bind).map_err(|_| Error::Io)?;
    let mut next_accept = Instant::now();
    for stream in listener.incoming() {
        let stream = stream.map_err(|_| Error::Io)?;
        let now = Instant::now();
        if now < next_accept {
            continue;
        }
        next_accept = now
            .checked_add(config.min_accept_interval)
            .ok_or(Error::Configuration)?;
        // One blocking connection at a time. Absolute lifetime bounds slow input.
        let _ = serve_connection(stream, &config, &store, authority.clone());
    }
    Ok(())
}
fn set_deadline(stream: &TcpStream, deadline: Instant) -> Result<(), Error> {
    let left = deadline
        .checked_duration_since(Instant::now())
        .filter(|d| !d.is_zero())
        .ok_or(Error::Protocol)?;
    stream.set_read_timeout(Some(left)).map_err(|_| Error::Io)?;
    stream.set_write_timeout(Some(left)).map_err(|_| Error::Io)
}
/// One connection for an embedding caller or an explicit local listener.
/// RequestToken is unavailable without a separate AppTokenIssuer integration.
pub fn serve_connection(
    mut stream: TcpStream,
    config: &Config,
    store: &MemoryAccounts,
    authority: Arc<TokenAuthority>,
) -> Result<(), Error> {
    let start = Instant::now();
    let deadline = start
        .checked_add(config.session_lifetime)
        .ok_or(Error::Configuration)?;
    let mut rng = OsRng;
    let mut login = Session::new(config.binding.clone(), authority, &mut rng)
        .map_err(|_| Error::Configuration)?;
    login
        .handle(
            Request::Connect(config.binding.clone()),
            store,
            &mut rng,
            start,
        )
        .map_err(|_| Error::Protocol)?;
    let mut session =
        wire::Session::new(login, config.user_center.clone()).map_err(|_| Error::Configuration)?;
    let mut connected = config.connect_mode == ConnectMode::LoginStart;
    let mut connect = Vec::new();
    let codec = Codec {
        max_header_bytes: 32_767,
        max_body_bytes: 65_536,
    };
    let mut buffer = [0; 8192];
    loop {
        set_deadline(&stream, deadline)?;
        let count = stream.read(&mut buffer).map_err(|_| Error::Io)?;
        if count == 0 {
            return Ok(());
        }
        let mut input = &buffer[..count];
        if !connected {
            connect.extend_from_slice(input);
            if connect.len() > 32_767 + 65_536 + buffer.len() {
                return Err(Error::Protocol);
            }
            let Some((frame, used)) = codec.decode(&connect).map_err(|_| Error::Protocol)? else {
                continue;
            };
            if frame.start
                != (StartLine::Request {
                    protocol: "Sts".into(),
                    command: "Connect".into(),
                })
            {
                return Err(Error::Protocol);
            }
            let transaction = frame
                .headers
                .iter()
                .find(|h| h.name == "s")
                .map(|h| Transaction::parse(&h.value).map_err(|_| Error::Protocol))
                .transpose()?;
            if transaction.is_some_and(|t| t.complete.is_some() || t.sequence != 0) {
                return Err(Error::Protocol);
            }
            let reply = Frame {
                start: StartLine::Response {
                    status: 200,
                    reason: "OK".into(),
                },
                headers: transaction
                    .map(|t| {
                        Transaction {
                            id: t.id,
                            complete: Some(true),
                            sequence: 0,
                        }
                        .to_header()
                        .map_err(|_| Error::Protocol)
                    })
                    .transpose()?
                    .into_iter()
                    .collect(),
                body: b"<Reply/>".to_vec(),
            };
            set_deadline(&stream, deadline)?;
            stream
                .write_all(&codec.encode(&reply).map_err(|_| Error::Protocol)?)
                .map_err(|_| Error::Io)?;
            connected = true;
            input = &connect[used..];
        }
        let replies = session
            .receive(input, store, &mut rng, Instant::now())
            .map_err(|_| Error::Protocol)?;
        connect.clear();
        for reply in replies {
            set_deadline(&stream, deadline)?;
            stream.write_all(&reply).map_err(|_| Error::Io)?;
        }
        if session.state() == State::Closed {
            return Ok(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        messages::client::guid_text,
        test_client::{generated, Client},
    };
    use rand::{rngs::StdRng, RngCore, SeedableRng};
    fn args() -> Vec<String> {
        let bind = SocketAddr::from((std::net::Ipv4Addr::LOCALHOST, 0)).to_string();
        [
            "--bind",
            &bind,
            "--program",
            "43",
            "--game-code",
            "TEST",
            "--user-center",
            "0",
            "--accounts",
            "generated.tsv",
            "--token-seconds",
            "60",
            "--token-capacity",
            "16",
            "--session-seconds",
            "10",
            "--min-accept-ms",
            "25",
            "--connect-mode",
            "login-start",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    }
    #[test]
    fn all_cli_values_are_required_and_local() {
        assert!(Config::parse(args()).is_ok());
        assert!(Config::parse(Vec::new()).is_err());
        for i in (0..args().len()).step_by(2) {
            let mut a = args();
            a.drain(i..i + 2);
            assert!(Config::parse(a).is_err());
        }
        let mut a = args();
        a[1] = SocketAddr::from((std::net::Ipv4Addr::UNSPECIFIED, 0)).to_string();
        assert!(Config::parse(a).is_err());
        let mut a = args();
        a.extend(["--program".into(), "43".into()]);
        assert!(Config::parse(a).is_err());
        let mut a = args();
        a.extend(["--unknown".into(), "1".into()]);
        assert!(Config::parse(a).is_err());
    }
    #[test]
    fn generated_verifier_file_import_and_duplicate_rejection() {
        let mut rng = StdRng::seed_from_u64(513);
        let mut login = [0; 16];
        let mut password = [0; 32];
        rng.fill_bytes(&mut login);
        rng.fill_bytes(&mut password);
        let login = guid_text(login);
        let record = VerifierRecord::enroll(&login, &password, &mut rng);
        let mut portal = [0; 16];
        let mut account = [0; 16];
        rng.fill_bytes(&mut portal);
        rng.fill_bytes(&mut account);
        let line = format!(
            "{}\t{}\t{}\t{}\t1\t{}\t{}\t{}\t{}",
            login,
            login,
            guid_text(portal),
            guid_text(account),
            login,
            STANDARD.encode(record.salt),
            STANDARD.encode(record.verifier),
            srp6::SCHEME
        );
        let binding = Config::parse(args()).unwrap().binding;
        assert!(parse_accounts(&line, &binding).is_ok());
        assert!(parse_accounts(&format!("{line}\n{line}"), &binding).is_err());
        assert!(parse_accounts("", &binding).is_err());
        assert!(parse_accounts("malformed", &binding).is_err());
    }
    fn read_frame(stream: &mut TcpStream, cipher: &mut Option<crate::rc4::Rc4>) -> Frame {
        let codec = Codec::default();
        let mut bytes = Vec::new();
        loop {
            let mut byte = [0];
            stream.read_exact(&mut byte).unwrap();
            if let Some(cipher) = cipher {
                cipher.apply(&mut byte);
            }
            bytes.push(byte[0]);
            if let Some((frame, used)) = codec.decode(&bytes).unwrap() {
                assert_eq!(used, bytes.len());
                return frame;
            }
        }
    }
    #[test]
    fn blocking_tcp_client_login_and_token_round_trip() {
        use crate::{
            messages::client::{Reply as V, Request as W},
            rc4::Rc4,
        };
        let mut rng = StdRng::seed_from_u64(514);
        let login = generated(&mut rng);
        let password = generated(&mut rng);
        let alias = generated(&mut rng);
        let mut config = Config::parse(args()).unwrap();
        config.connect_mode = ConnectMode::UnverifiedConnect;
        let mut portal = [0; 16];
        let mut account_id = [0; 16];
        rng.fill_bytes(&mut portal);
        rng.fill_bytes(&mut account_id);
        let mut store = MemoryAccounts::default();
        store
            .insert(
                Account::from_verifier(
                    Identity {
                        portal_id: portal,
                        account_id,
                        login_name: login.clone(),
                        user_name: generated(&mut rng),
                        slot_id: 1,
                    },
                    VerifierRecord::enroll(&login, password.as_bytes(), &mut rng),
                    vec![Grant {
                        binding: config.binding.clone(),
                        game_alias: alias.clone(),
                    }],
                )
                .unwrap(),
            )
            .unwrap();
        let authority =
            Arc::new(TokenAuthority::new(config.token_lifetime, config.token_capacity).unwrap());
        let check = authority.clone();
        let binding = config.binding.clone();
        let listener =
            TcpListener::bind(SocketAddr::from((std::net::Ipv4Addr::LOCALHOST, 0))).unwrap();
        let address = listener.local_addr().unwrap();
        let worker = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            serve_connection(stream, &config, &store, authority)
        });
        let mut stream = TcpStream::connect(address).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        let codec = Codec::default();
        // The test exercises the explicit, unverified local Connect policy.
        let connect = Frame {
            start: StartLine::Request {
                protocol: "Sts".into(),
                command: "Connect".into(),
            },
            headers: vec![],
            body: b"<Request/>".to_vec(),
        };
        let mut first = codec.encode(&connect).unwrap();
        first.extend(
            codec
                .encode(
                    &W::LoginStart {
                        login_name: login.clone(),
                    }
                    .frame(Some(1))
                    .unwrap(),
                )
                .unwrap(),
        );
        stream.write_all(&first).unwrap();
        let mut receive = None;
        assert_eq!(read_frame(&mut stream, &mut receive).body, b"<Reply/>");
        let frame = read_frame(&mut stream, &mut receive);
        let V::LoginStart { salt, public } = V::parse(3101, &frame.body).unwrap() else {
            panic!("challenge")
        };
        let client = Client::new(&login, password.as_bytes(), &salt, &public, &mut rng);
        stream
            .write_all(
                &codec
                    .encode(
                        &W::KeyData {
                            public: client.public,
                            proof: client.proof,
                        }
                        .frame(Some(2))
                        .unwrap(),
                    )
                    .unwrap(),
            )
            .unwrap();
        let frame = read_frame(&mut stream, &mut receive);
        let V::KeyData { proof } = V::parse(3102, &frame.body).unwrap() else {
            panic!("proof")
        };
        assert_eq!(proof.as_slice(), client.server_proof);
        let mut send = Rc4::new(&client.key).unwrap();
        receive = Rc4::new(&client.key);
        for (request, id, reply_id) in [
            (W::LoginFinish { language: 0 }, 3, 3103),
            (
                W::RequestGameToken {
                    game_code: binding.game_code.clone(),
                    account_alias: alias,
                },
                4,
                3107,
            ),
        ] {
            let mut bytes = codec.encode(&request.frame(Some(id)).unwrap()).unwrap();
            send.apply(&mut bytes);
            stream.write_all(&bytes).unwrap();
            let frame = read_frame(&mut stream, &mut receive);
            let reply = V::parse(reply_id, &frame.body).unwrap();
            if let V::GameToken { token } = reply {
                assert!(check
                    .consume(token, portal, &binding, Instant::now())
                    .is_ok());
                assert!(check
                    .consume(token, portal, &binding, Instant::now())
                    .is_err());
            }
        }
        let mut bytes = codec
            .encode(&W::LogoutMyClient.frame(Some(5)).unwrap())
            .unwrap();
        send.apply(&mut bytes);
        stream.write_all(&bytes).unwrap();
        assert_eq!(read_frame(&mut stream, &mut receive).body, b"<Reply/>");
        drop(stream);
        worker.join().unwrap().unwrap();
    }
}
