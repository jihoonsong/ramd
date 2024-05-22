use std::sync::Arc;

use crate::config::NodeConfig;
use crate::handlers::LiveObjectHandler;
use ramd_db::storage::Storage;
use ramd_processor::{Action, CreateLiveObjectAction, ExecuteLiveObjectAction, Message, Processor};
use tracing::info;

pub struct Node<S>
where
    S: Storage<Vec<u8>, Vec<u8>> + 'static,
{
    processor: Processor<S>,
}

impl<S> Node<S>
where
    S: Storage<Vec<u8>, Vec<u8>>,
{
    pub fn new(_config: &NodeConfig, storage: Arc<S>) -> eyre::Result<Self> {
        Ok(Node {
            processor: Processor::new(storage.clone()),
        })
    }
}

impl<S> LiveObjectHandler for Node<S>
where
    S: Storage<Vec<u8>, Vec<u8>>,
{
    fn create_live_object(&self, wasm_bytes: Vec<u8>) {
        let messages = vec![Message {
            action: Action::CreateLiveObject(CreateLiveObjectAction { wasm_bytes }),
        }];

        // TODO: log message ID.
        info!(target: "ramd::node", "New message with create action");

        self.processor.process_messages(&messages);
    }

    fn execute_live_object(&self, live_object_id: String, method: String, args: Vec<u8>) {
        let messages = vec![Message {
            action: Action::ExecuteLiveObject(ExecuteLiveObjectAction {
                live_object_id,
                method,
                args,
            }),
        }];

        // TODO: log message ID.
        info!(target: "ramd::node", "New message with execute action");

        self.processor.process_messages(&messages);
    }
}
