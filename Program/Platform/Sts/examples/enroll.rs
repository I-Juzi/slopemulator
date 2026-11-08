//! Make one STS account TSV record (SRP verifier enrollment).
//! Usage: enroll <login> <password> <alias> <portal_guid> <account_guid> <slot> [user_name]
//! Prints one tab-separated line for the sts-local --accounts file. No secret is stored.
use sts::password::VerifierRecord;
use base64::{engine::general_purpose::STANDARD, Engine};
use rand::rngs::OsRng;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    if a.len() < 6 {
        eprintln!("usage: enroll <login> <password> <alias> <portal_guid> <account_guid> <slot> [user_name]");
        std::process::exit(2);
    }
    let (login, password, alias, portal, account, slot) =
        (&a[0], &a[1], &a[2], &a[3], &a[4], &a[5]);
    let user = a.get(6).cloned().unwrap_or_else(|| login.clone());
    let rec = VerifierRecord::enroll(login, password.as_bytes(), &mut OsRng);
    println!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        login, user, portal, account, slot, alias,
        STANDARD.encode(&rec.salt), STANDARD.encode(&rec.verifier), rec.scheme
    );
}
