use tdn::prelude::Peer;

#[inline]
pub(crate) fn network_seeds() -> Vec<Peer> {
    #[rustfmt::skip]
    let seeds: Vec<(&str, &str)> = vec![
        ("1.15.156.199:7364", "quic"),    // CN
        ("184.170.220.231:7364", "quic"), // US
        //("8.214.101.49:7364", "tcp"),     // SG
    ];

    seeds
        .iter()
        .map(|(v, t)| Peer::socket_transport(v.parse().unwrap(), t))
        .collect()
}
