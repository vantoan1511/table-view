//! SQL and PL/SQL execution, queries, cursors, and batch operations.

use bytes::Bytes;
use crate::batch::{BatchBinds, BatchResult};
use crate::constants::{BindDirection, FetchOrientation, PacketType, PACKET_HEADER_SIZE};
use crate::cursor::{ScrollableCursor, ScrollResult};
use crate::error::{Error, Result};
use crate::implicit::ImplicitResult;
use crate::messages::{ExecuteMessage, ExecuteOptions, FetchMessage};
use crate::row::Value;
use crate::statement::{BindParam, ColumnInfo, Statement, StatementType};

use super::inner::ConnectionInner;
use super::types::{ConnectionState, PlsqlResult, QueryResult};
use super::Connection;

impl Connection {
    /// Execute a SQL statement and return the result
    pub async fn execute(&self, sql: &str, params: &[Value]) -> Result<QueryResult> {
        self.ensure_ready().await?;

        let (statement, from_cache) = {
            let mut inner = self.inner.lock().await;
            if let Some(ref mut cache) = inner.statement_cache {
                if let Some(cached_stmt) = cache.get(sql) {
                    tracing::trace!(sql = sql, cursor_id = cached_stmt.cursor_id(), "Using cached statement (execute)");
                    (cached_stmt, true)
                } else {
                    (Statement::new(sql), false)
                }
            } else {
                (Statement::new(sql), false)
            }
        };

        let result = match statement.statement_type() {
            StatementType::Query => self.execute_query_with_params(&statement, params).await,
            _ => self.execute_dml_with_params(&statement, params).await,
        };

        match &result {
            Ok(query_result) => {
                let mut inner = self.inner.lock().await;
                if let Some(ref mut cache) = inner.statement_cache {
                    let should_close_cursor = if statement.statement_type() == StatementType::Query {
                        !query_result.has_more_rows
                    } else {
                        true
                    };

                    if from_cache {
                        cache.return_statement(sql);
                        if should_close_cursor {
                            cache.mark_cursor_closed(sql);
                        }
                    } else if query_result.cursor_id > 0 && !statement.is_ddl() {
                        let mut stmt_to_cache = statement.clone();
                        stmt_to_cache.set_cursor_id(query_result.cursor_id);
                        stmt_to_cache.set_executed(true);
                        cache.put(sql.to_string(), stmt_to_cache);
                        if should_close_cursor {
                            cache.mark_cursor_closed(sql);
                        }
                    }
                }
            }
            Err(_) => {
                if from_cache {
                    let mut inner = self.inner.lock().await;
                    if let Some(ref mut cache) = inner.statement_cache {
                        cache.return_statement(sql);
                        cache.mark_cursor_closed(sql);
                    }
                }
            }
        }

        result
    }

    /// Execute a query and return rows
    pub async fn query(&self, sql: &str, params: &[Value]) -> Result<QueryResult> {
        self.ensure_ready().await?;

        let (statement, from_cache) = {
            let mut inner = self.inner.lock().await;
            if let Some(ref mut cache) = inner.statement_cache {
                if let Some(cached_stmt) = cache.get(sql) {
                    tracing::trace!(sql = sql, cursor_id = cached_stmt.cursor_id(), "Using cached statement");
                    (cached_stmt, true)
                } else {
                    (Statement::new(sql), false)
                }
            } else {
                (Statement::new(sql), false)
            }
        };

        let cached_columns = if from_cache {
            Some(statement.columns().to_vec())
        } else {
            None
        };

        let mut result = self.execute_query_with_params(&statement, params).await;

        if let (Ok(ref mut query_result), Some(columns)) = (&mut result, cached_columns) {
            if query_result.columns.is_empty() && !columns.is_empty() {
                query_result.columns = columns;
            }
        }

        match &result {
            Ok(query_result) => {
                let mut inner = self.inner.lock().await;
                if let Some(ref mut cache) = inner.statement_cache {
                    if from_cache {
                        cache.return_statement(sql);
                        if !query_result.has_more_rows {
                            cache.mark_cursor_closed(sql);
                        }
                    } else if query_result.cursor_id > 0 && !statement.is_ddl() {
                        let mut stmt_to_cache = statement.clone();
                        stmt_to_cache.set_cursor_id(query_result.cursor_id);
                        stmt_to_cache.set_executed(true);
                        stmt_to_cache.set_columns(query_result.columns.clone());
                        cache.put(sql.to_string(), stmt_to_cache);
                        if !query_result.has_more_rows {
                            cache.mark_cursor_closed(sql);
                        }
                    }
                }
            }
            Err(_) => {
                if from_cache {
                    let mut inner = self.inner.lock().await;
                    if let Some(ref mut cache) = inner.statement_cache {
                        cache.return_statement(sql);
                        cache.mark_cursor_closed(sql);
                    }
                }
            }
        }

        self.handle_result(result)
    }

