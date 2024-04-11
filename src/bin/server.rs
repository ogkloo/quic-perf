use clap::Parser;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::{from_utf8, FromStr};
use std::usize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use quic_perf::{generate_self_signed_cert, Proto};

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
    #[arg(short, long)]
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
            // for the server side:
            let server_addr = "127.0.0.1:5001".parse::<SocketAddr>().unwrap();
            let (cert, key) = generate_self_signed_cert().expect("Cert generation failed.");
            let server_config = quinn::ServerConfig::with_single_cert(vec![cert], key)
                .expect("Cert config failed.");
            let server = quinn::Endpoint::server(server_config, server_addr).unwrap();
            while let Some(handshake) = server.accept().await {
                let connection = handshake.await.unwrap();
                while let Ok((mut send, mut recv)) = connection.accept_bi().await {
                    // Because it is a bidirectional stream, we can both send and receive.
                    println!("request: {:?}", recv.read_to_end(50).await.unwrap());
            
                    send.write_all(b"response").await.unwrap();
                    send.finish().await.unwrap();
                }

            }
        }
    }
}
