use crate::compression::Compression;
use crate::encryption::Encryption;
use crate::error::NetworkCodecError;
use bedrock_protocol_core::error::ProtoCodecError;
use bedrock_protocol_core::{PacketHeader, Packets, ProtoCodec, ProtoCodecVAR};
use std::io::Cursor;

pub const MAX_BATCH_PACKETS: usize = 4096;
pub const MAX_BATCH_LEN: usize = 10 << 20;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UnreadBytes {
    #[default]
    Warn,
    Reject,
}

/// Encodes `packets` with the default header, then compresses and encrypts the batch when asked.
///
/// # Errors
///
/// Returns a [`NetworkCodecError`] when a packet fails to serialize, or compression or
/// encryption fails.
pub fn encode_packets<T: Packets>(
    packets: &[T],
    compression: Option<&Compression>,
    encryption: Option<&mut Encryption>,
) -> Result<Vec<u8>, NetworkCodecError> {
    encode_packets_with_header(packets, &PacketHeader::default(), compression, encryption)
}

/// Encodes `packets` with the sender and target sub-client ids of `header` on every frame.
/// `header.packet_id` is ignored; each frame carries its own packet's id. Servers route an
/// incoming packet by its sender sub-client id and clients by its target sub-client id.
///
/// # Errors
///
/// Returns a [`NetworkCodecError`] when a packet fails to serialize, or compression or
/// encryption fails.
pub fn encode_packets_with_header<T: Packets>(
    packets: &[T],
    header: &PacketHeader,
    compression: Option<&Compression>,
    encryption: Option<&mut Encryption>,
) -> Result<Vec<u8>, NetworkCodecError> {
    tracing::trace!("Encoding packets");

    let mut packets_stream = batch_packets::<T>(packets, *header)?;
    packets_stream = compress_packets(packets_stream, compression)?;
    packets_stream = encrypt_packets(packets_stream, encryption)?;

    Ok(packets_stream)
}

/// Decodes a batch into its packets, each with the header it arrived under so the sender and
/// target sub-client ids are kept.
///
/// # Errors
///
/// Returns a [`NetworkCodecError`] when decryption, decompression or framing of the batch fails,
/// or when any packet fails to decode, or leaves bytes unread under [`UnreadBytes::Reject`].
pub fn decode_packets<T: Packets>(
    mut packets_stream: Vec<u8>,
    compression: Option<&Compression>,
    encryption: Option<&mut Encryption>,
    max_batch_len: usize,
    unread_bytes: UnreadBytes,
) -> Result<Vec<(PacketHeader, T)>, NetworkCodecError> {
    tracing::trace!("Decoding packets");

    packets_stream = decrypt_packets(packets_stream, encryption)?;
    packets_stream = decompress_packets(packets_stream, compression, max_batch_len)?;
    let packets = separate_packets::<T>(&packets_stream, unread_bytes)?;

    Ok(packets)
}

pub type DecodedPackets<T> = Vec<(PacketHeader, Result<T, NetworkCodecError>)>;

/// Decodes every frame of a batch independently, so one undecodable packet does not hide the
/// others. A frame whose header cannot be read reports the default header. The outer error is a
/// failure of the batch itself: decryption, decompression or framing.
///
/// # Errors
///
/// Returns a [`NetworkCodecError`] when decryption, decompression or framing of the batch fails.
pub fn decode_packets_each<T: Packets>(
    mut packets_stream: Vec<u8>,
    compression: Option<&Compression>,
    encryption: Option<&mut Encryption>,
    max_batch_len: usize,
    unread_bytes: UnreadBytes,
) -> Result<DecodedPackets<T>, NetworkCodecError> {
    tracing::trace!("Decoding packets one by one");

    packets_stream = decrypt_packets(packets_stream, encryption)?;
    packets_stream = decompress_packets(packets_stream, compression, max_batch_len)?;

    Ok(split_batch(&packets_stream)?
        .into_iter()
        .map(|frame| match decode_packet::<T>(frame, unread_bytes) {
            Ok((header, packet)) => (header, Ok(packet)),
            Err(error) => (frame_header(frame), Err(error)),
        })
        .collect())
}

fn frame_header(frame: &[u8]) -> PacketHeader {
    <PacketHeader as ProtoCodec>::deserialize(&mut Cursor::new(frame)).unwrap_or_default()
}

