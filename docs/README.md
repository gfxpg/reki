# Adventures in GCN

* [AMD Vega Instruction Set Architecture](http://developer.amd.com/wordpress/media/2013/12/Vega_Shader_ISA_28July2017.pdf)

Custom scripts referenced throughout this text are located in the `bin/`
directory and are included in `$PATH` inside the development image.

## Compiling an OpenCL kernel

Let's begin with a simple kernel that zeroes its output buffer:

```opencl
__kernel void zeroify(__global float* a) {
  const int i = get_global_id(0);
  a[i] = 0;
}
```

An AMDGCN GFX9 assembly listing can be generated from the OpenCL source by
running `cl_asm source.cl`.

The first notable thing there is the `.amd_kernel_code_t` directive,
which specifies key-value pairs to emit the kernel code object.
According to [LLVM docs](https://llvm.org/docs/AMDGPUUsage.html#amd-kernel-code-t),

> it must be placed immediately after the function label and before any instructions.

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

Next, the kernel invokes an external function (OpenCL's `get_global_id`),
defined in [libclc](https://github.com/llvm-mirror/libclc/blob/c45b9dfe5257f8dfec9a193c07073ee95210ecc1/generic/lib/workitem/get_global_id.cl).

While we can compile it separately:

```bash
hcc -S -x cl -cl-std=CL2.0 -target amdgcn-amd-amdhsa -mcpu=gfx900 -I../../include -c get_global_id.cl
```

...all it consists of is four uninlined function calls.

### _De_compiling an OpenCL kernel

Does a workitem's ID lookup really require all the trouble? Not really.
`get_global_id` is always inlined, but only during a later compiler pass.
To see the kernel as it is actually executed, we'll have to disassemble
the compiled (`.hsaco`) version. This can be done by running `cloc -s source.cl`,
which outputs a _source.s_ file with the code.

The generated source is shorter than the snippet produced by `cl_asm`:
it omits both flat scratch setup and function calls.

### Revisiting amd_kernel_code_t

Before trying to make sense of the program, we'll have to know the initial
state of scalar general-purpose registers (SGPRs, starting with `s`).
The state is specified by `amd_kernel_code_t`, but LLVM does not disassemble
it back into a readable set of key-value pairs (like in the listing produced
by `cl_asm`).

The `amd_kernel_code_t` object is located in the first 256 bytes of the `.text`
section, and since `.hsaco` is an ELF, the data can be extracted using standard
Linux utilities:

```bash
objdump -s --section=.text binary.hsaco | head -n 20
```

A hexdump is not very helpful by itself, though. A lot of information there is
packed into bitfields, not really intended to be analyzed by eye.
I wrote a small script (`print_amd_kernel_code_t`) to output a human-readable
list of parameters from a `.hsaco` file.

### Analyzing the program itself

Based on those parameters, this is what I think the register layout looks like:

```
s0 - s3: private segment buffer (?), since enable_sgpr_private_segment_buffer = 1
s4 - s5: address of the dispatch packet (see below), since enable_sgpr_dispatch_ptr = 1
s6 - s7: kernarg segment address, since enable_sgpr_kernarg_segment_ptr = 1
s8: 32-bit workgroup id in X dimension, since enable_sgpr_workgroup_id_x = 1

v0: 32-bit workitem id in X dimension of workgroup, always present
```

Now, let's try to make sense of what the kernel is doing:

```asm
; http://www.hsafoundation.com/html/Content/Runtime/Topics/02_Core/hsa_kernel_dispatch_packet_t.htm
; s2 = s[4:5] + 0x4 (hsa_kernel_dispatch_packet_t.workgroup_size_x)
s_load_dword s2, s[4:5], 0x4
; s[0:1] = kernarg[0] (the output buffer pointer)
s_load_dwordx2 s[0:1], s[6:7], 0x0
; s3 = HiddenGlobalOffsetX, OpenCL's offset used to calculate the global ID of a workitem
s_load_dword s3, s[6:7], 0x8
; v1 = 0
v_mov_b32_e32 v1, 0
s_waitcnt lgkmcnt(0)
; workgroup_size_x &= 0xffff (? the lower 16 bits ?)
s_and_b32 s2, s2, 0xffff
; s8 (workgroup id) = workgroup id * workgroup_size
s_mul_i32 s8, s8, s2
; v0 (workitem id inside workgroup) = workitem id + (workitem id * workgroup size)
v_add_u32_e32 v0, s8, v0
; v2 = global offset + v0 = global id
v_add_u32_e32 v2, s3, v0
; v[2:3] = (global id extended to i64) * 4 (sizeof(float)) = buffer index
; see below for a more detailed explanation
v_ashrrev_i64 v[2:3], 30, v[1:2]
; v0 = s1 (higher dword of output buffer ptr?)
v_mov_b32_e32 v0, s1
; v2 = (lower dword of buffer ptr) + (lower dword of buffer index)
; carry flag is stored in vcc
v_add_co_u32_e32 v2, vcc, s0, v2
; v3 = (higher dword of buffer ptr) + (higher dword of buffer index) + carry
v_addc_co_u32_e32 v3, vcc, v0, v3, vcc
; *(v[2:3]) = v1 (zero)
global_store_dword v[2:3], v1, off
s_endpgm
```

This listing looks much less intimidating when we know the initial register state.
I'd like to highlight `v_ashrrev_i64 v[2:3], 30, v[1:2]`, because I was impressed by
how clever it is:

```c
typedef union { uint64_t qw; uint32_t dws[2]; } ashr_i64;
ashr_i64 v12;

v12.dws[0] = 0; // v1 is assigned zero (4th instruction from the beginning)
v12.dws[1] = 7; // v2 is a 32-bit global id (7 for the sake of example)
printf("v[1:2] >> 32 = %lu; v[1:2] >> 30 = %lu\n", v12.qw >> 32, v12.qw >> 30);
// "v[1:2] >> 32 = 7; v[1:2] >> 30 = 28"
```

It essentially packs two operations, extending a 32-bit value to 64-bits and
converting it to an array pointer offset, into one instruction.

[To be continued...](https://youtu.be/cPCLFtxpadE)
