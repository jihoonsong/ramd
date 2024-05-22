use std::sync::Arc;

use crate::{Context, ImportObject, MemorySlice, MemorySlicePtr, MAX_WASM_MEMORY_SIZE};
use ramd_db::storage::Storage;
use tracing::info;
use wasmer::{FunctionEnv, Instance, Module, Store, Value};

/// The runtime that creates and runs the WASM instance.
pub struct Runtime {
    store: Store,
    instance: Instance,
}

impl Runtime {
    /// Create a new `Runtime`.
    pub fn new<S>(storage: Arc<S>, wasm_bytes: Vec<u8>) -> eyre::Result<Self>
    where
        S: Storage<Vec<u8>, Vec<u8>> + 'static,
    {
        // Create a Store.
        let mut store = Store::default();

        // Compile the WASM module.
        let module = Module::new(&store, wasm_bytes)?;

        // Create a function environment.
        let function_env = FunctionEnv::new(&mut store, Context::new(storage));

        // Create an import object.
        let import_object = ImportObject::new(&mut store, &function_env);

        // Instantiate the WASM instance.
        let instance = Instance::new(&mut store, &module, &import_object.0)?;

        // Set the Context of the WASM instance. The WASM instance will use this Context to interact with the host.
        let mut function_env = function_env.into_mut(&mut store);
        let context = function_env.data_mut();

        let memory = instance.exports.get_memory("memory").cloned()?;
        let allocate = instance.exports.get_function("allocate").cloned()?;

        context.memory = Some(memory);
        context.allocate = Some(allocate);

        info!(target: "ramd::vm", "Runtime is created");

        Ok(Self { store, instance })
    }

    /// Run the specified function with arguments on the WASM instance.
    pub fn run(&mut self, method: String, args: Vec<u8>) -> eyre::Result<String> {
        // Allocate `MemorySlice`.
        let args_ptr = self
            .call_function("allocate", &[Value::from(args.len() as u32)])?
            .ok_or(eyre::eyre!("Failed to allocate memory"))?;

        // Write parameters to `MemorySlice`.
        self.write_memory(args_ptr, args.as_slice())?;

        // Call function.
        let result_ptr = self
            .call_function(&method, &[args_ptr.into()])?
            .ok_or(eyre::eyre!("Failed to call the function `{}`", method))?;

        // Read return value from `MemorySlice`.
        let result = self.read_memory(result_ptr)?;
        let result = String::from_utf8(result)?;

        // Deallocate `MemorySlice`.
        self.call_function("deallocate", &[result_ptr.into()])?;

        Ok(result)
    }

    /// Call the specified function with arguments on the WASM instance.
    fn call_function(
        &mut self,
        method: &str,
        args_ptr: &[Value],
    ) -> eyre::Result<Option<MemorySlicePtr>> {
        let func = self.instance.exports.get_function(method)?;

        let result_ptr = func.call(&mut self.store, args_ptr)?;

        if result_ptr.is_empty() {
            return Ok(None);
        }

        let result_ptr: MemorySlicePtr = result_ptr[0]
            .clone()
            .try_into()
            .map_err(|err: &str| eyre::eyre!(err))?;

        Ok(Some(result_ptr))
    }

    /// Read data from the WASM (guest) memory.
    fn read_memory(&mut self, memory_slice_ptr: MemorySlicePtr) -> eyre::Result<Vec<u8>> {
        let memory = self.instance.exports.get_memory("memory")?;
        let memory_view = memory.view(&self.store);
        let memory_slice = MemorySlice::new(&memory_view, memory_slice_ptr)?;

        let data = memory_slice.read(&memory_view, MAX_WASM_MEMORY_SIZE)?;

        Ok(data)
    }

    /// Write data to the WASM (guest) memory.
    fn write_memory(&mut self, memory_slice_ptr: MemorySlicePtr, data: &[u8]) -> eyre::Result<()> {
        let memory = self.instance.exports.get_memory("memory")?;
        let memory_view = memory.view(&self.store);
        let memory_slice = MemorySlice::new(&memory_view, memory_slice_ptr)?;

        memory_slice.write(&memory_view, data)?;

        Ok(())
    }
}
