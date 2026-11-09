//! The image of every image layer of every loaded texture set is held to
//! its header: one that reads, with a channel count the layer takes. The load reads no image,
//! so the rule reads the bytes the stack holds; an image that is not resident is told once by
//! the pass and held to nothing.

use bedrock_addon_core::pack_identity::PackSide;

use crate::pack::link::prelude::*;

/// The image of every image layer of every loaded texture set of the resource stack: an image
/// whose header does not read, and one whose channel count the layer does not take, is an
/// error at the set's layer (the client shows the missing texture in place of the whole set).
/// The channel count is the decoded image's as the header reader gives it, for every format.
// The load runs neither check; a pass that has the bytes does.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    for layer in cx.resource().layer_images() {
        let Some(bytes) = cx.bytes(PackSide::Resource, layer.pack, layer.image.path, out) else {
            continue;
        };
        if let Some(finding) = layer.check(&bytes) {
            out.push(finding);
        }
    }
}
