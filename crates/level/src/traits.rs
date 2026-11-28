use std::io::{Cursor, Read};

pub trait CursorExt {
    fn has_remaining(&self) -> bool;
}

impl<R: AsRef<[u8]>> CursorExt for Cursor<R>
where
    Cursor<R>: Read,
{
    fn has_remaining(&self) -> bool {
        self.get_ref().as_ref().len() as u64 != self.position()
    }
}
