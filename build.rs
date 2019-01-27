extern crate cc;

const LLVM_CONFIG: &'static str = "llvm-config-8";

fn llvm_config(args: &[&str]) -> std::io::Result<String> {
    std::process::Command::new(LLVM_CONFIG)
        .args(args)
        .output()
        .map(|output| String::from_utf8(output.stdout).unwrap())
}

fn extract_libs_from_flags(libflags: &str) -> Vec<&str> {
    libflags[2..].trim_end().split(" -l").collect::<Vec<_>>()
}

fn main() {
    let libdir = llvm_config(&["--libdir"]).unwrap();
    println!("cargo:libdir={}", libdir);
    println!("cargo:rustc-link-search=native={}", libdir);

    let static_libs_str = llvm_config(&["--link-static", "--libs", "amdgpu"]).unwrap();
    for lib in extract_libs_from_flags(&static_libs_str) {
        println!("cargo:rustc-link-lib=static={}", lib);
    }

    let dyn_libs_str = llvm_config(&["--link-static", "--system-libs"]).unwrap();
    for lib in extract_libs_from_flags(&dyn_libs_str) {
        println!("cargo:rustc-link-lib=dylib={}", lib);
    }

    println!("cargo:rustc-link-lib=dylib=stdc++");

    let cflags_str = llvm_config(&["--cflags"]).unwrap();
    let cflags = &cflags_str[1..].split(" -")
        .map(|flag| format!("-{}", flag.trim_end()))
        .collect::<Vec<_>>();

    let mut cc = cc::Build::new();

    for flag in cflags {
        cc.flag(&flag);
    }

    cc.file("llvm_disasm.c").compile("libllvm_disasm");
}
