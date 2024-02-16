//! Hydrogen // Commands // Roll
//!
//! '/roll' command registration and execution.

use hydrogen_i18n::I18n;
use serenity::{
    all::{CommandDataOptionValue, CommandInteraction, CommandOptionType},
    builder::{CreateCommand, CreateCommandOption},
    client::Context,
};
use tracing::error;

use crate::{
    handler::{Response, Result},
    roll::{DiceType, Params},
    HydrogenContext, HYDROGEN_BUG_URL,
};

/// Executes the `/roll` command.
pub async fn execute(
    hydrogen: &HydrogenContext,
    _: &Context,
    interaction: &CommandInteraction,
) -> Result {
    // Get the title of the embed.
    let title = hydrogen
        .i18n
        .translate(&interaction.locale, "roll", "embed_title");

    // Get the sub-command.
    let sub_command = match interaction.data.options.get(0) {
        Some(sub_command) => sub_command,
        None => {
            error!("cannot get the 'sub-command' option");

            return Err(Response::Generic {
                title,
                description: hydrogen
                    .i18n
                    .translate(&interaction.locale, "error", "unknown")
                    .replace("{url}", HYDROGEN_BUG_URL),
            });
        }
    };

    // Get the sub-command data.
    let CommandDataOptionValue::SubCommand(ref sub_command_data) = sub_command.value else {
        error!("cannot get the 'sub-command' data");

        return Err(Response::Generic {
            title,
            description: hydrogen
                .i18n
                .translate(&interaction.locale, "error", "unknown")
                .replace("{url}", HYDROGEN_BUG_URL),
        });
    };

    // Get the roll parameters. The index of the options is different for each sub-command.
    let params = match sub_command.name.as_str() {
        "fate" => {
            // Create the parameters.
            let mut params = Params {
                dice_type: DiceType::Fate,
                ..Default::default()
            };

            // Get the dice count value, changing the default value if it's present.
            if let Some(dice_count) = sub_command_data.get(0).map(|v| v.value.as_i64()).flatten() {
                params.dice_count = dice_count as u8;
            }

            // Get the repetitions value, changing the default value if it's present.
            if let Some(repetitions) = sub_command_data.get(1).map(|v| v.value.as_i64()).flatten() {
                params.repeat = repetitions as u8;
            }

            // Get the modifier value, changing the default value if it's present.
            if let Some(modifier) = sub_command_data
                .get(2)
                .map(|v| v.value.as_str())
                .flatten()
                .map(|s| hydrogen.roll_parser.evaluate_modifier(s))
                .flatten()
            {
                params.modifier = modifier;
            }

            params
        }
        "sided" => {
            // Create the parameters.
            let mut params = Params::default();

            // Get the dice sides value, changing the default value if it's present.
            if let Some(dice_sides) = sub_command_data.get(0).map(|v| v.value.as_i64()).flatten() {
                params.dice_type = DiceType::Sided(dice_sides as u8);
            };

            // Get the dice count value, changing the default value if it's present.
            if let Some(dice_count) = sub_command_data.get(1).map(|v| v.value.as_i64()).flatten() {
                params.dice_count = dice_count as u8;
            }

            // Get the repetitions value, changing the default value if it's present.
            if let Some(repetitions) = sub_command_data.get(2).map(|v| v.value.as_i64()).flatten() {
                params.repeat = repetitions as u8;
            }

            // Get the modifier value, changing the default value if it's present.
            if let Some(modifier) = sub_command_data
                .get(3)
                .map(|v| v.value.as_str())
                .flatten()
                .map(|s| hydrogen.roll_parser.evaluate_modifier(s))
                .flatten()
            {
                params.modifier = modifier;
            }

            params
        }
        _ => unreachable!(),
    };

    // Roll the dice.
    let result = match params.roll() {
        Ok(v) => v,
        Err(e) => {
            error!(
                "cannot roll the dice for the user {}: {}",
                interaction.user.id, e
            );

            return Err(Response::Generic {
                title,
                description: hydrogen
                    .i18n
                    .translate(&interaction.locale, "error", "unknown")
                    .replace("{url}", HYDROGEN_BUG_URL),
            });
        }
    };

    Ok(Response::Generic {
        title,
        description: result.to_string(),
    })
}

