//! The empty Molang walks of the leaf types a `#[derive(Schema)]` struct holds: the derive
//! hands every field that is not a Molang slot to `MolangVisitor::nested`, so numbers,
//! `bool`, `String`, `json::Value` and `EventTrigger` walk nothing (and `Option`, `Vec` and
//! `IndexMap` of them forward to these).

crate::impl_no_molang!(
    i8,
    i16,
    i32,
    i64,
    isize,
    u8,
    u16,
    u32,
    u64,
    usize,
    f32,
    f64,
    bool,
    char,
    String,
    crate::json::Value,
    crate::shared::EventTrigger,
);
