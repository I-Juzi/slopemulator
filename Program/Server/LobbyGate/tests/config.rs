use lobbygate::config::{options, Config};
use std::path::Path;
const G: &str = include_str!("fixtures/lobbygate.config.xml");
const B: &str = include_str!("fixtures/bns.common.config.xml");
const C: &str = include_str!("fixtures/ContentsRule.lobbygate.config.xml");
#[test]
fn original_format_defaults_and_operator_guid() {
    let c = Config::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")).unwrap();
    assert_eq!(c.gate.client_port, 10900);
    assert_eq!(c.gate.max_client, 200000);
    assert!(c.gate.max_idle_time_enable);
    assert!(!c.gate.disconnect_after_idle_time);
    assert_eq!(
        c.common.np_app_id.as_deref(),
        Some("00000000-0000-0000-0000-000000000001")
    );
    assert!(c.common.np_client_app_id.is_none());
    assert!(c.contents.enable_play_warrior);
    assert_eq!(c.contents.account_exp_quota_reset_time, 6);
    assert_eq!(c.contents.enable_job, ";");
    assert!(c.gate.worker_count() >= 2);
}
#[test]
fn validation_and_unknown_keys() {
    for bad in [
        G.replace("name='cert'", "name='other'"),
        G.replace("module='lobbygate'", "module='wrong'"),
        G.replace("value='10'", "value='-1'"),
        G.replace("future-unknown-key", "max-client")
            .replace("value='ignored'", "value='0'"),
    ] {
        assert!(Config::from_xml(&bad, B, C).is_err());
    }
    assert!(Config::from_xml(
        G,
        &B.replace("00000000-0000-0000-0000-000000000001", "bad"),
        C
    )
    .is_err());
    for (key, value) in [("ranking-service-mode", "12"), ("np-program-id", "-1")] {
        let xml = format!(
            "<config module='bns-common-config'><option name='{key}' value='{value}'/></config>"
        );
        assert!(Config::from_xml(G, &xml, C).is_err());
    }
    for (key, value) in [
        ("dummy-data", "1235"),
        ("account-exp-quota-reset-time", "24"),
        ("account-level-max", "1001"),
        ("enable-play-warrior", "true"),
    ] {
        let xml = format!("<config module='lobbygate-contents-rule-config'><option name='{key}' value='{value}'/></config>");
        assert!(Config::from_xml(G, B, &xml).is_err());
    }
}
#[test]
fn xml_structure_and_bounds() {
    for xml in ["<!DOCTYPE config><config module='x'/>","<config module='x'><bad/></config>","<config module='x'/><config module='x'/>","<config module='x'><option name='a' value='1'/><option name='a' value='2'/></config>","<config module='x'><option name='a' value='1'><option name='b' value='2'/></option></config>"] {
        assert!(options(xml,"x").is_err(),"{xml}");
    }
    assert!(options(&" ".repeat(1024 * 1024 + 1), "x").is_err());
    assert_eq!(
        options(
            "<config module='x'><option name='a' value='&amp;'/></config>",
            "x"
        )
        .unwrap()["a"],
        "&"
    );
}
