use std::path::{Path, PathBuf};

use bindgen::{Builder, EnumVariation, callbacks::ParseCallbacks};
use clap::Parser;

const LIB: &str = "libamd_smi";

fn main() {
    let args = Args::parse();

    let mut builder = Builder::default()
        .header(args.input_header.to_str().unwrap())
        .parse_callbacks(Box::new(DocFix))
        .dynamic_library_name(LIB)
        .default_enum_style(EnumVariation::NewType {
            is_bitfield: false,
            is_global: false,
        });
    builder = parse_whitelist(&args.whitelist, builder);
    builder
        .generate()
        .expect("failed to generate the bindings")
        .write_to_file(args.output)
        .expect("failed to write the bindings");
}

#[derive(Debug)]
struct DocFix;

impl ParseCallbacks for DocFix {
    fn process_comment(&self, comment: &str) -> Option<String> {
        // Transform C/C++ documentation to avoid Rust doc-test errors
        Some(format!("```text\n{comment}\n```"))
    }
}

fn parse_whitelist(path: &Path, mut builder: bindgen::Builder) -> bindgen::Builder {
    let content = std::fs::read_to_string(path).expect("failed to read whitelist");
    for line in content.lines() {
        let line = line.trim_ascii();
        if !line.is_empty() && !line.starts_with("#") {
            builder = builder.allowlist_item(line);
        }
    }
    builder
}

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    input_header: PathBuf,

    #[arg(short, long, default_value = "bindings-generator/input/whitelist.txt")]
    whitelist: PathBuf,

    #[arg(
        short,
        long,
        default_value = "amd-smi-wrapper-sys/src/versions/latest.rs"
    )]
    output: PathBuf,
}
