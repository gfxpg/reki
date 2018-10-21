use kernel_meta::{KernelArg, VGPRWorkItemId};
use assembly::{Disassembly, Instruction};

pub struct ExecutionState {
    pub sgprs: Vec<String>,
    pub vgprs: Vec<String>,
    pub kernel_args: Vec<KernelArg>,
    pub instrs: Vec<Instruction>
}

impl From<Disassembly> for ExecutionState {
    fn from(disassembly: Disassembly) -> Self {
        let (kcode, kernel_args, instrs) = disassembly;

        let mut sgprs: Vec<&'static str> = Vec::with_capacity(16);

        /* https://llvm.org/docs/AMDGPUUsage.html#amdgpu-amdhsa-sgpr-register-set-up-order-table */
        if kcode.code_props.enable_sgpr_private_segment_buffer {
            sgprs.extend_from_slice(&[""; 4]);
        }
        if kcode.code_props.enable_sgpr_dispatch_ptr {
            sgprs.extend_from_slice(&["AQL_DISPATCH_PACKET"; 2]);
        }
        if kcode.code_props.enable_sgpr_queue_ptr {
            sgprs.extend_from_slice(&["AMD_QUEUE_T"; 2]);
        }
        if kcode.code_props.enable_sgpr_kernarg_segment_ptr {
            sgprs.extend_from_slice(&["KERNARG"; 2]);
        }
        if kcode.code_props.enable_sgpr_dispatch_id {
            sgprs.extend_from_slice(&["DISPATCH_ID"; 2]);
        }
        if kcode.code_props.enable_sgpr_flat_scratch_init {
            sgprs.extend_from_slice(&["FLAT_SCRATCH_INIT"; 2]);
        }
        if kcode.code_props.enable_sgpr_grid_workgroup_count_x {
            sgprs.push("WORKGROUP_COUNT_X");
        }
        if kcode.code_props.enable_sgpr_grid_workgroup_count_y && sgprs.len() < 16 {
            sgprs.push("WORKGROUP_COUNT_Y");
        }
        if kcode.code_props.enable_sgpr_grid_workgroup_count_z && sgprs.len() < 16 {
            sgprs.push("WORKGROUP_COUNT_Z");
        }
        if kcode.code_props.enable_sgpr_grid_workgroup_count_z && sgprs.len() < 16 {
            sgprs.push("WORKGROUP_COUNT_Z");
        }
        if kcode.pgm_props.enable_sgpr_workgroup_id_x {
            sgprs.push("get_group_id(0)");
        }
        if kcode.pgm_props.enable_sgpr_workgroup_id_y {
            sgprs.push("get_group_id(1)");
        }
        if kcode.pgm_props.enable_sgpr_workgroup_id_z {
            sgprs.push("get_group_id(2)");
        }
        if kcode.pgm_props.enable_sgpr_workgroup_info {
            sgprs.push("WORKGROUP_INFO");
        }
        if kcode.pgm_props.enable_sgpr_private_segment_wavefront_offset {
            sgprs.push("PRIVATE_SEGMENT_WAVEFRONT_OFFSET");
        }

        /* https://llvm.org/docs/AMDGPUUsage.html#amdgpu-amdhsa-vgpr-register-set-up-order-table */
        let vgprs: Vec<&'static str> = match kcode.pgm_props.enable_vgpr_workitem_id {
            VGPRWorkItemId::X => vec!["get_local_id(0)"],
            VGPRWorkItemId::XY => vec!["get_local_id(0)", "get_local_id(1)"],
            VGPRWorkItemId::XYZ => vec!["get_local_id(0)", "get_local_id(1)", "get_local_id(2)"]
        };
        
        ExecutionState {
            sgprs: sgprs.into_iter().map(|s| s.to_string()).collect(),
            vgprs: vgprs.into_iter().map(|s| s.to_string()).collect(),
            kernel_args,
            instrs
        }
    }
}
