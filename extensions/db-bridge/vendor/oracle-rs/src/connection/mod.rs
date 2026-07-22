//! Oracle database connection
//!
//! This module provides the main `Connection` type for interacting with Oracle databases.

pub mod execute;
pub mod inner;
pub mod lob;
pub mod response;
pub mod types;

pub use types::{ConnectionState, PlsqlResult, QueryOptions, QueryResult, ServerInfo};

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::buffer::{ReadBuffer, WriteBuffer};
use crate::config::{AuthMode, Config, ServiceMethod};
use crate::constants::{FunctionCode, MessageType, PacketType, PACKET_HEADER_SIZE};
use crate::error::{Error, Result};
use crate::messages::{AcceptMessage, AuthMessage, AuthPhase, ConnectMessage};
use crate::packet::Packet;
use crate::transport::{connect_tls, TlsConfig};

use inner::{ConnectionInner, OracleStream};

/// An Oracle database connection.
///
/// This is the main type for interacting with Oracle databases. It provides
/// methods for executing queries, DML statements, PL/SQL blocks, and managing
/// transactions.
///
/// Connections are created using [`Connection::connect`] or
/// [`Connection::connect_with_config`]. For connection pooling, use the
/// `deadpool-oracle` crate.
///
/// # Example
///
/// ```rust,no_run
/// use oracle_rs::{Config, Connection, Value};
///
/// # async fn example() -> oracle_rs::Result<()> {
/// // Create a connection
/// let config = Config::new("localhost", 1521, "FREEPDB1", "user", "password");
/// let conn = Connection::connect_with_config(config).await?;
///
/// // Execute a query
/// let result = conn.query("SELECT * FROM employees WHERE dept_id = :1", &[10.into()]).await?;
/// for row in &result.rows {
///     let name = row.get_by_name("name").and_then(|v| v.as_str()).unwrap_or("");
///     println!("Employee: {}", name);
/// }
///
/// // Execute DML with transaction
/// conn.execute("INSERT INTO logs (msg) VALUES (:1)", &["Hello".into()]).await?;
/// conn.commit().await?;
///
/// // Close the connection
/// conn.close().await?;
/// # Ok(())
/// # }
/// ```
///
/// # Thread Safety
///
/// `Connection` is `Send` and `Sync`, but operations are serialized internally
/// via a mutex. For parallel query execution, use multiple connections (e.g.,
/// via a connection pool).
pub struct Connection {
    pub(crate) inner: Arc<Mutex<ConnectionInner>>,
    pub(crate) config: Config,
    pub(crate) closed: AtomicBool,
    pub(crate) id: u32,
}

static CONNECTION_ID_COUNTER: AtomicU32 = AtomicU32::new(1);

impl Connection {
    /// Create a new connection to an Oracle database
    ///
    /// # Arguments
    ///
    /// * `connect_string` - Connection string in EZConnect format (e.g., "host:port/service")
    /// * `username` - Database username
    /// * `password` - Database password
    pub async fn connect(connect_string: &str, username: &str, password: &str) -> Result<Self> {
        let mut config: Config = connect_string.parse()?;
        config.username = username.to_string();
        config.set_password(password);
        Self::connect_with_config(config).await
    }

    /// Create a new connection using a [`Config`].
    pub async fn connect_with_config(config: Config) -> Result<Self> {
        let id = CONNECTION_ID_COUNTER.fetch_add(1, Ordering::Relaxed);

        let addr = config.socket_addr();
        let tcp_stream = TcpStream::connect(&addr).await?;

        tcp_stream.set_nodelay(true)?;

        let stream = if config.is_tls_enabled() {
            let tls_config = config.tls_config.as_ref()
                .cloned()
                .unwrap_or_else(TlsConfig::new);

            let tls_stream = connect_tls(tcp_stream, &config.host, &tls_config).await?;
            OracleStream::Tls(tls_stream)
        } else {
            OracleStream::Plain(tcp_stream)
        };

        let mut inner = ConnectionInner::new_with_cache(config.stmtcachesize);
        inner.stream = Some(stream);
        inner.state = ConnectionState::Connected;

        let conn = Connection {
            inner: Arc::new(Mutex::new(inner)),
            config,
            closed: AtomicBool::new(false),
            id,
        };

        conn.perform_handshake().await?;

        Ok(conn)
    }

