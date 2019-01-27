# ROCm + Rust nightly toolchain

FROM rocm/rocm-terminal:2.0

ENV RUSTUP_HOME=/home/rocm-user/.rustup \
    CARGO_HOME=/home/rocm-user/.cargo \
    PATH=/home/rocm-user/.cargo/bin:$PATH

RUN sudo apt-get update \
 && sudo apt-get install -y wget \
 && sudo rm -rf /var/lib/apt/lists/* \
 && wget "https://static.rust-lang.org/rustup/archive/1.16.0/x86_64-unknown-linux-gnu/rustup-init" \
 && chmod +x rustup-init \
 && ./rustup-init -y --no-modify-path --default-toolchain nightly \
 && rm rustup-init \
 && chmod -R a+w $RUSTUP_HOME $CARGO_HOME \
 && rustup --version && cargo --version && rustc --version

# hexdump (bsdmainutils) is required by cloc.sh to produce disassembled listings
RUN sudo sh -c "echo 'deb http://apt.llvm.org/xenial/ llvm-toolchain-xenial-8 maindeb-src http://apt.llvm.org/xenial/ llvm-toolchain-xenial-8 main' > /etc/apt/sources.list.d/llvm.list" \
 && sudo sh -c "echo 'deb-src http://apt.llvm.org/xenial/ llvm-toolchain-xenial-8 main' >> /etc/apt/sources.list.d/llvm.list" \
 && wget -O - https://apt.llvm.org/llvm-snapshot.gpg.key | sudo apt-key add - \
 && sudo apt-get update \
 && sudo apt-get install -y bsdmainutils zlib1g-dev lld-8 llvm-8 clang-format-8 \
 && sudo rm -rf /var/lib/apt/lists/*

# Required by bin/ scripts
RUN wget https://raw.githubusercontent.com/ryanmjacobs/c/master/c \
 && sudo install -m 755 c /usr/bin/c \
 && rm ./c \
 && sudo wget https://raw.githubusercontent.com/llvm-mirror/llvm/993ef0ca960f8ffd107c33bfbf1fd603bcf5c66c/lib/Target/AMDGPU/AMDKernelCodeT.h -P /usr/local/include/ \
 && sudo sed '/#include "llvm/d;s/cstddef/stddef.h/;s/cstdint/stdint.h/' -i /usr/local/include/AMDKernelCodeT.h

RUN sudo update-alternatives --install /usr/bin/ld ld /usr/bin/lld-8 30

ENV PATH="/src/bin:${PATH}" LIBRARY_PATH=/opt/rocm/lib:/opt/rocm/hsa/lib:/opt/rocm/hip/lib:/opt/rocm/hcc/lib
