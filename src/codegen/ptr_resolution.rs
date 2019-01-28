use crate::asm::kernel_args::KernelArgs;
use crate::expr_tree::{ProgramStatement, BoundExpr};
use crate::data_flow::types::{BuiltIn, DataKind};

pub fn builtin_ptr(args: &KernelArgs, ptr: BuiltIn, offset: i32, kind: DataKind) -> String {
    use BuiltIn::*;

    match ptr {
        PtrKernarg { .. } =>
            match args.find_idx_and_dword(offset as u32) {
                Some((arg_idx, dword)) => {
                    let arg = match args[arg_idx].name.as_ref() {
                        "HiddenGlobalOffsetX" => "get_global_offset(0)",
                        "HiddenGlobalOffsetY" => "get_global_offset(1)",
                        user_arg => user_arg
                    };
                    format!("{}.dword[{}]", arg, dword)
                },
                None => panic!("Unable to resolve kernel argument at offset {}; args struct: {:#?}", offset, args)
            },
        pointer => {
            eprintln!("Unable to resolve pointer derefence of {:?} at offset {}", pointer, offset);
            "/* expr: Placeholder */".to_string()
        }
    }
}
