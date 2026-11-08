//! Parse a SeasonContentsScheduleData.xml file, print counts per mode and
//! derive the result schedules for each mode (UTC offset in minutes, arg 2).
use rankingdaemon::config::{load_result_schedules, parse_season_data_at};

fn main() {
    let path = std::env::args().nth(1).expect("path");
    let offset: i64 = std::env::args()
        .nth(2)
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
        * 60;
    let text = std::fs::read_to_string(path).expect("read");
    match parse_season_data_at(&text, offset) {
        Ok(s) => {
            let mut modes = std::collections::BTreeMap::new();
            for x in &s {
                *modes.entry(x.service_mode).or_insert(0) += 1;
            }
            println!("ok {} {:?}", s.len(), modes);
            for &mode in modes.keys() {
                let mut all = s.clone();
                match load_result_schedules(&mut all, mode, offset) {
                    Ok(()) => {
                        let rows: usize = all.iter().map(|x| x.results.len()).sum();
                        let forced = all
                            .iter()
                            .filter(|x| x.service_mode == mode && x.forced_termination)
                            .count();
                        println!("mode {mode}: {rows} result rows, {forced} forced");
                    }
                    Err(e) => println!("mode {mode}: {e}"),
                }
            }
        }
        Err(e) => println!("error {e:?}"),
    }
}
