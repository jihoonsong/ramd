use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};

#[derive(Serialize, Deserialize)]
pub struct LiveObjectInfo {
    pub id: String,
    pub hash: Vec<u8>,
    pub wasm_bytes: Vec<u8>,
}

impl TryInto<Vec<u8>> for LiveObjectInfo {
    type Error = serde_json::Error;

    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        serde_json::to_vec(&self)
    }
}

impl TryFrom<Vec<u8>> for LiveObjectInfo {
    type Error = serde_json::Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        serde_json::from_slice(&bytes)
    }
}

impl LiveObjectInfo {
    pub fn new(wasm_bytes: Vec<u8>) -> Self {
        let hash = Keccak256::digest(&wasm_bytes).to_vec();

        // TODO: modify `id` to be defined when the LiveObject is instantiated.
        let id = hex::encode(hash.clone());

        Self {
            id,
            hash,
            wasm_bytes,
        }
    }
}
