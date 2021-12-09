#[rustfmt::skip]
pub(super) const WALLET_VERSIONS: [&str; 2] = [
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
];