    /// Execute DML (INSERT, UPDATE, DELETE) and return rows affected
    pub async fn execute_dml_sql(&self, sql: &str, params: &[Value]) -> Result<u64> {
        self.ensure_ready().await?;

        let (statement, from_cache) = {
            let mut inner = self.inner.lock().await;
            if let Some(ref mut cache) = inner.statement_cache {
                if let Some(cached_stmt) = cache.get(sql) {
                    tracing::trace!(sql = sql, cursor_id = cached_stmt.cursor_id(), "Using cached DML statement");
                    (cached_stmt, true)
                } else {
                    (Statement::new(sql), false)
                }
            } else {
                (Statement::new(sql), false)
            }
        };

        let result = self.execute_dml_with_params(&statement, params).await;

        match &result {
            Ok(query_result) => {
                let mut inner = self.inner.lock().await;
                if let Some(ref mut cache) = inner.statement_cache {
                    if from_cache {
                        cache.return_statement(sql);
                        cache.mark_cursor_closed(sql);
                    } else if query_result.cursor_id > 0 && !statement.is_ddl() {
                        let mut stmt_to_cache = statement.clone();
                        stmt_to_cache.set_cursor_id(query_result.cursor_id);
                        stmt_to_cache.set_executed(true);
                        cache.put(sql.to_string(), stmt_to_cache);
                        cache.mark_cursor_closed(sql);
                    }
                }
            }
            Err(_) => {
                if from_cache {
                    let mut inner = self.inner.lock().await;
                    if let Some(ref mut cache) = inner.statement_cache {
                        cache.return_statement(sql);
                        cache.mark_cursor_closed(sql);
                    }
                }
            }
        }

        self.handle_result(result).map(|r| r.rows_affected)
    }

