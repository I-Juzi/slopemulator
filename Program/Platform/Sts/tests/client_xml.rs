mod common;
use rand::{rngs::StdRng, SeedableRng};
use sts::{messages::client::*, Codec};

#[test]
fn all_request_bodies_and_frames_round_trip() {
    let mut rng = StdRng::seed_from_u64(301);
    let name = format!("{}<&é水", common::generated(&mut rng));
    for request in [
        Request::LoginStart {
            login_name: name.clone(),
        },
        Request::KeyData {
            public: vec![2, 0, 0, 0],
            proof: vec![7; 32],
        },
        Request::LoginFinish { language: 10 },
        Request::RequestGameToken {
            game_code: "TEST".into(),
            account_alias: name.clone(),
        },
        Request::RequestToken { app_id: name },
        Request::ListMyAccounts {
            game_code: "TEST".into(),
        },
        Request::LogoutMyClient,
        Request::GetMyUserInfo,
    ] {
        let body = request.body().unwrap();
        assert!(Request::parse(request.command(), &body).unwrap() == request);
        let frame = request.frame(Some(19)).unwrap();
        let encoded = Codec::default().encode(&frame).unwrap();
        let (frame, used) = Codec::default().decode(&encoded).unwrap().unwrap();
        assert_eq!(used, encoded.len());
        assert!(Request::from_frame(&frame).unwrap() == request);
    }
    assert_eq!(Request::LogoutMyClient.body().unwrap(), b"<Request/>");
    assert_eq!(Request::GetMyUserInfo.body().unwrap(), b"<Request/>");
}
#[test]
fn all_documented_reply_fields_round_trip() {
    let mut rng = StdRng::seed_from_u64(302);
    let name = common::generated(&mut rng);
    let cases = [
        (
            3101,
            Reply::LoginStart {
                salt: vec![4; 32],
                public: vec![2, 0, 0, 0],
            },
        ),
        (3102, Reply::KeyData { proof: [8; 32] }),
        (
            3103,
            Reply::LoginFinish(LoginInfo {
                user_id: Some([3; 16]),
                user_center: Some(name.clone()),
                roles: vec![1, -2, 3],
                auth_type: Some(0),
                status: Some(0),
                pc_cafe_code: Some(name.clone()),
            }),
        ),
        (3103, Reply::LoginFinish(LoginInfo::default())),
        (3107, Reply::GameToken { token: [5; 16] }),
        (
            3106,
            Reply::GameAccountList {
                aliases: vec![name.clone(), "second".into()],
            },
        ),
        (3106, Reply::GameAccountList { aliases: vec![] }),
        (
            3110,
            Reply::Token {
                authentication_token: name.clone(),
            },
        ),
        (
            3114,
            Reply::UserInfo(UserInfo {
                user_id: [3; 16],
                user_center: name.clone(),
                user_name: name.clone(),
                login_name: name.clone(),
                external_account: (0..4)
                    .map(|i| (format!("Value{i}"), name.clone()))
                    .collect(),
            }),
        ),
    ];
    for (id, reply) in cases {
        let body = reply.body().unwrap();
        assert!(Reply::parse(id, &body).unwrap() == reply);
        let frame = reply.frame(Some(20)).unwrap();
        assert_eq!(frame.headers[0].value, "20R");
    }
}
#[test]
fn guid_registry_text_has_windows_byte_order() {
    let bytes = [
        0x33, 0x22, 0x11, 0x00, 0x55, 0x44, 0x77, 0x66, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee,
        0xff,
    ];
    let text = "00112233-4455-6677-8899-aabbccddeeff";
    assert_eq!(guid_text(bytes), text);
    assert_eq!(parse_guid(text).unwrap(), bytes);
    assert_eq!(parse_guid(&text.to_uppercase()).unwrap(), bytes);
    for text in [
        "",
        "00112233_4455-6677-8899-aabbccddeeff",
        "00112233-4455-6677-8899-aabbccddeefg",
        "00112233-4455-6677-8899-aabbccddeeé",
    ] {
        assert!(parse_guid(text).is_err());
    }
}
#[test]
fn malformed_xml_and_missing_or_duplicate_fields_fail() {
    for xml in [
        "<Request/>",
        "<Request><LoginName>x</LoginName><LoginName>y</LoginName></Request>",
        "<Request><LoginName><Nested/></LoginName></Request>",
        "<Request><LoginName>x</LoginName></Wrong>",
        "<Request><LoginName>&unknown;</LoginName></Request>",
        "<Request><LoginName>&#0;</LoginName></Request>",
        "<!DOCTYPE Request [<!ENTITY x 'x'>]><Request><LoginName>&x;</LoginName></Request>",
        "<Request><LoginName>x</LoginName></Request><Request/>",
        "<Request a='b'><LoginName>x</LoginName></Request>",
        "<Request><LoginName>x</LoginName><Unexpected/></Request>",
    ] {
        assert!(
            Request::parse("LoginStart", xml.as_bytes()).is_err(),
            "{xml}"
        );
    }
    assert!(Request::parse("LoginFinish", b"<Request><Language>-1</Language></Request>").is_err());
    assert!(Request::parse("LogoutMyClient", b"<Request><Unexpected/></Request>").is_err());
    assert!(Reply::parse(3102, b"<Reply/>").is_err());
    assert!(Reply::parse(3107, b"<Reply><Token>bad</Token></Reply>").is_err());
    for missing in ["UserId", "UserCenter", "UserName", "LoginName"] {
        let mut xml = String::from("<Reply>");
        for (key, value) in [
            ("UserId", "00112233-4455-6677-8899-aabbccddeeff"),
            ("UserCenter", "0"),
            ("UserName", "x"),
            ("LoginName", "y"),
        ] {
            if key != missing {
                xml.push_str(&format!("<{key}>{value}</{key}>"));
            }
        }
        xml.push_str("</Reply>");
        assert!(Reply::parse(3114, xml.as_bytes()).is_err());
    }
}
#[test]
fn xml_entities_cdata_and_empty_values_parse() {
    let request = Request::parse(
        "LoginStart",
        b"<Request><LoginName>&lt;&amp;&#233;<![CDATA[<x>]]></LoginName></Request>",
    )
    .unwrap();
    assert!(
        request
            == Request::LoginStart {
                login_name: "<&é<x>".into()
            }
    );
    let request = Request::parse("LoginStart", b"<Request><LoginName/></Request>").unwrap();
    assert!(
        request
            == Request::LoginStart {
                login_name: String::new()
            }
    );
}

