use bedrock_level::db::{Database, WriteBatch};

#[test]
fn a_synced_write_is_read_back_after_reopening() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("db");
    {
        let db = Database::open(path.to_str().unwrap()).unwrap();
        let mut batch = WriteBatch::new();
        batch.insert(b"kept", b"value");
        batch.insert(b"dropped", b"value");
        batch.remove(b"dropped");
        db.write_sync(batch).unwrap();
    }
    let db = Database::open(path.to_str().unwrap()).unwrap();
    assert_eq!(db.get(b"kept").unwrap().as_deref(), Some(&b"value"[..]));
    assert!(db.get(b"dropped").unwrap().is_none());
}
