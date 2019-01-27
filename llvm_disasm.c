#include <stdio.h>

#include <llvm-c/Target.h>
#include <llvm-c/Disassembler.h>

void llvm_disasm() {
  printf("h from llvm_diasm!");

  LLVMInitializeAMDGPUTargetInfo();
  LLVMInitializeAMDGPUTargetMC();
  LLVMInitializeAMDGPUDisassembler();
}
