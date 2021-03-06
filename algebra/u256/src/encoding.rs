use crate::U256;
use std::prelude::v1::*;

#[cfg(feature = "std")]
use std::{fmt, format};

impl U256 {
    // Can not use Self inside the macro
    #[allow(clippy::use_self)]
    pub fn from_decimal_str(s: &str) -> Result<Self, ParseError> {
        // ceil(2^256 / 10)
        let max10: Self = Self::from_limbs([
            0x9999_9999_9999_999a_u64,
            0x9999_9999_9999_9999_u64,
            0x9999_9999_9999_9999_u64,
            0x1999_9999_9999_9999_u64,
        ]);
        if s.is_empty() {
            return Err(ParseError::Empty);
        }
        // TODO: Support other radices
        // TODO: Implement as trait
        // OPT: Convert 19 digits at a time using u64.
        let mut result = Self::ZERO;
        for (i, _c) in s.chars().enumerate() {
            if result > max10 {
                return Err(ParseError::Overflow);
            }
            result *= Self::from(10_u64);
            let digit = Self::from(u64::from_str_radix(&s[i..=i], 10)?);
            if &result + &digit < result {
                return Err(ParseError::Overflow);
            }
            result += digit;
        }
        Ok(result)
    }

    pub fn to_decimal_str(&self) -> String {
        if *self == Self::ZERO {
            return "0".to_string();
        }
        let mut result = String::new();
        let mut copy = self.clone();
        while copy > Self::ZERO {
            // OPT: Convert 19 digits at a time using u64.
            let digit = (&copy % Self::from(10_u64)).limb(0);
            result.push_str(&digit.to_string());
            copy /= Self::from(10_u64);
        }
        // Reverse digits
        // Note: Chars are safe here instead of graphemes, because all graphemes
        // are a single codepoint.
        result.chars().rev().collect()
    }

    #[cfg(feature = "std")]
    pub fn from_hex_str(s: &str) -> Self {
        let byte_string = format!("{:0>64}", s.trim_start_matches("0x"));
        let bytes = hex::decode(byte_string).unwrap();
        let mut array = [0_u8; 32];
        array.copy_from_slice(&bytes[..32]);
        Self::from_bytes_be(&array)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    Empty,
    Overflow,
    InnerError(core::num::ParseIntError),
}

impl From<core::num::ParseIntError> for ParseError {
    fn from(error: core::num::ParseIntError) -> Self {
        Self::InnerError(error)
    }
}

#[cfg(feature = "std")]
impl fmt::Display for U256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:016x}{:016x}{:016x}{:016x}",
            self.limb(3),
            self.limb(2),
            self.limb(1),
            self.limb(0)
        )
    }
}

#[cfg(feature = "std")]
impl fmt::Debug for U256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "u256h!(\"{:016x}{:016x}{:016x}{:016x}\")",
            self.limb(3),
            self.limb(2),
            self.limb(1),
            self.limb(0)
        )
    }
}

// TODO: Replace literals with u256h!
#[allow(clippy::unreadable_literal)]
// Quickcheck requires pass by value
#[allow(clippy::needless_pass_by_value)]
#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck_macros::quickcheck;
    use zkp_macros_decl::u256h;

    #[test]
    fn test_from_decimal_str() {
        assert_eq!(U256::from_decimal_str(""), Err(ParseError::Empty));
        assert_eq!(U256::from_decimal_str("0"), Ok(U256::ZERO));
        assert_eq!(U256::from_decimal_str("00"), Ok(U256::ZERO));
        assert_eq!(U256::from_decimal_str("000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"), Ok(U256::ZERO));
        assert_eq!(U256::from_decimal_str("1"), Ok(U256::ONE));
        assert_eq!(
            U256::from_decimal_str(
                "115792089237316195423570985008687907853269984665640564039457584007913129639935"
            ),
            Ok(u256h!(
                "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
            ))
        );
        assert_eq!(
            U256::from_decimal_str(
                "115792089237316195423570985008687907853269984665640564039457584007913129639936"
            ),
            Err(ParseError::Overflow)
        );
        assert_eq!(
            U256::from_decimal_str(
                "1000000000000000000000000000000000000000000000000000000000000000000000000000000"
            ),
            Err(ParseError::Overflow)
        );
        assert!(U256::from_decimal_str("12a3").is_err());
    }

    #[quickcheck]
    fn test_decimal_to_from(n: U256) -> bool {
        let decimal = n.clone().to_decimal_str();
        let m = U256::from_decimal_str(&decimal).unwrap();
        n == m
    }
}
