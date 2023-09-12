async fn update_now_playing(&self, guild_id: GuildId) {
    if let Some(player) = self.player.read().await.get(&guild_id.into()) {
        let mut player_state = HydrogenPlayerState::Playing;

        let (translated_message, requester) = match player.now().await {
            Some(v) => {
                let message = match v.uri {
                    Some(v) => self
                        .i18n
                        .translate(&player.guild_locale(), "playing", "description_uri")
                        .replace("${uri}", &v),
                    None => self
                        .i18n
                        .translate(&player.guild_locale(), "playing", "description"),
                }
                .replace("${music}", &v.title)
                .replace("${author}", &v.author);

                (message, Some(v.requester_id))
            }
            None => (
                self.i18n
                    .translate(&player.guild_locale(), "playing", "empty"),
                None,
            ),
        };

        let mut author_obj = None;
        if let Some(author) = requester {
            if let Ok(author_user) = author.to_user(self).await {
                let mut inner_author_obj = CreateEmbedAuthor::default();

                inner_author_obj.name(author_user.name.clone());

                if let Some(avatar_url) = author_user.avatar_url() {
                    inner_author_obj.icon_url(avatar_url);
                }

                author_obj = Some(inner_author_obj.to_owned());
            }
        }

        if requester.is_none() && player.queue().await.len() == 0 {
            player_state = HydrogenPlayerState::Nothing;
        }

        self.update_play_message(
            guild_id,
            &translated_message,
            HYDROGEN_PRIMARY_COLOR,
            player_state,
            player.pause(),
            player.loop_type().await,
            author_obj,
        )
        .await;
    }
}

async fn update_play_message(
    &self,
    guild_id: GuildId,
    description: &str,
    color: i32,
    player_state: HydrogenPlayerState,
    paused: bool,
    loop_type: LoopType,
    author_obj: Option<CreateEmbedAuthor>,
) {
    let players = self.player.read().await;
    let mut messages = self.message.write().await;

    if let Some(player) = players.get(&guild_id) {
        if let Some(message) = messages.get(&guild_id) {
            match player
                .text_channel_id()
                .edit_message(self.http.clone(), message, |message| {
                    message
                        .embed(|embed| {
                            if let Some(author_obj) = author_obj.clone() {
                                embed.set_author(author_obj);
                            }

                            embed
                                .title(self.i18n.translate(
                                    &player.guild_locale(),
                                    "playing",
                                    "title",
                                ))
                                .description(description)
                                .color(color)
                                .footer(|footer| {
                                    footer
                                        .text(self.i18n.translate(
                                            &player.guild_locale(),
                                            "embed",
                                            "footer_text",
                                        ))
                                        .icon_url(HYDROGEN_LOGO_URL)
                                })
                        })
                        .set_components(Self::play_components(
                            player_state.clone(),
                            paused,
                            loop_type.clone(),
                        ))
                })
                .await
            {
                Ok(_) => return,
                Err(e) => {
                    warn!("can't edit player message: {}", e);
                }
            }
        }

        match player
            .text_channel_id()
            .send_message(self.http.clone(), |message| {
                message
                    .embed(|embed| {
                        if let Some(author_obj) = author_obj {
                            embed.set_author(author_obj);
                        }

                        embed
                            .title(
                                self.i18n
                                    .translate(&player.guild_locale(), "playing", "title"),
                            )
                            .description(description)
                            .color(color)
                            .footer(|footer| {
                                footer
                                    .text(self.i18n.translate(
                                        &player.guild_locale(),
                                        "embed",
                                        "footer_text",
                                    ))
                                    .icon_url(HYDROGEN_LOGO_URL)
                            })
                    })
                    .set_components(Self::play_components(player_state, paused, loop_type))
            })
            .await
        {
            Ok(v) => {
                messages.insert(guild_id, v.id);
                ()
            }
            Err(e) => warn!("can't send a new playing message: {}", e),
        };
    }
}

fn play_components(
    state: HydrogenPlayerState,
    paused: bool,
    loop_queue: LoopType,
) -> CreateComponents {
    let mut prev_style = ButtonStyle::Primary;
    let mut pause_style = ButtonStyle::Primary;
    let mut skip_style = ButtonStyle::Primary;

    let mut prev_disabled = false;
    let mut pause_disabled = false;
    let mut skip_disabled = false;
    let mut loop_disabled = false;
    let mut stop_disabled = false;
    // QUEUE WILL REMAIN AS WIP UNTIL ALPHA 3.
    let mut queue_disabled = true;

    let mut pause_emoji = ReactionType::Unicode(String::from("⏸"));
    let mut loop_emoji = ReactionType::Unicode(String::from("⬇️"));

    match loop_queue {
        LoopType::None => (),
        LoopType::NoAutostart => {
            loop_emoji = ReactionType::Unicode(String::from("⏺"));
        }
        LoopType::Music => {
            loop_emoji = ReactionType::Unicode(String::from("🔂"));
        }
        LoopType::Queue => {
            loop_emoji = ReactionType::Unicode(String::from("🔁"));
        }
        LoopType::Random => {
            loop_emoji = ReactionType::Unicode(String::from("🔀"));
        }
    };

    if paused {
        pause_style = ButtonStyle::Success;
        pause_emoji = ReactionType::Unicode(String::from("▶️"));
    }

    match state {
        HydrogenPlayerState::Playing => (),
        HydrogenPlayerState::Nothing => {
            prev_disabled = true;
            pause_disabled = true;
            skip_disabled = true;
            queue_disabled = true;

            prev_style = ButtonStyle::Secondary;
            pause_style = ButtonStyle::Secondary;
            skip_style = ButtonStyle::Secondary;
        }
        HydrogenPlayerState::Thinking => {
            prev_disabled = true;
            pause_disabled = true;
            skip_disabled = true;
            loop_disabled = true;
            stop_disabled = true;
            queue_disabled = true;
        }
    }

    CreateComponents::default()
        .create_action_row(|action_row| {
            action_row
                .create_button(|button| {
                    button
                        .custom_id("prev")
                        .disabled(prev_disabled)
                        .emoji('⏮')
                        .style(prev_style)
                })
                .create_button(|button| {
                    button
                        .custom_id("pause")
                        .disabled(pause_disabled)
                        .emoji(pause_emoji)
                        .style(pause_style)
                })
                .create_button(|button| {
                    button
                        .custom_id("skip")
                        .disabled(skip_disabled)
                        .emoji('⏭')
                        .style(skip_style)
                })
        })
        .create_action_row(|action_row| {
            action_row
                .create_button(|button| {
                    button
                        .custom_id("loop")
                        .disabled(loop_disabled)
                        .emoji(loop_emoji)
                        .style(ButtonStyle::Secondary)
                })
                .create_button(|button| {
                    button
                        .custom_id("stop")
                        .disabled(stop_disabled)
                        .emoji('⏹')
                        .style(ButtonStyle::Danger)
                })
                .create_button(|button| {
                    button
                        .custom_id("queue")
                        .disabled(queue_disabled)
                        .emoji(ReactionType::Unicode("ℹ️".to_owned()))
                        .style(ButtonStyle::Secondary)
                })
        })
        .to_owned()
}
