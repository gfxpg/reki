use exec_state::ExecutionState;
use expr::{Expr, RegState, Binding, BindingIdx, DataSize};

fn resolve_load_ptr(src_lo: &RegState, src_hi: &RegState) -> BindingIdx {
    match (src_lo, src_hi) {
        (&RegState::QwLo(idx_lo), &RegState::QwHi(idx_hi)) if idx_lo == idx_hi => idx_lo,
        _ => panic!("Cannot resolve load, got invalid pointer (lo: #{:?}, hi: #{:?})", src_lo, src_hi)
    }
}

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

pub fn eval_pgm(st: &mut ExecutionState) -> Vec<Expr> {
    use assembly::Operand::*;
    let mut instr_iter = st.instrs.iter().peekable();

    let mut exprs: Vec<Expr> = Vec::new();

    while let Some((instr, ops)) = instr_iter.next() {
        println!("SGRPs: {:?}", st.sgprs);
        println!("VGRPs: {:?}\n", st.vgprs);
        println!("===== {} {:?}", instr, ops);

        match (instr.as_str(), ops.as_slice()) {
            ("s_load_dword", [SReg(ref dst), SRegs(ref src_lo, ref src_hi), Lit(ref offset)]) => {
                let ptr = resolve_load_ptr(&st.sgprs[*src_lo], &st.sgprs[*src_hi]);

                st.bindings.push(Binding::Deref { ptr, offset: *offset, size: DataSize::Dword });
                st.sgprs[*dst] = RegState::Dw(st.bindings.len() - 1);
            },
            ("s_load_dwordx2", [SRegs(ref dst_lo, ref dst_hi), SRegs(ref src_lo, ref src_hi), Lit(ref offset)]) => {
                let ptr = resolve_load_ptr(&st.sgprs[*src_lo], &st.sgprs[*src_hi]);

                st.bindings.push(Binding::Deref { ptr, offset: *offset, size: DataSize::Qword });
                st.sgprs[*dst_lo] = RegState::QwLo(st.bindings.len() - 1);
                st.sgprs[*dst_hi] = RegState::QwHi(st.bindings.len() - 1);
            },
            ("v_mov_b32_e32", [VReg(ref dst), src]) => {
                let contents = match src {
                    SReg(ref i) => st.sgprs[*i],
                    VReg(ref i) => st.vgprs[*i],
                    Lit(ref contents) => {
                        st.bindings.push(Binding::U32(*contents));
                        RegState::Dw(st.bindings.len() - 1)
                    },
                    invalid => panic!("Unrecognized operand {:?}", invalid)
                };
                insert_into!(st.vgprs, *dst, contents);
            },
            ("s_mul_i32", [SReg(ref dst), SReg(ref op1), SReg(ref op2)]) => {
                let expr = match (st.sgprs[*op1], st.sgprs[*op2]) {
                    (RegState::Dw(op1_idx), RegState::Dw(op2_idx)) =>
                        Expr::Mul(op1_idx, op2_idx),
                    _ => panic!("Operation not supported: s_mul_i32 {:?} {:?}", st.sgprs[*op1], st.sgprs[*op2])
                };
                st.bindings.push(Binding::Computed { expr, size: DataSize::Dword });
                insert_into!(st.vgprs, *dst, RegState::Dw(st.bindings.len() - 1));
            },
//            ("v_add_u32_e32", [VReg(ref dst), op1, op2]) => {
//                let result = format!("({} + {})", operand!(st, op1), operand!(st, op2));
//                insert_into!(st.vgprs, *dst, result);
//            },
//            ("v_ashrrev_i64", [VRegs(ref dst_lo, ref dst_hi), Lit(ref shift), VRegs(ref src_lo, ref src_hi)]) => {
//                /* Most likely an i32 -> i64 conversion with optional multiplication/division
//                 * by a power of two expressed as a shift relative to 32 */
//                if st.vgprs[*src_lo].as_str() == "0" {
//                    let result = if *shift == 32 {
//                        st.vgprs[*src_hi].to_owned()
//                    }
//                    else if *shift < 32 {
//                        format!("{} * {}", st.vgprs[*src_hi], 2u32.pow(32 - shift))
//                    }
//                    else {
//                        format!("{} / {}", st.vgprs[*src_hi], 2u32.pow(shift - 32))
//                    };
//                    insert_into!(st.vgprs, *dst_lo, result.to_owned());
//                    insert_into!(st.vgprs, *dst_hi, result);
//                }
//            },
//            ("v_add_co_u32_e32", [VReg(ref dst), VCC, op1, op2]) => {
//                /* Check if this is actually 64-bit addition */
//                if let Some("v_addc_co_u32_e32") = instr_iter.peek().map(|i| i.0.as_str()) {
//                    let ops_hi = &instr_iter.peek().unwrap().1;
//                    if ops_hi[0] == VReg(*dst + 1) && ops_hi[4] == VCC {
//                        /* This is most likely 64-bit addition
//                         * FIXME: check the operands of addc to make sure */
//                        /* Assume that dst = sum of operands of the first addition (this works
//                         * because we do not yet differentiate between higher and lower dwords */
//                        let result = format!("({} + {})", operand!(st, op1), operand!(st, op2));
//                        insert_into!(st.vgprs, *dst, result.to_owned());
//                        insert_into!(st.vgprs, *dst + 1, result);
//                    }
//                }
//            },
            _ => ()
        }
    }

    exprs
}
