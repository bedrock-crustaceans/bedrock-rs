use bedrock_addon_core::schema::Schema;
use facet::Facet;

#[derive(Facet, Schema)]
#[repr(u8)]
enum Shape {
    Unit,
    Data(i32, i32),
}

#[derive(Facet, Schema)]
struct Fixed<const N: usize> {
    value: [u8; N],
}

#[derive(Facet, Schema)]
struct Generic<T> {
    value: T,
}

fn main() {}
