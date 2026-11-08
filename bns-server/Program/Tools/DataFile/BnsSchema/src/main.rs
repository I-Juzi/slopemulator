use bnsschema::{export, Error};
use std::{env, path::PathBuf};
fn run() -> Result<(), Error> {
    let mut args = env::args_os().skip(1);
    let usage = || Error::Format("usage: bnsschema EXE OUTDIR [--array 0xVA]".into());
    let exe = PathBuf::from(args.next().ok_or_else(usage)?);
    let out = PathBuf::from(args.next().ok_or_else(usage)?);
    let array = match args.next() {
        None => None,
        Some(flag) if flag == "--array" => {
            let value = args.next().ok_or_else(usage)?;
            let text = value.to_str().ok_or_else(usage)?;
            let digits = text
                .strip_prefix("0x")
                .or_else(|| text.strip_prefix("0X"))
                .ok_or_else(usage)?;
            Some(u64::from_str_radix(digits, 16).map_err(|_| usage())?)
        }
        Some(_) => return Err(usage()),
    };
    if args.next().is_some() {
        return Err(usage());
    }
    let found = export(&exe, array, &out)?;
    println!(
        "width={:?} layout={} array={:#x} tables={} unknown_types={:?}",
        found.width, found.layout, found.array_va, found.tables, found.unknown_types
    );
    Ok(())
}
fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            std::process::ExitCode::FAILURE
        }
    }
}
