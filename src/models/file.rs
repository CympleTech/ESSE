use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) enum FileType {
    Dir,
    File,
    SessionFile,
    ServiceFile,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct FileId([u8; 32]);

pub(crate) struct _File {
    pub id: i64,
    pub file_id: [u8; 32],
    pub parent: i64,
    pub f_type: FileType,
    pub name: String,
    pub desc: String,
    pub device: Vec<i64>,
    pub datetime: i64,
}
