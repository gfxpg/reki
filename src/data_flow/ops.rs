use crate::data_flow::{Program, exec_state::ExecState};
use crate::data_flow::types::{Reg, Expr, Statement, Condition, Binding, BindingIdx, DataKind};
use crate::asm::{Operand, Operand::*};

macro_rules! insert_into {
    ($vec:expr, $index:expr, $contents:expr) => {
        if $vec.len() <= $index {
            while $vec.len() < $index {
                $vec.push(Reg(std::usize::MAX, 0));
            }
            $vec.push($contents);
        }
        else {
            $vec[$index] = $contents;
        }
    }
}

pub fn eval_gcn_instruction(st: &mut ExecState, pgm: &mut Program, instr_idx: usize, instr: &str, ops: &[Operand]) {
    match instr {
        "s_waitcnt" | "s_endpgm" => (),
        "global_store_dword" => match ops {
            [VRegs(dst_lo, dst_hi), VReg(src), _] if st.vgprs[*dst_lo].0 == st.vgprs[*dst_hi].0 && st.vgprs[*dst_lo].1 == 0 && st.vgprs[*dst_hi].1 == 1 => {
                let Reg(binding_dst, _) = st.vgprs[*dst_lo];
                let Reg(binding_src, _) = st.vgprs[*src];
                pgm.push((instr_idx + 1, Statement::Store { addr: binding_dst, data: binding_src, kind: DataKind::Dword }))
            },
            _ => ()
        },
        instr if instr.starts_with("s_load") => eval_s_load(st, instr, ops),
        instr if instr.starts_with("global_load") => eval_global_load(st, instr, ops),
        instr if instr.starts_with("s_") => eval_salu_op(st, instr, ops),
        instr if instr.starts_with("v_") => eval_valu_op(st, instr, ops),
        unsupported => panic!("Operation not supported: {:?}", unsupported)
    }
}

