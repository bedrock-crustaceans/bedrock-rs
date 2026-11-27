//! Tells the crate whether it is built without optimisation: the stack a level of a typed
//! JSON read takes is eight times larger then, and `json::stack` decides by it when a read
//! may run on the stack there is.

#![allow(
    clippy::disallowed_methods,
    reason = "a build script reads the profile cargo hands it; the crate reads no environment"
)]

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rustc-check-cfg=cfg(unoptimised)");
    // Cargo sets `OPT_LEVEL` to the profile's `opt-level` for this crate.
    if std::env::var("OPT_LEVEL").as_deref() == Ok("0") {
        println!("cargo::rustc-cfg=unoptimised");
    }
}
