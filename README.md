# DeerSoftware // Hydrogen

An Open Source Discord bot designed to play music on voice calls with efficiency, speed, and performance.

[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/H2H4NKWWN)

## Contributing

If you want to contribute to this project, you can create your fork or branch, make the changes you want and then create a Merge Request for the development branch, remembering that this project follows the following standards and conventions:

- <https://www.conventionalcommits.org/en/v1.0.0/>
- <https://keepachangelog.com/en/1.0.0/>
- <https://semver.org/lang/pt-BR/>

*Remembering that so far this project does not follow any standard or convention regarding naming branches, naming and describing Merge Requests or naming and describing Issues, just be coherent and concise in your naming and description, if you want to suggest a standard or convention for this project, use the Issues tab.

If you want to contribute with the translations, you can do this through our [Crowdin](https://crowdin.com/project/hydrogen), but if you prefer to edit the files directly, they are named by the language code used and provided by Discord (link to this list below), inside the folder `assets/langs` and formatted in JSON with syntax similar to that used by Crowdin. Copy and modify the original English file (`en-US.json`), remembering that the words involved in ${} are variables inserted at runtime, so they should not be translated, you can use the `pt-BR.json` file as an example.

- <https://discord.com/developers/docs/reference#locales>

## Using Docker

If you don't want to compile from source, you can use the official Docker image, this is the only alternative officially supported by DeerSoftware, you can access it at: <https://hub.docker.com/r/nashiradeer/hydrogen>

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

Hydrogen is a project by Nashira Deer, inspired by Hellion, and licensed under the GNU General Public License v3.

Thanks to the following projects for making the development of Hydrogen possible:

- [async-trait](https://github.com/dtolnay/async-trait) by [dtolnay](https://github.com/dtolnay), licensed with [Apache License 2.0](https://github.com/dtolnay/async-trait/blob/master/LICENSE-APACHE) and [MIT License](https://github.com/dtolnay/async-trait/blob/master/LICENSE-MIT)
- [async-tungstenite](https://github.com/sdroege/async-tungstenite) by [sdroege](https://github.com/sdroege), licensed with [MIT License](https://github.com/sdroege/async-tungstenite/blob/main/LICENSE)
- [base64](https://github.com/marshallpierce/rust-base64) by [marshallpierce](https://github.com/marshallpierce), licensed with [Apache License 2.0](https://github.com/marshallpierce/rust-base64/blob/master/LICENSE-APACHE) and [MIT License](https://github.com/marshallpierce/rust-base64/blob/master/LICENSE-MIT)
- [futures](https://github.com/rust-lang/futures-rs) by [rust-lang](https://github.com/rust-lang), licensed with [Apache License 2.0](https://github.com/rust-lang/futures-rs/blob/master/LICENSE-APACHE) and [MIT License](https://github.com/rust-lang/futures-rs/blob/master/LICENSE-MIT)
- [http](https://github.com/hyperium/http) by [hyperium](https://github.com/hyperium), licensed with [Apache License 2.0](https://github.com/hyperium/http/blob/master/LICENSE-APACHE) and [MIT License](https://github.com/hyperium/http/blob/master/LICENSE-MIT)
- [Lavalink](https://github.com/lavalink-devs/Lavalink/) by [lavalink-devs](https://github.com/lavalink-devs), licensed with [MIT License](https://github.com/lavalink-devs/Lavalink/blob/master/LICENSE)
- [Rand](https://github.com/rust-random/rand) by [rust-random](https://github.com/rust-random), licensed with [Apache License 2.0](https://github.com/rust-random/rand/blob/master/LICENSE-APACHE) and [MIT License](https://github.com/rust-random/rand/blob/master/LICENSE-MIT)
- [Reqwest](https://github.com/seanmonstar/reqwest) by [seanmonstar](https://github.com/seanmonstar), licensed with [Apache License 2.0](https://github.com/seanmonstar/reqwest/blob/master/LICENSE-APACHE) and [MIT License](https://github.com/seanmonstar/reqwest/blob/master/LICENSE-MIT)
- [Rust](https://github.com/rust-lang/rust) by [rust-lang](https://github.com/rust-lang), licensed with [Apache License 2.0](https://github.com/rust-lang/rust/blob/master/LICENSE-APACHE) and [MIT License](https://github.com/rust-lang/rust/blob/master/LICENSE-MIT)
- [Rustls](https://github.com/rustls/rustls) by [rustls](https://github.com/rustls), licensed with [Apache License 2.0](https://github.com/rustls/rustls/blob/main/LICENSE-APACHE), [ISC License](https://github.com/rustls/rustls/blob/main/LICENSE-ISC), and [MIT License](https://github.com/rustls/rustls/blob/main/LICENSE-MIT)
- [Serde](https://github.com/serde-rs/serde) by [serde-rs](https://github.com/serde-rs), licensed with [Apache License 2.0](https://github.com/serde-rs/serde/blob/master/LICENSE-APACHE) and [MIT License](https://github.com/serde-rs/serde/blob/master/LICENSE-MIT)
- [Serde JSON](https://github.com/serde-rs/json) by [serde-rs](https://github.com/serde-rs), licensed with [Apache License 2.0](https://github.com/serde-rs/json/blob/master/LICENSE-APACHE) and [MIT License](https://github.com/serde-rs/json/blob/master/LICENSE-MIT)
- [Serenity](https://github.com/serenity-rs/serenity) by [serenity-rs](https://github.com/serenity-rs), licensed with [ISC License](https://github.com/serenity-rs/serenity/blob/current/LICENSE.md)
- [Songbird](https://github.com/serenity-rs/songbird) by [serenity-rs](https://github.com/serenity-rs), licensed with [ISC License](https://github.com/serenity-rs/songbird/blob/current/LICENSE.md)
- [Tokio](https://github.com/tokio-rs/tokio) by [tokio-rs](https://github.com/tokio-rs), licensed with [MIT License](https://github.com/tokio-rs/tokio/blob/master/LICENSE)
- [tokio-rustls](https://github.com/rustls/tokio-rustls) by [rustls](https://github.com/rustls), licensed with [Apache License 2.0](https://github.com/rustls/tokio-rustls/blob/main/LICENSE-APACHE) and [MIT License](https://github.com/rustls/tokio-rustls/blob/main/LICENSE-MIT)
- [Tracing](https://github.com/tokio-rs/tracing) by [tokio-rs](https://github.com/tokio-rs), licensed with [MIT License](https://github.com/tokio-rs/tracing/blob/master/LICENSE)
- [tracing-subscriber](https://github.com/tokio-rs/tracing) by [tokio-rs](https://github.com/tokio-rs), licensed with [MIT License](https://github.com/tokio-rs/tracing/blob/master/LICENSE)
- [Tungstenite](https://github.com/snapview/tungstenite-rs) by [snapview](https://github.com/snapview), licensed with [Apache License 2.0](https://github.com/snapview/tungstenite-rs/blob/master/LICENSE-APACHE) and [MIT License](https://github.com/snapview/tungstenite-rs/blob/master/LICENSE-MIT)
