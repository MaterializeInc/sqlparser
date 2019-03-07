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

extern crate log;
extern crate sqlparser;

use sqlparser::dialect::AnsiSqlDialect;
use sqlparser::sqlast::*;
use sqlparser::sqlparser::*;

#[test]
fn parse_simple_select() {
    let sql = String::from("SELECT id, fname, lname FROM customer WHERE id = 1");
    let ast = Parser::parse_sql(&AnsiSqlDialect {}, sql).unwrap();
    assert_eq!(1, ast.len());
    match ast.first().unwrap() {
        SQLStatement::SQLSelect(SQLQuery {
            body: SQLSetExpr::Select(SQLSelect { projection, .. }),
            ..
        }) => {
            assert_eq!(3, projection.len());
        }
        _ => assert!(false),
    }
}
