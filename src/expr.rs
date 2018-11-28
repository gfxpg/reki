pub type BindingIdx = usize;

#[derive(Debug, Copy, Clone)]
pub enum RegState {
    QwHi(BindingIdx),
    QwLo(BindingIdx),
    Dw(BindingIdx)
}

#[derive(Debug)]
pub enum DataSize {
    Dword, Qword
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
