// Copyright 2019 Timely Data, Inc. All rights reserved.
//
// This file may not be used or distributed without the express permission of
// Timely Data, Inc.

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
        visit_statement(self, statement)
    }

    fn visit_query(&mut self, query: &'ast SQLQuery) {
        visit_query(self, query)
    }

    fn visit_cte(&mut self, cte: &'ast Cte) {
        visit_cte(self, cte)
    }

    fn visit_select(&mut self, select: &'ast SQLSelect) {
        visit_select(self, select)
    }

    fn visit_select_item(&mut self, select_item: &'ast SQLSelectItem) {
        visit_select_item(self, select_item)
    }

    fn visit_table_factor(&mut self, table_factor: &'ast TableFactor) {
        visit_table_factor(self, table_factor)
    }

    fn visit_table_table_factor(
        &mut self,
        name: &'ast SQLObjectName,
        alias: Option<&'ast SQLIdent>,
        args: &'ast Vec<ASTNode>,
        with_hints: &'ast Vec<ASTNode>,
    ) {
        visit_table_table_factor(self, name, alias, args, with_hints)
    }

    fn visit_derived_table_factor(
        &mut self,
        subquery: &'ast SQLQuery,
        alias: Option<&'ast SQLIdent>,
    ) {
        visit_derived_table_factor(self, subquery, alias)
    }

    fn visit_join(&mut self, join: &'ast Join) {
        visit_join(self, join)
    }

    fn visit_join_operator(&mut self, op: &'ast JoinOperator) {
        visit_join_operator(self, op)
    }

    fn visit_join_constraint(&mut self, constraint: &'ast JoinConstraint) {
        visit_join_constraint(self, constraint)
    }

    fn visit_where(&mut self, expr: &'ast ASTNode) {
        visit_where(self, expr)
    }

    fn visit_group_by(&mut self, exprs: &'ast Vec<ASTNode>) {
        visit_group_by(self, exprs)
    }

    fn visit_having(&mut self, expr: &'ast ASTNode) {
        visit_having(self, expr)
    }

    fn visit_set_expr(&mut self, set_expr: &'ast SQLSetExpr) {
        visit_set_expr(self, set_expr)
    }

    fn visit_set_operation(
        &mut self,
        left: &'ast SQLSetExpr,
        op: &'ast SQLSetOperator,
        right: &'ast SQLSetExpr,
        all: bool,
    ) {
        visit_set_operation(self, left, op, right, all)
    }

    fn visit_set_operator(&mut self, _operator: &'ast SQLSetOperator) {}

    fn visit_order_by(&mut self, order_by: &'ast SQLOrderByExpr) {
        visit_order_by(self, order_by)
    }

    fn visit_limit(&mut self, expr: &'ast ASTNode) {
        visit_limit(self, expr)
    }

    fn visit_type(&mut self, _data_type: &'ast SQLType) {}

    fn visit_expr(&mut self, expr: &'ast ASTNode) {
        visit_expr(self, expr)
    }

    fn visit_unnamed_expression(&mut self, expr: &'ast ASTNode) {
        visit_unnamed_expression(self, expr)
    }

    fn visit_expression_with_alias(&mut self, expr: &'ast ASTNode, alias: &'ast SQLIdent) {
        visit_expression_with_alias(self, expr, alias)
    }

    fn visit_object_name(&mut self, object_name: &'ast SQLObjectName) {
        visit_object_name(self, object_name)
    }

    fn visit_identifier(&mut self, _ident: &'ast SQLIdent) {}

    fn visit_compound_identifier(&mut self, idents: &'ast Vec<SQLIdent>) {
        visit_compound_identifier(self, idents)
    }

    fn visit_wildcard(&mut self) {}

    fn visit_qualified_wildcard(&mut self, idents: &'ast Vec<SQLIdent>) {
        visit_qualified_wildcard(self, idents)
    }

    fn visit_is_null(&mut self, expr: &'ast ASTNode) {
        visit_is_null(self, expr)
    }

    fn visit_is_not_null(&mut self, expr: &'ast ASTNode) {
        visit_is_not_null(self, expr)
    }

    fn visit_in_list(&mut self, expr: &'ast ASTNode, list: &'ast Vec<ASTNode>, negated: bool) {
        visit_in_list(self, expr, list, negated)
    }

    fn visit_in_subquery(&mut self, expr: &'ast ASTNode, subquery: &'ast SQLQuery, negated: bool) {
        visit_in_subquery(self, expr, subquery, negated)
    }

    fn visit_between(
        &mut self,
        expr: &'ast ASTNode,
        low: &'ast ASTNode,
        high: &'ast ASTNode,
        negated: bool,
    ) {
        visit_between(self, expr, low, high, negated)
    }

    fn visit_binary_expr(
        &mut self,
        left: &'ast ASTNode,
        op: &'ast SQLOperator,
        right: &'ast ASTNode,
    ) {
        visit_binary_expr(self, left, op, right)
    }

    fn visit_operator(&mut self, _op: &'ast SQLOperator) {}

    fn visit_cast(&mut self, expr: &'ast ASTNode, data_type: &'ast SQLType) {
        visit_cast(self, expr, data_type)
    }

    fn visit_extract(&mut self, field: &'ast SQLDateTimeField, expr: &'ast ASTNode) {
        visit_extract(self, field, expr)
    }

    fn visit_date_time_field(&mut self, _field: &'ast SQLDateTimeField) {}

    fn visit_collate(&mut self, expr: &'ast ASTNode, collation: &'ast SQLObjectName) {
        visit_collate(self, expr, collation)
    }

    fn visit_nested(&mut self, expr: &'ast ASTNode) {
        visit_nested(self, expr)
    }

    fn visit_unary(&mut self, expr: &'ast ASTNode, op: &'ast SQLOperator) {
        visit_unary(self, expr, op)
    }

    fn visit_value(&mut self, _val: &'ast Value) {}

    fn visit_function(&mut self, func: &'ast SQLFunction) {
        visit_function(self, func)
    }

    fn visit_window_spec(&mut self, window_spec: &'ast SQLWindowSpec) {
        visit_window_spec(self, window_spec)
    }

    fn visit_window_frame(&mut self, window_frame: &'ast SQLWindowFrame) {
        visit_window_frame(self, window_frame)
    }

    fn visit_window_frame_units(&mut self, _window_frame_units: &'ast SQLWindowFrameUnits) {}

    fn visit_window_frame_bound(&mut self, _window_frame_bound: &'ast SQLWindowFrameBound) {}

    fn visit_case(
        &mut self,
        operand: Option<&'ast ASTNode>,
        conditions: &'ast Vec<ASTNode>,
        results: &'ast Vec<ASTNode>,
        else_result: Option<&'ast ASTNode>,
    ) {
        visit_case(self, operand, conditions, results, else_result)
    }

    fn visit_exists(&mut self, subquery: &'ast SQLQuery) {
        visit_exists(self, subquery)
    }

    fn visit_subquery(&mut self, subquery: &'ast SQLQuery) {
        visit_subquery(self, subquery)
    }

    fn visit_insert(
        &mut self,
        table_name: &'ast SQLObjectName,
        columns: &'ast Vec<SQLIdent>,
        source: &'ast SQLQuery,
    ) {
        visit_insert(self, table_name, columns, source)
    }

    fn visit_values(&mut self, rows: &'ast SQLValues) {
        visit_values(self, rows)
    }

    fn visit_values_row(&mut self, row: &'ast Vec<ASTNode>) {
        visit_values_row(self, row)
    }

    fn visit_copy(
        &mut self,
        table_name: &'ast SQLObjectName,
        columns: &'ast Vec<SQLIdent>,
        values: &'ast Vec<Option<String>>,
    ) {
        visit_copy(self, table_name, columns, values)
    }

    fn visit_copy_values(&mut self, values: &'ast Vec<Option<String>>) {
        visit_copy_values(self, values)
    }

    fn visit_copy_values_row(&mut self, _row: Option<&String>) {}

    fn visit_update(
        &mut self,
        table_name: &'ast SQLObjectName,
        assignments: &'ast Vec<SQLAssignment>,
        selection: Option<&'ast ASTNode>,
    ) {
        visit_update(self, table_name, assignments, selection)
    }

    fn visit_assignment(&mut self, assignment: &'ast SQLAssignment) {
        visit_assignment(self, assignment)
    }

    fn visit_delete(&mut self, table_name: &'ast SQLObjectName, selection: Option<&'ast ASTNode>) {
        visit_delete(self, table_name, selection)
    }

    fn visit_create_data_source(
        &mut self,
        name: &'ast SQLObjectName,
        url: &'ast String,
        schema: &'ast DataSourceSchema,
        with_options: &'ast Vec<SQLOption>,
    ) {
        visit_create_data_source(self, name, url, schema, with_options)
    }

    fn visit_data_source_schema(&mut self, data_source_schema: &'ast DataSourceSchema) {
        visit_data_source_schema(self, data_source_schema)
    }

    fn visit_create_data_sink(
        &mut self,
        name: &'ast SQLObjectName,
        from: &'ast SQLObjectName,
        url: &'ast String,
        with_options: &'ast Vec<SQLOption>,
    ) {
        visit_create_data_sink(self, name, from, url, with_options)
    }

    fn visit_literal_string(&mut self, _string: &'ast String) {}

    fn visit_create_view(
        &mut self,
        name: &'ast SQLObjectName,
        columns: &'ast Vec<SQLIdent>,
        query: &'ast SQLQuery,
        materialized: bool,
        with_options: &'ast Vec<SQLOption>,
    ) {
        visit_create_view(self, name, columns, query, materialized, with_options)
    }

    fn visit_create_table(
        &mut self,
        name: &'ast SQLObjectName,
        columns: &'ast Vec<SQLColumnDef>,
        constraints: &'ast Vec<SQLTableConstraint>,
        with_options: &'ast Vec<SQLOption>,
        external: bool,
        file_format: &'ast Option<FileFormat>,
        location: &'ast Option<String>,
    ) {
        visit_create_table(
            self,
            name,
            columns,
            constraints,
            with_options,
            external,
            file_format,
            location,
        )
    }

    fn visit_table_constraint(&mut self, constraint: &'ast SQLTableConstraint) {
        visit_table_constraint(self, constraint)
    }

    fn visit_column_def(&mut self, column_def: &'ast SQLColumnDef) {
        visit_column_def(self, column_def)
    }

    fn visit_column_constraint(&mut self, constraint: &'ast SQLColumnConstraint) {
        visit_column_constraint(self, constraint)
    }

    fn visit_file_format(&mut self, _file_format: &'ast FileFormat) {}

    fn visit_option(&mut self, option: &'ast SQLOption) {
        visit_option(self, option)
    }

    fn visit_drop(&mut self, drop: &'ast SQLDrop) {
        visit_drop(self, drop)
    }

    fn visit_alter_table(&mut self, name: &'ast SQLObjectName, operation: &'ast AlterOperation) {
        visit_alter_table(self, name, operation)
    }

    fn visit_alter_operation(&mut self, operation: &'ast AlterOperation) {
        visit_alter_operation(self, operation)
    }

    fn visit_alter_add_constraint(&mut self, table_key: &'ast TableKey) {
        visit_alter_add_constraint(self, table_key)
    }

    fn visit_table_key(&mut self, table_key: &'ast TableKey) {
        visit_table_key(self, table_key)
    }

    fn visit_primary_key(&mut self, key: &'ast Key) {
        visit_primary_key(self, key)
    }

    fn visit_unique_key(&mut self, key: &'ast Key) {
        visit_unique_key(self, key)
    }

    fn visit_foreign_key(
        &mut self,
        key: &'ast Key,
        foreign_table: &'ast SQLObjectName,
        referred_columns: &'ast Vec<SQLIdent>,
    ) {
        visit_foreign_key(self, key, foreign_table, referred_columns)
    }

    fn visit_key(&mut self, key: &'ast Key) {
        visit_key(self, key)
    }

    fn visit_alter_remove_constraint(&mut self, name: &'ast SQLIdent) {
        visit_alter_remove_constraint(self, name)
    }

    fn visit_peek(&mut self, name: &'ast SQLObjectName) {
        visit_peek(self, name)
    }

    fn visit_tail(&mut self, name: &'ast SQLObjectName) {
        visit_tail(self, name)
    }
}

pub fn visit_statement<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    statement: &'ast SQLStatement,
) {
    match statement {
        SQLStatement::SQLQuery(query) => visitor.visit_query(query),
        SQLStatement::SQLInsert {
            table_name,
            columns,
            source,
        } => visitor.visit_insert(table_name, columns, source),
        SQLStatement::SQLCopy {
            table_name,
            columns,
            values,
        } => visitor.visit_copy(table_name, columns, values),
        SQLStatement::SQLUpdate {
            table_name,
            assignments,
            selection,
        } => visitor.visit_update(table_name, assignments, selection.as_ref()),
        SQLStatement::SQLDelete {
            table_name,
            selection,
        } => visitor.visit_delete(table_name, selection.as_ref()),
        SQLStatement::SQLCreateDataSource {
            name,
            url,
            schema,
            with_options,
        } => visitor.visit_create_data_source(name, url, schema, with_options),
        SQLStatement::SQLCreateDataSink {
            name,
            from,
            url,
            with_options,
        } => visitor.visit_create_data_sink(name, from, url, with_options),
        SQLStatement::SQLCreateView {
            name,
            columns,
            query,
            materialized,
            with_options,
        } => visitor.visit_create_view(name, columns, query, *materialized, with_options),
        SQLStatement::SQLDropTable(drop) => visitor.visit_drop(drop),
        SQLStatement::SQLDropDataSource(drop) => visitor.visit_drop(drop),
        SQLStatement::SQLDropView(drop) => visitor.visit_drop(drop),
        SQLStatement::SQLCreateTable {
            name,
            columns,
            constraints,
            external,
            with_options,
            file_format,
            location,
        } => visitor.visit_create_table(
            name,
            columns,
            constraints,
            with_options,
            *external,
            file_format,
            location,
        ),
        SQLStatement::SQLAlterTable { name, operation } => {
            visitor.visit_alter_table(name, operation)
        }
        SQLStatement::SQLPeek { name } => {
            visitor.visit_peek(name);
        }
        SQLStatement::SQLTail { name } => {
            visitor.visit_tail(name);
        }
    }
}

pub fn visit_query<'ast, V: Visit<'ast> + ?Sized>(visitor: &mut V, query: &'ast SQLQuery) {
    for cte in &query.ctes {
        visitor.visit_cte(cte);
    }
    visitor.visit_set_expr(&query.body);
    for order_by in &query.order_by {
        visitor.visit_order_by(order_by);
    }
    match query.limit {
        Some(ref expr) => visitor.visit_limit(expr),
        None => (),
    }
}

pub fn visit_cte<'ast, V: Visit<'ast> + ?Sized>(visitor: &mut V, cte: &'ast Cte) {
    visitor.visit_identifier(&cte.alias);
    visitor.visit_query(&cte.query);
}

pub fn visit_select<'ast, V: Visit<'ast> + ?Sized>(visitor: &mut V, select: &'ast SQLSelect) {
    for select_item in &select.projection {
        visitor.visit_select_item(select_item)
    }
    match select.relation {
        Some(ref table_factor) => visitor.visit_table_factor(table_factor),
        None => (),
    }
    for join in &select.joins {
        visitor.visit_join(join);
    }
    match select.selection {
        Some(ref expr) => visitor.visit_where(expr),
        None => (),
    }
    if !select.group_by.is_empty() {
        visitor.visit_group_by(&select.group_by);
    }
    match select.having {
        Some(ref expr) => visitor.visit_having(expr),
        None => (),
    }
}

pub fn visit_select_item<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    select_item: &'ast SQLSelectItem,
) {
    match select_item {
        SQLSelectItem::UnnamedExpression(expr) => visitor.visit_unnamed_expression(expr),
        SQLSelectItem::ExpressionWithAlias { expr, alias } => {
            visitor.visit_expression_with_alias(expr, alias)
        }
        SQLSelectItem::QualifiedWildcard(object_name) => {
            visitor.visit_qualified_wildcard(&object_name.0)
        }
        SQLSelectItem::Wildcard => visitor.visit_wildcard(),
    }
}

pub fn visit_table_factor<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    table_factor: &'ast TableFactor,
) {
    match table_factor {
        TableFactor::Table {
            name,
            alias,
            args,
            with_hints,
        } => visitor.visit_table_table_factor(name, alias.as_ref(), args, with_hints),
        TableFactor::Derived { subquery, alias } => {
            visitor.visit_derived_table_factor(subquery, alias.as_ref())
        }
    }
}

pub fn visit_table_table_factor<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    name: &'ast SQLObjectName,
    alias: Option<&'ast SQLIdent>,
    args: &'ast Vec<ASTNode>,
    with_hints: &'ast Vec<ASTNode>,
) {
    visitor.visit_object_name(name);
    if let Some(ident) = alias {
        visitor.visit_identifier(ident);
    }
    for expr in args {
        visitor.visit_expr(expr);
    }
    for expr in with_hints {
        visitor.visit_expr(expr);
    }
}

pub fn visit_derived_table_factor<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    subquery: &'ast SQLQuery,
    alias: Option<&'ast SQLIdent>,
) {
    visitor.visit_subquery(subquery);
    match alias {
        Some(ident) => visitor.visit_identifier(ident),
        None => (),
    }
}

pub fn visit_join<'ast, V: Visit<'ast> + ?Sized>(visitor: &mut V, join: &'ast Join) {
    visitor.visit_table_factor(&join.relation);
    visitor.visit_join_operator(&join.join_operator);
}

pub fn visit_join_operator<'ast, V: Visit<'ast> + ?Sized>(visitor: &mut V, op: &'ast JoinOperator) {
    match op {
        JoinOperator::Inner(constraint) => visitor.visit_join_constraint(constraint),
        JoinOperator::LeftOuter(constraint) => visitor.visit_join_constraint(constraint),
        JoinOperator::RightOuter(constraint) => visitor.visit_join_constraint(constraint),
        JoinOperator::FullOuter(constraint) => visitor.visit_join_constraint(constraint),
        JoinOperator::Implicit | JoinOperator::Cross => (),
    }
}

pub fn visit_join_constraint<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    constraint: &'ast JoinConstraint,
) {
    match constraint {
        JoinConstraint::On(expr) => visitor.visit_expr(expr),
        JoinConstraint::Using(idents) => {
            for ident in idents {
                visitor.visit_identifier(ident);
            }
        }
        JoinConstraint::Natural => (),
    }
}

pub fn visit_where<'ast, V: Visit<'ast> + ?Sized>(visitor: &mut V, expr: &'ast ASTNode) {
    visitor.visit_expr(expr);
}

pub fn visit_group_by<'ast, V: Visit<'ast> + ?Sized>(visitor: &mut V, exprs: &'ast Vec<ASTNode>) {
    for expr in exprs {
        visitor.visit_expr(expr);
    }
}

pub fn visit_having<'ast, V: Visit<'ast> + ?Sized>(visitor: &mut V, expr: &'ast ASTNode) {
    visitor.visit_expr(expr);
}

pub fn visit_set_expr<'ast, V: Visit<'ast> + ?Sized>(visitor: &mut V, set_expr: &'ast SQLSetExpr) {
    match set_expr {
        SQLSetExpr::Select(select) => visitor.visit_select(select),
        SQLSetExpr::Query(query) => visitor.visit_query(query),
        SQLSetExpr::Values(values) => visitor.visit_values(values),
        SQLSetExpr::SetOperation {
            left,
            op,
            right,
            all,
        } => visitor.visit_set_operation(left, op, right, *all),
    }
}

pub fn visit_set_operation<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    left: &'ast SQLSetExpr,
    op: &'ast SQLSetOperator,
    right: &'ast SQLSetExpr,
    _all: bool,
) {
    visitor.visit_set_expr(left);
    visitor.visit_set_operator(op);
    visitor.visit_set_expr(right);
}

pub fn visit_order_by<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    order_by: &'ast SQLOrderByExpr,
) {
    visitor.visit_expr(&order_by.expr);
}

pub fn visit_limit<'ast, V: Visit<'ast> + ?Sized>(visitor: &mut V, expr: &'ast ASTNode) {
    visitor.visit_expr(expr)
}

pub fn visit_expr<'ast, V: Visit<'ast> + ?Sized>(visitor: &mut V, expr: &'ast ASTNode) {
    match expr {
        ASTNode::SQLIdentifier(ident) => visitor.visit_identifier(ident),
        ASTNode::SQLWildcard => visitor.visit_wildcard(),
        ASTNode::SQLQualifiedWildcard(idents) => visitor.visit_qualified_wildcard(idents),
        ASTNode::SQLCompoundIdentifier(idents) => visitor.visit_compound_identifier(idents),
        ASTNode::SQLIsNull(expr) => visitor.visit_is_null(expr),
        ASTNode::SQLIsNotNull(expr) => visitor.visit_is_not_null(expr),
        ASTNode::SQLInList {
            expr,
            list,
            negated,
        } => visitor.visit_in_list(expr, list, *negated),
        ASTNode::SQLInSubquery {
            expr,
            subquery,
            negated,
        } => visitor.visit_in_subquery(expr, subquery, *negated),
        ASTNode::SQLBetween {
            expr,
            negated,
            low,
            high,
        } => visitor.visit_between(expr, low, high, *negated),
        ASTNode::SQLBinaryExpr { left, op, right } => visitor.visit_binary_expr(left, op, right),
        ASTNode::SQLCast { expr, data_type } => visitor.visit_cast(expr, data_type),
        ASTNode::SQLExtract { field, expr } => visitor.visit_extract(field, expr),
        ASTNode::SQLCollate { expr, collation } => visitor.visit_collate(expr, collation),
        ASTNode::SQLNested(expr) => visitor.visit_nested(expr),
        ASTNode::SQLUnary { expr, operator } => visitor.visit_unary(expr, operator),
        ASTNode::SQLValue(val) => visitor.visit_value(val),
        ASTNode::SQLFunction(func) => visitor.visit_function(func),
        ASTNode::SQLCase {
            operand,
            conditions,
            results,
            else_result,
        } => visitor.visit_case(
            operand.as_ref().map(|o| o.as_ref()),
            conditions,
            results,
            else_result.as_ref().map(|r| r.as_ref()),
        ),
        ASTNode::SQLExists(query) => visitor.visit_exists(query),
        ASTNode::SQLSubquery(query) => visitor.visit_subquery(query),
    }
}

pub fn visit_unnamed_expression<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    expr: &'ast ASTNode,
) {
    visitor.visit_expr(expr);
}

