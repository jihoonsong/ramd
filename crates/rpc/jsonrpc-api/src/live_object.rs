use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use ramd_jsonrpc_types::live_object::{CreateLiveObject, ExecuteLiveObject};

#[rpc(server, client, namespace = "live_object")]
pub trait LiveObjectApi {
    #[method(name = "create")]
    async fn create_live_object(&self, request: CreateLiveObject) -> RpcResult<String>;

    #[method(name = "execute")]
    async fn execute_live_object(&self, request: ExecuteLiveObject) -> RpcResult<String>;
}
