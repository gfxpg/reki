# Control Flow

Before delving into conditional execution, it helps to understand the
high-level concepts that underpin AMDGCN architecture.

The GPU groups 64 workitems into a _wavefront_ — a group of SIMD threads that is
executed simultaneously. The last bit is imporant; there is only one
_Program Counter_ per wavefront.

The processor contains a _scalar_ and a _vector_ ALUs. The _scalar_ ALU operates
on one value per wavefront (shared between all 64 threads), while the _vector_ ALU
operates on unique values per each thread.

To support conditional execution, a wavefront has a 64-bit thread execution mask (`EXEC`).
Threads that are active (`EXEC[thread_index] = 1`) execute vector instructons (`v_` as well
as vector-memory loads/stores), while inactive threads (`EXEC[thread_index] = 0`) perform a NOP.

Control flow is thus handled using _scalar_ ALU instructions that modify the `EXEC` mask.

## Examples

Only the relevant parts of AMDGCN assembly are presented. Full listings
can be obtained by running `cloc -s source.cl` inside the development container.

### Ternary expressions

```opencl
__kernel void cond_expression(__global int* a) {
  const int i = get_global_id(0);
  a[i] = i == 3 ? 5 : 7;
}
```

The ternary expression above (and the equivalent `if (..) a[i] = 5; else a[i] = 7;`
construct) gets compiled to two instructions. They operate on `vcc` (Vector Condition Code),
a bit vector with one bit per executing thread. For each thread `t`, `v_cmp_eq`
sets the `vcc[t]` bit to `v2 == 3`, then `v_cndmask` computes `v3 = vcc[t] ? 5 : 7`:


```asm
; v2 = get_global_id(0)
v_cmp_eq_u32_e32 vcc, 3, v2
v_cndmask_b32_e64 v3, 7, 5, vcc
```

### Conditional branches

```opencl
__kernel void cond_branch(__global int* a) {
  const int i = get_global_id(0);
  if (i == 3)
    a[0] = a[i] + 7;
  else
    a[i] = i;
}
```

```asm
; v2 = i
; vcc = v2 != 3 (note the branch reordering)
v_cmp_ne_u32_e32 vcc, 3, v2
; s0:1 = old exec mask, exec &= vcc
; v_ instructions will now be executed only for threads with v2 != 3
s_and_saveexec_b64 s[0:1], vcc
; s0:1 = old exec mask ^ vcc (inactive threads)
s_xor_b64 s[0:1], exec, s[0:1]
; store a[i] = i
global_store_dword v[0:1], v2, off
; exec |= s0:1 (restore full exec)
; s0:1 = old exec (if branch)
s_or_saveexec_b64 s[0:1], s[0:1]
; exec ^ s0:1 (if branch) = else branch
s_xor_b64 exec, exec, s[0:1]
; v0 = a[i]
global_load_dword v0, v[0:1], off
s_waitcnt vmcnt(0)
; v2 = a[i] + 7
v_add_u32_e32 v2, 7, v0
; restore (a + i)
v_mov_b32_e32 v0, s2
v_mov_b32_e32 v1, s3
; *(a + i) = v2
global_store_dword v[0:1], v2, off
```
