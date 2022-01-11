use tdn_did::{Proof, PROOF_LENGTH};
use tdn_types::group::{GroupId, GROUP_LENGTH};

use crate::{Data, OwnerId};

impl OwnerId for GroupId {
    type Proof = Proof;

    fn len() -> usize {
        PROOF_LENGTH
    }

    fn proof_len() -> usize {
        GROUP_LENGTH
    }

    fn verify(&self, data: &Data<Self>) -> bool {
        data.proof
            .verify_bytes(&data.owner, &data.to_bytes())
            .is_ok()
    }

    fn id_to_bytes(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    fn proof_to_bytes(proof: &Self::Proof) -> Vec<u8> {
        proof.0.clone()
    }

    fn id_from_bytes(id_bytes: &[u8]) -> Result<Self, ()> {
        if id_bytes.len() >= GROUP_LENGTH {
            let mut bytes = [0u8; GROUP_LENGTH];
            bytes.copy_from_slice(&id_bytes[0..GROUP_LENGTH]);
            Ok(GroupId(bytes))
        } else {
            Err(())
        }
    }

    fn proof_from_bytes(proof_bytes: &[u8]) -> Result<Self::Proof, ()> {
        if proof_bytes.len() >= PROOF_LENGTH {
            let bytes = proof_bytes[0..PROOF_LENGTH].to_vec();
            Ok(Proof(bytes))
        } else {
            Err(())
        }
    }
}

pub type TdnData = Data<GroupId>;
