# ROCm + Rust nightly toolchain

FROM rocm/rocm-terminal:latest

ENV RUSTUP_HOME=/home/rocm-user/.rustup \
    CARGO_HOME=/home/rocm-user/.cargo \
    PATH=/home/rocm-user/.cargo/bin:$PATH

RUN sudo apt-get update \
 && sudo apt-get install -y wget \
 && sudo rm -rf /var/lib/apt/lists/* \
 && wget "https://static.rust-lang.org/rustup/archive/1.13.0/x86_64-unknown-linux-gnu/rustup-init" \
 && chmod +x rustup-init \
 && ./rustup-init -y --no-modify-path --default-toolchain nightly \
 && rm rustup-init \
 && chmod -R a+w $RUSTUP_HOME $CARGO_HOME \
 && rustup --version && cargo --version && rustc --version

RUN wget https://github.com/ROCm-Developer-Tools/hcc2/releases/download/rel_0.5-3/hcc2_0.5-3_amd64.deb \
 && sudo dpkg -i hcc2_0.5-3_amd64.deb \
 && rm hcc2_0.5-3_amd64.deb

# hexdump (bsdmainutils) is required by cloc.sh to produce disassembled listings
RUN sudo apt-get update \
 && sudo apt-get install -y bsdmainutils \
 && sudo rm -rf /var/lib/apt/lists/*

# Run C source files as CLI scripts
RUN wget https://raw.githubusercontent.com/ryanmjacobs/c/master/c \
 && sudo install -m 755 c /usr/bin/c \
 && rm ./c

# Required for amd_kernel_code_t printing 
RUN sudo wget https://raw.githubusercontent.com/llvm-mirror/llvm/993ef0ca960f8ffd107c33bfbf1fd603bcf5c66c/lib/Target/AMDGPU/AMDKernelCodeT.h -P /usr/local/include/ \
 && sudo sed '/#include "llvm/d;s/cstddef/stddef.h/;s/cstdint/stdint.h/' -i /usr/local/include/AMDKernelCodeT.h

ENV CC=gcc

ENV LIBRARY_PATH=/opt/rocm/lib:/opt/rocm/hsa/lib:/opt/rocm/hip/lib:/opt/rocm/hcc/lib
