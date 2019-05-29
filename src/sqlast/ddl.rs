//! AST types specific to CREATE/ALTER variants of `SQLStatement`
//! (commonly referred to as Data Definition Language, or DDL)
use super::{ASTNode, SQLIdent, SQLObjectName, SQLType};

/// An `ALTER TABLE` (`SQLStatement::SQLAlterTable`) operation
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum AlterTableOperation {
    /// `ADD <table_constraint>`
    AddConstraint(TableConstraint),
    /// TODO: implement `DROP CONSTRAINT <name>`
    DropConstraint { name: SQLIdent },
}

impl ToString for AlterTableOperation {
    fn to_string(&self) -> String {
        match self {
            AlterTableOperation::AddConstraint(c) => format!("ADD {}", c.to_string()),
            AlterTableOperation::DropConstraint { name } => format!("DROP CONSTRAINT {}", name),
        }
    }
}

/// A table-level constraint, specified in a `CREATE TABLE` or an
/// `ALTER TABLE ADD <constraint>` statement.
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum TableConstraint {
    /// `[ CONSTRAINT <name> ] { PRIMARY KEY | UNIQUE } (<columns>)`
    Unique {
        name: Option<SQLIdent>,
        columns: Vec<SQLIdent>,
        /// Whether this is a `PRIMARY KEY` or just a `UNIQUE` constraint
        is_primary: bool,
    },
    /// A referential integrity constraint (`[ CONSTRAINT <name> ] FOREIGN KEY (<columns>)
    /// REFERENCES <foreign_table> (<referred_columns>)`)
    ForeignKey {
        name: Option<SQLIdent>,
        columns: Vec<SQLIdent>,
        foreign_table: SQLObjectName,
        referred_columns: Vec<SQLIdent>,
    },
    /// `[ CONSTRAINT <name> ] CHECK (<expr>)`
    Check {
        name: Option<SQLIdent>,
        expr: Box<ASTNode>,
    },
}

impl ToString for TableConstraint {
    fn to_string(&self) -> String {
        match self {
            TableConstraint::Unique {
                name,
                columns,
                is_primary,
            } => format!(
                "{}{} ({})",
                format_constraint_name(name),
                if *is_primary { "PRIMARY KEY" } else { "UNIQUE" },
                columns.join(", ")
            ),
            TableConstraint::ForeignKey {
                name,
                columns,
                foreign_table,
                referred_columns,
            } => format!(
                "{}FOREIGN KEY ({}) REFERENCES {}({})",
                format_constraint_name(name),
                columns.join(", "),
                foreign_table.to_string(),
                referred_columns.join(", ")
            ),
            TableConstraint::Check { name, expr } => format!(
                "{}CHECK ({})",
                format_constraint_name(name),
                expr.to_string()
            ),
        }
    }
}

/// SQL column definition
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct SQLColumnDef {
    pub name: SQLIdent,
    pub data_type: SQLType,
    pub collation: Option<SQLObjectName>,
    pub constraints: Vec<ColumnConstraint>,
}

impl ToString for SQLColumnDef {
    fn to_string(&self) -> String {
        format!(
            "{} {}{}",
            self.name,
            self.data_type.to_string(),
            self.constraints
                .iter()
                .map(|c| format!(" {}", c.to_string()))
                .collect::<Vec<_>>()
                .join("")
        )
    }
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum ColumnConstraint {
    /// `NULL`
    ///
    /// The ANSI specification technically allows NULL constraints to have a
    /// name, but no known databases retain that name, if they even parse it
    /// at all. Just omit it until we have evidence that it's important.
    Null,
    /// `NOT NULL`
    ///
    /// As with `NULL`, `NOT NULL` constraints can technically have a name,
    /// but we choose to omit it.
    NotNull,
    /// `[ CONSTRAINT <name> ] DEFAULT <restricted-expr>`
    Default {
        name: Option<SQLIdent>,
        expr: ASTNode,
    },
    /// `[ CONSTRAINT <name> ] { PRIMARY KEY | UNIQUE }`
    Unique {
        name: Option<SQLIdent>,
        /// Whether this is a `PRIMARY KEY` or just a `UNIQUE` constraint
        is_primary: bool,
    },
    /// A referential integrity constraint (`[ CONSTRAINT <name> ] FOREIGN KEY
    /// REFERENCES <foreign_table> (<referred_columns>)`)
    ForeignKey {
        name: Option<SQLIdent>,
        foreign_table: SQLObjectName,
        referred_columns: Vec<SQLIdent>,
    },
    // `[ CONSTRAINT <name> ] CHECK (<expr>)`
    Check {
        name: Option<SQLIdent>,
        expr: Box<ASTNode>,
    },
}

impl ToString for ColumnConstraint {
    fn to_string(&self) -> String {
        use ColumnConstraint::*;
        match self {
            Null => "NULL".to_string(),
            NotNull => "NOT NULL".to_string(),
            Default { name, expr } => format!(
                "{}DEFAULT {}",
                format_constraint_name(name),
                expr.to_string()
            ),
            Unique { name, is_primary } => format!(
                "{}{}",
                format_constraint_name(name),
                if *is_primary { "PRIMARY KEY" } else { "UNIQUE" },
            ),
            ForeignKey {
                name,
                foreign_table,
                referred_columns,
            } => format!(
                "{}REFERENCES {} ({})",
                format_constraint_name(name),
                foreign_table.to_string(),
                referred_columns.join(", ")
            ),
            Check { name, expr } => format!(
                "{}CHECK ({})",
                format_constraint_name(name),
                expr.to_string()
            ),
        }
    }
}

fn format_constraint_name(name: &Option<SQLIdent>) -> String {
    name.as_ref()
        .map(|name| format!("CONSTRAINT {} ", name))
        .unwrap_or_default()
}
