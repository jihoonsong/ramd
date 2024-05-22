use crate::{Context, MemorySlicePtr};
use ramd_db::storage::Storage;
use wasmer::{imports, AsStoreMut, Function, FunctionEnv, FunctionEnvMut, Imports};

/// The import object that has import functions.
pub struct ImportObject(pub Imports);

impl ImportObject {
    /// Create a new `ImportObject`.
    pub fn new<S>(mut store: &mut impl AsStoreMut, function_env: &FunctionEnv<Context<S>>) -> Self
    where
        S: Storage<Vec<u8>, Vec<u8>> + 'static,
    {
        let import_object = imports! {
            "env" => {
                "storage_read" => Function::new_typed_with_env(&mut store, function_env, Self::storage_read),
                "storage_write" => Function::new_typed_with_env(&mut store, function_env, Self::storage_write),
            }
        };

        ImportObject(import_object)
    }

    /// Read data from the storage.
    fn storage_read<S>(
        mut env: FunctionEnvMut<Context<S>>,
        key_ptr: MemorySlicePtr,
    ) -> eyre::Result<MemorySlicePtr, wasmer::RuntimeError>
    where
        S: Storage<Vec<u8>, Vec<u8>> + 'static,
    {
        let (context, mut store) = env.data_and_store_mut();

        let key = context.read_memory(&store, key_ptr)?;

        let value = context
            .storage
            .get(key)
            .map_err(|err| wasmer::RuntimeError::new(err.to_string()))?;

        let value_ptr = context
            .allocate_memory(&mut store, value.len())
            .map_err(|err| wasmer::RuntimeError::new(err.to_string()))?;

        context.write_memory(&store, value_ptr, &value)?;

        Ok(value_ptr)
    }

    /// Write data to the storage.
    fn storage_write<S>(
        mut env: FunctionEnvMut<Context<S>>,
        key_ptr: MemorySlicePtr,
        value_ptr: MemorySlicePtr,
    ) -> eyre::Result<(), wasmer::RuntimeError>
    where
        S: Storage<Vec<u8>, Vec<u8>> + 'static,
    {
        let (context, store) = env.data_and_store_mut();

        let key = context.read_memory(&store, key_ptr)?;
        let value = context.read_memory(&store, value_ptr)?;

        context
            .storage
            .set(key, value)
            .map_err(|err| wasmer::RuntimeError::new(err.to_string()))?;

        Ok(())
    }
}
