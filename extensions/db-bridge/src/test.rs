use sqlx::{postgres::{PgPool, PgRow}, Row, TypeInfo, ValueRef};

fn test(row: &PgRow) {
    let raw = row.try_get_raw(0).unwrap();
    let bytes = raw.as_bytes();
}
