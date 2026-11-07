//! Calculator domain: pure, synchronous maths rules (ADR-0003, ADR-0004).
//! No HTTP, JSON or logging in here.

use std::fmt;

pub use rust_decimal::Decimal;

/// Exclusive magnitude limit for operands and results (NFR-3, FR-10): 10^28.
const LIMIT: Decimal = Decimal::from_parts(0x1000_0000, 0x3E25_0261, 0x204F_CE5E, false, 0);

/// The operations in PRD scope (FR-1 – FR-8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    Sqrt,
    Modulo,
    Percentage,
}

impl Operation {
    /// The wire name from ADR-0002, e.g. `"divide"`.
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "add" => Self::Add,
            "subtract" => Self::Subtract,
            "multiply" => Self::Multiply,
            "divide" => Self::Divide,
            "power" => Self::Power,
            "sqrt" => Self::Sqrt,
            "modulo" => Self::Modulo,
            "percentage" => Self::Percentage,
            _ => return None,
        })
    }

    /// Number of operands the operation takes.
    pub fn arity(self) -> usize {
        if self == Self::Sqrt { 1 } else { 2 }
    }
}

/// Why a calculation failed. Each variant maps to one PRD error code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalcError {
    /// The operand is not an acceptable number (FR-9).
    InvalidOperand(&'static str),
    /// Divide or modulo by zero (FR-4, FR-7).
    DivisionByZero,
    /// Not defined in real numbers (FR-5, FR-6).
    UndefinedResult(&'static str),
    /// Result magnitude is 10^28 or more (FR-10).
    ResultOutOfRange,
}

impl CalcError {
    /// The stable machine-readable code from PRD section 6.
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidOperand(_) => "INVALID_INPUT",
            Self::DivisionByZero => "DIVISION_BY_ZERO",
            Self::UndefinedResult(_) => "UNDEFINED_RESULT",
            Self::ResultOutOfRange => "RESULT_OUT_OF_RANGE",
        }
    }
}

impl fmt::Display for CalcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOperand(why) | Self::UndefinedResult(why) => f.write_str(why),
            Self::DivisionByZero => f.write_str("division by zero"),
            Self::ResultOutOfRange => f.write_str("result magnitude must be below 10^28"),
        }
    }
}

/// Read an operand from its decimal text, exactly (ADR-0003 rule 1).
///
/// Accepts plain (`-2.5`) and scientific (`2.5e-3`) notation. Rejects
/// anything else, more than 28 decimal places, and magnitude ≥ 10^28.
pub fn parse_operand(text: &str) -> Result<Decimal, CalcError> {
    // The library also accepts `_` separators; JSON numbers never contain them.
    if text.is_empty() || !text.bytes().all(|c| b"0123456789.-+eE".contains(&c)) {
        return Err(CalcError::InvalidOperand("not a decimal number"));
    }
    let parsed = if text.contains(['e', 'E']) {
        Decimal::from_scientific(text)
    } else {
        Decimal::from_str_exact(text)
    };
    let value = parsed.map_err(|_| {
        CalcError::InvalidOperand(
            "must be a decimal with at most 28 decimal places and magnitude below 10^28",
        )
    })?;
    if value.abs() >= LIMIT {
        return Err(CalcError::InvalidOperand("magnitude must be below 10^28"));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dec(s: &str) -> Decimal {
        s.parse().unwrap()
    }

    // NFR-3: decimal text is read exactly, never through a binary float.
    #[test]
    fn parses_exact_decimals_and_scientific_notation() {
        assert_eq!(parse_operand("0.1").unwrap(), dec("0.1"));
        assert_eq!(parse_operand("-2.50").unwrap(), dec("-2.5"));
        assert_eq!(parse_operand("1e3").unwrap(), dec("1000"));
        assert_eq!(parse_operand("2.5E-3").unwrap(), dec("0.0025"));
        assert_eq!(
            parse_operand("9999999999999999999999999999").unwrap(),
            dec("9999999999999999999999999999")
        );
        assert_eq!(
            parse_operand("0.0000000000000000000000000001").unwrap(),
            dec("0.0000000000000000000000000001")
        );
    }

    // FR-9: anything that is not an in-range decimal is INVALID_INPUT.
    #[test]
    fn rejects_non_numbers_and_out_of_range_operands() {
        for bad in [
            "",
            " 1",
            "abc",
            "1_000",
            "0x10",
            "NaN",
            "inf",
            "1,5",
            "1e400",
            "1e-40",
            "1e28",
            "-1e28",
            "10000000000000000000000000000",
            "0.00000000000000000000000000001",
        ] {
            let err = parse_operand(bad).unwrap_err();
            assert_eq!(err.code(), "INVALID_INPUT", "{bad:?}");
        }
    }

    // FR-9: every PRD operation name parses, anything else does not.
    #[test]
    fn operation_names_parse() {
        let names = [
            ("add", Operation::Add),
            ("subtract", Operation::Subtract),
            ("multiply", Operation::Multiply),
            ("divide", Operation::Divide),
            ("power", Operation::Power),
            ("sqrt", Operation::Sqrt),
            ("modulo", Operation::Modulo),
            ("percentage", Operation::Percentage),
        ];
        for (name, op) in names {
            assert_eq!(Operation::from_name(name), Some(op), "{name}");
        }
        for bad in ["", "Add", "plus", "sqrt ", "evaluate"] {
            assert_eq!(Operation::from_name(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn sqrt_takes_one_operand_others_two() {
        assert_eq!(Operation::Sqrt.arity(), 1);
        for op in [
            Operation::Add,
            Operation::Subtract,
            Operation::Multiply,
            Operation::Divide,
            Operation::Power,
            Operation::Modulo,
            Operation::Percentage,
        ] {
            assert_eq!(op.arity(), 2, "{op:?}");
        }
    }

    // PRD section 6: the four stable error codes.
    #[test]
    fn limit_is_ten_to_the_28() {
        assert_eq!(LIMIT, dec("10000000000000000000000000000"));
    }

    #[test]
    fn errors_map_to_prd_codes() {
        assert_eq!(CalcError::InvalidOperand("x").code(), "INVALID_INPUT");
        assert_eq!(CalcError::DivisionByZero.code(), "DIVISION_BY_ZERO");
        assert_eq!(CalcError::UndefinedResult("x").code(), "UNDEFINED_RESULT");
        assert_eq!(CalcError::ResultOutOfRange.code(), "RESULT_OUT_OF_RANGE");
    }
}
