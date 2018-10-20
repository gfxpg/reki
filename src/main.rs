#![feature(try_from)]

extern crate llvm_sys;
extern crate libc;
extern crate elf;
extern crate byteorder;

mod kernel_code_object;
mod disasm;
mod reg_state;
mod eval;

use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: reki <compiled.hsaco>");
        return;
    }
    let hsaco = elf::File::open_path(&PathBuf::from(&args[1])).unwrap();
    let (kernel_code, instrs) = disasm::disassemble(hsaco).unwrap();

    let sgprs = reg_state::initial_sgprs(&kernel_code);
    let vgprs = reg_state::initial_vgprs(&kernel_code);

    println!("{:#?}", kernel_code);

    eval::eval_loads(eval::ExecutionState { instrs, sgprs, vgprs });
}
