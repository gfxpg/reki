use crate::expr_tree::{ProgramStatement};

type CodegenResult = Result<String, std::fmt::Error>;

pub fn tree(tree: Vec<ProgramStatement>) -> CodegenResult {
    use std::fmt::Write;

    let mut code = String::new();

    write!(&mut code, "int main(){{return 0;}}")?;

    Ok(code)
}
