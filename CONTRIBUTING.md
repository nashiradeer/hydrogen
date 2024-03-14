# Hydrogen // Contributing

## Making contributions

*TODO: Write about how contributions can be made and what is the process used by Hydrogen.*

## Standards and conventions

### Commits

The Hydrogen's Commit Standard is based on [Angular's Commit Message Guidelines](https://github.com/angular/angular/blob/22b96b9/CONTRIBUTING.md#-commit-message-guidelines) with a simpler format that only the header (`<type>(<scope>): <subject>`) is present.

#### Commit types

- build: Changes on the build system related files. (like `Dockerfile` or `Cargo.toml`)
- chore: Other types of changes like changes on the VSCode's `launch.json` or `.gitignore`.
- ci: Changes to our CI configuration files and scripts.
- docs: Changes to the documentation files.
- feat: A new feature.
- fix: A bug fix.
- i18n: Changes to the translation files.
- refactor: A code rewrite that doesn't affect the API.
- refactor!: A code rewrite that affects the API.

#### Commit scopes

##### Scopes for feat, fix, and refactor

- commands
- components
- lavalink
- config
- handler
- main
- manager
- player
- parsers
- player
- roll
- utils

##### Scopes for docs

- changelog
- contributing
- example
- readme

##### Scopes for build

- cargo
- docker

##### Scopes for i18n

You can use the file name without the extension (`pt-BR.json` becomes `pt-BR`) as the scope for the i18n type.

#### Commit subjects

A description containing what happened on this commit using the imperative, present tense ("change" not "changed" nor "changes") with the first letter lowercase, not ending with dot (.).

### Branches

*TODO: Write the Hydrogen's Branch Naming Convention based on <https://dev.to/varbsan/a-simplified-convention-for-naming-branches-and-commits-in-git-il4>*

### Changelog

The [Hydrogen's Changelog file](CHANGELOG.md) and [GitHub Releases](https://github.com/nashiradeer/hydrogen/releases) is based on [Keep a changelog v1.1.0](https://keepachangelog.com/en/1.1.0/) without any change.

## Translations

If you want to contribute with translations, you can edit the files found on `assets/langs` path, they are named using the [Discord's language code](https://discord.com/developers/docs/reference#locales) with `.json` on the end. Just copy the `en-US.json` file and modify your content, remembering that the words involved in {} are variables replaced at runtime, so they should not be translated, you can use the `pt-BR.json`, `de.json` and `es-ES.json` files as example.

To link a language to another you can create a file named with the [Discord's language code](https://discord.com/developers/docs/reference#locales) ending with `.link` (`es-419.link`) and with the content `_link:` followed with the language code to be linked (like `_link:es-ES`).

## About 'Hydrogen Framework' project

Hydrogen Framework is composed by all the projects related to Hydrogen like `hydrogen-i18n` and `hydrolink`, and there's a [GitHub Project](https://github.com/users/nashiradeer/projects/8) to track the proposed ideas and development progress of Hydrogen, if you want to implement something please check if it is already tracked on the project (anything not tracked will be rejected) and if not, consider suggesting thought [GitHub Issues](https://github.com/nashiradeer/hydrogen/issues).
