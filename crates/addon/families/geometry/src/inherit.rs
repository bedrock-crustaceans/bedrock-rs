//! The legacy parents across the stack: each child gets the bones of the geometry it names as
//! its parent; a child whose parent no pack defines, or whose chain of parents returns to
//! itself, is not built.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::codes;
use crate::read::{MOBS, detached_origin};
use crate::validate::missing_bone_parents;
use crate::{FileContribution, Geometry, Inherited};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, Diagnostic, Diagnostics, Phase};
use bedrock_addon_core::document::{Contribution, Document, Notice, Rejected};
use bedrock_addon_core::id::GeometryId;
use bedrock_addon_core::pack_identity::PackPath;

/// What a geometry gives the geometries that inherit from it.
#[derive(Clone)]
enum Parent {
    /// Its own bones and its own inheritance.
    Built(Arc<Inherited>),
    /// It is not built: a parent up its chain is missing, or the chain returns to itself.
    NotBuilt,
}

/// Where the effective definition of an identifier is: file and contribution index.
type Slot = (usize, usize);

fn stable(files: &[FileContribution], slot: Slot) -> Option<&Document<Geometry>> {
    match &files[slot.0].contributions[slot.1] {
        Contribution::Stable(_, document) => Some(document),
        _ => None,
    }
}

/// How the walk up from a geometry ended.
enum End {
    /// At a geometry settled by an earlier walk (not on this walk).
    Settled(Parent),
    /// At a geometry without a parent (the top of the walk).
    Root,
    /// At a geometry whose parent no pack defines (the top of the walk).
    Missing,
    /// At a geometry already on the walk.
    Cycle,
}

/// Resolves every legacy parent against the definitions that win, without recursion: a child
/// gets its parent's bones; a child whose parent no pack defines, or whose chain of parents
/// returns to itself, is not built.
pub(crate) fn inherit(files: &mut [FileContribution]) {
    let mut winners: HashMap<GeometryId, Slot> = HashMap::new();
    for (f, file) in files.iter().enumerate() {
        for (c, contribution) in file.contributions.iter().enumerate() {
            if let Contribution::Stable(id, _) = contribution {
                winners.insert(id.clone(), (f, c));
            }
        }
    }
    let parent_of = |slot: Slot| -> Option<GeometryId> {
        stable(files, slot)?.value().parent().map(GeometryId::parse)
    };
    let mut children: Vec<Slot> = winners
        .values()
        .copied()
        .filter(|slot| parent_of(*slot).is_some())
        .collect();
    children.sort_unstable();
    let mut settled: HashMap<Slot, Parent> = HashMap::new();
    let mut inherited: Vec<Heir> = Vec::new();
    let mut failed: Vec<(Slot, Code)> = Vec::new();
    for start in children {
        let mut walk: Vec<Slot> = Vec::new();
        let mut on_walk: HashSet<Slot> = HashSet::new();
        let mut at = start;
        let end = loop {
            if let Some(parent) = settled.get(&at) {
                break End::Settled(parent.clone());
            }
            if !on_walk.insert(at) {
                break End::Cycle;
            }
            walk.push(at);
            let Some(parent) = parent_of(at) else {
                break End::Root;
            };
            match winners.get(&parent) {
                Some(next) => at = *next,
                None => break End::Missing,
            }
        };
        // What the geometry on top of the walk inherits; `Err` when it is not built.
        let cycle = matches!(end, End::Cycle);
        // `at` is the settled geometry the walk stopped at.
        let mut above: Result<Option<(Slot, Arc<Inherited>)>, ()> = match end {
            End::Settled(Parent::Built(node)) => Ok(Some((at, node))),
            End::Root => Ok(None),
            End::Settled(Parent::NotBuilt) | End::Missing | End::Cycle => Err(()),
        };
        for slot in walk.into_iter().rev() {
            let Some(geometry) = stable(files, slot).map(Document::value) else {
                continue;
            };
            if let Ok(parent) = above {
                if let Some((parent, node)) = &parent {
                    inherited.push(Heir {
                        slot,
                        parent: *parent,
                        node: node.clone(),
                    });
                }
                let node = Arc::new(Inherited::new(
                    geometry.description.identifier.clone(),
                    geometry.bones().to_vec(),
                    parent.map(|(_, node)| node),
                ));
                settled.insert(slot, Parent::Built(node.clone()));
                above = Ok(Some((slot, node)));
            } else {
                let code = if cycle {
                    Code::from(&core_codes::REFERENCE_CYCLE)
                } else {
                    codes::PARENT_NOT_DEFINED.with([
                        geometry.description.identifier.as_str(),
                        geometry.parent().unwrap_or_default(),
                    ])
                };
                failed.push((slot, code));
                settled.insert(slot, Parent::NotBuilt);
            }
        }
    }
    let missing = unknown_bone_parents(files, &inherited);
    apply(files, inherited, missing, failed);
}

/// A geometry that inherits: where it is, where its parent is, and what it inherits.
struct Heir {
    slot: Slot,
    parent: Slot,
    node: Arc<Inherited>,
}

