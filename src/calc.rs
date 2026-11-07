//! Calculator domain: pure, synchronous maths rules (ADR-0003, ADR-0004).
//! No HTTP, JSON or logging in here.

use std::fmt;

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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn errors_map_to_prd_codes() {
        assert_eq!(CalcError::InvalidOperand("x").code(), "INVALID_INPUT");
        assert_eq!(CalcError::DivisionByZero.code(), "DIVISION_BY_ZERO");
        assert_eq!(CalcError::UndefinedResult("x").code(), "UNDEFINED_RESULT");
        assert_eq!(CalcError::ResultOutOfRange.code(), "RESULT_OUT_OF_RANGE");
    }
}
