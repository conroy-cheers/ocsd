use std::{error::Error, fmt::Display};

/// Error used when a constructed temperature value does not fit
/// into the OCSD representation's range.
#[derive(Debug, Clone, Copy)]
pub struct TempOutOfRange;

impl Display for TempOutOfRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "provided temperature does not fit in range".fmt(f)
    }
}

impl Error for TempOutOfRange {}

/// Represents a signed integer temperature in degrees Celsius,
/// stored as a single-byte raw value.
#[derive(Default)]
pub struct Celsius {
    value: i8,
}

impl Celsius {
    const OFFSET: i8 = 0;

    /// Constructs a new Celsius value.
    ///
    /// Returns a Result of the constructed value, or TempOutOfRange
    /// when the passed value cannot fit into the raw value field.
    ///
    /// # Examples
    /// ```
    /// # use std::error::Error;
    /// # use ocsd::protocol::temperature::TempOutOfRange;
    /// use ocsd::protocol::temperature::Celsius;
    ///
    /// # fn main() -> Result<(), TempOutOfRange> {
    /// let temperature = Celsius::new(30)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(degrees: i16) -> Result<Self, TempOutOfRange> {
        let value = degrees + Self::OFFSET as i16;
        match value.try_into() {
            Ok(value) => Ok(Self { value }),
            Err(_) => Err(TempOutOfRange),
        }
    }

    /// Returns the temperature as raw OCSD representation.
    pub fn raw_value(&self) -> u8 {
        self.value as u8
    }

    /// Returns the temperature as degrees.
    ///
    /// # Examples
    /// ```
    /// # use std::error::Error;
    /// # use ocsd::protocol::temperature::TempOutOfRange;
    /// use ocsd::protocol::temperature::Celsius;
    ///
    /// # fn main() -> Result<(), TempOutOfRange> {
    /// let temperature = Celsius::new(30)?;
    /// assert_eq!(temperature.degrees(), 30);
    /// # Ok(())
    /// # }
    /// ```
    pub fn degrees(&self) -> i16 {
        self.value as i16 - Self::OFFSET as i16
    }

    /// Constructs a new Celsius value from the raw OCSD representation.
    ///
    /// # Examples
    /// ```
    /// # use std::error::Error;
    /// # use ocsd::protocol::temperature::TempOutOfRange;
    /// use ocsd::protocol::temperature::Celsius;
    ///
    /// # fn main() -> Result<(), TempOutOfRange> {
    /// let temperature = Celsius::from_raw(50);
    /// assert_eq!(temperature.raw_value(), 50);
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_raw(value: u8) -> Self {
        Self { value: value as i8 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temperature() {
        assert_eq!(Celsius::new(0).unwrap().raw_value(), 0);
        assert_eq!(Celsius::new(40).unwrap().raw_value(), 40);
        assert_eq!(Celsius::from_raw(40).value, Celsius::new(40).unwrap().value);
        assert_eq!(Celsius::new(-1).unwrap().raw_value(), 255);
    }
}
