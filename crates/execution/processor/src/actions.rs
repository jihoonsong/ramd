use std::sync::Arc;

use ramd_db::storage::Storage;
use ramd_vm::{LiveObjectInfo, Runtime};
use tracing::{error, info};

pub enum Action {
    CreateLiveObject(CreateLiveObjectAction),
    ExecuteLiveObject(ExecuteLiveObjectAction),
}

impl Action {
    pub(crate) fn perform<S>(&self, cache: Arc<S>) -> eyre::Result<()>
    where
        S: Storage<Vec<u8>, Vec<u8>> + 'static,
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
        let live_object_info = LiveObjectInfo::new(self.wasm_bytes.clone());
        info!(target: "ramd::processor", "Successfully created live object with id `{}`", live_object_info.id);

        if let Err(e) = cache.set(
            live_object_info.id.as_bytes().to_vec(),
            live_object_info.try_into()?,
        ) {
            error!(target: "ramd::processor", "Failed to store the created live object with error `{}`", e.to_string());
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
        S: Storage<Vec<u8>, Vec<u8>> + 'static,
    {
        let live_object_info_bytes = match cache.get(self.live_object_id.as_bytes().to_vec()) {
            Ok(bytes) => bytes,
            Err(e) => {
                error!(target: "ramd::processor", "Failed to get wasm bytes from cache with error `{}`", e.to_string());
                return Err(e);
            }
        };
        let live_object_info = LiveObjectInfo::try_from(live_object_info_bytes.clone())?;
        info!(target: "ramd::processor", "Successfully read live object with id `{}`", live_object_info.id);

        let mut runtime = Runtime::new(cache, live_object_info)?;
        let result = runtime.run(self.method.clone(), self.args.clone())?;
        info!(target: "ramd::processor", "Successfully called method `{}` to get result `{}`", self.method, result);

        info!(target: "ramd::processor", "Successfully performed execute action");
        Ok(())
    }
}
