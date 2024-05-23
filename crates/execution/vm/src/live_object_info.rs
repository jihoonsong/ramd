// Copyright (C) 2024 Jihoon Song

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

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
