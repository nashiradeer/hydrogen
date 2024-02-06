# Hydrogen // Changelog

## [Unreleased]

### Added

- New default 'builtin-language' feature to include 'assets/langs/en-US.json' in the binary as default language.
- Old component message auto remover.

### Changed

- Replace internal i18n with 'hydrogen-i18n'.
- New command register and handler.
- New component handler.

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
