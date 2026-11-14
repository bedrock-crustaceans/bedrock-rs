#[cfg(feature = "packet-dyn")]
use crate::PacketDyn;
#[cfg(feature = "packet-meta")]
use crate::PacketMeta;

#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct UnknownPacket {
    pub id: u16,
    pub buf: Box<[u8]>,
}

#[cfg(feature = "packet-dyn")]
impl PacketDyn for UnknownPacket {
    #[inline]
    fn id(&self) -> u16 {
        self.id
    }

    #[cfg(feature = "packet-meta")]
    #[inline]
    fn meta(&self) -> &'static PacketMeta {
        const META: PacketMeta = PacketMeta {
            name: "UnknownPacket",
            direction: None,
        };

        &META
    }
}
