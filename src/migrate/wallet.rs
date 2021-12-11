#[rustfmt::skip]
pub(super) const WALLET_VERSIONS: [&str; 3] = [
  "CREATE TABLE IF NOT EXISTS addresses(
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    chain INTEGER NOT NULL,
    indx INTEGER NOT NULL,
    name TEXT NOT NULL,
    address TEXT NOT NULL,
    secret TEXT NOT NULL,
    balance TEXT NOT NULL);",
  "CREATE TABLE IF NOT EXISTS tokens(
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    chain INTEGER NOT NULL,
    network INTEGER NOT NULL,
    name TEXT NOT NULL,
    contract TEXT NOT NULL,
    decimal INTEGER NOT NULL);",
  "INSERT INTO tokens (chain, network, name, contract, decimal) VALUES (2, 1, 'USDT', '0xdac17f958d2ee523a2206206994597c13d831ec7', 6);", // default eth mainnet USDT.
];
