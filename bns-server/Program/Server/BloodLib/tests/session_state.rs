use bloodlib::session::SessionPacketCrypt;
use serde_json::Value;
fn bytes(v: &Value, field: &str) -> Vec<u8> {
    let s = v[field].as_str().unwrap();
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
#[test]
fn all_25_saved_session_cases_with_state() {
    let root: Value = serde_json::from_str(include_str!("vectors/emu_381.json")).unwrap();
    let mut count = 0;
    for c in root["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["kind"].as_str().unwrap().starts_with("session_"))
    {
        count += 1;
        let key0: [u8; 16] = bytes(c, "key0").try_into().unwrap();
        let mut crypt = if c["case"] == "no_key" {
            SessionPacketCrypt::default()
        } else {
            SessionPacketCrypt::new(key0)
        };
        if c["before"]["pending"] == 1 || c["before"]["send"] == 1 {
            crypt
                .prepare_key_change(bytes(c, "key1").try_into().unwrap())
                .unwrap();
        }
        if c["before"]["send"] == 1 {
            crypt.encrypt(&[0; 16], 18, false).unwrap();
        }
        if c["before"]["receive"] == 1 {
            let packet = crypt.encrypt(&[0; 16], 18, false).unwrap();
            crypt.decrypt(&packet, 16).unwrap();
        }
        let result = if c["kind"] == "session_encrypt" {
            crypt.encrypt(
                &bytes(c, "input"),
                c["capacity"].as_u64().unwrap() as usize,
                c["flag"] == true,
            )
        } else {
            crypt.decrypt(&bytes(c, "input"), 4000).map(|(out, _)| out)
        };
        if c["result"] == 0 {
            assert_eq!(result.unwrap(), bytes(c, "output"), "{}", c["case"]);
        } else {
            assert_eq!(
                u64::from(result.unwrap_err().code().unwrap()),
                c["result"].as_u64().unwrap(),
                "{}",
                c["case"]
            );
        }
        assert_eq!(
            crypt.state(),
            (
                c["after"]["send"].as_u64().unwrap() as usize,
                c["after"]["receive"].as_u64().unwrap() as usize,
                c["after"]["pending"] == 1
            ),
            "{}",
            c["case"]
        );
    }
    assert_eq!(count, 25);
}
#[test]
fn key_change_waits_for_receive_and_rejects_retired_slot() {
    let mut crypt = SessionPacketCrypt::new([1; 16]);
    crypt.prepare_key_change([2; 16]).unwrap();
    assert!(crypt.prepare_key_change([3; 16]).is_err());
    let old = crypt.encrypt(&[0; 16], 18, false).unwrap();
    assert!(crypt.prepare_key_change([3; 16]).is_err());
    crypt.decrypt(&old, 16).unwrap();
    let new = crypt.encrypt(&[0; 16], 18, false).unwrap();
    crypt.decrypt(&new, 16).unwrap();
    assert_eq!(crypt.decrypt(&old, 16).unwrap_err().code(), Some(7));
    crypt.prepare_key_change([3; 16]).unwrap();
}
