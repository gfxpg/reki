# Documentation

* [AMD Vega Instruction Set Architecture](http://developer.amd.com/wordpress/media/2013/12/Vega_Shader_ISA_28July2017.pdf)

## Studying GCN assembly

A `cl_asm` utility (located in the repository root) is provided to compile
an OpenCL 2.0 source file to a GCN GFX9 assembly listing.

### get_global_id

I'll start with a simple kernel that fills its output buffer with 0s:

```opencl
__kernel void zeroify(__global float* a) {
  const int i = get_global_id(0);
  a[i] = 0;
}
```

The first notable thing in the assembly listing is the `.amd_kernel_code_t`
directive, which specifies key-value pairs to emit the kernel code object.
According to [LLVM docs](https://llvm.org/docs/AMDGPUUsage.html#amd-kernel-code-t),

> [it] must be placed immediately after the function label and before any instructions.

The entries are described both in the
[LLVM user guide](https://llvm.org/docs/AMDGPUUsage.html#kernel-descriptor-for-gfx6-gfx9) and the
[AMDGPU ABI documentation](https://github.com/ROCm-Developer-Tools/ROCm-ComputeABI-Doc/blob/master/AMDGPU-ABI.md#amd-kernel-code).

I won't go over each one yet, and move on to the the actual program for now:

```asm
s_mov_b32 s33, s9
s_mov_b32 s32, s33
s_mov_b32 flat_scratch_lo, s7
s_add_u32 s6, s6, s33
s_lshr_b32 flat_scratch_hi, s6, 8
s_load_dwordx2 s[34:35], s[4:5], 0x0
s_getpc_b64 s[6:7]
s_add_u32 s6, s6, _Z13get_global_idj@rel32@lo+4
s_addc_u32 s7, s7, _Z13get_global_idj@rel32@hi+4
v_mov_b32_e32 v0, 0
s_mov_b32 s4, s33
s_swappc_b64 s[30:31], s[6:7]
v_mov_b32_e32 v1, 0
v_mov_b32_e32 v2, v0
s_waitcnt lgkmcnt(0)
v_mov_b32_e32 v0, s35
v_ashr_i64 v[2:3], v[1:2], 30
v_add_i32_e32 v2, vcc, s34, v2
v_addc_u32_e32 v3, vcc, v0, v3, vcc
flat_store_dword v[2:3], v1
s_endpgm
```

The initial state of scalar general-purpose registers (SGPRs, starting with `s`)
depends on the settings in `amd_kernel_code_t`. This is what I think the layout
looks like, but I have yet to verify it:

```
# set up by CP, apply to all wavefronts of the grid

s0 - s3: private segment buffer (?), since enable_sgpr_private_segment_buffer = 1
s4 - s5: kernarg segment address, since enable_sgpr_kernarg_segment_ptr = 1
s6 - s7: flat scratch address (see below), since enable_sgpr_flat_scratch_init = 1
s8: private segment size (?)

# system sgprs, can have different values for each wavefront

s9: 32 bit work-group id in X dimension of grid, since enable_sgpr_workgroup_id_x = 1
```

Regarding the first five instructions, this is what my search
[turned up](https://llvm.org/docs/AMDGPUUsage.html#memory-spaces):

Private memory accesses are mapped to physical addresses as
`wavefront-scratch-base + (private-address * wavefront-size * 4) + (wavefront-lane-id * 4)`.
The memory space is flat, and depending on what range — _aperture_ in AMD's parlance —
the address falls in, it maps to global, private (scratch), or group (LDS) memory.
Flat access to scratch requires setup in the kernel prologue (`wavefront-scratch-base`),
and this seems to be what the first five instructions do.

Next, the kernel seems to be invoking an external function (OpenCL's `get_global_id`).

[To be continued...](https://youtu.be/cPCLFtxpadE)
