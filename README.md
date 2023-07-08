# DeerSoftware // Hydrogen

An Open Source Discord bot designed to play music on voice calls with efficiency, speed and performance.

## Donating

If this project has been of use to you and you want to help but don't know how to contribute to the project, please consider donating to Hydrogen's contributors:

- Nashira Deer: <https://www.patreon.com/nashiradeer>

## Contributing

If you want to contribute to this project, you can create your fork or branch, make the changes you want and then create a Merge Request for the development branch, remembering that this project follows the following standards and conventions:

- <https://www.conventionalcommits.org/en/v1.0.0/>
- <https://keepachangelog.com/en/1.0.0/>
- <https://semver.org/lang/pt-BR/>

*Remembering that so far this project does not follow any standard or convention regarding naming branches, naming and describing Merge Requests or naming and describing Issues, just be coherent and concise in your naming and description, if you want to suggest a standard or convention for this project, use the Issues tab.

If you want to contribute with the translations, they are in files named by the language code used and provided by Discord (link to this list below), inside the `assets/langs` folder and formatted in JSON with a syntax similar to that used by Crowdin. Please copy and modify the original English file (`en-US.json`), remembering that the words wrapped in ${} are variables inserted at runtime so they should not be translated, you can use the file `pt-BR.json` as an example.

- <https://discord.com/developers/docs/reference#locales>

## Building

This project can be built from simple commands in Cargo, there are no changes, scripts or requirements in the process of building this project, if you don't know how to do this, below is a step by step on how to assemble the debug or release versions of project.

### Debug

1. Run in a terminal: `cargo build`

When finished, you will find the executable under `target/debug` and it will be named `hydrogen` on UNIX platforms or `hydrogen.exe` on Windows platforms.

### Release

1. Run in a terminal: `cargo build -r`

When finished, you will find the executable under `target/release` and it will be named `hydrogen` on UNIX platforms or `hydrogen.exe` on Windows platforms.

## Running

_TODO: Add details regarding Hydrogen dependencies and settings._

## Credits

_TODO: Add due credit to DeerSoftware, the contributors and developers of the libraries and projects used._
