// The families of the legacy probe: every family with a step, the readers of its document
// types and the keys a step's payload is wrapped in. Included by `writable_legacy.rs` and the
// binaries of the large families.

/// One test per row: `name: [test and fixture folders], [payload keys],
/// [(reader name, type, pack path, context), …], steps run alone;`. A row `test = "family" in
/// (from, below): …` probes the documents of the family in that band of versions.
macro_rules! legacy_families {
    ($( $test:ident $(= $name:literal in ($from:expr, $below:expr))? :
        [$($folder:literal),*], [$($key:literal),*],
        [$(($reader:literal, $type:ty, $path:literal, $cx:expr)),* $(,)?], $runs:expr; )*) => {
        $(
            #[test]
            fn $test() {
                audit(&Family {
                    name: legacy_families!(@name $test $($name)?),
                    folders: &[$($folder),*],
                    keys: &[$($key),*],
                    readers: vec![$(reader::<$type>($reader, $path, $cx)),*],
                    runs: $runs,
                    band: legacy_families!(@band $($from, $below)?),
                });
            }
        )*
    };
    (@name $test:ident) => { stringify!($test) };
    (@name $test:ident $name:literal) => { $name };
    (@band) => { None };
    (@band $from:expr, $below:expr) => { Some(($from, $below)) };
}

/// The steps of `types`, each run alone in the context `load` gives.
macro_rules! steps {
    ($load:expr; $($type:ty),* $(,)?) => {
        vec![$(step::<$type>($load)),*]
    };
}
