//! Calculator domain: pure, synchronous maths rules (ADR-0003, ADR-0004).
//! No HTTP, JSON or logging in here.

use std::fmt;

pub use rust_decimal::Decimal;
use rust_decimal::{MathematicalOps, RoundingStrategy};

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

/// Apply `op` to `operands` under the PRD numeric rules (ADR-0003 rules 2–6).
pub fn evaluate(op: Operation, operands: &[Decimal]) -> Result<Decimal, CalcError> {
    let (a, b) = match (op.arity(), operands) {
        (1, [a]) => (*a, Decimal::ZERO),
        (2, [a, b]) => (*a, *b),
        _ => return Err(CalcError::InvalidOperand("wrong number of operands")),
    };
    let result = match op {
        Operation::Add => a.checked_add(b),
        Operation::Subtract => a.checked_sub(b),
        Operation::Multiply => a.checked_mul(b),
        Operation::Divide => {
            if b.is_zero() {
                return Err(CalcError::DivisionByZero);
            }
            a.checked_div(b)
        }
        Operation::Modulo => {
            if b.is_zero() {
                return Err(CalcError::DivisionByZero);
            }
            a.checked_rem(b)
        }
        // ADR-0003 rule 5: a x b can overflow when a% of b is still in range.
        Operation::Percentage => a
            .checked_mul(b)
            .and_then(|p| p.checked_div(Decimal::ONE_HUNDRED))
            .or_else(|| a.checked_div(Decimal::ONE_HUNDRED)?.checked_mul(b)),
        Operation::Power => return power(a, b),
        Operation::Sqrt => {
            if a.is_sign_negative() && !a.is_zero() {
                return Err(CalcError::UndefinedResult(
                    "square root of a negative number",
                ));
            }
            a.sqrt().map(round_approximate)
        }
    };
    finish(result)
}

/// `a ^ b` with the domain, rounding and underflow rules of ADR-0003.
fn power(a: Decimal, b: Decimal) -> Result<Decimal, CalcError> {
    let integer_exponent = b.fract().is_zero();
    if a.is_zero() && b.is_sign_negative() && !b.is_zero() {
        return Err(CalcError::UndefinedResult("zero to a negative power"));
    }
    if a.is_sign_negative() && !a.is_zero() && !integer_exponent {
        return Err(CalcError::UndefinedResult(
            "negative base with a fractional exponent is not a real number",
        ));
    }
    match a.checked_powd(b) {
        Some(r) if integer_exponent => finish(Some(r)),
        Some(r) => finish(Some(round_approximate(r))),
        // Rule 5: the library also fails on underflow. |a| > 1 with b < 0, or
        // |a| < 1 with b > 0, shrinks towards 0, below 28 decimal places.
        None if (a.abs() > Decimal::ONE) == b.is_sign_negative() => Ok(Decimal::ZERO),
        None => Err(CalcError::ResultOutOfRange),
    }
}

/// Round an approximate result (`√`, fractional power) to 20 significant
/// digits, half-to-even, keeping only digits known to be correct (ADR-0003 rule 4).
fn round_approximate(d: Decimal) -> Decimal {
    d.round_sf_with_strategy(20, RoundingStrategy::MidpointNearestEven)
        .unwrap_or(d)
}

/// Range check and normalise a computed result (ADR-0003 rules 2 and 6).
fn finish(result: Option<Decimal>) -> Result<Decimal, CalcError> {
    match result {
        Some(d) if d.abs() < LIMIT => Ok(normalise(d)),
        _ => Err(CalcError::ResultOutOfRange),
    }
}

