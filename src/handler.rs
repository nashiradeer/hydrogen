//! Hydrogen // Handler
//!
//! Command and component handler for Hydrogen. Created to decrease the repeated code and heap allocations from the original handler.

use std::{
    collections::HashMap,
    result,
    sync::Arc,
    time::{Duration, SystemTime},
};

use dashmap::DashMap;
use hydrogen_i18n::I18n;
use rand::{thread_rng, Rng};
use serenity::{
    all::{
        ChannelId, Command, CommandId, CommandInteraction, ComponentInteraction,
        CreateInteractionResponse, CreateInteractionResponseMessage, UserId,
    },
    builder::{CreateEmbed, CreateEmbedFooter, EditInteractionResponse},
    client::Context,
    http::{CacheHttp, Http},
};
use tokio::{spawn, sync::RwLock, task::JoinHandle, time::sleep};
use tracing::{debug, error, info, warn};

use crate::{
    commands, components, HydrogenContext, HYDROGEN_COLOR, HYDROGEN_ERROR_COLOR, HYDROGEN_LOGO_URL,
    HYDROGEN_PRIMARY_COLOR, HYDROGEN_REPOSITORY_URL, HYDROGEN_WARNING_PROBABILITY,
    HYDROGEN_WARNING_TIMEOUT,
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

/// Type used to monitor the responses sent by the bot.
pub type AutoRemoverKey = (ChannelId, UserId);

/// Handles a command interaction.
pub async fn handle_command(
    hydrogen: &HydrogenContext,
    context: &Context,
    command: &CommandInteraction,
) {
    if thread_rng().gen_bool(HYDROGEN_WARNING_PROBABILITY) && hydrogen.public_instance {
        // Send a message to the user.
        if let Err(e) = command
            .create_response(&context.http, hydrogen_end_message(command, &hydrogen.i18n))
            .await
        {
            error!("(handle_command): cannot respond to the interaction: {}", e);
            return;
        }

        sleep(Duration::from_secs(HYDROGEN_WARNING_TIMEOUT)).await;
    } else {
        // Defer the interaction to avoid the "This interaction failed" message.
        if let Err(e) = command.defer_ephemeral(&context.http).await {
            error!("(handle_command): failed to defer interaction: {}", e);
            return;
        }
    }

    // Execute the command.
    let response = match command.data.name.as_str() {
        "join" => commands::join::execute(hydrogen, context, command).await,
        "seek" => commands::seek::execute(hydrogen, context, command).await,
        "play" => commands::play::execute(hydrogen, context, command).await,
        "about" => commands::about::execute(hydrogen, context, command).await,
        "roll" => commands::roll::execute(hydrogen, context, command).await,
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

/// Handles a component interaction.
pub async fn handle_component(
    hydrogen: &HydrogenContext,
    context: &Context,
    component: &ComponentInteraction,
) {
    // Defer the interaction to avoid the "This interaction failed" message.
    if let Err(e) = component.defer_ephemeral(&context.http).await {
        error!("(handle_component): failed to defer interaction: {}", e);
        return;
    }

    // Execute the component.
    let response = match component.data.custom_id.as_str() {
        "loop" => components::loop_switch::execute(hydrogen, context, component).await,
        "pause" => components::pause::execute(hydrogen, context, component).await,
        "prev" => components::prev::execute(hydrogen, context, component).await,
        "skip" => components::skip::execute(hydrogen, context, component).await,
        "stop" => components::stop::execute(hydrogen, context, component).await,
        _ => {
            error!(
                "(handle_component): unknown component: {}",
                component.data.custom_id
            );
            return;
        }
    };

    // Get the footer's text.
    let footer_text = hydrogen
        .i18n
        .translate(&component.locale, "generic", "embed_footer");

    // Create the embed.
    let message = match response {
        Ok(response) => create_embed(response, HYDROGEN_PRIMARY_COLOR, &footer_text),
        Err(response) => create_embed(response, HYDROGEN_ERROR_COLOR, &footer_text),
    };

    // Edit the response with the embed.
    match component.edit_response(&context.http, message).await {
        Ok(v) => {
            // Clone the objects to send them to the autoremover.
            let responses = hydrogen.components_responses.clone();

            // Create the autoremover key.
            let auto_remover_key = (v.channel_id, component.user.id);

            // Spawn the autoremover.
            let auto_remover = spawn(async move {
                autoremover(auto_remover_key, responses).await;
            });

            // Store the new message in the cache.
            if let Some((auto_remover, old_component)) = hydrogen
                .components_responses
                .insert(auto_remover_key, (auto_remover, component.clone()))
            {
                // Abort the handler.
                auto_remover.abort();

                // Delete the old message.
                if let Err(e) = old_component.delete_response(context.http()).await {
                    warn!(
                        "(handle_component): cannot delete the message {:?}: {}",
                        auto_remover_key, e
                    );
                }
            }
        }
        Err(e) => {
            error!(
                "(handle_component): cannot respond to the interaction: {}",
                e
            );
        }
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
    let commands = [
        commands::join::register(i18n),
        commands::seek::register(i18n),
        commands::play::register(i18n),
        commands::about::register(i18n),
        commands::roll::register(i18n),
    ];

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

/// Removes the response after a certain time.
async fn autoremover(
    key: AutoRemoverKey,
    responses: Arc<DashMap<AutoRemoverKey, (JoinHandle<()>, ComponentInteraction)>>,
) {
    sleep(Duration::from_secs(10)).await;
    debug!("(autoremover): removing response {:?} from cache...", key);
    responses.remove(&key);
}

fn hydrogen_end_message(command: &CommandInteraction, i18n: &I18n) -> CreateInteractionResponse {
    CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .ephemeral(true)
            .embed(
                CreateEmbed::new()
                    .title(i18n.translate(&command.locale, "public_instance", "title"))
                    .description(format!(
                        "{}\n\n{}",
                        i18n.translate(&command.locale, "public_instance", "ending")
                            .replace("{time}", "<t:1714489200>")
                            .replace("{url}", HYDROGEN_REPOSITORY_URL),
                        i18n.translate(&command.locale, "public_instance", "running_in")
                            .replace(
                                "{time}",
                                &format!(
                                    "<t:{}:R>",
                                    (SystemTime::now()
                                        + Duration::from_secs(HYDROGEN_WARNING_TIMEOUT + 2))
                                    .duration_since(SystemTime::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs()
                                )
                            )
                    ))
                    .color(HYDROGEN_COLOR)
                    .footer(
                        CreateEmbedFooter::new(i18n.translate(
                            &command.locale,
                            "generic",
                            "embed_footer",
                        ))
                        .icon_url(HYDROGEN_LOGO_URL),
                    ),
            ),
    )
}