pub fn visit_expression_with_alias<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    expr: &'ast ASTNode,
    alias: &'ast SQLIdent,
) {
    visitor.visit_expr(expr);
    visitor.visit_identifier(alias);
}

pub fn visit_object_name<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    object_name: &'ast SQLObjectName,
) {
    for ident in &object_name.0 {
        visitor.visit_identifier(ident)
    }
}

pub fn visit_compound_identifier<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    idents: &'ast Vec<SQLIdent>,
) {
    for ident in idents {
        visitor.visit_identifier(ident);
    }
}

pub fn visit_qualified_wildcard<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    idents: &'ast Vec<SQLIdent>,
) {
    for ident in idents {
        visitor.visit_identifier(ident);
    }
}

pub fn visit_is_null<'ast, V: Visit<'ast> + ?Sized>(visitor: &mut V, expr: &'ast ASTNode) {
    visitor.visit_expr(expr);
}

pub fn visit_is_not_null<'ast, V: Visit<'ast> + ?Sized>(visitor: &mut V, expr: &'ast ASTNode) {
    visitor.visit_expr(expr);
}

pub fn visit_in_list<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    expr: &'ast ASTNode,
    list: &'ast Vec<ASTNode>,
    _negated: bool,
) {
    visitor.visit_expr(expr);
    for e in list {
        visitor.visit_expr(e);
    }
}

