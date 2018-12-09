use assembly::{Operand, Instruction};

pub type ControlFlowMap = std::collections::HashMap<usize, ControlFlowNode>;

#[derive(Debug)]
pub enum ControlFlowNode {
    ForkSCCSet(usize),
    ForkSCCUnset(usize),
    Uncond(usize)
}

pub fn build_map(instrs: &Vec<Instruction>) -> ControlFlowMap {
    use self::ControlFlowNode::*;

    let mut map = ControlFlowMap::new();

    for (idx, (instr, ops)) in instrs.iter().enumerate() {
        match instr.as_str() {
            "s_branch" => {
                map.insert(idx, Uncond(branch_destination(idx, ops.as_slice())));
            },
            "s_cbranch_scc1" => {
                map.insert(idx, ForkSCCSet(branch_destination(idx, ops.as_slice())));
            },
            "s_cbranch_scc0" => {
                map.insert(idx, ForkSCCUnset(branch_destination(idx, ops.as_slice())));
            },
            _ => ()
        }
    }

    map
}

fn branch_destination(instr_idx: usize, branch_ops: &[Operand]) -> usize {
    use assembly::Operand::*;

    match branch_ops {
        [Lit(branch_fwd_idx)] if *branch_fwd_idx <= 32767 =>
            instr_idx + 1 + (*branch_fwd_idx as usize),
        [Lit(branch_bwd_idx)] =>
            (instr_idx as i64 + 1 + (*branch_bwd_idx as i16) as i64) as usize,
        _ =>
            panic!("Unrecognized branch operands: {:?}", branch_ops)
    }
}
