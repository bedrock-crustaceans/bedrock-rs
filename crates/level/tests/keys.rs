use bedrock_level::db::Database;

fn database_with(keys: &[&[u8]]) -> (tempfile::TempDir, Database) {
    let tmp = tempfile::tempdir().unwrap();
    let db = Database::open(tmp.path().join("db").to_str().unwrap()).unwrap();
    for key in keys {
        db.insert(key, [*key, b"-value"].concat()).unwrap();
    }
    (tmp, db)
}

#[test]
fn next_ref_reads_what_the_iterator_reads() {
    let (_tmp, db) = database_with(&[b"a", b"b", b"c"]);

    let owned: Vec<_> = (&mut db.keys().unwrap())
        .map(|kv| (kv.key().to_vec(), kv.value().to_vec()))
        .collect();
    let mut borrowed = Vec::new();
    let mut keys = db.keys().unwrap();
    while let Some((k, v)) = keys.next_ref() {
        borrowed.push((k.to_vec(), v.to_vec()));
    }

    assert_eq!(owned.len(), 3);
    assert_eq!(borrowed, owned);
    assert_eq!(borrowed[1], (b"b".to_vec(), b"b-value".to_vec()));
}

#[test]
fn current_is_the_record_seek_or_next_stands_on() {
    let (_tmp, db) = database_with(&[b"a", b"c"]);
    let mut keys = db.keys().unwrap();

    keys.seek(b"b");
    assert_eq!(keys.current(), Some((&b"c"[..], &b"c-value"[..])));
    assert_eq!(
        keys.next_ref().map(|(k, _)| k.to_vec()),
        Some(b"c".to_vec())
    );
    assert_eq!(keys.current().map(|(k, _)| k.to_vec()), Some(b"c".to_vec()));
    assert!(keys.next_ref().is_none());
    assert!(keys.current().is_none());
    assert!(keys.status().is_ok());
}

#[test]
fn status_reports_a_table_that_cannot_be_read() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("db");
    {
        let db = Database::open(path.to_str().unwrap()).unwrap();
        for i in 0..2000u32 {
            db.insert(i.to_be_bytes(), [7u8; 100]).unwrap();
        }
    }
    drop(Database::open(path.to_str().unwrap()).unwrap());
    let table = std::fs::read_dir(&path)
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.extension().is_some_and(|e| e == "ldb"))
        .expect("reopening the database writes its log to a table");
    let mut bytes = std::fs::read(&table).unwrap();
    bytes[..64].fill(0xAB);
    std::fs::write(&table, bytes).unwrap();

    let db = Database::open(path.to_str().unwrap()).unwrap();
    let mut keys = db.keys().unwrap();
    while keys.next_ref().is_some() {}
    assert!(keys.status().is_err());
}

#[test]
fn seek_starts_at_the_first_key_not_before_it() {
    let (_tmp, db) = database_with(&[b"a", b"c", b"e"]);
    let mut keys = db.keys().unwrap();

    keys.seek(b"b");
    assert_eq!(
        keys.next_ref().map(|(k, _)| k.to_vec()),
        Some(b"c".to_vec())
    );
    assert_eq!(
        keys.next_ref().map(|(k, _)| k.to_vec()),
        Some(b"e".to_vec())
    );
    assert!(keys.next_ref().is_none());

    keys.seek(b"a");
    assert_eq!(
        (&mut keys).next().map(|kv| kv.key().to_vec()),
        Some(b"a".to_vec())
    );

    keys.seek(b"f");
    assert!(keys.next_ref().is_none());
    assert!((&mut keys).next().is_none());
}
