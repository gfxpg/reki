use std::error::Error;
use std::fmt;

pub type RekiResult<T> = Result<T, RekiError>;

#[derive(Debug, Clone)]
pub struct RekiError(String);

impl fmt::Display for RekiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for RekiError {}

impl<'a> RekiError {
    pub fn msg(&'a self) -> &'a str {
        self.0.as_str()
    }
}

impl From<&str> for RekiError {
    fn from(e: &str) -> Self {
        RekiError(e.into())
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
