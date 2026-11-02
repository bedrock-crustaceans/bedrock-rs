use std::io::{Cursor, Seek, SeekFrom};

/// Adds extra functionality to the [`Seek`] trait.
///
/// [`Seek`]: std::io::Seek
pub trait SeekExt {
    /// Performs multiple seeks to figure out the length of this buffer.
    ///
    /// Currently this is a direct copy of [`stream_len`]. It is called
    /// `stream_len_ext` to ensure it does not collide with the standard library method.
    ///
    /// [`stream_len`]: std::io::Seek::stream_len
    fn stream_len_ext(&mut self) -> std::io::Result<u64>;
}

impl<R> SeekExt for Cursor<R>
where
    Cursor<R>: Seek,
{
    fn stream_len_ext(&mut self) -> std::io::Result<u64> {
        let old_pos = self.stream_position()?;
        let len = self.seek(SeekFrom::End(0))?;

        if old_pos != len {
            self.seek(SeekFrom::Start(old_pos))?;
        }

        Ok(len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_len_ext_reports_the_length_and_keeps_the_position() {
        let mut cursor = Cursor::new([1_u8, 2, 3, 4, 5]);
        cursor.seek(SeekFrom::Start(2)).unwrap();

        assert_eq!(cursor.stream_len_ext().unwrap(), 5);
        assert_eq!(cursor.stream_position().unwrap(), 2);
    }
}
