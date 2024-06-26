use std::sync::Arc;

use async_trait::async_trait;
use jsonrpsee::core::RpcResult;
use ramd_jsonrpc_api::server::LiveObjectApiServer;
use ramd_jsonrpc_types::live_object::{CreateLiveObject, ExecuteLiveObject};
use ramd_node::LiveObjectHandler;
use tracing::info;

pub struct LiveObjectApi<H>
where
    H: LiveObjectHandler,
{
    node: Arc<H>,
}

impl<H> LiveObjectApi<H>
where
    H: LiveObjectHandler,
{
    pub fn new(node: Arc<H>) -> Self {
        Self { node: node.clone() }
    }
}

#[async_trait]
impl<H> LiveObjectApiServer for LiveObjectApi<H>
where
    H: LiveObjectHandler + 'static,
{
    async fn create_live_object(&self, request: CreateLiveObject) -> RpcResult<String> {
        info!(target: "ramd::jsonrpc", "Request to create a live object");

        Ok(self.node.create_live_object(request.decode_wasm_bytes()?))
    }

    async fn execute_live_object(&self, request: ExecuteLiveObject) -> RpcResult<String> {
        info!(target: "ramd::jsonrpc", "Request to execute a live object");

        Ok(self.node.execute_live_object(
            request.live_object_id,
            request.method,
            request.args.as_bytes().to_vec(),
        ))
    }
}
