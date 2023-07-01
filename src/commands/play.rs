use async_trait::async_trait;
use serenity::{prelude::Context, model::prelude::{application_command::ApplicationCommandInteraction, command::CommandOptionType, ChannelId}, builder::CreateApplicationCommand};
use tracing::warn;

use crate::{HydrogenContext, lavalink::rest::{LavalinkUpdatePlayer, LavalinkVoiceState}, HydrogenCommandListener};

pub struct PlayCommand;

impl PlayCommand {
    async fn _execute(&self, hydrogen: HydrogenContext, context: Context, interaction: ApplicationCommandInteraction) -> Result<(), String> {
        let query = interaction
            .data.options.get(0).ok_or("required 'query' parameter missing".to_owned())?
            .value.clone().ok_or("required 'query' parameter missing".to_owned())?
            .as_str().ok_or("can't convert required 'query' to str".to_owned())?
            .to_owned();

        
        if let Err(e) = interaction.defer_ephemeral(&context.http).await {
            warn!("can't defer the response: {}", e);
        }

        let voice_manager = songbird::get(&context).await.ok_or("songbird not registered".to_owned())?;
        let guild_id = interaction.guild_id.ok_or("interaction doesn't have a guild_id".to_owned())?;
        let guild = context.cache.guild(guild_id).ok_or("guild isn't present in the cache".to_owned())?;

        let channel_id = match move || -> Result<ChannelId, Result<(), String>> {
            Ok(guild.voice_states.get(&interaction.user.id).ok_or(Err("can't find the user voice state in the origin guild".to_owned()))?
                .channel_id.ok_or(Err("can't get the channel id from the voice state".to_owned()))?)
        }() {
            Ok(v) => v,
            Err(e) => {
                if let Err(e) = interaction.edit_original_interaction_response(&context.http, |response| {
                    response
                        .content("You aren't in a voice chat!")
                }).await {
                    warn!("can't response to interaction: {:?}", e);
                }
                return e;
            }
        };

        let mut player = LavalinkUpdatePlayer::new().identifier(&query);

        if voice_manager.get(guild_id).is_none() {
            match voice_manager.join_gateway(guild_id, channel_id).await.1 {
                Ok(v) => {
                    player.voice_state(LavalinkVoiceState::new(&v.token, &v.endpoint, &v.session_id));

                    Ok(())
                },
                Err(e) => {
                    if let Err(e) = interaction.edit_original_interaction_response(&context.http, |response| {
                        response
                            .content("Can't connect to the voice chat!")
                    }).await {
                        warn!("can't response to interaction: {:?}", e);
                    }

                    Err(format!("can't join the voice chat: {}", e))
                }
            }?;
        }

        let music = hydrogen.lavalink.update_player(&guild_id.0.to_string(), false, player).await.map_err(|e| format!("can't update lavalink player: {:?}", e))?;
        let track = music.track.ok_or("lavalink update player don't has returned the track".to_owned())?;

        if let Err(e) = interaction.edit_original_interaction_response(&context.http, |response| {
            response
                .content(format!("Requested music: {}", track.info.title))
        }).await {
            warn!("can't response to interaction: {:?}", e);
        }

        Ok(())
    }
}

#[async_trait]
impl HydrogenCommandListener for PlayCommand {
    fn register<'a, 'b>(&'a self,command: &'b mut CreateApplicationCommand) ->  &'b mut CreateApplicationCommand {
        command
            .description("Searches and plays the requested song, initializing the player if necessary.")
            .create_option(|option| option
                .kind(CommandOptionType::String)
                .name("query")
                .description("The query to search for.")
                .required(true)
            )
            .dm_permission(false)
    }

    async fn execute(&self, hydrogen: HydrogenContext, context: Context, interaction: ApplicationCommandInteraction) {
        if let Err(e) = self._execute(hydrogen, context, interaction).await {
            warn!("{}", e);
        }
    }
}