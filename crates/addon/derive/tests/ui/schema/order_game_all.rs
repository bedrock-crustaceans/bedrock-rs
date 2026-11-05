use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

#[derive(Facet, Schema)]
#[facet(schema::order = "game")]
#[repr(u8)]
enum Short {
    North,
    South,
    East,
}

impl Short {
    const ALL: [Short; 2] = [Short::North, Short::East];
}

#[derive(Facet, Schema)]
#[facet(schema::order = "game")]
#[repr(u8)]
enum Twice {
    North,
    South,
}

impl Twice {
    const ALL: &'static [Twice] = &[Twice::North, Twice::North];
}

fn main() {}
