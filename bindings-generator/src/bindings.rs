//! Bindings generation with rust-bindgen.
use std::{collections::HashSet, path::Path};

use bindgen::{Builder, EnumVariation, callbacks::ParseCallbacks};

const LIB: &str = "libamd_smi";

#[derive(Debug)]
struct DocFix;

impl ParseCallbacks for DocFix {
    fn process_comment(&self, comment: &str) -> Option<String> {
        // Transform C/C++ documentation to avoid Rust doc-test errors
        Some(format!("```text\n{comment}\n```"))
    }
}

#[derive(Debug, Clone, Default)]
pub struct ItemFilter {
    pub whitelist: ItemSet,
    pub blacklist: ItemSet,
}

#[derive(Debug, Clone, Default)]
pub struct ItemSet(HashSet<String>);

impl ItemSet {
    /// Parses a file line by line.
    ///
    /// # Format
    /// - 1 line = 1 item
    /// - leading and trailing whitespaces are removed
    /// - comments are supported: a line starting with `#` is ignored
    pub fn parse(path: &Path) -> Self {
        let mut items = HashSet::default();
        let content = std::fs::read_to_string(path).expect("failed to read whitelist");
        for line in content.lines() {
            let line = line.trim_ascii();
            if !line.is_empty() && !line.starts_with("#") {
                items.insert(line.to_owned());
            }
        }
        Self(items)
    }

    /// Creates a set from an iterator.
    pub fn from_iter(items: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        let items = items.into_iter().map(|i| i.as_ref().to_owned()).collect();
        Self(items)
    }

    /// Adds multiple items to the set.
    pub fn extend(&mut self, items: impl IntoIterator<Item = impl AsRef<str>>) {
        self.0
            .extend(items.into_iter().map(|i| i.as_ref().to_owned()));
    }

    /// Removes multiple items from the set.
    pub fn remove_all(&mut self, items: impl IntoIterator<Item = impl AsRef<str>>) {
        for item in items {
            self.0.remove(item.as_ref());
        }
    }
}

/// Generates Rust bindings for a C file.
///
/// Only the items present in the whitelist file are included.
pub fn generate_bindings(c_input: &Path, filter: &ItemFilter, rust_output: &Path) {
    let header_file_name = c_input.file_name().unwrap().to_str().unwrap();

    let mut builder = Builder::default()
        .header(c_input.to_str().unwrap())
        .parse_callbacks(Box::new(DocFix))
        .dynamic_library_name(LIB)
        .dynamic_link_require_all(false)
        .default_enum_style(EnumVariation::NewType {
            is_bitfield: false,
            is_global: false,
        })
        .raw_line(format!("/* generated from: {header_file_name} */"));

    for item in &filter.whitelist.0 {
        builder = builder.allowlist_item(item);
    }
    for item in &filter.blacklist.0 {
        builder = builder.blocklist_item(item);
    }
    builder
        .generate()
        .expect("failed to generate the bindings")
        .write_to_file(rust_output)
        .expect("failed to write the bindings");
}
