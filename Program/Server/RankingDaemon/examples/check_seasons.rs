//! Parse a SeasonContentsScheduleData.xml file and print counts per mode.
fn main() {
    let path = std::env::args().nth(1).expect("path");
    let text = std::fs::read_to_string(path).expect("read");
    match rankingdaemon::config::parse_season_data(&text) {
        Ok(s) => {
            let mut modes = std::collections::BTreeMap::new();
            for x in &s {
                *modes.entry(x.service_mode).or_insert(0) += 1;
            }
            println!("ok {} {:?}", s.len(), modes);
        }
        Err(e) => println!("error {e:?}"),
    }
}