pub fn visit_in_subquery<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    expr: &'ast ASTNode,
    subquery: &'ast SQLQuery,
    _negated: bool,
) {
    visitor.visit_expr(expr);
    visitor.visit_query(subquery);
}

pub fn visit_between<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    expr: &'ast ASTNode,
    low: &'ast ASTNode,
    high: &'ast ASTNode,
    _negated: bool,
) {
    visitor.visit_expr(expr);
    visitor.visit_expr(low);
    visitor.visit_expr(high);
}

pub fn visit_binary_expr<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    left: &'ast ASTNode,
    op: &'ast SQLOperator,
    right: &'ast ASTNode,
) {
    visitor.visit_expr(left);
    visitor.visit_operator(op);
    visitor.visit_expr(right);
}

pub fn visit_cast<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    expr: &'ast ASTNode,
    data_type: &'ast SQLType,
) {
    visitor.visit_expr(expr);
    visitor.visit_type(data_type);
}

pub fn visit_extract<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    field: &'ast SQLDateTimeField,
    expr: &'ast ASTNode,
) {
    visitor.visit_date_time_field(field);
    visitor.visit_expr(expr);
}

pub fn visit_collate<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    expr: &'ast ASTNode,
    collation: &'ast SQLObjectName,
) {
    visitor.visit_expr(expr);
    visitor.visit_object_name(collation);
}

