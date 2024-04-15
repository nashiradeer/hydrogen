# Nashira Deer // Hydrogen

**Warning: Hydrogen is still in the alpha development stage.**

Open-source Dungeon Master helper with useful features and a music player for your sessions.

[![PayPal](https://img.shields.io/badge/Paypal-003087?style=for-the-badge&logo=paypal&logoColor=%23fff)
](https://www.paypal.com/donate/?business=QQGMTC3FQAJF6&no_recurring=0&item_name=Thanks+for+donating+for+me%2C+this+helps+me+a+lot+to+continue+developing+and+maintaining+my+projects.&currency_code=USD)
[![GitHub Sponsor](https://img.shields.io/badge/GitHub%20Sponsor-181717?style=for-the-badge&logo=github&logoColor=%23fff)
](https://github.com/sponsors/nashiradeer)
[![Discord](https://img.shields.io/badge/Discord%20Bot-5865F2?style=for-the-badge&logo=discord&logoColor=%23fff)](https://discord.com/api/oauth2/authorize?client_id=1128087591179268116&permissions=275417975808&scope=bot+applications.commands)
[![Docker](https://img.shields.io/docker/v/nashiradeer/hydrogen?style=for-the-badge&logo=docker&logoColor=%23fff&label=Docker&labelColor=%232496ED&color=%232496ED)](https://hub.docker.com/r/nashiradeer/hydrogen)

Manage RPG campaigns and role-play characters inside Discord without downloading external tools. You can create campaigns, sheet models, register notes, schedule sessions, generate loot and encounters, play music on voice chats, and many other features with Hydrogen!

## Donating

Whether you are using a public instance or your own, please consider donating to support Hydrogen's development. You can donate through Nashira Deer's [PayPal](https://www.paypal.com/donate/?business=QQGMTC3FQAJF6&no_recurring=0&item_name=Thanks+for+donating+for+me%2C+this+helps+me+a+lot+to+continue+developing+and+maintaining+my+projects.&currency_code=USD) or [GitHub Sponsor](https://github.com/sponsors/nashiradeer).

## Official Public Instance

**Warning: The public instance of Hydrogen will end on 30 April 2024 at 15:00.**

If you are interested in a public instance of Hydrogen, you can add our official instance, hosted by [Nashira Deer](https://github.com/nashiradeer), to your Discord server, by [clicking here](https://discord.com/api/oauth2/authorize?client_id=1128087591179268116&permissions=275417975808&scope=bot+applications.commands).

## Building/running

Only the methods listed below are officially supported and tested by Nashira Deer. We don't recommend using any other alternatives to build Hydrogen, as it isn't developed to support them.

### Docker

You can build Hydrogen using `docker build -t hydrogen:latest .` in a terminal with [Docker](https://docker.com) (Podman not supported) installed and running, after the build is completed, you will have a Docker image ready for use, named "hydrogen:latest".

If you don't want to build your own image, you can use our prebuilt image found on [Docker Hub](https://hub.docker.com/r/nashiradeer/hydrogen). To run it, you can see our example using [Docker Compose](compose.yaml).

## Configuring

There are two ways to configure Hydrogen, using the config file or environment variables, remembering that the environment variable is only used if the equivalent field isn't present on the config file.

### Config file

Hydrogen doesn't create the config file, so you need to download the example from [here](config.toml).

When starting, Hydrogen searches for the config file in `$XDG_CONFIG_HOME/hydrogen/config.toml` (`/etc/hydrogen/config.toml` when `$XDG_CONFIG_HOME` is not set) on UNIX-like platforms or `%APPDATA%\Hydrogen\Config.toml` (`C:\ProgramData\Hydrogen\Config.toml` when `%APPDATA` is not set) on Windows, you can change the path using the command line argument `--config-file [path]` or the environment variable `HYDROGEN_CONFIG_FILE` (the environment variable is only considered if the command line argument is not set).

### Environment Variables

- HYDROGEN_DISCORD_TOKEN: Sets the token that will be used to access Discord. (required)
- HYDROGEN_LAVALINK: Set the list of Lavalink nodes that can be used, read more below. (required)
- HYDROGEN_DEFAULT_LANGUAGE: Sets a new default language to Hydrogen. (optional)
- HYDROGEN_LANGUAGE_PATH: Sets the path where the Hydrogen translation files can be found. (optional)

You can see our example using [Docker Compose](compose.yaml).

#### HYDROGEN_LAVALINK syntax

```plain
value           = single-node *(";" single-node)
single-node     = host "," password ["," tls]
host            = ip ":" port
tls             = "true" / "enabled" / "on"
```

## Credits

Hydrogen is a project by Nashira Deer, licensed under [GNU General Public License v3](LICENSE.txt).
