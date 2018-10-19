use kernel_code_object::{KernelCode, VGPRWorkItemId};

pub fn initial_sgprs(kcode: &KernelCode) -> Vec<&'static str> {
    let mut sgprs: Vec<&'static str> = Vec::with_capacity(16);

    /* https://llvm.org/docs/AMDGPUUsage.html#amdgpu-amdhsa-sgpr-register-set-up-order-table */
    if kcode.code_props.enable_sgpr_private_segment_buffer {
        /* Four SGPRs to access private memory space */
        sgprs.push("");
        sgprs.push("");
        sgprs.push("");
        sgprs.push("");
    }
    if kcode.code_props.enable_sgpr_dispatch_ptr {
        sgprs.push("AQL_DISPATCH_PACKET_LO");
        sgprs.push("AQL_DISPATCH_PACKET_HI");
    }
    if kcode.code_props.enable_sgpr_queue_ptr {
        sgprs.push("AMD_QUEUE_T_LO");
        sgprs.push("AMD_QUEUE_T_HI");
    }
    if kcode.code_props.enable_sgpr_kernarg_segment_ptr {
        sgprs.push("KERNARG_LO");
        sgprs.push("KERNARG_HI");
    }
    if kcode.code_props.enable_sgpr_dispatch_id {
        sgprs.push("DISPATCH_ID_LO");
        sgprs.push("DISPATCH_ID_HI");
    }
    if kcode.code_props.enable_sgpr_flat_scratch_init {
        sgprs.push("FLAT_SCRATCH_INIT_LO");
        sgprs.push("FLAT_SCRATCH_INIT_HI");
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
        sgprs.push("WORKGROUP_ID_X");
    }
    if kcode.pgm_props.enable_sgpr_workgroup_id_y {
        sgprs.push("WORKGROUP_ID_Y");
    }
    if kcode.pgm_props.enable_sgpr_workgroup_id_z {
        sgprs.push("WORKGROUP_ID_Z");
    }
    if kcode.pgm_props.enable_sgpr_workgroup_info {
        sgprs.push("WORKGROUP_INFO");
    }
    if kcode.pgm_props.enable_sgpr_private_segment_wavefront_offset {
        sgprs.push("PRIVATE_SEGMENT_WAVEFRONT_OFFSET");
    }

    sgprs
}

pub fn initial_vgprs(kcode: &KernelCode) -> Vec<&'static str> {
    /* https://llvm.org/docs/AMDGPUUsage.html#amdgpu-amdhsa-vgpr-register-set-up-order-table */
    match kcode.pgm_props.enable_vgpr_workitem_id {
        VGPRWorkItemId::X => vec!["WORKITEM_ID_X"],
        VGPRWorkItemId::XY => vec!["WORKITEM_ID_X", "WORKITEM_ID_Y"],
        VGPRWorkItemId::XYZ => vec!["WORKITEM_ID_X", "WORKITEM_ID_Y", "WORKITEM_ID_Z"]
    }
}
