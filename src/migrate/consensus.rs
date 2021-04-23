pub(crate) const ACCOUNT_TABLE_PATH: i64 = 0;
pub(crate) const FRIEND_TABLE_PATH: i64 = 1;
pub(crate) const REQUEST_TABLE_PATH: i64 = 2;
pub(crate) const MESSAGE_TABLE_PATH: i64 = 3;
pub(crate) const FILE_TABLE_PATH: i64 = 4;

#[rustfmt::skip]
pub(super) const CONSENSUS_VERSIONS: [&str; 9] = [
  "CREATE TABLE IF NOT EXISTS devices(
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name TEXT NOT NULL,
    info TEXT NOT NULL,
    addr TEXT NOT NULL,
    lasttime INTEGER NOT NULL,
    is_deleted INTEGER NOT NULL);",
  "CREATE TABLE IF NOT EXISTS db_tables(
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    db_name TEXT NOT NULL,
    table_name TEXT NOT NULL);",
  "CREATE TABLE IF NOT EXISTS events(
    id INTEGER NOT NULL,
    hash TEXT NOT NULL,
    db_table INTEGER NOT NULL,
    row INTEGER NOT NULL);",
  "CREATE INDEX events_id
    ON events (id);",
  "CREATE INDEX events_hash
    ON events (hash);",
  "INSERT INTO db_tables (db_name, table_name) values ('session.db', 'friends')",
  "INSERT INTO db_tables (db_name, table_name) values ('session.db', 'requests')",
  "INSERT INTO db_tables (db_name, table_name) values ('session.db', 'messages')",
  "INSERT INTO db_tables (db_name, table_name) values ('file.db', 'files')",
];
