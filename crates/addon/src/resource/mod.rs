use std::fs;
use std::path::Path;
use std::sync::Arc;

use crate::Addon;
use crate::error::AddonError;
use crate::error::AddonError::{IOError, JsonError};
use crate::language::Languages;
use crate::manifest::AddonManifest;

#[derive(Debug, Clone)]
pub struct ResourcePack {
    pub manifest: AddonManifest,
    pub languages: Languages,
}

impl Addon for ResourcePack {
    fn manifest(&self) -> &AddonManifest {
        &self.manifest
    }

    fn import(path: impl AsRef<Path>) -> Result<Self, AddonError>
    where
        Self: Sized,
    {
        let path = path.as_ref().to_path_buf();

        // Manifest
        let manifest_path = path.join("manifest.json");
        let manifest = fs::read_to_string(&manifest_path)
            .map_err(|e| IOError(Arc::new(e), manifest_path.clone()))?;
        let manifest: AddonManifest =
            serde_json::from_str(&manifest).map_err(|e| JsonError(Arc::new(e), manifest_path))?;

        // Languages
        let languages = Languages::import(path.join("texts"))?;

        Ok(Self {
            manifest,
            languages,
        })
    }

    fn export(_path: impl AsRef<Path>) -> Result<Self, AddonError>
    where
        Self: Sized,
    {
        unimplemented!()
    }

    fn merge(_addons: Vec<Self>) -> Self
    where
        Self: Sized,
    {
        unimplemented!()
    }
}
