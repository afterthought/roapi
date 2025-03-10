use datafusion::arrow;

use crate::error::QueryError;

pub async fn exec_query(
    dfctx: &datafusion::execution::context::SessionContext,
    sql: &str,
) -> Result<Vec<arrow::record_batch::RecordBatch>, QueryError> {
    let plan = dfctx
        .state()
        .create_logical_plan(sql)
        .await
        .map_err(QueryError::plan_sql)?;

    let df = datafusion::dataframe::DataFrame::new(dfctx.state(), plan);

    df.collect().await.map_err(QueryError::query_exec)
}

#[cfg(test)]
mod tests {
    use arrow::array::*;
    use datafusion::execution::context::SessionContext;

    use super::*;
    use crate::test_util::*;

    #[tokio::test]
    async fn group_by_aggregation() -> anyhow::Result<()> {
        let mut dfctx = SessionContext::new();
        register_table_properties(&mut dfctx)?;

        let batches = exec_query(
            &dfctx,
            r#"
            SELECT DISTINCT(landlord), COUNT(address)
            FROM properties
            GROUP BY landlord
            ORDER BY landlord
            "#,
        )
        .await?;

        let batch = &batches[0];

        assert_eq!(
            batch.column(0).as_ref(),
            &StringArray::from(vec!["Carl", "Daniel", "Mike", "Roger", "Sam",]),
        );

        assert_eq!(
            batch.column(1).as_ref(),
            &Int64Array::from(vec![3, 3, 4, 3, 2]),
        );

        Ok(())
    }
}
