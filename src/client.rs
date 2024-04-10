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
    let bufsize: usize = args[3].parse::<usize>().unwrap();

    let sk_addr = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::from_str(server_addr).unwrap()),
        server_port,
    );

    let mut stream = TcpStream::connect(sk_addr)?;
    stream.write(format!("{:?}", bufsize).as_bytes())?;

    let mut recv_buf = vec![0; bufsize];
    for _ in 1..10 {
        let mut kb_sent: u64 = 0;
        let now = Instant::now();
        while now.elapsed() < Duration::from_secs(1) {
            // Timing recieve
            stream.read_exact(&mut recv_buf)?;
            kb_sent += 1;
        }

        println!("Rate {:?}Mbps", (kb_sent * 8 * 1024) / 1_000_000);
    }

    Ok(())
}
