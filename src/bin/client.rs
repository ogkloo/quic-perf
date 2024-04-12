use clap::Parser;
use quinn::ClientConfig;
// use rustls::client;
use std::io::prelude::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use ring::rand::*;

use quic_perf::{hex_dump, Proto};

const MAX_DATAGRAM_SIZE: usize = 1350;

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Client for quic-perf.",
    long_about = "An iperf-like tool for measuring 
          performance across quic backends (and TCP) implemented in Rust."
)]
struct Args {
    /// Address of quic-perf server.
    #[arg(short = 'c', long = "server")]
    connection_addr: String,

    /// Port of quic-perf server.
    #[arg(short = 'p', long, default_value_t = 5201)]
    port: u16,

    /// Size of buffer to attempt to send in bytes.
    #[arg(short, long, default_value_t = 1024)]
    bufsize: usize,

    /// Time to run for in seconds.
    #[arg(short, long, default_value_t = 10)]
    time: usize,

    /// Ouput format in order of magnitude.
    ///
    /// Output format in order of magnitude. Use capital letters for values
    /// in bits, lowercase for values in bytes. E.g. M for megabits per second,
    /// m for megabytes per second.
    #[arg(short, long, default_value_t = 'M')]
    format: char,

    /// Precision of rate output.
    #[arg(short = 'r', long, default_value_t = 2)]
    precision: usize,

    /// Maximum transmission to send in an interval. Warning: Broken, does
    /// not give accurate rate estimate.
    ///
    /// Maximum transmission to send in an interval. By default there is no
    /// limit, it will actually saturate the link. This can be an issue if you
    /// are for instance on a public or metered wireless connection.
    ///
    /// Measured in bytes.
    #[arg(short, long)]
    max_transmission: Option<usize>,

    /// Protocol to use.
    #[arg(long)]
    backend: Option<String>,
}

/// Implementation of `ServerCertVerifier` that verifies everything as trustworthy.
struct SkipServerVerification;

impl SkipServerVerification {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl rustls::client::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}

fn configure_client() -> ClientConfig {
    let crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_no_client_auth();

    ClientConfig::new(Arc::new(crypto))
}

