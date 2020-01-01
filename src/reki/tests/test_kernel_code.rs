use reki::CodeObject;

pub fn fixture(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn it_parses_kernel_code_t() {
    let co_bytes = std::fs::read(fixture("kernel_code.co")).unwrap();
    let co = CodeObject::new(co_bytes).unwrap();
}
