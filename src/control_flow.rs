use crate::asm::{Instruction, Operand, Operand::*};

#[derive(Debug, Copy, Clone)]
pub enum BranchKind {
    SCCSet, SCCUnset, Uncond
}

type LabelIdx = usize;
type InstructionIdx = usize;

#[derive(Debug)]
pub struct ControlFlowMap {
    jumps: Vec<(InstructionIdx, BranchKind, LabelIdx)>,
    labels: Vec<usize>
}

impl ControlFlowMap {
    pub fn label_at_instruction(&self, instruction_idx: usize) -> Option<usize> {
        self.labels.iter().position(|&idx| idx == instruction_idx)
    }

    pub fn branch_at_instruction(&self, instruction_idx: usize) -> Option<(BranchKind, LabelIdx, InstructionIdx)> {
        self.jumps.iter()
            .find(|&(idx, _, _)| *idx == instruction_idx)
            .map(|&(_, kind, label_idx)| (kind, label_idx, self.labels[label_idx]))
    }
}


pub fn build_map(instrs: &Vec<Instruction>) -> ControlFlowMap {
    let mut jumps: Vec<(InstructionIdx, BranchKind, LabelIdx)> = Vec::new();
    let mut labels: Vec<usize> = Vec::new();

    for (idx, (instr, ops)) in instrs.iter().enumerate() {
        match instr.as_str() {
            "s_branch" => {
                labels.push(branch_destination(idx, ops));
                jumps.push((idx, BranchKind::Uncond, labels.len() - 1));
            },
            "s_cbranch_scc1" => {
                labels.push(branch_destination(idx, ops));
                jumps.push((idx, BranchKind::SCCSet, labels.len() - 1));
            },
            "s_cbranch_scc0" => {
                labels.push(branch_destination(idx, ops));
                jumps.push((idx, BranchKind::SCCUnset, labels.len() - 1));
            },
            _ => ()
        }
    }

    ControlFlowMap { jumps, labels }
}

fn branch_destination(instr_idx: usize, branch_ops: &[Operand]) -> usize {
    match branch_ops {
        [Lit(branch_fwd_idx)] if *branch_fwd_idx <= 32767 =>
            instr_idx + 1 + (*branch_fwd_idx as usize),
        [Lit(branch_bwd_idx)] =>
            (instr_idx as i64 + 1 + (*branch_bwd_idx as i16) as i64) as usize,
        _ =>
            panic!("Unrecognized branch operands: {:?}", branch_ops)
    }
}
