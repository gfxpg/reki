use kernel_meta::{KernelCode, VGPRWorkItemId};
use expr::{Binding, Reg};

pub struct ExecutionState {
    pub sgprs: Vec<Reg>,
    pub vgprs: Vec<Reg>,
    pub bindings: Vec<Binding>
}

use std::fmt;

impl fmt::Debug for ExecutionState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bindings:\n")?;
        for (i, ref binding) in self.bindings.iter().enumerate() {
            write!(f, "{:4} {:?}\n", i, binding)?;
        }
        write!(f, "SGPRS: {:?}\n", self.sgprs.iter().enumerate().collect::<Vec<(usize, &Reg)>>())?;
        write!(f, "VGPRS: {:?}", self.vgprs.iter().enumerate().collect::<Vec<(usize, &Reg)>>())
    }
}

macro_rules! bind_init_state {
    (qword $val:expr, $bindings:expr, $regfile:expr) => {
        $bindings.push($val);
        $regfile.push(Reg($bindings.len() - 1, 0));
        $regfile.push(Reg($bindings.len() - 1, 1));
    };
    (dword $val:expr, $bindings:expr, $regfile:expr) => {
        $bindings.push($val);
        $regfile.push(Reg($bindings.len() - 1, 0));
    }
}

impl From<KernelCode> for ExecutionState {
    fn from(kcode: KernelCode) -> Self {
        let mut sgprs: Vec<Reg> = Vec::with_capacity(16);
        let mut bindings: Vec<Binding> = Vec::with_capacity(16);

        /* https://llvm.org/docs/AMDGPUUsage.html#amdgpu-amdhsa-sgpr-register-set-up-order-table */
        if kcode.code_props.enable_sgpr_private_segment_buffer {
            bindings.push(Binding::PrivateSegmentBuffer);
            for i in 0..4 { sgprs.push(Reg(bindings.len() - 1, i)); }
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
        let vgprs: Vec<Reg> = match kcode.pgm_props.enable_vgpr_workitem_id {
            VGPRWorkItemId::X => {
                bindings.push(Binding::WorkitemIdX);
                vec![Reg(bindings.len() - 1, 0)]
            },
            VGPRWorkItemId::XY => {
                bindings.push(Binding::WorkitemIdX);
                bindings.push(Binding::WorkitemIdY);
                vec![Reg(bindings.len() - 2, 0), Reg(bindings.len() - 1, 0)]
            },
            VGPRWorkItemId::XYZ => {
                bindings.push(Binding::WorkitemIdX);
                bindings.push(Binding::WorkitemIdY);
                bindings.push(Binding::WorkitemIdZ);
                vec![Reg(bindings.len() - 3, 0), Reg(bindings.len() - 2, 0), Reg(bindings.len() - 1, 0)]
            }
        };
        
        ExecutionState { sgprs, vgprs, bindings }
    }
}
