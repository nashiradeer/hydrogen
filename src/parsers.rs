//! Hydrogen // Parsers
//!
//! Contains the parsers used by Hydrogen.

use regex::Regex;

use crate::roll::{DiceType, Modifier, Params};

/// Holds the parsers used to parse different  time syntaxes.
pub struct TimeParser {
    /// Regex parser for the suffix syntax.
    suffix_parser: Regex,

    /// Regex parser for the semicolon syntax.
    semicolon_parser: Regex,
}

impl TimeParser {
    /// Creates a new instance of the time parser.
    pub fn new() -> Result<Self, regex::Error> {
        Ok(Self {
            suffix_parser: Regex::new(r"^(([0-9]{1,3})[sS]?|([0-9]{1,3})[mM]|([0-9]{1,3})[hH])$")?,
            semicolon_parser: Regex::new(
                r"^((([0-9]{1,3}):([0-5][0-9])|([0-9]{1,3})):([0-5][0-9]))$",
            )?,
        })
    }

    /// Parses a time suffix returning the number of milliseconds.
    pub fn suffix_syntax(&self, data: &str) -> Option<u32> {
        let captures = self.suffix_parser.captures(data)?;

        if let Some(seconds) = captures.get(2) {
            // `00s` syntax.
            let seconds = seconds.as_str().parse::<u32>().ok()?;

            Some(seconds * 1000)
        } else if let Some(minutes) = captures.get(3) {
            // `00m` syntax.
            let minutes = minutes.as_str().parse::<u32>().ok()?;

            Some(minutes * 60 * 1000)
        } else if let Some(hours) = captures.get(4) {
            // `00h` syntax.
            let hours = hours.as_str().parse::<u32>().ok()?;

            Some(hours * 60 * 60 * 1000)
        } else {
            None
        }
    }

    /// Parses a time semicolon syntax returning the number of milliseconds.
    pub fn semicolon_syntax(&self, data: &str) -> Option<u32> {
        let captures = self.semicolon_parser.captures(data)?;

        let hours_minutes = match captures.get(3) {
            Some(x) => {
                // `00:00:00` syntax.
                let hours = x.as_str().parse::<u32>().ok()?;
                let minutes = captures.get(4)?.as_str().parse::<u32>().ok()?;

                (hours * 60 * 60 * 1000) + (minutes * 60 * 1000)
            }
            None => {
                // `00:00` syntax.
                let minutes = captures.get(5)?.as_str().parse::<u32>().ok()?;

                minutes * 60 * 1000
            }
        };

        let seconds = captures.get(6)?.as_str().parse::<u32>().ok()?;

        Some(hours_minutes + (seconds * 1000))
    }
}

/// Holds the parser for the roll syntax.
pub struct RollParser {
    /// Regex parser for the roll syntax.
    roll_parser: Regex,

    /// Regex parser for the modifier syntax.
    modifier_parser: Regex,
}

impl RollParser {
    /// Creates a new instance of the roll parser.
    pub fn new() -> Result<Self, regex::Error> {
        Ok(Self {
            roll_parser: Regex::new(
                r"(?: |^)(?:(10|[1-9])#)?(50|0?[1-9]|[1-4][0-9])?d(100|[fF]|0?[2-9]|[1-9][0-9])((?:[+\-*\/][0-9]{1,3}|){0,3})(?:$| )",
            )?,
            modifier_parser: Regex::new(r"([+\-*\/])([0-9]{1,3})")?,
        })
    }

    /// Parses the modifier syntax, returning the modifier.
    pub fn evaluate_modifier(&self, data: &str) -> Option<Modifier> {
        self.modifier_parser
            .captures_iter(data)
            .map(|x| {
                (
                    x.get(1).unwrap().as_str(),
                    x.get(2).unwrap().as_str().parse::<i32>().unwrap(),
                )
            })
            .map(|(op, value)| match op {
                "+" => Modifier::Add(value),
                "-" => Modifier::Subtract(value),
                "*" => Modifier::Multiply(value),
                "/" => Modifier::Divide(value),
                _ => unreachable!(),
            })
            .reduce(|acc, x| acc.unify(x))
    }

    /// Evaluates the roll syntax, returning the parameters.
    pub fn evaluate(&self, data: &str) -> Option<Params> {
        // Default roll parameters.
        let mut params = Params::default();

        // Parse the roll syntax.
        let captures = self.roll_parser.captures(data)?;

        // If repeat is present, parse it.
        if let Some(repeat) = captures.get(1).map(|x| x.as_str().parse::<u8>().unwrap()) {
            params.repeat = repeat;
        }

        // If dice count is present, parse it.
        if let Some(dice_count) = captures.get(2).map(|x| x.as_str().parse::<u8>().unwrap()) {
            params.dice_count = dice_count;
        }

        // If dice sides is present, parse it.
        if let Some(dice_sides) = captures.get(3).map(|x| x.as_str()) {
            if dice_sides.to_lowercase() == "f" {
                params.dice_type = DiceType::Fate;
            } else {
                params.dice_type = DiceType::Sided(dice_sides.parse::<u8>().unwrap());
            }
        }

        // If modifier is present, parse it.
        if let Some(modifier) = captures
            .get(4)
            .and_then(|x| self.evaluate_modifier(x.as_str()))
        {
            params.modifier = modifier;
        }

        Some(params)
    }
}