    /// Execute a PL/SQL block with IN/OUT/INOUT parameters
    pub async fn execute_plsql(&self, sql: &str, params: &[BindParam]) -> Result<PlsqlResult> {
        self.ensure_ready().await?;

        let statement = Statement::new(sql);

        let bind_values: Vec<Value> = params
            .iter()
            .map(|p| {
                if p.direction == BindDirection::Output {
                    p.placeholder_value()
                } else {
                    p.value.clone().unwrap_or(Value::Null)
                }
            })
            .collect();

        let bind_metadata: Vec<crate::messages::BindMetadata> = params
            .iter()
            .zip(bind_values.iter())
            .map(|(p, v)| {
                let buffer_size = if p.buffer_size > 0 {
                    p.buffer_size
                } else {
                    match v {
                        Value::String(s) => std::cmp::max(s.len() as u32, 1),
                        Value::Bytes(b) => std::cmp::max(b.len() as u32, 1),
                        Value::Integer(_) | Value::Number(_) => 22,
                        Value::Float(_) => 8,
                        Value::Boolean(_) => 1,
                        Value::Timestamp(_) => 13,
                        Value::Date(_) => 7,
                        Value::RowId(_) => 18,
                        _ => 100,
                    }
                };
                crate::messages::BindMetadata {
                    oracle_type: p.oracle_type,
                    buffer_size,
                }
            })
            .collect();

        let options = ExecuteOptions::for_plsql();
        let mut execute_msg = ExecuteMessage::new(&statement, options);
        execute_msg.set_bind_values(bind_values);
        execute_msg.set_bind_metadata(bind_metadata);

        let mut inner = self.inner.lock().await;
        let large_sdu = inner.large_sdu;
        let seq_num = inner.next_sequence_number();
        execute_msg.set_sequence_number(seq_num);
        let request = execute_msg.build_request_with_sdu(&inner.capabilities, large_sdu)?;
        inner.send(&request).await?;

        let response = inner.receive().await?;
        if response.len() <= PACKET_HEADER_SIZE {
            return Err(Error::Protocol("Empty PL/SQL response".to_string()));
        }

        let packet_type = response[4];
        if packet_type == PacketType::Marker as u8 {
            let error_response = inner.handle_marker_reset().await?;
            let payload = &error_response[PACKET_HEADER_SIZE..];
            let _: QueryResult = self.parse_error_response(payload)?;
            return Err(Error::Protocol("PL/SQL execution failed".to_string()));
        }

        let payload = &response[PACKET_HEADER_SIZE..];
        let caps = inner.capabilities.clone();
        drop(inner);

        self.parse_plsql_response(payload, &caps, params)
    }

    /// Execute a batch of DML statements with multiple rows of bind values
    pub async fn execute_batch(&self, batch: &BatchBinds) -> Result<BatchResult> {
        self.ensure_ready().await?;

        batch.validate()?;

        if batch.rows.is_empty() {
            return Ok(BatchResult::new());
        }

        let mut options = ExecuteOptions::for_dml(batch.options.auto_commit);
        options.num_execs = batch.rows.len() as u32;
        options.batch_errors = batch.options.batch_errors;
        options.dml_row_counts = batch.options.array_dml_row_counts;

        let mut execute_msg = ExecuteMessage::new(&batch.statement, options);
        execute_msg.set_batch_bind_values(batch.rows.clone());

        let mut inner = self.inner.lock().await;
        let large_sdu = inner.large_sdu;
        let seq_num = inner.next_sequence_number();
        execute_msg.set_sequence_number(seq_num);
        let request = execute_msg.build_request_with_sdu(&inner.capabilities, large_sdu)?;
        inner.send(&request).await?;

        let mut response = inner.receive_response().await?;
        if response.len() <= PACKET_HEADER_SIZE {
            return Err(Error::Protocol("Empty batch response".to_string()));
        }

        let packet_type = response[4];
        if packet_type == PacketType::Marker as u8 {
            response = self.handle_marker_protocol(&mut inner, response).await?;
        }

        let payload = &response[PACKET_HEADER_SIZE..];
        drop(inner);

        self.parse_batch_response(payload, batch.rows.len(), batch.options.array_dml_row_counts)
    }

    /// Handle MARKER packet protocol (BREAK/RESET)
    async fn handle_marker_protocol(
        &self,
        inner: &mut ConnectionInner,
        initial_response: Bytes,
    ) -> Result<Bytes> {
        let marker_type = if initial_response.len() >= PACKET_HEADER_SIZE + 3 {
            initial_response[PACKET_HEADER_SIZE + 2]
        } else {
            1
        };

        if marker_type == 1 {
            self.send_marker(inner, 2).await?;

            loop {
                match inner.receive().await {
                    Ok(pkt) => {
                        if pkt.len() < PACKET_HEADER_SIZE + 1 {
                            break;
                        }
                        let pkt_type = pkt[4];
                        if pkt_type == PacketType::Marker as u8 {
                            if pkt.len() >= PACKET_HEADER_SIZE + 3 {
                                let mk_type = pkt[PACKET_HEADER_SIZE + 2];
                                if mk_type == 2 {
                                    break;
                                }
                            }
                        } else if pkt_type == PacketType::Data as u8 {
                            return Ok(pkt);
                        } else {
                            break;
                        }
                    }
                    Err(e) => {
                        inner.state = ConnectionState::Closed;
                        return Err(e);
                    }
                }
            }

            loop {
                match inner.receive().await {
                    Ok(pkt) => {
                        let pkt_type = pkt[4];
                        if pkt_type == PacketType::Marker as u8 {
                            continue;
                        } else if pkt_type == PacketType::Data as u8 {
                            return Ok(pkt);
                        } else {
                            return Err(Error::Protocol(format!(
                                "Unexpected packet type {} after reset",
                                pkt_type
                            )));
                        }
                    }
                    Err(e) => {
                        inner.state = ConnectionState::Closed;
                        return Err(e);
                    }
                }
            }
        }

        Ok(initial_response)
    }

