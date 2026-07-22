//! Response parsers for TNS data packets.

use crate::buffer::ReadBuffer;
use crate::capabilities::Capabilities;
use crate::constants::{BindDirection, MessageType, OracleType};
use crate::dbobject::{CollectionType, DbObject, DbObjectType};
use crate::error::{Error, Result};
use crate::implicit::{ImplicitResult, ImplicitResults};
use crate::row::{Row, Value};
use crate::statement::{BindParam, ColumnInfo};
use crate::types::{decode_collection, decode_vector, LobLocator, LobValue, OsonDecoder, RefCursor};

use super::types::{PlsqlResult, QueryResult};
use super::Connection;

impl Connection {
    /// Parse query response to extract columns and rows
    pub(crate) fn parse_query_response(&self, payload: &[u8], caps: &Capabilities) -> Result<QueryResult> {
        self.parse_query_response_with_columns(payload, caps, &[])
    }

    /// Parse query response with pre-known columns (for re-execute after define)
    pub(crate) fn parse_query_response_with_columns(
        &self,
        payload: &[u8],
        caps: &Capabilities,
        known_columns: &[ColumnInfo],
    ) -> Result<QueryResult> {
        if payload.len() < 3 {
            return Err(Error::Protocol("Query response too short".to_string()));
        }

        let mut buf = ReadBuffer::from_slice(payload);

        // Skip data flags
        buf.skip(2)?;

        // Use known columns if provided, otherwise parse from describe info
        let mut columns: Vec<ColumnInfo> = known_columns.to_vec();
        let mut rows: Vec<Row> = Vec::new();
        let mut cursor_id: u16 = 0;
        let mut row_count: u64 = 0;
        let mut end_of_response = false;

        // Bit vector for duplicate column optimization
        let mut bit_vector: Option<Vec<u8>> = None;
        let mut previous_row_values: Option<Vec<Value>> = None;

        // Process messages until we hit end of response or run out of data
        while !end_of_response && buf.remaining() > 0 {
            let msg_type = buf.read_u8()?;

            match msg_type {
                // DescribeInfo (16) - column metadata
                x if x == MessageType::DescribeInfo as u8 => {
                    buf.skip_raw_bytes_chunked()?;
                    columns = self.parse_describe_info(&mut buf, caps.ttc_field_version)?;
                }

                // RowHeader (6) - header info for rows
                x if x == MessageType::RowHeader as u8 => {
                    self.parse_row_header(&mut buf)?;
                }

                // RowData (7) - actual row values
                x if x == MessageType::RowData as u8 => {
                    let row = self.parse_row_data_with_bitvector(
                        &mut buf,
                        &columns,
                        caps,
                        bit_vector.as_deref(),
                        previous_row_values.as_ref(),
                    )?;
                    previous_row_values = Some(row.values().to_vec());
                    bit_vector = None;
                    rows.push(row);
                }

                // Error (4) - completion or error
                x if x == MessageType::Error as u8 => {
                    let (error_code, error_msg, cid, rc) = self.parse_error_info_with_rowcount(&mut buf)?;
                    cursor_id = cid;
                    row_count = rc;
                    if error_code != 0 && error_code != 1403 {
                        return Err(Error::OracleError {
                            code: error_code,
                            message: error_msg.unwrap_or_default(),
                        });
                    }
                    end_of_response = true;
                }

                // Parameter (8) - return parameters
                x if x == MessageType::Parameter as u8 => {
                    self.parse_return_parameters(&mut buf)?;
                }

                // Status (9) - call status
                x if x == MessageType::Status as u8 => {
                    let _call_status = buf.read_ub4()?;
                    let _end_to_end_seq = buf.read_ub2()?;
                }

                // BitVector (21)
                21 => {
                    let _num_columns_sent = buf.read_ub2()?;
                    let num_bytes = (columns.len() + 7) / 8;
                    if num_bytes > 0 {
                        let bv = buf.read_bytes_vec(num_bytes)?;
                        bit_vector = Some(bv);
                    }
                }

                _ => {
                    break;
                }
            }
        }

        Ok(QueryResult {
            columns,
            rows,
            rows_affected: row_count,
            has_more_rows: false,
            cursor_id,
        })
    }