pub fn visit_nested<'ast, V: Visit<'ast> + ?Sized>(visitor: &mut V, expr: &'ast ASTNode) {
    visitor.visit_expr(expr);
}

pub fn visit_unary<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    expr: &'ast ASTNode,
    op: &'ast SQLOperator,
) {
    visitor.visit_expr(expr);
    visitor.visit_operator(op);
}

pub fn visit_function<'ast, V: Visit<'ast> + ?Sized>(visitor: &mut V, func: &'ast SQLFunction) {
    visitor.visit_object_name(&func.name);
    for arg in &func.args {
        visitor.visit_expr(arg);
    }
    if let Some(over) = &func.over {
        visitor.visit_window_spec(over);
    }
}

pub fn visit_window_spec<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    window_spec: &'ast SQLWindowSpec,
) {
    for expr in &window_spec.partition_by {
        visitor.visit_expr(expr);
    }
    for order_by in &window_spec.order_by {
        visitor.visit_order_by(order_by);
    }
    if let Some(window_frame) = &window_spec.window_frame {
        visitor.visit_window_frame(window_frame);
    }
}

pub fn visit_window_frame<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    window_frame: &'ast SQLWindowFrame,
) {
    visitor.visit_window_frame_units(&window_frame.units);
    visitor.visit_window_frame_bound(&window_frame.start_bound);
    if let Some(end_bound) = &window_frame.end_bound {
        visitor.visit_window_frame_bound(end_bound);
    }
}

