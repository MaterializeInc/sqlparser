use crate::ast::ParsedDateTime;
use crate::parser::{DateTimeField, ParserError};

pub(crate) fn tokenize_interval(value: &str) -> Result<Vec<IntervalToken>, ParserError> {
    let mut toks = vec![];
    let mut num_buf = String::with_capacity(4);
    fn parse_num(n: &str, idx: usize) -> Result<IntervalToken, ParserError> {
        Ok(IntervalToken::Num(n.parse().map_err(|e| {
            ParserError::ParserError(format!(
                "Unable to parse value as a number at index {}: {}",
                idx, e
            ))
        })?))
    };
    let mut last_field_is_frac = false;
    for (i, chr) in value.chars().enumerate() {
        match chr {
            '-' => {
                // dashes at the beginning mean make it negative
                if !num_buf.is_empty() {
                    toks.push(parse_num(&num_buf, i)?);
                    num_buf.clear();
                }
                toks.push(IntervalToken::Dash);
            }
            ' ' => {
                toks.push(parse_num(&num_buf, i)?);
                num_buf.clear();
                toks.push(IntervalToken::Space);
            }
            ':' => {
                toks.push(parse_num(&num_buf, i)?);
                num_buf.clear();
                toks.push(IntervalToken::Colon);
            }
            '.' => {
                toks.push(parse_num(&num_buf, i)?);
                num_buf.clear();
                toks.push(IntervalToken::Dot);
                last_field_is_frac = true;
            }
            chr if chr.is_digit(10) => num_buf.push(chr),
            chr => {
                return Err(ParserError::TokenizerError(format!(
                    "Invalid character at offset {} in {}: {:?}",
                    i, value, chr
                )))
            }
        }
    }
    if !num_buf.is_empty() {
        if !last_field_is_frac {
            toks.push(parse_num(&num_buf, 0)?);
        } else {
            let raw: u32 = num_buf.parse().map_err(|e| {
                ParserError::ParserError(format!(
                    "couldn't parse fraction of second {}: {}",
                    num_buf, e
                ))
            })?;
            let leading_zeroes = num_buf.chars().take_while(|c| c == &'0').count() as u32;
            let multiplicand = 1_000_000_000 / 10_u32.pow(1 + leading_zeroes);

            toks.push(IntervalToken::Nanos(raw * multiplicand));
        }
    }
    Ok(toks)
}

/// Get the tokens that you *might* end up parsing starting with a most significant unit
///
/// For example, parsing `INTERVAL '9-5 4:3' MONTH` is *illegal*, but you
/// should interpret that as `9 months 5 days 4 hours 3 minutes`. This function
/// doesn't take any perspective on what things should be, it just teslls you
/// what the user might have meant.
fn potential_interval_tokens(from: &DateTimeField) -> Vec<IntervalToken> {
    use DateTimeField::*;
    use IntervalToken::*;

    let all_toks = [
        Num(0), // year
        Dash,
        Num(0), // month
        Dash,
        Num(0), // day
        Space,
        Num(0), // hour
        Colon,
        Num(0), // minute
        Colon,
        Num(0), // second
        Dot,
        Nanos(0), // Nanos
    ];
    let offset = match from {
        Year => 0,
        Month => 2,
        Day => 4,
        Hour => 6,
        Minute => 8,
        Second => 10,
    };
    all_toks[offset..].to_vec()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IntervalToken {
    Dash,
    Space,
    Colon,
    Dot,
    Num(u64),
    Nanos(u32),
}

pub(crate) fn build_parsed_datetime(
    tokens: &[IntervalToken],
    leading_field: &DateTimeField,
    precision: &Option<DateTimeField>,
) -> Result<(ParsedDateTime, Vec<String>), ParserError> {
    use IntervalToken::*;

    // if no precision is specified, then you should use the least possible precision
    let precision = match precision {
        Some(p) => p,
        None => leading_field,
    };

    let expected = potential_interval_tokens(&leading_field);
    let mut actual = tokens.iter().peekable();

    let mut warnings = vec![];

    if expected.len() > tokens.len() - 1 {
        warnings.push(format!(
            "More precision requested than supplied. Requested {} but only provided {} fields",
            precision,
            tokens.len(),
        ))
    }

    let is_positive = match actual.peek() {
        Some(val) if val == &&IntervalToken::Dash => {
            actual.next();
            false
        }
        _ => true,
    };
    let mut current_field = leading_field.clone();
    let mut pdt = ParsedDateTime {
        is_positive,
        ..Default::default()
    };
    let mut seconds_seen = 0;
    for (atok, etok) in actual.zip(&expected) {
        match (atok, etok) {
            (Dash, Dash) | (Space, Space) | (Colon, Colon) | (Dot, Dot) => {
                /* matching punctuation */
            }
            (Num(val), Num(_)) => {
                let val = *val;
                match current_field {
                    DateTimeField::Year => pdt.year = Some(val),
                    DateTimeField::Month => pdt.month = Some(val),
                    DateTimeField::Day => pdt.day = Some(val),
                    DateTimeField::Hour => pdt.hour = Some(val),
                    DateTimeField::Minute => pdt.minute = Some(val),
                    DateTimeField::Second if seconds_seen == 0 => {
                        seconds_seen += 1;
                        pdt.second = Some(val);
                    }
                    DateTimeField::Second => {
                        return parser_err!("Too many numbers to parse as a second at {}", val)
                    }
                }
                if current_field != DateTimeField::Second {
                    current_field = current_field
                        .into_iter()
                        .next()
                        .expect("Exhausted day iterator");
                }
            }
            (Nanos(val), Nanos(_)) if seconds_seen == 1 => pdt.nano = Some(*val),
            (provided, expected) => {
                return parser_err!(
                    "Invalid interval part: string provided {:?} but expected {:?}",
                    provided,
                    expected,
                )
            }
        }
    }

    Ok((pdt, warnings))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser::*;

    #[test]
    fn test_potential_interval_tokens() {
        use DateTimeField::*;
        use IntervalToken::*;
        assert_eq!(
            potential_interval_tokens(&Year),
            vec![
                Num(0),
                Dash,
                Num(0),
                Dash,
                Num(0),
                Space,
                Num(0),
                Colon,
                Num(0),
                Colon,
                Num(0),
                Dot,
                Nanos(0),
            ]
        );

        assert_eq!(
            potential_interval_tokens(&Day),
            vec![
                Num(0),
                Space,
                Num(0),
                Colon,
                Num(0),
                Colon,
                Num(0),
                Dot,
                Nanos(0),
            ]
        );
    }
}
