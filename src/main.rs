#![feature(try_from)]

extern crate llvm_sys;
extern crate libc;
extern crate elf;
extern crate byteorder;

mod kernel_code_object;
mod disasm;

use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: reki <compiled.hsaco>");
        return;
    }
    let hsaco = elf::File::open_path(&PathBuf::from(&args[1])).unwrap();
    let (kernel_code, instrs) = disasm::disassemble(hsaco).unwrap();

    println!("{:#?}", kernel_code);
    println!("{:#?}", instrs);
}
