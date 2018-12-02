pub type BindingIdx = usize;
pub type DwordIdx = u8;

#[derive(Debug, Copy, Clone)]
pub struct Reg(pub BindingIdx, pub DwordIdx);

#[derive(Debug)]
pub enum DataKind {
    Dword, Qword, DQword, U16
}

#[derive(Debug)]
pub enum Binding {
    U32(u32),
    Deref { ptr: BindingIdx, offset: u32, kind: DataKind },
    Computed { expr: Expr, kind: DataKind },

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

#[derive(Debug)]
pub enum Expr {
    Mul(BindingIdx, BindingIdx)
}
