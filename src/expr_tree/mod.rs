use data_flow::types::{Program, Statement, Binding, BuiltIn, DataKind, Expr};
use data_flow::exec_state::ExecState;

#[derive(Debug)]
pub enum BoundExpr {
    Mul(Box<BoundExpr>, Box<BoundExpr>),
    Add(Box<BoundExpr>, Box<BoundExpr>),
    And(Box<BoundExpr>, Box<BoundExpr>),
    InitState(BuiltIn),
    Placeholder
}

pub fn build(st: ExecState, program: Program) {
    let mut roots: Vec<usize> = Vec::new();

    for (_, stmt) in program {
        println!("{:?}", stmt);
        match stmt {
            Statement::DwordVarAssignment { binding_idx, .. } if binding_idx < std::usize::MAX => {
                roots.push(binding_idx);
            }
            _ => ()
        }
    }

    let mut tree: Vec<BoundExpr> = Vec::with_capacity(roots.len());

    for &idx in roots.iter() {
        let node = reduce_binding_to_expr(idx, &st.bindings);
        println!("=== Binding: {:#?},\n=== Tree node: {:#?},\n", st.bindings.get(idx), node);
    }
}

fn reduce_binding_to_expr(idx: usize, bindings: &Vec<Binding>) -> BoundExpr {
    match bindings[idx] {
        Binding::Computed { expr, kind } => {
            match expr {
                Expr::Mul(lhs, rhs) => {
                    BoundExpr::Mul(Box::new(BoundExpr::Placeholder), Box::new(BoundExpr::Placeholder))
                },
                _ => panic!("Unhandled expr: {:?}", expr)
            }
        },
        Binding::Deref { ptr, offset, kind } => {
            BoundExpr::Placeholder
            // TODO: resolve_dereference
        },
        Binding::InitState(builtin) => {
            BoundExpr::InitState(builtin)
        },
        other => panic!("Unhandled binding: {:?}", other)
    }
}

fn resolve_dereference(bindings: &Vec<Binding>, ptr: usize, offset: i32, kind: DataKind) {
    use data_flow::types::Binding::*;

    match (bindings[ptr], offset, kind) {
        (PtrKernarg, _, _) => {
            panic!("Kernarg {} ({:?})", offset, kind);
        },
        _ => ()
    }
}
