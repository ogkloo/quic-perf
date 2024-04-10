use std::env;
use std::io::prelude::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::str::FromStr;
use std::time::{Duration, Instant};

/// Client for all Rust versions.
fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let server_addr: &str = &args[1];
    let server_port: u16 = args[2].parse::<u16>().unwrap();

    let sk_addr = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::from_str(server_addr).unwrap()),
        server_port,
    );

    let mut kb_sent: u64 = 0;
    let mut stream = TcpStream::connect(sk_addr)?;
    stream.write("OK".as_bytes())?;
    let now = Instant::now();

    while now.elapsed() < Duration::from_secs(1) {
        let mut recv_buf: [u8; 10 ^ 9] = [0; 10 ^ 9];

        // Timing recieve
        stream.read(&mut recv_buf)?;
        kb_sent += 1;
    }

    println!("kb sent: {:?}", (kb_sent));

    Ok(())
}
