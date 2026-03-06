use std::fs::File;

use bedrockrs_level::settings::LevelSettings;
use bedrockrs_level::{
    db::Database,
    key::{Key, KeyVariant},
    subchunk::{SubChunk, Unpacked},
    traits::DatabaseAccess,
};
use flate2::read::GzDecoder;
use tar::Archive;

fn extract_test_dir() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("Failed to create temp dir");

    let tar_gz = File::open("tests/level.tar.gz").expect("Seed missing");
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);

    archive.unpack(tmp.path()).expect("Failed to unpack seed");

    tmp
}

fn open_test_db() -> Database {
    let tmp = extract_test_dir();
    let db_path = tmp.path().join("test_level/db");

    Database::open(db_path.to_str().unwrap()).unwrap()
}

#[test]
#[ignore = "currently not properly implemented"]
fn read_level_dat() {
    let tmp = extract_test_dir();
    let dat_path = tmp.path().join("test_level/level.dat");

    let data = std::fs::read(&dat_path).unwrap();
    let settings = LevelSettings::read(data.as_slice()).unwrap();

    println!("{settings:?}");
}

#[test]
fn read_biome() {
    let db = open_test_db();
    let mut keys = db.keys();

    for kv in &mut keys {
        let mut key_buf = kv.key();
        let key = Key::deserialize(&mut key_buf);

        // match key.data {
        //     KeyVariant::Biome3d => {
        //         println!("{key:?}");
        //
        //         // break
        //     },
        //     _ => {}
        // }
    }
}

#[test]
fn read_subchunk() {
    let db = open_test_db();
    let mut keys = db.keys();

    for kv in &mut keys {
        let mut key_buf = kv.key();
        let key = Key::deserialize(&mut key_buf).unwrap();

        match key.data {
            KeyVariant::SubChunk { .. } => {
                println!("{key:?}");

                let mut buf = Vec::new();
                key.serialize(&mut buf).unwrap();

                let val = db.get(buf).unwrap().unwrap();
                let chunk = SubChunk::from_disk::<Unpacked, _>(val.as_ref()).unwrap();

                println!("{chunk:?}");

                break;
            }
            _ => {}
        }
    }
}
