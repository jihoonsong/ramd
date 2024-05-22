use std::mem::size_of;

use crate::MAX_WASM_MEMORY_SIZE;
use wasmer::{MemoryAccessError, WasmPtr};

#[derive(Debug)]
pub enum MemorySliceError {
    ExceedMaxWASMMemorySize,
    ExceedMemorySliceSize,
    ReadError(MemoryAccessError),
    WriteError(MemoryAccessError),
    NullPointer,
}

impl std::fmt::Display for MemorySliceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for MemorySliceError {}

impl From<MemorySliceError> for wasmer::RuntimeError {
    fn from(err: MemorySliceError) -> Self {
        wasmer::RuntimeError::new(err.to_string())
    }
}

/// A block of memory in the WASM (guest) memory.
pub struct MemorySlice {
    /// A pointer to the start of this memory slice,
    /// measured in bytes from the beginning of the WASM (guest) memory.
    pub ptr: u32,
    /// The number of bytes in this memory slice.
    pub len: u32,
}

pub type MemorySlicePtr = u32;

type MemorySlicePtrBytes = [u8; size_of::<MemorySlice>()];

impl MemorySlice {
    /// Read in a `MemorySlice` from the WASM (guest) memory and return it.
    pub fn new(
        memory: &wasmer::MemoryView,
        ptr: u32,
    ) -> eyre::Result<MemorySlice, MemorySliceError> {
        let wasm_ptr = WasmPtr::<MemorySlicePtrBytes>::new(ptr);
        let memory_slice_ptr_bytes = wasm_ptr
            .deref(memory)
            .read()
            .map_err(|err| MemorySliceError::ReadError(err))?;
        let memory_slice = MemorySlice::from_memory_slice_ptr_bytes(memory_slice_ptr_bytes);

        MemorySlice::validate(&memory_slice)?;

        Ok(memory_slice)
    }

    /// Read the memory slice.
    pub fn read(
        self,
        memory: &wasmer::MemoryView,
        max_len: usize,
    ) -> eyre::Result<Vec<u8>, MemorySliceError> {
        if self.len as usize > max_len {
            return Err(MemorySliceError::ExceedMemorySliceSize);
        }

        let mut data = vec![0u8; self.len as usize];

        memory
            .read(self.ptr as u64, &mut data)
            .map_err(|err| MemorySliceError::ReadError(err))?;

        Ok(data)
    }

    /// Write the given data to the memory slice.
    pub fn write(
        self,
        memory: &wasmer::MemoryView,
        data: &[u8],
    ) -> eyre::Result<(), MemorySliceError> {
        if data.len() > self.len as usize {
            return Err(MemorySliceError::ExceedMemorySliceSize);
        }

        memory
            .write(self.ptr as u64, data)
            .map_err(|err| MemorySliceError::WriteError(err))?;

        Ok(())
    }

    /// Convert a `MemorySlicePtrBytes` to a `MemorySlice`.
    fn from_memory_slice_ptr_bytes(bytes: MemorySlicePtrBytes) -> Self {
        MemorySlice {
            ptr: u32::from_le_bytes(bytes[0..4].try_into().unwrap()),
            len: u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
        }
    }

    /// Validate the memory slice.
    fn validate(memory_slice: &MemorySlice) -> eyre::Result<(), MemorySliceError> {
        if memory_slice.ptr == 0 {
            return Err(MemorySliceError::NullPointer);
        }

        if memory_slice.len > (MAX_WASM_MEMORY_SIZE as u32 - memory_slice.ptr) {
            return Err(MemorySliceError::ExceedMaxWASMMemorySize);
        }

        Ok(())
    }
}
