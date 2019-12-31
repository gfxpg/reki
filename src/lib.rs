use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Reki {
    a: i32,
    b: i32
}

#[wasm_bindgen]
impl Reki {
    pub fn new(a: i32, b: i32) -> Self {
        Reki { a, b }
    }

    pub fn sum3(&self, c: i32) -> i32 {
        self.a + self.b + c
    }
}
