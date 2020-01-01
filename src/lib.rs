use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Reki {
    code_object: reki::CodeObject,
}

#[wasm_bindgen]
impl Reki {
    pub fn new(co_bytes: Vec<u8>) -> Result<Reki, JsValue> {
        let code_object =
            reki::CodeObject::new(co_bytes).map_err(|e| js_sys::Error::new(e.msg()))?;
        Ok(Reki { code_object })
    }
}
