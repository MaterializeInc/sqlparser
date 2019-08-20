use std::fmt;
use std::time::Duration;

use super::ValueError;

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

impl IntervalValue {
    /// Get Either the number of Months or the Duration specified by this interval
    ///
    /// This computes the fiels permissively: it assumes that the leading field
    /// (i.e. the lead in `INTERVAL 'str' LEAD [TO LAST]`) is valid and parses
    /// all field in the `str` starting at the leading field, ignoring the
    /// truncation that should be specified by `LAST`.
    ///
    /// See also the related [`fields_match_precision`] function that will give
    /// an error if the interval string does not exactly match the `FROM TO
    /// LAST` spec.
    ///
    /// # Errors
    ///
    /// If a required field is missing (i.e. there is no value) or the `TO
    /// LAST` field is larger than the `LEAD`.
    pub fn computed_permissive(&self) -> Result<Interval, ValueError> {
        use DateTimeField::*;
        match &self.leading_field {
            Year => match &self.last_field {
                Some(Month) => Ok(Interval::Months(
                    self.positivity() * self.parsed.year.unwrap_or(0) as i64 * 12
                        + self.parsed.month.unwrap_or(0) as i64,
                )),
                Some(Year) | None => self
                    .parsed
                    .year
                    .ok_or_else(|| ValueError("No YEAR provided".into()))
                    .map(|year| Interval::Months(self.positivity() * year as i64 * 12)),
                Some(invalid) => Err(ValueError(format!(
                    "Invalid specifier for YEAR precision: {}",
                    &invalid
                ))),
            },
            Month => match &self.last_field {
                Some(Month) | None => self
                    .parsed
                    .month
                    .ok_or_else(|| ValueError("No MONTH provided".into()))
                    .map(|m| Interval::Months(self.positivity() * m as i64)),
                Some(invalid) => Err(ValueError(format!(
                    "Invalid specifier for MONTH precision: {}",
                    &invalid
                ))),
            },
            durationlike_field => {
                let mut seconds = 0u64;
                match self.units_of(&durationlike_field) {
                    Some(time) => seconds += time * seconds_multiplier(&durationlike_field),
                    None => {
                        return Err(ValueError(format!(
                            "No {} provided in value string for {}",
                            durationlike_field, self.value
                        )))
                    }
                }
                let min_field = &self
                    .last_field
                    .clone()
                    .unwrap_or_else(|| durationlike_field.clone());
                for field in durationlike_field
                    .clone()
                    .into_iter()
                    .take_while(|f| f <= min_field)
                {
                    if let Some(time) = self.units_of(&field) {
                        seconds += time * seconds_multiplier(&field);
                    }
                }
                let duration = match (min_field, self.parsed.nano) {
                    (DateTimeField::Second, Some(nanos)) => Duration::new(seconds, nanos),
                    (_, _) => Duration::from_secs(seconds),
                };
                Ok(Interval::Duration {
                    is_positive: self.parsed.is_positive,
                    duration,
                })
            }
        }
    }

    /// Retrieve the number that we parsed out of the literal string for the `field`
    fn units_of(&self, field: &DateTimeField) -> Option<u64> {
        match field {
            DateTimeField::Year => self.parsed.year,
            DateTimeField::Month => self.parsed.month,
            DateTimeField::Day => self.parsed.day,
            DateTimeField::Hour => self.parsed.hour,
            DateTimeField::Minute => self.parsed.minute,
            DateTimeField::Second => self.parsed.second,
        }
    }

    /// Verify that the fields in me make sense
    ///
    /// Returns Ok if the fields are fully specified, otherwise an error
    ///
    /// # Examples
    ///
    /// ```sql
    /// INTERVAL '1 5' DAY TO HOUR -- Ok
    /// INTERVAL '1 5' DAY         -- Err
    /// INTERVAL '1:2:3' HOUR TO SECOND   -- Ok
    /// INTERVAL '1:2:3' HOUR TO MINUTE   -- Err
    /// INTERVAL '1:2:3' MINUTE TO SECOND -- Err
    /// INTERVAL '1:2:3' DAY TO SECOND    -- Err
    /// ```
    pub fn fields_match_precision(&self) -> Result<(), ValueError> {
        let mut errors = vec![];
        let last_field = self
            .last_field
            .as_ref()
            .unwrap_or_else(|| &self.leading_field);
        let mut extra_leading_fields = vec![];
        let mut extra_trailing_fields = vec![];
        // check for more data in the input string than was requested in <FIELD> TO <FIELD>
        for field in std::iter::once(DateTimeField::Year).chain(DateTimeField::Year.into_iter()) {
            if self.units_of(&field).is_none() {
                continue;
            }

            if field < self.leading_field {
                extra_leading_fields.push(field.clone());
            }
            if &field > last_field {
                extra_trailing_fields.push(field.clone());
            }
        }

        if !extra_leading_fields.is_empty() {
            errors.push(format!(
                "The interval string '{}' specifies {}s but the significance requested is {}",
                self.value,
                fields_msg(extra_leading_fields.into_iter()),
                self.leading_field
            ));
        }
        if !extra_trailing_fields.is_empty() {
            errors.push(format!(
                "The interval string '{}' specifies {}s but the requested precision would truncate to {}",
                self.value, fields_msg(extra_trailing_fields.into_iter()), last_field
            ));
        }

        // check for data requested by the <FIELD> TO <FIELD> that does not exist in the data
        let missing_fields = match (
            self.units_of(&self.leading_field),
            self.units_of(&last_field),
        ) {
            (Some(_), Some(_)) => vec![],
            (None, Some(_)) => vec![&self.leading_field],
            (Some(_), None) => vec![last_field],
            (None, None) => vec![&self.leading_field, last_field],
        };

        if !missing_fields.is_empty() {
            errors.push(format!(
                "The interval string '{}' provides {} - which does not include the requested field(s) {}",
                self.value, self.present_fields(), fields_msg(missing_fields.into_iter().cloned())));
        }

        if !errors.is_empty() {
            Err(ValueError(errors.join("; ")))
        } else {
            Ok(())
        }
    }

    fn present_fields(&self) -> String {
        fields_msg(
            std::iter::once(DateTimeField::Year)
                .chain(DateTimeField::Year.into_iter())
                .filter(|field| self.units_of(&field).is_some()),
        )
    }

    /// `1` if is_positive, otherwise `-1`
    fn positivity(&self) -> i64 {
        if self.parsed.is_positive {
            1
        } else {
            -1
        }
    }
}

fn fields_msg(fields: impl Iterator<Item = DateTimeField>) -> String {
    fields
        .map(|field: DateTimeField| field.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn seconds_multiplier(field: &DateTimeField) -> u64 {
    match field {
        DateTimeField::Day => 60 * 60 * 24,
        DateTimeField::Hour => 60 * 60,
        DateTimeField::Minute => 60,
        DateTimeField::Second => 1,
        _other => unreachable!("Do not call with a non-duration field"),
    }
}

/// The result of parsing an `INTERVAL '<value>' <unit> [TO <precision>]`
///
/// Units of type `YEAR` or `MONTH` are semantically some multiple of months,
/// which are not well defined, and this parser normalizes them to some number
/// of months.
///
/// Intervals of unit [`DateTimeField::Day`] or smaller are semantically a
/// multiple of seconds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Interval {
    /// A possibly negative number of months for field types like `YEAR`
    Months(i64),
    /// An actual timespan, possibly negative, because why not
    Duration {
        is_positive: bool,
        duration: Duration,
    },
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
