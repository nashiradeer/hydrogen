use std::{collections::HashMap, error::Error};

use serde_json::Value;
use serenity::{
    builder::CreateApplicationCommand,
    http::client::Http,
    json,
    model::application::interaction::{
        application_command::ApplicationCommandInteraction, autocomplete::AutocompleteInteraction,
        message_component::MessageComponentInteraction, modal::ModalSubmitInteraction,
    },
    prelude::Context,
};

use crate::lang::HydrogenLang;

pub trait HydrogenApplicationCommand<T> {
    fn command_name(&self) -> String;
    fn register(
        &self,
        i18n: &HydrogenLang,
        command: &mut CreateApplicationCommand,
    ) -> &mut CreateApplicationCommand;
    fn command(
        &self,
        handler: &mut HydrogenHandler<T>,
        i18n: &HydrogenLang,
        ctx: Context,
        interaction: ApplicationCommandInteraction,
    ) -> Result<(), Box<dyn Error>>;
    fn autocomplete(
        &self,
        _handler: &mut HydrogenHandler<T>,
        _i18n: &HydrogenLang,
        _ctx: Context,
        _interaction: AutocompleteInteraction,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

pub trait HydrogenMessageComponent<T> {
    fn custom_id(&self) -> String;
    fn component(
        &self,
        handler: &mut HydrogenHandler<T>,
        i18n: &HydrogenLang,
        ctx: Context,
        interaction: MessageComponentInteraction,
    ) -> Result<(), Box<dyn Error>>;
}

pub trait HydrogenModal<T> {
    fn custom_id(&self) -> String;
    fn modal(
        &self,
        handler: &mut HydrogenHandler<T>,
        i18n: &HydrogenLang,
        ctx: Context,
        interaction: ModalSubmitInteraction,
    ) -> Result<(), Box<dyn Error>>;
}

pub enum HydrogenCommand<T> {
    ApplicationCommand(Box<dyn HydrogenApplicationCommand<T>>),
    MessageComponent(Box<dyn HydrogenMessageComponent<T>>),
    Modal(Box<dyn HydrogenModal<T>>),
}

pub struct HydrogenHandler<T> {
    commands: HashMap<String, Box<dyn HydrogenApplicationCommand<T>>>,
    components: HashMap<String, Box<dyn HydrogenMessageComponent<T>>>,
    modals: HashMap<String, Box<dyn HydrogenModal<T>>>,

    pub i18n: HydrogenLang,
    pub memory: T,
}

impl<T> HydrogenHandler<T> {
    pub fn new(i18n: HydrogenLang, memory: T) -> Self {
        HydrogenHandler {
            commands: HashMap::new(),
            components: HashMap::new(),
            modals: HashMap::new(),
            i18n,
            memory,
        }
    }

    pub fn register(&mut self, command: HydrogenCommand<T>) {
        match command {
            HydrogenCommand::ApplicationCommand(application_command) => {
                self.commands
                    .insert(application_command.command_name(), application_command);
            }

            HydrogenCommand::MessageComponent(message_component) => {
                self.components
                    .insert(message_component.custom_id(), message_component);
            }

            HydrogenCommand::Modal(modal) => {
                self.modals.insert(modal.custom_id(), modal);
            }
        }
    }

    pub async fn create_application_commands(&self, http: Http) -> Result<i32, serenity::Error> {
        let mut result = 0;

        for command in self.commands.values() {
            let mut command_data = CreateApplicationCommand::default();

            command.register(&self.i18n, &mut command_data);

            let command_data_json = json::hashmap_to_json_map(command_data.0);

            if let Err(err) = http
                .as_ref()
                .create_global_application_command(&Value::from(command_data_json))
                .await
            {
                return Err(err);
            }

            result += 1;
        }

        Ok(result)
    }
}
