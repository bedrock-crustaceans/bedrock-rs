//! A fallback getter's value is the point of calling it: dropping it is a `must_use` warning.
#![deny(unused_must_use)]

use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

#[derive(Facet, Schema)]
struct Herd {
    #[facet(schema::fallback = 1)]
    min: Option<i32>,
}

fn main() {
    let herd = Herd { min: None };
    herd.min();
}
