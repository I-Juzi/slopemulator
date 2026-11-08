//! Operator contents rules. Spec 05 §5. No runtime reload.
use crate::{config::ContentsConfig, wire::invalid};
use bloodlib::{
    fragments::compose_fragments,
    packet::{compose, Value},
};
use std::io;

pub struct ContentsRules {
    packets: Vec<Vec<u8>>,
}
impl ContentsRules {
    pub fn new(config: &ContentsConfig) -> io::Result<Self> {
        let values = serde_json::to_value(config).map_err(|e| invalid(&e.to_string()))?;
        let names = [
            "dummy-data",
            "use-time-based-play-schedule",
            "enable-play-warrior",
            "enable-fish-system",
            "enable-account-contents",
            "account-contents-required-pc-level",
            "account-contents-required-pc-mastery-level",
            "account-contents-required-account-top-pc-level",
            "account-contents-required-account-top-pc-mastery-level",
            "account-exp-quota-per-day",
            "account-exp-quota-reset-time",
            "account-level-max",
            "enable-guild-join-renewal",
            "enable-job",
        ];
        let mut lines = vec![vec![Value::Bytes(b"<?xml version=\"1.0\" encoding=\"utf-8\"?><config major-version=\"74\" minor-version=\"0\" module=\"contents-rule-config\" release-module=\"LocalizationData\" release-side=\"client\">".to_vec())]];
        for name in names {
            let value = &values[name];
            let text = match value {
                serde_json::Value::Bool(b) => {
                    if *b {
                        "yes".to_owned()
                    } else {
                        "no".to_owned()
                    }
                }
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                _ => return Err(invalid("invalid contents rule")),
            };
            let line = format!("<option name='{name}' value='{text}'/>");
            if line.len() > 259 {
                return Err(invalid("contents option exceeds 259 UTF-8 bytes"));
            }
            lines.push(vec![Value::Bytes(line.into_bytes())]);
        }
        lines.push(vec![Value::Bytes(b"</config>".to_vec())]);
        let packets = compose_fragments(
            107,
            "hihhiCb;",
            &[
                Value::I16(74),
                Value::I16(0),
                Value::I32(0),
                Value::Repeat(lines),
            ],
        )
        .map_err(|e| invalid(&e.to_string()))?;
        Ok(Self { packets })
    }
    pub fn reply(&self, now: u64, last: &mut u64) -> Vec<Vec<u8>> {
        if *last == 0 || now <= 30000 || now.saturating_sub(*last) >= 30000 {
            *last = now;
            self.packets.clone()
        } else {
            vec![compose("hE", &[Value::I16(108), Value::I16(234)]).expect("fixed fail")]
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use bloodlib::packet::{decode_fields, decompose};
    #[test]
    fn fixture_lines_order_version_limit_and_grace() {
        let mut c = Config::from_xml(
            include_str!("../tests/fixtures/lobbygate.config.xml"),
            include_str!("../tests/fixtures/bns.common.config.xml"),
            include_str!("../tests/fixtures/ContentsRule.lobbygate.config.xml"),
        )
        .unwrap()
        .contents;
        let rules = ContentsRules::new(&c).unwrap();
        let mut last = 0;
        let packets = rules.reply(29999, &mut last);
        let v = decode_fields("hhiCb;", &packets[0][8..], Default::default()).unwrap();
        assert_eq!(&v[..3], &[Value::I16(74), Value::I16(0), Value::U32(0)]);
        let Value::Repeat(lines) = &v[3] else {
            panic!()
        };
        assert_eq!(lines.len(), 16);
        assert_eq!(
            lines[1],
            vec![Value::Bytes(
                b"<option name='dummy-data' value='123'/>".to_vec()
            )]
        );
        assert_eq!(rules.reply(30000, &mut last), packets);
        assert_eq!(
            decompose("hE", &rules.reply(30001, &mut last)[0], Default::default()).unwrap(),
            vec![Value::I16(108), Value::I16(234)]
        );
        assert_eq!(rules.reply(60000, &mut last), packets);
        c.enable_job = "x".repeat(260);
        assert!(ContentsRules::new(&c).is_err());
    }
}
