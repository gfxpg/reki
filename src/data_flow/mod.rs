pub mod exec_state;
mod types;
mod ops;

use itertools::Itertools;

use asm::Instruction;
use control_flow::ControlFlowMap;
use self::exec_state::ExecState;
use self::types::{Program, Reg, Statement, Binding, Variable};

pub fn analyze(st: &mut ExecState, instrs: &[Instruction], cf_map: &ControlFlowMap) -> Program {
    eval_instructions_within_block(st, instrs.iter().enumerate(), instrs.len(), cf_map)
}

type InstructionIter<'a> = std::iter::Enumerate<std::slice::Iter<'a, Instruction>>;

fn eval_instructions_within_block(st: &mut ExecState, mut instr_iter: InstructionIter, instr_count: usize, cf_map: &ControlFlowMap) -> Program {
    let mut pgm = Program::new();

    while let Some((instr_idx, (instr, ops))) = instr_iter.next() {
        if let Some(index) = cf_map.label_at_instruction(instr_idx) {
            pgm.push((instr_idx + 1, Statement::Label { index }));
        }

        use control_flow::BranchKind;
        match cf_map.branch_at_instruction(instr_idx) {
            Some((BranchKind::Uncond, _, dst)) => {
                if dst < instr_idx {
                    /* I'm not sure what a real-world use case for this would be and whether we
                     * need to recompute the state and diff it against the current one to bind
                     * variables */
                    panic!("Unconditional backward jump from {} to {}", instr_idx, dst);
                }
                /* Simply skip the instructions we're jumping over, no need to insert a goto */
                let _ = instr_iter.nth(dst - instr_idx - 2);
                continue;
            },
            Some((ref kind, label_idx, dst)) if dst > instr_idx => {
                /* A forward conditional branch wraps a block of instructions:
                 *
                 * branch_if_cond label0
                 * ...instructions
                 * label0:
                 * ...rest of the program
                 *
                 * We clone the execution state and pass it through the block, then diff it
                 * against the state before the jump. Registers with different contents are
                 * then changed to point to the same _variable_, which is reassigned within
                 * the block. */

                let mut st_block = st.clone();
                let block_instr_iter = instr_iter.clone().dropping_back(instr_count - dst);
                let mut block = eval_instructions_within_block(&mut st_block, block_instr_iter, instr_count, cf_map);

                let (mut declarations, mut assignments_executed, mut assignments_skipped) =
                    block_variables(&mut st_block, st);

                st.sgprs = st_block.sgprs;
                st.vgprs = st_block.vgprs;
                st.bindings = st_block.bindings;
                st.variables = st_block.variables;

                pgm.append(&mut declarations.into_iter().map(|statement| (instr_idx + 1, statement)).collect());
                pgm.append(&mut assignments_skipped.into_iter().map(|statement| (instr_idx + 1, statement)).collect());

                pgm.push((instr_idx + 1, match kind {
                    BranchKind::SCCSet => Statement::JumpIf { cond: st.scc.unwrap(), label_idx },
                    BranchKind::SCCUnset => Statement::JumpUnless { cond: st.scc.unwrap(), label_idx },
                    _ => panic!("Unhandled branch kind: {:?}", kind)
                }));
                pgm.append(&mut assignments_executed.into_iter().map(|statement| (instr_idx + 2, statement)).collect());
                /* TODO: Update expressions to point at variables where bindings were used */
                pgm.append(&mut block);

                /* Skip the block, we've already evaluated it */
                let _ = instr_iter.nth(dst - instr_idx - 2);
                continue;
            },
            /* backward conditional branch */
            Some((ref kind, label_idx, _)) => {
                pgm.push((instr_idx + 1, match kind {
                    BranchKind::SCCSet => Statement::JumpIf { cond: st.scc.unwrap(), label_idx },
                    BranchKind::SCCUnset => Statement::JumpUnless { cond: st.scc.unwrap(), label_idx },
                    _ => panic!("Unhandled branch kind: {:?}", kind)
                }));

                continue;
            },
            None => ()
        }

        ops::eval_gcn_instruction(st, &mut pgm, instr_idx, instr.as_str(), ops.as_slice());
    }

    pgm
}

