pub mod mysql;
pub mod oracle;
pub mod postgres;
pub mod sqlite;
pub mod utils;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

// ─── Config ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(rename = "type", default)]
    pub db_type: String,
    #[serde(default)]
    pub host: String,
    #[serde(default)]
    pub port: u16,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub database: String,
    #[serde(default)]
    pub ssl: bool,
    #[serde(rename = "connectionTimeout", default = "default_timeout")]
    pub connection_timeout: u32,
    #[serde(rename = "oracleConnectType", default = "default_oracle_connect_type")]
    pub oracle_connect_type: String,
    #[serde(rename = "oracleRole", default = "default_oracle_role")]
    pub oracle_role: String,
    #[serde(rename = "displayAllDatabases", default)]
    pub display_all_databases: bool,
}

fn default_timeout() -> u32 {
    30
}

fn default_oracle_connect_type() -> String {
    "serviceName".to_string()
}

fn default_oracle_role() -> String {
    "normal".to_string()
}

// ─── Result Types ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct ColumnInfo {
    pub name: String,
    #[serde(rename = "dataTypeID")]
    pub data_type: String,
    #[serde(rename = "isPrimaryKey")]
    pub is_primary_key: bool,
    #[serde(rename = "isNullable")]
    pub is_nullable: bool,
    #[serde(rename = "displayName", skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct QueryResult {
    pub rows: Vec<HashMap<String, Value>>,
    pub fields: Vec<ColumnInfo>,
    #[serde(rename = "rowCount")]
    pub row_count: usize,
    #[serde(rename = "executionTime")]
    pub execution_time: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TableDataResult {
    pub rows: Vec<HashMap<String, Value>>,
    pub fields: Vec<ColumnInfo>,
    #[serde(rename = "totalCount")]
    pub total_count: i64,
    #[serde(rename = "executionTime")]
    pub execution_time: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SchemaObject {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub obj_type: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SchemaResult {
    pub tables: Vec<SchemaObject>,
    pub views: Vec<SchemaObject>,
    pub functions: Vec<SchemaObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schemas: Option<Vec<SchemaObject>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub databases: Option<Vec<SchemaObject>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKeyDef {
    #[serde(rename = "targetTable")]
    pub target_table: String,
    #[serde(rename = "targetColumn")]
    pub target_column: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableColumn {
    pub name: String,
    #[serde(rename = "dataType")]
    pub data_type: String,
    pub nullable: bool,
    #[serde(rename = "isPrimaryKey")]
    pub is_primary_key: bool,
    pub default: Option<String>,
    #[serde(rename = "foreignKey", default)]
    pub foreign_key: Option<ForeignKeyDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRelation {
    #[serde(rename = "sourceTable")]
    pub source_table: String,
    #[serde(rename = "sourceColumn")]
    pub source_column: String,
    #[serde(rename = "targetTable")]
    pub target_table: String,
    #[serde(rename = "targetColumn")]
    pub target_column: String,
    #[serde(rename = "constraintName")]
    pub constraint_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbIndex {
    pub name: String,
    pub columns: Vec<String>,
    #[serde(rename = "isUnique")]
    pub is_unique: bool,
    #[serde(rename = "isPrimaryKey")]
    pub is_primary_key: bool,
    #[serde(rename = "indexType", skip_serializing_if = "Option::is_none")]
    pub index_type: Option<String>,
    #[serde(rename = "ddl", skip_serializing_if = "Option::is_none")]
    pub ddl: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableConstraint {
    pub name: String,
    #[serde(rename = "constraintType")]
    pub constraint_type: String, // PRIMARY KEY, FOREIGN KEY, UNIQUE, CHECK
    pub definition: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<TableColumn>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDetails {
    pub tables: Vec<TableInfo>,
    pub relations: Vec<TableRelation>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AlterOperation {
    #[serde(rename = "type")]
    pub op_type: String, // e.g. "ADD_COLUMN", "DROP_COLUMN", "RENAME_COLUMN", "DROP_CONSTRAINT", "ADD_FOREIGN_KEY"
    #[serde(default)]
    pub name: String,
    #[serde(rename = "oldName", default)]
    pub old_name: String,
    #[serde(rename = "newName", default)]
    pub new_name: String,
    #[serde(rename = "dataType", default)]
    pub data_type: String,
    pub nullable: Option<bool>,
    pub default: Option<Value>,
    #[serde(rename = "foreignKey", default)]
    pub foreign_key: Option<ForeignKeyDef>,
    // Note: constraint_name and these fields are kept for future extensions where existing constraints
    // (such as foreign keys and primary keys) can be altered or dropped directly from the UI.
    #[serde(rename = "constraintName", default)]
    pub constraint_name: Option<String>,
    #[serde(default)]
    pub definition: Option<String>,
}

// ─── Driver Trait ───────────────────────────────────────────────────────────

#[async_trait]
pub trait DatabaseDriver: Send + Sync {
    async fn connect(&mut self, config: &Config) -> Result<(), String>;
    async fn disconnect(&mut self) -> Result<(), String>;
    async fn get_schema(
        &self,
        all_databases: bool,
        schema_name: Option<&str>,
    ) -> Result<SchemaResult, String>;
    async fn fetch_table_data(
        &self,
        table_name: &str,
        limit: i64,
        offset: i64,
        sort_column: &str,
        sort_direction: &str,
        filter: &str,
    ) -> Result<TableDataResult, String>;
    async fn query(&self, sql: &str) -> Result<QueryResult, String>;
    async fn update_cell(
        &self,
        table_name: &str,
        pk_column: &str,
        pk_value: &Value,
        target_column: &str,
        new_value: &Value,
    ) -> Result<(), String>;
    async fn insert_row(
        &self,
        table_name: &str,
        data: &HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, String>;
    async fn delete_rows(
        &self,
        table_name: &str,
        pk_column: &str,
        pk_values: &[Value],
    ) -> Result<(), String>;
    async fn get_table_columns(&self, table_name: &str) -> Result<Vec<TableColumn>, String>;
    async fn alter_table(
        &self,
        table_name: &str,
        operations: &[AlterOperation],
    ) -> Result<(), String>;
    async fn create_table(&self, table_name: &str, columns: &[TableColumn]) -> Result<(), String>;
    async fn drop_table(&self, table_name: &str, cascade: bool) -> Result<(), String>;
    async fn create_schema(&self, schema_name: &str) -> Result<(), String>;
    async fn drop_schema(&self, schema_name: &str) -> Result<(), String>;
    async fn drop_database(&self, db_name: &str) -> Result<(), String>;
    async fn create_database(&self, db_name: &str) -> Result<(), String>;
    async fn export_to_csv(&self, table_name: &str, export_path: &str) -> Result<(), String>;
    async fn get_schema_details(&self, schema_name: &str) -> Result<SchemaDetails, String>;
    async fn get_table_indexes(&self, table_name: &str) -> Result<Vec<DbIndex>, String>;
    async fn get_table_constraints(&self, table_name: &str) -> Result<Vec<TableConstraint>, String>;
}

/// Factory function to create a driver by type name.
pub fn create_driver(db_type: &str) -> Option<Box<dyn DatabaseDriver>> {
    match db_type {
        "postgresql" | "postgres" => Some(Box::new(postgres::PostgresDriver::new())),
        "mysql" => Some(Box::new(mysql::MysqlDriver::new())),
        "oracle" => Some(Box::new(oracle::OracleDriver::new())),
        "sqlite" => Some(Box::new(sqlite::SqliteDriver::new())),
        _ => None,
    }
}