pub fn visit_case<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    operand: Option<&'ast ASTNode>,
    conditions: &'ast Vec<ASTNode>,
    results: &'ast Vec<ASTNode>,
    else_result: Option<&'ast ASTNode>,
) {
    if let Some(operand) = operand {
        visitor.visit_expr(operand);
    }
    for cond in conditions {
        visitor.visit_expr(cond);
    }
    for res in results {
        visitor.visit_expr(res);
    }
    match else_result {
        Some(expr) => visitor.visit_expr(expr),
        _ => (),
    }
}

pub fn visit_exists<'ast, V: Visit<'ast> + ?Sized>(visitor: &mut V, subquery: &'ast SQLQuery) {
    visitor.visit_query(subquery)
}

pub fn visit_subquery<'ast, V: Visit<'ast> + ?Sized>(visitor: &mut V, subquery: &'ast SQLQuery) {
    visitor.visit_query(subquery)
}

pub fn visit_insert<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    table_name: &'ast SQLObjectName,
    columns: &'ast Vec<SQLIdent>,
    source: &'ast SQLQuery,
) {
    visitor.visit_object_name(table_name);
    for column in columns {
        visitor.visit_identifier(column);
    }
    visitor.visit_query(source);
}

pub fn visit_values<'ast, V: Visit<'ast> + ?Sized>(visitor: &mut V, rows: &'ast SQLValues) {
    for row in &rows.0 {
        visitor.visit_values_row(row)
    }
}

