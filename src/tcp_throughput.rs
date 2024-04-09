use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

async fn test_client(mut stream: TcpStream) {
    // wait for client to initiate
    let mut recv_buf: [u8; 128] = [0; 128];

    // 1Mb buffer, this should be configurable
    let send_buf: [u8; 40 * 1024 ^ 8] = [0; 40 * 1024 ^ 8];

    stream.read(&mut recv_buf).await.unwrap();
    stream.write(&send_buf).await.unwrap();
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:5201").await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();

        test_client(socket).await;
    }
}
