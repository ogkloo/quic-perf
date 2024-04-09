use std::env;
use std::io::prelude::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::str::FromStr;
use std::time::Instant;

/// Client for all Rust versions.
fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let server_addr: &str = &args[1];
    let server_port: u16 = args[2].parse::<u16>().unwrap();

    let sk_addr = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::from_str(server_addr).unwrap()),
        server_port,
    );

    let n = 20;

    for _ in 1..n {
        let mut stream = TcpStream::connect(sk_addr)?;

        // Establish connection
        stream.write("OK".as_bytes())?;
        let mut recv_buf: [u8; 40 * 1024 ^ 8] = [0; 40 * 1024 ^ 8];

        // Timing recieve
        let now = Instant::now();
        stream.read(&mut recv_buf)?;
        let elapsed = now.elapsed();

        println!("Time: {:.2?}", elapsed);
    }

    Ok(())
}
