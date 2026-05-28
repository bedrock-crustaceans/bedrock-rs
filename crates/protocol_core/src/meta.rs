#[cfg(feature = "packet-meta")]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum PacketDirection {
    ClientToServer,
    ServerToClient,
}

#[cfg(feature = "packet-meta")]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct PacketMeta {
    pub name: &'static str,
    pub direction: Option<PacketDirection>,
}