/// Client for all Rust versions.
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let cli = Args::parse();
    let server_addr = cli.connection_addr;
    let server_port = cli.port;
    let bufsize = cli.bufsize;
    let time = cli.time;
    let format = match cli.format {
        // Bits per second.
        'K' => 1_000,
        'M' => 1_000_000,
        'G' => 1_000_000_000,
        // Bytes per second. Must divide off the 8.
        'k' => 8_192,
        'm' => 8_388_608,
        'g' => 8_589_934_592,
        e => panic!("Invalid format argument: {:?}", e),
    };
    let format_string = match cli.format {
        // Bits
        'K' => "Kbits",
        'M' => "Mbits",
        'G' => "Gbits",
        // Bytes
        'k' => "KB",
        'm' => "MB",
        'g' => "GB",
        // Unreachable
        _ => "",
    };
    let precision = cli.precision;
    let max_transmission = cli.max_transmission;
    let backend = match cli.backend {
        Some(proto) => match proto.as_str() {
            "TCP" => Proto::Tcp,
            "Quiche" => Proto::Quiche,
            "Quinn" => Proto::Quinn,
            e => panic!("Unknown protocol: {}", e),
        },
        None => Proto::Tcp,
    };

    let sk_addr = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::from_str(&server_addr).unwrap()),
        server_port,
    );
    match backend {
        Proto::Tcp => {
            let mut stream = TcpStream::connect(sk_addr)?;
            stream.write(format!("{:?}", bufsize).as_bytes())?;

            println!("Interval \t Transfer \t Rate");
            let mut recv_buf = vec![0; bufsize];
            let mut total_buffers_sent: usize = 0;
            for t in 0..time {
                let mut buffers_sent: usize = 0;
                let now = Instant::now();
                // If the user has set a maximum size for the interval.
                let mut approx_rate: f64 = 0.0;
                while now.elapsed() < Duration::from_secs(1) {
                    stream.read_exact(&mut recv_buf)?;
                    buffers_sent += 1;
                    if let Some(m_tr) = max_transmission {
                        if m_tr < (buffers_sent * bufsize) {
                            let time_spent = now.elapsed().as_secs_f64();
                            approx_rate = (buffers_sent * bufsize) as f64 / time_spent;
                            println!("{}", time_spent);
                            break;
                        }
                    }
                }

                if let Some(_) = max_transmission {
                    println!(
                        "{} \t\t {}{} \t {:.*}{}/s",
                        t + 1,
                        (buffers_sent * 8 * bufsize) / format,
                        format_string,
                        precision,
                        approx_rate / format as f64,
                        format_string
                    );
                } else {
                    println!(
                        "{} \t\t {:.*}{} \t {:.*}{}/s",
                        t + 1,
                        precision,
                        (buffers_sent * 8 * bufsize) as f64 / format as f64,
                        format_string,
                        precision,
                        (buffers_sent as f64 * 8.0 * bufsize as f64) / format as f64,
                        format_string
                    );
                }
                total_buffers_sent += buffers_sent;
            }

            println!(
                "Average rate over {} seconds: {:.*}{}",
                time,
                precision,
                (total_buffers_sent as f64 * 8.0 * bufsize as f64) / (format * time) as f64,
                format_string
            );
        }

        Proto::Quiche => {
            let mut config =
                quiche::Config::new(quiche::PROTOCOL_VERSION).expect("Quiche config failed");
            config
                .set_application_protos(&[b"quic-perf"])
                .expect("ALPN configuration failed");
            // WARNING: Turns off MitM protection.
            config.verify_peer(false);
            config.set_max_idle_timeout(5000);
            config.set_max_recv_udp_payload_size(MAX_DATAGRAM_SIZE);
            config.set_max_send_udp_payload_size(MAX_DATAGRAM_SIZE);
            config.set_initial_max_data(10_000_000);
            config.set_initial_max_stream_data_bidi_local(1_000_000);
            config.set_initial_max_stream_data_bidi_remote(1_000_000);
            config.set_initial_max_streams_bidi(100);
            config.set_initial_max_streams_uni(100);
            config.set_disable_active_migration(true);

            let mut poll = mio::Poll::new().unwrap();
            let mut events = mio::Events::with_capacity(1024);

            let client_addr = "0.0.0.0:0".parse::<SocketAddr>().unwrap();

            let mut socket = mio::net::UdpSocket::bind(client_addr).expect(&format!(
                "Failed to bind socket with address {:?}",
                client_addr
            ));
            poll.registry()
                .register(&mut socket, mio::Token(0), mio::Interest::READABLE)
                .expect("Failed to register socket to event poll");

            // Connection ID setup
            let mut scid = [0; quiche::MAX_CONN_ID_LEN];
            SystemRandom::new().fill(&mut scid[..]).unwrap();
            let scid = quiche::ConnectionId::from_ref(&scid);

            // Prepare for connection
            let socket_addr = socket.local_addr().unwrap();
            let mut conn = quiche::connect(None, &scid, socket_addr, sk_addr, &mut config)
                .expect("Failed to connect to server");

            println!(
                "Connecting to {:} on {:} with SCID {:}",
                sk_addr,
                socket_addr,
                hex_dump(&scid)
            );

            let mut buf = [0; 65535];
            let mut out = [0; MAX_DATAGRAM_SIZE];
        }

        Proto::Quinn => {
            // TODO: Make this actually random so it doesn't potentially fail to bind.
            //
            // As is, if port 43422 is taken the client will just die.
            let client_addr = "0.0.0.0:0".parse::<SocketAddr>().unwrap();
            println!("Connecting to {:?} on {:?}", sk_addr, client_addr);
            // Configure crypto
            let client_config = configure_client();
            let client = quinn::Endpoint::client(client_addr).unwrap();
            // Establish connection
            let connection = client
                .connect_with(client_config, sk_addr, "example.com")
                .unwrap()
                .await?;
            println!("Connected to {:?} on {:?}", sk_addr, client_addr);

            let (mut send, mut recv) = connection.open_bi().await?;

            // Send buffer size preference
            send.write_all(format!("{:?}", bufsize).as_bytes()).await?;
            send.finish().await?;

            println!("Interval \t Transfer \t Rate");
            let mut total_buffers_sent = 0;

            for t in 0..time {
                let mut buffers_sent = 0;
                let now = Instant::now();

                while now.elapsed() < Duration::from_secs(1) {
                    let _ = recv.read_to_end(bufsize * 2).await;
                    buffers_sent += 1;
                }

                println!(
                    "{} \t\t {:.*}{} \t {:.*}{}/s",
                    t + 1,
                    precision,
                    (buffers_sent * 8 * bufsize) as f64 / format as f64,
                    format_string,
                    precision,
                    (buffers_sent as f64 * 8.0 * bufsize as f64) / format as f64,
                    format_string
                );

                total_buffers_sent += buffers_sent;
            }

            println!(
                "Average rate over {} seconds: {:.*}{}",
                time,
                precision,
                (total_buffers_sent as f64 * 8.0 * bufsize as f64) / (format * time) as f64,
                format_string
            );
        }
    }

    Ok(())
}