fn eval_global_load(st: &mut ExecState, instr: &str, ops: &[Operand]) {
    let kind = match &instr[12..] {
        "ushort" => DataKind::U16,
        "dword" => DataKind::Dword,
        _ => panic!("Unknown data type modifier {}", &instr[12..])
    };
    let binding = match ops {
        [_, src, _, Offset(ref offset)] =>
            Binding::Deref { ptr: load_ptr_binding(st, src), offset: *offset, kind },
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

fn eval_s_load(st: &mut ExecState, instr: &str, ops: &[Operand]) {
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

fn eval_salu_op(st: &mut ExecState, instr: &str, ops: &[Operand]) {
    match (instr, ops) {
        ("s_mul_i32", [SReg(ref dst), SReg(ref op1), SReg(ref op2)]) => {
            let expr = match (st.sgprs[*op1], st.sgprs[*op2]) {
                (Reg(op1_idx, 0), Reg(op2_idx, 0)) => Expr::Mul(op1_idx, op2_idx),
                _ => panic!("Operation not supported: {:?} {:?}", instr, ops)
            };
            st.bindings.push(Binding::Computed { expr, kind: DataKind::Dword });
            insert_into!(st.sgprs, *dst, Reg(st.bindings.len() - 1, 0));
        },
        ("s_add_i32", [SReg(ref dst), op1_raw, op2_raw]) => {
            let op1 = operand_binding_dw(st, op1_raw, "i32");
            let op2 = operand_binding_dw(st, op2_raw, "i32");
            st.bindings.push(Binding::Computed { expr: Expr::Add(op1, op2), kind: DataKind::Dword });
            insert_into!(st.sgprs, *dst, Reg(st.bindings.len() - 1, 0));
        },
        ("s_and_b32", [SReg(ref dst), op_raw, mask_raw]) => {
            let op = operand_binding_dw(st, op_raw, "u32");
            let mask = operand_binding_dw(st, mask_raw, "u32");
            let kind = match st.bindings[mask] {
                Binding::U32(65535) => DataKind::U16, /* 0xffff is most likely a 32 -> 16 downcast */
                _ => DataKind::Dword
            };
            st.bindings.push(Binding::Computed { expr: Expr::And(op, mask), kind });
            insert_into!(st.sgprs, *dst, Reg(st.bindings.len() - 1, 0));
        },
        ("s_cmp_lt_i32", [op1, op2]) =>
            match (operand_reg(st, op1, "i32"), operand_reg(st, op2, "i32")) {
                (Reg(op1_idx, 0), Reg(op2_idx, 0)) => {
                    st.scc = Some(Condition::Lt(op1_idx, op2_idx))
                },
                other => panic!("Unrecognized operands: {:?}", other)
            },
        ("s_cmp_eq_u32", [op1, op2]) =>
            match (operand_reg(st, op1, "u32"), operand_reg(st, op2, "u32")) {
                (Reg(op1_idx, 0), Reg(op2_idx, 0)) => {
                    st.scc = Some(Condition::Eql(op1_idx, op2_idx))
                },
                other => panic!("Unrecognized operands: {:?}", other)
            },
        unsupported => panic!("Operation not supported: {:?}", unsupported)
    }
}

fn eval_valu_op(st: &mut ExecState, instr: &str, ops: &[Operand]) {
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
        ("v_ashrrev_i32_e32", [VReg(ref dst), Lit(31), VReg(ref src)]) if *dst == *src + 1 => {
            /* This is most likely sign-extension of i32 to i64 */
            let Reg(src_idx, _) = st.vgprs[*src];
            st.bindings.push(Binding::Cast { source: src_idx, kind: DataKind::I64 });
            for i in 0..2 { insert_into!(st.vgprs, *src + i as usize, Reg(st.bindings.len() - 1, i)); }
        },
        ("v_lshlrev_b64", [VRegs(ref dst_lo, _), shift_by, VRegs(ref src_lo, _)]) => {
            let shift = operand_binding_dw(st, shift_by, "u32");
            let Reg(src_idx, _) = st.vgprs[*src_lo];
            st.bindings.push(Binding::Computed { expr: Expr::Shl(src_idx, shift), kind: DataKind::Qword });
            for i in 0..2 { insert_into!(st.vgprs, *dst_lo + i as usize, Reg(st.bindings.len() - 1, i)); }
        },
        ("v_add_co_u32_e32", [VReg(ref dst), VCC, op1, op2]) => {
            let op1_idx = operand_binding_dw(st, op1, "u32");
            let op2_idx = operand_binding_dw(st, op2, "u32");
            st.bindings.push(Binding::Computed { expr: Expr::Add(op1_idx, op2_idx), kind: DataKind::Dword });
            insert_into!(st.vgprs, *dst, Reg(st.bindings.len() - 1, 0));
        },
        ("v_addc_co_u32_e32", [_, VCC, Lit(0), _, VCC]) | ("v_addc_co_u32_e32", [_, VCC, _, Lit(0), VCC]) => (/* ¯\_(ツ)_/¯ */),
        ("v_addc_co_u32_e32", [VReg(ref dst), VCC, op1, op2, VCC]) => {
            /* We assume that this instruction is used for 64-bit addition — the previous operation in
             * this case must have been "v_add_co_u32_e32", which used VCC as a carry flag. */
            let Reg(lo_idx, _) = st.vgprs[*dst - 1];
            let lo_binding = st.bindings[lo_idx];
            
            let (op1_reg, op2_reg) = match (op1, op2) {
                (VReg(op1_idx), VReg(op2_idx)) => (st.vgprs[*op1_idx], st.vgprs[*op2_idx]),
                _ => panic!("Unexpected v_addc_co_u32_e32 operands: {:?}", ops)
            };

            if let Binding::Computed { expr: Expr::Add(lo_op1, lo_op2), kind: _ } = lo_binding {
                let expr = match addc_qword_matching_operands(&mut st.bindings, op1_reg, op2_reg, lo_op1, lo_op2) {
                    Some((op1_adc, op2_adc)) => Expr::Add(op1_adc, op2_adc),
                    _ => {
                        /* If we can't figure out qword operands from the previous v_add instruction,
                         * we'll have to have four operands */
                        Expr::AddHiLo {
                            lo_op1, lo_op2,
                            hi_op1: operand_binding_dw(st, op1, "u32"),
                            hi_op2: operand_binding_dw(st, op2, "u32")
                        }
                    }
                };
                st.bindings[lo_idx] = Binding::Computed { expr, kind: DataKind::Qword };
                for i in 0..2 { insert_into!(st.vgprs, *dst - 1 + i as usize, Reg(lo_idx, i)); }
            }
            else {
                panic!("64-bit addition heuristic failed; v_addc_co_u32_e32 is _not_ used to add up the high part of a 64-bit int");
            }
        },
        ("v_mac_f32_e32", [VReg(ref dst), op1, op2]) => {
            let Reg(dst_idx, _) = st.vgprs[*dst];
            let op1_idx = operand_binding_dw(st, op1, "f32");
            let op2_idx = operand_binding_dw(st, op2, "f32");
            st.bindings.push(Binding::Computed { expr: Expr::Mul(op1_idx, op2_idx), kind: DataKind::Dword });
            let mul_idx = st.bindings.len() - 1;
            st.bindings.push(Binding::Computed { expr: Expr::Add(dst_idx, mul_idx), kind: DataKind::Dword });
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
    }
}

fn addc_qword_matching_operands(bindings: &mut Vec<Binding>, op1: Reg, op2: Reg, lo_op1: BindingIdx, lo_op2: BindingIdx) -> Option<(BindingIdx, BindingIdx)> {
    let op1_adc = match (bindings[lo_op1], op1, op2) {
        (Binding::DwordElement { of, dword }, Reg(of_hi, dword_hi), _) if of == of_hi && dword + 1 == dword_hi => {
            bindings.push(Binding::QwordElement { of, dword });
            bindings.len() - 1
        },
        (Binding::DwordElement { of, dword }, _, Reg(of_hi, dword_hi)) if of == of_hi && dword + 1 == dword_hi => {
            bindings.push(Binding::QwordElement { of, dword });
            bindings.len() - 1
        },
        (_, Reg(of_hi, dword_hi), _) if lo_op1 == of_hi && dword_hi == 1 => lo_op1,
        (_, _, Reg(of_hi, dword_hi)) if lo_op1 == of_hi && dword_hi == 1 => lo_op1,
        _ => return None
    };
    let op2_adc = match (bindings[lo_op2], op1, op2) {
        (Binding::DwordElement { of, dword }, Reg(of_hi, dword_hi), _) if of == of_hi && dword + 1 == dword_hi => {
            bindings.push(Binding::QwordElement { of, dword });
            bindings.len() - 1
        },
        (Binding::DwordElement { of, dword }, _, Reg(of_hi, dword_hi)) if of == of_hi && dword + 1 == dword_hi => {
            bindings.push(Binding::QwordElement { of, dword });
            bindings.len() - 1
        },
        (_, Reg(of_hi, dword_hi), _) if lo_op2 == of_hi && dword_hi == 1 => lo_op2,
        (_, _, Reg(of_hi, dword_hi)) if lo_op2 == of_hi && dword_hi == 1 => lo_op2,
        _ => return None
    };
    Some((op1_adc, op2_adc))
}

fn load_ptr_binding(st: &ExecState, source: &Operand) -> BindingIdx {
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

fn operand_reg(st: &mut ExecState, op: &Operand, typehint: &str) -> Reg {
    match op {
        SReg(ref i) => st.sgprs[*i],
        VReg(ref i) => st.vgprs[*i],
        Lit(ref contents) => {
            match typehint {
                "i32" => st.bindings.push(Binding::I32(*contents)),
                "u32" | _ => st.bindings.push(Binding::U32(*contents as u32))
            }
            Reg(st.bindings.len() - 1, 0)
        },
        _ => panic!("Unrecognized operand {:?}", op)
    }
}

fn operand_binding_dw(st: &mut ExecState, op: &Operand, typehint: &str) -> BindingIdx {
    let Reg(of, dword) = operand_reg(st, op, typehint);

    if let Binding::Deref { kind: DataKind::DQword, .. } = st.bindings[of] {
        st.bindings.push(Binding::DwordElement { of, dword });
        st.bindings.len() - 1
    }
    else if dword == 0 {
        of
    }
    else {
        st.bindings.push(Binding::DwordElement { of, dword });
        st.bindings.len() - 1
    }
}
