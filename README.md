<br />
<div align="center">

# bedrock-rs

A toolkit for Minecraft Bedrock Edition written in Rust.

[![rust][rust_badge_url]][rust_url]
[![discord][discord_badge_url]][discord_url]
[![license][license_badge_url]][license_url]

</div>

<!-- BADGES -->
[rust_badge_url]: https://img.shields.io/badge/rust-2024-%23D34516?style=flat-square&logo=rust&logoColor=%23D34516&labelColor=white
[rust_url]: https://rust-lang.org/

[discord_badge_url]: https://img.shields.io/badge/discord-join-%235865F2?style=flat-square&logo=discord&logoColor=white
[discord_url]: https://discord.com/invite/VCVcrvt3JC

[license_badge_url]: https://img.shields.io/github/license/bedrock-crustaceans/bedrock-rs?style=flat-square
[license_url]: LICENSE
<!-- BADGES -->

### Installation

```toml
[dependencies]
bedrock = { git = "https://github.com/bedrock-crustaceans/bedrock-rs.git", features = ["full"] }
```

Everything is reached through the `bedrock` crate; enable only the features you need.

### Features

- **`protocol`** - every packet of the Bedrock protocol, one `protocol-v<N>` feature per version.
- **`network`** - batching, compression and encryption, plus login helpers. Add **`network-tokio`** for an async listener and client over RakNet, and **`network-nethernet-tokio`** for NetherNet.
- **`auth`** - Xbox Live login verification. Bring your own HTTP client, or enable **`auth-ureq`** or **`auth-reqwest`**.
- **`level`** - reading and writing worlds.
- **`form`** - the JSON forms the client renders.
- **`addon`** / **`addon-std`** - behaviour and resource packs as typed data, loaded from disk with **`addon-std`**.
- **`full`** - `protocol`, `network-nethernet-tokio`, `auth` and `level`.

### Usage

- server:

    ```rust
    let mut listener = Listener::builder()
        .raknet("0.0.0.0:19132".parse()?)
        .name("Bedrock in Rust")
        .protocol::<V2225>()
        .max_players(100)
        .bind()
        .await?;

    loop {
        let conn = listener.accept().await?.into_ver::<V2225>();
        tokio::spawn(async move {
            // log the player in, then send and receive packets
        });
    }
    ```

- add-ons:

    ```rust
    let mut driver = Driver::new(Options::default())?;
    let repository = driver.repository([
        DirSource::packs("behavior_packs", PackOrigin::World),
        DirSource::packs("resource_packs", PackOrigin::World),
    ])?;
    let world = read_world_packs("worlds/Bedrock level")?;
    let stacks = driver.load_stacks(
        repository.stacks(&world, LoadOptions::default()),
        StackOptions::default(),
    )?;

    for finding in stacks.load_all().at_least(Severity::Warning) {
        eprintln!("{finding}");
    }
    ```

Full login flows are in [`bedrock/examples`](bedrock/examples):

```sh
cargo run --example server --features network-tokio,protocol-v2225,protocol-unknown
cargo run --example client --features network-tokio,protocol-v2225,protocol-unknown
```

### Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md), and come say hi on [Discord][discord_url].

[![Contributors](https://contrib.rocks/image?repo=bedrock-crustaceans/bedrock-rs)](https://github.com/bedrock-crustaceans/bedrock-rs/graphs/contributors)
