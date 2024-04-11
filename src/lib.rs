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
pub fn generate_self_signed_cert() -> Result<(rustls::Certificate, rustls::PrivateKey), Box<dyn Error>>
{
    let alt_name = vec!["localhost".to_string()];
    let rcgen::CertifiedKey {cert, key_pair: _ }: rcgen::CertifiedKey = rcgen::generate_simple_self_signed(alt_name)?;
    let key = rustls::PrivateKey(cert.der().to_vec());
    let cert = rustls::Certificate(cert.der().to_vec());
    Ok((cert, key))
}
