#[rustfmt::skip]
pub(super) const WALLET_VERSIONS: [&str; 5] = [
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
  "CREATE TABLE IF NOT EXISTS balances(
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    address INTEGER NOT NULL,
    token INTEGER NOT NULL,
    value TEXT NOT NULL);",
  "INSERT INTO tokens (chain, network, name, contract, decimal) VALUES (2, 1, 'USDT', '0xdac17f958d2ee523a2206206994597c13d831ec7', 6);", // default eth mainnet USDT.
  "INSERT INTO tokens (chain, network, name, contract, decimal) VALUES (3, 2, 'ESNFT', '0x5721d49269A4a73CEAD795D269076b555e27eD6C', 0);", // default ESSE NFT ropsten.
];
