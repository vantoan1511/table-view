use crate::drivers::{AlterOperation, TableColumn};
use sqlx::postgres::PgPool;

pub fn quote(ident: &str) -> String {
    format!("\"{}\"", ident.replace('"', "\"\""))
}

pub fn split_table_name(table_name: &str) -> (&str, &str) {
    table_name
        .rsplit_once('.')
        .unwrap_or(("public", table_name))
}

pub fn qualified_table_name(table_name: &str) -> String {
    let (schema, table) = split_table_name(table_name);
    format!("{}.{}", quote(schema), quote(table))
}

pub fn build_drop_table_sql(table_name: &str, cascade: bool) -> String {
    let safe_table = qualified_table_name(table_name);
    if cascade {
        format!("DROP TABLE {} CASCADE", safe_table)
    } else {
        format!("DROP TABLE {}", safe_table)
    }
}

pub async fn alter_table(
    pool: &PgPool,
    table_name: &str,
    operations: &[AlterOperation],
) -> Result<(), String> {
    let safe_table = qualified_table_name(table_name);

    for op in operations {
        let sql = match op.op_type.as_str() {
            "ADD_COLUMN" => {
                if !crate::drivers::utils::is_safe_data_type(&op.data_type) {
                    return Err(format!("Invalid or unsafe data type: {}", op.data_type));
                }
                let mut q = format!(
                    "ALTER TABLE {} ADD COLUMN {} {}",
                    safe_table,
                    quote(&op.name),
                    op.data_type
                );
                if op.nullable == Some(false) {
                    q.push_str(" NOT NULL");
                }
                if let Some(ref d) = op.default {
                    let d_str = d.to_string();
                    if !crate::drivers::utils::is_safe_default(&d_str) {
                        return Err(format!("Invalid or unsafe default value: {}", d_str));
                    }
                    q.push_str(&format!(" DEFAULT {}", d_str));
                }
                if let Some(ref fk) = op.foreign_key {
                    let quoted_target_table = if fk.target_table.contains('.') {
                        fk.target_table
                            .split('.')
                            .map(quote)
                            .collect::<Vec<String>>()
                            .join(".")
                    } else {
                        quote(&fk.target_table)
                    };
                    q.push_str(&format!(
                        " REFERENCES {} ({})",
                        quoted_target_table,
                        quote(&fk.target_column)
                    ));
                }
                q
            }
            "DROP_COLUMN" => format!(
                "ALTER TABLE {} DROP COLUMN {}",
                safe_table,
                quote(&op.name)
            ),
            "RENAME_COLUMN" => format!(
                "ALTER TABLE {} RENAME COLUMN {} TO {}",
                safe_table,
                quote(&op.old_name),
                quote(&op.new_name)
            ),
            "DROP_CONSTRAINT" => {
                let constraint = op
                    .constraint_name
                    .as_ref()
                    .ok_or("constraint_name is required")?;
                format!(
                    "ALTER TABLE {} DROP CONSTRAINT {}",
                    safe_table,
                    quote(constraint)
                )
            }
            "ADD_FOREIGN_KEY" => {
                let fk = op.foreign_key.as_ref().ok_or("foreignKey is required")?;
                let fk_name = format!("fk_{}_{}", table_name, op.name);
                let quoted_target_table = if fk.target_table.contains('.') {
                    fk.target_table
                        .split('.')
                        .map(quote)
                        .collect::<Vec<String>>()
                        .join(".")
                } else {
                    quote(&fk.target_table)
                };
                format!(
                    "ALTER TABLE {} ADD CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {} ({})",
                    safe_table,
                    quote(&fk_name),
                    quote(&op.name),
                    quoted_target_table,
                    quote(&fk.target_column)
                )
            }
            "ADD_CONSTRAINT" => {
                let definition = op
                    .definition
                    .as_ref()
                    .ok_or("definition is required for ADD_CONSTRAINT")?;
                format!(
                    "ALTER TABLE {} ADD CONSTRAINT {} {}",
                    safe_table,
                    quote(&op.name),
                    definition
                )
            }
            _ => continue,
        };

        sqlx::query(&sql)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub async fn create_table(
    pool: &PgPool,
    table_name: &str,
    columns: &[TableColumn],
) -> Result<(), String> {
    let safe_table = qualified_table_name(table_name);
    let sql = crate::drivers::utils::build_create_table_sql_generic(&safe_table, columns, quote)?;
    sqlx::query(&sql)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn drop_table(pool: &PgPool, table_name: &str, cascade: bool) -> Result<(), String> {
    let sql = build_drop_table_sql(table_name, cascade);
    sqlx::query(&sql)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn create_schema(pool: &PgPool, schema_name: &str) -> Result<(), String> {
    let sql = format!("CREATE SCHEMA {}", quote(schema_name));
    sqlx::query(&sql)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn drop_schema(pool: &PgPool, schema_name: &str) -> Result<(), String> {
    let sql = format!("DROP SCHEMA {} CASCADE", quote(schema_name));
    sqlx::query(&sql)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn drop_database(pool: &PgPool, db_name: &str) -> Result<(), String> {
    let sql = format!("DROP DATABASE {}", quote(db_name));
    sqlx::query(&sql)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn create_database(
    pool: &PgPool,
    db_name: &str,
    _password: Option<&str>,
) -> Result<(), String> {
    let sql = format!("CREATE DATABASE {}", quote(db_name));
    sqlx::query(&sql)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
