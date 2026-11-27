//! The stack helpers of the block tests: a pack from its files.

use bedrock_addon::pack::{OpenOptions, Pack, PackOrigin};
use bedrock_addon::storage::PackPath;

/// A pack from its files.
pub fn pack(files: Vec<(&str, Vec<u8>)>) -> Pack {
    let files = files
        .into_iter()
        .map(|(path, bytes)| (PackPath::new(path).expect("a pack path"), bytes));
    let options = OpenOptions {
        origin: PackOrigin::User,
        ..OpenOptions::default()
    };
    Pack::from_files(files, &options).expect("the pack opens")
}
