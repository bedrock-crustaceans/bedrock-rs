//! Packs built from small in-memory manifests for the `pack` tests.

#![allow(dead_code, reason = "each test file uses part of the helpers")]

use bedrock_addon::pack::{OpenOptions, Pack, PackIdVersion, PackType};
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::SemVersion;
use bytes::Bytes;
use uuid::Uuid;

/// One pack to build: a format-2 manifest and some files.
#[derive(Clone, Debug)]
pub struct Spec {
    pub name: String,
    pub uuid: Uuid,
    pub version: [u16; 3],
    /// Module types, in manifest order (`data`, `resources`, `script`, …).
    pub modules: Vec<&'static str>,
    /// Pack dependencies.
    pub dependencies: Vec<(Uuid, [u16; 3])>,
    /// `(folder_name, name, memory_tier)`.
    pub subpacks: Vec<(&'static str, &'static str, u8)>,
    pub files: Vec<(&'static str, &'static [u8])>,
}

/// A behaviour pack `n`, version 1.0.0.
pub fn bp(n: u128) -> Spec {
    Spec {
        name: format!("bp {n}"),
        uuid: Uuid::from_u128(n),
        version: [1, 0, 0],
        modules: vec!["data"],
        dependencies: Vec::new(),
        subpacks: Vec::new(),
        files: Vec::new(),
    }
}

/// A resource pack `n`, version 1.0.0.
pub fn rp(n: u128) -> Spec {
    Spec {
        name: format!("rp {n}"),
        modules: vec!["resources"],
        ..bp(n)
    }
}

impl Spec {
    pub fn version(mut self, version: [u16; 3]) -> Self {
        self.version = version;
        self
    }

    pub fn depends_on(mut self, other: &Spec) -> Self {
        self.dependencies.push((other.uuid, other.version));
        self
    }

    pub fn subpack(mut self, folder_name: &'static str, name: &'static str, tier: u8) -> Self {
        self.subpacks.push((folder_name, name, tier));
        self
    }

    pub fn file(mut self, path: &'static str, bytes: &'static [u8]) -> Self {
        self.files.push((path, bytes));
        self
    }

    pub fn id(&self) -> PackIdVersion {
        let pack_type = if self.modules.last() == Some(&"resources") {
            PackType::Resource
        } else {
            PackType::Behavior
        };
        let [major, minor, patch] = self.version;
        PackIdVersion::new(self.uuid, SemVersion::new(major, minor, patch), pack_type)
    }

    /// The manifest text.
    pub fn manifest(&self) -> String {
        let version = |[a, b, c]: [u16; 3]| format!("[{a}, {b}, {c}]");
        let modules: Vec<String> = self
            .modules
            .iter()
            .enumerate()
            .map(|(index, kind)| {
                let module = Uuid::from_u128(
                    self.uuid.as_u128() ^ (0xabcd_u128 << 64 | (index as u128 + 1)),
                );
                format!(r#"{{ "type": "{kind}", "uuid": "{module}", "version": [1, 0, 0] }}"#)
            })
            .collect();
        let dependencies: Vec<String> = self
            .dependencies
            .iter()
            .map(|(uuid, v)| format!(r#"{{ "uuid": "{uuid}", "version": {} }}"#, version(*v)))
            .collect();
        let subpacks: Vec<String> = self
            .subpacks
            .iter()
            .map(|(folder, name, tier)| {
                // Format 2 writes the legacy `memory_tier`: the smallest value of each tier.
                let legacy = [0, 11, 12, 18, 24, 32][usize::from(*tier).min(5)];
                format!(
                    r#"{{ "folder_name": "{folder}", "name": "{name}", "memory_tier": {legacy} }}"#
                )
            })
            .collect();
        format!(
            r#"{{
  "format_version": 2,
  "header": {{
    "name": "{name}",
    "description": "",
    "uuid": "{uuid}",
    "version": {version},
    "min_engine_version": [1, 21, 0]
  }},
  "modules": [{modules}],
  "dependencies": [{dependencies}],
  "subpacks": [{subpacks}]
}}"#,
            name = self.name,
            uuid = self.uuid,
            version = version(self.version),
            modules = modules.join(", "),
            dependencies = dependencies.join(", "),
            subpacks = subpacks.join(", "),
        )
    }

    /// The manifest, parsed with no error-level finding (the specs are valid packs).
    pub fn parsed_manifest(&self) -> bedrock_addon::manifest::Manifest {
        let text = self.manifest();
        let document = bedrock_addon::manifest::Manifest::parse(
            &PackPath::from_static("manifest.json"),
            text.as_bytes(),
            &bedrock_addon::manifest::CapabilityRegistry::default(),
        )
        .unwrap_or_else(|rejected| panic!("{text}\n{rejected:?}"));
        assert!(
            !document.diagnostics().has_errors(),
            "{text}\n{:?}",
            document.diagnostics()
        );
        document.into_value()
    }

    /// The pack, opened from memory.
    pub fn open(&self) -> Pack {
        let mut files: Vec<(PackPath, Bytes)> = vec![(
            PackPath::from_static("manifest.json"),
            Bytes::from(self.manifest()),
        )];
        files.extend(
            self.files
                .iter()
                .map(|(path, bytes)| (PackPath::from_static(path), Bytes::from_static(bytes))),
        );
        Pack::from_files(files, &OpenOptions::default()).expect("the pack opens")
    }
}
