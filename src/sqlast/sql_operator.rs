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

/// SQL Operator
#[derive(Debug, PartialEq, Clone)]
pub enum SQLOperator {
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulus,
    Gt,
    Lt,
    GtEq,
    LtEq,
    Eq,
    NotEq,
    And,
    Or,
    Not,
    Like,
    NotLike,
}

impl ToString for SQLOperator {
    fn to_string(&self) -> String {
        match self {
            SQLOperator::Plus => "+".to_string(),
            SQLOperator::Minus => "-".to_string(),
            SQLOperator::Multiply => "*".to_string(),
            SQLOperator::Divide => "/".to_string(),
            SQLOperator::Modulus => "%".to_string(),
            SQLOperator::Gt => ">".to_string(),
            SQLOperator::Lt => "<".to_string(),
            SQLOperator::GtEq => ">=".to_string(),
            SQLOperator::LtEq => "<=".to_string(),
            SQLOperator::Eq => "=".to_string(),
            SQLOperator::NotEq => "!=".to_string(),
            SQLOperator::And => "AND".to_string(),
            SQLOperator::Or => "OR".to_string(),
            SQLOperator::Not => "NOT".to_string(),
            SQLOperator::Like => "LIKE".to_string(),
            SQLOperator::NotLike => "NOT LIKE".to_string(),
        }
    }
}
