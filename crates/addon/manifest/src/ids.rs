//! Manifest UUIDs: vanilla's text form and its replacement of a malformed one.

use uuid::Uuid;

use super::md5;

/// The salt vanilla prepends to a malformed UUID's text before hashing it.
const UPGRADE_SALT: &str = "pack_id";

/// A UUID as vanilla reads it: 36 characters hyphenated or 32 hex digits, any letter case. The
/// nil UUID counts as malformed. `None` for anything else.
pub(super) fn parse(text: &str) -> Option<Uuid> {
    // Vanilla takes 32 or 36 characters, case-insensitively, and counts an explicit nil UUID
    // as malformed. The `uuid` crate also reads braced and URN forms, which vanilla does not.
    if text.len() != 32 && text.len() != 36 {
        return None;
    }
    Uuid::try_parse(text).ok().filter(|uuid| !uuid.is_nil())
}

/// The id vanilla gives a pack whose UUID text is malformed: the MD5 of `"pack_id"` followed by
/// the text as written, with the version nibble set to 3, so the id is stable across loads.
pub(super) fn salvage(raw: &str) -> Uuid {
    // The MD5 of the salt followed by the raw text, with the version nibble forced to 3.
    let mut input = Vec::with_capacity(UPGRADE_SALT.len() + raw.len());
    input.extend_from_slice(UPGRADE_SALT.as_bytes());
    input.extend_from_slice(raw.as_bytes());
    let mut bytes = md5::digest(&input);
    bytes[6] = (bytes[6] & 0x0f) | 0x30;
    Uuid::from_bytes(bytes)
}

/// A stable id derived from `text` alone, the way vanilla re-derives a header UUID that a
/// format-0 module duplicates (the same hash over the UUID's text, with no salt): MD5 with the
/// version nibble set to 3.
pub(super) fn hash(text: &str) -> Uuid {
    let mut bytes = md5::digest(text.as_bytes());
    bytes[6] = (bytes[6] & 0x0f) | 0x30;
    Uuid::from_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `hash` over more than one MD5 block, recorded from the hand MD5 and checked against
    /// Python's `hashlib`: 57 bytes (the length crosses the 56-byte pad edge) and 70 (a full
    /// block and part of a second). `salvage` is pinned end to end in
    /// `tests/characterize_manifest.txt` ("legacy uuid malformed v0 pad edge", "... two blocks").
    #[test]
    fn hash_over_more_than_one_block_is_pinned() {
        let pad_edge = format!("{}0123456", "not-a-uuid".repeat(5));
        assert_eq!(pad_edge.len(), 57);
        assert_eq!(
            hash(&pad_edge).hyphenated().to_string(),
            "a9dcce50-f29f-3752-e866-fabfa96b8308"
        );
        assert_eq!(
            hash(&"not-a-uuid".repeat(7)).hyphenated().to_string(),
            "a74d62d9-5e96-3ad5-d026-085f8fc86227"
        );
        assert_eq!(
            salvage(&"not-a-uuid".repeat(7)).hyphenated().to_string(),
            "ffe97b72-7572-369f-24ef-5594735028e7"
        );
    }
}
