//! Internal TNS stream and connection state handling.

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::buffer::WriteBuffer;
use crate::capabilities::Capabilities;
use crate::constants::{PACKET_HEADER_SIZE, PacketType};
use crate::error::{Error, Result};
use crate::statement_cache::StatementCache;
use crate::transport::TlsOracleStream;

use super::types::{ConnectionState, ServerInfo};

/// Stream type that can be either plain TCP or TLS-encrypted
pub(crate) enum OracleStream {
    /// Plain TCP connection
    Plain(TcpStream),
    /// TLS-encrypted connection
    Tls(TlsOracleStream),
}

impl OracleStream {
    pub(crate) async fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        match self {
            OracleStream::Plain(stream) => {
                AsyncReadExt::read_exact(stream, buf).await?;
                Ok(())
            }
            OracleStream::Tls(stream) => {
                AsyncReadExt::read_exact(stream, buf).await?;
                Ok(())
            }
        }
    }

    pub(crate) async fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        match self {
            OracleStream::Plain(stream) => stream.write_all(buf).await,
            OracleStream::Tls(stream) => stream.write_all(buf).await,
        }
    }

    pub(crate) async fn flush(&mut self) -> std::io::Result<()> {
        match self {
            OracleStream::Plain(stream) => stream.flush().await,
            OracleStream::Tls(stream) => stream.flush().await,
        }
    }
}

/// Internal connection state shared across async operations
pub(crate) struct ConnectionInner {
    pub(crate) stream: Option<OracleStream>,
    pub(crate) capabilities: Capabilities,
    pub(crate) state: ConnectionState,
    pub(crate) server_info: ServerInfo,
    pub(crate) sdu_size: u16,
    pub(crate) large_sdu: bool,
    /// Sequence number for TTC messages (increments per message)
    pub(crate) sequence_number: u8,
    /// Statement cache for prepared statement reuse
    pub(crate) statement_cache: Option<StatementCache>,
}

impl ConnectionInner {
    pub(crate) fn new_with_cache(cache_size: usize) -> Self {
        Self {
            stream: None,
            capabilities: Capabilities::default(),
            state: ConnectionState::Disconnected,
            server_info: ServerInfo::default(),
            sdu_size: 8192,
            large_sdu: false,
            sequence_number: 0,
            statement_cache: if cache_size > 0 {
                Some(StatementCache::new(cache_size))
            } else {
                None
            },
        }
    }

    /// Get the next sequence number (auto-increments, wraps at 255 to 1)
    pub(crate) fn next_sequence_number(&mut self) -> u8 {
        self.sequence_number = self.sequence_number.wrapping_add(1);
        if self.sequence_number == 0 {
            self.sequence_number = 1;
        }
        self.sequence_number
    }

    pub(crate) async fn send(&mut self, data: &[u8]) -> Result<()> {
        if let Some(stream) = &mut self.stream {
            stream.write_all(data).await?;
            stream.flush().await?;
            Ok(())
        } else {
            Err(Error::ConnectionClosed)
        }
    }

    /// Send a payload that may need to be split across multiple packets.
    ///
    /// This is used for large LOB writes and other operations where the payload
    /// exceeds the SDU size. The payload is split into multiple DATA packets,
    /// each with proper headers.
    pub(crate) async fn send_multi_packet(&mut self, payload: &[u8], data_flags: u16) -> Result<()> {
        let stream = self.stream.as_mut().ok_or(Error::ConnectionClosed)?;

        // Calculate max payload per packet: SDU - header (8) - data flags (2)
        let max_payload_per_packet = self.sdu_size as usize - PACKET_HEADER_SIZE - 2;

        let mut offset = 0;
        let mut is_first = true;

        while offset < payload.len() {
            let remaining = payload.len() - offset;
            let chunk_size = std::cmp::min(remaining, max_payload_per_packet);
            let is_last = offset + chunk_size >= payload.len();

            // Build packet
            let packet_len = PACKET_HEADER_SIZE + 2 + chunk_size; // header + data flags + payload
            let mut packet = Vec::with_capacity(packet_len);

            // Header
            if self.large_sdu {
                packet.extend_from_slice(&(packet_len as u32).to_be_bytes());
            } else {
                packet.extend_from_slice(&(packet_len as u16).to_be_bytes());
                packet.extend_from_slice(&[0, 0]); // Checksum
            }
            packet.push(PacketType::Data as u8);
            packet.push(0); // Flags
            packet.extend_from_slice(&[0, 0]); // Header checksum

            // Data flags - only include on first packet
            if is_first {
                packet.extend_from_slice(&data_flags.to_be_bytes());
                is_first = false;
            } else {
                // Continuation packets still need data flags position but value is 0
                packet.extend_from_slice(&0u16.to_be_bytes());
            }

            // Payload chunk
            packet.extend_from_slice(&payload[offset..offset + chunk_size]);

            // Send this packet
            stream.write_all(&packet).await?;

            offset += chunk_size;

            // Don't flush until the last packet to improve performance
            if is_last {
                stream.flush().await?;
            }
        }

        Ok(())
    }

