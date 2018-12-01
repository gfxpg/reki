pub type BindingIdx = usize;
pub type DwordIdx = u8;

#[derive(Debug, Copy, Clone)]
pub struct Reg(pub BindingIdx, pub DwordIdx);

#[derive(Debug)]
pub enum DataSize {
    Dword, Qword, DQword
}

#[derive(Debug)]
pub enum Binding {
    U32(u32),
    Deref { ptr: BindingIdx, offset: u32, size: DataSize },
    Computed { expr: Expr, size: DataSize },

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
