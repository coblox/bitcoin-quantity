extern crate bigdecimal;
#[cfg(feature = "serde")]
extern crate serde;

use bigdecimal::ParseBigDecimalError;
#[cfg(feature = "serde")]
use serde::{
    de::{self, Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};
use std::{
    fmt,
    ops::{Add, Sub},
    str::FromStr,
};

#[derive(PartialEq, Clone, Debug, Copy, PartialOrd)]
pub struct BitcoinQuantity(u64);

impl BitcoinQuantity {
    pub fn from_satoshi(sats: u64) -> Self {
        BitcoinQuantity(sats)
    }
    pub fn from_bitcoin(btc: f64) -> Self {
        BitcoinQuantity((btc * 100_000_000.0).round() as u64)
    }
    pub fn satoshi(self) -> u64 {
        self.0
    }
    pub fn bitcoin(self) -> f64 {
        (self.0 as f64) / 100_000_000.0
    }
}

impl Add for BitcoinQuantity {
    type Output = BitcoinQuantity;

    fn add(self, rhs: BitcoinQuantity) -> BitcoinQuantity {
        BitcoinQuantity(self.0 + rhs.0)
    }
}

impl Sub for BitcoinQuantity {
    type Output = BitcoinQuantity;

    fn sub(self, rhs: BitcoinQuantity) -> BitcoinQuantity {
        BitcoinQuantity(self.0 - rhs.0)
    }
}

impl fmt::Display for BitcoinQuantity {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{} BTC", self.bitcoin())
    }
}

impl FromStr for BitcoinQuantity {
    type Err = ParseBigDecimalError;

    fn from_str(string: &str) -> Result<BitcoinQuantity, Self::Err> {
        let dec = string.parse()?;
        Ok(Self::from_bitcoin(dec))
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for BitcoinQuantity {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl<'vde> de::Visitor<'vde> for Visitor {
            type Value = BitcoinQuantity;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
                formatter.write_str("A string representing a satoshi quantity")
            }

            fn visit_str<E>(self, v: &str) -> Result<BitcoinQuantity, E>
            where
                E: de::Error,
            {
                Ok(v.parse()
                    .map(BitcoinQuantity::from_satoshi)
                    .map_err(E::custom)?)
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

#[cfg(feature = "serde")]
impl Serialize for BitcoinQuantity {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.0.to_string().as_str())
    }
}

#[cfg(test)]
mod tests {
    extern crate serde_json;
    extern crate spectral;

    use super::*;
    use tests::spectral::prelude::*;

    #[test]
    fn hundred_million_sats_is_a_bitcoin() {
        assert_that(&BitcoinQuantity::from_satoshi(100_000_000).bitcoin()).is_equal_to(&1.0);
    }

    #[test]
    fn a_bitcoin_is_a_hundred_million_sats() {
        assert_that(&BitcoinQuantity::from_bitcoin(1.0).satoshi()).is_equal_to(&100_000_000);
    }
    #[test]
    fn a_bitcoin_as_string_is_a_hundred_million_sats() {
        assert_that(&BitcoinQuantity::from_str("1.00000001").unwrap())
            .is_equal_to(&BitcoinQuantity::from_bitcoin(1.000_000_01));
    }

    #[test]
    fn bitcoin_with_small_fraction_format() {
        assert_eq!(
            format!("{}", BitcoinQuantity::from_str("1234.00000100").unwrap()),
            "1234.000001 BTC"
        )
    }

    #[test]
    fn one_hundred_bitcoin_format() {
        assert_eq!(
            format!("{}", BitcoinQuantity::from_str("100").unwrap()),
            "100 BTC"
        )
    }

    #[test]
    fn display_bitcoin() {
        assert_eq!(format!("{}", BitcoinQuantity::from_bitcoin(42.0)), "42 BTC");
        assert_eq!(
            format!("{}", BitcoinQuantity::from_satoshi(200_000_000)),
            "2 BTC"
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serialize_bitcoin_quantity() {
        let quantity = BitcoinQuantity::from_satoshi(100_000_000);
        assert_eq!(serde_json::to_string(&quantity).unwrap(), "\"100000000\"");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize_bitcoin_quantity() {
        let quantity = serde_json::from_str::<BitcoinQuantity>("\"100000000\"").unwrap();
        assert_eq!(quantity, BitcoinQuantity::from_satoshi(100_000_000))
    }

    #[test]
    fn bitcoin_with_more_than_seven_decimal_places_is_truncated() {
        assert_that(&BitcoinQuantity::from_bitcoin(0.000000495).satoshi()).is_equal_to(&50);
    }
}
