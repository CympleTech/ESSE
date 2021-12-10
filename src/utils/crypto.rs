use aes_gcm::{
    aead::{generic_array::GenericArray, Aead, NewAead},
    Aes256Gcm,
};
use sha2::{Digest, Sha256};

const FIX_PADDING: [u8; 19] = [
    69, 83, 83, 69, 70, 111, 114, 68, 97, 116, 97, 83, 101, 99, 117, 114, 105, 116, 121,
];

/// Hash the given pin.
pub fn hash_pin(salt: &[u8], pin: &str, index: i64) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(salt); // for avoid same hash when no-pin in other derives.
    hasher.update(pin.as_bytes());
    hasher.update(index.to_le_bytes()); // for avoid same hash when no-pin in one device.
    hasher.finalize().to_vec()
}

/// check the pin is the given hash pre-image.
pub fn check_pin(salt: &[u8], pin: &str, index: i64, hash: &[u8]) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(salt);
    hasher.update(pin.as_bytes());
    hasher.update(index.to_le_bytes());
    let hash_key = hasher.finalize();
    &hash_key[..] == hash
}

fn build_cipher(salt: &[u8], pin: &str) -> Aes256Gcm {
    let mut hasher = blake3::Hasher::new();
    hasher.update(salt);
    hasher.update(pin.as_bytes());
    hasher.update(&FIX_PADDING);
    let hash_key = hasher.finalize();
    Aes256Gcm::new(GenericArray::from_slice(hash_key.as_bytes())) // 256-bit key.
}

/// encrypted bytes.
pub fn encrypt(salt: &[u8], pin: &str, ptext: &[u8]) -> anyhow::Result<Vec<u8>> {
    let cipher = build_cipher(salt, pin);

    let mut nonce = Sha256::new();
    nonce.update(pin.as_bytes());
    nonce.update(&FIX_PADDING);
    let res = nonce.finalize();
    let nonce = GenericArray::from_slice(&res[0..12]); // 96-bit key.

    cipher
        .encrypt(nonce, ptext)
        .or(Err(anyhow!("encrypt data failure.")))
}

pub fn encrypt_multiple(salt: &[u8], pin: &str, ptext: Vec<&[u8]>) -> anyhow::Result<Vec<Vec<u8>>> {
    let cipher = build_cipher(salt, pin);

    let mut nonce = Sha256::new();
    nonce.update(pin.as_bytes());
    nonce.update(&FIX_PADDING);
    let res = nonce.finalize();
    let nonce = GenericArray::from_slice(&res[0..12]); // 96-bit key.

    let mut ebytes = vec![];
    for p in ptext {
        ebytes.push(
            cipher
                .encrypt(nonce, p)
                .or(Err(anyhow!("encrypt data failure.")))?,
        );
    }
    Ok(ebytes)
}

/// decrypted bytes.
pub fn decrypt(salt: &[u8], pin: &str, ctext: &[u8]) -> anyhow::Result<Vec<u8>> {
    let cipher = build_cipher(salt, pin);

    let mut nonce = Sha256::new();
    nonce.update(pin.as_bytes());
    nonce.update(&FIX_PADDING);
    let res = nonce.finalize();
    let nonce = GenericArray::from_slice(&res[0..12]); // 96-bit key.

    cipher
        .decrypt(nonce, ctext)
        .or(Err(anyhow!("decrypt data failure.")))
}

pub fn decrypt_multiple(salt: &[u8], pin: &str, ctext: Vec<&[u8]>) -> anyhow::Result<Vec<Vec<u8>>> {
    let cipher = build_cipher(salt, pin);

    let mut nonce = Sha256::new();
    nonce.update(pin.as_bytes());
    nonce.update(&FIX_PADDING);
    let res = nonce.finalize();
    let nonce = GenericArray::from_slice(&res[0..12]); // 96-bit key.

    let mut pbytes = vec![];
    for c in ctext {
        pbytes.push(
            cipher
                .decrypt(nonce, c)
                .or(Err(anyhow!("decrypt data failure.")))?,
        );
    }
    Ok(pbytes)
}
