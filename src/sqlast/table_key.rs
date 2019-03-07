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

use super::{SQLIdent, SQLObjectName};

#[derive(Debug, PartialEq, Clone)]
pub enum AlterOperation {
    AddConstraint(TableKey),
    RemoveConstraint { name: SQLIdent },
}

impl ToString for AlterOperation {
    fn to_string(&self) -> String {
        match self {
            AlterOperation::AddConstraint(table_key) => {
                format!("ADD CONSTRAINT {}", table_key.to_string())
            }
            AlterOperation::RemoveConstraint { name } => format!("REMOVE CONSTRAINT {}", name),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Key {
    pub name: SQLIdent,
    pub columns: Vec<SQLIdent>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TableKey {
    PrimaryKey(Key),
    UniqueKey(Key),
    Key(Key),
    ForeignKey {
        key: Key,
        foreign_table: SQLObjectName,
        referred_columns: Vec<SQLIdent>,
    },
}

impl ToString for TableKey {
    fn to_string(&self) -> String {
        match self {
            TableKey::PrimaryKey(ref key) => {
                format!("{} PRIMARY KEY ({})", key.name, key.columns.join(", "))
            }
            TableKey::UniqueKey(ref key) => {
                format!("{} UNIQUE KEY ({})", key.name, key.columns.join(", "))
            }
            TableKey::Key(ref key) => format!("{} KEY ({})", key.name, key.columns.join(", ")),
            TableKey::ForeignKey {
                key,
                foreign_table,
                referred_columns,
            } => format!(
                "{} FOREIGN KEY ({}) REFERENCES {}({})",
                key.name,
                key.columns.join(", "),
                foreign_table.to_string(),
                referred_columns.join(", ")
            ),
        }
    }
}
