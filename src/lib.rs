use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

mod error;
use error::RekiResult;

#[wasm_bindgen]
pub struct Reki {
    bin: elf::File,
}

#[wasm_bindgen]
impl Reki {
    pub fn new(co_bytes_view: Uint8Array) -> Result<Reki, JsValue> {
        reki_new(co_bytes_view.to_vec()).map_err(Into::into)
    }
}

fn reki_new(co_bytes: Vec<u8>) -> RekiResult<Reki> {
    use std::io::Cursor;
    let bin = elf::File::open_stream(&mut Cursor::new(co_bytes))?;

    Ok(Reki { bin })
}