pub fn visit_values_row<'ast, V: Visit<'ast> + ?Sized>(visitor: &mut V, row: &'ast Vec<ASTNode>) {
    for expr in row {
        visitor.visit_expr(expr)
    }
}

pub fn visit_copy<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    table_name: &'ast SQLObjectName,
    columns: &'ast Vec<SQLIdent>,
    values: &'ast Vec<Option<String>>,
) {
    visitor.visit_object_name(table_name);
    for column in columns {
        visitor.visit_identifier(column);
    }
    visitor.visit_copy_values(values);
}

pub fn visit_copy_values<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    values: &'ast Vec<Option<String>>,
) {
    for value in values {
        visitor.visit_copy_values_row(value.as_ref());
    }
}

pub fn visit_update<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    table_name: &'ast SQLObjectName,
    assignments: &'ast Vec<SQLAssignment>,
    selection: Option<&'ast ASTNode>,
) {
    visitor.visit_object_name(&table_name);
    for assignment in assignments {
        visitor.visit_assignment(assignment);
    }
    match selection {
        Some(ref exprs) => visitor.visit_where(exprs),
        None => (),
    }
}

pub fn visit_assignment<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    assignment: &'ast SQLAssignment,
) {
    visitor.visit_identifier(&assignment.id);
    visitor.visit_expr(&assignment.value);
}

