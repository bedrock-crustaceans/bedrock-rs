macro_rules! export {
    ($name:ident) => {
        mod $name;
        pub use $name::*;
    };
}

export!(connection_fail_reason);
export!(persona_piece_type);
