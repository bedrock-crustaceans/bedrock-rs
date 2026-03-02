use std::process::Command;

fn main() {
    // Only build the `leveldb` module when the `mojang-leveldb` feature is enabled and we are not
    // generating docs.
    #[cfg(all(feature = "mojang-leveldb", not(doc)))]
    {
        // Check whether the `leveldb` submodule exists.
        if std::fs::read_dir("ffi/leveldb").unwrap().next().is_none() {
            // Submodule does not exist, retrieve it
            Command::new("git")
                .args(&["submodule", "update", "--init"])
                .status()
                .expect("Failed to download `leveldb` submodule");
        }

        // Build `leveldb` submodule
        println!("cargo:rerun-if-changed=ffi");

        let mut config = cmake::Config::new("ffi");
        let mut ffi_dst = config.build().join("build");
        let mut leveldb_dst = ffi_dst.join("leveldb");

        if cfg!(target_env = "msvc") {
            let profile = config.get_profile();
            ffi_dst = ffi_dst.join(profile);
            leveldb_dst = leveldb_dst.join(profile);

            println!("cargo:rustc-link-lib=shell32");
        }

        println!(
            "Searching for leveldb-ffi and leveldb-mcpe in {}",
            ffi_dst.display()
        );
        println!("cargo:rustc-link-search=native={}", ffi_dst.display());
        println!("cargo:rustc-link-search=native={}", leveldb_dst.display());
        println!("cargo:rustc-link-lib=static=leveldb-ffi");
        println!("cargo:rustc-link-lib=static=leveldb-mcpe");

        #[cfg(unix)]
        println!("cargo:rustc-link-lib=dylib=stdc++");
    }
}
