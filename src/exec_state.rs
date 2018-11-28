use kernel_meta::{KernelArg, VGPRWorkItemId};
use assembly::{Disassembly, Instruction};
use expr::{Binding, RegState};

pub struct ExecutionState {
    pub sgprs: Vec<RegState>,
    pub vgprs: Vec<RegState>,
    pub bindings: Vec<Binding>,
    pub kernel_args: Vec<KernelArg>,
    pub instrs: Vec<Instruction>
}

macro_rules! bind_init_state {
    (qword $val:expr, $bindings:expr, $regfile:expr) => {
        $bindings.push($val);
        $regfile.push(RegState::QwLo($bindings.len() - 1));
        $regfile.push(RegState::QwHi($bindings.len() - 1));
    };
    (dword $val:expr, $bindings:expr, $regfile:expr) => {
        $bindings.push($val);
        $regfile.push(RegState::Dw($bindings.len() - 1));
    }
}

impl From<Disassembly> for ExecutionState {
    fn from(disassembly: Disassembly) -> Self {
        let (kcode, kernel_args, instrs) = disassembly;

        let mut sgprs: Vec<RegState> = Vec::with_capacity(16);
        let mut bindings: Vec<Binding> = Vec::with_capacity(16);

        /* https://llvm.org/docs/AMDGPUUsage.html#amdgpu-amdhsa-sgpr-register-set-up-order-table */
        if kcode.code_props.enable_sgpr_private_segment_buffer {
            bindings.push(Binding::PrivateSegmentBuffer);
            for _ in 1..=4 { sgprs.push(RegState::Dw(bindings.len() - 1)); }
        }
        if kcode.code_props.enable_sgpr_dispatch_ptr {
            bind_init_state!(qword Binding::PtrDispatchPacket, bindings, sgprs);
        }
        if kcode.code_props.enable_sgpr_queue_ptr {
            bind_init_state!(qword Binding::PtrQueue, bindings, sgprs);
        }
        if kcode.code_props.enable_sgpr_kernarg_segment_ptr {
            bind_init_state!(qword Binding::PtrKernarg, bindings, sgprs);
        }
        if kcode.code_props.enable_sgpr_dispatch_id {
            bind_init_state!(qword Binding::DispatchId, bindings, sgprs);
        }
        if kcode.code_props.enable_sgpr_flat_scratch_init {
            bind_init_state!(qword Binding::FlatScratchInit, bindings, sgprs);
        }
        if kcode.code_props.enable_sgpr_grid_workgroup_count_x {
            bind_init_state!(dword Binding::WorkgroupCountX, bindings, sgprs);
        }
        if kcode.code_props.enable_sgpr_grid_workgroup_count_y && sgprs.len() < 16 {
            bind_init_state!(dword Binding::WorkgroupCountY, bindings, sgprs);
        }
        if kcode.code_props.enable_sgpr_grid_workgroup_count_z && sgprs.len() < 16 {
            bind_init_state!(dword Binding::WorkgroupCountZ, bindings, sgprs);
        }
        if kcode.pgm_props.enable_sgpr_workgroup_id_x {
            bind_init_state!(dword Binding::WorkgroupIdX, bindings, sgprs);
        }
        if kcode.pgm_props.enable_sgpr_workgroup_id_y {
            bind_init_state!(dword Binding::WorkgroupIdY, bindings, sgprs);
        }
        if kcode.pgm_props.enable_sgpr_workgroup_id_z {
            bind_init_state!(dword Binding::WorkgroupIdZ, bindings, sgprs);
        }
        if kcode.pgm_props.enable_sgpr_workgroup_info {
            bind_init_state!(dword Binding::WorkgroupInfo, bindings, sgprs);
        }
        if kcode.pgm_props.enable_sgpr_private_segment_wavefront_offset {
            bind_init_state!(dword Binding::PrivateSegmentWavefrontOffset, bindings, sgprs);
        }

        /* https://llvm.org/docs/AMDGPUUsage.html#amdgpu-amdhsa-vgpr-register-set-up-order-table */
        let vgprs: Vec<RegState> = match kcode.pgm_props.enable_vgpr_workitem_id {
            VGPRWorkItemId::X => {
                bindings.push(Binding::WorkitemIdX);
                vec![RegState::Dw(bindings.len() - 1)]
            },
            VGPRWorkItemId::XY => {
                bindings.push(Binding::WorkitemIdX);
                bindings.push(Binding::WorkitemIdY);
                vec![RegState::Dw(bindings.len() - 2), RegState::Dw(bindings.len() - 1)]
            },
            VGPRWorkItemId::XYZ => {
                bindings.push(Binding::WorkitemIdX);
                bindings.push(Binding::WorkitemIdY);
                bindings.push(Binding::WorkitemIdZ);
                vec![RegState::Dw(bindings.len() - 3), RegState::Dw(bindings.len() - 2), RegState::Dw(bindings.len() - 1)]
            }
        };
        
        ExecutionState { sgprs, vgprs, kernel_args, bindings, instrs }
    }
}
