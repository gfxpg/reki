#![feature(try_from)]
#![feature(slice_patterns)]

extern crate llvm_sys;
extern crate libc;
extern crate elf;
extern crate byteorder;
extern crate itertools;

mod kernel_meta;
mod assembly;
mod eval;
mod expr;
mod exec_state;
mod control_flow;

use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: reki <compiled.hsaco>");
        return;
    }
    let hsaco = elf::File::open_path(&PathBuf::from(&args[1])).unwrap();
    let (kcode, kernel_args, instructions) = assembly::disassemble(hsaco).unwrap();
    let cf_map = control_flow::build_map(&instructions);

    println!("{:#?}", kcode);
    println!("Args: {:#?}", kernel_args);
    println!("Control flow map: {:?}", cf_map);

    let mut state = exec_state::ExecutionState::from(kcode);

    println!("instrs: {:?}", instructions.len());
    eval::eval_pgm(&mut state, instructions.as_slice(), &cf_map);
}
