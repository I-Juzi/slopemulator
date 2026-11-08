//! Test client: `dbcall <addr> <Db.Logical> <Exec|ExecBatched> '<Request>…</Request>'`.
//! Prints the reply status, the `s` header of every frame and the body.

use sts_core::conn::{header, set_header, write_frame, FrameReader};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let [addr, protocol, command, body] = &a[..] else {
        eprintln!("usage: dbcall <addr> <Db.Logical> <Exec|ExecBatched> <xml>");
        std::process::exit(2)
    };
    let sock = tokio::net::TcpStream::connect(addr).await?;
    let (rd, mut wr) = sock.into_split();
    let mut f = sts_core::conn::request(protocol, command, None);
    f.body = body.as_bytes().to_vec();
    set_header(&mut f, "s", "1");
    write_frame(&mut wr, &f).await?;
    let mut reader = FrameReader::new(rd);
    while let Some(f) = reader.next().await? {
        let s = header(&f, "s").unwrap_or("").to_owned();
        println!("{:?} s={s} len={}", f.start, f.body.len());
        println!("{}", String::from_utf8_lossy(&f.body));
        if s.contains('R') {
            break;
        }
    }
    Ok(())
}
