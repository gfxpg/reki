use std::collections::HashMap;

use asm::kernel_args::KernelArg;
use data_flow::types::{Program, Statement, Binding, BuiltIn, DataKind, Expr};
use data_flow::exec_state::ExecState;

#[derive(Debug)]
pub enum BoundExpr {
    Mul(Box<BoundExpr>, Box<BoundExpr>),
    Add(Box<BoundExpr>, Box<BoundExpr>),
    And(Box<BoundExpr>, Box<BoundExpr>),
    I32(i32),
    U32(u32),
    InitState(BuiltIn),
    Variable { idx: usize, dword: u8 },
    Placeholder
}

#[derive(Debug)]
pub enum ProgramStatement {
    Assignment { var_idx: usize, expr: BoundExpr }
}

pub fn build(args: &Vec<KernelArg>, st: ExecState, program: Program) {
    /* Binding index -> variable index */
    let mut var_bindings: HashMap<usize, usize> = HashMap::new();

    let mut stmts: Vec<ProgramStatement> = Vec::with_capacity(var_bindings.len());

    for (_, stmt) in program {
        println!("{:?}", stmt);
        match stmt {
            Statement::DwordVarAssignment { binding_idx, var_idx, .. } if binding_idx < std::usize::MAX => {
                println!("binding {:?} -> var {:?}", binding_idx, var_idx);
                var_bindings.insert(binding_idx, var_idx);
                let expr = reduce_binding_to_expr(binding_idx, &st.bindings, &var_bindings, args);
                stmts.push(ProgramStatement::Assignment { var_idx, expr });
            }
            _ => ()
        }
    }

    println!("Program: {:#?}", stmts);
}

fn reduce_binding_to_expr(idx: usize, bindings: &Vec<Binding>, vars: &HashMap<usize, usize>, args: &Vec<KernelArg>) -> BoundExpr {
    match bindings[idx] {
        Binding::Computed { expr, kind } => {
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
                _ => panic!("Unhandled expr: {:?}", expr)
            }
        },
        Binding::U32(val) => BoundExpr::U32(val),
        Binding::I32(val) => BoundExpr::I32(val),
        Binding::Deref { ptr, offset, kind } => {
            BoundExpr::Placeholder
            // TODO: resolve_dereference
        },
        Binding::InitState(builtin) => {
            BoundExpr::InitState(builtin)
        },
        Binding::DwordElement { of, dword } if vars.contains_key(&of) => {
            BoundExpr::Variable { idx: of, dword }
        },
        Binding::DwordElement { of, dword } | Binding::QwordElement { of, dword } => {
            match bindings[of] {
                Binding::Deref { ptr, offset, kind } => {
                    resolve_dereference(bindings, ptr, offset + dword as i32, DataKind::Dword, args)
                },
                _ => panic!("Unable to resolve dword element #{:?} of {:?}", dword, bindings[of])
            }
        },
        other => panic!("Unhandled binding: {:?}", other)
    }
}

fn resolve_dereference(bindings: &Vec<Binding>, ptr: usize, offset: i32, kind: DataKind, args: &Vec<KernelArg>) -> BoundExpr {
    use data_flow::types::Binding::*;

    match (bindings[ptr], offset, kind) {
        (PtrKernarg, _, _) => {

            panic!("Kernarg {} ({:?})", offset, kind)
        },
        (pointer, _, _) =>
            panic!("Unable to resolve pointer derefence of {:?} at offset {}", pointer, offset)
    }
}
