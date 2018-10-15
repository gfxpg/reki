# Studying GCN assembly

* [AMD Vega Instruction Set Architecture](http://developer.amd.com/wordpress/media/2013/12/Vega_Shader_ISA_28July2017.pdf)

A `cl_asm` utility (located in the repository root) is provided to compile
an OpenCL 2.0 source file to a GCN GFX9 assembly listing.

## OpenCL

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

Here's the beginning of the actual kernel code (I skipped the rest for reasons
that will become clear later):

```asm
s_mov_b32 s33, s9
s_load_dwordx2 s[34:35], s[4:5], 0x0
s_add_u32 flat_scratch_lo, s6, s33
s_addc_u32 flat_scratch_hi, s7, 0
s_getpc_b64 s[6:7]
s_add_u32 s6, s6, _Z13get_global_idj@rel32@lo+4
s_addc_u32 s7, s7, _Z13get_global_idj@rel32@hi+4
v_mov_b32_e32 v0, 0
s_mov_b32 s4, s33
s_mov_b32 s32, s33
s_swappc_b64 s[30:31], s[6:7]
# ...
```

The first four instructions seem to be setting up
[flat access to scratch memory](https://llvm.org/docs/AMDGPUUsage.html#memory-spaces). In short,
the memory space is flat, and depending on what range — _aperture_ in AMD's parlance —
the address falls in, it maps to global, private (scratch), or group (LDS) memory.
Private (scratch) memory accesses are mapped to physical addresses as
`wavefront-scratch-base + (private-address * wavefront-size * 4) + (wavefront-lane-id * 4)`,
and `wavefront-scratch-base` requires setup in the kernel prologue.

Next, the kernel seems to be invoking an external function (OpenCL's `get_global_id`).
It is defined in [libclc](https://github.com/llvm-mirror/libclc/blob/c45b9dfe5257f8dfec9a193c07073ee95210ecc1/generic/lib/workitem/get_global_id.cl),
and looks innocuous enough until you try to compile it:

```bash
hcc -S -x cl -cl-std=CL2.0 -target amdgcn-amd-amdhsa -mcpu=gfx900 -I../../include -c get_global_id.cl
```

...and it decomposes to four uninlined function calls.

### Does a global ID lookup really require all the trouble?

Not really, and I found that out by inspecting the disassembled listings
of the executable `.hsaco` file (obtained via `cloc.sh -mcpu gfx900 -s source.cl`).

The generated source is shorter than the snippet I got from `cl_asm`, as it
omits both flat scratch setup and the `get_global_id` call.

### Revisiting amd_kernel_code_t

The initial state of scalar general-purpose registers (SGPRs, starting with `s`)
depends on the settings in `amd_kernel_code_t`, so naturally, I'd need to know
them first before trying to make sense of the program.

The listing produced by `cl_asm` has a readable set of key-value pairs for
`amd_kernel_code_t`, but unfortunately they do not match the object produced
by `cloc.sh`. The reason for that is that the latter performs several steps
to produce the binary (run `cloc.sh -mcpu gfx900 -n source.cl` to see them),
one of which is optimization. It is responsible for inlinining our `get_global_id`
call and further simplyfing the assembly, resulting in a slightly different
initial kernel configuration.

I couldn't find a way to dump `amd_kernel_code_t` from one of the intermediate
steps, so I decided I could extract it from the `.hsaco` binary.

The object is located in the first 256 bytes of the `.text` section, and
since `.hsaco` is an ELF, the data can be extracted using standard Linux utilities:

```bash
objdump -s --section=.text binary.hsaco | head -n 20
```

A hexdump is not very helpful by itself; a lot of information there is packed into
bitfields, not really intended to be analyzed by eye. I wrote a small script
(available in `bin/print_amd_kernel_code_t`) to output a human-readable
`amd_kernel_code_t` from a `.hsaco` file.

The layout is defined in
[AMDKernelCodeT.h](https://github.com/llvm-mirror/llvm/blob/993ef0ca960f8ffd107c33bfbf1fd603bcf5c66c/lib/Target/AMDGPU/AMDKernelCodeT.h#L528).

### The program itself

Based on the initial kernel state, this is what I think the register layout looks like,
but I have yet to verify it:

```
# set up by CP, apply to all wavefronts of the grid

s0 - s3: private segment buffer (?), since enable_sgpr_private_segment_buffer = 1
s4 - s5: kernarg segment address, since enable_sgpr_kernarg_segment_ptr = 1
s6 - s7: flat scratch address (see below), since enable_sgpr_flat_scratch_init = 1

# ...
```

Now, let's try to make sense of what the kernel is doing:

```asm
s_load_dword s2, s[4:5], 0x4
s_load_dwordx2 s[0:1], s[6:7], 0x0
s_load_dword s3, s[6:7], 0x8
v_mov_b32_e32 v1, 0
s_waitcnt lgkmcnt(0)
s_and_b32 s2, s2, 0xffff
s_mul_i32 s8, s8, s2
v_add_u32_e32 v0, s8, v0
v_add_u32_e32 v2, s3, v0
v_ashrrev_i64 v[2:3], 30, v[1:2]
v_mov_b32_e32 v0, s1
v_add_co_u32_e32 v2, vcc, s0, v2
v_addc_co_u32_e32 v3, vcc, v0, v3, vcc
global_store_dword v[2:3], v1, off
s_endpgm
```

[To be continued...](https://youtu.be/cPCLFtxpadE)