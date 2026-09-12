use super::convert::pg_rows_to_maps;
use super::ddl::{qualified_table_name, quote, split_table_name};
use crate::bind_json_value;
use crate::drivers::{ColumnInfo, QueryResult, TableDataResult};
use futures_util::StreamExt;
use serde_json::Value;
use sqlx::postgres::PgPool;
use sqlx::{Either, Executor};
use std::collections::HashMap;
use tokio::sync::Mutex;

pub async fn execute_query(
    pool: &PgPool,
    sql: &str,
    params: &[Value],
) -> Result<(Vec<HashMap<String, Value>>, Vec<ColumnInfo>, u64), String> {
    let mut q = sqlx::query(sql);
    for p in params {
        q = bind_json_value!(q, p);
    }
    log::info!("postgres: executing query: {}", sql);
    let start = std::time::Instant::now();
    let rows = q.fetch_all(pool).await.map_err(|e| e.to_string())?;
    let elapsed = start.elapsed().as_millis() as u64;
    log::info!("postgres: query finished in {}ms", elapsed);

    if rows.is_empty() {
        return Ok((vec![], vec![], elapsed));
    }

    let (data, fields) = pg_rows_to_maps(&rows);
    Ok((data, fields, elapsed))
}

pub async fn fetch_table_data(
    pool: &PgPool,
    table_name: &str,
    limit: i64,
    offset: i64,
    sort_column: &str,
    sort_direction: &str,
    filter: &str,
) -> Result<TableDataResult, String> {
    let (schema_name, bare_table_name) = split_table_name(table_name);
    let safe_table = qualified_table_name(table_name);

    let column_sql = "SELECT \
            c.column_name::text, \
            c.udt_name::text as data_type, \
            (c.is_nullable = 'YES') as is_nullable, \
            EXISTS ( \
                SELECT 1 \
                FROM information_schema.table_constraints tc \
                JOIN information_schema.key_column_usage kcu \
                    ON tc.constraint_name = kcu.constraint_name \
                    AND tc.table_schema = kcu.table_schema \
                    AND tc.table_name = kcu.table_name \
                WHERE tc.constraint_type = 'PRIMARY KEY' \
                    AND tc.table_schema = c.table_schema \
                    AND tc.table_name = c.table_name \
                    AND kcu.column_name = c.column_name \
            ) as is_primary_key \
        FROM information_schema.columns c \
        WHERE c.table_schema::text = $1::text AND c.table_name::text = $2::text \
        ORDER BY c.ordinal_position";

    let where_clause = if !filter.trim().is_empty() {
        format!(" WHERE {}", filter.trim())
    } else {
        String::new()
    };

    let order_clause = if !sort_column.is_empty() {
        let dir = if sort_direction.eq_ignore_ascii_case("desc") {
            "DESC"
        } else {
            "ASC"
        };
        format!(" ORDER BY {} {}", quote(sort_column), dir)
    } else {
        String::new()
    };

    let data_sql = format!(
        "SELECT * FROM {}{}{} LIMIT $1 OFFSET $2",
        safe_table, where_clause, order_clause
    );
    let count_sql = format!("SELECT COUNT(*) FROM {}{}", safe_table, where_clause);

    let column_params = [
        Value::String(schema_name.to_string()),
        Value::String(bare_table_name.to_string()),
    ];
    let data_params = [Value::Number(limit.into()), Value::Number(offset.into())];
    let count_params = [];

    // Run all three queries in parallel
    let (column_res, data_res, count_res) = tokio::join!(
        execute_query(pool, column_sql, &column_params),
        execute_query(pool, &data_sql, &data_params),
        execute_query(pool, &count_sql, &count_params)
    );

    let (column_rows, _, _) = column_res?;
    let (data, mut fields, elapsed) = data_res?;
    let (count_rows, _, _) = count_res?;

    let column_meta: Vec<ColumnInfo> = column_rows
        .iter()
        .map(|r| ColumnInfo {
            name: r
                .get("column_name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            data_type: r
                .get("data_type")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            is_primary_key: r
                .get("is_primary_key")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            is_nullable: r
                .get("is_nullable")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            display_name: None,
        })
        .collect();

    if !column_meta.is_empty() {
        let metadata_by_name: std::collections::HashMap<String, ColumnInfo> = column_meta
            .iter()
            .map(|column| (column.name.clone(), column.clone()))
            .collect();

        if fields.is_empty() {
            fields = column_meta;
        } else {
            for field in &mut fields {
                if let Some(metadata) = metadata_by_name.get(&field.name) {
                    field.is_primary_key = metadata.is_primary_key;
                    field.is_nullable = metadata.is_nullable;
                    field.data_type = metadata.data_type.clone();
                }
            }
        }
    }

    let total = count_rows
        .first()
        .and_then(|r| r.values().next())
        .and_then(|v| {
            if let Value::Number(n) = v {
                n.as_i64()
            } else if let Value::String(s) = v {
                s.parse::<i64>().ok()
            } else {
                None
            }
        })
        .unwrap_or(0);

    Ok(TableDataResult {
        rows: data,
        fields,
        total_count: total,
        execution_time: elapsed,
    })
}

pub async fn query(
    pool: &PgPool,
    tx_conn: &Mutex<Option<sqlx::pool::PoolConnection<sqlx::Postgres>>>,
    sql: &str,
) -> Result<QueryResult, String> {
    let start = std::time::Instant::now();
    log::info!("postgres: executing raw query: {}", sql);

    let mut rows = Vec::new();
    let mut rows_affected = 0;

    let mut tx_guard = tx_conn.lock().await;
    let mut stream = if let Some(ref mut conn) = *tx_guard {
        (&mut **conn).fetch_many(sqlx::raw_sql(sql))
    } else {
        pool.fetch_many(sqlx::raw_sql(sql))
    };

    while let Some(res) = stream.next().await {
        match res.map_err(|e| e.to_string())? {
            Either::Left(result) => {
                rows_affected += result.rows_affected();
            }
            Either::Right(row) => {
                rows.push(row);
            }
        }
    }

    let elapsed = start.elapsed().as_millis() as u64;
    log::info!("postgres: raw query finished in {}ms", elapsed);

    if rows.is_empty() {
        return Ok(QueryResult {
            rows: vec![],
            fields: vec![],
            row_count: rows_affected as usize,
            execution_time: elapsed,
        });
    }

    let (data, fields) = pg_rows_to_maps(&rows);
    let count = data.len();
    Ok(QueryResult {
        rows: data,
        fields,
        row_count: count,
        execution_time: elapsed,
    })
}

pub async fn update_cell(
    pool: &PgPool,
    table_name: &str,
    pk_column: &str,
    pk_value: &Value,
    target_column: &str,
    new_value: &Value,
) -> Result<(), String> {
    let safe_table = qualified_table_name(table_name);
    let sql = format!(
        "UPDATE {} SET {} = $1 WHERE {}::text = $2::text",
        safe_table,
        quote(target_column),
        quote(pk_column)
    );
    let mut q = sqlx::query(&sql);
    q = bind_json_value!(q, new_value);
    q = bind_json_value!(q, pk_value);
    q.execute(pool).await.map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn insert_row(
    pool: &PgPool,
    table_name: &str,
    data: &HashMap<String, Value>,
) -> Result<HashMap<String, Value>, String> {
    let safe_table = qualified_table_name(table_name);

    if data.is_empty() {
        let sql = format!("INSERT INTO {} DEFAULT VALUES RETURNING *", safe_table);
        let (rows, _, _) = execute_query(pool, &sql, &[]).await?;
        return rows.into_iter().next().ok_or("No row returned".to_string());
    }

    let cols: Vec<String> = data.keys().map(|k| quote(k)).collect();
    let placeholders: Vec<String> = (1..=data.len()).map(|i| format!("${}", i)).collect();
    let values: Vec<Value> = data.values().cloned().collect();

    let sql = format!(
        "INSERT INTO {} ({}) VALUES ({}) RETURNING *",
        safe_table,
        cols.join(", "),
        placeholders.join(", ")
    );

    let (rows, _, _) = execute_query(pool, &sql, &values).await?;
    rows.into_iter().next().ok_or("No row returned".to_string())
}

pub async fn delete_rows(
    pool: &PgPool,
    table_name: &str,
    pk_column: &str,
    pk_values: &[Value],
) -> Result<(), String> {
    let safe_table = qualified_table_name(table_name);
    let placeholders: Vec<String> = (1..=pk_values.len())
        .map(|i| format!("${}::text", i))
        .collect();
    let sql = format!(
        "DELETE FROM {} WHERE {}::text IN ({})",
        safe_table,
        quote(pk_column),
        placeholders.join(", ")
    );
    let mut q = sqlx::query(&sql);
    for v in pk_values {
        q = bind_json_value!(q, v);
    }
    q.execute(pool).await.map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn export_to_csv(
    pool: &PgPool,
    table_name: &str,
    export_path: &str,
) -> Result<(), String> {
    let safe_table = qualified_table_name(table_name);
    let sql = format!("SELECT * FROM {}", safe_table);

    let (rows, _, _) = execute_query(pool, &sql, &[]).await?;
    crate::drivers::utils::export_rows_to_csv(&rows, export_path)
}
