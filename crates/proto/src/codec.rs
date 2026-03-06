use crate::compression::Compression;
use crate::encryption::Encryption;
use crate::helper::ProtoHelper;
use bedrockrs_proto_core::GamePacketsAll;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::sub_client::SubClientID;
use std::io::Cursor;

pub fn encode_gamepackets<T: ProtoHelper>(
    gamepackets: &[T::GamePacketType],
    compression: Option<&Compression>,
    encryption: Option<&mut Encryption>,
) -> Result<Vec<u8>, ProtoCodecError> {
    log::trace!("Encoding gamepackets");

    let mut gamepacket_stream = batch_gamepackets::<T>(gamepackets)?;
    gamepacket_stream = compress_gamepackets::<T>(gamepacket_stream, compression)?;
    gamepacket_stream = encrypt_gamepackets::<T>(gamepacket_stream, encryption)?;

    Ok(gamepacket_stream)
}

pub fn decode_gamepackets<T: ProtoHelper>(
    mut gamepacket_stream: Vec<u8>,
    compression: Option<&Compression>,
    encryption: Option<&mut Encryption>,
) -> Result<Vec<T::GamePacketType>, ProtoCodecError> {
    log::trace!("Decoding gamepackets");

    gamepacket_stream = decrypt_gamepackets::<T>(gamepacket_stream, encryption)?;
    gamepacket_stream = decompress_gamepackets::<T>(gamepacket_stream, compression)?;
    let gamepackets = separate_gamepackets::<T>(gamepacket_stream)?;

    Ok(gamepackets)
}

fn batch_gamepackets<T: ProtoHelper>(
    gamepackets: &[T::GamePacketType],
) -> Result<Vec<u8>, ProtoCodecError> {
    let gamepacket_stream_size = gamepackets
        .iter()
        .map(T::GamePacketType::get_size_prediction)
        .sum::<usize>();

    // Create a Vector with the predicted size
    let mut gamepacket_stream = Vec::with_capacity(gamepacket_stream_size);

    // Batch all gamepackets together
    gamepackets.iter().try_for_each(|gamepacket| {
        gamepacket.pk_serialize(
            &mut gamepacket_stream,
            SubClientID::PrimaryClient,
            SubClientID::PrimaryClient,
        )
    })?;

    Ok(gamepacket_stream)
}

fn separate_gamepackets<T: ProtoHelper>(
    gamepacket_stream: Vec<u8>,
) -> Result<Vec<T::GamePacketType>, ProtoCodecError> {
    let mut gamepacket_stream = Cursor::new(gamepacket_stream.as_slice());
    let mut gamepackets = vec![];

    loop {
        if gamepacket_stream.position() == gamepacket_stream.get_ref().len() as u64 {
            break;
        }

        gamepackets.push(T::GamePacketType::pk_deserialize(&mut gamepacket_stream)?.0);
    }

    Ok(gamepackets)
}

pub fn compress_gamepackets<T: ProtoHelper>(
    mut gamepacket_stream: Vec<u8>,
    compression: Option<&Compression>,
) -> Result<Vec<u8>, ProtoCodecError> {
    if let Some(compression) = compression {
        gamepacket_stream = compression.compress(gamepacket_stream)?;
    }

    Ok(gamepacket_stream)
}

pub fn decompress_gamepackets<T: ProtoHelper>(
    mut gamepacket_stream: Vec<u8>,
    compression: Option<&Compression>,
) -> Result<Vec<u8>, ProtoCodecError> {
    if let Some(compression) = compression {
        gamepacket_stream = compression.decompress(gamepacket_stream)?;
    }

    Ok(gamepacket_stream)
}

pub fn encrypt_gamepackets<T: ProtoHelper>(
    mut gamepacket_stream: Vec<u8>,
    encryption: Option<&mut Encryption>,
) -> Result<Vec<u8>, ProtoCodecError> {
    if let Some(encryption) = encryption {
        gamepacket_stream = encryption.encrypt(gamepacket_stream)?;
    }

    Ok(gamepacket_stream)
}

pub fn decrypt_gamepackets<T: ProtoHelper>(
    mut gamepacket_stream: Vec<u8>,
    encryption: Option<&mut Encryption>,
) -> Result<Vec<u8>, ProtoCodecError> {
    if let Some(encryption) = encryption {
        gamepacket_stream = encryption.decrypt(gamepacket_stream)?;
    }

    Ok(gamepacket_stream)
}
