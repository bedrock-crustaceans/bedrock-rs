use bedrockrs_level::{
    db::Database,
    key::{Key, KeyVariant},
    subchunk::SubChunk,
};

#[test]
fn open_database() {
    let database = Database::open("test_level/db").unwrap();
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
                let chunk = SubChunk::deserialize_disk(val.as_ref()).unwrap();

                println!("{chunk:?}");

                break;
            }
            _ => {}
        }
    }
}