    /// Parse fetch response for subsequent row fetches
    pub(crate) fn parse_fetch_response(&self, payload: &[u8], columns: &[ColumnInfo], caps: &Capabilities) -> Result<QueryResult> {
        if payload.len() < 3 {
            return Err(Error::Protocol("Fetch response too short".to_string()));
        }

        let mut buf = ReadBuffer::from_slice(payload);
        let mut rows = Vec::new();
        let mut has_more_rows = false;

        let mut bit_vector: Option<Vec<u8>> = None;
        let mut previous_row_values: Option<Vec<Value>> = None;

        buf.skip(2)?;

        while buf.remaining() >= 1 {
            let msg_type = buf.read_u8()?;

            match msg_type {
                x if x == MessageType::RowHeader as u8 => {
                    buf.skip(1)?;
                    buf.skip_ub2()?;
                    buf.skip_ub4()?;
                    buf.skip_ub4()?;
                    buf.skip_ub2()?;
                    let num_bytes = buf.read_ub4()?;
                    if num_bytes > 0 {
                        buf.skip(1)?;
                        let bv = buf.read_bytes_vec(num_bytes as usize)?;
                        bit_vector = Some(bv);
                    }
                    let rxhrid_bytes = buf.read_ub4()?;
                    if rxhrid_bytes > 0 {
                        buf.skip_raw_bytes_chunked()?;
                    }
                }
                x if x == MessageType::RowData as u8 => {
                    let row = self.parse_row_data_with_bitvector(
                        &mut buf,
                        columns,
                        caps,
                        bit_vector.as_deref(),
                        previous_row_values.as_ref(),
                    )?;
                    previous_row_values = Some(row.values().to_vec());
                    bit_vector = None;
                    rows.push(row);
                }
                x if x == MessageType::BitVector as u8 => {
                    let _num_columns_sent = buf.read_ub2()?;
                    let num_bytes = (columns.len() + 7) / 8;
                    if num_bytes > 0 {
                        let bv = buf.read_bytes_vec(num_bytes)?;
                        bit_vector = Some(bv);
                    }
                }
                x if x == MessageType::Error as u8 => {
                    let (error_code, error_msg, more_rows) = self.parse_error_message_info(&mut buf)?;
                    has_more_rows = more_rows;
                    if error_code != 0 && error_code != 1403 {
                        return Err(Error::OracleError {
                            code: error_code,
                            message: error_msg,
                        });
                    }
                    break;
                }
                x if x == MessageType::Status as u8 => {
                    break;
                }
                x if x == MessageType::EndOfResponse as u8 => {
                    break;
                }
                _ => {
                    break;
                }
            }
        }

        Ok(QueryResult {
            columns: columns.to_vec(),
            rows,
            rows_affected: 0,
            has_more_rows,
            cursor_id: 0,
        })
    }

    /// Parse a PL/SQL response containing OUT parameter values
    pub(crate) fn parse_plsql_response(
        &self,
        payload: &[u8],
        caps: &Capabilities,
        params: &[BindParam],
    ) -> Result<PlsqlResult> {
        if payload.len() < 3 {
            return Err(Error::Protocol("PL/SQL response too short".to_string()));
        }

        let mut buf = ReadBuffer::from_slice(payload);

        buf.skip(2)?;

        let mut out_values: Vec<Value> = Vec::new();
        let mut _out_indices: Vec<usize> = Vec::new();
        let mut row_count: u64 = 0;
        let mut cursor_id: Option<u16> = None;
        let mut end_of_response = false;
        let mut implicit_results = ImplicitResults::new();

        let mut out_columns: Vec<ColumnInfo> = Vec::new();

        while !end_of_response && buf.remaining() > 0 {
            let msg_type = buf.read_u8()?;

            match msg_type {
                x if x == MessageType::IoVector as u8 => {
                    let (indices, cols) = self.parse_io_vector(&mut buf, params)?;
                    _out_indices = indices;
                    out_columns = cols;
                }

                x if x == MessageType::RowHeader as u8 => {
                    self.parse_row_header(&mut buf)?;
                }

                x if x == MessageType::RowData as u8 => {
                    if !out_columns.is_empty() {
                        let row = self.parse_row_data_single(&mut buf, &out_columns, caps)?;
                        for (idx, value) in row.into_values().into_iter().enumerate() {
                            if let Value::Cursor(cursor) = &value {
                                if cursor_id.is_none() && cursor.cursor_id() != 0 {
                                    cursor_id = Some(cursor.cursor_id());
                                }
                            }
                            if idx < out_values.len() {
                                out_values[idx] = value;
                            } else {
                                out_values.push(value);
                            }
                        }
                    } else {
                        break;
                    }
                }

                x if x == MessageType::DescribeInfo as u8 => {
                    buf.skip_raw_bytes_chunked()?;
                    let cursor_columns = self.parse_describe_info(&mut buf, caps.ttc_field_version)?;
                    let _ = cursor_columns;
                }

                x if x == MessageType::FlushOutBinds as u8 => {}

                x if x == MessageType::Error as u8 => {
                    let (error_code, error_msg, _cid, rc) = self.parse_error_info_with_rowcount(&mut buf)?;
                    row_count = rc;
                    if error_code != 0 {
                        return Err(Error::OracleError {
                            code: error_code,
                            message: error_msg.unwrap_or_default(),
                        });
                    }
                    end_of_response = true;
                }

                x if x == MessageType::Parameter as u8 => {
                    self.parse_return_parameters(&mut buf)?;
                }

                x if x == MessageType::Status as u8 => {
                    let _call_status = buf.read_ub4()?;
                    let _end_to_end_seq = buf.read_ub2()?;
                }

                x if x == MessageType::ImplicitResultset as u8 => {
                    let parsed_results = self.parse_implicit_results(&mut buf, caps)?;
                    implicit_results = parsed_results;
                }

                _ => {
                    break;
                }
            }
        }

        Ok(PlsqlResult {
            out_values,
            rows_affected: row_count,
            cursor_id,
            implicit_results,
        })
    }

