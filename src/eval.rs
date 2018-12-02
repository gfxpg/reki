use exec_state::ExecutionState;
use expr::{Reg, Expr, Statement, Condition, Binding, BindingIdx, DataKind};

pub type FlatProgram = Vec<(usize, Statement)>;

macro_rules! insert_into {
    ($vec:expr, $index:expr, $contents:expr) => {
        if $vec.len() <= $index {
            $vec.push($contents);
        }
        else {
            $vec[$index] = $contents;
        }
    }
}

use assembly::{Instruction, Operand};
use assembly::Operand::*;

fn load_ptr_binding(st: &ExecutionState, source: &Operand) -> BindingIdx {
    let (src_lo, src_hi) = match source {
        SRegs(ref src_lo, ref src_hi) => (st.sgprs[*src_lo], st.sgprs[*src_hi]),
        VRegs(ref src_lo, ref src_hi) => (st.vgprs[*src_lo], st.vgprs[*src_hi]),
        _ => panic!("Cannot resolve load, unrecognized source operand {:?}", source)
    };
    match (src_lo, src_hi) {
        (Reg(idx_lo, 0), Reg(idx_hi, 1)) if idx_lo == idx_hi => idx_lo,
        _ => panic!("Cannot resolve load, got invalid pointer (lo: {:?}, hi: {:?})", src_lo, src_hi)
    }
}

fn eval_global_load(st: &mut ExecutionState, instr: &str, ops: &[Operand]) {
    let kind = match &instr[12..] {
        "ushort" => DataKind::U16,
        "dword" => DataKind::Dword,
        _ => panic!("Unknown data type modifier {}", &instr[12..])
    };
    let binding = match ops {
        [_, src, _, Offset(ref offset)] =>
            Binding::Deref { ptr: load_ptr_binding(st, src), offset: *offset as u32, kind },
        [_, src, _] =>
            Binding::Deref { ptr: load_ptr_binding(st, src), offset: 0, kind },
        _ =>
            panic!("Cannot resolve load, unrecognized operands {:?}", ops)
    };
    st.bindings.push(binding);
    match ops[0] {
        VReg(ref dst) => insert_into!(st.vgprs, *dst, Reg(st.bindings.len() - 1, 0)),
        _ => panic!("Cannot resolve load, unrecognized destination {:?}", ops[0])
    }
}

fn eval_s_load(st: &mut ExecutionState, instr: &str, ops: &[Operand]) {
    let (ptr, offset) = match ops {
        [_, source, Lit(ref offset)] => (load_ptr_binding(st, source), offset),
        _ => panic!("Received invalid operands in eval_s_load")
    };

    match (instr, ops) {
        ("s_load_dword", [SReg(ref dst), _, _]) => {
            st.bindings.push(Binding::Deref { ptr, offset: *offset, kind: DataKind::Dword });
            insert_into!(st.sgprs, *dst, Reg(st.bindings.len() - 1, 0));
        },
        ("s_load_dwordx2", [SRegs(ref dst_lo, _), _, _]) => {
            st.bindings.push(Binding::Deref { ptr, offset: *offset, kind: DataKind::Qword });
            for i in 0..2 { insert_into!(st.sgprs, *dst_lo + i as usize, Reg(st.bindings.len() - 1, i)); }
        },
        ("s_load_dwordx4", [SRegs(ref dst_lo, _), _, _]) => {
            st.bindings.push(Binding::Deref { ptr, offset: *offset, kind: DataKind::DQword });
            for i in 0..4 { insert_into!(st.sgprs, *dst_lo + i as usize, Reg(st.bindings.len() - 1, i)); }
        },
        unsupported => panic!("Operation not supported: {:?}", unsupported)
    }
}

fn eval_salu_op(st: &mut ExecutionState, instr: &str, ops: &[Operand]) {
    match (instr, ops) {
        ("s_mul_i32", [SReg(ref dst), SReg(ref op1), SReg(ref op2)]) => {
            let expr = match (st.sgprs[*op1], st.sgprs[*op2]) {
                (Reg(op1_idx, 0), Reg(op2_idx, 0)) => Expr::Mul(op1_idx, op2_idx),
                _ => panic!("Operation not supported: {:?} {:?}", instr, ops)
            };
            st.bindings.push(Binding::Computed { expr, kind: DataKind::Dword });
            insert_into!(st.sgprs, *dst, Reg(st.bindings.len() - 1, 0));
        },
        ("s_and_b32", [SReg(ref dst), SReg(ref src), Lit(ref mask)]) => {
            let expr = match st.sgprs[*src] {
                Reg(src_idx, 0) => Expr::And(src_idx, *mask),
                other => panic!("Operand not supported: {:?} in s_and_b32", other)
            };
            let kind = match mask {
                65535 => DataKind::U16, /* 0xffff is most likely a 32 -> 16 downcast */
                _ => DataKind::Dword
            };
            st.bindings.push(Binding::Computed { expr, kind });
            insert_into!(st.sgprs, *dst, Reg(st.bindings.len() - 1, 0));
        },
        ("s_cmp_lt_i32", [op1, op2]) =>
            match (operand_reg(st, op1, "i32"), operand_reg(st, op2, "i32")) {
                (Reg(op1_idx, 0), Reg(op2_idx, 0)) => {
                    st.scc = Some(Condition::Lt(op1_idx, op2_idx))
                },
                other => panic!("Unrecognized operands: {:?}", other)
            },
        unsupported => panic!("Operation not supported: {:?}", unsupported)
    }
}

