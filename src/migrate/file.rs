#[rustfmt::skip]
pub(super) const FILE_VERSIONS: [&str; 1] = [
  "CREATE TABLE IF NOT EXISTS files(
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    parent INTEGER NOT NULL,
    f_type INTEGER NOT NULL,
    name TEXT NOT NULL,
    desc TEXT NOT NULL,
    device TEXT NOT NULL,
    datetime INTEGER NOT NULL);",
];
