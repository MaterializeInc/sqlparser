use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IntervalValue {
    /// The raw `[value]` that was present in `INTERVAL '[value]'`
    pub value: String,
    /// the fully parsed date time
    pub parsed: ParsedDateTime,
    /// The unit of the first field in the interval. `INTERVAL 'T' MINUTE`
    /// means `T` is in minutes
    pub leading_field: DateTimeField,
    /// How many digits the leading field is allowed to occupy.
    ///
    /// The interval `INTERVAL '1234' MINUTE(3)` is **illegal**, but `INTERVAL
    /// '123' MINUTE(3)` is fine.
    ///
    /// This parser does not do any validation that fields fit.
    pub leading_precision: Option<u64>,
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
    pub last_field: Option<DateTimeField>,
    /// The seconds precision can be specified in SQL source as
    /// `INTERVAL '__' SECOND(_, x)` (in which case the `leading_field`
    /// will be `Second` and the `last_field` will be `None`),
    /// or as `__ TO SECOND(x)`.
    pub fractional_seconds_precision: Option<u64>,
}

/// All of the fields that can appear in a literal `TIMESTAMP` or `INTERVAL` string
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParsedDateTime {
    pub is_positive: bool,
    pub year: Option<u64>,
    pub month: Option<u64>,
    pub day: Option<u64>,
    pub hour: Option<u64>,
    pub minute: Option<u64>,
    pub second: Option<u64>,
    pub nano: Option<u32>,
}

impl Default for ParsedDateTime {
    fn default() -> ParsedDateTime {
        ParsedDateTime {
            is_positive: true,
            year: None,
            month: None,
            day: None,
            hour: None,
            minute: None,
            second: None,
            nano: None,
        }
    }
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
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
        f.write_str(match self {
            DateTimeField::Year => "YEAR",
            DateTimeField::Month => "MONTH",
            DateTimeField::Day => "DAY",
            DateTimeField::Hour => "HOUR",
            DateTimeField::Minute => "MINUTE",
            DateTimeField::Second => "SECOND",
        })
    }
}

/// Iterate over `DateTimeField`s in descending significance
impl IntoIterator for DateTimeField {
    type Item = DateTimeField;
    type IntoIter = DateTimeFieldIterator;
    fn into_iter(self) -> DateTimeFieldIterator {
        DateTimeFieldIterator(Some(self))
    }
}

/// An iterator over DateTimeFields
///
/// Always starts with the value smaller than the current one.
///
/// ```
/// use sqlparser::ast::DateTimeField::*;
/// let mut itr = Hour.into_iter();
/// assert_eq!(itr.next(), Some(Minute));
/// assert_eq!(itr.next(), Some(Second));
/// assert_eq!(itr.next(), None);
/// ```
pub struct DateTimeFieldIterator(Option<DateTimeField>);

/// Go through fields in descending significance order
impl Iterator for DateTimeFieldIterator {
    type Item = DateTimeField;
    fn next(&mut self) -> Option<Self::Item> {
        use DateTimeField::*;
        self.0 = match self.0 {
            Some(Year) => Some(Month),
            Some(Month) => Some(Day),
            Some(Day) => Some(Hour),
            Some(Hour) => Some(Minute),
            Some(Minute) => Some(Second),
            Some(Second) => None,
            None => None,
        };
        self.0.clone()
    }
}
