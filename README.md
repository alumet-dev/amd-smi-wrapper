# Rust wrapper for the AMD SMI library

🚧 **WARNING**: This is a work in progress, use at your own risk.

## Crates

This repository contains 3 crates:
- `amd-smi-wrapper`: this is the main library crate, use it in your programs
- `amd-smi-wrapper-sys`: basic bindings to the C library
- `bindings-generator`: executable tool to help us generate the bindings

### Regenerate the Bindings

To regen the bindings, use the bindings generator:

```sh
cargo run -p bindings-generator -- --input-header bindings-generator/input/amdsmi-rocm-7.2.0.h
```

This updates `amd-smi-wrapper-sys/src/versions/latest.rs`.

Only the symbols declared in the [whitelist](bindings-generator/input/whitelist.txt) are generated.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
