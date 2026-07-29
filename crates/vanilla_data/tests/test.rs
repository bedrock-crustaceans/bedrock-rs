use std::fs::File;
use std::io::Cursor;

use bedrock_level::db::Database;
use bedrock_level::key::{Key, KeyVariant};
use bedrock_level::traits::CursorExt;
use bedrock_vanilla_data::block_entities::BlockEntity;

use flate2::read::GzDecoder;
use tar::Archive;

fn extract_test_dir() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("Failed to create temp dir");

    // The world fixture is shared with the level crate rather than duplicated here.
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../level/tests/level.tar.gz");
    let tar_gz = File::open(fixture).expect("Seed missing");
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);

    archive.unpack(tmp.path()).expect("Failed to unpack seed");

    tmp
}

fn open_test_db() -> Database {
    let tmp = extract_test_dir();
    let db_path = tmp.path().join("debug/db");

    Database::open(db_path.to_str().unwrap()).unwrap()
}

#[test]
fn read_block_entity() {
    let db = open_test_db();
    let mut keys = db.keys().unwrap();
    println!("keys count: {}", keys.count());
    drop(keys);

    let mut keys = db.keys().unwrap();

    for kv in &mut keys {
        let mut key_buf = Cursor::new(kv.key());

        let Ok(key) = Key::deserialize(&mut key_buf) else {
            println!("failed: {:?}", String::from_utf8_lossy(key_buf.get_ref()));
            continue;
        };

        println!("key: {key:?}");

        match key.data {
            KeyVariant::BlockEntity => {
                let mut untyped_value = Cursor::new(kv.value());
                let mut typed_value = Cursor::new(kv.value());

                while untyped_value.has_remaining() {
                    let untyped: nbtx::Value = nbtx::from_le_bytes(&mut untyped_value).unwrap();

                    match BlockEntity::from_disk(&mut typed_value) {
                        Ok(typed) => {
                            println!("{typed:?}");
                        }
                        Err(err) => {
                            dbg!(untyped);
                            panic!("{err:?}");
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
