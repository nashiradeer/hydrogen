# Hydrogen // Changelog

## [Unreleased]

## [0.0.1-alpha.4] - 2024-03-12

### Added

- Create a link from ES-419 (LATAM) to ES-ES.
- Create a message handler to roll dices.
- Create a roll engine.
- Create a roll syntax parser.
- Create '/about' command.
- Create '/roll' command.
- New default 'builtin-language' feature to include 'assets/langs/en-US.json' in the binary as default language.
- Old component message auto remover.

### Changed

- Decrease update_voice_server and update_voice_state logs spamming, ignoring them when nothing has been occurred.
- Replace internal i18n with 'hydrogen-i18n'.
- Resume when Player::play() is called when Lavalink is playing nothing.
- New command register and handler.
- New component handler.

### Fixed

- Change from '\d' to '[0-9]' to avoid Regex match non-ASCII digits.

## [0.0.1-alpha.3] - 2023-01-08

### Added

- Create de-DE translation.
- Create es-ES translation.
- Create a HashMap with command's IDs.

### Changed

- Refactor en-US translation.
- Refactor error messages.
- Refactor log messages.
- Refactor translation keys.
- Update pt-BR translation.

### Fixed

- Missing variable value in JoinCommand. (Issue #15)
- Wrong translation key in LoopComponent. (Issue #16)
- Wrong translations variables in HydrogenManager. (Issue #16)
