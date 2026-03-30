//! String parsing utilities.

use std::num::ParseIntError;
use std::str::FromStr;

/// Extension trait for parsing strings.
pub trait ParseExt {
    /// Parse as i8.
    fn parse_i8(&self) -> Result<i8, ParseIntError>;

    /// Parse as i16.
    fn parse_i16(&self) -> Result<i16, ParseIntError>;

    /// Parse as i32.
    fn parse_i32(&self) -> Result<i32, ParseIntError>;

    /// Parse as i64.
    fn parse_i64(&self) -> Result<i64, ParseIntError>;

    /// Parse as u8.
    fn parse_u8(&self) -> Result<u8, ParseIntError>;

    /// Parse as u16.
    fn parse_u16(&self) -> Result<u16, ParseIntError>;

    /// Parse as u32.
    fn parse_u32(&self) -> Result<u32, ParseIntError>;

    /// Parse as u64.
    fn parse_u64(&self) -> Result<u64, ParseIntError>;

    /// Parse as f32.
    fn parse_f32(&self) -> Result<f32, std::num::ParseFloatError>;

    /// Parse as f64.
    fn parse_f64(&self) -> Result<f64, std::num::ParseFloatError>;

    /// Parse as bool (accepts "true", "false", "1", "0").
    fn parse_bool(&self) -> Result<bool, ParseBoolError>;

    /// Parse using FromStr.
    fn parse<T: FromStr>(&self) -> Result<T, T::Err>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseBoolError {
    pub value: String,
}

impl std::fmt::Display for ParseBoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid boolean value: {}", self.value)
    }
}

impl std::error::Error for ParseBoolError {}

impl<T: AsRef<str>> ParseExt for T {
    fn parse_i8(&self) -> Result<i8, ParseIntError> {
        self.as_ref().parse()
    }

    fn parse_i16(&self) -> Result<i16, ParseIntError> {
        self.as_ref().parse()
    }

    fn parse_i32(&self) -> Result<i32, ParseIntError> {
        self.as_ref().parse()
    }

    fn parse_i64(&self) -> Result<i64, ParseIntError> {
        self.as_ref().parse()
    }

    fn parse_u8(&self) -> Result<u8, ParseIntError> {
        self.as_ref().parse()
    }

    fn parse_u16(&self) -> Result<u16, ParseIntError> {
        self.as_ref().parse()
    }

    fn parse_u32(&self) -> Result<u32, ParseIntError> {
        self.as_ref().parse()
    }

    fn parse_u64(&self) -> Result<u64, ParseIntError> {
        self.as_ref().parse()
    }

    fn parse_f32(&self) -> Result<f32, std::num::ParseFloatError> {
        self.as_ref().parse()
    }

    fn parse_f64(&self) -> Result<f64, std::num::ParseFloatError> {
        self.as_ref().parse()
    }

    fn parse_bool(&self) -> Result<bool, ParseBoolError> {
        match self.as_ref().to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Ok(true),
            "false" | "0" | "no" | "off" => Ok(false),
            _ => Err(ParseBoolError {
                value: self.as_ref().to_string(),
            }),
        }
    }

    fn parse<T: FromStr>(&self) -> Result<T, T::Err> {
        self.as_ref().parse()
    }
}
