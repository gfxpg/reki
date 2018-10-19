#![feature(try_from)]

extern crate llvm_sys;
extern crate libc;
extern crate elf;
extern crate byteorder;

mod kernel_code_object;
mod disasm;
mod reg_state;

use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: reki <compiled.hsaco>");
        return;
    }
    let hsaco = elf::File::open_path(&PathBuf::from(&args[1])).unwrap();
    let (kernel_code, instrs) = disasm::disassemble(hsaco).unwrap();

    let initial_sgprs = reg_state::initial_sgprs(&kernel_code);
    let initial_vgprs = reg_state::initial_vgprs(&kernel_code);

    println!("{:#?}", kernel_code);
    println!("{:#?}", instrs);
    println!("{:#?}", initial_sgprs);
    println!("{:#?}", initial_vgprs);
}