/// Strip trailing zeros and turn `-0` into `0` (FR-13).
fn normalise(d: Decimal) -> Decimal {
    if d.is_zero() {
        Decimal::ZERO
    } else {
        d.normalize()
    }
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
    fn eval(op: Operation, operands: &[&str]) -> Result<String, CalcError> {
        let operands: Vec<Decimal> = operands.iter().map(|s| dec(s)).collect();
        evaluate(op, &operands).map(|d| d.to_string())
    }

    fn ok(op: Operation, operands: &[&str], expected: &str) {
        assert_eq!(
            eval(op, operands),
            Ok(expected.to_string()),
            "{op:?} {operands:?}"
        );
    }

    #[test]
    fn fr1_add() {
        ok(Operation::Add, &["2", "3"], "5");
        ok(Operation::Add, &["-1.5", "0.5"], "-1");
        ok(Operation::Add, &["0.1", "0.2"], "0.3");
    }

    #[test]
    fn fr2_subtract() {
        ok(Operation::Subtract, &["5", "3"], "2");
        ok(Operation::Subtract, &["3", "5"], "-2");
    }

    #[test]
    fn fr3_multiply() {
        ok(Operation::Multiply, &["4", "2.5"], "10");
        ok(Operation::Multiply, &["-3", "0"], "0");
    }

    #[test]
    fn fr4_divide() {
        ok(Operation::Divide, &["10", "4"], "2.5");
        ok(
            Operation::Divide,
            &["1", "3"],
            "0.3333333333333333333333333333",
        );
        ok(
            Operation::Divide,
            &["1", "300"],
            "0.0033333333333333333333333333",
        );
        ok(
            Operation::Divide,
            &["2", "3"],
            "0.6666666666666666666666666667",
        );
    }

    #[test]
    fn fr4_divide_by_zero() {
        assert_eq!(
            eval(Operation::Divide, &["1", "0"]),
            Err(CalcError::DivisionByZero)
        );
        assert_eq!(
            eval(Operation::Divide, &["0", "0"]),
            Err(CalcError::DivisionByZero)
        );
    }

    #[test]
    fn fr7_modulo_sign_follows_dividend() {
        ok(Operation::Modulo, &["10", "3"], "1");
        ok(Operation::Modulo, &["-10", "3"], "-1");
        ok(Operation::Modulo, &["10", "-3"], "1");
        ok(Operation::Modulo, &["5.5", "2"], "1.5");
        ok(Operation::Modulo, &["-9", "3"], "0");
        assert_eq!(
            eval(Operation::Modulo, &["1", "0"]),
            Err(CalcError::DivisionByZero)
        );
    }

    #[test]
    fn fr8_percentage() {
        ok(Operation::Percentage, &["20", "50"], "10");
        ok(Operation::Percentage, &["150", "40"], "60");
        ok(Operation::Percentage, &["-10", "0"], "0");
        // ADR-0003 rule 5: a x b overflows, but the result is in range.
        ok(
            Operation::Percentage,
            &["5000000000000000000000000000", "50"],
            "2500000000000000000000000000",
        );
        assert_eq!(
            eval(
                Operation::Percentage,
                &["9999999999999999999999999999", "200"]
            ),
            Err(CalcError::ResultOutOfRange)
        );
    }

    #[test]
    fn fr5_power() {
        ok(Operation::Power, &["2", "10"], "1024");
        ok(Operation::Power, &["4", "0.5"], "2");
        ok(Operation::Power, &["9", "0.5"], "3");
        ok(Operation::Power, &["2", "0.5"], "1.4142135623730950488");
        ok(Operation::Power, &["2", "-1"], "0.5");
        ok(Operation::Power, &["-2", "3"], "-8");
        ok(Operation::Power, &["-2", "-2"], "0.25");
        ok(Operation::Power, &["1.5", "2"], "2.25");
        ok(Operation::Power, &["0", "0"], "1"); // Q-6
        ok(Operation::Power, &["0", "5"], "0");
    }

    // ADR-0003 rule 5: tiny results round to 0 instead of overflowing.
    #[test]
    fn fr5_power_underflow_rounds_to_zero() {
        ok(Operation::Power, &["2", "-100"], "0");
        ok(Operation::Power, &["0.5", "200"], "0");
        ok(Operation::Power, &["-10", "-29"], "0");
    }

    #[test]
    fn fr5_power_errors() {
        let undefined = |a, b| eval(Operation::Power, &[a, b]).unwrap_err().code();
        assert_eq!(undefined("0", "-1"), "UNDEFINED_RESULT");
        assert_eq!(undefined("-8", "0.5"), "UNDEFINED_RESULT");
        assert_eq!(undefined("-1", "1.5"), "UNDEFINED_RESULT");
        assert_eq!(
            eval(Operation::Power, &["10", "30"]),
            Err(CalcError::ResultOutOfRange)
        );
        assert_eq!(
            eval(Operation::Power, &["0.5", "-200"]),
            Err(CalcError::ResultOutOfRange)
        );
        assert_eq!(
            eval(Operation::Power, &["10", "28"]),
            Err(CalcError::ResultOutOfRange)
        );
    }

    #[test]
    fn fr6_sqrt() {
        ok(Operation::Sqrt, &["9"], "3");
        ok(Operation::Sqrt, &["0.25"], "0.5");
        ok(Operation::Sqrt, &["2"], "1.4142135623730950488");
        ok(Operation::Sqrt, &["0"], "0");
        ok(
            Operation::Sqrt,
            &["9999999999999999999999999999"],
            "100000000000000",
        );
        assert_eq!(
            eval(Operation::Sqrt, &["-1"]).unwrap_err().code(),
            "UNDEFINED_RESULT"
        );
    }

    #[test]
    fn fr10_out_of_range() {
        let huge = "9999999999999999999999999999";
        for (op, operands) in [
            (
                Operation::Multiply,
                ["100000000000000000000", "10000000000"],
            ),
            (Operation::Add, [huge, "1"]),
            (Operation::Subtract, ["-9999999999999999999999999999", "1"]),
            (Operation::Divide, [huge, "0.1"]),
            (Operation::Multiply, [huge, huge]),
        ] {
            assert_eq!(
                eval(op, &operands),
                Err(CalcError::ResultOutOfRange),
                "{op:?}"
            );
        }
    }

    #[test]
    fn fr13_never_negative_zero_or_trailing_zeros() {
        ok(Operation::Multiply, &["-3", "0"], "0");
        ok(Operation::Divide, &["0", "-5"], "0");
        ok(Operation::Add, &["-0.5", "0.5"], "0");
        ok(Operation::Multiply, &["0.5", "5"], "2.5");
        ok(Operation::Add, &["1.25", "1.25"], "2.5");
    }

    #[test]
    fn wrong_operand_count_is_invalid_input() {
        assert_eq!(
            eval(Operation::Add, &["1"]).unwrap_err().code(),
            "INVALID_INPUT"
        );
        assert_eq!(
            eval(Operation::Add, &["1", "2", "3"]).unwrap_err().code(),
            "INVALID_INPUT"
        );
        assert_eq!(
            eval(Operation::Sqrt, &["1", "2"]).unwrap_err().code(),
            "INVALID_INPUT"
        );
    }

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

/// Property tests over generated operands (ADR-0005, T-07).
#[cfg(test)]
mod properties {
    use super::*;
    use proptest::prelude::*;

    const OPS: [Operation; 8] = [
        Operation::Add,
        Operation::Subtract,
        Operation::Multiply,
        Operation::Divide,
        Operation::Power,
        Operation::Sqrt,
        Operation::Modulo,
        Operation::Percentage,
    ];

    /// Any valid operand: magnitude below 10^28, 0–28 decimal places.
    fn operand() -> impl Strategy<Value = Decimal> {
        let max = 10i128.pow(28) - 1;
        (-max..=max, 0u32..=28).prop_map(|(m, scale)| Decimal::from_i128_with_scale(m, scale))
    }

    /// Small operands: magnitude below 10^15, at most 10 decimal places.
    fn small() -> impl Strategy<Value = Decimal> {
        (-10i64.pow(15) + 1..10i64.pow(15), 0u32..=10).prop_map(|(m, scale)| Decimal::new(m, scale))
    }

    fn well_formed(d: Decimal) -> bool {
        d.abs() < LIMIT
            && !(d.is_zero() && d.is_sign_negative())
            && d == d.normalize()
            && d.scale() == d.normalize().scale()
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2000))]

        // NFR-4, FR-10, FR-13: never panics; every Ok result is in range,
        // never -0, no trailing zeros; every error is a PRD code.
        #[test]
        fn evaluate_is_total_and_results_are_well_formed(
            op in prop::sample::select(&OPS[..]),
            a in operand(),
            b in operand(),
        ) {
            let operands = if op.arity() == 1 { vec![a] } else { vec![a, b] };
            match evaluate(op, &operands) {
                Ok(d) => prop_assert!(well_formed(d), "{op:?}({a}, {b}) = {d:?}"),
                Err(e) => prop_assert!(
                    ["DIVISION_BY_ZERO", "UNDEFINED_RESULT", "RESULT_OUT_OF_RANGE"].contains(&e.code()),
                    "{op:?}({a}, {b}) -> {e:?}"
                ),
            }
        }

        #[test]
        fn add_and_multiply_commute(a in operand(), b in operand()) {
            prop_assert_eq!(evaluate(Operation::Add, &[a, b]), evaluate(Operation::Add, &[b, a]));
            prop_assert_eq!(evaluate(Operation::Multiply, &[a, b]), evaluate(Operation::Multiply, &[b, a]));
        }

        // Exact for small operands; larger ones may round to 28 places by design.
        #[test]
        fn subtract_undoes_add(a in small(), b in small()) {
            let sum = evaluate(Operation::Add, &[a, b]).unwrap();
            prop_assert_eq!(evaluate(Operation::Subtract, &[sum, b]).unwrap(), a.normalize());
        }

        #[test]
        fn sqrt_matches_power_one_half(x in operand()) {
            let x = x.abs();
            prop_assert_eq!(
                evaluate(Operation::Sqrt, &[x]),
                evaluate(Operation::Power, &[x, Decimal::new(5, 1)])
            );
        }

        // Parsing is the inverse of printing for every valid operand.
        #[test]
        fn parse_operand_round_trips(x in operand()) {
            prop_assert_eq!(parse_operand(&x.to_string()).unwrap(), x);
        }
    }
}
