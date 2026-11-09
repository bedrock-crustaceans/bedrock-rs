//! The caustics texture of the deferred water settings.
//!
//! `minecraft:water_settings/caustics/texture` is a texture path: the client loads the image
//! at it (or the texture set beside it) from the highest pack that has one, and needs its
//! frames stacked vertically, each as tall as the image is wide.
// The check and its texts are the deferred-settings family's (`check_links`): this file
// answers what a path names.

use std::cell::RefCell;

use bedrock_addon_core::pack_identity::PackSide;

use crate::families::deferred_settings::{Targets, Texture, check_links};
use crate::families::texture_file::ImageInfo;
use crate::id::{BlockId, TexturePath};
use crate::pack::link::prelude::*;

/// An image nothing is said of: one frame.
const ONE_FRAME: Texture = Texture::Image {
    width: 1,
    height: 1,
};

/// What the texture paths of the deferred settings name on the resource stack of a pass.
#[derive(Debug)]
struct Stack<'c, 'a, 'o> {
    cx: &'c LinkCx<'a>,
    /// Where a file whose bytes the pack does not hold is told.
    out: RefCell<&'o mut Findings>,
}

impl Targets for Stack<'_, '_, '_> {
    /// The local lights are another rule's: every block is one here.
    fn block(&self, _: &BlockId) -> bool {
        true
    }

    /// The image the client loads for `path`, by its header: missing when no pack has a
    /// texture there, undecodable when the header is not of the file's format. A texture
    /// set that did not load (told at load), a uniform colour, a file without an image
    /// extension and a file whose bytes are not resident are not looked into.
    fn texture(&self, path: &str) -> Texture {
        let Some(file) = self.cx.texture_file(&TexturePath::parse(path)) else {
            return Texture::Missing;
        };
        let Some(image) = file.image else {
            return ONE_FRAME;
        };
        let Some(format) = image.format else {
            return ONE_FRAME;
        };
        let mut out = self.out.borrow_mut();
        let Some(bytes) = self
            .cx
            .bytes(PackSide::Resource, file.pack, image.path, &mut out)
        else {
            return ONE_FRAME;
        };
        match ImageInfo::read(format, &bytes) {
            Some(info) => Texture::Image {
                width: info.width,
                height: info.height,
            },
            None => Texture::Undecodable,
        }
    }
}

/// The `caustics.texture` of every effective water
/// settings document must be a texture of the resource stack (`TextureNotFound`, a kept error
/// with the client's text: the client uses its own caustics) whose image decodes and is a
/// whole number of square frames tall (`InvalidTexture`, a kept error with the client's text).
/// The image is judged from the bytes the pack holds; one that is not resident is skipped.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    let settings = cx.resource().deferred_settings();
    if settings.water.is_empty() {
        return;
    }
    let found = {
        let stack = Stack {
            cx,
            out: RefCell::new(&mut *out),
        };
        check_links(settings, &stack)
    };
    out.extend(found);
}

#[cfg(test)]
mod tests {
    use bedrock_addon_core::load::LoadOptions;

    use crate::pack::StackSet;
    use crate::pack::link::tests::support::{pack, set_of};
    use crate::pack::link::{LinkOptions, link_with_rules};

    /// `text` for the life of the test process.
    fn leak(text: String) -> &'static str {
        Box::leak(text.into_boxed_str())
    }

    /// The header of an uncompressed 32-bit TGA of 16 by `height` pixels: bytes that are
    /// text, as the fixture packs hold their files.
    fn tga(height: u8) -> &'static str {
        let header = [0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16, 0, height, 0, 32, 0];
        leak(String::from_utf8(header.to_vec()).expect("bytes below 0x80"))
    }

    /// A world of one resource pack with `n` water settings, each with a caustics texture
    /// of its own; every second image is no whole number of frames.
    fn world(n: usize) -> StackSet {
        let mut files: Vec<(&'static str, Option<&str>)> = Vec::new();
        for i in 0..n {
            files.push((
                leak(format!("water/w{i}.json")),
                Some(leak(format!(
                    r#"{{"format_version": "1.26.20", "minecraft:water_settings": {{"description": {{"identifier": "demo:w{i}"}},
                        "caustics": {{"texture": "textures/c{i}"}}}}}}"#
                ))),
            ));
            files.push((
                leak(format!("textures/c{i}.tga")),
                Some(tga(if i % 2 == 0 { 32 } else { 40 })),
            ));
        }
        let set = set_of(
            Vec::new(),
            vec![pack(2, "resources", &files)],
            &LoadOptions::default(),
        );
        assert_eq!(set.resource.deferred_settings().water.len(), n);
        let _ = (set.resource.texture_files(), set.resource.texture_sets());
        set
    }

    /// The cost of the caustics rule: `n` water settings, each resolving its texture path,
    /// reading the image's bytes and its header, against four times as many.
    #[test]
    fn the_caustics_textures_of_n_water_settings_cost_n() {
        crate::test_cost::assert_linear_at(
            "link: the caustics textures of n water settings",
            2_500,
            world,
            |set: &StackSet| {
                let n = set.resource.deferred_settings().water.len();
                let found = link_with_rules(set, LinkOptions::default(), &[super::run]);
                assert_eq!(found.len(), n / 2);
                found.len()
            },
        );
    }
}
