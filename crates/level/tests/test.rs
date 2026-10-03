use std::collections::HashMap;
use std::fs::File;
use std::io::Cursor;

use bedrock_level::Greedy;
use bedrock_level::biome::Biomes;
use bedrock_level::player::PlayerData;
use bedrock_level::settings::LevelSettings;
use bedrock_level::subchunk::BlockDef;
use bedrock_level::types::BlockPosition;
use bedrock_level::{
    db::Database,
    key::{Key, KeyVariant},
    subchunk::SubChunk,
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
fn reopened_database_reads_back_inserted_value() {
    let tmp = extract_test_dir();
    let db_path = tmp.path().join("test_level/db");
    let db_path = db_path.to_str().unwrap();

    let key = b"bedrock-rs test key";
    let value = b"bedrock-rs test value".to_vec();

    {
        let db = Database::open(db_path).unwrap();
        db.insert(key, value.clone()).unwrap();
    }

    {
        let db = Database::open(db_path).unwrap();
        let read = db.get(key).unwrap().unwrap();
        assert_eq!(&*read, value.as_slice());

        db.remove(key).unwrap();
        assert!(db.get(key).unwrap().is_none());
    }
}

#[test]
fn get_of_missing_key_is_none() {
    let db = open_test_db();
    assert!(db.get(b"this key does not exist").unwrap().is_none());
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
#[ignore = "currently not properly implemented"]
fn read_local_player() {
    let db = open_test_db();
    let mut keys = db.keys().unwrap();

    for kv in &mut keys {
        let mut key_buf = Cursor::new(kv.key());
        let key = Key::deserialize(&mut key_buf);

        let Ok(key) = key else { continue };
        if matches!(key.data, KeyVariant::LocalPlayer) {
            let data = kv.value();
            let nbt: PlayerData = nbtx::from_le_bytes(&mut data.as_ref()).unwrap();
            println!("{nbt:#?}");
        }
    }
}

#[test]
fn read_biome() {
    let db = open_test_db();
    let mut keys = db.keys().unwrap();

    for kv in &mut keys {
        let mut key_buf = Cursor::new(kv.key());
        let Ok(key) = Key::deserialize(&mut key_buf) else {
            continue;
        };

        if key.data == KeyVariant::Biome3d {
            let mut value = Cursor::new(kv.value());
            let biome = Biomes::from_disk::<Greedy, _>(&mut value).unwrap();

            let mut writer = Cursor::new(Vec::new());
            biome.to_disk(&mut writer).unwrap();

            let value = writer.into_inner();
            let mut reader = Cursor::new(value.as_slice());
            let biome2 = Biomes::from_disk::<Greedy, _>(&mut reader).unwrap();

            assert_eq!(biome, biome2);

            println!("{biome:?}");

            // break
        }
    }
}

#[test]
fn read_subchunk() {
    let db = open_test_db();
    let mut keys = db.keys().unwrap();

    for kv in &mut keys {
        let mut key_buf = Cursor::new(kv.key());
        let Ok(key) = Key::deserialize(&mut key_buf) else {
            continue;
        };

        if let KeyVariant::SubChunk { .. } = key.data {
            println!("{key:?}");

            let mut buf = Vec::new();
            key.serialize(&mut buf).unwrap();

            let mut val = Cursor::new(db.get(buf).unwrap().unwrap());
            let mut chunk = SubChunk::from_disk_lazy(&mut val).unwrap();

            let layer = chunk.get_layer_mut(0).unwrap();
            for i in 0..40 {
                layer.set(
                    BlockPosition(0, i, 0),
                    BlockDef {
                        name: "test".to_string(),
                        states: HashMap::from([(
                            String::from("test2"),
                            nbtx::Value::Byte(i as i8),
                        )]),
                        version: Some(0x0102_0304),
                    },
                );
            }

            break;
        }
    }
}
