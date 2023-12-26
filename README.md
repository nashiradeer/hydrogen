# Nashira Deer // Hydrogen

An open-source music bot for Discord, powered by Lavalink and created in Rust.

[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/H2H4NKWWN)

## Contributing

If you want to contribute to this project, you can create your fork or branch, make the changes you want and then create a Merge Request for the development branch, remembering that this project follows the following standards and conventions:

- <https://www.conventionalcommits.org/en/v1.0.0/>
- <https://keepachangelog.com/en/1.0.0/>
- <https://semver.org/lang/pt-BR/>
- <https://dev.to/varbsan/a-simplified-convention-for-naming-branches-and-commits-in-git-il4>

*Remembering that so far this project does not follow any standard or convention regarding naming and describing Merge Requests or naming and describing Issues, just be coherent and concise in your naming and description, if you want to suggest a standard or convention for this project, use the Issues tab.

If you want to contribute with the translations, you can do this editing the files in this repository, they are named by the language code used and provided by Discord (link to this list below), inside the folder `assets/langs` and formatted in JSON. Copy and modify the original English file (`en-US.json`), remembering that the words involved in {} are variables inserted at runtime, so they should not be translated, you can use the `pt-BR.json` file as an example.

- <https://discord.com/developers/docs/reference#locales>

## Using Docker

If you don't want to compile from source, you can use the official Docker image, this is the only alternative officially supported by Nashira Deer, you can access it at: <https://hub.docker.com/r/nashiradeer/hydrogen>

## Building

This project can be built from simple commands in Cargo or Docker, there are no changes, scripts or requirements in the process of building this project, if you don't know how to do this, below is a step by step on how to assemble the debug or release versions of project.

### Docker

1. Run in a terminal: `docker build .`

This will build a new fully ready-to-use Hydrogen image, no additional work is required.

### Debug

1. Install libc headers, required to build dependencies.
2. Run in a terminal: `cargo build`

When finished, you will find the executable under `target/debug` and it will be named `hydrogen` on UNIX platforms or `hydrogen.exe` on Windows platforms.

### Release

1. Install libc headers, required to build dependencies.
2. Run in a terminal: `cargo build -r`

When finished, you will find the executable under `target/release` and it will be named `hydrogen` on UNIX platforms or `hydrogen.exe` on Windows platforms.

## Running

Before we configure Hydrogen, we need to ensure that the following dependencies are up and running:

- [Lavalink Server compatible with the Lavalink API v3](https://github.com/lavalink-devs/Lavalink/)

To configure Hydrogen you will use the following environment variables:

- LANGUAGE_PATH: Sets the path where the Hydrogen translation files can be found. (required if not Docker image)
- LAVALINK: Set the list of Lavalink nodes that can be used, read more below. (required)
- DISCORD_TOKEN: Sets the token that will be used to access a Discord. (required)

### LAVALINK environment variable syntax

```plain
value           = single-node *(";" single-node)
single-node     = host "," password ["," tls]
host            = ip ":" port
tls             = "true" / "enabled" / "on"
```

*Due to a bug, the TLS parameter needs to be written in lower case.

## Credits

Hydrogen is a project by Nashira Deer, based on another Nashira Deer's Discord Bot, Hellion, and licensed under the [GNU General Public License v3](https://github.com/nashiradeer/hydrogen/blob/main/LICENSE.txt).

### Third-party projects

- [async-trait](https://github.com/dtolnay/async-trait), licensed with [Apache License 2.0](https://github.com/dtolnay/async-trait/blob/master/LICENSE-APACHE) and [MIT License](https://github.com/dtolnay/async-trait/blob/master/LICENSE-MIT)
- [async-tungstenite](https://github.com/sdroege/async-tungstenite), licensed with [MIT License](https://github.com/sdroege/async-tungstenite/blob/main/LICENSE)
- [base64](https://github.com/marshallpierce/rust-base64), licensed with [Apache License 2.0](https://github.com/marshallpierce/rust-base64/blob/master/LICENSE-APACHE) and [MIT License](https://github.com/marshallpierce/rust-base64/blob/master/LICENSE-MIT)
- [futures](https://github.com/rust-lang/futures-rs), licensed with [Apache License 2.0](https://github.com/rust-lang/futures-rs/blob/master/LICENSE-APACHE) and [MIT License](https://github.com/rust-lang/futures-rs/blob/master/LICENSE-MIT)
- [http](https://github.com/hyperium/http), licensed with [Apache License 2.0](https://github.com/hyperium/http/blob/master/LICENSE-APACHE) and [MIT License](https://github.com/hyperium/http/blob/master/LICENSE-MIT)
- [Lavalink](https://github.com/lavalink-devs/Lavalink/), licensed with [MIT License](https://github.com/lavalink-devs/Lavalink/blob/master/LICENSE)
- [Rand](https://github.com/rust-random/rand), licensed with [Apache License 2.0](https://github.com/rust-random/rand/blob/master/LICENSE-APACHE) and [MIT License](https://github.com/rust-random/rand/blob/master/LICENSE-MIT)
- [Reqwest](https://github.com/seanmonstar/reqwest), licensed with [Apache License 2.0](https://github.com/seanmonstar/reqwest/blob/master/LICENSE-APACHE) and [MIT License](https://github.com/seanmonstar/reqwest/blob/master/LICENSE-MIT)
- [Rust](https://github.com/rust-lang/rust), licensed with [Apache License 2.0](https://github.com/rust-lang/rust/blob/master/LICENSE-APACHE) and [MIT License](https://github.com/rust-lang/rust/blob/master/LICENSE-MIT)
- [Rustls](https://github.com/rustls/rustls), licensed with [Apache License 2.0](https://github.com/rustls/rustls/blob/main/LICENSE-APACHE), [ISC License](https://github.com/rustls/rustls/blob/main/LICENSE-ISC), and [MIT License](https://github.com/rustls/rustls/blob/main/LICENSE-MIT)
- [Serde](https://github.com/serde-rs/serde), licensed with [Apache License 2.0](https://github.com/serde-rs/serde/blob/master/LICENSE-APACHE) and [MIT License](https://github.com/serde-rs/serde/blob/master/LICENSE-MIT)
- [Serde JSON](https://github.com/serde-rs/json), licensed with [Apache License 2.0](https://github.com/serde-rs/json/blob/master/LICENSE-APACHE) and [MIT License](https://github.com/serde-rs/json/blob/master/LICENSE-MIT)
- [Serenity](https://github.com/serenity-rs/serenity), licensed with [ISC License](https://github.com/serenity-rs/serenity/blob/current/LICENSE.md)
- [Songbird](https://github.com/serenity-rs/songbird), licensed with [ISC License](https://github.com/serenity-rs/songbird/blob/current/LICENSE.md)
- [Tokio](https://github.com/tokio-rs/tokio), licensed with [MIT License](https://github.com/tokio-rs/tokio/blob/master/LICENSE)
- [tokio-rustls](https://github.com/rustls/tokio-rustls), licensed with [Apache License 2.0](https://github.com/rustls/tokio-rustls/blob/main/LICENSE-APACHE) and [MIT License](https://github.com/rustls/tokio-rustls/blob/main/LICENSE-MIT)
- [Tracing](https://github.com/tokio-rs/tracing), licensed with [MIT License](https://github.com/tokio-rs/tracing/blob/master/LICENSE)
- [tracing-subscriber](https://github.com/tokio-rs/tracing), licensed with [MIT License](https://github.com/tokio-rs/tracing/blob/master/LICENSE)
- [Tungstenite](https://github.com/snapview/tungstenite-rs), licensed with [Apache License 2.0](https://github.com/snapview/tungstenite-rs/blob/master/LICENSE-APACHE) and [MIT License](https://github.com/snapview/tungstenite-rs/blob/master/LICENSE-MIT)
