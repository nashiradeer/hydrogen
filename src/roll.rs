//! Hydrogen // Roll
//!
//! This module provides all the backend functionality for the roll command.

use std::{
    fmt::{self, Display, Formatter},
    ops::RangeInclusive,
    result,
};

use rand::{thread_rng, Rng};

/// Errors that can occur when preparing a roll.
#[derive(Debug)]
pub enum Error {
    /// The number of dice is invalid.
    InvalidDiceCount(u8, RangeInclusive<u8>),

    /// The number of sides is invalid.
    InvalidDiceSides(u8, RangeInclusive<u8>),

    /// The number of repetitions is invalid.
    InvalidRepetition(u8, RangeInclusive<u8>),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::InvalidDiceCount(count, range) => {
                write!(f, "Invalid dice count: {}. Expected: {:?}", count, range)
            }
            Error::InvalidDiceSides(sides, range) => {
                write!(f, "Invalid dice sides: {}. Expected: {:?}", sides, range)
            }
            Error::InvalidRepetition(repeat, range) => {
                write!(
                    f,
                    "Invalid repetition count: {}. Expected: {:?}",
                    repeat, range
                )
            }
        }
    }
}

/// A result type for the roll module.
pub type Result<T> = result::Result<T, Error>;

/// Represents the different types of dice that can be rolled.
#[derive(Debug, Clone)]
pub enum Dice {
    /// Fate dice, which can be either -, 0, or +.
    Fate,

    /// A standard dice with a number of sides.
    Sided(u8),
}

/// Represents the different types of modifiers that can be applied to a roll.
#[derive(Debug, Clone)]
pub enum Modifier {
    /// Adds a value to the roll.
    Add(i32),

    /// Subtracts a value from the roll.
    Subtract(i32),

    /// Multiplies the roll by a value.
    Multiply(i32),

    /// Divides the roll by a value.
    Divide(i32),
}

impl Modifier {
    /// Applies the modifier to the given value.
    pub fn apply(&self, value: i32) -> i32 {
        match self {
            Self::Add(v) => value + v,
            Self::Subtract(v) => value - v,
            Self::Multiply(v) => value * v,
            Self::Divide(v) => value / v,
        }
    }

    /// Unifies two modifiers applying the second to the first.
    pub fn unify(self, other: Modifier) -> Modifier {
        match self {
            Self::Add(me) => Self::Add(other.apply(me)),
            Self::Subtract(me) => Self::Subtract(other.apply(me)),
            Self::Multiply(me) => Self::Multiply(other.apply(me)),
            Self::Divide(me) => Self::Divide(other.apply(me)),
        }
    }
}

/// The parameters for a roll.
#[derive(Debug, Clone)]
pub struct Params {
    /// The number of dice to roll.
    pub dice_count: u8,

    /// The type of dice to roll.
    pub dice: Dice,

    /// The modifier to add to the roll.
    pub modifier: Modifier,

    /// The number of times to repeat the roll.
    pub repeat: u8,
}

impl Params {
    /// Creates a new set of roll parameters.
    pub fn new(dice_count: u8, dice: Dice, modifier: Modifier, repeat: u8) -> Self {
        Self {
            dice_count,
            dice,
            modifier,
            repeat,
        }
    }

    /// Validates the parameters.
    pub fn validate(&self) -> Result<()> {
        // Check for the dice count.
        let dice_count_range = 0..=50;
        if !dice_count_range.contains(&self.dice_count) {
            return Err(Error::InvalidDiceCount(self.dice_count, dice_count_range));
        }

        // Check for the dice sides.
        if let Dice::Sided(sides) = self.dice {
            let dice_sides_range = 1..=100;
            if !dice_sides_range.contains(&sides) {
                return Err(Error::InvalidDiceSides(sides, dice_sides_range));
            }
        }

        // Check for the repetition count.
        let repetition_range = 1..=50;
        if !repetition_range.contains(&self.repeat) {
            return Err(Error::InvalidRepetition(self.repeat, repetition_range));
        }

        Ok(())
    }

    /// Rolls the dice with the given parameters.
    pub fn roll(&self) -> Result<Roll> {
        self.validate()?;
        let mut rng = thread_rng();

        match self.dice {
            Dice::Fate => {
                let mut rolls = Vec::new();

                for _ in 0..self.repeat {
                    let mut roll = Vec::new();

                    for _ in 0..self.dice_count {
                        roll.push(rng.gen_range(-1..=1));
                    }

                    rolls.push(roll);
                }

                Ok(Roll::Fate(rolls, self.modifier.clone()))
            }

            Dice::Sided(sides) => {
                let mut rolls = Vec::new();

                for _ in 0..self.repeat {
                    let mut roll = Vec::new();

                    for _ in 0..self.dice_count {
                        roll.push(rng.gen_range(1..=sides));
                    }

                    rolls.push(roll);
                }

                Ok(Roll::Sided(rolls, self.modifier.clone()))
            }
        }
    }
}

impl Default for Params {
    fn default() -> Self {
        Self::new(1, Dice::Sided(6), Modifier::Add(0), 1)
    }
}

/// Results of a roll.
pub enum Roll {
    /// Results of a roll of fate dice.
    Fate(Vec<Vec<i8>>, Modifier),

    /// Results of a roll of standard dice.
    Sided(Vec<Vec<u8>>, Modifier),
}

impl ToString for Roll {
    fn to_string(&self) -> String {
        match self {
            Self::Fate(rolls, modifier) => {
                let mut result = String::new();
                for roll in rolls {
                    let total = roll.iter().cloned().map(|v| i32::from(v)).sum::<i32>();

                    result.push_str(&format!(
                        "[{}]: {} = {}\n",
                        roll.iter()
                            .map(|r| match r {
                                -1 => "-",
                                0 => "0",
                                1 => "+",
                                _ => unreachable!(),
                            })
                            .collect::<Vec<_>>()
                            .join(", "),
                        total,
                        modifier.apply(total)
                    ));
                }
                result
            }

            Self::Sided(rolls, modifier) => {
                let mut result = String::new();
                for roll in rolls {
                    let total = roll.iter().cloned().map(|v| i32::from(v)).sum::<i32>();

                    result.push_str(&format!(
                        "[{}]: {} = {}\n",
                        roll.iter()
                            .map(|r| r.to_string())
                            .collect::<Vec<_>>()
                            .join(", "),
                        total,
                        modifier.apply(total)
                    ));
                }
                result
            }
        }
    }
}
