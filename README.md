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

## Installation

```toml
[dependencies]
bedrock = { git = "https://github.com/bedrock-crustaceans/bedrock-rs.git", features = ["full"] }
```

Everything is reached through the `bedrock` crate, enable only the features you need.

## Features

- **`protocol`** - Every packet of the Bedrock protocol.
- **`network`** - Batching, compression and encryption, plus login helpers.
- **`auth`** - Xbox Live login verification.
- **`level`** - Reading and writing worlds.
- **`form`** - The JSON forms the client renders.
- **`addon`** - Behaviour and resource packs as typed data
- **`full`** - All the above.

## Usage

Usage examples can be found in [`bedrock/examples`](bedrock/examples).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md), and come say hi on [Discord][discord_url].

[![Contributors](https://contrib.rocks/image?repo=bedrock-crustaceans/bedrock-rs)](https://github.com/bedrock-crustaceans/bedrock-rs/graphs/contributors)

## License

bedrock-rs is licensed under the [Apache License, Version 2.0](LICENSE).

Unless you explicitly state otherwise, any contribution you intentionally submit for inclusion
in this project, as defined in the Apache-2.0 license, shall be licensed as above, without any
additional terms or conditions.
