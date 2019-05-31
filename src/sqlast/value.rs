// Copyright 2018 Grove Enterprises LLC
//
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
//
// Additional modifications to this file may have been made by Timely
// Data, Inc. See the version control log for precise modification
// information. The derived work is copyright 2019 Timely Data and
// is not licensed under the terms of the above license.

use ordered_float::OrderedFloat;

/// SQL values such as int, double, string, timestamp
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum Value {
    /// Literal signed long
    Long(i64),
    /// Literal floating point value
    Double(OrderedFloat<f64>),
    /// 'string value'
    SingleQuotedString(String),
    /// N'string value'
    NationalStringLiteral(String),
    /// X'hex value'
    HexStringLiteral(String),
    /// Boolean value true or false,
    Boolean(bool),
    /// Date literals
    Date(String),
    /// Time literals
    Time(String),
    /// Timestamp literals, which include both a date and time
    Timestamp(String),
    /// Time intervals
    Interval {
        value: String,
        start_qualifier: SQLIntervalQualifier,
        end_qualifier: SQLIntervalQualifier,
    },
    /// NULL value in insert statements,
    Null,
}

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Value::Long(v) => v.to_string(),
            Value::Double(v) => v.to_string(),
            Value::SingleQuotedString(v) => format!("'{}'", escape_single_quote_string(v)),
            Value::NationalStringLiteral(v) => format!("N'{}'", v),
            Value::HexStringLiteral(v) => format!("X'{}'", v),
            Value::Boolean(v) => v.to_string(),
            Value::Date(v) => format!("DATE '{}'", escape_single_quote_string(v)),
            Value::Time(v) => format!("TIME '{}'", escape_single_quote_string(v)),
            Value::Timestamp(v) => format!("TIMESTAMP '{}'", escape_single_quote_string(v)),
            Value::Interval {
                value,
                start_qualifier,
                end_qualifier,
            } => format_interval(value, start_qualifier, end_qualifier),
            Value::Null => "NULL".to_string(),
        }
    }
}

fn format_interval(
    value: &str,
    start_qualifier: &SQLIntervalQualifier,
    end_qualifier: &SQLIntervalQualifier,
) -> String {
    let mut s = format!("INTERVAL '{}' ", escape_single_quote_string(value),);
    match (start_qualifier, end_qualifier) {
        (
            SQLIntervalQualifier {
                field: SQLDateTimeField::Second,
                precision: Some(p1),
            },
            SQLIntervalQualifier {
                field: SQLDateTimeField::Second,
                precision: Some(p2),
            },
        ) => {
            // Both the start and end fields are in seconds, and both have
            // precisions. The SQL standard special cases how this is formatted.
            s += &format!("SECOND ({}, {})", p1, p2);
        }

        (start, end) if start == end => {
            // The start and end qualifiers are the same. In this case we can
            // output only the start field.
            s += &start_qualifier.to_string()
        }

        _ => {
            // General case: output both, with precisions.
            s += &format!(
                "{} TO {}",
                start_qualifier.to_string(),
                end_qualifier.to_string()
            );
        }
    }
    s
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct SQLIntervalQualifier {
    pub field: SQLDateTimeField,
    pub precision: Option<usize>,
}

impl ToString for SQLIntervalQualifier {
    fn to_string(&self) -> String {
        let mut s = self.field.to_string();
        if let Some(precision) = self.precision {
            s += &format!(" ({})", precision);
        }
        s
    }
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum SQLDateTimeField {
    Year,
    Month,
    Day,
    Hour,
    Minute,
    Second,
}

impl ToString for SQLDateTimeField {
    fn to_string(&self) -> String {
        match self {
            SQLDateTimeField::Year => "YEAR".to_string(),
            SQLDateTimeField::Month => "MONTH".to_string(),
            SQLDateTimeField::Day => "DAY".to_string(),
            SQLDateTimeField::Hour => "HOUR".to_string(),
            SQLDateTimeField::Minute => "MINUTE".to_string(),
            SQLDateTimeField::Second => "SECOND".to_string(),
        }
    }
}

fn escape_single_quote_string(s: &str) -> String {
    let mut escaped = String::new();
    for c in s.chars() {
        if c == '\'' {
            escaped.push_str("\'\'");
        } else {
            escaped.push(c);
        }
    }
    escaped
}