    /// Fetch more rows from an open cursor
    pub async fn fetch_more(
        &self,
        cursor_id: u16,
        columns: &[ColumnInfo],
        fetch_size: u32,
    ) -> Result<QueryResult> {
        self.ensure_ready().await?;

        let fetch_msg = FetchMessage::new(cursor_id, fetch_size);

        let mut inner = self.inner.lock().await;
        let request = fetch_msg.build_request(&inner.capabilities)?;
        inner.send(&request).await?;

        let response = inner.receive_response().await?;
        if response.len() <= PACKET_HEADER_SIZE {
            return Err(Error::Protocol("Empty fetch response".to_string()));
        }

        let payload = &response[PACKET_HEADER_SIZE..];
        let caps = inner.capabilities.clone();
        drop(inner);
        self.parse_fetch_response(payload, columns, &caps)
    }

    /// Fetch rows from a REF CURSOR
    pub async fn fetch_cursor(&self, cursor: &crate::types::RefCursor) -> Result<QueryResult> {
        self.fetch_cursor_with_size(cursor, 100).await
    }

    /// Fetch rows from a REF CURSOR with a specified fetch size
    pub async fn fetch_cursor_with_size(
        &self,
        cursor: &crate::types::RefCursor,
        fetch_size: u32,
    ) -> Result<QueryResult> {
        if cursor.cursor_id() == 0 {
            return Err(Error::InvalidCursor("Cursor ID is 0 (not initialized)".to_string()));
        }

        self.ensure_ready().await?;

        let mut stmt = Statement::new("");
        stmt.set_cursor_id(cursor.cursor_id());
        stmt.set_columns(cursor.columns().to_vec());
        stmt.set_executed(true);
        stmt.set_statement_type(StatementType::Query);

        let options = ExecuteOptions::for_ref_cursor(fetch_size);
        let mut execute_msg = ExecuteMessage::new(&stmt, options);

        let mut inner = self.inner.lock().await;
        let large_sdu = inner.large_sdu;
        let seq_num = inner.next_sequence_number();
        execute_msg.set_sequence_number(seq_num);

        let request = execute_msg.build_request_with_sdu(&inner.capabilities, large_sdu)?;
        inner.send(&request).await?;

        let response = inner.receive_response().await?;
        if response.len() <= PACKET_HEADER_SIZE {
            return Err(Error::Protocol("Empty cursor response".to_string()));
        }

        let packet_type = response[4];
        if packet_type == PacketType::Marker as u8 {
            let error_response = inner.handle_marker_reset().await?;
            let payload = &error_response[PACKET_HEADER_SIZE..];
            return self.parse_error_response(payload);
        }

        let payload = &response[PACKET_HEADER_SIZE..];
        let caps = inner.capabilities.clone();
        drop(inner);
        self.parse_fetch_response(payload, cursor.columns(), &caps)
    }

    /// Fetch rows from an implicit result set
    pub async fn fetch_implicit_result(&self, result: &ImplicitResult) -> Result<QueryResult> {
        self.fetch_implicit_result_with_size(result, 100).await
    }

