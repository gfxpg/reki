use std::error::Error;
use std::fmt;
use wasm_bindgen::JsValue;

pub type RekiResult<T> = Result<T, RekiError>;

#[derive(Debug, Clone)]
pub struct RekiError(js_sys::Error);

impl fmt::Display for RekiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&String::from(self.0.message()))
    }
}

impl From< RekiError> for JsValue {
    fn from(e: RekiError) -> Self {
        e.0.into()
    }
}

impl Error for RekiError {}

impl From<&str> for RekiError {
    fn from(e: &str) -> Self {
        RekiError(js_sys::Error::new(e))
    }
}

impl From<std::io::Error> for RekiError {
    fn from(e: std::io::Error) -> Self {
        format!("{}", e).as_str().into()
    }
}

impl From<elf::ParseError> for RekiError {
    fn from(e: elf::ParseError) -> Self {
        match e {
            elf::ParseError::IoError(io) => RekiError::from(io),
            _ => RekiError::from("Unable to read the file as an ELF"),
        }
    }
}
