use bedrockrs_network::connection::shard::arc::ConnectionShared;
use bedrockrs_proto_core::Packets;
use shipyard::Component;

#[derive(Component)]
pub struct Connected<T: Packets + 'static>
where
    T: Sync + Send,
{
    pub connection: ConnectionShared<T>,
}