fn batch_packets<T: Packets>(
    packets: &[T],
    template: PacketHeader,
) -> Result<Vec<u8>, NetworkCodecError> {
    let frames = packets
        .iter()
        .map(|packet| {
            let header = PacketHeader {
                packet_id: packet.id(),
                ..template
            };

            let mut frame = Vec::with_capacity(packet.size_hint(&header));
            packet.serialize(&header, &mut frame)?;
            Ok(frame)
        })
        .collect::<Result<Vec<_>, NetworkCodecError>>()?;

    Ok(join_batch(&frames))
}

fn separate_packets<T: Packets>(
    packets_stream: &[u8],
    unread_bytes: UnreadBytes,
) -> Result<Vec<(PacketHeader, T)>, NetworkCodecError> {
    split_batch(packets_stream)?
        .into_iter()
        .map(|frame| decode_packet(frame, unread_bytes))
        .collect()
}

/// Decodes one frame into its packet and the header it carried.
///
/// # Errors
///
/// Returns a [`NetworkCodecError`] when the frame fails to deserialize, or leaves bytes unread
/// under [`UnreadBytes::Reject`].
pub fn decode_packet<T: Packets>(
    frame: &[u8],
    unread_bytes: UnreadBytes,
) -> Result<(PacketHeader, T), NetworkCodecError> {
    let mut frame = Cursor::new(frame);
    let (packet, header) = T::deserialize(&mut frame)?;

    let unread = frame.get_ref().len() - cursor_position(&frame);
    if unread > 0 && unread_bytes == UnreadBytes::Reject {
        return Err(ProtoCodecError::LeftOvers(unread).into());
    }
    if unread > 0 {
        tracing::warn!(
            "packet {} deserializer left {unread} unread bytes, skipping to next packet",
            header.packet_id,
        );
    }

    Ok((header, packet))
}

/// The position of `cursor` over a slice, which cannot exceed the slice length.
fn cursor_position(cursor: &Cursor<&[u8]>) -> usize {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the position of a cursor over a slice is at most the slice length, a usize"
    )]
    let position = cursor.position() as usize;
    position
}

/// Splits a decrypted, decompressed batch into its frames, without the length prefixes.
///
/// # Errors
///
/// Returns a [`NetworkCodecError`] when a length prefix cannot be read, is zero, runs past the end
/// of the batch, or the batch holds more than [`MAX_BATCH_PACKETS`] frames.
pub fn split_batch(batch: &[u8]) -> Result<Vec<&[u8]>, NetworkCodecError> {
    let mut stream = Cursor::new(batch);
    let mut frames = vec![];

    while cursor_position(&stream) < batch.len() {
        let declared = <u32 as ProtoCodecVAR>::deserialize(&mut stream)?;
        let rest = &batch[cursor_position(&stream)..];
        if declared == 0 {
            return Err(NetworkCodecError::EmptyPacket);
        }
        if frames.len() == MAX_BATCH_PACKETS {
            return Err(NetworkCodecError::PacketCountExceeded(MAX_BATCH_PACKETS));
        }
        let Some(frame) = rest.get(..declared as usize) else {
            return Err(NetworkCodecError::PacketLengthOutOfBounds {
                declared,
                remaining: rest.len(),
            });
        };

        frames.push(frame);
        stream.set_position(stream.position() + u64::from(declared));
    }

    Ok(frames)
}

/// Prefixes each of `frames` with its length and concatenates them into a batch.
///
/// # Panics
///
/// Panics if serializing a length into a `Vec` fails, which cannot happen.
pub fn join_batch<F: AsRef<[u8]>>(frames: &[F]) -> Vec<u8> {
    let mut batch = Vec::with_capacity(frames.iter().map(|frame| frame.as_ref().len() + 5).sum());

    for frame in frames {
        let frame = frame.as_ref();
        #[expect(
            clippy::cast_possible_truncation,
            reason = "a frame is never 4 GiB or larger"
        )]
        let len = frame.len() as u32;
        <u32 as ProtoCodecVAR>::serialize(&len, &mut batch).expect("writing to a Vec cannot fail");
        batch.extend_from_slice(frame);
    }

    batch
}

/// Compresses a batch when `compression` is set.
///
/// # Errors
///
/// Returns a [`NetworkCodecError`] when compression fails.
pub fn compress_packets(
    mut packet_stream: Vec<u8>,
    compression: Option<&Compression>,
) -> Result<Vec<u8>, NetworkCodecError> {
    if let Some(compression) = compression {
        packet_stream = compression.compress(&packet_stream)?;
    }

    Ok(packet_stream)
}

