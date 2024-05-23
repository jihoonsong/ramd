use std::sync::Arc;

use crate::message::Message;
use ramd_cache::{Cache, InMemoryCache};
use ramd_db::storage::Storage;
use tracing::error;

pub struct Processor<S>
where
    S: Storage<Vec<u8>, Vec<u8>> + 'static,
{
    storage: Arc<S>,
}

impl<S> Processor<S>
where
    S: Storage<Vec<u8>, Vec<u8>> + 'static,
{
    pub fn new(storage: Arc<S>) -> Self {
        Self { storage }
    }

    pub fn process_messages(&self, messages: &[Message]) {
        let cache = Arc::new(InMemoryCache::new(self.storage.clone()));

        // TODO: add to messsage pool and then process messages.

        for message in messages {
            if let Err(err) = message.process(cache.clone()) {
                // TODO: log message ID.
                error!(target: "ramd::processor", "Failed to process a message with error `{}`", err.to_string());
                return;
            }
        }

        if let Err(err) = cache.commit() {
            error!(target: "ramd::processor", "Failed to commit cache with error `{}`", err.to_string());
            return;
        }
    }
}
