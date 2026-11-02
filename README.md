# bedrock-rs

**_Universal Toolkit for Minecraft Bedrock Edition in Rust_**

**bedrock-rs** is a comprehensive and user-friendly library written in Rust, designed to provide a universal solution for working with Minecraft Bedrock Edition. This project offers:  

- **Standards:** Adhering to best practices and conventions.  
- **Common Implementations:** Reusable components for various Minecraft Bedrock needs.  
- **Easy-to-Use API:** Streamlined interfaces to make development efficient and enjoyable.  

Join our growing community on Discord to learn more about the project’s future, seek support, or collaborate with others:
**[Join our Discord](https://discord.com/invite/VCVcrvt3JC)**  

---

## Crates

To maintain modularity and scalability, **bedrock-rs** is divided into multiple crates. Each crate focuses on a specific functionality, making it easier to use and manage. All crates are accessible through the primary `bedrock` crate. Additionally, the library offers a variety of optional features you can enable to suit your needs.

### Crate Breakdown:

- [`bedrock::core`](crates/shared) (`bedrock_shared`)  
  - Shared data types (vectors, actor runtime/unique IDs, world enums) used across other crates.  

- [`bedrock_protocol_core`](crates/protocol_core) and [`bedrock_macros`](crates/macros)  
  - The `ProtoCodec` traits, packet header, error types, and codecs for primitives.  
  - `#[derive(ProtoCodec)]` and `#[packet(id = ...)]`, which every packet, type, and enum is built from.  
  - Re-exported through `bedrock::protocol`.  

- [`bedrock::protocol`](crates/protocol)  
  - Complete implementation of the Minecraft Bedrock protocol.  
  - Support for both server-side and client-side operations.  
  - Multi-protocol compatibility: one feature per protocol version (`protocol-v662` through `protocol-v2225`), each version described as a diff over the previous one and expanded by `cargo xtask`.  
  - `protocol-v2168hotfix4` (`V2168Hotfix4`) is 1.26.44, which reports protocol 2168 like 1.26.40 but writes SetScore remove entries differently; a server picks it after login by the client's game version (`Connection::into_ver`).  

- [`bedrock::network`](crates/network)  
  - RakNet-based transport layer for accepting, tracking, and communicating with client connections.  
  - Packet batching/codec support, including Zlib/Snappy compression and AES encryption.  
  - Server MOTD (server list ping) construction and connection listener utilities.  
  - The `network` feature is sans-io (codec, compression, encryption, and the version-agnostic login helpers in `network::login`: `LoginOptions`, `NetworkSettings`, `check_protocol`, `begin_encryption`, `ClientIdentity`) and pulls neither tokio nor RakNet; enable `network-tokio` for the `network::tokio` module: `Connection`, `Listener::builder` and `Client`. A server or client receives, matches its own version's login packets, calls the helpers and sends the reply; `bedrock/examples/server.rs` and `client.rs` show the whole exchange.  
  - `network-nethernet` adds the sans-io NetherNet helpers (`IdentityStore`, `server_data`); `network-nethernet-tokio` adds NetherNet to `Listener::builder`, which takes any mix of `raknet`, `nethernet_http` and `nethernet_lan`.  

- [`bedrock::auth`](crates/auth)  
  - Validation of Xbox Live login identity chains (JWTs) against Microsoft’s OIDC discovery service.  
  - Online and offline (self-signed) logins; guest (split-screen) logins are rejected unless the caller opts in (`AuthData::validate_allowing_guests`, `LoginOptions::allow_guests`), and are then verified like online logins.  
  - No I/O of its own: bring any HTTP client, or enable `auth-ureq` (blocking, no tokio) or `auth-reqwest` (async).  

- [`bedrock::form`](crates/form)  
  - Implementation of the JSON form format used by Minecraft Bedrock Edition.  

- [`bedrock::addon`](crates/addon/addon) (`bedrock_addon`) and [`bedrock::addon_std`](crates/addon/std) (`bedrock_addon_std`)
  - Behaviour and resource packs as typed Rust data: manifests, containers, pack stacks, every content family the game loads, diagnostics shaped like the game's content log, and write-back.
  - `bedrock_addon` is sans-IO; `bedrock_addon_std` is the driver that answers its loaders with the file system and, optionally, a thread pool. Enabled through the `addon` and `addon-std` features.
  - Built from `bedrock_addon_core` (shared vocabulary), `bedrock_addon_derive` (the component derive), `bedrock_addon_support` (helpers shared by families) and one crate per content family under `crates/addon/families/`.

- [`bedrock::level`](crates/level)  
  - Data structures for managing Minecraft Bedrock levels.  
  - Implementation of Bedrock's level format on top of a pure-Rust LevelDB implementation
    (`rusty-leveldb`) that understands Bedrock's on-disk block compression ids, so no native
    toolchain is required to build it.

---

## Features

- **Modular Architecture:** Enable only the features you need for your project.  
- **Multi-Protocol Support:** Work with different protocol versions effortlessly.  
- **Cross-Platform Compatibility:** Designed to work seamlessly across platforms.  
- **Lightweight and Efficient:** Built with Rust’s performance and safety features.  

---

## Getting Started

To use **bedrock-rs** in your Rust project, add the following to your `Cargo.toml`:  

```toml
[dependencies]
bedrock = { git = "https://github.com/bedrock-crustaceans/bedrock-rs.git", features = ["full"] }
```

Refer to the individual crate documentation for details on specific modules and features.

We also plan to release bedrock-rs on [crates.io](https://crates.io) in the future.

---

## Contributors  

A huge thank you to all the amazing individuals who have contributed to **bedrock-rs**! Your time, effort, and expertise are what make this project possible.  

[![Contributors](https://contrib.rocks/image?repo=bedrock-crustaceans/bedrock-rs)](https://github.com/bedrock-crustaceans/bedrock-rs/graphs/contributors)

Whether it’s fixing bugs, implementing features, or providing feedback, every contribution helps shape the future of this library. We appreciate each and every one of you!  

Want to join this incredible group? Check out our Contributing Guide and make your mark on the project.  

---

## Contributing

We welcome contributions of all kinds, including bug fixes, new features, docs updates, and improvements across crates.  
Please read the full contribution guide here: **[CONTRIBUTING.md](CONTRIBUTING.md)**  
It covers setup, where things live, the test-first workflow, the checks CI runs, and how protocol versions are added.
[CLAUDE.md](CLAUDE.md) is the step-by-step working method for changes to this repo.  

For guidance or collaboration, connect with the community on Discord.  

If you find **bedrock-rs** helpful, don’t forget to give the repository a ⭐ on GitHub.  

---

## License

**bedrock-rs** is open-source software licensed under the [Apache-2.0 License](LICENSE).  