/// Registers the `/roll` command.
///
/// If `i18n` is `None`, the translation will be ignored.
pub fn register(i18n: Option<&I18n>) -> CreateCommand {
    // Create the option for the number of sides on the dice.
    let mut dice_side_option = CreateCommandOption::new(
        CommandOptionType::Integer,
        "sides",
        "The amount of sides the dice will have.",
    )
    .required(true)
    .max_int_value(100)
    .min_int_value(2);

    // Create the option for the number of dice to roll.
    let mut dice_count_option = CreateCommandOption::new(
        CommandOptionType::Integer,
        "dice_count",
        "The amount of dices to roll.",
    )
    .required(false)
    .max_int_value(50)
    .min_int_value(1);

    // Create the option for the number of times to roll the dices.
    let mut roll_repeat_option = CreateCommandOption::new(
        CommandOptionType::Integer,
        "repetitions",
        "The amount of times to roll the dices.",
    )
    .required(false)
    .max_int_value(10)
    .min_int_value(1);

    // Create the option for the modifier.
    let mut modifier_option = CreateCommandOption::new(
        CommandOptionType::String,
        "modifier",
        "The modifier to add to the roll. (Need to be modifier like +2 or -2)",
    )
    .required(false);

    // Translate the options.
    if let Some(i18n) = i18n {
        // Translate the dice side option.
        dice_side_option =
            i18n.serenity_command_option_name("roll", "dice_sides_name", dice_side_option);
        dice_side_option = i18n.serenity_command_option_description(
            "roll",
            "dice_sides_description",
            dice_side_option,
        );

        // Translate the dice count option.
        dice_count_option =
            i18n.serenity_command_option_name("roll", "dice_count_name", dice_count_option);
        dice_count_option = i18n.serenity_command_option_description(
            "roll",
            "dice_count_description",
            dice_count_option,
        );

        // Translate the roll repeat option.
        roll_repeat_option =
            i18n.serenity_command_option_name("roll", "repetitions_name", roll_repeat_option);
        roll_repeat_option = i18n.serenity_command_option_description(
            "roll",
            "repetitions_description",
            roll_repeat_option,
        );

        // Translate the modifier option.
        modifier_option =
            i18n.serenity_command_option_name("roll", "modifier_name", modifier_option);
        modifier_option = i18n.serenity_command_option_description(
            "roll",
            "modifier_description",
            modifier_option,
        );
    }

    // Create the fate sub-command.
    let mut fate_command =
        CreateCommandOption::new(CommandOptionType::SubCommand, "fate", "Roll a fate dice.")
            .add_sub_option(dice_count_option.clone())
            .add_sub_option(roll_repeat_option.clone())
            .add_sub_option(modifier_option.clone());

    // Create the sides sub-command.
    let mut sided_command = CreateCommandOption::new(
        CommandOptionType::SubCommand,
        "sided",
        "Roll a dice with a specific number of sides.",
    )
    .add_sub_option(dice_side_option)
    .add_sub_option(dice_count_option)
    .add_sub_option(roll_repeat_option)
    .add_sub_option(modifier_option);

    // Translate the sub-commands.
    if let Some(i18n) = i18n {
        // Translate the fate sub-command.
        fate_command = i18n.serenity_command_option_name("roll", "fate_name", fate_command);
        fate_command =
            i18n.serenity_command_option_description("roll", "fate_description", fate_command);

        // Translate the sided sub-command.
        sided_command = i18n.serenity_command_option_name("roll", "sided_name", sided_command);
        sided_command =
            i18n.serenity_command_option_description("roll", "sided_description", sided_command);
    }

    // Create the roll command.
    let mut command = CreateCommand::new("roll")
        .add_option(fate_command)
        .add_option(sided_command);

    // Translate the command.
    if let Some(i18n) = i18n {
        command = i18n.serenity_command_name("roll", "name", command);
        command = i18n.serenity_command_description("roll", "description", command);
    }

    command.description("Roll a dice.")
}
