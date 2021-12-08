#[rustfmt::skip]
pub(super) const WALLET_VERSIONS: [&str; 1] = [
  "CREATE TABLE IF NOT EXISTS addresses(
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    chain INTEGER NOT NULL,
    indx INTEGER NOT NULL,
    name TEXT NOT NULL,
    address TEXT NOT NULL,
    secret TEXT NOT NULL);",
];
