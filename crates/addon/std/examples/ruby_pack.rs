//! Build a block in code, add it to a new pack with `PackWriter::add`, write the pack to a
//! directory with `bedrock_addon_std::write_dir`, then load the directory back and find the
//! same block.
//!
//! This is how a tool that generates content writes it back out. The pack is written into a
//! temporary directory; nothing else on the host is touched.
//!
//! Run with `cargo run -p bedrock_addon_std --example ruby_pack`.

use bedrock_addon::families::block::component::{
    DestructibleByMining, Geometry, LightEmission, MaterialInstance, MaterialInstanceEntry,
    MaterialInstances,
};
use bedrock_addon::families::block::{Block, Permutation, StateValues};
use bedrock_addon::id::BlockId;
use bedrock_addon::json::{StrictInt, StrictString};
use bedrock_addon::load::{LoadContext, LoadOptions, StackOptions};
use bedrock_addon::manifest::Manifest;
use bedrock_addon::pack::{PackOrigin, PackType, PackWriter, WorldPacks, WriteError};
use bedrock_addon::shared::MenuCategory;
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::MinEngineVersion;
use bedrock_addon_std::{DirSource, Driver, Options, SaveError, load_dir, write_dir};

/// The block of the example: a lamp with one state of its own and one permutation that lights
/// it while the state is set.
fn ruby_lamp() -> Block {
    let mut ruby = Block::new(BlockId::from_static("demo:ruby_lamp"));
    ruby.description.menu_category = Some(MenuCategory::construction());
    ruby.description
        .states
        .insert("demo:lit".to_owned(), StateValues::bools());
    ruby.components.insert(DestructibleByMining::seconds(1.5));
    // A block with a `minecraft:geometry` needs `minecraft:material_instances` too: one
    // without the other is an error, in the game and in `Block::validate`.
    ruby.components
        .insert(Geometry::Identifier("geometry.ruby_lamp".to_owned()));
    let mut materials = MaterialInstances::default();
    materials.0.insert(
        "*".to_owned(),
        MaterialInstanceEntry::Instance(MaterialInstance {
            texture: Some(StrictString("ruby_lamp".to_owned())),
            ..MaterialInstance::default()
        }),
    );
    ruby.components.insert(materials);
    ruby.permutations
        .push(Permutation::when("q.block_state('demo:lit')").with(LightEmission(StrictInt(15))));
    ruby
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ruby = ruby_lamp();

    // A block built in code is checked by the rules a loaded one is checked by.
    let findings = ruby.validate(&LoadContext::detached());
    assert!(!findings.has_errors(), "{findings:?}");

    let mut manifest = Manifest::new("Ruby lamps", PackType::Behavior);
    manifest.header.min_engine_version = Some(MinEngineVersion::new(1, 26, 20));
    let identity = manifest.identity();
    let mut writer = PackWriter::new(manifest);

    // `add` writes the document at its default path and says which; a second document at
    // that path is refused, never replaced.
    let path = writer.add(&ruby)?;
    assert_eq!(path, PackPath::from_static("blocks/demo/ruby_lamp.json"));
    assert!(matches!(
        writer.add(&ruby),
        Err(WriteError::DuplicatePath { .. })
    ));

    let out = tempfile::tempdir()?;
    let dir = out.path().join("ruby_lamps");
    write_dir(&writer, &dir)?;
    for (path, bytes) in writer.files() {
        println!("wrote {path} ({} bytes)", bytes.len());
    }

    // `write_dir` never merges into a folder that holds something.
    assert!(matches!(
        write_dir(&writer, &dir),
        Err(SaveError::NotEmpty { .. })
    ));

    // Load the directory back as a server would: the folder that holds the pack is a packs
    // directory, a world selects the pack by its identity, and the driver reads and parses
    // the stacks. (`load_dir` alone opens the pack, its manifest and its list of files; the
    // content files are read when a stack is loaded.)
    let pack = load_dir(&dir, &Options::default())?;
    assert_eq!(*pack.id(), identity);
    assert!(!pack.report().has_errors(), "{:?}", pack.report());
    assert_eq!(pack.paths().count(), 2);

    let mut driver = Driver::new(Options::default())?;
    let repository = driver.repository([DirSource::packs(out.path(), PackOrigin::World)])?;
    let world = WorldPacks::parse(
        Some(
            format!(
                r#"[{{ "pack_id": "{}", "version": [1, 0, 0] }}]"#,
                identity.uuid
            )
            .as_bytes(),
        ),
        None,
    )?;
    let stacks = driver.load_stacks(
        repository.stacks(&world, LoadOptions::default()),
        StackOptions::default(),
    )?;
    assert!(!stacks.load_all().has_errors(), "{}", stacks.load_all());

    let blocks = stacks.behavior.blocks();
    assert!(blocks.rejected().is_empty(), "{:#?}", blocks.rejected());
    let loaded = blocks
        .get(&BlockId::from_static("demo:ruby_lamp"))
        .expect("the block that was written");
    assert_eq!(
        loaded.origin().path,
        PackPath::from_static("blocks/demo/ruby_lamp.json")
    );
    assert_eq!(loaded.value(), &ruby, "the block that was added");
    assert_eq!(loaded.permutations.len(), 1);
    println!(
        "loaded {} back from {}",
        loaded.description.identifier,
        loaded.origin()
    );
    Ok(())
}
