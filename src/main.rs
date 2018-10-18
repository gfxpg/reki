#![feature(try_from)]

extern crate llvm_sys;
extern crate libc;
extern crate elf;
extern crate byteorder;

mod kernel_code_object;

use kernel_code_object::KernelCodeObject;
use std::convert::TryFrom;

use std::path::PathBuf;
use std::ffi::CString;
use llvm_sys::disassembler::{LLVMCreateDisasmCPU, LLVMDisasmInstruction, LLVMDisasmDispose};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: reki <compiled.hsaco>");
        return;
    }
    let hsaco = elf::File::open_path(&PathBuf::from(&args[1])).unwrap();
    let mut pgm_data = hsaco.get_section(".text").unwrap().data.to_owned();
    let (amd_kernel_code_raw, instructions) = pgm_data.split_at_mut(256);
    let kernel_code_obj = KernelCodeObject::try_from(amd_kernel_code_raw as &[u8]).unwrap();
    println!("{:#?}", kernel_code_obj);

    unsafe {
        llvm_sys::target::LLVM_InitializeAllTargetInfos();
        llvm_sys::target::LLVM_InitializeAllTargetMCs();
        llvm_sys::target::LLVM_InitializeAllDisassemblers();
    }

    let asmctx = unsafe {
        LLVMCreateDisasmCPU(
            CString::new("amdgcn--amdhsa").unwrap().as_ptr(),
            CString::new("gfx900").unwrap().as_ptr(),
            std::ptr::null_mut(), 0, None, None)
    };

    let mut pos = 0;
    let mut instr_buf = [0u8; 256];
    while pos < instructions.len() {
        unsafe {
            pos += LLVMDisasmInstruction(asmctx,
                instructions.as_mut_ptr().offset(pos as isize), (instructions.len() - pos) as u64, 0, instr_buf.as_mut_ptr() as *mut i8, 256);
        }
        let tab_skip = 1;
        let string_end = instr_buf.iter().position(|&r| r == 0).unwrap();
        println!("{} {:?}", pos, std::str::from_utf8(&instr_buf[tab_skip..string_end]).unwrap());
    }

    unsafe {
        LLVMDisasmDispose(asmctx);
    }
}
