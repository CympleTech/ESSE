use std::net::SocketAddr;

#[inline]
pub(crate) fn network_seeds() -> Vec<SocketAddr> {
    #[rustfmt::skip]
    let seeds: Vec<&str> = vec![
        "1.15.156.199:7364",
    ];

    seeds.iter().map(|v| v.parse().unwrap()).collect()
}
