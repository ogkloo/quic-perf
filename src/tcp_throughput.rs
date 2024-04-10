use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

async fn test_client(mut stream: TcpStream) {
    // wait for client to initiate
    let mut recv_buf: [u8; 128] = [0; 128];

    // 1Mb buffer, this should be configurable
    let send_buf: [u8; 1024] = [0; 1024];

    stream.read(&mut recv_buf).await.unwrap();
    loop {
        match stream.write(&send_buf).await {
            Ok(_) => {}
            Err(_) => break,
        };
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let server_addr: &str = &args[1];
    let server_port: u16 = args[2].parse::<u16>().unwrap();

    let sk_addr = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::from_str(server_addr).unwrap()),
        server_port,
    );

    let listener = TcpListener::bind(sk_addr).await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();

        test_client(socket).await;
    }
}
