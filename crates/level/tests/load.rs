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

pub fn extract_test_db() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("Failed to create temp dir");

    let tar_gz = File::open("tests/level.tar.gz").expect("Seed missing");
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);

    archive.unpack(tmp.path()).expect("Failed to unpack seed");

    tmp
}

#[test]
#[ignore = "currently not properly implemented"]
fn read_level_dat() {
    let tmp = extract_test_db();
    let dat_path = tmp.path().join("test_level/level.dat");

    let data = std::fs::read(&dat_path).unwrap();
    let settings = LevelSettings::read(data.as_slice()).unwrap();

    println!("{settings:?}");
}

#[test]
fn mojang_read_chunk() {
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
                let chunk = SubChunk::from_disk::<Unpacked, _>(val.as_ref()).unwrap();

                println!("{chunk:?}");

                break;
            }
            _ => {}
        }
    }
}
