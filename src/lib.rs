use std::error::Error;

/// Helper enum denoting available backends.
pub enum Proto {
    /// TCP as implemented by OS.
    Tcp,
    /// Low-level QUIC library designed by Cloudflare.
    Quiche,
    /// Higher level QUIC library.
    Quinn,
}

/// Use the rcgen crate to make self-signed certs
pub fn generate_self_signed_cert(
) -> Result<(rustls::Certificate, rustls::PrivateKey), Box<dyn Error>> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    Ok((
        rustls::Certificate(cert.serialize_der().unwrap()),
        rustls::PrivateKey(cert.serialize_private_key_der()),
    ))
}
