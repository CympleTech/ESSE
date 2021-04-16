#[rustfmt::skip]
pub(super) const ACCOUNT_VERSIONS: [&str; 2] = [
  "CREATE TABLE accounts(
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    gid TEXT NOT NULL,
    name TEXT NOT NULL,
    lock TEXT NOT NULL,
    secret TEXT NOT NULL,
    mnemonic TEXT NOT NULL,
    avatar TEXT NOT NULL,
    height INTEGER NOT NULL,
    event TEXT NOT NULL,
    datetime INTEGER NOT NULL);",
  "CREATE TABLE versions(
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    account_version INTEGER NOT NULL,
    consensus_version INTEGER NOT NULL,
    session_version INTEGER NOT NULL,
    file_version INTEGER NOT NULL,
    service_version INTEGER NOT NULL);",
];
