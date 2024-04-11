/// Helper enum denoting available backends.
pub enum Proto {
    /// TCP as implemented by OS.
    Tcp,
    /// Low-level QUIC library designed by Cloudflare.
    Quiche,
    /// Higher level QUIC library.
    Quinn,
}