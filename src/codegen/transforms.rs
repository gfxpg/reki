use std::fmt::Write;
use itertools::Itertools;

use crate::asm::kernel_args::{KernelArgs, KernelArg};
use crate::expr_tree::{ProgramStatement, BoundExpr};
use crate::data_flow::types::BuiltIn;
use crate::codegen::ptr_resolution;

type CodegenResult = Result<String, std::fmt::Error>;

const VAR_UNION: &'static str = r#"
typedef union {
    int dword[4];
    long qword[2];
} univar_t;
"#;

pub fn tree(tree: Vec<ProgramStatement>, args: &KernelArgs) -> CodegenResult {
    let mut code = String::new();

    for stmt in tree {
        use ProgramStatement::*;
        use BoundExpr::*;

        match stmt {
            Assignment { var_idx, expr } =>
                writeln!(&mut code, "v{}.? = {};", var_idx, bound_expr(&expr, args))?,
            Declaration { var_idx } =>
                writeln!(&mut code, "univar_t v{};", var_idx)?,
            Label { label_idx } =>
                writeln!(&mut code, "label{}:", label_idx)?,
            _ =>
                writeln!(&mut code, "/* Unhandled statement {:?} */", stmt)?
        }
    }

    Ok(format!("{} __kernel void decompiled({}) {{ {} }}", VAR_UNION, kernel_args(args), code))
}

fn bound_expr(expr: &BoundExpr, args: &KernelArgs) -> String {
    use BoundExpr::*;

    match expr {
        Deref { ptr: box InitState(builtin), offset, kind } =>
            ptr_resolution::builtin_ptr(args, *builtin, *offset, *kind),
        InitState(builtin) =>
            init_state(builtin),
        Mul(lhs, rhs) =>
            format!("{} * {}", bound_expr(lhs.as_ref(), args), bound_expr(rhs.as_ref(), args)),
        Add(lhs, rhs) =>
            format!("({} + {})", bound_expr(lhs, args), bound_expr(rhs, args)),
        Shl(lhs, rhs) =>
            format!("{} << {}", bound_expr(lhs, args), bound_expr(rhs, args)),
        Cast(expr, kind) =>
            format!("({:?}) {}", kind, bound_expr(expr, args)),
        U32(lit) =>
            format!("{}", lit),
        _ =>
            format!("(/* expr {:?} */)", expr)
    }
}

fn init_state(builtin: &BuiltIn) -> String {
    use BuiltIn::*;

    match builtin {
        WorkgroupIdX =>
            "get_group_id(0)".to_string(),
        WorkgroupIdY =>
            "get_group_id(1)".to_string(),
        WorkitemIdX =>
            "get_local_id(0)".to_string(),
        WorkitemIdY =>
            "get_local_id(1)".to_string(),
        _ =>
            format!("(/* builtin {:?} */)", builtin)
    }
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
