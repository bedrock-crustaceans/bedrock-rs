use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use json_comments::CommentSettings;
use walkdir::WalkDir;

use crate::Addon;
use crate::behavior::blocks::AddonBlock;
use crate::behavior::items::AddonItem;
use crate::error::AddonError;
use crate::error::AddonError::{IOError, JsonError};
use crate::language::Languages;
use crate::manifest::AddonManifest;

pub mod blocks;
pub mod items;
pub mod menu_category;

#[derive(Debug, Clone)]
pub struct BehaviorPack {
    pub manifest: AddonManifest,
    pub languages: Languages,
    pub blocks: HashMap<PathBuf, AddonBlock>,
    pub items: HashMap<PathBuf, AddonItem>,
    pub functions: HashMap<PathBuf, String>,
    pub scripts: HashMap<PathBuf, String>,
}

impl Addon for BehaviorPack {
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

        // Blocks
        let blocks_path = path.join("blocks");
        let mut blocks = HashMap::new();

        // If dir exists, read all blocks
        if blocks_path.is_dir() {
            'blocks_walk: for entry in WalkDir::new(&blocks_path)
                .into_iter()
                .filter(|v| {
                    if let Ok(v) = v {
                        v.file_type().is_file()
                    } else {
                        false
                    }
                })
                .flatten()
            {
                let block_path = entry.path();

                let block = fs::read_to_string(block_path)
                    .map_err(|e| IOError(Arc::new(e), block_path.to_path_buf()))?;

                // If the file is empty skip it
                if block.trim().is_empty() {
                    continue 'blocks_walk;
                }

                // Strip all c-style comments
                let stripper = CommentSettings::c_style().strip_comments(block.as_bytes());

                let block: AddonBlock = serde_json::from_reader(stripper)
                    .map_err(|e| JsonError(Arc::new(e), block_path.to_path_buf()))?;

                blocks.insert(block_path.to_path_buf(), block);
            }
        }

        // Items
        let items_path = path.join("items");
        let mut items = HashMap::new();

        // If dir exists, read all items
        if items_path.is_dir() {
            'items_walk: for entry in WalkDir::new(&items_path)
                .into_iter()
                .filter(|v| {
                    if let Ok(v) = v {
                        v.file_type().is_file()
                    } else {
                        false
                    }
                })
                .flatten()
            {
                let item_path = entry.path();

                let item = fs::read_to_string(item_path)
                    .map_err(|e| IOError(Arc::new(e), item_path.to_path_buf()))?;

                // If the file is empty skip it
                if item.trim().is_empty() {
                    continue 'items_walk;
                }

                // Strip all c-style comments
                let stripper = CommentSettings::c_style().strip_comments(item.as_bytes());

                let item: AddonItem = serde_json::from_reader(stripper)
                    .map_err(|e| JsonError(Arc::new(e), item_path.to_path_buf()))?;

                items.insert(item_path.to_path_buf(), item);
            }
        }

        // Functions
        let functions_path = path.join("functions");
        let mut functions = HashMap::new();

        // If dir exists read all functions
        if functions_path.is_dir() {
            for entry in WalkDir::new(&functions_path)
                .into_iter()
                .filter(|v| {
                    if let Ok(v) = v {
                        v.file_type().is_file()
                    } else {
                        false
                    }
                })
                .flatten()
            {
                let function_path = entry.path();

                let function = fs::read_to_string(function_path)
                    .map_err(|e| IOError(Arc::new(e), function_path.to_path_buf()))?;

                functions.insert(function_path.to_path_buf(), function);
            }
        }

        // Scripts
        let scripts_path = path.join("scripts");
        let mut scripts = HashMap::new();

        // If dir exists read all scripts
        if scripts_path.is_dir() {
            for entry in WalkDir::new(&scripts_path)
                .into_iter()
                .filter(|v| {
                    if let Ok(v) = v {
                        v.file_type().is_file()
                    } else {
                        false
                    }
                })
                .flatten()
            {
                let script_path = entry.path();

                let script = fs::read_to_string(script_path)
                    .map_err(|e| IOError(Arc::new(e), script_path.to_path_buf()))?;

                scripts.insert(script_path.to_path_buf(), script);
            }
        }

        Ok(Self {
            manifest,
            languages,
            blocks,
            items,
            functions,
            scripts,
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
