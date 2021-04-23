#[rustfmt::skip]
pub(super) const ACCOUNT_VERSIONS: [&str; 7] = [
  "CREATE TABLE IF NOT EXISTS accounts(
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
  "CREATE TABLE IF NOT EXISTS migrates(
    db_name TEXT NOT NULL,
    version INTEGER NOT NULL);",
  "INSERT INTO migrates (db_name, version) values ('account.db', 1)",
  "INSERT INTO migrates (db_name, version) values ('consensus.db', 9)",
  "INSERT INTO migrates (db_name, version) values ('session.db', 3)",
  "INSERT INTO migrates (db_name, version) values ('file.db', 1)",
  "INSERT INTO migrates (db_name, version) values ('assistant.db', 0)",
];