/// The missing-parent check for every geometry that inherits: per geometry, the indices of its bones whose parent
/// is neither one of its own bones nor a bone of a geometry up its chain.
///
/// One walk of the inheritance forest from its roots, with the bone names of the geometries
/// between the root and the geometry in hand counted in one map: each bone of each geometry is
/// looked at once, whatever the length of a chain. No recursion.
fn unknown_bone_parents(files: &[FileContribution], heirs: &[Heir]) -> HashMap<Slot, Vec<usize>> {
    let mut children: HashMap<Slot, Vec<Slot>> = HashMap::new();
    let mut inherits: HashSet<Slot> = HashSet::new();
    for heir in heirs {
        children.entry(heir.parent).or_default().push(heir.slot);
        inherits.insert(heir.slot);
    }
    let mut roots: Vec<Slot> = children
        .keys()
        .copied()
        .filter(|slot| !inherits.contains(slot))
        .collect();
    roots.sort_unstable();
    let bones = |slot: Slot| stable(files, slot).map_or(&[][..], |d| d.value().bones());
    let mut missing: HashMap<Slot, Vec<usize>> = HashMap::new();
    // How many geometries above the one in hand have a bone of each name.
    let mut above: HashMap<&str, usize> = HashMap::new();
    // `true`: the geometry's subtree is done and its names leave the map.
    let mut pending: Vec<(Slot, bool)> = roots.into_iter().map(|slot| (slot, false)).collect();
    while let Some((slot, done)) = pending.pop() {
        let own = bones(slot);
        if done {
            for bone in own {
                if let Some(count) = above.get_mut(bone.name.as_str()) {
                    *count = count.saturating_sub(1);
                }
            }
            continue;
        }
        if inherits.contains(&slot) {
            let names: HashSet<&str> = own.iter().map(|bone| bone.name.as_str()).collect();
            let unknown: Vec<usize> = own
                .iter()
                .enumerate()
                .filter(|(_, bone)| {
                    bone.parent.as_deref().is_some_and(|parent| {
                        !names.contains(parent) && above.get(parent).is_none_or(|n| *n == 0)
                    })
                })
                .map(|(index, _)| index)
                .collect();
            if !unknown.is_empty() {
                missing.insert(slot, unknown);
            }
        }
        for bone in own {
            *above.entry(bone.name.as_str()).or_default() += 1;
        }
        pending.push((slot, true));
        if let Some(below) = children.get(&slot) {
            pending.extend(below.iter().map(|child| (*child, false)));
        }
    }
    missing
}

/// Gives each child the chain it inherits and its bone-parent findings, and turns each
/// geometry that is not built into a rejected contribution with its finding.
fn apply(
    files: &mut [FileContribution],
    inherited: Vec<Heir>,
    mut missing: HashMap<Slot, Vec<usize>>,
    failed: Vec<(Slot, Code)>,
) {
    for Heir { slot, node, .. } in inherited {
        let unknown = missing.remove(&slot).unwrap_or_default();
        replace(files, slot, |id, document| {
            let origin = document.origin().clone();
            let version = document.version().clone();
            let mut diagnostics = document.diagnostics().clone();
            let mut geometry = document.into_value();
            geometry.inherited = Some(node);
            let mut found = Diagnostics::default();
            // A file's registered geometries are its first contributions, in file order.
            missing_bone_parents(&geometry, slot.1, &unknown, &mut found);
            diagnostics.extend(found.located(&origin));
            Contribution::Stable(id, Document::new(geometry, origin, version, diagnostics))
        });
    }
    let mut not_built: HashSet<GeometryId> = HashSet::new();
    for (slot, code) in failed {
        replace(files, slot, |id, document| {
            not_built.insert(id.clone());
            let origin = document.origin().clone();
            let mut diagnostics = document.diagnostics().clone();
            diagnostics.push(
                Diagnostic::error(code)
                    .phase(Phase::Validate)
                    .in_file(&origin),
            );
            Contribution::Rejected(Some(id), Rejected::new(origin, diagnostics))
        });
    }
    shadow_for_good(files, &not_built);
}

/// The last loaded definition of an identifier is its only candidate. The earlier
/// definitions of an identifier whose last one is not built leave the walk here, each with the
/// note a replaced definition gets, so that the registry does not make one of them effective
/// in the rejected one's place.
fn shadow_for_good(files: &mut [FileContribution], not_built: &HashSet<GeometryId>) {
    if not_built.is_empty() {
        return;
    }
    for file in files {
        let mut kept = Vec::with_capacity(file.contributions.len());
        for contribution in std::mem::take(&mut file.contributions) {
            match contribution {
                Contribution::Stable(id, document) if not_built.contains(&id) => {
                    let origin = document.origin();
                    file.findings.push(
                        Diagnostic::info(Notice::shadowed().code)
                            .phase(Phase::Validate)
                            .in_file(origin),
                    );
                }
                other => kept.push(other),
            }
        }
        file.contributions = kept;
    }
}

/// Replaces the stable contribution at `slot` by what `with` makes of it.
fn replace(
    files: &mut [FileContribution],
    slot: Slot,
    with: impl FnOnce(GeometryId, Document<Geometry>) -> Contribution<Geometry>,
) {
    let contributions = &mut files[slot.0].contributions;
    let placeholder = Contribution::Rejected(
        None,
        Rejected::new(
            detached_origin(&PackPath::from_static(MOBS)),
            Diagnostics::default(),
        ),
    );
    match std::mem::replace(&mut contributions[slot.1], placeholder) {
        Contribution::Stable(id, document) => contributions[slot.1] = with(id, document),
        other => contributions[slot.1] = other,
    }
}