pub fn visit_delete<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    table_name: &'ast SQLObjectName,
    selection: Option<&'ast ASTNode>,
) {
    visitor.visit_object_name(table_name);
    match selection {
        Some(expr) => visitor.visit_where(expr),
        None => (),
    }
}

pub fn visit_create_data_source<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    name: &'ast SQLObjectName,
    url: &'ast String,
    schema: &'ast DataSourceSchema,
    with_options: &'ast Vec<SQLOption>,
) {
    visitor.visit_object_name(name);
    visitor.visit_literal_string(url);
    visitor.visit_data_source_schema(schema);
    for option in with_options {
        visitor.visit_option(option);
    }
}

fn visit_data_source_schema<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    data_source_schema: &'ast DataSourceSchema,
) {
    match data_source_schema {
        DataSourceSchema::Raw(schema) => visitor.visit_literal_string(schema),
        DataSourceSchema::Registry(url) => visitor.visit_literal_string(url),
    }
}

pub fn visit_create_data_sink<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    name: &'ast SQLObjectName,
    from: &'ast SQLObjectName,
    url: &'ast String,
    with_options: &'ast Vec<SQLOption>,
) {
    visitor.visit_object_name(name);
    visitor.visit_object_name(from);
    visitor.visit_literal_string(url);
    for option in with_options {
        visitor.visit_option(option);
    }
}

pub fn visit_drop<'ast, V: Visit<'ast> + ?Sized>(visitor: &mut V, drop: &'ast SQLDrop) {
    for name in &drop.names {
        visitor.visit_object_name(name);
    }
}

pub fn visit_create_view<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    name: &'ast SQLObjectName,
    columns: &'ast Vec<SQLIdent>,
    query: &'ast SQLQuery,
    _materialized: bool,
    with_options: &'ast Vec<SQLOption>,
) {
    visitor.visit_object_name(name);
    for column in columns {
        visitor.visit_identifier(column);
    }
    visitor.visit_query(&query);
    for option in with_options {
        visitor.visit_option(option);
    }
}

pub fn visit_create_table<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    name: &'ast SQLObjectName,
    columns: &'ast Vec<SQLColumnDef>,
    constraints: &'ast Vec<SQLTableConstraint>,
    with_options: &'ast Vec<SQLOption>,
    _external: bool,
    file_format: &'ast Option<FileFormat>,
    location: &'ast Option<String>,
) {
    visitor.visit_object_name(name);
    for column in columns {
        visitor.visit_column_def(column);
    }
    for constraint in constraints {
        visitor.visit_table_constraint(constraint)
    }
    for option in with_options {
        visitor.visit_option(option);
    }
    if let Some(file_format) = file_format {
        visitor.visit_file_format(file_format);
    }
    if let Some(location) = location {
        visitor.visit_literal_string(location);
    }
}

pub fn visit_table_constraint<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    constraint: &'ast SQLTableConstraint,
) {
    use SQLTableConstraint::*;
    match constraint {
        Check { name, expr } => {
            if let Some(name) = name {
                visitor.visit_identifier(name);
            }
            visitor.visit_expr(expr)
        }
        Unique { name, columns } | PrimaryKey { name, columns } => {
            if let Some(name) = name {
                visitor.visit_identifier(name);
            }
            for column in columns {
                visitor.visit_identifier(column)
            }
        }
    }
}

pub fn visit_column_def<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    column_def: &'ast SQLColumnDef,
) {
    visitor.visit_identifier(&column_def.name);
    visitor.visit_type(&column_def.data_type);
    for constraint in &column_def.constraints {
        visitor.visit_column_constraint(constraint)
    }
}

