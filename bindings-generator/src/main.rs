use clap::Parser;

use crate::{
    bindings::{ItemFilter, ItemSet},
    ch_hist_report::JsonHistSummaryReport,
};

mod bindings;
mod ch_hist_report;
mod cli;

fn main() {
    let args = cli::Cli::parse();
    match args.command {
        cli::Command::Single(args) => run_single(args),
        cli::Command::History(args) => run_history(args),
    }
}

fn run_single(args: cli::Single) {
    let whitelist = ItemSet::parse(&args.whitelist);
    let filter = ItemFilter {
        whitelist,
        blacklist: ItemSet::default(),
    };
    bindings::generate_bindings(&args.input_header, &filter, &args.output);
}

const ALWAYS_EMIT_IN_PARTIAL_MODULES: &[&'static str] = &[
    "AMDSMI_LIB_VERSION_YEAR",
    "AMDSMI_LIB_VERSION_MAJOR",
    "AMDSMI_LIB_VERSION_MINOR",
    "AMDSMI_LIB_VERSION_RELEASE",
];

fn run_history(args: cli::History) {
    fn version_to_module_name(v: &str) -> String {
        format!("v{}", v.replace(".", "_"))
    }

    use std::fmt::Write;

    // parse the report
    let file_content =
        std::fs::read_to_string(args.input_report).expect("failed to read report file");
    let report: JsonHistSummaryReport =
        serde_json::from_str(&file_content).expect("failed to parse report file");
    let first_header = report.changes_per_version[0].input_old.as_path();
    let first_version = report.changes_per_version[0].version_old.as_str();

    // remove the old output
    for file in std::fs::read_dir(&args.output_dir).unwrap() {
        std::fs::remove_file(file.unwrap().path()).unwrap();
    }

    // build the mod.rs bit by bit
    let mut modules = Vec::new();

    // generate the stable API
    {
        let stable_output = args.output_dir.join("stable.rs");

        let whitelist = ItemSet::from_iter(&report.stable);
        let mut blacklist = ItemSet::from_iter(&report.unstable.breaking);
        blacklist.extend(&report.unstable.dubious);
        let filter = ItemFilter {
            whitelist,
            blacklist,
        };
        bindings::generate_bindings(first_header, &filter, &stable_output);
        modules.push(Module {
            name: String::from("stable"),
            comment: vec![String::from("Stable symbols across all analysed versions.")],
        });
    }

    // generate the base (first version)
    {
        let module_name = format!("v{}", first_version.replace(".", "_"));
        let partial_output = args.output_dir.join(format!("{module_name}.rs"));

        let mut whitelist = ItemSet::default();
        whitelist.extend(&report.unstable.breaking);
        whitelist.extend(&report.unstable.dubious);
        whitelist.extend(&report.unstable.compatible);
        whitelist.extend(ALWAYS_EMIT_IN_PARTIAL_MODULES);

        let filter = ItemFilter {
            whitelist,
            blacklist: ItemSet::default(),
        };

        bindings::generate_bindings(first_header, &filter, &partial_output);

        modules.push(Module {
            name: module_name,
            comment: vec![format!(
                "Unstable symbols for version {first_version} (base)."
            )],
        });
    }

    // generate the base version and the partial versions
    for diff in report.changes_per_version {
        if diff.changed.breaking.is_empty() && diff.changed.dubious.is_empty() {
            // Note that we're not interested in backward-compatible changes.
            continue;
        }

        let module_name = version_to_module_name(&diff.version_new);
        let partial_output = args.output_dir.join(format!("{module_name}.rs"));

        // To make ease the multi-version implementation, export all the unstable symbols in each partial version.
        // The alternative (export only `diff.changed.*`) would require to tweak rust-bindgen (see further below).
        let mut whitelist = ItemSet::default();
        whitelist.extend(&report.unstable.breaking);
        whitelist.extend(&report.unstable.dubious);
        whitelist.extend(&report.unstable.compatible);
        whitelist.extend(ALWAYS_EMIT_IN_PARTIAL_MODULES);

        let filter = ItemFilter {
            whitelist,
            blacklist: ItemSet::default(),
        };

        let c_input = diff.input_new.as_path();
        bindings::generate_bindings(c_input, &filter, &partial_output);

        modules.push(Module {
            name: module_name,
            comment: vec![format!(
                "Unstable symbols for version {} (contains some breaking changes from {}).",
                diff.version_new, diff.version_old
            )],
        });
    }

    // TODO (?) expose per-symbol changes, to make multi-version implementation easier
    // Actually, it cannot be done currently, because rust-bindgen always export dynamically loaded functions as methods, and we cannot create "pub use" aliases to methods.

    // declare all the modules in mod.rs
    let mut mod_content = String::new();
    for Module { name, comment } in modules {
        for line in comment {
            writeln!(mod_content, "/// {line}").unwrap();
        }
        writeln!(mod_content, "pub mod {name};").unwrap();
    }

    let mod_path = args.output_dir.join("mod.rs");
    std::fs::write(mod_path, mod_content).unwrap();
}

struct Module {
    name: String,
    comment: Vec<String>,
}
