mod error;
use error::RekiResult;

pub struct CodeObject {
    bin: elf::File,
}

impl CodeObject {
    pub fn new(co_bytes: Vec<u8>) -> RekiResult<CodeObject> {
        use std::io::Cursor;
        let bin = elf::File::open_stream(&mut Cursor::new(co_bytes))?;

        Ok(CodeObject { bin })
    }
}
