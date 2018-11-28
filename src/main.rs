#![feature(try_from)]
#![feature(slice_patterns)]

extern crate llvm_sys;
extern crate libc;
extern crate elf;
extern crate byteorder;

mod kernel_meta;
mod assembly;
mod eval;
mod expr;
mod exec_state;

use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: reki <compiled.hsaco>");
        return;
    }
    let hsaco = elf::File::open_path(&PathBuf::from(&args[1])).unwrap();
    let disassembly = assembly::disassemble(hsaco).unwrap();

    println!("{:#?}", disassembly.0);
    println!("Args: {:#?}", disassembly.1);

    let mut state = exec_state::ExecutionState::from(disassembly);

    println!("Expression tree: {:#?}", eval::eval_pgm(&mut state));
    println!("Bindings: {:#?}", state.bindings);
}