    /// Fetch rows from an implicit result set with a specified fetch size
    pub async fn fetch_implicit_result_with_size(
        &self,
        result: &ImplicitResult,
        fetch_size: u32,
    ) -> Result<QueryResult> {
        let cursor = crate::types::RefCursor::new(result.cursor_id, result.columns.clone());
        self.fetch_cursor_with_size(&cursor, fetch_size).await
    }

    /// Open a scrollable cursor for bidirectional navigation
    pub async fn open_scrollable_cursor(&self, sql: &str) -> Result<ScrollableCursor> {
        self.ensure_ready().await?;

        let statement = Statement::new(sql);

        let mut options = ExecuteOptions::for_query(1);
        options.scrollable = true;

        let mut execute_msg = ExecuteMessage::new(&statement, options);

        let mut inner = self.inner.lock().await;
        let large_sdu = inner.large_sdu;
        let seq_num = inner.next_sequence_number();
        execute_msg.set_sequence_number(seq_num);
        let request = execute_msg.build_request_with_sdu(&inner.capabilities, large_sdu)?;
        inner.send(&request).await?;

        let response = inner.receive_response().await?;

        if response.len() <= PACKET_HEADER_SIZE {
            return Err(Error::Protocol("Empty scrollable cursor response".to_string()));
        }

        let packet_type = response[4];
        if packet_type == PacketType::Marker as u8 {
            let error_response = inner.handle_marker_reset().await?;
            let payload = &error_response[PACKET_HEADER_SIZE..];
            let _: QueryResult = self.parse_error_response(payload)?;
            return Err(Error::Protocol("Unexpected successful response after MARKER".to_string()));
        }

        let payload = &response[PACKET_HEADER_SIZE..];
        let result = self.parse_query_response(payload, &inner.capabilities)?;

        Ok(ScrollableCursor::new(result.cursor_id, result.columns))
    }

    /// Scroll to a position in a scrollable cursor and fetch rows
    pub async fn scroll(
        &self,
        cursor: &mut ScrollableCursor,
        orientation: FetchOrientation,
        offset: i64,
    ) -> Result<ScrollResult> {
        self.ensure_ready().await?;

        if !cursor.is_open() {
            return Err(Error::CursorClosed);
        }

        let mut stmt = Statement::new("");
        stmt.set_cursor_id(cursor.cursor_id);
        stmt.set_columns(cursor.columns.clone());
        stmt.set_executed(true);
        stmt.set_statement_type(StatementType::Query);

        let mut options = ExecuteOptions::for_query(1);
        options.scrollable = true;
        options.scroll_operation = true;
        options.fetch_orientation = orientation as u32;
        options.fetch_pos = match orientation {
            FetchOrientation::First => 1,
            FetchOrientation::Last => 0,
            FetchOrientation::Absolute => offset.max(0) as u32,
            FetchOrientation::Relative => (cursor.position + offset).max(0) as u32,
            FetchOrientation::Next => (cursor.position + 1).max(0) as u32,
            FetchOrientation::Prior => (cursor.position - 1).max(0) as u32,
            FetchOrientation::Current => cursor.position.max(0) as u32,
        };

        let mut execute_msg = ExecuteMessage::new(&stmt, options);

        let mut inner = self.inner.lock().await;
        let large_sdu = inner.large_sdu;
        let seq_num = inner.next_sequence_number();
        execute_msg.set_sequence_number(seq_num);
        let request = execute_msg.build_request_with_sdu(&inner.capabilities, large_sdu)?;
        inner.send(&request).await?;

        let response = inner.receive_response().await?;
        if response.len() <= PACKET_HEADER_SIZE {
            return Err(Error::Protocol("Empty scroll response".to_string()));
        }

        let packet_type = response[4];
        if packet_type == PacketType::Marker as u8 {
            let error_response = inner.handle_marker_reset().await?;
            let payload = &error_response[PACKET_HEADER_SIZE..];
            let _: QueryResult = self.parse_error_response(payload)?;
            return Err(Error::Protocol("Scroll operation failed".to_string()));
        }

        let payload = &response[PACKET_HEADER_SIZE..];
        let query_result = self.parse_query_response_with_columns(payload, &inner.capabilities, &cursor.columns)?;

        let new_position = if !query_result.rows.is_empty() {
            query_result.rows_affected as i64
        } else {
            match orientation {
                FetchOrientation::First => 0,
                FetchOrientation::Last => cursor.row_count.unwrap_or(0) as i64 + 1,
                FetchOrientation::Next => cursor.position + 1,
                FetchOrientation::Prior => cursor.position - 1,
                FetchOrientation::Absolute => offset,
                FetchOrientation::Relative => cursor.position + offset,
                FetchOrientation::Current => cursor.position,
            }
        };

        cursor.update_position(new_position);

        let mut result = ScrollResult::new(query_result.rows, new_position);
        result.at_end = !query_result.has_more_rows;
        result.at_beginning = new_position <= 1;

        Ok(result)
    }

