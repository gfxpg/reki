use std::collections::HashMap;

use crate::asm::kernel_args::KernelArgs;
use crate::data_flow::types::{Program, Statement, Binding, BuiltIn, DataKind, Expr, Condition};
use crate::data_flow::exec_state::ExecState;

#[derive(Debug)]
pub enum BoundExpr {
    Mul(Box<BoundExpr>, Box<BoundExpr>),
    Add(Box<BoundExpr>, Box<BoundExpr>),
    And(Box<BoundExpr>, Box<BoundExpr>),
    Shl(Box<BoundExpr>, Box<BoundExpr>),
    CompareLt(Box<BoundExpr>, Box<BoundExpr>),
    CompareEql(Box<BoundExpr>, Box<BoundExpr>),
    Negate(Box<BoundExpr>),
    Cast(Box<BoundExpr>, DataKind),
    I32(i32),
    U32(u32),
    InitState(BuiltIn),
    DwordArg { arg_idx: usize, dword: u8 },
    Variable { idx: usize, dword: u8 },
    Placeholder
}

#[derive(Debug)]
pub enum ProgramStatement {
    Declaration { var_idx: usize },
    Assignment { var_idx: usize, expr: BoundExpr },
    JumpIf { label_idx: usize, cond: BoundExpr },
    Label { label_idx: usize },
    Store { addr: usize, data: BoundExpr, kind: DataKind }
}

pub fn build(args: &KernelArgs, st: ExecState, program: Program) -> Vec<ProgramStatement> {
    /* Binding index -> variable index */
    let mut var_bindings: HashMap<usize, usize> = HashMap::new();

    let mut stmts: Vec<ProgramStatement> = Vec::with_capacity(var_bindings.len());

    for (_, stmt) in program {
        match stmt {
            Statement::VarAssignment { binding_idx, var_idx, .. } if binding_idx < std::usize::MAX => {
                // TODO FIXME: handle binding_dword and data kind
                println!("binding {:?} -> var {:?}", binding_idx, var_idx);
                var_bindings.insert(binding_idx, var_idx);
                let expr = reduce_binding_to_expr(binding_idx, &st.bindings, &var_bindings, args);
                stmts.push(ProgramStatement::Assignment { var_idx, expr });
            },
            Statement::VarAssignment { .. } => (), // std::usize::max => uninitialized register state
            Statement::VarDecl { var_idx } => {
                stmts.push(ProgramStatement::Declaration { var_idx })
            },
            Statement::JumpIf { cond, label_idx } => {
                let cond_expr = match cond {
                    Condition::Lt(lhs, rhs) =>
                        BoundExpr::CompareLt(Box::new(reduce_binding_to_expr(lhs, &st.bindings, &var_bindings, args)),
                                             Box::new(reduce_binding_to_expr(rhs, &st.bindings, &var_bindings, args))),
                    Condition::Eql(lhs, rhs) =>
                        BoundExpr::CompareEql(Box::new(reduce_binding_to_expr(lhs, &st.bindings, &var_bindings, args)),
                                              Box::new(reduce_binding_to_expr(rhs, &st.bindings, &var_bindings, args)))
                };
                stmts.push(ProgramStatement::JumpIf { label_idx, cond: cond_expr });
            },
            Statement::JumpUnless { cond, label_idx } => {
                let cond_expr = match cond {
                    Condition::Lt(lhs, rhs) =>
                        BoundExpr::CompareLt(Box::new(reduce_binding_to_expr(lhs, &st.bindings, &var_bindings, args)),
                                             Box::new(reduce_binding_to_expr(rhs, &st.bindings, &var_bindings, args))),
                    Condition::Eql(lhs, rhs) =>
                        BoundExpr::CompareEql(Box::new(reduce_binding_to_expr(lhs, &st.bindings, &var_bindings, args)),
                                              Box::new(reduce_binding_to_expr(rhs, &st.bindings, &var_bindings, args)))
                };
                stmts.push(ProgramStatement::JumpIf { label_idx, cond: BoundExpr::Negate(Box::new(cond_expr)) });
            },
            Statement::Label { index } => {
                stmts.push(ProgramStatement::Label { label_idx: index })
            },
            Statement::Store { addr, data, kind } => {
                stmts.push(ProgramStatement::Store { addr, kind, data: reduce_binding_to_expr(data, &st.bindings, &var_bindings, args) })
            }
        }
    }

    stmts
}

fn reduce_binding_to_expr(idx: usize, bindings: &Vec<Binding>, vars: &HashMap<usize, usize>, args: &KernelArgs) -> BoundExpr {
    match bindings[idx] {
        Binding::Computed { expr, kind: _ } => {
            match expr {
                Expr::Mul(lhs, rhs) => {
                    BoundExpr::Mul(Box::new(reduce_binding_to_expr(lhs, bindings, vars, args)),
                                   Box::new(reduce_binding_to_expr(rhs, bindings, vars, args)))
                },
                Expr::Add(lhs, rhs) => {
                    BoundExpr::Add(Box::new(reduce_binding_to_expr(lhs, bindings, vars, args)),
                                   Box::new(reduce_binding_to_expr(rhs, bindings, vars, args)))
                },
                Expr::And(lhs, rhs) => {
                    BoundExpr::And(Box::new(reduce_binding_to_expr(lhs, bindings, vars, args)),
                                   Box::new(reduce_binding_to_expr(rhs, bindings, vars, args)))
                },
                Expr::Shl(lhs, rhs) => {
                    BoundExpr::Shl(Box::new(reduce_binding_to_expr(lhs, bindings, vars, args)),
                                   Box::new(reduce_binding_to_expr(rhs, bindings, vars, args)))
                },
                _ => panic!("Unhandled expr: {:?}", expr)
            }
        },
        Binding::U32(val) => BoundExpr::U32(val),
        Binding::I32(val) => BoundExpr::I32(val),
        Binding::Deref { ptr, offset, kind } => {
            resolve_dereference(bindings, ptr, offset as u32, kind, args)
        },
        Binding::InitState(builtin) => {
            BoundExpr::InitState(builtin)
        },
        Binding::DwordElement { of, dword } if vars.contains_key(&of) => {
            BoundExpr::Variable { idx: of, dword }
        },
        Binding::DwordElement { of, dword } | Binding::QwordElement { of, dword } => {
            match bindings[of] {
                Binding::Deref { ptr, offset, kind: _ } => {
                    resolve_dereference(bindings, ptr, offset as u32 + dword as u32, DataKind::Dword, args)
                },
                _ => panic!("Unable to resolve dword element #{:?} of {:?}", dword, bindings[of])
            }
        },
        Binding::Cast { source, kind } => {
            BoundExpr::Cast(Box::new(reduce_binding_to_expr(source, bindings, vars, args)), kind)
        },
        other => panic!("Unhandled binding: {:?}", other)
    }
}

fn resolve_dereference(bindings: &Vec<Binding>, ptr: usize, offset: u32, _kind: DataKind, args: &KernelArgs) -> BoundExpr {
    match bindings[ptr] {
        Binding::InitState(BuiltIn::PtrKernarg { .. }) => {
            match args.find_idx_and_dword(offset) {
                Some((arg_idx, dword)) => BoundExpr::DwordArg { arg_idx, dword },
                None => panic!("Unable to resolve kernel argument at offset {}; args struct: {:#?}", offset, args)
            }
        },
        pointer => {
            eprintln!("Unable to resolve pointer derefence of {:?} at offset {}", pointer, offset);
            BoundExpr::Placeholder
        }
    }
}
