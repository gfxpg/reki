use exec_state::ExecutionState;

fn resolve_load(st: &ExecutionState, mem_loc: &str, offset: u32) -> String {
    match mem_loc {
        "AQL_DISPATCH_PACKET" =>
            match offset {
                4 => "get_local_size(0)".to_string(),
                _ => panic!("Unable to resolve offset into AQL_DISPATCH_PACKET struct")
            },
        "KERNARG" =>
            st.kernel_args.iter()
                .find(|arg| arg.offset == offset)
                .map(|arg| match arg.name.as_str() {
                    "HiddenGlobalOffsetX" => "get_global_offset(0)".to_string(),
                    _ => arg.name.to_owned()
                })
                .expect("Unable to resolve offset into kernel arguments"),
        _ =>
            panic!("Unable to resolve {} (offset {})", mem_loc, offset)
    }
}

macro_rules! check_load_src {
    ($regfile:expr, $src_lo:expr, $src_hi:expr) => {
        assert!($src_hi - $src_lo == 1);
        assert!($regfile[$src_lo] == $regfile[$src_hi], "lower and higher dwords of load source point to different locations");
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

macro_rules! operand {
    ($exec_state:expr, $reg:expr) => {
        (match $reg {
            SReg(ref i) => &$exec_state.sgprs[*i],
            VReg(ref i) => &$exec_state.vgprs[*i],
            invalid => panic!("Unrecognized operand {:?}", invalid)
        })
    }
}

pub fn eval_pgm(st: &mut ExecutionState) {
    use assembly::Operand::*;
    for (instr, ops) in st.instrs.iter() {
        println!("{} {:?}", instr, ops);

        match (instr.as_str(), ops.as_slice()) {
            ("s_load_dword", [SReg(ref dst), SRegs(ref src_lo, ref src_hi), Lit(ref offset)]) => {
                check_load_src!(st.sgprs, *src_lo, *src_hi);
                st.sgprs[*dst] = resolve_load(&st, &st.sgprs[*src_lo], *offset);
            },
            ("s_load_dwordx2", [SRegs(ref dst_lo, ref dst_hi), SRegs(ref src_lo, ref src_hi), Lit(ref offset)]) => {
                check_load_src!(st.sgprs, *src_lo, *src_hi);
                st.sgprs[*dst_lo] = resolve_load(&st, &st.sgprs[*src_lo], *offset);
                /* FIXME: make resolve_load accepts an arbitrary offset into kernargs, then rewrite as *offset + 4 */
                st.sgprs[*dst_hi] = resolve_load(&st, &st.sgprs[*src_lo], *offset);
            },
            ("v_mov_b32_e32", [VReg(ref dst), Lit(ref contents)]) => {
                insert_into!(st.vgprs, *dst, contents.to_string())
            },
            ("s_mul_i32", [SReg(ref dst), op1, op2]) => {
                let result = format!("{} * {}", operand!(st, op1), operand!(st, op2));
                insert_into!(st.sgprs, *dst, result);
            },
            ("v_add_u32_e32", [VReg(ref dst), op1, op2]) => {
                let result = format!("({} + {})", operand!(st, op1), operand!(st, op2));
                insert_into!(st.vgprs, *dst, result);
            },
            _ => ()
        }
    }
}
