//! Official client command IDs as `u16` constants, for `match` arms and ID lists.
//!
//! `build.rs` makes one constant for each `packet_ids::GameCmd` variant. The constant has the
//! name of the variant and takes its value from `GameCmd::<name>.id()`. A wrong name does not
//! compile.

use packet_ids::GameCmd;

include!(concat!(env!("OUT_DIR"), "/game_cmd_ids.rs"));