    /// Get the connection ID
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Check if the connection is closed
    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Relaxed)
    }

    /// Mark the connection as closed
    pub fn mark_closed(&self) {
        self.closed.store(true, Ordering::Relaxed);
    }

    /// Helper to mark connection as closed if the result is a connection error
    pub(crate) fn handle_result<T>(&self, result: Result<T>) -> Result<T> {
        if let Err(ref e) = result {
            if e.is_connection_error() {
                self.mark_closed();
            }
        }
        result
    }

    /// Get server information
    pub async fn server_info(&self) -> ServerInfo {
        let inner = self.inner.lock().await;
        inner.server_info.clone()
    }

    /// Get the current connection state
    pub async fn state(&self) -> ConnectionState {
        let inner = self.inner.lock().await;
        inner.state
    }

    /// Perform the connection handshake
    async fn perform_handshake(&self) -> Result<()> {
        self.send_connect_packet().await?;

        let needs_oob_check = {
            let inner = self.inner.lock().await;
            inner.server_info.protocol_version >= crate::constants::version::MIN_OOB_CHECK
                && inner.server_info.supports_oob
        };
        if needs_oob_check {
            self.send_oob_check().await?;
        }

        self.negotiate_protocol().await?;
        self.negotiate_data_types().await?;
        self.authenticate().await?;

        Ok(())
    }

    /// Send OOB (Out of Band) check
    async fn send_oob_check(&self) -> Result<()> {
        let mut inner = self.inner.lock().await;
        let large_sdu = inner.large_sdu;

        inner.send(&[0x21]).await?;

        let marker_payload = [1u8, 0u8, crate::constants::MarkerType::Reset as u8];
        let mut packet_buf = WriteBuffer::new();

        if large_sdu {
            packet_buf.write_u32_be((PACKET_HEADER_SIZE + marker_payload.len()) as u32)?;
        } else {
            packet_buf.write_u16_be((PACKET_HEADER_SIZE + marker_payload.len()) as u16)?;
            packet_buf.write_u16_be(0)?;
        }
        packet_buf.write_u8(PacketType::Marker as u8)?;
        packet_buf.write_u8(0)?;
        packet_buf.write_u16_be(0)?;
        packet_buf.write_bytes(&marker_payload)?;

        inner.send(&packet_buf.freeze()).await?;

        let response = inner.receive().await?;

        if response.len() > 4 && response[4] == PacketType::Marker as u8 {
            Ok(())
        } else {
            Ok(())
        }
    }

    /// Send the initial CONNECT packet
    async fn send_connect_packet(&self) -> Result<()> {
        let mut inner = self.inner.lock().await;

        let connect_msg = ConnectMessage::from_config(&self.config);
        let (connect_packet, continuation) = connect_msg.build_with_continuation()?;

        inner.send(&connect_packet).await?;

        if let Some(ref data_packet) = continuation {
            inner.send(data_packet).await?;
        }

        const MAX_RESENDS: u8 = 3;
        let mut resend_count: u8 = 0;

        loop {
            let response = inner.receive().await?;

            if response.len() < PACKET_HEADER_SIZE {
                return Err(Error::PacketTooShort {
                    expected: PACKET_HEADER_SIZE,
                    actual: response.len(),
                });
            }

            let packet_type = response[4];

            match packet_type {
                2 => {
                    let packet = Packet::from_bytes(response)?;
                    let accept = AcceptMessage::parse(&packet)?;

                    inner.large_sdu = accept.uses_large_sdu();

                    inner.server_info.protocol_version = accept.protocol_version;
                    inner.server_info.supports_oob = accept.supports_oob;
                    inner.sdu_size = accept.sdu.min(65535) as u16;

                    inner.state = ConnectionState::Connected;
                    return Ok(());
                }
                4 => {
                    let mut buf = ReadBuffer::new(response.slice(PACKET_HEADER_SIZE..));
                    let _reason = buf.read_u8()?;
                    let _user_reason = buf.read_u8()?;

                    return Err(Error::ConnectionRefused {
                        error_code: None,
                        message: Some("Connection refused by server".to_string()),
                    });
                }
                5 => {
                    return Err(Error::ConnectionRedirect(
                        "redirect not implemented".to_string(),
                    ));
                }
                11 => {
                    resend_count += 1;
                    if resend_count > MAX_RESENDS {
                        return Err(Error::ProtocolError(
                            "Server requested too many resends during connect".to_string(),
                        ));
                    }
                    inner.send(&connect_packet).await?;
                    if let Some(ref data_packet) = continuation {
                        inner.send(data_packet).await?;
                    }
                }
                _ => {
                    return Err(Error::ProtocolError(format!(
                        "Unexpected packet type during connect: {}",
                        packet_type,
                    )));
                }
            }
        }
    }

    /// Negotiate protocol version and capabilities
    async fn negotiate_protocol(&self) -> Result<()> {
        use crate::messages::ProtocolMessage;

        let mut inner = self.inner.lock().await;
        let large_sdu = inner.large_sdu;

        let protocol_msg = ProtocolMessage::new();
        let packet = protocol_msg.build_request(large_sdu)?;
        inner.send(&packet).await?;

        let response = inner.receive().await?;

        if response.len() <= 4 || response[4] != PacketType::Data as u8 {
            return Err(Error::ProtocolError("Protocol negotiation failed".to_string()));
        }

        let payload = &response[PACKET_HEADER_SIZE..];
        let mut protocol_msg = ProtocolMessage::new();
        protocol_msg.parse_response(payload, &mut inner.capabilities)?;

        if let Some(banner) = &protocol_msg.server_banner {
            inner.server_info.banner = banner.clone();
        }

        inner.state = ConnectionState::ProtocolNegotiated;
        Ok(())
    }

    /// Negotiate data types
    async fn negotiate_data_types(&self) -> Result<()> {
        use crate::messages::DataTypesMessage;

        let mut inner = self.inner.lock().await;
        let large_sdu = inner.large_sdu;

        let data_types_msg = DataTypesMessage::new();
        let packet = data_types_msg.build_request(&inner.capabilities, large_sdu)?;
        inner.send(&packet).await?;

        let response = inner.receive().await?;

        if response.len() > 4 && response[4] == PacketType::Data as u8 {
            inner.state = ConnectionState::DataTypesNegotiated;
            Ok(())
        } else {
            Err(Error::ProtocolError("Data types negotiation failed".to_string()))
        }
    }

    /// Perform authentication
    async fn authenticate(&self) -> Result<()> {
        let service_name = match &self.config.service {
            ServiceMethod::ServiceName(name) => name.clone(),
            ServiceMethod::Sid(sid) => sid.clone(),
        };

        let mut auth = AuthMessage::new(
            &self.config.username,
            self.config.password().as_bytes(),
            &service_name,
        );

        auth = match self.config.auth_mode {
            AuthMode::Normal => auth,
            AuthMode::SysDba => auth.with_sysdba(),
            AuthMode::SysOper => auth.with_sysoper(),
        };

        {
            let mut inner = self.inner.lock().await;
            let large_sdu = inner.large_sdu;
            let request = auth.build_request(&inner.capabilities, large_sdu)?;
            inner.send(&request).await?;

            let response = inner.receive().await?;
            if response.len() <= PACKET_HEADER_SIZE {
                return Err(Error::Protocol("Empty auth response".to_string()));
            }

            if response.len() > PACKET_HEADER_SIZE + 2 {
                let msg_type = response[PACKET_HEADER_SIZE + 2];
                if msg_type == MessageType::Error as u8 {
                    return Err(Error::AuthenticationFailed(
                        "Server rejected authentication phase one".to_string(),
                    ));
                }
            }

            auth.parse_response(&response[PACKET_HEADER_SIZE..])?;
        }

        if auth.phase() == AuthPhase::Two {
            let mut inner = self.inner.lock().await;
            let large_sdu = inner.large_sdu;
            let request = auth.build_request(&inner.capabilities, large_sdu)?;
            inner.send(&request).await?;

            let response = inner.receive().await?;
            if response.len() <= PACKET_HEADER_SIZE {
                return Err(Error::Protocol("Empty auth phase two response".to_string()));
            }

            let packet_type = response[4];
            if packet_type == 12 {
                return Err(Error::AuthenticationFailed("Server sent MARKER - authentication rejected".to_string()));
            }

            if response.len() > PACKET_HEADER_SIZE + 2 {
                let msg_type = response[PACKET_HEADER_SIZE + 2];
                if msg_type == MessageType::Error as u8 {
                    return Err(Error::InvalidCredentials);
                }
            }

            auth.parse_response(&response[PACKET_HEADER_SIZE..])?;
        }

        if !auth.is_complete() {
            return Err(Error::AuthenticationFailed(
                "Authentication did not complete".to_string(),
            ));
        }

        let mut inner = self.inner.lock().await;
        if let Some(combo_key) = auth.combo_key() {
            inner.capabilities.combo_key = Some(combo_key.to_vec());
        }
        inner.sequence_number = 2;
        inner.state = ConnectionState::Ready;

        Ok(())
    }

    /// Commit the current transaction.
    pub async fn commit(&self) -> Result<()> {
        self.ensure_ready().await?;
        self.execute("COMMIT", &[]).await?;
        Ok(())
    }

    /// Rollback the current transaction.
    pub async fn rollback(&self) -> Result<()> {
        self.ensure_ready().await?;
        self.execute("ROLLBACK", &[]).await?;
        Ok(())
    }

    /// Create a savepoint within the current transaction
    pub async fn savepoint(&self, name: &str) -> Result<()> {
        self.ensure_ready().await?;
        self.execute(&format!("SAVEPOINT {}", name), &[]).await?;
        Ok(())
    }

    /// Rollback to a previously created savepoint
    pub async fn rollback_to_savepoint(&self, name: &str) -> Result<()> {
        self.ensure_ready().await?;
        self.execute(&format!("ROLLBACK TO SAVEPOINT {}", name), &[]).await?;
        Ok(())
    }

    /// Ping the server to check if the connection is still alive.
    pub async fn ping(&self) -> Result<()> {
        self.ensure_ready().await?;
        self.query("SELECT 1 FROM DUAL", &[]).await?;
        Ok(())
    }

    /// Clear the statement cache
    pub async fn clear_statement_cache(&self) {
        let mut inner = self.inner.lock().await;
        if let Some(ref mut cache) = inner.statement_cache {
            cache.clear();
        }
    }

    /// Close the connection.
    pub async fn close(&self) -> Result<()> {
        if self.closed.swap(true, Ordering::Relaxed) {
            return Ok(());
        }

        let mut inner = self.inner.lock().await;

        if inner.state == ConnectionState::Ready {
            let _ = self.send_simple_function_inner(&mut inner, FunctionCode::Logoff).await;
        }

        inner.state = ConnectionState::Closed;

        if let Some(stream) = inner.stream.take() {
            drop(stream);
        }

        Ok(())
    }

    /// Send a marker packet to the server
    async fn send_marker(&self, inner: &mut ConnectionInner, marker_type: u8) -> Result<()> {
        let mut packet_buf = WriteBuffer::new();

        let packet_len = PACKET_HEADER_SIZE + 3;
        packet_buf.write_u16_be(packet_len as u16)?;
        packet_buf.write_u16_be(0)?;
        packet_buf.write_u8(PacketType::Marker as u8)?;
        packet_buf.write_u8(0)?;
        packet_buf.write_u16_be(0)?;

        packet_buf.write_u8(1)?;
        packet_buf.write_u8(0)?;
        packet_buf.write_u8(marker_type)?;

        inner.send(&packet_buf.freeze()).await
    }

    pub(crate) async fn send_simple_function_inner(
        &self,
        inner: &mut ConnectionInner,
        function_code: FunctionCode,
    ) -> Result<()> {
        let mut buf = WriteBuffer::new();

        let seq_num = inner.next_sequence_number();

        buf.write_u16_be(0)?;
        buf.write_u8(MessageType::Function as u8)?;
        buf.write_u8(function_code as u8)?;
        buf.write_u8(seq_num)?;

        if inner.capabilities.ttc_field_version >= 18 {
            buf.write_ub8(0)?;
        }

        let data_payload = buf.freeze();
        let mut packet_buf = WriteBuffer::new();
        let packet_len = PACKET_HEADER_SIZE + data_payload.len();
        packet_buf.write_u16_be(packet_len as u16)?;
        packet_buf.write_u16_be(0)?;
        packet_buf.write_u8(PacketType::Data as u8)?;
        packet_buf.write_u8(0)?;
        packet_buf.write_u16_be(0)?;
        packet_buf.write_bytes(&data_payload)?;

        let packet_bytes = packet_buf.freeze();
        inner.send(&packet_bytes).await?;

        let response = inner.receive().await?;

        if response.len() <= 4 {
            return Err(Error::Protocol("Response too short".to_string()));
        }

        let packet_type = response[4];

        if packet_type == PacketType::Marker as u8 {
            if response.len() >= PACKET_HEADER_SIZE + 3 {
                let marker_type = response[PACKET_HEADER_SIZE + 2];

                if marker_type == 1 {
                    if function_code == FunctionCode::Logoff {
                        inner.state = ConnectionState::Closed;
                        return Ok(());
                    }

                    if let Err(e) = self.send_marker(inner, 2).await {
                        inner.state = ConnectionState::Closed;
                        return Err(e);
                    }

                    let mut current_packet_type: u8;
                    loop {
                        match inner.receive().await {
                            Ok(pkt) => {
                                if pkt.len() < PACKET_HEADER_SIZE + 1 {
                                    break;
                                }
                                current_packet_type = pkt[4];

                                if current_packet_type == PacketType::Marker as u8 {
                                    if pkt.len() >= PACKET_HEADER_SIZE + 3 {
                                        let mk_type = pkt[PACKET_HEADER_SIZE + 2];
                                        if mk_type == 2 {
                                            break;
                                        }
                                    }
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
                                if pkt.len() < PACKET_HEADER_SIZE + 1 {
                                    break;
                                }
                                current_packet_type = pkt[4];

                                if current_packet_type == PacketType::Marker as u8 {
                                    continue;
                                }

                                if current_packet_type == PacketType::Data as u8 {
                                    if pkt.len() > PACKET_HEADER_SIZE + 2 {
                                        let msg_type = pkt[PACKET_HEADER_SIZE + 2];
                                        if msg_type == MessageType::Error as u8 {
                                            let payload = &pkt[PACKET_HEADER_SIZE..];
                                            let mut buf = ReadBuffer::from_slice(payload);
                                            buf.skip(2)?;
                                            buf.skip(1)?;
                                            let (error_code, error_msg, _) = self.parse_error_info(&mut buf)?;
                                            if error_code != 0 {
                                                return Err(Error::OracleError {
                                                    code: error_code,
                                                    message: error_msg.unwrap_or_else(|| format!("ORA-{:05}", error_code)),
                                                });
                                            }
                                        }
                                    }
                                }
                                break;
                            }
                            Err(_) => {
                                if matches!(function_code,
                                    FunctionCode::Logoff |
                                    FunctionCode::Commit |
                                    FunctionCode::Rollback
                                ) {
                                    inner.state = ConnectionState::Closed;
                                    return Ok(());
                                }
                                inner.state = ConnectionState::Closed;
                                if function_code == FunctionCode::Ping {
                                    return Ok(());
                                }
                                return Ok(());
                            }
                        }
                    }

                    return Ok(());
                }
            }
            return Ok(());
        }

        if packet_type == PacketType::Data as u8 {
            if response.len() > PACKET_HEADER_SIZE + 2 {
                let msg_type = response[PACKET_HEADER_SIZE + 2];
                if msg_type == MessageType::Error as u8 {
                    let payload = &response[PACKET_HEADER_SIZE..];
                    let mut buf = ReadBuffer::from_slice(payload);
                    buf.skip(2)?;
                    buf.skip(1)?;
                    let (error_code, error_msg, _) = self.parse_error_info(&mut buf)?;
                    if error_code != 0 {
                        return Err(Error::OracleError {
                            code: error_code,
                            message: error_msg.unwrap_or_else(|| format!("ORA-{:05}", error_code)),
                        });
                    }
                }
            }
            return Ok(());
        }

        Err(Error::Protocol(format!("Unexpected packet type {} for function call", packet_type)))
    }

    /// Ensure the connection is ready for operations
    pub(crate) async fn ensure_ready(&self) -> Result<()> {
        if self.is_closed() {
            return Err(Error::ConnectionClosed);
        }

        let inner = self.inner.lock().await;
        if inner.state != ConnectionState::Ready {
            return Err(Error::ConnectionNotReady);
        }

        Ok(())
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.closed.store(true, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::row::{Row, Value};
    use crate::statement::ColumnInfo;

    #[test]
    fn test_query_options_default() {
        let opts = QueryOptions::default();
        assert_eq!(opts.prefetch_rows, 100);
        assert_eq!(opts.array_size, 100);
        assert!(!opts.auto_commit);
    }

    #[test]
    fn test_query_result_empty() {
        let result = QueryResult::empty();
        assert!(result.is_empty());
        assert_eq!(result.column_count(), 0);
        assert_eq!(result.row_count(), 0);
        assert!(result.first().is_none());
    }

    #[test]
    fn test_query_result_with_rows() {
        let columns = vec![ColumnInfo::new("ID", crate::constants::OracleType::Number)];
        let rows = vec![Row::new(vec![Value::Integer(1)])];

        let result = QueryResult {
            columns,
            rows,
            rows_affected: 0,
            has_more_rows: false,
            cursor_id: 1,
        };

        assert!(!result.is_empty());
        assert_eq!(result.column_count(), 1);
        assert_eq!(result.row_count(), 1);
        assert!(result.first().is_some());
        assert!(result.column_by_name("ID").is_some());
        assert!(result.column_by_name("id").is_some());
        assert_eq!(result.column_index("ID"), Some(0));
    }

    #[test]
    fn test_server_info_default() {
        let info = ServerInfo::default();
        assert!(info.version.is_empty());
        assert_eq!(info.session_id, 0);
    }

    #[test]
    fn test_connection_state_transitions() {
        assert_eq!(ConnectionState::Disconnected, ConnectionState::Disconnected);
        assert_ne!(ConnectionState::Connected, ConnectionState::Ready);
    }

    #[test]
    fn test_query_result_iterator() {
        let rows = vec![
            Row::new(vec![Value::Integer(1)]),
            Row::new(vec![Value::Integer(2)]),
        ];
        let result = QueryResult {
            columns: vec![],
            rows,
            rows_affected: 0,
            has_more_rows: false,
            cursor_id: 0,
        };

        let collected: Vec<_> = result.iter().collect();
        assert_eq!(collected.len(), 2);
    }

    #[test]
    fn test_query_result_into_iterator() {
        let rows = vec![
            Row::new(vec![Value::Integer(1)]),
            Row::new(vec![Value::Integer(2)]),
        ];
        let result = QueryResult {
            columns: vec![],
            rows,
            rows_affected: 0,
            has_more_rows: false,
            cursor_id: 0,
        };

        let collected: Vec<Row> = result.into_iter().collect();
        assert_eq!(collected.len(), 2);
    }
}
