use crate::compression::Compression;
use crate::encryption::Encryption;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::{PacketHeader, Packets};
use std::io::Cursor;

pub fn encode_packets<T: Packets>(
    packets: &[T],
    compression: Option<&Compression>,
    encryption: Option<&mut Encryption>,
) -> Result<Vec<u8>, ProtoCodecError> {
    log::trace!("Encoding packets");

    let mut packets_stream = batch_packets::<T>(packets)?;
    packets_stream = compress_packets(packets_stream, compression)?;
    packets_stream = encrypt_packets(packets_stream, encryption)?;

    Ok(packets_stream)
}

pub fn decode_packets<T: Packets>(
    mut packets_stream: Vec<u8>,
    compression: Option<&Compression>,
    encryption: Option<&mut Encryption>,
) -> Result<Vec<T>, ProtoCodecError> {
    log::trace!("Decoding packets");

    packets_stream = decrypt_packets(packets_stream, encryption)?;
    packets_stream = decompress_packets(packets_stream, compression)?;
    let packets = separate_packets::<T>(packets_stream)?;

    Ok(packets)
}

fn batch_packets<T: Packets>(packets: &[T]) -> Result<Vec<u8>, ProtoCodecError> {
    let packets_stream_size = packets
        .iter()
        .map(|p| {
            p.size_hint(&PacketHeader {
                packet_id: p.id(),
                sender_sub_client_id: 0,
                target_sub_client_id: 0,
            })
        })
        .sum::<usize>();

    let mut packets_stream = Vec::with_capacity(packets_stream_size);

    packets.iter().try_for_each(|packet| {
        packet.serialize(
            &PacketHeader {
                packet_id: packet.id(),
                sender_sub_client_id: 0,
                target_sub_client_id: 0,
            },
            &mut packets_stream,
        )
    })?;

    Ok(packets_stream)
}

fn separate_packets<T: Packets>(packets_stream: Vec<u8>) -> Result<Vec<T>, ProtoCodecError> {
    let mut packets_stream = Cursor::new(packets_stream.as_slice());
    let mut packets = vec![];

    loop {
        if packets_stream.position() == packets_stream.get_ref().len() as u64 {
            break;
        }

        packets.push(T::deserialize(&mut packets_stream)?.0);
    }

    Ok(packets)
}

pub fn compress_packets(
    mut packet_stream: Vec<u8>,
    compression: Option<&Compression>,
) -> Result<Vec<u8>, ProtoCodecError> {
    if let Some(compression) = compression {
        packet_stream = compression.compress(packet_stream)?;
    }

    Ok(packet_stream)
}

pub fn decompress_packets(
    mut packet_stream: Vec<u8>,
    compression: Option<&Compression>,
) -> Result<Vec<u8>, ProtoCodecError> {
    if let Some(compression) = compression {
        packet_stream = compression.decompress(packet_stream)?;
    }

    Ok(packet_stream)
}

pub fn encrypt_packets(
    mut packet_stream: Vec<u8>,
    encryption: Option<&mut Encryption>,
) -> Result<Vec<u8>, ProtoCodecError> {
    if let Some(encryption) = encryption {
        packet_stream = encryption.encrypt(packet_stream)?;
    }

    Ok(packet_stream)
}

pub fn decrypt_packets(
    mut packet_stream: Vec<u8>,
    encryption: Option<&mut Encryption>,
) -> Result<Vec<u8>, ProtoCodecError> {
    if let Some(encryption) = encryption {
        packet_stream = encryption.decrypt(packet_stream)?;
    }

    Ok(packet_stream)
}
