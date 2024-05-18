use wasmer::{imports, Imports};

pub struct ImportObject(pub Imports);

impl ImportObject {
    pub fn new() -> Self {
        let import_object = imports! {};

        // TODO: add import functions such as storage_read and storage_write to `import_object`.

        ImportObject(import_object)
    }
}