pub fn visit_column_constraint<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    constraint: &'ast SQLColumnConstraint,
) {
    use SQLColumnConstraint::*;
    match constraint {
        Check { name, expr } => {
            if let Some(name) = name {
                visitor.visit_identifier(name);
            }
            visitor.visit_expr(expr);
        }
        Default(expr) => visitor.visit_expr(expr),
        Unique(Some(ident)) | PrimaryKey(Some(ident)) => visitor.visit_identifier(ident),
        Unique(None) | PrimaryKey(None) | Null | NotNull => (),
    }
}

pub fn visit_option<'ast, V: Visit<'ast> + ?Sized>(visitor: &mut V, option: &'ast SQLOption) {
    visitor.visit_identifier(&option.name);
    visitor.visit_value(&option.value);
}

pub fn visit_alter_table<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    name: &'ast SQLObjectName,
    operation: &'ast AlterOperation,
) {
    visitor.visit_object_name(name);
    visitor.visit_alter_operation(operation);
}

pub fn visit_alter_operation<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    operation: &'ast AlterOperation,
) {
    match operation {
        AlterOperation::AddConstraint(table_key) => visitor.visit_alter_add_constraint(table_key),
        AlterOperation::RemoveConstraint { name } => visitor.visit_alter_remove_constraint(name),
    }
}

pub fn visit_alter_add_constraint<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    table_key: &'ast TableKey,
) {
    visitor.visit_table_key(table_key);
}

pub fn visit_table_key<'ast, V: Visit<'ast> + ?Sized>(visitor: &mut V, table_key: &'ast TableKey) {
    match table_key {
        TableKey::PrimaryKey(key) => visitor.visit_primary_key(key),
        TableKey::UniqueKey(key) => visitor.visit_unique_key(key),
        TableKey::Key(key) => visitor.visit_key(key),
        TableKey::ForeignKey {
            key,
            foreign_table,
            referred_columns,
        } => visitor.visit_foreign_key(key, foreign_table, referred_columns),
    }
}

pub fn visit_primary_key<'ast, V: Visit<'ast> + ?Sized>(visitor: &mut V, key: &'ast Key) {
    visitor.visit_key(key)
}

pub fn visit_unique_key<'ast, V: Visit<'ast> + ?Sized>(visitor: &mut V, key: &'ast Key) {
    visitor.visit_key(key)
}

pub fn visit_foreign_key<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    key: &'ast Key,
    foreign_table: &'ast SQLObjectName,
    referred_columns: &'ast Vec<SQLIdent>,
) {
    visitor.visit_key(key);
    visitor.visit_object_name(foreign_table);
    for column in referred_columns {
        visitor.visit_identifier(column);
    }
}

pub fn visit_key<'ast, V: Visit<'ast> + ?Sized>(visitor: &mut V, key: &'ast Key) {
    visitor.visit_identifier(&key.name);
    for column in &key.columns {
        visitor.visit_identifier(column);
    }
}

pub fn visit_alter_remove_constraint<'ast, V: Visit<'ast> + ?Sized>(
    visitor: &mut V,
    name: &'ast SQLIdent,
) {
    visitor.visit_identifier(name);
}

pub fn visit_peek<'ast, V: Visit<'ast> + ?Sized>(visitor: &mut V, name: &'ast SQLObjectName) {
    visitor.visit_object_name(name);
}

pub fn visit_tail<'ast, V: Visit<'ast> + ?Sized>(visitor: &mut V, name: &'ast SQLObjectName) {
    visitor.visit_object_name(name);
}

#[cfg(test)]
mod tests {
    use super::Visit;
    use crate::dialect::GenericSqlDialect;
    use crate::sqlast::SQLIdent;
    use crate::sqlparser::Parser;
    use std::error::Error;

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

        let stmts = Parser::parse_sql(
            &GenericSqlDialect {},
            r#"
            SELECT *, foo.*, bar FROM baz JOIN zab ON baz.a = zab.b WHERE q;
            INSERT INTO db.bazzle (a, b, c) VALUES (1, 2, 3);
            DELETE FROM db2.razzle WHERE z = y AND Y = Z AND w BETWEEN 2 AND x;
"#
            .into(),
        )?;

        let mut visitor = Visitor {
            seen_idents: Vec::new(),
        };
        for stmt in &stmts {
            visitor.visit_statement(stmt);
        }

        assert_eq!(
            visitor.seen_idents,
            &[
                "foo", "bar", "baz", "zab", "baz", "a", "zab", "b", "q", "db", "bazzle", "a", "b",
                "c", "db2", "razzle", "z", "y", "Y", "Z", "w", "x"
            ]
        );

        Ok(())
    }
}
