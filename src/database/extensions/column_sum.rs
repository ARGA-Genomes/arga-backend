use std::marker::PhantomData;

use diesel::expression::{ValidGrouping, AsExpression};
use diesel::pg::Pg;
use diesel::query_builder::{QueryFragment, AstPass, QueryId};
use diesel::sql_types::{BigInt, SqlType, SingleValue};
use diesel::{QueryResult, Expression, DieselNumericOps, SelectableExpression, AppearsOnTable};


/// Count the amount of times the specified expression resolves true.
///
/// This expression builds a SUM(CASE..) select expression so that a single
/// query can make multiple counts through one iteration.
///
/// Example:
///   my_table.select((
///     sum_if(my_column.eq(true)),
///     sum_if(my_other_column.eq(true)),
///   )).load::<(i64, i64)>();
///
/// The result expression looks like the following:
///
///   SELECT
///     SUM(CASE WHEN my_column = true THEN 1 ELSE 0 END),
///     SUM(CASE WHEN my_other_column = true THEN 1 ELSE 0 END)
///   FROM my_table
///
pub fn sum_if<T, E>(expr: E) -> ColumnSum<T, E::Expression>
where
    T: SqlType + SingleValue,
    E: AsExpression<T>,
{
    ColumnSum {
        expr: expr.as_expression(),
        _marker: PhantomData,
    }
}


#[derive(Debug, Clone, Copy, QueryId, DieselNumericOps)]
pub struct ColumnSum<T, E> {
    expr: E,
    _marker: PhantomData<T>,
}

impl<T, E> QueryFragment<Pg> for ColumnSum<T, E>
where
    T: SqlType + SingleValue,
    E: QueryFragment<Pg>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Pg>) -> QueryResult<()> {
        out.push_sql("SUM(CASE WHEN ");
        self.expr.walk_ast(out.reborrow())?;
        out.push_sql(" THEN 1 ELSE 0 END)");
        Ok(())
    }
}


impl<T, E> Expression for ColumnSum<T, E>
where
    T: SqlType + SingleValue,
    E: Expression,
{
    type SqlType = BigInt;
}

impl<T, E, GB> ValidGrouping<GB> for ColumnSum<T, E>
where T: SqlType + SingleValue,
{
    type IsAggregate = diesel::expression::is_aggregate::Yes;
}

impl<T, E, QS> SelectableExpression<QS> for ColumnSum<T, E>
where
    Self: AppearsOnTable<QS>,
    E: SelectableExpression<QS>,
{
}

impl<T, E, QS> AppearsOnTable<QS> for ColumnSum<T, E>
where
    Self: Expression,
    E: AppearsOnTable<QS>,
{
}
