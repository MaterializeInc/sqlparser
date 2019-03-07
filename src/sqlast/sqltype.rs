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

use super::SQLObjectName;

/// SQL datatypes for literals in SQL statements
#[derive(Debug, Clone, PartialEq)]
pub enum SQLType {
    /// Fixed-length character type e.g. CHAR(10)
    Char(Option<usize>),
    /// Variable-length character type e.g. VARCHAR(10)
    Varchar(Option<usize>),
    /// Uuid type
    Uuid,
    /// Large character object e.g. CLOB(1000)
    Clob(usize),
    /// Fixed-length binary type e.g. BINARY(10)
    Binary(usize),
    /// Variable-length binary type e.g. VARBINARY(10)
    Varbinary(usize),
    /// Large binary object e.g. BLOB(1000)
    Blob(usize),
    /// Decimal type with optional precision and scale e.g. DECIMAL(10,2)
    Decimal(Option<usize>, Option<usize>),
    /// Small integer
    SmallInt,
    /// Integer
    Int,
    /// Big integer
    BigInt,
    /// Floating point with optional precision e.g. FLOAT(8)
    Float(Option<usize>),
    /// Floating point e.g. REAL
    Real,
    /// Double e.g. DOUBLE PRECISION
    Double,
    /// Boolean
    Boolean,
    /// Date
    Date,
    /// Time
    Time,
    /// Timestamp
    Timestamp,
    /// Regclass used in postgresql serial
    Regclass,
    /// Text
    Text,
    /// Bytea
    Bytea,
    /// Custom type such as enums
    Custom(SQLObjectName),
    /// Arrays
    Array(Box<SQLType>),
}

impl ToString for SQLType {
    fn to_string(&self) -> String {
        match self {
            SQLType::Char(size) => {
                if let Some(size) = size {
                    format!("char({})", size)
                } else {
                    "char".to_string()
                }
            }
            SQLType::Varchar(size) => {
                if let Some(size) = size {
                    format!("character varying({})", size)
                } else {
                    "character varying".to_string()
                }
            }
            SQLType::Uuid => "uuid".to_string(),
            SQLType::Clob(size) => format!("clob({})", size),
            SQLType::Binary(size) => format!("binary({})", size),
            SQLType::Varbinary(size) => format!("varbinary({})", size),
            SQLType::Blob(size) => format!("blob({})", size),
            SQLType::Decimal(precision, scale) => {
                if let Some(scale) = scale {
                    format!("numeric({},{})", precision.unwrap(), scale)
                } else {
                    if let Some(precision) = precision {
                        format!("numeric({})", precision)
                    } else {
                        format!("numeric")
                    }
                }
            }
            SQLType::SmallInt => "smallint".to_string(),
            SQLType::Int => "int".to_string(),
            SQLType::BigInt => "bigint".to_string(),
            SQLType::Float(size) => {
                if let Some(size) = size {
                    format!("float({})", size)
                } else {
                    "float".to_string()
                }
            }
            SQLType::Real => "real".to_string(),
            SQLType::Double => "double".to_string(),
            SQLType::Boolean => "boolean".to_string(),
            SQLType::Date => "date".to_string(),
            SQLType::Time => "time".to_string(),
            SQLType::Timestamp => "timestamp".to_string(),
            SQLType::Regclass => "regclass".to_string(),
            SQLType::Text => "text".to_string(),
            SQLType::Bytea => "bytea".to_string(),
            SQLType::Array(ty) => format!("{}[]", ty.to_string()),
            SQLType::Custom(ty) => ty.to_string(),
        }
    }
}