    pub(crate) async fn receive(&mut self) -> Result<bytes::Bytes> {
        if let Some(stream) = &mut self.stream {
            // Read packet header first (always 8 bytes)
            // large_sdu only affects how the length field is interpreted, not header size
            let mut header_buf = vec![0u8; PACKET_HEADER_SIZE];
            stream.read_exact(&mut header_buf).await?;

            // Parse header to get payload length
            // In large_sdu mode, first 4 bytes are length; otherwise first 2 bytes
            let packet_len = if self.large_sdu {
                u32::from_be_bytes([header_buf[0], header_buf[1], header_buf[2], header_buf[3]])
                    as usize
            } else {
                u16::from_be_bytes([header_buf[0], header_buf[1]]) as usize
            };

            // Read remaining payload
            let payload_len = packet_len.saturating_sub(PACKET_HEADER_SIZE);
            let mut payload_buf = vec![0u8; payload_len];
            if payload_len > 0 {
                stream.read_exact(&mut payload_buf).await?;
            }

            // Combine header and payload
            let mut full_packet = header_buf.clone();
            full_packet.extend(payload_buf);

            Ok(bytes::Bytes::from(full_packet))
        } else {
            Err(Error::ConnectionClosed)
        }
    }

    /// Receive a complete response that may span multiple packets
    ///
    /// This method accumulates packets until the END_OF_RESPONSE flag is detected
    /// in the data flags. It's used for operations like LOB reads that may return
    /// data spanning multiple TNS packets.
    ///
    /// Returns the combined payload of all packets (excluding headers).
    pub(crate) async fn receive_response(&mut self) -> Result<bytes::Bytes> {
        use crate::constants::{data_flags, MessageType};

        let mut accumulated_payload = Vec::new();
        let mut is_first_packet = true;

        loop {
            let packet = self.receive().await?;

            if packet.len() < PACKET_HEADER_SIZE {
                return Err(Error::Protocol("Packet too small".to_string()));
            }

            // Check packet type - only DATA packets can be accumulated
            let packet_type = packet[4];
            if packet_type != PacketType::Data as u8 {
                // Non-DATA packet (e.g., MARKER) - return as-is for special handling
                return Ok(packet);
            }

            // Get payload (everything after the 8-byte header)
            let payload = &packet[PACKET_HEADER_SIZE..];

            if payload.len() < 2 {
                return Err(Error::Protocol("DATA packet payload too small".to_string()));
            }

            // Read data flags (first 2 bytes of payload)
            let data_flags_value = u16::from_be_bytes([payload[0], payload[1]]);

            // Check for end of response - Python checks both flags and message type
            let has_end_flag = (data_flags_value & data_flags::END_OF_RESPONSE) != 0;
            let has_eof_flag = (data_flags_value & data_flags::EOF) != 0;
            let no_more_data = (data_flags_value & data_flags::MORE_DATA_TO_FOLLOW) == 0;

            // Also check for EndOfResponse message type (header + 3 bytes with msg type 29)
            let has_end_message = payload.len() == 3
                && payload[2] == MessageType::EndOfResponse as u8;

            // Accumulate payload first
            if is_first_packet {
                // First packet: include data flags in accumulated payload
                accumulated_payload.extend_from_slice(payload);
                is_first_packet = false;
            } else {
                // Subsequent packets: skip the data flags, append only the message data
                accumulated_payload.extend_from_slice(&payload[2..]);
            }

            // Check for end of response using data flags from this packet
            let is_end_of_response = has_end_flag || has_eof_flag || has_end_message || no_more_data;

            // If data flags don't indicate end, scan the ACCUMULATED message data
            // for terminal messages. We scan accumulated data (not just current packet)
            // because messages can span packet boundaries.
            let has_terminal_message = if !is_end_of_response && accumulated_payload.len() > 2 {
                self.scan_for_terminal_message(&accumulated_payload[2..])
            } else {
                false
            };

            // Check if this is the last packet
            if is_end_of_response || has_terminal_message {
                break;
            }
        }

        // Build a synthetic packet with combined payload
        let total_len = PACKET_HEADER_SIZE + accumulated_payload.len();
        let mut result = Vec::with_capacity(total_len);

        // Build header
        if self.large_sdu {
            result.extend_from_slice(&(total_len as u32).to_be_bytes());
        } else {
            result.extend_from_slice(&(total_len as u16).to_be_bytes());
            result.extend_from_slice(&[0, 0]); // Checksum
        }
        result.push(PacketType::Data as u8);
        result.push(0); // Flags
        result.extend_from_slice(&[0, 0]); // Header checksum

        // Add combined payload
        result.extend_from_slice(&accumulated_payload);

        Ok(bytes::Bytes::from(result))
    }

