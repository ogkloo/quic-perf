use clap::Parser;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::{from_utf8, FromStr};
use std::usize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use quic_perf::Proto;

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Server for quic-perf.",
    long_about = "An iperf-like tool for measuring 
          performance across quic backends (and TCP) implemented in Rust."
)]
struct Args {
    /// Address of quic-perf server.
    #[arg(short = 's', long = "server")]
    connection_addr: String,

    /// Port of quic-perf server.
    #[arg(short = 'p', long, default_value_t = 5201)]
    port: u16,

    /// Which backend to use.
    #[arg(long)]
    backend: Option<String>,
}

/// Run a single TCP test for a client.
async fn test_client(mut stream: TcpStream) {
    // wait for client to initiate
    let mut recv_buf: [u8; 128] = [0; 128];
    println!("Connection from {}", stream.peer_addr().unwrap());

    stream.read(&mut recv_buf).await.unwrap();
    let send_bufsize = match from_utf8(&recv_buf) {
        Ok(s) => {
            let s = s.trim_matches(char::from(0));
            match s.parse::<usize>() {
                Ok(s) => s,
                Err(_) => {
                    println!("Parsing failed!");
                    return;
                }
            }
        }
        Err(_) => {
            println!("Parsing failed!");
            return;
        }
    };

    let send_buf = vec![0; send_bufsize];
    loop {
        match stream.write(&send_buf).await {
            Ok(_) => {}
            Err(_) => break,
        };
    }
}

#[tokio::main]
async fn main() {
    let cli = Args::parse();
    let server_addr = cli.connection_addr;
    let server_port = cli.port;
    let backend = match cli.backend {
        Some(proto) => match proto.as_str() {
            "TCP" => Proto::Tcp,
            "Quiche" => Proto::Quiche,
            "Quinn" => Proto::Quinn,
            e => panic!("Unknown protocol: {}", e),
        },
        None => Proto::Tcp,
    };

    match backend {
        Proto::Tcp => {
            let sk_addr = SocketAddr::new(
                IpAddr::V4(Ipv4Addr::from_str(&server_addr).unwrap()),
                server_port,
            );

            let listener = TcpListener::bind(sk_addr).await.unwrap();

            loop {
                let (socket, _) = listener.accept().await.unwrap();

                test_client(socket).await;
            }
        }
        Proto::Quiche => {
            todo!("Quiche not finished")
        }
        Proto::Quinn => {
            todo!("Quinn not finished")
        }
    }
}
