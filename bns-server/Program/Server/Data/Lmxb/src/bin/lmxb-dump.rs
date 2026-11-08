use std::{
    env, fs,
    io::{self, Write},
    process::ExitCode,
};

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let path = args.next().ok_or("usage: lmxb-dump <file>")?;
    if args.next().is_some() {
        return Err("usage: lmxb-dump <file>".into());
    }
    let bytes = fs::read(path)?;
    let document = lmxb::decode(&bytes)?;
    let xml = lmxb::to_xml_string(&document)?;
    io::stdout().lock().write_all(xml.as_bytes())?;
    Ok(())
}
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("lmxb-dump: {error}");
            ExitCode::FAILURE
        }
    }
}
