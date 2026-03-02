use bedrockrs_level::db::Database;

#[test]
fn open_database() {
    let database = Database::open("test_level/db").unwrap();
    let mut keys = database.iter();

    for kv in &mut keys {
        let key = kv.key();
        let string = String::from_utf8_lossy(key.as_ref());

        println!("{string}");
    }
}
