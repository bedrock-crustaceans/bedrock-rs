//! The `std` driver of `bedrock_addon`: answers the sans-IO loaders with blocking `std::fs`
//! calls, runs their jobs on the calling thread or a rayon pool, reads the server-side files
//! around packs and writes packs to disk. `bedrock_addon` never touches the host; this crate
//! is where that happens.
//!
//! # One path per item
//!
//! The driver's items live at the crate root, and nothing of `bedrock_addon` is re-exported:
//! its types keep their one path there, so a user names both crates.
//!
//! - [`Driver`], [`Options`], [`Strategy`], [`DirSource`]: the driver, how it opens packs and
//!   where its work runs, and the packs directories and archives it scans.
//! - [`load_dir`], `load_archive`: one pack, opened with a fresh driver.
//! - the readers: [`read_world_packs`], [`read_app_data`], [`read_saved_settings`],
//!   [`read_saved_settings_document`], and with `script` `read_script_config`,
//!   `read_script_config_files`, `read_server_properties`.
//! - the writers: [`write_dir`] puts the files of a `bedrock_addon::pack::PackWriter` below
//!   an empty or absent directory; `write_archive` replaces an archive file with the writer's
//!   zip bytes through a synced temporary file.
//! - the errors: [`LoadError`], [`ReadError`], [`SaveError`].
//!
//! Modules:
//! - `io`: the root table (`Roots`) and `respond`, which answers one loader request with
//!   blocking `std::fs` calls; reusable by any loop.
//! - `keys`: content key lookup (`ContentKeyProvider`, `read_sidecar_key`).
//!
//! # Features
//!
//! All on by default:
//! - `rayon`: `Strategy::Rayon` and `Strategy::RayonPool`, jobs and reads on a rayon pool
//!   built with `RECOMMENDED_JOB_STACK`.
//! - `zip`: `load_archive`, `Driver::pack_archive`, `write_archive`.
//! - `encrypted`: key sidecars (`keys::read_sidecar_key`, `Options::sidecar_keys`).
//! - `script`: `read_script_config`, `read_script_config_files`, `read_server_properties`
//!   (turns on `bedrock_addon`'s `script` module, where their types live).
//!
//! # Threads and stacks
//!
//! [`Strategy`] says where reads and jobs run, never what is loaded. The pool the driver
//! builds gives each thread `bedrock_addon::load::RECOMMENDED_JOB_STACK`; no file can
//! overflow a smaller stack by its nesting, down to one megabyte, so `Strategy::Sequential`
//! on a spawned thread is safe too.
//!
//! # Examples
//!
//! The README has a compiled quick start and its examples table lists `load_stacks`,
//! `rayon_driver` and `ruby_pack` under `examples/`; each runs with no argument on packs it
//! writes into a temporary directory.
// docs build with every feature set.

#![cfg_attr(docsrs, feature(doc_cfg))]

/// The driver re-exports nothing of `bedrock_addon`: its types keep their one path there.
///
/// ```compile_fail,E0432
/// use bedrock_addon_std::Pack;
/// ```
/// ```compile_fail,E0432
/// use bedrock_addon_std::PackStack;
/// ```
#[cfg(doctest)]
struct NoItemAtTheRoot;

/// Compiles the README's code blocks as doctests.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;

mod driver;
mod error;
pub mod io;
pub mod keys;
mod read;
#[cfg(feature = "script")]
mod script_config;
mod settings;
mod write;

#[cfg(feature = "zip")]
pub use driver::load_archive;
pub use driver::{DirSource, DirSourceKind, Driver, Options, Strategy, load_dir};
pub use error::{LoadError, ReadError, SaveError};
pub use read::{read_app_data, read_world_packs};
#[cfg(feature = "script")]
#[cfg_attr(docsrs, doc(cfg(feature = "script")))]
pub use script_config::{read_script_config, read_script_config_files, read_server_properties};
pub use settings::{read_saved_settings, read_saved_settings_document};
#[cfg(feature = "zip")]
pub use write::write_archive;
pub use write::write_dir;
