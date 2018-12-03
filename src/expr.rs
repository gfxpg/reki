pub type BindingIdx = usize;
pub type DwordIdx = u8;

#[derive(Debug, Copy, Clone)]
pub struct Reg(pub BindingIdx, pub DwordIdx);

#[derive(Debug, Copy, Clone)]
pub enum DataKind {
    Dword, Qword, DQword, U16, I64
}

#[derive(Debug, Copy, Clone)]
pub enum Binding {
    U32(u32),
    I32(i32),
    Deref { ptr: BindingIdx, offset: u32, kind: DataKind },
    Computed { expr: Expr, kind: DataKind },
    DwordElement { of: BindingIdx, dword: u8 },
    QwordElement { of: BindingIdx, dword: u8 },
    Cast { source: BindingIdx, kind: DataKind },

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
    And(BindingIdx, u32),
    Shl(BindingIdx, BindingIdx)
}

#[derive(Debug)]
pub enum Statement {
    JumpIf { cond: Condition, instr_offset: i16 }
}

#[derive(Debug, Copy, Clone)]
pub enum Condition {
    Lt(BindingIdx, BindingIdx)
}
