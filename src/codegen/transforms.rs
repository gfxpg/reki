use std::fmt::Write;
use itertools::Itertools;

use crate::asm::kernel_args::{KernelArgs, KernelArg};
use crate::expr_tree::{ProgramStatement};

type CodegenResult = Result<String, std::fmt::Error>;

pub fn tree(tree: Vec<ProgramStatement>, args: &KernelArgs) -> CodegenResult {
    let mut code = format!("__kernel void decompiled({}) {{",
        kernel_args(args));

    write!(&mut code, "}}")?;

    Ok(code)
}

fn kernel_args(args: &KernelArgs) -> String {
    args.iter()
        .filter_map(|KernelArg { name, typename, is_const, .. }| {
            if let Some(cl_type) = typename { 
                let modifier = if *is_const { "const " } else { "" };
                Some(format!("{}{} {}", modifier, cl_type, name))
            }
            else {
                None
            }
        })
        .join(",")
}