/// Decompresses a batch when `compression` is set, failing if it inflates past `max_len` bytes.
///
/// # Errors
///
/// Returns a [`NetworkCodecError`] when decompression fails or the batch inflates past `max_len`.
pub fn decompress_packets(
    mut packet_stream: Vec<u8>,
    compression: Option<&Compression>,
    max_len: usize,
) -> Result<Vec<u8>, NetworkCodecError> {
    if let Some(compression) = compression {
        packet_stream = compression.decompress(packet_stream, max_len)?;
    }

    Ok(packet_stream)
}

/// Encrypts a batch when `encryption` is set.
///
/// # Errors
///
/// Returns a [`NetworkCodecError`] when encryption fails.
pub fn encrypt_packets(
    mut packet_stream: Vec<u8>,
    encryption: Option<&mut Encryption>,
) -> Result<Vec<u8>, NetworkCodecError> {
    if let Some(encryption) = encryption {
        packet_stream = encryption.encrypt(&packet_stream)?;
    }

    Ok(packet_stream)
}

/// Decrypts a batch when `encryption` is set.
///
/// # Errors
///
/// Returns a [`NetworkCodecError`] when the batch is too short or its trailer does not match.
pub fn decrypt_packets(
    mut packet_stream: Vec<u8>,
    encryption: Option<&mut Encryption>,
) -> Result<Vec<u8>, NetworkCodecError> {
    if let Some(encryption) = encryption {
        packet_stream = encryption.decrypt(packet_stream)?;
    }

    Ok(packet_stream)
}

#[cfg(test)]
mod frame_tests {
    use super::*;

    #[test]
    fn joined_frames_split_back_into_the_same_frames() {
        let frames = [vec![1u8, 2, 3], vec![9u8; 200], vec![7u8]];

        let batch = join_batch(&frames);

        assert_eq!(
            split_batch(&batch).unwrap(),
            frames.iter().map(Vec::as_slice).collect::<Vec<_>>()
        );
    }

    #[test]
    fn batch_with_a_zero_length_frame_is_an_error() {
        assert!(matches!(
            split_batch(&[0]),
            Err(NetworkCodecError::EmptyPacket)
        ));
    }

    #[test]
    fn batch_whose_frame_overruns_it_is_an_error() {
        assert!(matches!(
            split_batch(&[5, 1, 2]),
            Err(NetworkCodecError::PacketLengthOutOfBounds {
                declared: 5,
                remaining: 2
            })
        ));
    }

    #[test]
    fn empty_batch_has_no_frames() {
        assert!(split_batch(&[]).unwrap().is_empty());
    }

    #[test]
    fn batch_with_more_frames_than_the_cap_is_an_error() {
        assert!(matches!(
            split_batch(&[1, 4].repeat(MAX_BATCH_PACKETS + 1)),
            Err(NetworkCodecError::PacketCountExceeded(MAX_BATCH_PACKETS))
        ));
        assert_eq!(
            split_batch(&[1, 4].repeat(MAX_BATCH_PACKETS))
                .unwrap()
                .len(),
            MAX_BATCH_PACKETS
        );
    }
}

#[cfg(test)]
mod unread_tests {
    use super::*;
    use crate::test_helpers::login_success;
    use bedrock_protocol::V2225;

    fn frame_with_a_trailing_byte() -> Vec<u8> {
        let batch = encode_packets::<V2225>(&[login_success()], None, None).unwrap();
        let mut frame = split_batch(&batch).unwrap()[0].to_vec();
        frame.push(0);
        frame
    }

    #[test]
    fn unread_bytes_are_tolerated_when_warning() {
        assert!(decode_packet::<V2225>(&frame_with_a_trailing_byte(), UnreadBytes::Warn).is_ok());
    }

    #[test]
    fn unread_bytes_are_an_error_when_rejecting() {
        assert!(matches!(
            decode_packet::<V2225>(&frame_with_a_trailing_byte(), UnreadBytes::Reject),
            Err(NetworkCodecError::ProtoCodecError(
                ProtoCodecError::LeftOvers(1)
            ))
        ));
    }
}

#[cfg(test)]
mod header_tests {
    use super::*;
    use crate::test_helpers::login_success;
    use bedrock_protocol::V2225;