    pub(crate) fn parse_implicit_results(&self, buf: &mut ReadBuffer, caps: &Capabilities) -> Result<ImplicitResults> {
        let num_results = buf.read_ub4()?;
        let mut results = ImplicitResults::new();

        for _ in 0..num_results {
            let num_bytes = buf.read_u8()?;
            if num_bytes > 0 {
                buf.skip(num_bytes as usize)?;
            }

            let columns = self.parse_describe_info(buf, caps.ttc_field_version)?;
            let cursor_id = buf.read_ub2()?;

            let result = ImplicitResult::new(cursor_id, columns, Vec::new());
            results.add(result);
        }

        Ok(results)
    }

    pub(crate) fn parse_io_vector(
        &self,
        buf: &mut ReadBuffer,
        params: &[BindParam],
    ) -> Result<(Vec<usize>, Vec<ColumnInfo>)> {
        buf.skip(1)?;
        let num_requests = buf.read_ub2()? as u32;
        let num_iters = buf.read_ub4()?;
        let num_binds = num_iters * 256 + num_requests;
        let _ = buf.read_ub4()?;
        let _ = buf.read_ub2()?;

        let num_bytes = buf.read_ub2()? as usize;
        if num_bytes > 0 {
            buf.skip(num_bytes)?;
        }

        let num_bytes = buf.read_ub4()? as usize;
        if num_bytes > 0 {
            buf.skip_raw_bytes_chunked()?;
        }

        let mut out_indices = Vec::new();
        let mut out_columns = Vec::new();

        for i in 0..(num_binds as usize).min(params.len()) {
            let dir_byte = buf.read_u8()?;
            let dir = BindDirection::try_from(dir_byte).unwrap_or(BindDirection::Input);

            if dir != BindDirection::Input {
                out_indices.push(i);

                let param = &params[i];
                let mut col = ColumnInfo::new(format!("OUT_{}", i), param.oracle_type);
                col.buffer_size = param.buffer_size;
                col.data_size = param.buffer_size;
                col.nullable = true;

                if let Some(Value::Collection(ref placeholder)) = param.value {
                    if let Some(Value::Integer(elem_type_code)) = placeholder.get("_element_type") {
                        col.element_type = crate::constants::OracleType::try_from(*elem_type_code as u8).ok();
                    }
                }

                out_columns.push(col);
            }
        }

        Ok((out_indices, out_columns))
    }

    pub(crate) fn parse_row_header(&self, buf: &mut ReadBuffer) -> Result<()> {
        buf.skip_ub1()?;
        buf.skip_ub2()?;
        buf.skip_ub4()?;
        buf.skip_ub4()?;
        buf.skip_ub2()?;
        let num_bytes = buf.read_ub4()? as usize;
        if num_bytes > 0 {
            buf.skip_ub1()?;
            buf.skip(num_bytes)?;
        }
        let num_bytes = buf.read_ub4()? as usize;
        if num_bytes > 0 {
            buf.skip_raw_bytes_chunked()?;
        }
        Ok(())
    }

    pub(crate) fn parse_return_parameters(&self, buf: &mut ReadBuffer) -> Result<()> {
        self.parse_return_parameters_internal(buf, false).map(|_| ())
    }

