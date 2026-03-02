use bedrockrs_level::db::Database;

#[test]
fn open_database() {
    let database = Database::open("../test_level/db").unwrap();
}