/// StsMsgCli32.dll 0.122 does not find `<KeyData>` directly after `<Reply>`
/// (`sub_10003B90` returns 6). Write one element per line, as the DLL's own
/// writer does. Verified by emulating the DLL on a captured reply, 2026-11-08.
#[test]
fn reply_puts_each_element_on_its_own_line() {
    let body = Reply::LoginStart {
        salt: vec![1; 32],
        public: vec![2; 128],
    }
    .body()
    .unwrap();
    let text = String::from_utf8(body).unwrap();
    assert!(text.starts_with("<Reply>\n<KeyData>"), "{text}");
    assert!(text.ends_with("</KeyData>\n</Reply>\n"), "{text}");
}
#[test]
fn list_my_accounts_uses_the_game_account_protocol() {
    let frame = Request::ListMyAccounts {
        game_code: "bns".into(),
    }
    .frame(Some(4))
    .unwrap();
    assert!(
        frame.start
            == sts::StartLine::Request {
                protocol: "GameAccount".into(),
                command: "ListMyAccounts".into(),
            }
    );
    let body = Reply::GameAccountList {
        aliases: vec!["katie".into()],
    }
    .body()
    .unwrap();
    assert_eq!(
        body,
        b"<Reply>\n<GameAccount>\n<Alias>katie</Alias>\n</GameAccount>\n</Reply>\n"
    );
    let mut wrong = frame.clone();
    wrong.start = sts::StartLine::Request {
        protocol: "Auth".into(),
        command: "ListMyAccounts".into(),
    };
    assert!(Request::from_frame(&wrong).is_err());
}

#[test]
fn get_balance_uses_the_virtual_currency_protocol_and_replies_empty() {
    use sts::messages::client::{Reply, Request};
    let body = b"<Request>\n<UserId>01234567-89AB-CDEF-0123-456789ABCDEF</UserId>\n<CurrencyGroupId>1</CurrencyGroupId>\n<CurrencyGroupUnit>1</CurrencyGroupUnit>\n</Request>\n";
    let mut frame = Request::GetBalance.frame(Some(7)).unwrap();
    assert!(matches!(&frame.start, sts::StartLine::Request { protocol, command } if protocol == "VirtualCurrency" && command == "GetBalance"));
    frame.body = body.to_vec();
    assert!(Request::from_frame(&frame).unwrap() == Request::GetBalance);
    let reply = Reply::EmptyBalance.body().unwrap();
    assert_eq!(reply, b"<Reply>\n<BalanceInfoListCount>0</BalanceInfoListCount>\n</Reply>\n");
    assert!(Reply::parse(3957, &reply).unwrap() == Reply::EmptyBalance);
    if let sts::StartLine::Request { protocol, .. } = &mut frame.start { *protocol = "Auth".into(); }
    assert!(Request::from_frame(&frame).is_err());
}
