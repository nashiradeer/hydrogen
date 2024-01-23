//! Hydrogen // Parsers
//!
//! Contains the parsers used by Hydrogen.

use regex::Regex;

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
            suffix_parser: Regex::new(r"^((\d{1,3})[sS]?|(\d{1,3})[mM]|(\d{1,3})[hH])$")?,
            semicolon_parser: Regex::new(r"^(((\d{1,3}):([0-5]\d)|(\d{1,3})):([0-5]\d))$")?,
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
