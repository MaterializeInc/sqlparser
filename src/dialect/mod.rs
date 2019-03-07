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

mod ansi_sql;
mod generic_sql;
pub mod keywords;
mod postgresql;

pub use self::ansi_sql::AnsiSqlDialect;
pub use self::generic_sql::GenericSqlDialect;
pub use self::postgresql::PostgreSqlDialect;

pub trait Dialect {
    /// Determine if a character starts a quoted identifier. The default
    /// implementation, accepting "double quoted" ids is both ANSI-compliant
    /// and appropriate for most dialects (with the notable exception of
    /// MySQL, MS SQL, and sqlite). You can accept one of characters listed
    /// in `SQLWord::matching_end_quote()` here
    fn is_delimited_identifier_start(&self, ch: char) -> bool {
        ch == '"'
    }
    /// Determine if a character is a valid start character for an unquoted identifier
    fn is_identifier_start(&self, ch: char) -> bool;
    /// Determine if a character is a valid unquoted identifier character
    fn is_identifier_part(&self, ch: char) -> bool;
}