    /// Close a scrollable cursor
    pub async fn close_cursor(&self, cursor: &mut ScrollableCursor) -> Result<()> {
        if !cursor.is_open() {
            return Ok(());
        }

        cursor.mark_closed();
        Ok(())
    }

    /// Get type information for a database object or collection type
    pub async fn get_type(&self, type_name: &str) -> Result<crate::dbobject::DbObjectType> {
        use crate::dbobject::{CollectionType, DbObjectType};

        self.ensure_ready().await?;

        let (schema, name) = parse_type_name(type_name, &self.config.username);

        let type_info = self
            .query(
                "SELECT typecode, type_oid FROM all_types WHERE owner = :1 AND type_name = :2",
                &[Value::String(schema.clone()), Value::String(name.clone())],
            )
            .await?;

        if type_info.rows.is_empty() {
            return Err(Error::OracleError {
                code: 4043,
                message: format!("Type {}.{} not found", schema, name),
            });
        }

        let row = &type_info.rows[0];
        let typecode = row.get(0).and_then(|v| v.as_str()).unwrap_or("");
        let type_oid = row.get(1).and_then(|v| v.as_bytes()).map(|b| b.to_vec());

        if typecode == "COLLECTION" {
            let coll_info = self.query(
                "SELECT coll_type, elem_type_name, elem_type_owner, upper_bound FROM all_coll_types WHERE owner = :1 AND type_name = :2",
                &[Value::String(schema.clone()), Value::String(name.clone())],
            ).await?;

            if coll_info.rows.is_empty() {
                return Err(Error::OracleError {
                    code: 4043,
                    message: format!("Collection type {}.{} metadata not found", schema, name),
                });
            }

            let coll_row = &coll_info.rows[0];
            let coll_type_str = coll_row.get(0).and_then(|v| v.as_str()).unwrap_or("");
            let elem_type_name = coll_row.get(1).and_then(|v| v.as_str()).unwrap_or("VARCHAR2");
            let _elem_type_owner = coll_row.get(2).and_then(|v| v.as_str());

            let collection_type = match coll_type_str {
                "VARYING ARRAY" => CollectionType::Varray,
                "TABLE" => CollectionType::NestedTable,
                _ => CollectionType::Varray,
            };

            let element_type = oracle_type_from_name(elem_type_name);

            let mut obj_type = DbObjectType::collection(schema, name, collection_type, element_type);
            obj_type.oid = type_oid;
            Ok(obj_type)
        } else {
            let mut obj_type = DbObjectType::new(schema, name);
            obj_type.oid = type_oid;
            Ok(obj_type)
        }
    }

