use std::mem::size_of;
use wasmer::WasmPtr;

#[derive(Debug)]
pub enum MemoryError {
    ExceedsMemorySize,
    ReadError,
    WriteError,
    NullPointer,
}

impl std::fmt::Display for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for MemoryError {}

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
    pub fn new(memory: &wasmer::MemoryView, ptr: u32) -> eyre::Result<MemorySlice, MemoryError> {
        let wasm_ptr = WasmPtr::<MemorySlicePtrBytes>::new(ptr);
        let memory_slice_ptr_bytes = wasm_ptr
            .deref(memory)
            .read()
            .map_err(|_err| MemoryError::ReadError)?;
        let memory_slice = MemorySlice::from_memory_slice_ptr_bytes(memory_slice_ptr_bytes);

        MemorySlice::validate(&memory_slice)?;

        Ok(memory_slice)
    }

    /// Write the given data to the memory slice.
    pub fn write(self, memory: &wasmer::MemoryView, data: &[u8]) -> eyre::Result<(), MemoryError> {
        if data.len() > self.len as usize {
            return Err(MemoryError::ExceedsMemorySize);
        }

        memory
            .write(self.ptr as u64, data)
            .map_err(|_err| MemoryError::WriteError)?;

        Ok(())
    }

    /// Read the memory slice.
    pub fn read(
        self,
        memory: &wasmer::MemoryView,
        max_len: usize,
    ) -> eyre::Result<Vec<u8>, MemoryError> {
        if self.len as usize > max_len {
            return Err(MemoryError::ExceedsMemorySize);
        }

        let mut data = vec![0u8; self.len as usize];
        memory
            .read(self.ptr as u64, &mut data)
            .map_err(|_err| MemoryError::ReadError)?;

        Ok(data)
    }

    /// Convert a `MemorySlicePtrBytes` to a `MemorySlice`.
    fn from_memory_slice_ptr_bytes(bytes: MemorySlicePtrBytes) -> Self {
        MemorySlice {
            ptr: u32::from_le_bytes(bytes[0..4].try_into().unwrap()),
            len: u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
        }
    }

    /// Validate the memory slice.
    fn validate(memory_slice: &MemorySlice) -> eyre::Result<(), MemoryError> {
        if memory_slice.ptr == 0 {
            return Err(MemoryError::NullPointer);
        }

        if memory_slice.len > (u32::MAX - memory_slice.ptr) {
            return Err(MemoryError::ExceedsMemorySize);
        }

        Ok(())
    }
}
