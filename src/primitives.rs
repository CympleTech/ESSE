use tdn::prelude::Peer;

#[inline]
pub(crate) fn network_seeds() -> Vec<Peer> {
    #[rustfmt::skip]
    let seeds: Vec<(&str, &str)> = vec![
        ("1.15.156.199:7364", "quic"),
    ];

    seeds
        .iter()
        .map(|(v, t)| Peer::socket_transport(v.parse().unwrap(), t))
        .collect()
}