fn operand_reg(st: &mut ExecutionState, op: &Operand, typehint: &str) -> Reg {
    match op {
        SReg(ref i) => st.sgprs[*i],
        VReg(ref i) => st.vgprs[*i],
        Lit(ref contents) => {
            match typehint {
                "i32" => st.bindings.push(Binding::I32(*contents as i32)),
                "u32" | _ => st.bindings.push(Binding::U32(*contents))
            }
            Reg(st.bindings.len() - 1, 0)
        },
        _ => panic!("Unrecognized operand {:?}", op)
    }
}

fn operand_binding_dw(st: &mut ExecutionState, op: &Operand, typehint: &str) -> BindingIdx {
    match operand_reg(st, op, typehint) {
        Reg(op_idx, 0) => op_idx,
        Reg(of, dword) => {
            st.bindings.push(Binding::DwordElement { of, dword });
            st.bindings.len() - 1
        }
    }
}

fn eval_valu_op(st: &mut ExecutionState, instr: &str, ops: &[Operand]) {
    match (instr, ops) {
        ("v_mov_b32_e32", [VReg(ref dst), src]) => {
            let contents = operand_reg(st, src, "u32");
            insert_into!(st.vgprs, *dst, contents);
        },
        ("v_add_u32_e32", [VReg(ref dst), op1, op2]) => {
            let op1_idx = operand_binding_dw(st, op1, "u32");
            let op2_idx = operand_binding_dw(st, op2, "u32");
            st.bindings.push(Binding::Computed { expr: Expr::Add(op1_idx, op2_idx), kind: DataKind::Dword });
            insert_into!(st.vgprs, *dst, Reg(st.bindings.len() - 1, 0));
        },
        ("v_mul_lo_u32", [VReg(ref dst), op1, op2]) => {
            let op1_idx = operand_binding_dw(st, op1, "u32");
            let op2_idx = operand_binding_dw(st, op2, "u32");
            st.bindings.push(Binding::Computed { expr: Expr::Mul(op1_idx, op2_idx), kind: DataKind::Dword });
            insert_into!(st.vgprs, *dst, Reg(st.bindings.len() - 1, 0));
        },
        unsupported => panic!("Operation not supported: {:?}", unsupported)
//       ("v_ashrrev_i64", [VRegs(ref dst_lo, ref dst_hi), Lit(ref shift), VRegs(ref src_lo, ref src_hi)]) => {
//           /* Most likely an i32 -> i64 conversion with optional multiplication/division
//            * by a power of two expressed as a shift relative to 32 */
//           if st.vgprs[*src_lo].as_str() == "0" {
//               let result = if *shift == 32 {
//                   st.vgprs[*src_hi].to_owned()
//               }
//               else if *shift < 32 {
//                   format!("{} * {}", st.vgprs[*src_hi], 2u32.pow(32 - shift))
//               }
//               else {
//                   format!("{} / {}", st.vgprs[*src_hi], 2u32.pow(shift - 32))
//               };
//               insert_into!(st.vgprs, *dst_lo, result.to_owned());
//               insert_into!(st.vgprs, *dst_hi, result);
//           }
//       },
//       ("v_add_co_u32_e32", [VReg(ref dst), VCC, op1, op2]) => {
//           /* Check if this is actually 64-bit addition */
//           if let Some("v_addc_co_u32_e32") = instr_iter.peek().map(|i| i.0.as_str()) {
//               let ops_hi = &instr_iter.peek().unwrap().1;
//               if ops_hi[0] == VReg(*dst + 1) && ops_hi[4] == VCC {
//                   /* This is most likely 64-bit addition
//                    * FIXME: check the operands of addc to make sure */
//                   /* Assume that dst = sum of operands of the first addition (this works
//                    * because we do not yet differentiate between higher and lower dwords */
//                   let result = format!("({} + {})", operand!(st, op1), operand!(st, op2));
//                   insert_into!(st.vgprs, *dst, result.to_owned());
//                   insert_into!(st.vgprs, *dst + 1, result);
//               }
//           }
//       },
    }
}

pub fn eval_pgm(st: &mut ExecutionState, instrs: Vec<Instruction>) -> FlatProgram {
    let mut instr_iter = instrs.iter().enumerate().peekable();

    let mut pgm: FlatProgram = Vec::new();

    while let Some((instr_idx, (instr, ops))) = instr_iter.next() {
        println!("{:?}\n\n~~~~~~~~~ {} {} {:?}", st, instr_idx + 1, instr, ops);

        match instr.as_str() {
            "s_waitcnt" => (),
            "s_cbranch_scc1" => match ops.as_slice() {
                [Lit(ref offset)] => pgm.push((instr_idx + 1, Statement::JumpIf {
                    cond: st.scc.unwrap().to_owned(), instr_offset: *offset as i16
                })),
                _ => ()
            },
            instr if instr.starts_with("s_load") =>
                eval_s_load(st, instr, ops.as_slice()),
            instr if instr.starts_with("global_load") =>
                eval_global_load(st, instr, ops.as_slice()),
            instr if instr.starts_with("s_") =>
                eval_salu_op(st, instr, ops.as_slice()),
            instr if instr.starts_with("v_") =>
                eval_valu_op(st, instr, ops.as_slice()),
            unsupported => panic!("Operation not supported: {:?}", unsupported)
        }

        println!("Program: {:?}", pgm);
    }

    pgm
}
