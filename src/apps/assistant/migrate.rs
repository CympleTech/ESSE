#[rustfmt::skip]
pub(crate) const ASSISTANT_VERSIONS: [&str; 1] = [
  "CREATE TABLE IF NOT EXISTS messages(
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    q_type INTEGER NOT NULL,
    q_content TEXT NOT NULL,
    a_type INTEGER NOT NULL,
    a_content TEXT NOT NULL,
    datetime INTEGER NOT NULL,
    is_deleted INTEGER NOT NULL);",
];
