# Hydrogen // Contributing

## How to make contributions?

First, create your branch following [Hydrogen's Branch Naming Convention](#branches) and commits using [Hydrogen's Commit Standard](#commits), remember that your branch needs to have only one change, this doesn't mean how many files are affected or commits have taken but about have been changed (if you are changing how commands are handled, you will not fix a bug in the music player for example).

After finishing your changes on your branch, create a Pull Request to the `main` branch, write a concise title in imperative, present tense (like `Change how commands are handled`) and describing what you have changed and why (if applicable), and a reviewer will approve close or comment, and depending on the decision of the reviewer, your branch will be merged. Consider that if your Pull Request is approved this doesn't mean it will be merged, because this depends on what type (major, minor or patch) the next version will be, in this case, you will need to wait before seeing your changes on the upstream.

Please, don't forget to log your changes in the [CHANGELOG.md file](CHANGELOG.md) following the [Keep a changelog v1.1.0](https://keepachangelog.com/en/1.1.0/) before making the Pull Request.

## Standards and conventions

### Commits

The Hydrogen's Commit Standard is based on [Angular's Commit Message Guidelines](https://github.com/angular/angular/blob/22b96b9/CONTRIBUTING.md#-commit-message-guidelines) with a simpler format that only the header (`<type>(<scope>): <subject>`) is present.

#### Commit types

- build: Changes on the build system-related files. (like `Dockerfile` or `Cargo.toml`)
- chore: Other types of changes like changes on the VSCode's `launch.json` or `.gitignore`.
- ci: Changes to our CI configuration files and scripts.
- docs: Changes to the documentation files.
- feat: A new feature.
- fix: A bug fix.
- i18n: Changes to the translation files.
- refactor: A code rewrite that doesn't affect the API.
- refactor!: A code rewrite that affects the API.

#### Commit scopes

Commit scopes describe what has been affected by the commit, below is a list of scopes used in this repository:

- commands
- components
- lavalink
- config
- handler
- main
- parser
- player
- roll
- utils

If your commit is of type `i18n` you will use the file name without the extension (`pt-BR.json` becomes `pt-BR`) as the scope.

You can omit the scope (example `feat: create new module for ...`) if your commit there's no scope documented or is from other types that don't `feat`, `fix`, `i18n` and `refactor`.

#### Commit subjects

A short description containing what happened on this commit using the imperative, present tense ("change" not "changed" nor "changes") with the first letter lowercase, not ending with a dot (.).

### Branches

Hydrogen's Branch Naming Conventions are based on the [Simplified Convention for Naming Branches](https://dev.to/varbsan/a-simplified-convention-for-naming-branches-and-commits-in-git-il4) and are similar to [Hydrogen's Commit Standard](#commits) and have the format `<type>/<description>`.

#### Branch type

- docs: Changes to the documentation files.
- feat: A new feature or refactor on some existing feature.
- fix: A bug fix.
- i18n: Changes to the translation files.
- chore: Changes to non-documented things, like CI or build system files.

#### Branch description

A short description containing what will be changed on that branch using the imperative, present tense ("change" not "changed" nor "changes").

### Changelog

The [Hydrogen's Changelog file](CHANGELOG.md) and [GitHub Releases](https://github.com/nashiradeer/hydrogen/releases) is based on [Keep a changelog v1.1.0](https://keepachangelog.com/en/1.1.0/) without any change.

## Translations

If you want to contribute with translations, you can edit the files found on `assets/langs` path, they are named using the [Discord's language code](https://discord.com/developers/docs/reference#locales) with `.json` on the end. Just copy the `en-US.json` file and modify your content, remembering that the words involved in {} are variables replaced at runtime, so they should not be translated, you can use the `pt-BR.json`, `de.json` and `es-ES.json` files as examples.

To link a language to another, you can create a file named with one of [Discord's language codes](https://discord.com/developers/docs/reference#locales) ending with `.link` (`es-419.link`) and with the content `_link:` followed by the language code to be linked (like `_link:es-ES`).

## About 'Hydrogen Framework' project

Hydrogen Framework is composed of all the projects related to Hydrogen like `hydrogen-i18n` and `hydrolink`, and there's a [GitHub Project](https://github.com/users/nashiradeer/projects/8) to track the proposed ideas and development progress of Hydrogen, if you want to implement something please check if it is already tracked on the project (anything not tracked will be rejected) and if not, consider suggesting thought [GitHub Issues](https://github.com/nashiradeer/hydrogen/issues).
