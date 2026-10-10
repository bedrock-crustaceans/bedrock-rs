use crate::error::NetworkCodecError;

pub struct RakNetGamePacket;

impl RakNetGamePacket {
    pub const ID: u8 = 0xfe;

    pub fn wrap(batch: &[u8]) -> Vec<u8> {
        let mut datagram = Vec::with_capacity(batch.len() + 1);
        datagram.push(Self::ID);
        datagram.extend_from_slice(batch);
        datagram
    }

    pub fn unwrap(datagram: &[u8]) -> Result<&[u8], NetworkCodecError> {
        match datagram.split_first() {
            Some((&Self::ID, batch)) => Ok(batch),
            Some((&found, _)) => Err(NetworkCodecError::InvalidGamePacketHeader(found)),
            None => Err(NetworkCodecError::EmptyPacket),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrapped_batch_unwraps_to_the_same_batch() {
        let datagram = RakNetGamePacket::wrap(&[1, 2, 3]);

        assert_eq!(datagram[0], RakNetGamePacket::ID);
        assert_eq!(RakNetGamePacket::unwrap(&datagram).unwrap(), [1, 2, 3]);
    }

    #[test]
    fn datagram_with_another_header_is_rejected() {
        assert!(matches!(
            RakNetGamePacket::unwrap(&[0x13, 1]),
            Err(NetworkCodecError::InvalidGamePacketHeader(0x13))
        ));
        assert!(matches!(
            RakNetGamePacket::unwrap(&[]),
            Err(NetworkCodecError::EmptyPacket)
        ));
    }
}
