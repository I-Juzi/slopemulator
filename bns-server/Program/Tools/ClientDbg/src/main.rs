//! `clientdbg` host program. See README.md.

use clientdbg::cli::{build_request, parse_args, parse_u32};
use clientdbg::host::{serve, tail, ServeConfig};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

const DEFAULT_LISTEN: &str = "100.77.181.120:47100";
const DEFAULT_CTL: &str = "127.0.0.1:47101";
const DEFAULT_LOG_DIR: &str = "logs/clientdbg";
const DEFAULT_TOKEN_FILE: &str = "secrets/clientdbg.token";

const USAGE: &str = "usage:
  clientdbg serve [--listen ADDR] [--ctl ADDR] [--token-file FILE] [--log-dir DIR]
  clientdbg tail [--n 50] [--filter key=value] [--log-dir DIR]
  clientdbg [--ctl ADDR] [--ida] <ping|status|modules|threads|rebase>
  clientdbg [--ctl ADDR] [--ida] read ADDR LEN
  clientdbg [--ctl ADDR] [--ida] write ADDR HEX
  clientdbg [--ctl ADDR] regs TID
  clientdbg [--ctl ADDR] [--ida] hbp ADDR exec|write|rw LEN SLOT [--capture SPEC] [--limit N]
  clientdbg [--ctl ADDR] hbp-clear [SLOT|all]";

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = match run(args).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            2
        }
    };
    std::process::exit(code);
}

async fn run(args: Vec<String>) -> Result<i32, String> {
    let o = parse_args(&args)?;
    let Some(cmd) = o.pos.first().cloned() else {
        println!("{USAGE}");
        return Ok(2);
    };
    let flag = |k: &str, d: &str| o.flags.get(k).cloned().unwrap_or_else(|| d.to_string());
    let log_dir = PathBuf::from(flag("log-dir", DEFAULT_LOG_DIR));
    match cmd.as_str() {
        "serve" => {
            let token_file = flag("token-file", DEFAULT_TOKEN_FILE);
            let token = std::fs::read_to_string(&token_file)
                .map_err(|e| format!("cannot read token file {token_file}: {e}"))?
                .trim()
                .to_string();
            let cfg = ServeConfig {
                listen: flag("listen", DEFAULT_LISTEN),
                ctl: flag("ctl", DEFAULT_CTL),
                token,
                log_dir,
                ready: None,
            };
            serve(cfg).await.map_err(|e| e.to_string())?;
            Ok(0)
        }
        "tail" => {
            let n = o
                .flags
                .get("n")
                .map(|s| parse_u32(s))
                .transpose()?
                .unwrap_or(50) as usize;
            let filter = o.flags.get("filter").map(|f| {
                f.split_once('=')
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .ok_or("filter must be key=value")
            });
            let filter = filter.transpose()?;
            let lines = tail(
                &log_dir,
                n,
                filter.as_ref().map(|(k, v)| (k.as_str(), v.as_str())),
            )
            .map_err(|e| format!("cannot read {}: {e}", log_dir.display()))?;
            for l in lines {
                println!("{l}");
            }
            Ok(0)
        }
        _ => {
            let req = build_request(&o)?;
            let ctl = flag("ctl", DEFAULT_CTL);
            let s = TcpStream::connect(&ctl)
                .await
                .map_err(|e| format!("cannot connect to {ctl}: {e}"))?;
            let (rd, mut wr) = s.into_split();
            wr.write_all(format!("{req}\n").as_bytes())
                .await
                .map_err(|e| e.to_string())?;
            let mut line = String::new();
            BufReader::new(rd)
                .read_line(&mut line)
                .await
                .map_err(|e| e.to_string())?;
            let line = line.trim();
            if line.is_empty() {
                return Err("host closed the connection".into());
            }
            println!("{line}");
            let failed = serde_json::from_str::<serde_json::Value>(line)
                .map(|v| v.get("err").is_some())
                .unwrap_or(true);
            Ok(if failed { 1 } else { 0 })
        }
    }
}