    /// Internal: Execute a query statement with optional bind parameters
    pub(crate) async fn execute_query_with_params(&self, statement: &Statement, params: &[Value]) -> Result<QueryResult> {
        let prefetch_rows = 100;

        let options = ExecuteOptions::for_query(prefetch_rows);
        let mut execute_msg = ExecuteMessage::new(statement, options);

        if !params.is_empty() {
            execute_msg.set_bind_values(params.to_vec());
        }

        let mut inner = self.inner.lock().await;
        let large_sdu = inner.large_sdu;
        let seq_num = inner.next_sequence_number();
        execute_msg.set_sequence_number(seq_num);
        let request = execute_msg.build_request_with_sdu(&inner.capabilities, large_sdu)?;
        inner.send(&request).await?;

        let response = inner.receive_response().await?;
        if response.len() <= PACKET_HEADER_SIZE {
            return Err(Error::Protocol("Empty query response".to_string()));
        }

        let packet_type = response[4];
        if packet_type == PacketType::Marker as u8 {
            let error_response = inner.handle_marker_reset().await?;
            let payload = &error_response[PACKET_HEADER_SIZE..];
            return self.parse_error_response(payload);
        }

        let payload = &response[PACKET_HEADER_SIZE..];
        let mut result = self.parse_query_response(payload, &inner.capabilities)?;

        let has_lob_columns = result.columns.iter().any(|col| col.is_lob());

        if has_lob_columns && !statement.requires_define() {
            let mut stmt_with_define = statement.clone();
            stmt_with_define.set_columns(result.columns.clone());
            stmt_with_define.set_cursor_id(result.cursor_id);
            stmt_with_define.set_requires_define(true);
            stmt_with_define.set_no_prefetch(true);
            stmt_with_define.set_executed(true);

            let define_options = ExecuteOptions::for_query(prefetch_rows);
            let mut define_msg = ExecuteMessage::new(&stmt_with_define, define_options);
            let seq_num = inner.next_sequence_number();
            define_msg.set_sequence_number(seq_num);

            let define_request = define_msg.build_request_with_sdu(&inner.capabilities, large_sdu)?;
            inner.send(&define_request).await?;

            let define_response = inner.receive_response().await?;
            if define_response.len() <= PACKET_HEADER_SIZE {
                return Err(Error::Protocol("Empty define response".to_string()));
            }

            let packet_type = define_response[4];
            if packet_type == PacketType::Marker as u8 {
                let error_response = inner.handle_marker_reset().await?;
                let payload = &error_response[PACKET_HEADER_SIZE..];
                return self.parse_error_response(payload);
            }

            let payload = &define_response[PACKET_HEADER_SIZE..];
            result = self.parse_query_response_with_columns(
                payload,
                &inner.capabilities,
                &stmt_with_define.columns(),
            )?;
        }

        Ok(result)
    }

