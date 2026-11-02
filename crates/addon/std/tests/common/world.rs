//! The world lists the smoke tests load a vanilla install with. The base-game layer is the
//! composer's, in vanilla's order. The packs outside it that a list names (a disabled
//! experimental pack, chemistry, editor and education packs) sit ABOVE that layer, as a
//! world's packs do: they are read so that every file is checked, and they can shadow the
//! layer's definitions, where in the game a pack whose experiment is off is not loaded at
//! all and so never wins.

#![allow(dead_code)]

use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{PackIdVersion, PackRepository, PackSelection, PackType, WorldPacks};

/// The world's lists of every pack of `repository`, a vanilla install registered with
/// `PackOrigin::Package`.
///
/// The base-game layer is not listed: the stack composer adds it below the world's packs, in
/// the game's order for a world with `options` (the
/// experimental packs whose experiment is on, the two script libraries, the `vanilla` slices
/// newest first, `vanilla`), and without hiding a base-game pack whose resource pack the
/// repository lacks, as a world-listed pack would be. The behaviour list holds the behaviour
/// packs that layer does not (an experimental pack whose experiment is off, `chemistry`, the
/// education and editor packs), in the order the repository registered them, so that every
/// file of the install is read. Which packs the layer holds is read from the stacks of a world
/// that lists nothing, so this helper holds no table of pack identities.
///
/// The resource list is the resource packs those behaviour packs depend on, in the order the
/// composer adds them, then every other resource pack in the order of the client's base-game
/// resource stack ([`resource_layer_rank`]): the packs on no base-game stack in the
/// repository's order, then the experiment packs, `vanilla_music`, the platform packs,
/// `vanilla_trial`, the resource slices newest first, `oreui`, `vanilla`, `vanilla_base`,
/// `persona`.
///
/// Listing every pack in the repository's order (the folders' names) would put `vanilla` above
/// every `vanilla_1.x` slice, which is not the order the game builds.
pub fn base_game_world(repository: &PackRepository, options: &LoadOptions) -> WorldPacks {
    let nothing = WorldPacks {
        behavior: Vec::new(),
        resource: Vec::new(),
    };
    let layer: Vec<PackIdVersion> = repository
        .stacks(&nothing, options.clone())
        .behavior
        .packs()
        .map(|pack| pack.id().clone())
        .collect();
    let mut behavior = Vec::new();
    // In an education world the composer appends the chemistry behaviour packs itself (a slice
    // it hides is in no stack's `packs()`), and a resource half is on a stack only as a
    // dependency it pulls in: neither is listed.
    let composed = |pack: &bedrock_addon::pack::Pack| options.education && is_chemistry(pack);
    for pack in repository.packs() {
        if pack.pack_type() == PackType::Behavior
            && !layer.contains(pack.id())
            && !behavior.contains(pack.id())
            && !composed(pack)
        {
            behavior.push(pack.id().clone());
        }
    }
    let behavior: Vec<PackSelection> = behavior.iter().map(selection).collect();
    // The resource packs the behaviour stack pulls in, top first.
    let with_dependencies = repository.stacks(
        &WorldPacks {
            behavior: behavior.clone(),
            resource: Vec::new(),
        },
        options.clone(),
    );
    let mut resource: Vec<PackIdVersion> = with_dependencies
        .resource
        .packs()
        .map(|pack| pack.id().clone())
        .collect();
    append_resources(&mut resource, repository);
    if options.education {
        let pulled: Vec<PackIdVersion> = with_dependencies
            .resource
            .packs()
            .map(|pack| pack.id().clone())
            .collect();
        resource.retain(|id| pulled.contains(id) || !repository.get(id).is_some_and(is_chemistry));
    }
    WorldPacks {
        behavior,
        resource: resource.iter().map(selection).collect(),
    }
}

/// The world a smoke test loads that checks the stack vanilla builds, not every pack:
/// `world_behavior` as the world's behaviour list when it names packs (a test pack of
/// the sample packs or of a client install), above the base-game layer. When it is the empty list
/// `[]`, the stack is the base-game layer alone, which the composer builds in the game's order (the
/// experimental packs whose experiment is on, the script libraries, the slices newest first,
/// `vanilla`), so an experimental pack whose experiment is off is not loaded, as in the game.
/// [`base_game_world`] is the list for the smokes that read every pack.
pub fn world_list(
    _repository: &PackRepository,
    world_behavior: &[u8],
    _options: &LoadOptions,
) -> WorldPacks {
    WorldPacks::parse(Some(world_behavior), Some(b"[]")).expect("a world list")
}

/// Appends the packs of `repository` of `pack_type` that `list` lacks, in the repository's
/// order.
fn append_rest(list: &mut Vec<PackIdVersion>, repository: &PackRepository, pack_type: PackType) {
    for pack in repository.packs() {
        if pack.pack_type() == pack_type && !list.contains(pack.id()) {
            list.push(pack.id().clone());
        }
    }
}

/// Whether the manifest of `pack` declares the `chemistry` capability (every chemistry pack of
/// a vanilla install does; the composer's reading of the chemistry category).
fn is_chemistry(pack: &bedrock_addon::pack::Pack) -> bool {
    pack.manifest()
        .capabilities
        .contains(&bedrock_addon::manifest::Capability::Chemistry)
}

