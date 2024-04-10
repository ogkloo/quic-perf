use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::{from_utf8, FromStr};
use std::{env, usize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

async fn test_client(mut stream: TcpStream) {
    // wait for client to initiate
    let mut recv_buf: [u8; 128] = [0; 128];

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
