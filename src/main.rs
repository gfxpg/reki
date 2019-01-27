#![feature(try_from)]
#![feature(slice_patterns)]

mod asm;
mod control_flow;
mod data_flow;
mod expr_tree;
mod codegen;

use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: reki <compiled.hsaco>");
        return;
    }
    let hsaco = elf::File::open_path(&PathBuf::from(&args[1])).unwrap();
    let (kcode, kernel_args, instructions) = asm::disassemble(hsaco).unwrap();

    let cf_map = control_flow::build_map(&instructions);

    println!("{:#?}", kcode);
    println!("Args: {:#?}", kernel_args);
    println!("Control flow map: {:?}", cf_map);

    let mut state = data_flow::exec_state::ExecState::from(kcode);
    let program = data_flow::analyze(&mut state, instructions.as_slice(), &cf_map);

    println!("State: {:?}", state);

    let tree = expr_tree::build(&kernel_args, state, program);

    let code = codegen::emit_c(tree, &kernel_args).unwrap();
    println!("Code:\n{}", code);
}
