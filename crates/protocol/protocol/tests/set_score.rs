#![allow(dead_code)]

use bedrock_protocol::ProtoVersionPackets;
use bedrock_protocol_core::ProtoCodec;
use std::io::Cursor;

type Packet<V> = <V as ProtoVersionPackets>::SetScorePacket;

fn round_trip<P: ProtoCodec>(bytes: &[u8]) -> P {
    let mut cursor = Cursor::new(bytes);
    let packet = P::deserialize(&mut cursor).expect("packet decodes");
    assert_eq!(
        usize::try_from(cursor.position()).unwrap(),
        bytes.len(),
        "packet left bytes unread"
    );
    let mut encoded = Vec::new();
    packet.serialize(&mut encoded).unwrap();
    assert_eq!(encoded, bytes, "encoding differs from the input");
    packet
}

fn remove_entry(scoreboard_id: u8, tail: &[u8]) -> Vec<u8> {
    let mut bytes = vec![0, 6];
    bytes.extend(b"remove");
    bytes.push(scoreboard_id * 2);
    bytes.extend(tail);
    bytes
}

fn change_player_entry() -> Vec<u8> {
    let mut bytes = vec![1, 12];
    bytes.extend(b"changeplayer");
    bytes.push(8);
    bytes.extend([3]);
    bytes.extend(b"obj");
    bytes.extend(7i32.to_le_bytes());
    bytes.push(10);
    bytes
}

fn packet(entries: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes = vec![u8::try_from(entries.len()).unwrap()];
    for entry in entries {
        bytes.extend(entry);
    }
    bytes
}

#[cfg(feature = "v2168hotfix4")]
#[test]
fn v2168_hotfix4_remove_entry_gates_the_objective_behind_a_presence_bool() {
    use bedrock_protocol::V2168Hotfix4;
    use bedrock_protocol::v2168hotfix4::packets::ScorePacketEntry;

    let bytes = packet(&[
        remove_entry(1, &[0]),
        remove_entry(2, &[1, 0]),
        remove_entry(3, &[1, 1, 3, b'o', b'b', b'j']),
        change_player_entry(),
    ]);
    let decoded = round_trip::<Packet<V2168Hotfix4>>(&bytes);

    let objectives = decoded
        .score_info
        .iter()
        .filter_map(|entry| match entry {
            ScorePacketEntry::Remove {
                scoreboard_id,
                objective_name,
            } => Some((scoreboard_id.id, objective_name.clone())),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        objectives,
        vec![
            (1, None),
            (2, Some(None)),
            (3, Some(Some(String::from("obj")))),
        ]
    );
    assert!(matches!(
        &decoded.score_info[3],
        ScorePacketEntry::ChangePlayer {
            scoreboard_id,
            objective_name,
            score_value: 7,
            player_unique_id: 5,
        } if scoreboard_id.id == 4 && objective_name == "obj"
    ));
}

#[cfg(feature = "v2169")]
#[test]
fn v2169_remove_entry_keeps_a_single_optional_objective() {
    use bedrock_protocol::V2169;
    use bedrock_protocol::v2168::packets::ScorePacketEntry;

    let bytes = packet(&[
        remove_entry(1, &[0]),
        remove_entry(3, &[1, 3, b'o', b'b', b'j']),
        change_player_entry(),
    ]);
    let decoded = round_trip::<Packet<V2169>>(&bytes);

    assert!(matches!(
        &decoded.score_info[1],
        ScorePacketEntry::Remove {
            objective_name: Some(name),
            ..
        } if name == "obj"
    ));
}
