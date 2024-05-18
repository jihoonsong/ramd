use std::sync::Arc;

use ramd_db::storage::Storage;
use ramd_vm::Context;
use ramd_vm_runtime::Runner;
use tracing::{error, info};

pub enum Action {
    CreateLiveObject(CreateLiveObjectAction),
    ExecuteLiveObject(ExecuteLiveObjectAction),
}

impl Action {
    pub(crate) fn perform<S>(&self, cache: Arc<S>) -> eyre::Result<()>
    where
        S: Storage<Vec<u8>, Vec<u8>>,
    {
        match self {
            Action::CreateLiveObject(action) => action.perform(cache),
            Action::ExecuteLiveObject(action) => action.perform(cache),
        }
    }
}

pub struct CreateLiveObjectAction {
    pub wasm_bytes: Vec<u8>,
}

impl CreateLiveObjectAction {
    fn perform<S>(&self, cache: Arc<S>) -> eyre::Result<()>
    where
        S: Storage<Vec<u8>, Vec<u8>>,
    {
        // TODO: use some cryptographic hash as a key.
        if let Err(e) = cache.set("0".as_bytes().to_vec(), self.wasm_bytes.clone()) {
            error!(target: "ramd::processor", "Failed to set wasm bytes to cache with error `{}`", e.to_string());
            return Err(e);
        }

        info!(target: "ramd::processor", "Successfully performed create action");
        Ok(())
    }
}

pub struct ExecuteLiveObjectAction {
    pub live_object_id: String,
    pub method: String,
    pub args: Vec<u8>,
}

impl ExecuteLiveObjectAction {
    fn perform<S>(&self, cache: Arc<S>) -> eyre::Result<()>
    where
        S: Storage<Vec<u8>, Vec<u8>>,
    {
        let wasm_bytes = match cache.get(self.live_object_id.as_bytes().to_vec()) {
            Ok(wasm_bytes) => wasm_bytes,
            Err(e) => {
                error!(target: "ramd::processor", "Failed to get wasm bytes from cache with error `{}`", e.to_string());
                return Err(e);
            }
        };

        let context = Context::new(cache, wasm_bytes)?;
        let result = Runner::new(context, self.method.clone(), self.args.clone()).run()?;
        info!(target: "ramd::processor", "Called method `{}` to get result `{}`", self.method, result);

        info!(target: "ramd::processor", "Successfully performed execute action");
        Ok(())
    }
}
