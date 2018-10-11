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

ENV LIBRARY_PATH=/opt/rocm/lib:/opt/rocm/hsa/lib:/opt/rocm/hip/lib:/opt/rocm/hcc/lib
