pub type BindingIdx = usize;
pub type DwordIdx = u8;
pub type AsmInstructionIdx = usize;

pub type Program = Vec<(AsmInstructionIdx, Statement)>;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Reg(pub BindingIdx, pub DwordIdx);

#[derive(Debug, Copy, Clone)]
pub enum DataKind {
    Dword, Qword, DQword, U16, I64
}

#[derive(Debug, Copy, Clone)]
pub enum Variable {
    Dword, Qword, DQword, PartialQword, PartialDQword
}

#[derive(Debug, Copy, Clone)]
pub enum Binding {
    U32(u32),
    I32(i32),
    Deref { ptr: BindingIdx, offset: i32, kind: DataKind },
    Computed { expr: Expr, kind: DataKind },
    DwordElement { of: BindingIdx, dword: u8 },
    QwordElement { of: BindingIdx, dword: u8 },
    Cast { source: BindingIdx, kind: DataKind },
    Variable { idx: usize },

    /* Built-ins (initial register state) */
    PrivateSegmentBuffer,
    PtrDispatchPacket,
    PtrQueue,
    PtrKernarg,
    DispatchId,
    FlatScratchInit,
    WorkgroupCountX,
    WorkgroupCountY,
    WorkgroupCountZ,
    WorkgroupIdX,
    WorkgroupIdY,
    WorkgroupIdZ,
    WorkgroupInfo,
    PrivateSegmentWavefrontOffset,
    WorkitemIdX,
    WorkitemIdY,
    WorkitemIdZ
}

#[derive(Debug, Copy, Clone)]
pub enum Expr {
    Mul(BindingIdx, BindingIdx),
    Add(BindingIdx, BindingIdx),
    And(BindingIdx, BindingIdx),
    Shl(BindingIdx, BindingIdx),
    AddHiLo { hi_op1: BindingIdx, hi_op2: BindingIdx, lo_op1: BindingIdx, lo_op2: BindingIdx }
}

#[derive(Debug)]
pub enum Statement {
    JumpIf { cond: Condition, label_idx: usize },
    JumpUnless { cond: Condition, label_idx: usize },
    Store { addr: BindingIdx, data: BindingIdx, kind: DataKind },
    Label { index: usize },
    VarDecl { var_idx: usize },
    DwordVarAssignment { var_idx: usize, binding_idx: BindingIdx, binding_dword: u8, var_dword: u8 },
    QwordVarAssignment { var_idx: usize, binding_idx: BindingIdx, binding_dword: u8, var_dword: u8 },
    DQwordVarAssignment { var_idx: usize, binding_idx: BindingIdx, binding_dword: u8, var_dword: u8 }
}

#[derive(Debug, Copy, Clone)]
pub enum Condition {
    Lt(BindingIdx, BindingIdx),
    Eql(BindingIdx, BindingIdx)
}
