#[rustfmt::skip]
pub(super) const CHAT_VERSIONS: [&str; 3] = [
  "CREATE TABLE IF NOT EXISTS friends(
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    gid TEXT NOT NULL,
    addr TEXT NOT NULL,
    name TEXT NOT NULL,
    remark TEXT,
    is_closed INTEGER NOT NULL,
    datetime INTEGER NOT NULL,
    is_deleted INTEGER NOT NULL);",
  "CREATE TABLE IF NOT EXISTS requests(
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    gid TEXT NOT NULL,
    addr TEXT NOT NULL,
    name TEXT,
    remark TEXT,
    is_me INTEGER NOT NULL,
    is_ok INTEGER NOT NULL,
    is_over INTEGER NOT NULL,
    is_delivery INTEGER NOT NULL,
    datetime INTEGER NOT NULL,
    is_deleted INTEGER NOT NULL);",
  "CREATE TABLE IF NOT EXISTS messages(
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    hash TEXT NOT NULL,
    fid INTEGER NOT NULL,
    is_me INTEGER NOT NULL,
    m_type INTEGER NOT NULL,
    content TEXT NOT NULL,
    is_delivery INTEGER NOT NULL,
    datetime INTEGER NOT NULL,
    is_deleted INTEGER NOT NULL);",
];
