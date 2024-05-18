pub trait LiveObjectHandler: Send + Sync {
    fn create_live_object(&self, wasm_bytes: Vec<u8>);

    fn execute_live_object(&self, live_object_id: String, method: String, args: Vec<u8>);
}