    /// Scan message data for terminal message types (ERROR or END_OF_RESPONSE)
    /// that indicate the response is complete.
    fn scan_for_terminal_message(&self, data: &[u8]) -> bool {
        use crate::buffer::ReadBuffer;
        use crate::constants::MessageType;

        if data.is_empty() {
            return false;
        }

        // Try to parse the message stream and look for ERROR or END_OF_RESPONSE
        let mut buf = ReadBuffer::from_slice(data);

        while buf.remaining() > 0 {
            let msg_type = match buf.read_u8() {
                Ok(t) => t,
                Err(_) => return false, // Can't read, assume incomplete
            };

            // END_OF_RESPONSE is a standalone message with no additional data
            if msg_type == MessageType::EndOfResponse as u8 {
                return true;
            }

            // ERROR message indicates end of response for older Oracle
            if msg_type == MessageType::Error as u8 {
                return true;
            }

            // STATUS message also indicates end of response
            if msg_type == MessageType::Status as u8 {
                return true;
            }

            // LOB_DATA message - skip the data
            if msg_type == MessageType::LobData as u8 {
                match buf.read_raw_bytes_chunked() {
                    Ok(_) => continue,
                    Err(_) => return false, // Incomplete LOB data, need more packets
                }
            }

            // PARAMETER message (8)
            if msg_type == MessageType::Parameter as u8 {
                let remaining = buf.remaining_bytes();
                if remaining.contains(&(MessageType::Error as u8))
                    || remaining.contains(&(MessageType::EndOfResponse as u8))
                    || remaining.contains(&(MessageType::Status as u8))
                {
                    return true;
                }
                return false;
            }

            return false;
        }

        false
    }

    /// Send a marker packet with the specified marker type
    pub(crate) async fn send_marker(&mut self, marker_type: u8) -> Result<()> {
        let mut buf = WriteBuffer::with_capacity(16);

        let payload_len = 3; // 0x01, 0x00, marker_type
        let total_len = (PACKET_HEADER_SIZE + payload_len) as u16;

        // Header
        buf.write_u16_be(total_len)?;
        buf.write_u16_be(0)?; // zeros in large_sdu position
        buf.write_u8(PacketType::Marker as u8)?;
        buf.write_u8(0)?; // flags
        buf.write_u16_be(0)?; // reserved

        // Payload
        buf.write_u8(0x01)?; // constant
        buf.write_u8(0x00)?; // constant
        buf.write_u8(marker_type)?;

        self.send(buf.as_slice()).await
    }

    /// Handle the reset protocol after receiving a MARKER packet
    pub(crate) async fn handle_marker_reset(&mut self) -> Result<bytes::Bytes> {
        const MARKER_TYPE_RESET: u8 = 2;

        // Send reset marker
        self.send_marker(MARKER_TYPE_RESET).await?;

        // Read packets until we get a reset marker back
        loop {
            let packet = self.receive().await?;
            if packet.len() < PACKET_HEADER_SIZE {
                return Err(Error::Protocol("Invalid packet received".to_string()));
            }

            let packet_type = packet[4];

            if packet_type == PacketType::Marker as u8 {
                if packet.len() >= PACKET_HEADER_SIZE + 3 {
                    let marker_type = packet[PACKET_HEADER_SIZE + 2];
                    if marker_type == MARKER_TYPE_RESET {
                        break;
                    }
                }
            } else {
                return Ok(packet);
            }
        }

        loop {
            match self.receive().await {
                Ok(packet) => {
                    let packet_type = packet[4];

                    if packet_type != PacketType::Marker as u8 {
                        return Ok(packet);
                    }
                }
                Err(_) => {
                    return Err(Error::ConnectionClosedByServer(
                        "Query failed - Oracle closed the connection without providing error details. \
                         This typically indicates insufficient privileges or the object doesn't exist.".to_string()
                    ));
                }
            }
        }
    }
}
