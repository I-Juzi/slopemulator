// The DLL must import only system DLLs. On the MinGW target, Rust unwinding
// would import libgcc_s_dw2-1.dll, so link libgcc (and the C runtime helpers)
// statically into the cdylib.
fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.ends_with("windows-gnu") {
        println!("cargo:rustc-link-arg-cdylib=-static-libgcc");
        println!("cargo:rustc-link-arg-cdylib=-static");
    }
    println!("cargo:rerun-if-changed=build.rs");
}
