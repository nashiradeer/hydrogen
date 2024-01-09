# Nashira Deer // Hydrogen

**Warning: Hydrogen still in the alpha development stage.**

Open-source Dungeon Master helper with useful features and a music player for your sessions.

[![PayPal](https://img.shields.io/badge/Paypal-003087?style=for-the-badge&logo=paypal&logoColor=%23fff)
](https://www.paypal.com/donate/?business=QQGMTC3FQAJF6&no_recurring=0&item_name=Thanks+for+donating+for+me%2C+this+helps+me+a+lot+to+continue+developing+and+maintaining+my+projects.&currency_code=USD)
[![GitHub Sponsor](https://img.shields.io/badge/GitHub%20Sponsor-181717?style=for-the-badge&logo=github&logoColor=%23fff)
](https://github.com/sponsors/nashiradeer)
[![Discord](https://img.shields.io/badge/Discord%20Bot-5865F2?style=for-the-badge&logo=discord&logoColor=%23fff)](https://discord.com/api/oauth2/authorize?client_id=1128087591179268116&permissions=275417975808&scope=bot+applications.commands)
[![Docker](https://img.shields.io/docker/v/nashiradeer/hydrogen?style=for-the-badge&logo=docker&logoColor=%23fff&label=Docker&labelColor=%23000&color=%232496ED)](https://hub.docker.com/r/nashiradeer/hydrogen)

Manage RPG campaigns and role-play characters inside Discord without downloading external tools. You can create campaigns, sheet models, register notes, schedule sessions, generate loot and encounters, play music on voice chats, and many other features with Hydrogen!

## Donating

Independent if you are using public instance, or it owns instance, consider donating to make Hydrogen development possible. You can donate thought Nashira Deer's [PayPal](https://www.paypal.com/donate/?business=QQGMTC3FQAJF6&no_recurring=0&item_name=Thanks+for+donating+for+me%2C+this+helps+me+a+lot+to+continue+developing+and+maintaining+my+projects.&currency_code=USD) or [GitHub Sponsor](https://github.com/sponsors/nashiradeer).

## Official Public Instance

If you are interested in a public instance of Hydrogen, you can add our official instance, hosted by [Nashira Deer](https://github.com/nashiradeer), to your Discord's server, [clicking here](https://discord.com/api/oauth2/authorize?client_id=1128087591179268116&permissions=275417975808&scope=bot+applications.commands).

## Building/running

Only the methods listed below is officially supported and tested by Nashira Deer, we don't recommend you using any other alternative to build Hydrogen as Hydrogen isn't developed to support it.

### Docker

You can build Hydrogen using `docker build -t hydrogen:latest .` in a terminal with [Docker](https://docker.com) (Podman not supported) installed and running, before the build is completed you will have a ready to use Docker image available with "hydrogen:latest" name.

If you don't want to build your own image, you can use our prebuilt image found on [Docker Hub](https://hub.docker.com/r/nashiradeer/hydrogen). To run it, you can see our example using [Docker Compose](https://github.com/nashiradeer/hydrogen/blob/main/compose.yaml).

## Configuring

To configure Hydrogen you will use the following environment variables:

- LANGUAGE_PATH: Sets the path where the Hydrogen translation files can be found. (optional)
- LAVALINK: Set the list of Lavalink nodes that can be used, read more below. (required)
- DISCORD_TOKEN: Sets the token that will be used to access a Discord. (required)

You can see our example using [Docker Compose](https://github.com/nashiradeer/hydrogen/blob/main/compose.yaml).

### LAVALINK environment variable syntax

```plain
value           = single-node *(";" single-node)
single-node     = host "," password ["," tls]
host            = ip ":" port
tls             = "true" / "enabled" / "on"
```

*TLS parameter needs to be written in lower case.

## Credits

Hydrogen is a Nashira Deer's project licensed under the [GNU General Public License v3](https://github.com/nashiradeer/hydrogen/blob/main/LICENSE.txt).
