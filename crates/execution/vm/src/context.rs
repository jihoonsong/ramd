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

use std::sync::Arc;

use crate::{MemorySlice, MemorySliceError, MemorySlicePtr, MAX_WASM_MEMORY_SIZE};
use ramd_db::storage::Storage;
use wasmer::{Function, Memory, StoreMut, Value};

/// The context of the WASM (guest) instance that is shared across import functions.
pub struct Context<S>
where
    S: Storage<Vec<u8>, Vec<u8>> + 'static,
{
    pub storage: Arc<S>,
    pub key_prefix: Vec<u8>,
    pub memory: Option<Memory>,
    pub allocate: Option<Function>,
}

impl<S> Context<S>
where
    S: Storage<Vec<u8>, Vec<u8>> + 'static,
{
    /// Create a new `Context`.
    pub fn new(storage: Arc<S>, key_prefix: String) -> Self {
        Self {
            storage,
            key_prefix: key_prefix.as_bytes().to_vec(),
            memory: None,
            allocate: None,
        }
    }

    pub fn prefix_key(&self, mut key: Vec<u8>) -> Vec<u8> {
        let mut prefixed_key = self.key_prefix.clone();
        prefixed_key.append(&mut key);
        prefixed_key
    }

    /// Read data from the WASM (guest) memory.
    pub fn read_memory(
        &self,
        store: &StoreMut,
        memory_slice_ptr: MemorySlicePtr,
    ) -> eyre::Result<Vec<u8>, MemorySliceError> {
        let memory = self.memory();
        let memory_view = memory.view(store);
        let memory_slice = MemorySlice::new(&memory_view, memory_slice_ptr)?;

        let data = memory_slice.read(&memory_view, MAX_WASM_MEMORY_SIZE)?;

        Ok(data)
    }

    /// Write data to the WASM (guest) memory.
    pub fn write_memory(
        &self,
        store: &StoreMut,
        memory_slice_ptr: MemorySlicePtr,
        data: &[u8],
    ) -> eyre::Result<(), MemorySliceError> {
        let memory = self.memory();
        let memory_view = memory.view(store);
        let memory_slice = MemorySlice::new(&memory_view, memory_slice_ptr)?;

        memory_slice.write(&memory_view, data)?;

        Ok(())
    }

    /// Allocate memory in the WASM (guest) memory.
    pub fn allocate_memory(
        &self,
        store: &mut StoreMut,
        len: usize,
    ) -> eyre::Result<MemorySlicePtr> {
        let memory_slice_ptr = self.allocate().call(store, &[Value::from(len as u32)])?;

        let memory_slice_ptr: MemorySlicePtr = memory_slice_ptr[0]
            .clone()
            .try_into()
            .map_err(|err: &str| eyre::eyre!(err))?;

        Ok(memory_slice_ptr)
    }

    /// Get the memory of the WASM instance.
    fn memory(&self) -> &Memory {
        self.memory
            .as_ref()
            .expect("ramd::VM: Context must have Memory")
    }

    /// Get the function to allocate memory in the WASM instance.
    fn allocate(&self) -> &Function {
        self.allocate
            .as_ref()
            .expect("ramd::VM: Context must have allocate function")
    }
}
