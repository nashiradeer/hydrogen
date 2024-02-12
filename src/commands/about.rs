//! Hydrogen // Commands // About
//!
//! '/about' command registration and execution.

use hydrogen_i18n::I18n;
use serenity::{all::CommandInteraction, builder::CreateCommand, client::Context};

use crate::{
    handler::{Response, Result},
    HydrogenContext, ShardManagerRunners, HYDROGEN_BUG_URL, HYDROGEN_NAME, HYDROGEN_REPOSITORY_URL,
    HYDROGEN_VERSION,
};

/// Executes the `/about` command.
pub async fn execute(
    hydrogen: &HydrogenContext,
    context: &Context,
    interaction: &CommandInteraction,
) -> Result {
    // Construct the "Software" section.

    let name = format!(
        "\n{}",
        hydrogen
            .i18n
            .translate(&interaction.locale, "about", "software_name")
            .replace("{value}", HYDROGEN_NAME)
    );

    let version = format!(
        "\n{}",
        hydrogen
            .i18n
            .translate(&interaction.locale, "about", "version")
            .replace("{value}", HYDROGEN_VERSION)
    );

    let source_code = format!(
        "\n{}",
        hydrogen
            .i18n
            .translate(&interaction.locale, "about", "source_code")
            .replace("{value}", HYDROGEN_REPOSITORY_URL)
    );

    let bug_report = format!(
        "\n{}",
        hydrogen
            .i18n
            .translate(&interaction.locale, "about", "bug_report")
            .replace("{value}", HYDROGEN_BUG_URL)
    );

    let software_section = format!(
        "### {}{}{}{}{}",
        hydrogen
            .i18n
            .translate(&interaction.locale, "about", "software"),
        name,
        version,
        source_code,
        bug_report
    );

    // Construct the "Statistics" section.

    let players_count = match hydrogen.manager.read().await.as_ref() {
        Some(manager) => format!(
            "\n{}",
            hydrogen
                .i18n
                .translate(&interaction.locale, "about", "players")
                .replace("{value}", &manager.count_players().await.to_string())
        ),
        None => String::new(),
    };

    let latency = match context.data.read().await.get::<ShardManagerRunners>() {
        Some(shards) => match shards
            .lock()
            .await
            .get(&context.shard_id)
            .map(|v| v.latency)
            .flatten()
            .map(|v| v.as_millis())
        {
            Some(ping) => format!(
                "\n{}",
                hydrogen
                    .i18n
                    .translate(&interaction.locale, "about", "latency")
                    .replace("{value}", &ping.to_string())
            ),
            None => String::new(),
        },
        None => String::new(),
    };

    let shards = format!(
        "\n{}",
        hydrogen
            .i18n
            .translate(&interaction.locale, "about", "shards")
            .replace("{value}", &context.cache.shard_count().to_string())
    );

    let guilds = format!(
        "\n{}",
        hydrogen
            .i18n
            .translate(&interaction.locale, "about", "guilds")
            .replace("{value}", &context.cache.guild_count().to_string())
    );

    let statistics_section = format!(
        "\n### {}{}{}{}{}",
        hydrogen
            .i18n
            .translate(&interaction.locale, "about", "statistics"),
        players_count,
        shards,
        guilds,
        latency,
    );

    // Respond with the information.
    Ok(Response::Generic {
        title: hydrogen
            .i18n
            .translate(&interaction.locale, "about", "embed_title"),
        description: format!("{}{}", software_section, statistics_section),
    })
}

/// Registers the `/about` command.
///
/// If `i18n` is `None`, the translation will be ignored.
pub fn register(i18n: Option<&I18n>) -> CreateCommand {
    let mut command = CreateCommand::new("about");

    if let Some(i18n) = i18n {
        command = i18n.serenity_command_name("about", "name", command);
        command = i18n.serenity_command_description("about", "description", command);
    }

    command
        .description("Shows information about the bot.")
        .dm_permission(true)
}