fn block_variables(st_executed: &mut ExecState, st_skipped: &ExecState) -> (Vec<Statement>, Vec<Statement>, Vec<Statement>) {
    let mut declarations: Vec<Statement> = Vec::new();

    let last_var_idx = if st_executed.variables.len() == 0 { 0 } else { st_executed.variables.len() - 1 };
    let (sgprs, mut sgpr_executed, mut sgpr_skipped) =
        compare_regs_extract_vars(&st_executed.sgprs, &st_skipped.sgprs, &mut st_executed.variables, &mut st_executed.bindings);
    let (vgprs, mut vgpr_executed, mut vgpr_skipped) =
        compare_regs_extract_vars(&st_executed.vgprs, &st_skipped.vgprs, &mut st_executed.variables, &mut st_executed.bindings);

    sgpr_executed.append(&mut vgpr_executed);
    sgpr_skipped.append(&mut vgpr_skipped);

    st_executed.sgprs = sgprs;
    st_executed.vgprs = vgprs;

    for i in last_var_idx..st_executed.variables.len() {
       declarations.push(Statement::VarDecl { var_idx: i + last_var_idx }); 
    }

    (declarations, sgpr_executed, sgpr_skipped)
}

fn compare_regs_extract_vars(regs_executed: &Vec<Reg>, regs_skipped: &Vec<Reg>, variables: &mut Vec<Variable>, bindings: &mut Vec<Binding>) -> (Vec<Reg>, Vec<Statement>, Vec<Statement>) {
    let mut reg_iter = regs_executed.iter().zip(regs_skipped.iter()).enumerate();

    let mut executed_branch: Vec<Statement> = Vec::new();
    let mut skipped_branch: Vec<Statement> = Vec::new();

    let mut new_regs = regs_executed.clone();

    while let Some((reg_idx, (exec_reg, skip_reg))) = reg_iter.next() {
        if exec_reg == skip_reg { continue; }

        let Reg(exec_idx, exec_lo_dword) = exec_reg;
        let Reg(_, exec_hi_dword) = regs_executed[reg_idx..].iter()
            .take_while(|&Reg(idx, _)| idx == exec_idx).last().unwrap();
        let Reg(skip_idx, skip_lo_dword) = skip_reg;
        let Reg(_, skip_hi_dword) = regs_skipped[reg_idx..].iter()
            .take_while(|&Reg(idx, _)| idx == skip_idx).last().unwrap();

        let exec_dwords = exec_hi_dword - exec_lo_dword + 1;
        let skip_dwords = skip_hi_dword - skip_lo_dword + 1;

        variables.push(match (exec_dwords, skip_dwords) {
            (1, 1) => Variable::Dword,
            (2, 2) => Variable::Qword,
            (4, 4) => Variable::DQword,
            (a, b) if a == b => panic!("{}-word variables are not supported", a),
            (2, 1) => Variable::PartialQword,
            (4, a) if a < 4 => Variable::PartialDQword,
            (a, b) => panic!("Unsupported variable size: {} dwords on the left, {} dwords on the right", a, b)
        });
        let var_dwords = std::cmp::max(exec_dwords as usize, skip_dwords as usize);
        let var_idx = variables.len() - 1;

        bindings.push(Binding::Variable { idx: var_idx });
        let var_binding_idx = bindings.len() - 1;

        create_assignments(&mut executed_branch, &regs_executed[reg_idx..reg_idx + var_dwords], var_idx);
        create_assignments(&mut skipped_branch, &regs_skipped[reg_idx..reg_idx + var_dwords], var_idx);

        for dw in 0..var_dwords { new_regs[reg_idx + dw] = Reg(var_binding_idx, dw as u8); }

        if var_dwords > 1 {
            // skip subsequent registers with the same binding
            let _ = reg_iter.nth(var_dwords - 1);
        }
    }
    (new_regs, executed_branch, skipped_branch)
}

fn create_assignments(assignments: &mut Vec<Statement>, var_regs: &[Reg], var_idx: usize) {
    let mut i = 0;
    while i < var_regs.len() {
        let Reg(binding_idx, binding_dword) = var_regs[i];
        let Reg(_, binding_hi_dword) = var_regs[i..].iter()
            .take_while(|&Reg(idx, _)| *idx == binding_idx).last().unwrap();

        let assignment_dwords = 1 + binding_hi_dword - binding_dword;

        assignments.push(match assignment_dwords {
            1 => Statement::DwordVarAssignment { var_idx, binding_idx, binding_dword, var_dword: i as u8 },
            2 => Statement::QwordVarAssignment { var_idx, binding_idx, binding_dword, var_dword: i as u8 },
            4 => Statement::DQwordVarAssignment { var_idx, binding_idx, binding_dword, var_dword: i as u8 },
            n => panic!("{}-word variables are not supported", n)
        });

        i += assignment_dwords as usize;
    }
}

