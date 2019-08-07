// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use bigdecimal::BigDecimal;
use std::fmt;

/// Primitive SQL values such as number and string
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value {
    /// Unsigned integer value
    Long(u64),
    /// Unsigned decimal fraction
    Decimal(BigDecimal),
    /// 'string value'
    SingleQuotedString(String),
    /// N'string value'
    NationalStringLiteral(String),
    /// X'hex value'
    HexStringLiteral(String),
    /// Boolean value true or false
    Boolean(bool),
    /// `DATE '...'` literals
    Date(String),
    /// `TIME '...'` literals
    Time(String),
    /// `TIMESTAMP '...'` literals
    Timestamp(String),
    /// INTERVAL literals, roughly in the following format:
    ///
    /// ```
    /// INTERVAL '<value>' <leading_field> [ (<leading_precision>) ]
    ///     [ TO <last_field> [ (<fractional_seconds_precision>) ] ]
    /// ```
    /// e.g. `INTERVAL '123:45.67' MINUTE(3) TO SECOND(2)`.
    ///
    /// The parser does not validate the `<value>`, nor does it ensure
    /// that the `<leading_field>` units >= the units in `<last_field>`,
    /// so the user will have to reject intervals like `HOUR TO YEAR`.
    Interval {
        /// The raw [value] that was present in `INTERVAL '[value]'`
        value: String,
        /// The unit of the first field in the interval. `INTERVAL 'T' MINUTE`
        /// means `T` is in minutes
        leading_field: DateTimeField,
        /// How many digits the leading field is allowed to occupy.
        ///
        /// The interval `INTERVAL '1234' MINUTE(3)` is **illegal**, but `INTERVAL
        /// '123' MINUTE(3)` is fine.
        ///
        /// This parser does not do any validation that fields fit.
        leading_precision: Option<u64>,
        /// How much precision to keep track of
        ///
        /// If this is ommitted, then you are supposed to ignore all of the
        /// non-lead fields. If it is less precise than the final field, you
        /// are supposed to ignore the final field.
        ///
        /// For the following specifications:
        ///
        /// * `INTERVAL '1:1:1' HOURS TO SECONDS` the `last_field` gets
        ///   `Some(DateTimeField::Second)` and interpreters should generate an
        ///   interval equivalent to `3661` seconds.
        /// * In `INTERVAL '1:1:1' HOURS` the `last_field` gets `None` and
        ///   interpreters should generate an interval equivalent to `3600`
        ///   seconds.
        /// * In `INTERVAL '1:1:1' HOURS TO MINUTES` the interval should be
        ///   equivalent to `3660` seconds.
        last_field: Option<DateTimeField>,
        /// The seconds precision can be specified in SQL source as
        /// `INTERVAL '__' SECOND(_, x)` (in which case the `leading_field`
        /// will be `Second` and the `last_field` will be `None`),
        /// or as `__ TO SECOND(x)`.
        fractional_seconds_precision: Option<u64>,
    },
    /// `NULL` value
    Null,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Long(v) => write!(f, "{}", v),
            Value::Decimal(v) => write!(f, "{}", v),
            Value::SingleQuotedString(v) => write!(f, "'{}'", escape_single_quote_string(v)),
            Value::NationalStringLiteral(v) => write!(f, "N'{}'", v),
            Value::HexStringLiteral(v) => write!(f, "X'{}'", v),
            Value::Boolean(v) => write!(f, "{}", v),
            Value::Date(v) => write!(f, "DATE '{}'", escape_single_quote_string(v)),
            Value::Time(v) => write!(f, "TIME '{}'", escape_single_quote_string(v)),
            Value::Timestamp(v) => write!(f, "TIMESTAMP '{}'", escape_single_quote_string(v)),
            Value::Interval {
                value,
                leading_field: DateTimeField::Second,
                leading_precision: Some(leading_precision),
                last_field,
                fractional_seconds_precision: Some(fractional_seconds_precision),
            } => {
                // When the leading field is SECOND, the parser guarantees that
                // the last field is None.
                assert!(last_field.is_none());
                write!(
                    f,
                    "INTERVAL '{}' SECOND ({}, {})",
                    escape_single_quote_string(value),
                    leading_precision,
                    fractional_seconds_precision
                )
            }
            Value::Interval {
                value,
                leading_field,
                leading_precision,
                last_field,
                fractional_seconds_precision,
            } => {
                write!(
                    f,
                    "INTERVAL '{}' {}",
                    escape_single_quote_string(value),
                    leading_field
                )?;
                if let Some(leading_precision) = leading_precision {
                    write!(f, " ({})", leading_precision)?;
                }
                if let Some(last_field) = last_field {
                    write!(f, " TO {}", last_field)?;
                }
                if let Some(fractional_seconds_precision) = fractional_seconds_precision {
                    write!(f, " ({})", fractional_seconds_precision)?;
                }
                Ok(())
            }
            Value::Null => write!(f, "NULL"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DateTimeField {
    Year,
    Month,
    Day,
    Hour,
    Minute,
    Second,
}

impl fmt::Display for DateTimeField {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DateTimeField::Year => "YEAR",
                DateTimeField::Month => "MONTH",
                DateTimeField::Day => "DAY",
                DateTimeField::Hour => "HOUR",
                DateTimeField::Minute => "MINUTE",
                DateTimeField::Second => "SECOND",
            }
        )
    }
}

struct EscapeSingleQuoteString<'a>(&'a str);
impl<'a> fmt::Display for EscapeSingleQuoteString<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for c in self.0.chars() {
            if c == '\'' {
                write!(f, "\'\'")?;
            } else {
                write!(f, "{}", c)?;
            }
        }
        Ok(())
    }
}
fn escape_single_quote_string(s: &str) -> EscapeSingleQuoteString<'_> {
    EscapeSingleQuoteString(s)
}
