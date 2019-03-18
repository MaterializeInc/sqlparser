//! SQL AST traversal.

use super::*;

/// A trait that represents a visitor that walks through a SQL AST.
///
/// Each function corresponds to a node in the SQL AST, and has a default
/// implementation that visits all of its child nodes. Implementors of this
/// trait can override functions as desired to hook into AST traversal without
/// writing code to traverse the entire AST.
pub trait Visit<'ast> {
    fn visit_statement(&mut self, statement: &'ast SQLStatement) {
        match statement {
            SQLStatement::SQLSelect(query) => self.visit_query(query),
            SQLStatement::SQLInsert { table_name, columns, values } => self.visit_insert(table_name, columns, values),
            SQLStatement::SQLCopy { table_name, columns, values } => self.visit_copy(table_name, columns, values),
            SQLStatement::SQLUpdate { table_name, assignments, selection } => self.visit_update(table_name, assignments, selection.as_ref()),
            SQLStatement::SQLDelete { table_name, selection } => self.visit_delete(table_name, selection.as_ref()),
            SQLStatement::SQLCreateDataSource { name, url, schema } => self.visit_create_data_source(name, url, schema),
            SQLStatement::SQLCreateView { name, query, materialized } => self.visit_create_view(name, query, *materialized),
            SQLStatement::SQLCreateTable { name, columns } => self.visit_create_table(name, columns),
            SQLStatement::SQLAlterTable { name, operation } => self.visit_alter_table(name, operation),
        }
    }

    fn visit_query(&mut self, query: &'ast SQLQuery) {
        for cte in &query.ctes {
            self.visit_cte(cte);
        }
        self.visit_set_expr(&query.body);
        match query.order_by {
            Some(ref order_bys) => {
                for order_by in order_bys {
                    self.visit_order_by(order_by);
                }
            }
            None => (),
        }
        match query.limit {
            Some(ref expr) => self.visit_limit(expr),
            None => (),
        }
    }

    fn visit_cte(&mut self, cte: &'ast Cte) {
        self.visit_identifier(&cte.alias);
        self.visit_query(&cte.query);
    }

    fn visit_select(&mut self, select: &'ast SQLSelect) {
        for select_item in &select.projection {
            self.visit_select_item(select_item)
        }
        match select.relation {
            Some(ref table_factor) => self.visit_table_factor(table_factor),
            None => (),
        }
        for join in &select.joins {
            self.visit_join(join);
        }
        match select.selection {
            Some(ref expr) => self.visit_where(expr),
            None => (),
        }
        match select.group_by {
            Some(ref exprs) => self.visit_group_by(exprs),
            None => (),
        }
        match select.having {
            Some(ref expr) => self.visit_having(expr),
            None => (),
        }
    }

    fn visit_select_item(&mut self, select_item: &'ast SQLSelectItem) {
        match select_item {
            SQLSelectItem::UnnamedExpression(expr) => self.visit_unnamed_expression(expr),
            SQLSelectItem::ExpressionWithAlias(expr, alias) => self.visit_expression_with_alias(expr, alias),
            SQLSelectItem::QualifiedWildcard(object_name) => self.visit_qualified_wildcard(&object_name.0),
            SQLSelectItem::Wildcard => self.visit_wildcard(),
        }
    }

    fn visit_table_factor(&mut self, table_factor: &'ast TableFactor) {
        match table_factor {
            TableFactor::Table { name, alias } => self.visit_table_table_factor(name, alias.as_ref()),
            TableFactor::Derived { subquery, alias } => self.visit_derived_table_factor(subquery, alias.as_ref()),
        }
    }

    fn visit_table_table_factor(&mut self, name: &'ast SQLObjectName, alias: Option<&'ast SQLIdent>) {
        self.visit_object_name(name);
        match alias {
            Some(ident) => self.visit_identifier(ident),
            None => (),
        }
    }

    fn visit_derived_table_factor(&mut self, subquery: &'ast SQLQuery, alias: Option<&'ast SQLIdent>) {
        self.visit_subquery(subquery);
        match alias {
            Some(ident) => self.visit_identifier(ident),
            None => (),
        }
    }

    fn visit_join(&mut self, join: &'ast Join) {
        self.visit_table_factor(&join.relation);
        self.visit_join_operator(&join.join_operator);
    }

    fn visit_join_operator(&mut self, op: &'ast JoinOperator) {
        match op {
            JoinOperator::Inner(constraint) => self.visit_join_constraint(constraint),
            JoinOperator::LeftOuter(constraint) => self.visit_join_constraint(constraint),
            JoinOperator::RightOuter(constraint) => self.visit_join_constraint(constraint),
            JoinOperator::FullOuter(constraint) => self.visit_join_constraint(constraint),
            JoinOperator::Implicit | JoinOperator::Cross => (),
        }
    }

    fn visit_join_constraint(&mut self, constraint: &'ast JoinConstraint) {
        match constraint {
            JoinConstraint::On(expr) => self.visit_expr(expr),
            JoinConstraint::Using(idents) => {
                for ident in idents {
                    self.visit_identifier(ident);
                }
            },
            JoinConstraint::Natural => (),
        }
    }

    fn visit_where(&mut self, expr: &'ast ASTNode) {
        self.visit_expr(expr);
    }

    fn visit_group_by(&mut self, exprs: &'ast Vec<ASTNode>) {
        for expr in exprs {
            self.visit_expr(expr);
        }
    }

    fn visit_having(&mut self, expr: &'ast ASTNode) {
        self.visit_expr(expr);
    }

    fn visit_set_expr(&mut self, set_expr: &'ast SQLSetExpr) {
        match set_expr {
            SQLSetExpr::Select(select) => self.visit_select(select),
            SQLSetExpr::Query(query) => self.visit_query(query),
            SQLSetExpr::SetOperation { left, op, right, all } => self.visit_set_operation(left, op, right, *all),
        }
    }

    fn visit_set_operation(&mut self, left: &'ast SQLSetExpr, op: &'ast SQLSetOperator, right: &'ast SQLSetExpr, _all: bool) {
        self.visit_set_expr(left);
        self.visit_set_operator(op);
        self.visit_set_expr(right);
    }

    fn visit_set_operator(&mut self, _operator: &'ast SQLSetOperator) {}

    fn visit_order_by(&mut self, order_by: &'ast SQLOrderByExpr) {
        self.visit_expr(&order_by.expr);
    }

    fn visit_limit(&mut self, expr: &'ast ASTNode) {
        self.visit_expr(expr)
    }

    fn visit_type(&mut self, _data_type: &'ast SQLType) {}

    fn visit_expr(&mut self, expr: &'ast ASTNode) {
        match expr {
            ASTNode::SQLIdentifier(ident) => self.visit_identifier(ident),
            ASTNode::SQLWildcard => self.visit_wildcard(),
            ASTNode::SQLQualifiedWildcard(idents) => self.visit_qualified_wildcard(idents),
            ASTNode::SQLCompoundIdentifier(idents) => self.visit_compound_identifier(idents),
            ASTNode::SQLIsNull(expr) => self.visit_is_null(expr),
            ASTNode::SQLIsNotNull(expr) => self.visit_is_not_null(expr),
            ASTNode::SQLInList { expr, list, negated } => self.visit_in_list(expr, list, *negated),
            ASTNode::SQLInSubquery { expr, subquery, negated } => self.visit_in_subquery(expr, subquery, *negated),
            ASTNode::SQLBetween { expr, negated, low, high } => self.visit_between(expr, low, high, *negated),
            ASTNode::SQLBinaryExpr { left, op, right } => self.visit_binary_expr(left, op, right),
            ASTNode::SQLCast { expr, data_type } => self.visit_cast(expr, data_type),
            ASTNode::SQLNested(expr) => self.visit_nested(expr),
            ASTNode::SQLUnary { expr, operator } => self.visit_unary(expr, operator),
            ASTNode::SQLValue(val) => self.visit_value(val),
            ASTNode::SQLFunction { id, args } => self.visit_function(id, args),
            ASTNode::SQLCase { conditions, results, else_result } => self.visit_case(conditions, results, else_result.as_ref().map(|r| r.as_ref())),
            ASTNode::SQLSubquery(query) => self.visit_subquery(query),
        }
    }

    fn visit_unnamed_expression(&mut self, expr: &'ast ASTNode) {
        self.visit_expr(expr);
    }

    fn visit_expression_with_alias(&mut self, expr: &'ast ASTNode, alias: &'ast SQLIdent) {
        self.visit_expr(expr);
        self.visit_identifier(alias);
    }

    fn visit_object_name(&mut self, object_name: &'ast SQLObjectName) {
        for ident in &object_name.0 {
            self.visit_identifier(ident)
        }
    }

    fn visit_identifier(&mut self, _ident: &'ast SQLIdent) {}

    fn visit_compound_identifier(&mut self, idents: &'ast Vec<SQLIdent>) {
        for ident in idents {
            self.visit_identifier(ident);
        }
    }

    fn visit_wildcard(&mut self) {}

    fn visit_qualified_wildcard(&mut self, idents: &'ast Vec<SQLIdent>) {
        for ident in idents {
            self.visit_identifier(ident);
        }
    }

    fn visit_is_null(&mut self, expr: &'ast ASTNode) {
        self.visit_expr(expr);
    }

    fn visit_is_not_null(&mut self, expr: &'ast ASTNode) {
        self.visit_expr(expr);
    }

    fn visit_in_list(&mut self, expr: &'ast ASTNode, list: &'ast Vec<ASTNode>, _negated: bool) {
        self.visit_expr(expr);
        for e in list {
            self.visit_expr(e);
        }
    }

    fn visit_in_subquery(&mut self, expr: &'ast ASTNode, subquery: &'ast SQLQuery, _negated: bool) {
        self.visit_expr(expr);
        self.visit_query(subquery);
    }

    fn visit_between(&mut self, expr: &'ast ASTNode, low: &'ast ASTNode, high: &'ast ASTNode, _negated: bool) {
        self.visit_expr(expr);
        self.visit_expr(low);
        self.visit_expr(high);
    }

    fn visit_binary_expr(&mut self, left: &'ast ASTNode, op: &'ast SQLOperator, right: &'ast ASTNode) {
        self.visit_expr(left);
        self.visit_operator(op);
        self.visit_expr(right);
    }

    fn visit_operator(&mut self, _op: &'ast SQLOperator) {}

    fn visit_cast(&mut self, expr: &'ast ASTNode, data_type: &'ast SQLType) {
        self.visit_expr(expr);
        self.visit_type(data_type);
    }

    fn visit_nested(&mut self, expr: &'ast ASTNode) {
        self.visit_expr(expr);
    }

    fn visit_unary(&mut self, expr: &'ast ASTNode, op: &'ast SQLOperator) {
        self.visit_expr(expr);
        self.visit_operator(op);
    }

    fn visit_value(&mut self, _val: &'ast Value) {}

    fn visit_function(&mut self, ident: &'ast SQLIdent, args: &'ast Vec<ASTNode>) {
        self.visit_identifier(ident);
        for arg in args {
            self.visit_expr(arg);
        }
    }

    fn visit_case(&mut self, conditions: &'ast Vec<ASTNode>, results: &'ast Vec<ASTNode>, else_result: Option<&'ast ASTNode>) {
        for cond in conditions {
            self.visit_expr(cond);
        }
        for res in results {
            self.visit_expr(res);
        }
        match else_result {
            Some(expr) => self.visit_expr(expr),
            _ => (),
        }
    }

    fn visit_subquery(&mut self, subquery: &'ast SQLQuery) {
        self.visit_query(subquery)
    }

    fn visit_insert(&mut self, table_name: &'ast SQLObjectName, columns: &'ast Vec<SQLIdent>, values: &'ast Vec<Vec<ASTNode>>) {
        self.visit_object_name(table_name);
        for column in columns {
            self.visit_identifier(column);
        }
        self.visit_values_clause(values);
    }

    fn visit_values_clause(&mut self, rows: &'ast Vec<Vec<ASTNode>>) {
        for row in rows {
            self.visit_values_row(row)
        }
    }

    fn visit_values_row(&mut self, row: &'ast Vec<ASTNode>) {
        for expr in row {
            self.visit_expr(expr)
        }
    }

    fn visit_copy(&mut self, table_name: &'ast SQLObjectName, columns: &'ast Vec<SQLIdent>, values: &'ast Vec<Option<String>>) {
        self.visit_object_name(table_name);
        for column in columns {
            self.visit_identifier(column);
        }
        self.visit_copy_values(values);
    }

    fn visit_copy_values(&mut self, values: &'ast Vec<Option<String>>) {
        for value in values {
            self.visit_copy_values_row(value.as_ref());
        }
    }

    fn visit_copy_values_row(&mut self, _row: Option<&String>) {}

    fn visit_update(&mut self, table_name: &'ast SQLObjectName, assignments: &'ast Vec<SQLAssignment>, selection: Option<&'ast ASTNode>) {
        self.visit_object_name(&table_name);
        for assignment in assignments {
            self.visit_assignment(assignment);
        }
        match selection {
            Some(ref exprs) => self.visit_where(exprs),
            None => (),
        }
    }

    fn visit_assignment(&mut self, assignment: &'ast SQLAssignment) {
        self.visit_identifier(&assignment.id);
        self.visit_expr(&assignment.value);
    }

    fn visit_delete(&mut self, table_name: &'ast SQLObjectName, selection: Option<&'ast ASTNode>) {
        self.visit_object_name(table_name);
        match selection {
            Some(expr) => self.visit_where(expr),
            None => (),
        }
    }

    fn visit_create_data_source(&mut self, name: &'ast SQLObjectName, url: &'ast String, schema: &'ast String) {
        self.visit_object_name(name);
        self.visit_literal_string(url);
        self.visit_literal_string(schema);
    }

    fn visit_literal_string(&mut self, _string: &'ast String) {}

    fn visit_create_view(&mut self, name: &'ast SQLObjectName, query: &'ast SQLQuery, _materialized: bool) {
        self.visit_object_name(name);
        self.visit_query(&query);
    }

    fn visit_create_table(&mut self, name: &'ast SQLObjectName, columns: &'ast Vec<SQLColumnDef>) {
        self.visit_object_name(name);
        for column in columns {
            self.visit_column_def(column);
        }
    }

    fn visit_column_def(&mut self, column_def: &'ast SQLColumnDef) {
        self.visit_identifier(&column_def.name);
        self.visit_type(&column_def.data_type);
        self.visit_column_default(column_def.default.as_ref());
    }

    fn visit_column_default(&mut self, default: Option<&'ast ASTNode>) {
        match default {
            Some(expr) => self.visit_expr(expr),
            None => (),
        }
    }

    fn visit_alter_table(&mut self, name: &'ast SQLObjectName, operation: &'ast AlterOperation) {
        self.visit_object_name(name);
        self.visit_alter_operation(operation);
    }

    fn visit_alter_operation(&mut self, operation: &'ast AlterOperation) {
        match operation {
            AlterOperation::AddConstraint(table_key) => self.visit_alter_add_constraint(table_key),
            AlterOperation::RemoveConstraint { name } => self.visit_alter_remove_constraint(name),
        }
    }

    fn visit_alter_add_constraint(&mut self, table_key: &'ast TableKey) {
        self.visit_table_key(table_key);
    }

    fn visit_table_key(&mut self, table_key: &'ast TableKey) {
        match table_key {
            TableKey::PrimaryKey(key) => self.visit_primary_key(key),
            TableKey::UniqueKey(key) => self.visit_unique_key(key),
            TableKey::Key(key) => self.visit_key(key),
            TableKey::ForeignKey { key, foreign_table, referred_columns } => self.visit_foreign_key(key, foreign_table, referred_columns),
        }
    }

    fn visit_primary_key(&mut self, key: &'ast Key) {
        self.visit_key(key)
    }

    fn visit_unique_key(&mut self, key: &'ast Key) {
        self.visit_key(key)
    }

    fn visit_foreign_key(&mut self, key: &'ast Key, foreign_table: &'ast SQLObjectName, referred_columns: &'ast Vec<SQLIdent>) {
        self.visit_key(key);
        self.visit_object_name(foreign_table);
        for column in referred_columns {
            self.visit_identifier(column);
        }
    }


    fn visit_key(&mut self, key: &'ast Key) {
        self.visit_identifier(&key.name);
        for column in &key.columns {
            self.visit_identifier(column);
        }
    }

    fn visit_alter_remove_constraint(&mut self, name: &'ast SQLIdent) {
        self.visit_identifier(name);
    }
}

#[cfg(test)]
mod tests {
    use crate::sqlast::SQLIdent;
    use crate::sqlparser::Parser;
    use crate::dialect::GenericSqlDialect;
    use std::error::Error;
    use super::Visit;

    #[test]
    fn test_basic_visitor() -> Result<(), Box<dyn Error>> {
        struct Visitor<'a> {
            seen_idents: Vec<&'a SQLIdent>,
        }

        impl<'a> Visit<'a> for Visitor<'a> {
            fn visit_identifier(&mut self, ident: &'a SQLIdent) {
                self.seen_idents.push(ident);
            }
        }

        let stmts = Parser::parse_sql(&GenericSqlDialect {}, r#"
            SELECT *, foo.*, bar FROM baz JOIN zab ON baz.a = zab.b WHERE q;
            INSERT INTO db.bazzle (a, b, c) VALUES (1, 2, 3);
            DELETE FROM db2.razzle WHERE z = y AND Y = Z AND w BETWEEN 2 AND x;
"#.into())?;

        let mut visitor = Visitor { seen_idents: Vec::new() };
        for stmt in &stmts {
            visitor.visit_statement(stmt);
        }

        assert_eq!(visitor.seen_idents, &[
            "foo",
            "bar",
            "baz",
            "zab",
            "baz",
            "a",
            "zab",
            "b",
            "q",
            "db",
            "bazzle",
            "a",
            "b",
            "c",
            "db2",
            "razzle",
            "z",
            "y",
            "Y",
            "Z",
            "w",
            "x"
        ]);

        Ok(())
    }
}