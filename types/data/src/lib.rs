pub type DataId = [u8; 32];

pub trait OwnerId: Clone + Eq + PartialEq {
    type Proof: Clone + Eq + PartialEq;

    fn len() -> usize;
    fn proof_len() -> usize;

    fn verify(&self, data: &Data<Self>) -> bool;
    fn id_to_bytes(&self) -> Vec<u8>;
    fn proof_to_bytes(proof: &Self::Proof) -> Vec<u8>;
    fn id_from_bytes(id_bytes: &[u8]) -> Result<Self, ()>;
    fn proof_from_bytes(proof_bytes: &[u8]) -> Result<Self::Proof, ()>;
}

/// common data structure.
#[derive(Clone, Eq, PartialEq)]
pub struct Data<T: OwnerId> {
    /// Data unique ID, default generate method is other fields hash.
    pub did: DataId,
    /// ParentID, default is None.
    pub pid: Option<DataId>,
    /// time lifetime, (from, to) timestamp.
    pub time: (i64, i64),
    /// Owner's info.
    pub owner: T,
    /// data owner proof. it can verify by owner.
    pub proof: T::Proof,
    /// MIME type, and value bytes.
    pub value: (String, Vec<u8>),
}

impl<T: OwnerId> Data<T> {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend_from_slice(&self.did);
        if let Some(p) = self.pid {
            bytes.extend_from_slice(&p);
        } else {
            bytes.extend_from_slice(&[0u8; 32]);
        }
        bytes.extend_from_slice(&mut self.time.0.to_le_bytes());
        bytes.extend_from_slice(&mut self.time.1.to_le_bytes());
        bytes.extend_from_slice(&mut self.owner.id_to_bytes());
        bytes.extend_from_slice(&mut T::proof_to_bytes(&self.proof));

        let mut mime_bytes = self.value.0.as_bytes();
        bytes.extend_from_slice(&mut (mime_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&mut mime_bytes);
        bytes.extend_from_slice(&self.value.1);

        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ()> {
        if bytes.len() < 84 + T::len() + T::proof_len() {
            return Err(());
        }

        let mut did_bytes = [0u8; 32];
        did_bytes.copy_from_slice(&bytes[0..32]);
        let mut pid_bytes = [0u8; 32];
        pid_bytes.copy_from_slice(&bytes[32..64]);
        let pid = if pid_bytes == [0u8; 32] {
            None
        } else {
            Some(pid_bytes)
        };
        let mut start_bytes = [0u8; 8];
        start_bytes.copy_from_slice(&bytes[64..72]);
        let start_time = i64::from_le_bytes(start_bytes);

        let mut end_bytes = [0u8; 8];
        end_bytes.copy_from_slice(&bytes[72..80]);
        let end_time = i64::from_le_bytes(end_bytes);

        let owner = T::id_from_bytes(&bytes[80..])?;
        let proof = T::proof_from_bytes(&bytes[80 + T::len()..])?;

        let m_l = 80 + T::len() + T::proof_len();
        let mut mime_len_bytes = [0u8; 4];
        mime_len_bytes.copy_from_slice(&bytes[m_l..m_l + 4]);
        let mime_len = u32::from_le_bytes(mime_len_bytes) as usize;
        let last = &bytes[m_l + 4..];
        if last.len() < mime_len {
            return Err(());
        }
        let v_t = std::str::from_utf8(&last[0..mime_len])
            .map_err(|_| ())?
            .to_owned();
        let v_v = last[mime_len..].to_vec();

        Ok(Self {
            pid,
            owner,
            proof,
            did: did_bytes,
            value: (v_t, v_v),
            time: (start_time, end_time),
        })
    }
}

#[cfg(feature = "tdn")]
pub mod tdn;
