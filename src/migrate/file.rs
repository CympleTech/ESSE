#[rustfmt::skip]
pub(super) const FILE_VERSIONS: [&str; 1] = [
  "CREATE TABLE IF NOT EXISTS files(
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    did TEXT NOT NULL,
    parent INTEGER NOT NULL,
    root INTEGER NOT NULL,
    name TEXT NOT NULL,
    starred INTEGER NOT NULL,
    device TEXT NOT NULL,
    datetime INTEGER NOT NULL);",
];