    /// Internal: Execute a DML statement with optional bind parameters
    pub(crate) async fn execute_dml_with_params(&self, statement: &Statement, params: &[Value]) -> Result<QueryResult> {
        let options = ExecuteOptions::for_dml(false);
        let mut execute_msg = ExecuteMessage::new(statement, options);

        if !params.is_empty() {
            execute_msg.set_bind_values(params.to_vec());
        }

        let mut inner = self.inner.lock().await;
        let large_sdu = inner.large_sdu;
        let seq_num = inner.next_sequence_number();
        execute_msg.set_sequence_number(seq_num);
        let request = execute_msg.build_request_with_sdu(&inner.capabilities, large_sdu)?;

        inner.send(&request).await?;

        let mut response = inner.receive_response().await?;
        if response.len() <= PACKET_HEADER_SIZE {
            return Err(Error::Protocol("Empty DML response".to_string()));
        }

        let packet_type = response[4];
        if packet_type == PacketType::Marker as u8 {
            let marker_type = if response.len() >= PACKET_HEADER_SIZE + 3 {
                response[PACKET_HEADER_SIZE + 2]
            } else {
                1
            };

            if marker_type == 1 {
                self.send_marker(&mut inner, 2).await?;

                let mut got_reset = false;
                let mut max_attempts = 5;
                loop {
                    match inner.receive().await {
                        Ok(pkt) => {
                            if pkt.len() < PACKET_HEADER_SIZE + 1 {
                                break;
                            }
                            let pkt_type = pkt[4];
                            if pkt_type == PacketType::Marker as u8 {
                                if pkt.len() >= PACKET_HEADER_SIZE + 3 {
                                    let mk_type = pkt[PACKET_HEADER_SIZE + 2];
                                    if mk_type == 2 {
                                        got_reset = true;
                                        break;
                                    }
                                    max_attempts -= 1;
                                    if max_attempts == 0 {
                                        return Err(Error::Protocol(
                                            "Server rejected operation (multiple BREAK markers)".to_string()
                                        ));
                                    }
                                    continue;
                                }
                            } else if pkt_type == PacketType::Data as u8 {
                                let payload = &pkt[PACKET_HEADER_SIZE..];
                                return self.parse_dml_response(payload);
                            } else {
                                break;
                            }
                        }
                        Err(e) => {
                            inner.state = ConnectionState::Closed;
                            return Err(Error::Protocol(format!(
                                "Server closed connection during error handling: {}",
                                e
                            )));
                        }
                    }
                }

                if got_reset {
                    loop {
                        match inner.receive().await {
                            Ok(pkt) => {
                                if pkt.len() < PACKET_HEADER_SIZE + 1 {
                                    break;
                                }
                                let pkt_type = pkt[4];
                                if pkt_type == PacketType::Marker as u8 {
                                    continue;
                                } else if pkt_type == PacketType::Data as u8 {
                                    response = pkt;
                                    break;
                                } else {
                                    return Err(Error::Protocol(format!(
                                        "Unexpected packet type {} after reset",
                                        pkt_type
                                    )));
                                }
                            }
                            Err(_e) => {
                                inner.state = ConnectionState::Closed;
                                return Err(Error::OracleError {
                                    code: 0,
                                    message: "Server rejected the operation and closed the connection. \
                                              This may happen when binding a temporary LOB to an INSERT statement. \
                                              Try using a different approach (e.g., DBMS_LOB procedures).".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        let payload = &response[PACKET_HEADER_SIZE..];
        self.parse_dml_response(payload)
    }
}

/// Parse a type name into (schema, name) components
fn parse_type_name(type_name: &str, default_schema: &str) -> (String, String) {
    let parts: Vec<&str> = type_name.split('.').collect();
    match parts.len() {
        1 => (default_schema.to_uppercase(), parts[0].to_uppercase()),
        2 => (parts[0].to_uppercase(), parts[1].to_uppercase()),
        _ => {
            (parts[0].to_uppercase(), parts[1..].join(".").to_uppercase())
        }
    }
}

/// Convert an Oracle type name from data dictionary to OracleType enum
fn oracle_type_from_name(type_name: &str) -> crate::constants::OracleType {
    use crate::constants::OracleType;

    match type_name.to_uppercase().as_str() {
        "NUMBER" => OracleType::Number,
        "INTEGER" | "INT" | "SMALLINT" => OracleType::Number,
        "FLOAT" | "REAL" | "DOUBLE PRECISION" => OracleType::BinaryDouble,
        "BINARY_FLOAT" => OracleType::BinaryFloat,
        "BINARY_DOUBLE" => OracleType::BinaryDouble,
        "VARCHAR2" | "VARCHAR" | "NVARCHAR2" => OracleType::Varchar,
        "CHAR" | "NCHAR" => OracleType::Char,
        "DATE" => OracleType::Date,
        "TIMESTAMP" => OracleType::Timestamp,
        "TIMESTAMP WITH TIME ZONE" => OracleType::TimestampTz,
        "TIMESTAMP WITH LOCAL TIME ZONE" => OracleType::TimestampLtz,
        "RAW" => OracleType::Raw,
        "BLOB" => OracleType::Blob,
        "CLOB" | "NCLOB" => OracleType::Clob,
        "BOOLEAN" | "PL/SQL BOOLEAN" => OracleType::Boolean,
        "ROWID" | "UROWID" => OracleType::Rowid,
        "XMLTYPE" => OracleType::Varchar,
        _ => OracleType::Varchar,
    }
}
