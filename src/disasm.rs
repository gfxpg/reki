use std::io;
use std::ffi::CString;
use std::convert::TryFrom;
use llvm_sys::disassembler::{LLVMCreateDisasmCPU, LLVMDisasmInstruction, LLVMDisasmDispose};

use kernel_meta::{extract_kernel_args, KernelCode, KernelArg};

pub fn disassemble(bin: elf::File) -> io::Result<(KernelCode, Vec<KernelArg>, Vec<String>)> {
    let mut pgm_data = bin
        .get_section(".text")
        .ok_or(io::Error::new(io::ErrorKind::InvalidData, "missing .text section"))?
        .data.to_owned();

    if pgm_data.len() < 256 {
        return Err(io::Error::new(io::ErrorKind::InvalidData,
            "program text must be at least 256 bytes long (the size of AMDKernelCodeT struct)"));
    }

    let pgm_note = bin
        .get_section(".note")
        .ok_or(io::Error::new(io::ErrorKind::InvalidData, "missing .note section with OpenCL metadata"))?;

    let args = extract_kernel_args(&pgm_note.data);

    let (amd_kernel_code_raw, instructions_raw) = pgm_data.split_at_mut(256);

    let code_obj = KernelCode::try_from(amd_kernel_code_raw as &[u8])?;
    let instructions = disassemble_instructions(instructions_raw)?;

    Ok((code_obj, args, instructions))
}

fn disassemble_instructions(instructions_raw: &mut [u8]) -> io::Result<Vec<String>> {
    unsafe {
        llvm_sys::target::LLVM_InitializeAllTargetInfos();
        llvm_sys::target::LLVM_InitializeAllTargetMCs();
        llvm_sys::target::LLVM_InitializeAllDisassemblers();
    }

    let asmctx = unsafe {
        LLVMCreateDisasmCPU(
            CString::new("amdgcn--amdhsa")?.as_ptr(),
            CString::new("gfx900")?.as_ptr(),
            std::ptr::null_mut(), 0, None, None)
    };

    let mut instructions: Vec<String> = Vec::new();

    let mut pos = 0;
    let mut instr_buf = [0u8; 256];
    while pos < instructions_raw.len() {
        unsafe {
            pos += LLVMDisasmInstruction(asmctx,
                instructions_raw.as_mut_ptr().offset(pos as isize),
                (instructions_raw.len() - pos) as u64, 
                0,
                instr_buf.as_mut_ptr() as *mut i8,
                256);
        }
        let tab_skip = 1;
        let string_end = instr_buf.iter().position(|&r| r == 0).unwrap();
        instructions.push(
            std::str::from_utf8(&instr_buf[tab_skip..string_end]).unwrap().to_string());
    }

    unsafe {
        LLVMDisasmDispose(asmctx);
    }

    Ok(instructions)
}
