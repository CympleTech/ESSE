#[rustfmt::skip]
pub(crate) const JARVIS_VERSIONS: [&str; 1] = [
  "CREATE TABLE IF NOT EXISTS messages(
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    is_me INTEGER NOT NULL,
    m_type INTEGER NOT NULL,
    content TEXT NOT NULL,
    datetime INTEGER NOT NULL);",
];
