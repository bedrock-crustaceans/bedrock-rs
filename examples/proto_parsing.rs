mod common;

use crate::common::logger::setup_logger;
use bedrockrs::proto::codec::decode_gamepackets;
use bedrockrs::proto::v800::ProtoHelperV800;
use bedrockrs_proto::compression::Compression;

fn main() {
    setup_logger().unwrap();

    let bytes = vec![
        1, 0, 0, 2, 254, 243, 252, 35, 32, 10, 0, 0, 123, 34, 99, 104, 97, 105, 110, 34, 58, 91,
        34, 101, 121, 74, 104, 98, 71, 99, 105, 79, 105, 74, 70, 85, 122, 77, 52, 78, 67, 73, 115,
        73, 110, 103, 49, 100, 83, 73, 54, 73, 107, 49, 73, 87, 88, 100, 70, 81, 86, 108, 73, 83,
        50, 57, 97, 83, 88, 112, 113, 77, 69, 78, 66, 85, 86, 108, 71, 83, 122, 82, 70, 82, 85, 70,
        68, 83, 85, 82, 90, 90, 48, 70, 70, 76, 50, 82, 89,
    ];

    let result = decode_gamepackets::<ProtoHelperV800>(bytes, Some(&Compression::None), None);

    println!("{:?}", result);
}
