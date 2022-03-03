#[rustfmt::skip]
pub(super) const SESSION_VERSIONS: [&str; 2] = [
  "CREATE TABLE IF NOT EXISTS sessions(
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    fid INTEGER NOT NULL,
    pid TEXT NOT NULL,
    addr TEXT NOT NULL,
    s_type INTEGER NOT NULL,
    name TEXT NOT NULL,
    is_top INTEGER NOT NULL,
    is_close INTEGER NOT NULL,
    last_datetime INTEGER,
    last_content TEXT,
    last_readed INTEGER);",
  "INSERT INTO sessions (fid, pid, addr, s_type, name, is_top, is_close, last_datetime, last_content, last_readed) VALUES (0, '', '', 3, '', 0, 0, 0, '', 1);", // Jarvis.
];
