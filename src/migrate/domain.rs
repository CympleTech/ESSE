#[rustfmt::skip]
pub(super) const DOMAIN_VERSIONS: [&str; 3] = [
  "CREATE TABLE IF NOT EXISTS names(
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    provider INTEGER NOT NULL,
    name TEXT NOT NULL,
    bio TEXT NOT NULL,
    is_ok INTEGER NOT NULL,
    is_actived INTEGER NOT NULL);",
  "CREATE TABLE IF NOT EXISTS providers(
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name TEXT NOT NULL,
    addr TEXT NOT NULL,
    is_ok INTEGER NOT NULL,
    is_default INTEGER NOT NULL,
    is_proxy INTEGER NOT NULL,
    is_actived INTEGER NOT NULL);",
  "INSERT INTO providers (name, addr, is_ok, is_default, is_proxy, is_actived) VALUES ('domain.esse', '35d1cc54836a151da67fc8d32ec4d3e92777f42f', true, true, true, true);", // domain.esse default inserted.
];
