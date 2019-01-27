use std::fmt;
use std::fmt::Write;

use crate::expr_tree::{ProgramStatement};

pub fn emit_c(_tree: Vec<ProgramStatement>) -> Result<String, fmt::Error> {
    let mut code = String::new(); 

    write!(&mut code, "int main(){{return 0;}}")?;

    Ok(code)
}