    fn split_screen_header() -> PacketHeader {
        PacketHeader {
            packet_id: 0,
            sender_sub_client_id: 1,
            target_sub_client_id: 2,
        }
    }

    #[test]
    fn sub_client_ids_survive_encode_then_decode() {
        let packet = login_success();
        let batch =
            encode_packets_with_header::<V2225>(&[packet], &split_screen_header(), None, None)
                .unwrap();

        let decoded =
            decode_packets::<V2225>(batch, None, None, MAX_BATCH_LEN, UnreadBytes::Reject).unwrap();

        assert_eq!(decoded.len(), 1);
        let (header, _) = &decoded[0];
        assert_eq!(header.sender_sub_client_id, 1);
        assert_eq!(header.target_sub_client_id, 2);
        assert_eq!(header.packet_id, 2);
    }

    #[test]
    fn every_frame_in_a_batch_carries_the_chosen_ids() {
        let packets = vec![login_success(); 3];
        let batch =
            encode_packets_with_header::<V2225>(&packets, &split_screen_header(), None, None)
                .unwrap();

        let decoded =
            decode_packets::<V2225>(batch, None, None, MAX_BATCH_LEN, UnreadBytes::Reject).unwrap();

        assert!(decoded.iter().all(
            |(header, _)| header.sender_sub_client_id == 1 && header.target_sub_client_id == 2
        ));
    }

    #[test]
    fn default_encoding_keeps_the_zero_sub_client_bytes() {
        let packet = login_success();

        let batch = encode_packets::<V2225>(&[packet], None, None).unwrap();

        assert_eq!(batch, [5, 2, 0, 0, 0, 0]);
    }

    #[test]
    fn chosen_ids_change_only_the_header_bytes() {
        let packet = login_success();

        let batch =
            encode_packets_with_header::<V2225>(&[packet], &split_screen_header(), None, None)
                .unwrap();

        assert_eq!(batch, [6, 0x82, 0x48, 0, 0, 0, 0]);
    }
}

#[cfg(test)]
mod each_tests {
    use super::*;
    use crate::test_helpers::login_success;
    use bedrock_protocol::V2225;

    const GARBAGE: [u8; 2] = [2, 1];

    fn good_frame() -> Vec<u8> {
        let batch = encode_packets::<V2225>(&[login_success()], None, None).unwrap();
        split_batch(&batch).unwrap()[0].to_vec()
    }

    fn decode_each(frames: &[Vec<u8>]) -> Vec<(PacketHeader, Result<V2225, NetworkCodecError>)> {
        decode_packets_each::<V2225>(
            join_batch(frames),
            None,
            None,
            MAX_BATCH_LEN,
            UnreadBytes::Reject,
        )
        .unwrap()
    }

    fn outcomes(frames: &[Vec<u8>]) -> Vec<bool> {
        decode_each(frames)
            .iter()
            .map(|(_, result)| result.is_ok())
            .collect()
    }

    #[test]
    fn a_bad_middle_frame_leaves_its_neighbours_decoded() {
        let frames = [good_frame(), GARBAGE.to_vec(), good_frame()];

        assert_eq!(outcomes(&frames), [true, false, true]);
    }

    #[test]
    fn a_bad_first_frame_leaves_the_rest_decoded() {
        let frames = [GARBAGE.to_vec(), good_frame(), good_frame()];

        assert_eq!(outcomes(&frames), [false, true, true]);
    }

    #[test]
    fn a_bad_last_frame_leaves_the_rest_decoded() {
        let frames = [good_frame(), good_frame(), GARBAGE.to_vec()];

        assert_eq!(outcomes(&frames), [true, true, false]);
    }

    #[test]
    fn a_bad_frame_keeps_its_header() {
        let decoded = decode_each(&[GARBAGE.to_vec()]);

        assert_eq!(decoded[0].0.packet_id, 2);
    }

    #[test]
    fn unread_bytes_fail_only_their_own_frame_when_rejecting() {
        let mut trailing = good_frame();
        trailing.push(0);

        assert_eq!(outcomes(&[good_frame(), trailing]), [true, false]);
    }

    #[test]
    fn framing_errors_still_fail_the_whole_batch() {
        assert!(matches!(
            decode_packets_each::<V2225>(
                vec![5, 1, 2],
                None,
                None,
                MAX_BATCH_LEN,
                UnreadBytes::Warn
            ),
            Err(NetworkCodecError::PacketLengthOutOfBounds { .. })
        ));
    }
}
