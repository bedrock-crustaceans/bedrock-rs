//! Byte ranges of derived `ProtoCodec` fields, recorded while decoding through [`trace`].
//! Without the `packet-trace` feature the derive hooks compile to nothing.

#[cfg(feature = "packet-trace")]
pub use enabled::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct FieldSpan {
    pub name: &'static str,
    pub index: Option<usize>,
    pub depth: usize,
    pub start: usize,
    pub end: usize,
}

#[cfg(not(feature = "packet-trace"))]
pub struct FieldGuard;

#[cfg(not(feature = "packet-trace"))]
#[inline(always)]
pub fn field(_name: &'static str, _index: Option<usize>) -> FieldGuard {
    FieldGuard
}

#[cfg(feature = "packet-trace")]
mod enabled {
    use super::FieldSpan;
    use std::cell::RefCell;
    use std::io::Read;

    #[derive(Default)]
    struct Tracer {
        pos: usize,
        open: Vec<usize>,
        spans: Vec<FieldSpan>,
    }

    thread_local! {
        static TRACER: RefCell<Option<Tracer>> = const { RefCell::new(None) };
    }

    pub struct TraceReader<R> {
        inner: R,
    }

    impl<R: Read> Read for TraceReader<R> {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            let n = self.inner.read(buf)?;
            TRACER.with_borrow_mut(|t| {
                if let Some(t) = t {
                    t.pos += n;
                }
            });
            Ok(n)
        }
    }

    pub fn trace<R: Read, T>(
        stream: R,
        decode: impl FnOnce(&mut TraceReader<R>) -> T,
    ) -> (T, Vec<FieldSpan>) {
        let previous = TRACER.replace(Some(Tracer::default()));
        let value = decode(&mut TraceReader { inner: stream });
        let tracer = TRACER.replace(previous).unwrap_or_default();
        (value, tracer.spans)
    }

    pub struct FieldGuard {
        active: bool,
    }

    pub fn field(name: &'static str, index: Option<usize>) -> FieldGuard {
        let active = TRACER.with_borrow_mut(|t| {
            let Some(t) = t else { return false };
            t.open.push(t.spans.len());
            t.spans.push(FieldSpan {
                name,
                index,
                depth: t.open.len() - 1,
                start: t.pos,
                end: t.pos,
            });
            true
        });
        FieldGuard { active }
    }

    impl Drop for FieldGuard {
        fn drop(&mut self) {
            if !self.active {
                return;
            }
            TRACER.with_borrow_mut(|t| {
                if let Some(t) = t
                    && let Some(i) = t.open.pop()
                {
                    t.spans[i].end = t.pos;
                }
            });
        }
    }
}
