extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn gen_bindings() {
    let bindings = bindgen::Builder::default()
        .header("../../../athcon.h")
        .generate_comments(true)
        // do not generate an empty enum for ATHCON_ABI_VERSION
        .constified_enum("")
        // generate Rust enums for each enum
        .rustified_enum(".*")
        // force deriving the Hash trait on basic types (address, bytes32)
        .derive_hash(true)
        // force deriving the PratialEq trait on basic types (address, bytes32)
        .derive_partialeq(true)
        .blocklist_type("athcon_host_context")
        .allowlist_type("athcon_.*")
        .allowlist_function("athcon_.*")
        .allowlist_var("ATHCON_ABI_VERSION")
        // TODO: consider removing this
        .size_t_is_usize(true)
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn main() {
    gen_bindings();
}
