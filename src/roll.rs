//! Hydrogen // Roll
//!
//! This module provides all the backend functionality for the roll command.

use std::{
    error,
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

impl error::Error for Error {}

/// A result type for the roll module.
pub type Result<T> = result::Result<T, Error>;

/// Represents the different types of dice that can be rolled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiceType {
    /// Fate dice, which can be either -, 0, or +, see [FateDice].
    Fate,

    /// A standard dice with a number of sides.
    Sided(u8),
}

/// Represents the different types of modifiers that can be applied to a roll.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pub dice_type: DiceType,

    /// The modifier to add to the roll.
    pub modifier: Modifier,

    /// The number of times to repeat the roll.
    pub repeat: u8,
}

impl Params {
    /// Creates a new set of roll parameters.
    pub fn new(dice_count: u8, dice_type: DiceType, modifier: Modifier, repeat: u8) -> Self {
        Self {
            dice_count,
            dice_type,
            modifier,
            repeat,
        }
    }

    /// Validates the parameters.
    pub fn validate(&self) -> Result<()> {
        // Check for the dice count.
        let dice_count_range = 1..=50;
        if !dice_count_range.contains(&self.dice_count) {
            return Err(Error::InvalidDiceCount(self.dice_count, dice_count_range));
        }

        // Check for the dice sides.
        if let DiceType::Sided(sides) = self.dice_type {
            let dice_sides_range = 2..=100;
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
        // Validate the parameters.
        self.validate()?;

        // Create a random number generator.
        let mut rng = thread_rng();

        // Create a vector to store the rolls.
        let mut rolls = Vec::new();

        // Generate the rolls for the given number of repetitions.
        for _ in 0..self.repeat {
            // Create a vector to store the repetition.
            let mut roll = Vec::new();

            // Generate the rolls for the given number of dice.
            for _ in 0..self.dice_count {
                // Generate a random number for the dice.
                let random = match self.dice_type {
                    DiceType::Fate => {
                        Dice::Fate(FateDice::try_from(rng.gen_range(-1..=1)).unwrap())
                    }
                    DiceType::Sided(sides) => Dice::Sided(rng.gen_range(1..=sides)),
                };

                // Add the roll to the repetition.
                roll.push(random);
            }

            // Add the repetition to the rolls.
            rolls.push(roll);
        }

        Ok(Roll(rolls, self.modifier))
    }
}

impl Default for Params {
    fn default() -> Self {
        Self::new(1, DiceType::Sided(6), Modifier::Add(0), 1)
    }
}

/// Represents different types of dice results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dice {
    /// The result of a roll of fate dice.
    Fate(FateDice),

    /// The result of a roll of standard dice.
    Sided(u8),
}

impl From<Dice> for i32 {
    fn from(dice: Dice) -> i32 {
        match dice {
            Dice::Fate(fate) => i32::from(i8::from(fate)),
            Dice::Sided(value) => i32::from(value),
        }
    }
}

impl ToString for Dice {
    fn to_string(&self) -> String {
        match self {
            Self::Fate(fate) => fate.to_string(),
            Self::Sided(value) => value.to_string(),
        }
    }
}

/// Results of a roll.
#[derive(Debug, Clone)]
pub struct Roll(Vec<Vec<Dice>>, Modifier);

impl ToString for Roll {
    fn to_string(&self) -> String {
        // Create a string to store the result.
        let mut result = String::new();

        // Iterate over the repetitions.
        for roll in self.0.iter() {
            // Calculate the total of the roll in the repetition.
            let total = roll.iter().cloned().map(|v| i32::from(v)).sum();

            // Add the result to the string, including the total with the modifier applied.
            result.push_str(&format!(
                "[{}]: {} = {}\n",
                roll.iter()
                    .map(|r| r.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
                total,
                self.1.apply(total)
            ));
        }

        result
    }
}

/// Represents a fate dice with its possible values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FateDice {
    /// Negative (-).
    Minus,

    /// Neutral (0).
    Zero,

    /// Positive (+).
    Plus,
}

impl FateDice {
    /// Converts a number to a fate dice.
    pub fn try_from(i: i8) -> Option<Self> {
        match i {
            -1 => Some(Self::Minus),
            0 => Some(Self::Zero),
            1 => Some(Self::Plus),
            _ => None,
        }
    }
}

impl ToString for FateDice {
    fn to_string(&self) -> String {
        match self {
            Self::Minus => "-".to_string(),
            Self::Zero => "0".to_string(),
            Self::Plus => "+".to_string(),
        }
    }
}

impl From<FateDice> for i8 {
    fn from(fate: FateDice) -> i8 {
        match fate {
            FateDice::Minus => -1,
            FateDice::Zero => 0,
            FateDice::Plus => 1,
        }
    }
}
