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
                "storage_has" => Function::new_typed_with_env(&mut store, function_env, Self::storage_has),
                "storage_read" => Function::new_typed_with_env(&mut store, function_env, Self::storage_read),
                "storage_write" => Function::new_typed_with_env(&mut store, function_env, Self::storage_write),
                "storage_delete" => Function::new_typed_with_env(&mut store, function_env, Self::storage_delete),
            }
        };

        ImportObject(import_object)
    }

    /// Check if data corresponding to the key exists in the storage.
    fn storage_has<S>(
        mut env: FunctionEnvMut<Context<S>>,
        key_ptr: MemorySlicePtr,
    ) -> eyre::Result<u32, wasmer::RuntimeError>
    where
        S: Storage<Vec<u8>, Vec<u8>> + 'static,
    {
        let (context, store) = env.data_and_store_mut();

        let key = context.read_memory(&store, key_ptr)?;

        let has = context
            .storage
            .has(context.prefix_key(key))
            .map_err(|err| wasmer::RuntimeError::new(err.to_string()))?;

        Ok(has as u32)
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
            .get(context.prefix_key(key))
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
            .set(context.prefix_key(key), value)
            .map_err(|err| wasmer::RuntimeError::new(err.to_string()))?;

        Ok(())
    }

    /// Delete data from the storage.
    fn storage_delete<S>(
        mut env: FunctionEnvMut<Context<S>>,
        key_ptr: MemorySlicePtr,
    ) -> eyre::Result<(), wasmer::RuntimeError>
    where
        S: Storage<Vec<u8>, Vec<u8>> + 'static,
    {
        let (context, store) = env.data_and_store_mut();

        let key = context.read_memory(&store, key_ptr)?;

        context
            .storage
            .delete(context.prefix_key(key))
            .map_err(|err| wasmer::RuntimeError::new(err.to_string()))?;

        Ok(())
    }
}
