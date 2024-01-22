//! Hydrogen // Handler
//!
//! Command and component handler for Hydrogen. Created to decrease the repeated code and heap allocations from the original handler.

use std::{collections::HashMap, result};

use hydrogen_i18n::I18n;
use serenity::{
    all::{Command, CommandId, CommandInteraction},
    builder::{CreateEmbed, CreateEmbedFooter, EditInteractionResponse},
    client::Context,
    http::Http,
};
use tokio::sync::RwLock;
use tracing::{debug, error, info};

use crate::{
    commands, HydrogenContext, HYDROGEN_ERROR_COLOR, HYDROGEN_LOGO_URL, HYDROGEN_PRIMARY_COLOR,
};

/// Type returned by commands and components to indicate how to respond to the interaction.
pub enum Response {
    /// Generic response, used for most commands and components.
    Generic {
        /// Embed's title.
        title: String,

        /// Embed's description.
        description: String,
    },
}

/// Command' and component's function return type.
pub type Result = result::Result<Response, Response>;

/// Handles a command interaction.
pub async fn handle_command(
    hydrogen: &HydrogenContext,
    context: &Context,
    command: &CommandInteraction,
) {
    // Defer the interaction to avoid the "This interaction failed" message.
    if let Err(e) = command.defer_ephemeral(&context.http).await {
        error!("(handle_command): failed to defer interaction: {}", e);
        return;
    }

    // Execute the command.
    let response = match command.data.name.as_str() {
        "join" => commands::join::execute(&hydrogen, &context, &command).await,
        _ => {
            error!("(handle_command): unknown command: {}", command.data.name);
            return;
        }
    };

    // Get the footer's text.
    let footer_text = hydrogen
        .i18n
        .translate(&command.locale, "generic", "embed_footer");

    // Create the embed.
    let message = match response {
        Ok(response) => create_embed(response, HYDROGEN_PRIMARY_COLOR, &footer_text),
        Err(response) => create_embed(response, HYDROGEN_ERROR_COLOR, &footer_text),
    };

    // Edit the response with the embed.
    if let Err(e) = command.edit_response(&context.http, message).await {
        error!("(handle_command): cannot respond to the interaction: {}", e);
    }
}

/// Creates an Discord embed.
fn create_embed(response: Response, color: i32, footer_text: &str) -> EditInteractionResponse {
    match response {
        Response::Generic { title, description } => EditInteractionResponse::new().embed(
            CreateEmbed::new()
                .title(title)
                .description(description)
                .color(color)
                .footer(CreateEmbedFooter::new(footer_text).icon_url(HYDROGEN_LOGO_URL)),
        ),
    }
}

/// Registers the commands.
pub async fn register_commands(
    i18n: Option<&I18n>,
    http: impl AsRef<Http>,
    commands_id: &RwLock<HashMap<String, CommandId>>,
) -> bool {
    // Prepare to write the commands' IDs.
    let mut commands_id = commands_id.write().await;

    // Create an array of commands.
    let commands = [commands::join::register(i18n)];

    // Register the commands.
    debug!(
        "(register_command): registering {} commands...",
        commands.len()
    );
    match Command::set_global_commands(http, commands.to_vec()).await {
        Ok(v) => {
            info!("(register_command): registered {} commands", v.len());

            // Write the commands' IDs.
            for commands in v {
                commands_id.insert(commands.name.clone(), commands.id);
            }

            true
        }
        Err(e) => {
            error!("(register_command): cannot register the commands: {}", e);

            false
        }
    }
}
