use assembly::Operand;
use exec_state::ExecutionState;

fn resolve_load<'a>(st: &'a ExecutionState, mem_loc: &str, offset: u32) -> &'a str {
    match mem_loc {
        "AQL_DISPATCH_PACKET" =>
            match offset {
                4 => "get_local_size(0)",
                _ => panic!("Unable to resolve offset into AQL_DISPATCH_PACKET struct")
            },
        "KERNARG" =>
            st.kernel_args.iter()
                .find(|arg| arg.offset == offset)
                .map(|arg| match arg.name.as_str() {
                    "HiddenGlobalOffsetX" => "get_global_offset(0)",
                    _ => arg.name.as_str()
                })
                .expect("Unable to resolve offset into kernel arguments"),
        _ =>
            panic!("Unable to resolve {} (offset {})", mem_loc, offset)
    }
}

pub fn eval_loads(st: ExecutionState) {
    for (instr, ops) in st.instrs.iter() {
        println!("{} {:?}", instr, ops);

        if instr == "s_load_dword" {
            if let Operand::ScalarRegRange(ref rng) = ops[1] {
                if let Operand::Lit(ref offset) = ops[2] {
                    let (lo, hi) = (rng.start(), rng.end());
                    assert!(hi - lo == 1);
                    let loc = st.sgprs[*lo];
                    assert!(loc == st.sgprs[*hi], "lower and higher dwords of load source point to different locations");

                    let contents = resolve_load(&st, loc, *offset);

                    println!("addr: {} {} -> {}", st.sgprs[*lo], st.sgprs[*hi], contents);
                }
            }
        }
    }
}

