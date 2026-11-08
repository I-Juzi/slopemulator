//! Local-only blocking service. All configuration is explicit.
fn main() {
    if let Err(error) = sts::listener::main_from_args(std::env::args().skip(1)) {
        // Fixed error categories only. Never print account records or arguments.
        eprintln!("sts-local: {error}");
        std::process::exit(1);
    }
}