    pub(crate) fn parse_return_parameters_internal(
        &self,
        buf: &mut ReadBuffer,
        want_row_counts: bool,
    ) -> Result<Option<Vec<u64>>> {
        let num_params = buf.read_ub2()?;
        for _ in 0..num_params {
            buf.skip_ub4()?;
        }

        let al8txl = buf.read_ub2()?;
        if al8txl > 0 {
            buf.skip(al8txl as usize)?;
        }

        let num_pairs = buf.read_ub2()?;
        for _ in 0..num_pairs {
            buf.read_bytes_with_length()?;
            buf.read_bytes_with_length()?;
            buf.skip_ub2()?;
        }

        let num_bytes = buf.read_ub2()?;
        if num_bytes > 0 {
            buf.skip(num_bytes as usize)?;
        }

        if want_row_counts && buf.remaining() >= 4 {
            let num_rows = buf.read_ub4()? as usize;
            let mut row_counts = Vec::with_capacity(num_rows);
            for _ in 0..num_rows {
                let count = buf.read_ub8()?;
                row_counts.push(count);
            }
            Ok(Some(row_counts))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn parse_row_data_single(
        &self,
        buf: &mut ReadBuffer,
        columns: &[ColumnInfo],
        caps: &Capabilities,
    ) -> Result<Row> {
        let mut values = Vec::with_capacity(columns.len());

        for col in columns {
            let value = self.parse_column_value(buf, col, caps)?;
            values.push(value);
        }

        Ok(Row::new(values))
    }

    pub(crate) fn parse_row_data_with_bitvector(
        &self,
        buf: &mut ReadBuffer,
        columns: &[ColumnInfo],
        caps: &Capabilities,
        bit_vector: Option<&[u8]>,
        previous_values: Option<&Vec<Value>>,
    ) -> Result<Row> {
        let mut values = Vec::with_capacity(columns.len());

        for (col_idx, col) in columns.iter().enumerate() {
            let is_duplicate = if let Some(bv) = bit_vector {
                let byte_num = col_idx / 8;
                let bit_num = col_idx % 8;
                if byte_num < bv.len() {
                    (bv[byte_num] & (1 << bit_num)) == 0
                } else {
                    false
                }
            } else {
                false
            };

            if is_duplicate {
                if let Some(prev) = previous_values {
                    if col_idx < prev.len() {
                        values.push(prev[col_idx].clone());
                    } else {
                        values.push(Value::Null);
                    }
                } else {
                    values.push(Value::Null);
                }
            } else {
                let value = self.parse_column_value(buf, col, caps)?;
                values.push(value);
            }
        }

        Ok(Row::new(values))
    }

    pub(crate) fn parse_column_value(&self, buf: &mut ReadBuffer, col: &ColumnInfo, caps: &Capabilities) -> Result<Value> {
        if col.is_lob() {
            return self.parse_lob_value(buf, col);
        }

        if col.oracle_type == OracleType::Cursor {
            return self.parse_cursor_value(buf, caps);
        }

        if col.oracle_type == OracleType::Object {
            return self.parse_object_value(buf, col);
        }

        let data = buf.read_bytes_with_length()?;

        match data {
            None => Ok(Value::Null),
            Some(bytes) if bytes.is_empty() => Ok(Value::Null),
            Some(bytes) => {
                match col.oracle_type {
                    OracleType::Number => {
                        let num = crate::types::decode_oracle_number(&bytes)?;
                        Ok(Value::String(num.value))
                    }
                    OracleType::Varchar | OracleType::Char | OracleType::Long => {
                        let s = String::from_utf8_lossy(&bytes).to_string();
                        Ok(Value::String(s))
                    }
                    OracleType::Raw | OracleType::LongRaw => {
                        Ok(Value::Bytes(bytes.to_vec()))
                    }
                    OracleType::Date => {
                        let date = crate::types::decode_oracle_date(&bytes)?;
                        Ok(Value::Date(date))
                    }
                    OracleType::Timestamp | OracleType::TimestampLtz => {
                        let ts = crate::types::decode_oracle_timestamp(&bytes)?;
                        Ok(Value::Timestamp(ts))
                    }
                    OracleType::TimestampTz => {
                        let ts = crate::types::decode_oracle_timestamp(&bytes)?;
                        Ok(Value::Timestamp(ts))
                    }
                    _ => {
                        let s = String::from_utf8_lossy(&bytes).to_string();
                        Ok(Value::String(s))
                    }
                }
            }
        }
    }

    pub(crate) fn parse_cursor_value(&self, buf: &mut ReadBuffer, caps: &Capabilities) -> Result<Value> {
        let _length = buf.read_u8()?;
        let cursor_columns = self.parse_describe_info(buf, caps.ttc_field_version)?;
        let cursor_id = buf.read_ub2()?;
        let ref_cursor = RefCursor::new(cursor_id, cursor_columns);

        Ok(Value::Cursor(ref_cursor))
    }

    pub(crate) fn parse_object_value(&self, buf: &mut ReadBuffer, col: &ColumnInfo) -> Result<Value> {
        let toid_len = buf.read_ub4()?;
        let _toid = if toid_len > 0 {
            buf.read_bytes_with_length()?
        } else {
            None
        };

        let oid_len = buf.read_ub4()?;
        let _oid = if oid_len > 0 {
            buf.read_bytes_with_length()?
        } else {
            None
        };

        let snapshot_len = buf.read_ub4()?;
        if snapshot_len > 0 {
            buf.skip_raw_bytes_chunked()?;
        }

        let _version = buf.read_ub2()?;
        let data_len = buf.read_ub4()?;
        let _flags = buf.read_ub2()?;

        if data_len == 0 {
            return Ok(Value::Null);
        }

        let packed_data = buf.read_bytes_with_length()?;

        match packed_data {
            None => Ok(Value::Null),
            Some(data) if data.is_empty() => Ok(Value::Null),
            Some(data) => {
                let type_name = col.type_name.clone().unwrap_or_else(|| "UNKNOWN".to_string());
                let is_collection = !data.is_empty() && (data[0] & 0x08) != 0;

                if is_collection {
                    let element_type = col.element_type.unwrap_or(OracleType::Varchar);
                    let collection_type = CollectionType::Varray;

                    let obj_type = DbObjectType::collection(
                        &col.type_schema.clone().unwrap_or_default(),
                        &type_name,
                        collection_type,
                        element_type,
                    );

                    match decode_collection(&obj_type, &data) {
                        Ok(collection) => Ok(Value::Collection(collection)),
                        Err(e) => {
                            tracing::warn!("Failed to decode collection: {}, data: {:02x?}", e, &data[..std::cmp::min(20, data.len())]);
                            Ok(Value::Bytes(data))
                        }
                    }
                } else {
                    let mut obj = DbObject::new(&type_name);
                    obj.set("_raw_data", Value::Bytes(data));
                    Ok(Value::Collection(obj))
                }
            }
        }
    }

    pub(crate) fn parse_lob_value(&self, buf: &mut ReadBuffer, col: &ColumnInfo) -> Result<Value> {
        let num_bytes = buf.read_ub4()?;

        if num_bytes == 0 {
            if col.oracle_type == OracleType::Json || col.is_json {
                return Ok(Value::Json(serde_json::Value::Null));
            }
            if col.oracle_type == OracleType::Vector {
                return Ok(Value::Null);
            }
            return Ok(Value::Lob(LobValue::Null));
        }

        let (size, chunk_size) = if col.oracle_type == OracleType::Bfile {
            (0u64, 0u32)
        } else {
            let size = buf.read_ub8()?;
            let chunk_size = buf.read_ub4()?;
            (size, chunk_size)
        };

        let data_bytes = buf.read_bytes_with_length()?;

        if col.oracle_type == OracleType::Json || col.is_json {
            let _locator = buf.read_bytes_with_length()?;

            if let Some(data) = data_bytes {
                if !data.is_empty() {
                    match OsonDecoder::decode(bytes::Bytes::from(data)) {
                        Ok(json_value) => return Ok(Value::Json(json_value)),
                        Err(e) => {
                            tracing::warn!("Failed to decode OSON: {}", e);
                            return Ok(Value::Json(serde_json::Value::Null));
                        }
                    }
                }
            }
            return Ok(Value::Json(serde_json::Value::Null));
        }

        if col.oracle_type == OracleType::Vector {
            let _locator = buf.read_bytes_with_length()?;

            if let Some(data) = data_bytes {
                if !data.is_empty() {
                    match decode_vector(&data) {
                        Ok(vector) => return Ok(Value::Vector(vector)),
                        Err(e) => {
                            tracing::warn!("Failed to decode VECTOR: {}", e);
                            return Ok(Value::Null);
                        }
                    }
                }
            }
            return Ok(Value::Null);
        }

        if let Some(locator_data) = data_bytes {
            if !locator_data.is_empty() {
                let locator = LobLocator::new(
                    bytes::Bytes::from(locator_data),
                    size,
                    chunk_size,
                    col.oracle_type,
                    col.csfrm,
                );
                return Ok(Value::Lob(LobValue::locator(locator)));
            }
        }

        if size == 0 {
            return Ok(Value::Lob(LobValue::Empty));
        }

        Ok(Value::Lob(LobValue::Empty))
    }

    pub(crate) fn parse_error_info(&self, buf: &mut ReadBuffer) -> Result<(u32, Option<String>, u16)> {
        let _call_status = buf.read_ub4()?;
        buf.skip_ub2()?;
        buf.skip_ub4()?;
        buf.skip_ub2()?;
        buf.skip_ub2()?;
        buf.skip_ub2()?;
        let cursor_id = buf.read_ub2()?;
        let _error_pos = buf.read_sb2()?;
        buf.skip_ub1()?;
        buf.skip_ub1()?;
        buf.skip_ub1()?;
        buf.skip_ub1()?;
        buf.skip_ub1()?;
        buf.skip_ub1()?;
        buf.skip_ub4()?;
        buf.skip_ub2()?;
        buf.skip_ub1()?;
        buf.skip_ub4()?;
        buf.skip_ub2()?;
        buf.skip_ub4()?;
        buf.skip_ub1()?;
        buf.skip_ub1()?;
        buf.skip_ub2()?;
        buf.skip_ub4()?;
        let oerrdd_len = buf.read_ub4()?;
        if oerrdd_len > 0 {
            buf.skip_raw_bytes_chunked()?;
        }

        let num_batch_errors = buf.read_ub2()?;
        if num_batch_errors > 0 {
            buf.skip_ub1()?;
            for _ in 0..num_batch_errors {
                buf.skip_ub2()?;
            }
        }

        let num_offsets = buf.read_ub4()?;
        if num_offsets > 0 {
            buf.skip_ub1()?;
            for _ in 0..num_offsets {
                buf.skip_ub4()?;
            }
        }

        let num_batch_msgs = buf.read_ub2()?;
        if num_batch_msgs > 0 {
            buf.skip_ub1()?;
            for _ in 0..num_batch_msgs {
                buf.skip_ub2()?;
                buf.read_string_with_length()?;
                buf.skip(2)?;
            }
        }

        let error_code = buf.read_ub4()?;
        let _row_count = buf.read_ub8()?;

        let error_msg = if error_code != 0 {
            buf.read_string_with_length()?.map(|s| s.trim().to_string())
        } else {
            None
        };

        Ok((error_code, error_msg, cursor_id))
    }

    pub(crate) fn parse_error_response(&self, payload: &[u8]) -> Result<QueryResult> {
        if payload.len() < 3 {
            return Err(Error::Protocol("Error response too short".to_string()));
        }

        let mut buf = ReadBuffer::from_slice(payload);

        buf.skip(2)?;

        let msg_type = buf.read_u8()?;

        if msg_type == MessageType::Error as u8 {
            let _call_status = buf.read_ub4()?;
            buf.skip_ub2()?;
            buf.skip_ub4()?;
            buf.skip_ub2()?;
            buf.skip_ub2()?;
            buf.skip_ub2()?;
            let _cursor_id = buf.read_ub2()?;
            let _error_pos = buf.read_sb2()?;
            buf.skip_ub1()?;
            buf.skip_ub1()?;
            buf.skip_ub1()?;
            buf.skip_ub1()?;
            buf.skip_ub1()?;
            buf.skip_ub1()?;

            buf.skip_ub4()?;
            buf.skip_ub2()?;
            buf.skip_ub1()?;
            buf.skip_ub4()?;
            buf.skip_ub2()?;

            buf.skip_ub4()?;
            buf.skip_ub1()?;
            buf.skip_ub1()?;
            buf.skip_ub2()?;
            buf.skip_ub4()?;

            let oerrdd_len = buf.read_ub4()?;
            if oerrdd_len > 0 {
                buf.skip_raw_bytes_chunked()?;
            }

            let num_batch_errors = buf.read_ub2()?;
            if num_batch_errors > 0 {
                buf.skip_ub1()?;
                for _ in 0..num_batch_errors {
                    buf.skip_ub2()?;
                }
            }

            let num_offsets = buf.read_ub4()?;
            if num_offsets > 0 {
                buf.skip_ub1()?;
                for _ in 0..num_offsets {
                    buf.skip_ub4()?;
                }
            }

            let num_batch_msgs = buf.read_ub2()?;
            if num_batch_msgs > 0 {
                buf.skip_ub1()?;
                for _ in 0..num_batch_msgs {
                    buf.skip_ub2()?;
                    buf.read_string_with_length()?;
                    buf.skip(2)?;
                }
            }

            let error_num = buf.read_ub4()?;
            let _row_count = buf.read_ub8()?;

            let error_msg = if error_num != 0 {
                buf.read_string_with_length()?.map(|s| s.trim().to_string())
            } else {
                None
            };

            return Err(Error::OracleError {
                code: error_num,
                message: error_msg.unwrap_or_else(|| format!("ORA-{:05}", error_num)),
            });
        }

        Err(Error::Protocol(format!(
            "Expected error message type 4, got {}",
            msg_type
        )))
    }

    pub(crate) fn parse_dml_response(&self, payload: &[u8]) -> Result<QueryResult> {
        if payload.len() < 3 {
            return Err(Error::Protocol("DML response too short".to_string()));
        }

        let mut buf = ReadBuffer::from_slice(payload);

        buf.skip(2)?;

        let mut rows_affected: u64 = 0;
        let mut cursor_id: u16 = 0;
        let mut end_of_response = false;

        while !end_of_response && buf.remaining() > 0 {
            let msg_type = buf.read_u8()?;

            match msg_type {
                x if x == MessageType::Error as u8 => {
                    let (error_code, error_msg, cid, row_count) = self.parse_error_info_with_rowcount(&mut buf)?;
                    cursor_id = cid;
                    rows_affected = row_count;
                    if error_code != 0 && error_code != 1403 {
                        return Err(Error::OracleError {
                            code: error_code,
                            message: error_msg.unwrap_or_default(),
                        });
                    }
                }

                x if x == MessageType::Parameter as u8 => {
                    self.parse_return_parameters(&mut buf)?;
                }

                x if x == MessageType::Status as u8 => {
                    let _call_status = buf.read_ub4()?;
                    let _end_to_end_seq = buf.read_ub2()?;
                }

                21 => {
                    let _num_columns_sent = buf.read_ub2()?;
                    if buf.remaining() > 0 {
                        let _byte = buf.read_u8()?;
                    }
                }

                29 => {
                    end_of_response = true;
                }

                _ => {}
            }
        }

        Ok(QueryResult {
            columns: Vec::new(),
            rows: Vec::new(),
            rows_affected,
            has_more_rows: false,
            cursor_id,
        })
    }

    pub(crate) fn parse_error_info_with_rowcount(&self, buf: &mut ReadBuffer) -> Result<(u32, Option<String>, u16, u64)> {
        let _call_status = buf.read_ub4()?;
        buf.skip_ub2()?;
        buf.skip_ub4()?;
        buf.skip_ub2()?;
        buf.skip_ub2()?;
        buf.skip_ub2()?;
        let cursor_id = buf.read_ub2()?;
        let _error_pos = buf.read_sb2()?;
        buf.skip_ub1()?;
        buf.skip_ub1()?;
        buf.skip_ub1()?;
        buf.skip_ub1()?;
        buf.skip_ub1()?;
        buf.skip_ub1()?;

        buf.skip_ub4()?;
        buf.skip_ub2()?;
        buf.skip_ub1()?;
        buf.skip_ub4()?;
        buf.skip_ub2()?;

        buf.skip_ub4()?;
        buf.skip_ub1()?;
        buf.skip_ub1()?;
        buf.skip_ub2()?;
        buf.skip_ub4()?;

        let oerrdd_len = buf.read_ub4()?;
        if oerrdd_len > 0 {
            buf.skip_raw_bytes_chunked()?;
        }

        let num_batch_errors = buf.read_ub2()?;
        if num_batch_errors > 0 {
            buf.skip_ub1()?;
            for _ in 0..num_batch_errors {
                buf.skip_ub2()?;
            }
        }

        let num_offsets = buf.read_ub4()?;
        if num_offsets > 0 {
            buf.skip_ub1()?;
            for _ in 0..num_offsets {
                buf.skip_ub4()?;
            }
        }

        let num_batch_msgs = buf.read_ub2()?;
        if num_batch_msgs > 0 {
            buf.skip_ub1()?;
            for _ in 0..num_batch_msgs {
                buf.skip_ub2()?;
                buf.read_string_with_length()?;
                buf.skip(2)?;
            }
        }

        let error_code = buf.read_ub4()?;
        let row_count = buf.read_ub8()?;

        buf.skip_ub4()?;
        buf.skip_ub4()?;

        let error_msg = if error_code != 0 {
            buf.read_string_with_length()?.map(|s| s.trim().to_string())
        } else {
            None
        };

        Ok((error_code, error_msg, cursor_id, row_count))
    }

    pub(crate) fn parse_error_message_info(&self, buf: &mut ReadBuffer) -> Result<(u32, String, bool)> {
        let _call_status = buf.read_ub4()?;
        buf.skip_ub2()?;
        buf.skip_ub4()?;
        buf.skip_ub2()?;
        buf.skip_ub2()?;
        buf.skip_ub2()?;
        let _cursor_id = buf.read_ub2()?;
        let _error_pos = buf.read_sb2()?;
        buf.skip(1)?;
        buf.skip(1)?;
        buf.skip(1)?;
        buf.skip(1)?;
        buf.skip(1)?;
        buf.skip(1)?;

        buf.skip(4)?;
        buf.skip(2)?;
        buf.skip(1)?;
        buf.skip(4)?;
        buf.skip(2)?;

        buf.skip(4)?;
        buf.skip(1)?;
        buf.skip(1)?;
        buf.skip(2)?;
        buf.skip(4)?;

        let num_bytes = buf.read_ub4()?;
        if num_bytes > 0 {
            buf.skip_raw_bytes_chunked()?;
        }

        let num_batch_errors = buf.read_ub2()?;
        if num_batch_errors > 0 {
            buf.skip(1)?;
            for _ in 0..num_batch_errors {
                buf.skip(2)?;
            }
        }

        let num_offsets = buf.read_ub4()?;
        if num_offsets > 0 {
            buf.skip(1)?;
            for _ in 0..num_offsets {
                buf.skip(4)?;
            }
        }

        let num_batch_msgs = buf.read_ub2()?;
        if num_batch_msgs > 0 {
            buf.skip(1)?;
            for _ in 0..num_batch_msgs {
                buf.skip(2)?;
                let _msg = buf.read_string_with_length()?;
                buf.skip(2)?;
            }
        }

        let error_code = buf.read_ub4()?;

        buf.skip_ub8()?; // row_count
        buf.skip_ub4()?; // sql_type
        buf.skip_ub4()?; // server_checksum

        let error_msg = if error_code != 0 && error_code != 1403 {
            buf.read_string_with_length()?.unwrap_or_default()
        } else {
            String::new()
        };

        let has_more_rows = error_code == 0;

        Ok((error_code, error_msg, has_more_rows))
    }

    pub(crate) fn parse_batch_response(
        &self,
        payload: &[u8],
        num_rows_sent: usize,
        want_row_counts: bool,
    ) -> Result<crate::batch::BatchResult> {
        if payload.len() < 3 {
            return Err(Error::Protocol("Batch response too short".to_string()));
        }

        let mut buf = ReadBuffer::from_slice(payload);
        buf.skip(2)?;

        let mut total_rows_affected: u64 = 0;
        let mut row_counts: Option<Vec<u64>> = None;
        let mut end_of_response = false;

        while !end_of_response && buf.remaining() > 0 {
            let msg_type = buf.read_u8()?;

            match msg_type {
                x if x == MessageType::Error as u8 => {
                    let (error_code, error_msg, _cid, row_count) = self.parse_error_info_with_rowcount(&mut buf)?;
                    total_rows_affected = row_count;
                    if error_code != 0 && error_code != 1403 {
                        return Err(Error::OracleError {
                            code: error_code,
                            message: error_msg.unwrap_or_default(),
                        });
                    }
                }

                x if x == MessageType::Parameter as u8 => {
                    let parsed_counts = self.parse_return_parameters_internal(&mut buf, want_row_counts)?;
                    if want_row_counts && parsed_counts.is_some() {
                        row_counts = parsed_counts;
                    }
                }

                x if x == MessageType::Status as u8 => {
                    let _call_status = buf.read_ub4()?;
                    let _end_to_end_seq = buf.read_ub2()?;
                }

                29 => {
                    end_of_response = true;
                }

                _ => {}
            }
        }

        let mut result = crate::batch::BatchResult::new();
        result.total_rows_affected = total_rows_affected;
        result.success_count = num_rows_sent;
        if want_row_counts {
            result.row_counts = Some(row_counts.unwrap_or_else(|| vec![1; num_rows_sent]));
        }
        Ok(result)
    }

    pub(crate) fn parse_describe_info(&self, buf: &mut ReadBuffer, ttc_field_version: u8) -> Result<Vec<ColumnInfo>> {
        use crate::constants::ccap_value;

        buf.skip_ub4()?;

        let num_columns = buf.read_ub4()? as usize;
        if num_columns == 0 {
            return Ok(Vec::new());
        }

        buf.skip_ub1()?;

        let mut columns = Vec::with_capacity(num_columns);

        for _col_idx in 0..num_columns {
            let ora_type_num = buf.read_u8()?;
            buf.skip_ub1()?;
            let precision = buf.read_u8()?;
            let scale = buf.read_u8()?;
            let buffer_size = buf.read_ub4()?;

            buf.skip_ub4()?;
            buf.skip_ub8()?;
            let _oid = buf.read_bytes_with_length()?;
            buf.skip_ub2()?;
            buf.skip_ub2()?;
            let _csfrm = buf.read_u8()?;
            let max_size = buf.read_ub4()?;

            if ttc_field_version >= ccap_value::FIELD_VERSION_12_2 {
                buf.skip_ub4()?;
            }

            let _nulls_allowed = buf.read_u8()?;
            buf.skip_ub1()?;
            let name = buf.read_string_with_ub4_length()?.unwrap_or_default();
            let _schema = buf.read_string_with_ub4_length()?;
            let _type_name = buf.read_string_with_ub4_length()?;
            buf.skip_ub2()?;
            buf.skip_ub4()?;

            if ttc_field_version >= ccap_value::FIELD_VERSION_23_1 {
                let _domain_schema = buf.read_string_with_ub4_length()?;
                let _domain_name = buf.read_string_with_ub4_length()?;
            }

            if ttc_field_version >= 20 {
                let num_annotations = buf.read_ub4()?;
                if num_annotations > 0 {
                    buf.skip_ub1()?;
                    let actual_num = buf.read_ub4()?;
                    buf.skip_ub1()?;
                    for _ in 0..actual_num {
                        let _key = buf.read_string_with_ub4_length()?;
                        let _value = buf.read_string_with_ub4_length()?;
                        buf.skip_ub4()?;
                    }
                    buf.skip_ub4()?;
                }
            }

            if ttc_field_version >= ccap_value::FIELD_VERSION_23_4 {
                buf.skip_ub4()?;
                buf.skip_ub1()?;
                buf.skip_ub1()?;
            }

            let oracle_type = crate::constants::OracleType::try_from(ora_type_num)
                .unwrap_or(crate::constants::OracleType::Varchar);

            let mut col = ColumnInfo::new(&name, oracle_type);
            col.data_size = if max_size > 0 { max_size } else { buffer_size };
            col.precision = precision as i16;
            col.scale = scale as i16;
            columns.push(col);
        }

        let current_date_indicator = buf.read_ub4()?;
        if current_date_indicator > 0 {
            buf.skip_raw_bytes_chunked()?;
        }

        buf.skip_ub4()?;
        buf.skip_ub4()?;
        buf.skip_ub4()?;
        buf.skip_ub4()?;

        let dcbqcky_indicator = buf.read_ub4()?;
        if dcbqcky_indicator > 0 {
            buf.skip_raw_bytes_chunked()?;
        }

        Ok(columns)
    }
}
