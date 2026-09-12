use super::OracleDriver;
use crate::drivers::{AlterOperation, TableColumn};

pub fn quote(ident: &str) -> String {
    format!("\"{}\"", ident.replace('"', "\"\""))
}

pub fn qualify_table(owner: &str, table_name: &str) -> String {
    if owner.is_empty() || owner.eq_ignore_ascii_case("default") {
        quote(table_name)
    } else {
        format!("{}.{}", quote(owner), quote(table_name))
    }
}

pub fn pagination_clause(limit: i64, offset: i64) -> String {
    format!(
        " OFFSET {} ROWS FETCH NEXT {} ROWS ONLY",
        offset.max(0),
        limit.max(0)
    )
}

pub async fn create_table(
    driver: &OracleDriver,
    table_name: &str,
    columns: &[TableColumn],
) -> Result<(), String> {
    let (owner, table) = driver.split_table_name(table_name);
    let safe_table = if owner.is_empty()
        || owner.eq_ignore_ascii_case("default")
        || owner.eq_ignore_ascii_case(driver.current_schema())
    {
        quote(table)
    } else {
        qualify_table(owner, table)
    };

    let sql = crate::drivers::utils::build_create_table_sql_generic(&safe_table, columns, quote)?;
    driver.run_dml(&sql, &[]).await
}

pub async fn alter_table(
    driver: &OracleDriver,
    table_name: &str,
    operations: &[AlterOperation],
) -> Result<(), String> {
    let (owner, table) = driver.split_table_name(table_name);
    let safe_table = qualify_table(owner, table);

    for op in operations {
        let sql = match op.op_type.as_str() {
            "ADD_COLUMN" => {
                if !crate::drivers::utils::is_safe_data_type(&op.data_type) {
                    return Err(format!("Invalid or unsafe data type: {}", op.data_type));
                }
                let mut q = format!(
                    "ALTER TABLE {} ADD {} {}",
                    safe_table,
                    quote(&op.name),
                    op.data_type
                );
                if let Some(ref d) = op.default {
                    let d_str = d.to_string();
                    if !crate::drivers::utils::is_safe_default(&d_str) {
                        return Err(format!("Invalid or unsafe default value: {}", d_str));
                    }
                    q.push_str(&format!(" DEFAULT {}", d_str));
                }
                if op.nullable == Some(false) {
                    q.push_str(" NOT NULL");
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
            "DROP_COLUMN" => format!("ALTER TABLE {} DROP COLUMN {}", safe_table, quote(&op.name)),
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

        driver.run_dml(&sql, &[]).await?;
    }
    Ok(())
}

pub async fn drop_table(
    driver: &OracleDriver,
    table_name: &str,
    cascade: bool,
) -> Result<(), String> {
    let (owner, table) = driver.split_table_name(table_name);
    let sql = if cascade {
        format!(
            "DROP TABLE {} CASCADE CONSTRAINTS",
            qualify_table(owner, table)
        )
    } else {
        format!("DROP TABLE {}", qualify_table(owner, table))
    };
    driver.run_dml(&sql, &[]).await
}

pub async fn create_schema(driver: &OracleDriver, schema_name: &str) -> Result<(), String> {
    let sql = format!(
        "CREATE USER {} IDENTIFIED BY {}",
        quote(schema_name),
        quote(schema_name)
    );
    driver.run_dml(&sql, &[]).await
}

pub async fn drop_schema(driver: &OracleDriver, schema_name: &str) -> Result<(), String> {
    let sql = format!("DROP USER {} CASCADE", quote(schema_name));
    driver.run_dml(&sql, &[]).await
}

pub async fn drop_database(driver: &OracleDriver, db_name: &str) -> Result<(), String> {
    let sql = format!("DROP USER {} CASCADE", quote(db_name));
    driver.run_dml(&sql, &[]).await
}

pub async fn create_database(
    driver: &OracleDriver,
    db_name: &str,
    password: Option<&str>,
) -> Result<(), String> {
    let pass = password.unwrap_or(db_name);
    let sql = format!(
        "CREATE USER {} IDENTIFIED BY \"{}\"",
        quote(db_name),
        pass.replace('"', "\"\"")
    );
    driver.run_dml(&sql, &[]).await?;
    let grant_sql = format!("GRANT CONNECT, RESOURCE TO {}", quote(db_name));
    driver.run_dml(&grant_sql, &[]).await
}
