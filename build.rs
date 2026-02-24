use bindgen::Builder;
use std::{env::var, path::PathBuf};

const LIB: &str = "libamd_smi";
const HEADER: &str = "include/amdsmi.h";

fn main() {
    if var("DOCS_RS").is_ok() {
        return;
    }

    let out_path = PathBuf::from(var("OUT_DIR").unwrap());

    Builder::default()
        .header(HEADER)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .dynamic_library_name(LIB)
        .generate()
        .expect("bindgen failed")
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Failed to write bindings");
}
