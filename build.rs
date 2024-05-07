use std::env;
use std::fs;
use std::io;
use std::path::Path;

fn main() -> io::Result<()> {
    println!("cargo:rerun-if-changed=src/tests");

    println!("cargo:rustc-check-cfg=cfg(trybuild_no_target)");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let target = env::var("TARGET").ok();
    let path = Path::new(&out_dir).join("target");
    let value = match target {
        Some(target) => format!(r#"Some("{}")"#, target.escape_debug()),
        None => "None".to_owned(),
    };
    fs::write(path, value)
}
