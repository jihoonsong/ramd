use ramd_vm::Context;

use ramd_db::storage::Storage;

pub struct Runner<S> {
    context: Context<S>,
    method: String,
    args: Vec<u8>,
}

impl<S> Runner<S>
where
    S: Storage<Vec<u8>, Vec<u8>>,
{
    pub fn new(context: Context<S>, method: String, args: Vec<u8>) -> Self {
        Runner {
            context,
            method,
            args,
        }
    }

    pub fn run(&mut self) -> eyre::Result<String> {
        // Allocate `MemorySlice`.
        let params_ptr = self.context.allocate_memory(self.args.len() as u32)?;

        // Write parameters to `MemorySlice`.
        self.context
            .write_memory(params_ptr, self.args.as_slice())?;

        // Call function.
        let result_ptr = self
            .context
            .call_function(&self.method, vec![params_ptr.into()])?;

        // Read return value from `MemorySlice`.
        let result = self.context.read_memory(result_ptr)?;

        // Deallocate `MemorySlice`.
        self.context.deallocate_memory(result_ptr)?;

        Ok(result)
    }
}
