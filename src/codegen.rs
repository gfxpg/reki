mod transforms;

use std::process::{Command, Stdio};

use crate::expr_tree::{ProgramStatement};

pub fn emit_c(tree: Vec<ProgramStatement>) -> std::io::Result<String> {
    use std::io::{Error, ErrorKind};

    self::transforms::tree(tree)
        .map_err(|fmt_e| Error::new(ErrorKind::Other, format!("{}", fmt_e)))
        .and_then(reformat_c)
}

fn reformat_c(code: String) -> std::io::Result<String> {
    use std::io::Write;

    let mut clang_format = Command::new("clang-format-7")
        .arg("--style=Chromium")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    clang_format.stdin.take().unwrap().write_all(code.as_bytes())?;

    Ok(String::from_utf8(clang_format.wait_with_output()?.stdout).unwrap())
}
