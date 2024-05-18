use std::sync::Arc;

use crate::{ImportObject, MemorySlice, MemorySlicePtr};

use ramd_db::storage::Storage;
use tracing::error;
use wasmer::{Instance, Module, Store, Value};

pub struct Context<S> {
    _storage: Arc<S>,
    store: Store,
    instance: Instance,
}

impl<S> Context<S>
where
    S: Storage<Vec<u8>, Vec<u8>>,
{
    pub fn new(storage: Arc<S>, wasm_bytes: Vec<u8>) -> eyre::Result<Self> {
        // Create a Store.
        let mut store = Store::default();

        // Compile the WASM module.
        let module = match Module::new(&store, wasm_bytes) {
            Ok(module) => module,
            Err(e) => {
                error!(target: "ramd::vm", "Failed to compile the WASM module with error `{}`", e.to_string());
                return Err(e.into());
            }
        };

        // Create an import object.
        let import_object = ImportObject::new();

        // Instantiate the WASM module.
        let instance = match Instance::new(&mut store, &module, &import_object.0) {
            Ok(instance) => instance,
            Err(e) => {
                error!(target: "ramd::vm", "Failed to instantiate the WASM module with error `{}`", e.to_string());
                return Err(e.into());
            }
        };

        Ok(Context {
            _storage: storage,
            store,
            instance,
        })
    }

    pub fn call_function(
        &mut self,
        method: &str,
        params_ptr: Vec<Value>,
    ) -> eyre::Result<MemorySlicePtr> {
        let func = match self.instance.exports.get_function(method) {
            Ok(func) => func,
            Err(e) => {
                error!(target: "ramd::vm", "Failed to get the function `{}` with error `{}`", method, e.to_string());
                return Err(e.into());
            }
        };

        let result_ptr = match func.call(&mut self.store, &params_ptr) {
            Ok(result_ptr) => result_ptr,
            Err(e) => {
                error!(target: "ramd::vm", "Failed to call the function `{}` with error `{}`", method, e.to_string());
                return Err(e.into());
            }
        };

        let result_ptr: MemorySlicePtr = match result_ptr[0].clone().try_into() {
            Ok(result_ptr) => result_ptr,
            Err(e) => {
                error!(target: "ramd::vm", "Failed to convert the result pointer with error `{}`", e.to_string());
                return Err(eyre::eyre!(e.to_string()));
            }
        };

        Ok(result_ptr)
    }

    pub fn allocate_memory(&mut self, size: u32) -> eyre::Result<MemorySlicePtr> {
        let allocate_func = match self.instance.exports.get_function("allocate") {
            Ok(func) => func,
            Err(e) => {
                error!(target: "ramd::vm", "Failed to get the function `allocate` with error `{}`", e.to_string());
                return Err(e.into());
            }
        };

        let memory_slice_ptr = match allocate_func.call(&mut self.store, &[size.into()]) {
            Ok(memory_slice_ptr) => memory_slice_ptr,
            Err(e) => {
                error!(target: "ramd::vm", "Failed to allocate memory with error `{}`", e.to_string());
                return Err(e.into());
            }
        };

        let memory_slice_ptr = memory_slice_ptr[0].clone();
        let memory_slice_ptr: MemorySlicePtr = match memory_slice_ptr.try_into() {
            Ok(memory_slice_ptr) => memory_slice_ptr,
            Err(e) => {
                error!(target: "ramd::vm", "Failed to convert the memory pointer with error `{}`", e.to_string());
                return Err(eyre::eyre!(e.to_string()));
            }
        };

        // Now `memory_slice_ptr` is a pointer to a struct MemorySlice in WASM memory.

        Ok(memory_slice_ptr)
    }

    pub fn deallocate_memory(&mut self, memory_slice_ptr: MemorySlicePtr) -> eyre::Result<()> {
        let deallocate_func = match self.instance.exports.get_function("deallocate") {
            Ok(func) => func,
            Err(e) => {
                error!(target: "ramd::vm", "Failed to get the function `deallocate` with error `{}`", e.to_string());
                return Err(e.into());
            }
        };

        if let Err(e) = deallocate_func.call(&mut self.store, &[memory_slice_ptr.into()]) {
            error!(target: "ramd::vm", "Failed to deallocate memory with error `{}`", e.to_string());
            return Err(e.into());
        }

        Ok(())
    }

    pub fn read_memory(&mut self, memory_slice_ptr: MemorySlicePtr) -> eyre::Result<String> {
        let memory = match self.instance.exports.get_memory("memory") {
            Ok(memory) => memory,
            Err(e) => {
                error!(target: "ramd::vm", "Failed to get the memory `memory` with error `{}`", e.to_string());
                return Err(e.into());
            }
        };

        let memory_view = memory.view(&self.store);
        let memory_slice = match MemorySlice::new(&memory_view, memory_slice_ptr) {
            Ok(memory_slice) => memory_slice,
            Err(e) => {
                error!(target: "ramd::vm", "Failed to create a memory slice with error `{}`", e.to_string());
                return Err(e.into());
            }
        };

        // TODO: make `max_len` configurable.
        let max_len = 2048 * 1024;
        let result = match memory_slice.read(&memory_view, max_len) {
            Ok(result) => result,
            Err(e) => {
                error!(target: "ramd::vm", "Failed to read memory with error `{}`", e.to_string());
                return Err(e.into());
            }
        };
        let result: String = match String::from_utf8(result) {
            Ok(result) => result,
            Err(e) => {
                error!(target: "ramd::vm", "Failed to convert the result to string with error `{}`", e.to_string());
                return Err(eyre::eyre!(e.to_string()));
            }
        };

        Ok(result)
    }

    pub fn write_memory(
        &mut self,
        memory_slice_ptr: MemorySlicePtr,
        data: &[u8],
    ) -> eyre::Result<()> {
        let memory = match self.instance.exports.get_memory("memory") {
            Ok(memory) => memory,
            Err(e) => {
                error!(target: "ramd::vm", "Failed to get the memory `memory` with error `{}`", e.to_string());
                return Err(e.into());
            }
        };
        let memory_view = memory.view(&self.store);
        let memory_slice = match MemorySlice::new(&memory_view, memory_slice_ptr) {
            Ok(memory_slice) => memory_slice,
            Err(e) => {
                error!(target: "ramd::vm", "Failed to create a memory slice with error `{}`", e.to_string());
                return Err(e.into());
            }
        };

        if let Err(e) = memory_slice.write(&memory_view, data) {
            error!(target: "ramd::vm", "Failed to write memory with error `{}`", e.to_string());
            return Err(e.into());
        }

        Ok(())
    }
}
