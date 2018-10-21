use std::io;
use std::ffi::CString;
use std::convert::TryFrom;
use llvm_sys::disassembler::{LLVMCreateDisasmCPU, LLVMDisasmInstruction, LLVMDisasmDispose};

use kernel_meta::{extract_kernel_args, KernelCode, KernelArg};

pub type Instruction = (String, Vec<Operand>);

#[derive(Debug)]
pub enum Operand {
    SReg(usize),
    VReg(usize),
    SRegs(usize, usize),
    VRegs(usize, usize),
    Lit(u32),
    VCC,
    Keyseq(String)
}

pub type Disassembly = (KernelCode, Vec<KernelArg>, Vec<Instruction>);

pub fn disassemble(bin: elf::File) -> io::Result<Disassembly> {
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

fn disassemble_instructions(instructions_raw: &mut [u8]) -> io::Result<Vec<Instruction>> {
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

    let mut instructions: Vec<Instruction> = Vec::new();

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
        let instr_raw = std::str::from_utf8(&instr_buf[tab_skip..string_end]).unwrap();

        instructions.push(parse_instruction(instr_raw));
    }

    unsafe {
        LLVMDisasmDispose(asmctx);
    }

    Ok(instructions)
}

fn parse_instruction(instr: &str) -> Instruction {
    let instr_ops: Vec<&str> = instr.splitn(2, ' ').collect();
    let instr_name = instr_ops[0].to_owned();

    if instr_ops.len() == 1 {
        (instr_name, Vec::new())
    }
    else {
        (instr_name, instr_ops[1].split(", ").map(Operand::from).collect())
    }
}

impl <'a> From<&'a str> for Operand {
    fn from(operand: &'a str) -> Self {
        if operand == "vcc" {
            return Operand::VCC;
        }
        /* Hexadecimal literal */
        if operand.len() > 2 && &operand[0..2] == "0x" {
            return Operand::Lit(u32::from_str_radix(&operand[2..], 16).unwrap())
        }

        let prefix_char = operand.chars().nth(0).unwrap();

        /* Decimal literal */
        if prefix_char.is_digit(10) {
            return Operand::Lit(u32::from_str_radix(operand, 10).unwrap())
        }
        /* Not a scalar/vector general-purpose register */
        if prefix_char != 's' && prefix_char != 'v' {
            return Operand::Keyseq(operand.to_string());
        }
        match operand[1..].parse::<usize>() {
            Ok(i) =>
                /* Single register reference (s0, v1) */
                if prefix_char == 's' { Operand::SReg(i) }
                else                  { Operand::VReg(i) }
            _ => {
                /* Register range (s[2:3], v[8:9]) */
                let sides: Vec<&str> = operand[2..operand.len() - 1].split(':').collect();
                let from = sides[0].parse::<usize>().unwrap();
                let to = sides[1].parse::<usize>().unwrap();

                if prefix_char == 's' { Operand::SRegs(from, to) }
                else                  { Operand::VRegs(from, to) }
            }
        }
    }
}
