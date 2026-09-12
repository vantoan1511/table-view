use super::ddl::{pagination_clause, qualify_table, quote};
use super::OracleDriver;
use crate::drivers::{QueryResult, TableDataResult};
use serde_json::Value as JsonValue;
use std::collections::{HashMap, HashSet};

pub async fn fetch_table_data(
    driver: &OracleDriver,
    table_name: &str,
    limit: i64,
    offset: i64,
    sort_column: &str,
    sort_direction: &str,
    filter: &str,
) -> Result<TableDataResult, String> {
    let (owner, table) = driver.split_table_name(table_name);
    let qualified_table = qualify_table(owner, table);

    let pk_sql = "SELECT cols.COLUMN_NAME FROM ALL_CONSTRAINTS cons \
        JOIN ALL_CONS_COLUMNS cols ON cons.OWNER = cols.OWNER \
        AND cons.CONSTRAINT_NAME = cols.CONSTRAINT_NAME \
        WHERE cons.CONSTRAINT_TYPE = 'P' \
        AND cons.OWNER = :1 AND cons.TABLE_NAME = :2";
    let pk_params = [
        JsonValue::String(owner.to_string()),
        JsonValue::String(table.to_string()),
    ];

    let mut where_clause = String::new();
    if !filter.trim().is_empty() {
        where_clause = format!(" WHERE {}", filter);
    }

    let mut order_clause = String::new();
    if !sort_column.trim().is_empty() {
        let direction = if sort_direction.eq_ignore_ascii_case("desc") {
            "DESC"
        } else {
            "ASC"
        };
        order_clause = format!(" ORDER BY {} {}", quote(sort_column), direction);
    }

    let data_sql = format!(
        "SELECT * FROM {}{}{}{}",
        qualified_table,
        where_clause,
        order_clause,
        pagination_clause(limit, offset)
    );
    let count_sql = format!(
        "SELECT COUNT(*) AS CNT FROM {}{}",
        qualified_table, where_clause
    );

    let (pk_res, data_res, count_res) = tokio::join!(
        driver.run_query(pk_sql, &pk_params),
        driver.run_query(&data_sql, &[]),
        driver.run_query(&count_sql, &[]),
    );

    let (pk_rows, _, _) = pk_res?;
    let (data, mut fields, elapsed) = data_res?;
    let (count_rows, _, _) = count_res?;

    let pk_set: HashSet<String> = pk_rows
        .iter()
        .filter_map(|r| {
            r.get("COLUMN_NAME")
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })
        .collect();

    for field in &mut fields {
        field.is_primary_key = pk_set.contains(&field.name);
    }

    let total = count_rows
        .first()
        .and_then(|r| r.get("CNT"))
        .and_then(|v| v.as_i64().or_else(|| v.as_str()?.parse::<i64>().ok()))
        .unwrap_or(0);

    Ok(TableDataResult {
        rows: data,
        fields,
        total_count: total,
        execution_time: elapsed,
    })
}

pub async fn query(driver: &OracleDriver, sql: &str) -> Result<QueryResult, String> {
    let (rows, fields, elapsed) = driver.run_query(sql, &[]).await?;
    let row_count = rows.len();
    Ok(QueryResult {
        rows,
        fields,
        row_count,
        execution_time: elapsed,
    })
}

pub async fn update_cell(
    driver: &OracleDriver,
    table_name: &str,
    pk_column: &str,
    pk_value: &JsonValue,
    target_column: &str,
    new_value: &JsonValue,
) -> Result<(), String> {
    let (owner, table) = driver.split_table_name(table_name);
    let sql = format!(
        "UPDATE {} SET {} = :1 WHERE {} = :2",
        qualify_table(owner, table),
        quote(target_column),
        quote(pk_column),
    );
    driver
        .run_dml(&sql, &[new_value.clone(), pk_value.clone()])
        .await
}

pub async fn insert_row(
    driver: &OracleDriver,
    table_name: &str,
    data: &HashMap<String, JsonValue>,
) -> Result<HashMap<String, JsonValue>, String> {
    let (owner, table) = driver.split_table_name(table_name);
    let mut cols = Vec::new();
    let mut placeholders = Vec::new();
    let mut values = Vec::new();

    for (idx, (col, val)) in data.iter().enumerate() {
        cols.push(quote(col));
        placeholders.push(format!(":{}", idx + 1));
        values.push(val.clone());
    }

    let sql = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        qualify_table(owner, table),
        cols.join(", "),
        placeholders.join(", ")
    );

    driver.run_dml(&sql, &values).await?;
    Ok(data.clone())
}

pub async fn delete_rows(
    driver: &OracleDriver,
    table_name: &str,
    pk_column: &str,
    pk_values: &[JsonValue],
) -> Result<(), String> {
    if pk_values.is_empty() {
        return Ok(());
    }

    let (owner, table) = driver.split_table_name(table_name);
    let placeholders: Vec<String> = (1..=pk_values.len()).map(|i| format!(":{}", i)).collect();
    let sql = format!(
        "DELETE FROM {} WHERE {} IN ({})",
        qualify_table(owner, table),
        quote(pk_column),
        placeholders.join(", ")
    );

    driver.run_dml(&sql, pk_values).await
}

pub async fn export_to_csv(
    driver: &OracleDriver,
    table_name: &str,
    export_path: &str,
) -> Result<(), String> {
    let (owner, table) = driver.split_table_name(table_name);
    let sql = format!("SELECT * FROM {}", qualify_table(owner, table));
    let (rows, _, _) = driver.run_query(&sql, &[]).await?;

    crate::drivers::utils::export_rows_to_csv(&rows, export_path)
}
