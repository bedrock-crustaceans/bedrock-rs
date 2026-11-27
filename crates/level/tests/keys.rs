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
