use std::fs::File;

use bedrockrs_level::{
    db::Database,
    key::{Key, KeyVariant},
    subchunk::{Greedy, SubChunk},
};
use flate2::read::GzDecoder;
use tar::Archive;

pub fn extract_test_db() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("Failed to create temp dir");

    let tar_gz = File::open("tests/level.tar.gz").expect("Seed missing");
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);

    archive.unpack(tmp.path()).expect("Failed to unpack seed");

    tmp
}

#[test]
fn open_database() {
    let tmp = extract_test_db();
    let tmp_path = tmp.path().join("test_level/db");
    let tmp_path = tmp_path.to_str().unwrap();

    let database = Database::open(tmp_path).unwrap();
    let mut keys = database.iter();

    for kv in &mut keys {
        let mut key_buf = kv.key();
        let key = Key::deserialize(&mut key_buf).unwrap();

        match key.data {
            KeyVariant::SubChunk { .. } => {
                println!("{key:?}");

                let mut buf = Vec::new();
                key.serialize(&mut buf).unwrap();

                let val = database.get(buf).unwrap().unwrap();
                let chunk = SubChunk::deserialize_from_disk::<Greedy, _>(val.as_ref()).unwrap();

                println!("{chunk:?}");

                break;
            }
            _ => {}
        }
    }
}