/// The world-list entry of a pack, with no subpack chosen.
fn selection(id: &PackIdVersion) -> PackSelection {
    PackSelection {
        pack_id: id.uuid,
        version: id.version.clone(),
        subpack: None,
    }
}

/// The `vanilla` resource pack and its slices (one identity, the slice's version).
const VANILLA_RESOURCE: uuid::Uuid = uuid::Uuid::from_u128(0x0575c61f_a5da_4b7f_9961_ffda2908861e);
/// The `oreui` resource pack.
const OREUI: uuid::Uuid = uuid::Uuid::from_u128(0xfdcdb085_9f06_464f_bf0e_25c0b37ff4a5);
/// The `vanilla_base` resource pack.
const VANILLA_BASE: uuid::Uuid = uuid::Uuid::from_u128(0x9ae3d2cc_7811_404c_b578_cb27689c8ac5);
/// The `persona` resource pack.
const PERSONA: uuid::Uuid = uuid::Uuid::from_u128(0x2099de18_429a_465a_a49b_fc4710a17bb3);
/// The `vanilla_music` resource pack.
const VANILLA_MUSIC: uuid::Uuid = uuid::Uuid::from_u128(0x6989c411_4355_4756_9163_51c1df5ef677);
/// The `vanilla_trial` resource pack.
const VANILLA_TRIAL: uuid::Uuid = uuid::Uuid::from_u128(0xbd264cd8_8969_4e5e_865a_21096f5ad838);
/// The platform resource packs, in the order the client adds them: the running platform's
/// own (the PC pack of the client installs), then Xbox, Google, Amazon, iOS.
const PLATFORMS: [uuid::Uuid; 5] = [
    uuid::Uuid::from_u128(0x133464ef_3b92_44ed_8546_ec6473265119),
    uuid::Uuid::from_u128(0x1feb601d_6865_431f_a4bb_17e63d9248a0),
    uuid::Uuid::from_u128(0x47e901c5_fbf4_4c31_8a4f_71e7dbd5b34c),
    uuid::Uuid::from_u128(0x425f5616_8af2_41d6_8643_6ecaab09e830),
    uuid::Uuid::from_u128(0xe6ce9a6a_9e9c_41c3_8c51_0ffae904c560),
];

/// Where a resource pack sits in the list, top first, as `(group, place in the group)`. Group
/// 0 is a pack on no base-game stack (chemistry: it joins the world's own
/// stack, above the layer; editor and beta packs; anything unknown), in the repository's
/// order. Then the client's base-game resource stack: the
/// experiment packs in the order of vanilla's table, `vanilla_music`, the platform packs,
/// `vanilla_trial`, the resource slices, `oreui`, `vanilla`, `vanilla_base`, `persona`.
fn resource_layer_rank(id: &PackIdVersion) -> (u8, usize) {
    let in_package = id.version.major == 0 && id.version.minor == 0 && id.version.patch == 1;
    let experiment = bedrock_addon::pack::VANILLA_EXPERIMENTAL_PACKS
        .iter()
        .filter(|(_, pack_type, _)| *pack_type == PackType::Resource)
        .position(|(uuid, _, _)| *uuid == id.uuid);
    if let Some(place) = experiment {
        (1, place)
    } else if id.uuid == VANILLA_MUSIC {
        (2, 0)
    } else if let Some(place) = PLATFORMS.iter().position(|uuid| *uuid == id.uuid) {
        (3, place)
    } else if id.uuid == VANILLA_TRIAL {
        (4, 0)
    } else if id.uuid == VANILLA_RESOURCE {
        if in_package { (7, 0) } else { (5, 0) }
    } else if id.uuid == OREUI {
        (6, 0)
    } else if id.uuid == VANILLA_BASE {
        (8, 0)
    } else if id.uuid == PERSONA {
        (9, 0)
    } else {
        (0, 0)
    }
}

/// Appends the resource packs of `repository` that `list` lacks, top first: by
/// [`resource_layer_rank`], the slices newest first, every other equal rank in the
/// repository's order.
fn append_resources(list: &mut Vec<PackIdVersion>, repository: &PackRepository) {
    let mut rest: Vec<PackIdVersion> = Vec::new();
    append_rest(&mut rest, repository, PackType::Resource);
    rest.retain(|id| !list.contains(id));
    // Stable: equal ranks keep the repository's order, but for the slices.
    rest.sort_by(|a, b| {
        let (rank_a, rank_b) = (resource_layer_rank(a), resource_layer_rank(b));
        rank_a.cmp(&rank_b).then_with(|| {
            if rank_a.0 == 5 {
                b.version.cmp(&a.version)
            } else {
                std::cmp::Ordering::Equal
            }
        })
    });
    list.extend(rest);
}

/// The client's base-game resource stack with no experiment on, as a world's resource list,
/// top first: the resource packs of `repository` that the client's base-game resource stack
/// holds (`vanilla_music`, the platform packs, `vanilla_trial`, the resource slices newest
/// first, `oreui`, `vanilla`, `vanilla_base`, `persona`), in that order. Left out: the packs
/// on no base-game stack (chemistry, editor, beta: group 0 of [`resource_layer_rank`]) and
/// the experiment packs (group 1: no experiment is on).
pub fn client_resource_layer(repository: &PackRepository) -> Vec<PackSelection> {
    let mut list = Vec::new();
    append_resources(&mut list, repository);
    list.retain(|id| resource_layer_rank(id).0 >= 2);
    list.iter().map(selection).collect()
}
