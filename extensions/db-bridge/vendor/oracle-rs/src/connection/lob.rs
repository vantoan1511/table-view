//! LOB (Large Object) and BFILE operations on Connection.

use crate::constants::{MessageType, OracleType, PacketType, PACKET_HEADER_SIZE};
use crate::error::{Error, Result};
use crate::messages::LobOpMessage;
use crate::types::{LobData, LobLocator};

use super::Connection;

impl Connection {
    /// Read the entire contents of a LOB
    pub async fn read_lob(&self, locator: &LobLocator) -> Result<LobData> {
        self.ensure_ready().await?;

        let offset = 1u64;
        let amount = locator.size();

        self.read_lob_internal(locator, offset, amount).await
    }

    /// Read a portion of a LOB
    pub async fn read_lob_range(
        &self,
        locator: &LobLocator,
        offset: u64,
        amount: u64,
    ) -> Result<LobData> {
        self.ensure_ready().await?;
        self.read_lob_internal(locator, offset, amount).await
    }

    /// Read a CLOB and return as String
    pub async fn read_clob(&self, locator: &LobLocator) -> Result<String> {
        if locator.is_blob() || locator.is_bfile() {
            return Err(Error::Protocol(
                "Cannot read BLOB/BFILE as string, use read_blob instead".to_string(),
            ));
        }

        let data = self.read_lob(locator).await?;
        match data {
            LobData::String(s) => Ok(s),
            LobData::Bytes(_) => Err(Error::Protocol(
                "Unexpected bytes from CLOB read".to_string(),
            )),
        }
    }

    /// Read a BLOB and return as bytes
    pub async fn read_blob(&self, locator: &LobLocator) -> Result<bytes::Bytes> {
        if locator.is_clob() {
            return Err(Error::Protocol(
                "Cannot read CLOB as bytes, use read_clob instead".to_string(),
            ));
        }

        let data = self.read_lob(locator).await?;
        match data {
            LobData::Bytes(b) => Ok(b),
            LobData::String(_) => Err(Error::Protocol(
                "Unexpected string from BLOB read".to_string(),
            )),
        }
    }

    /// Read a LOB in chunks, calling a callback for each chunk
    pub async fn read_lob_chunked<F, Fut>(
        &self,
        locator: &LobLocator,
        chunk_size: u64,
        mut callback: F,
    ) -> Result<()>
    where
        F: FnMut(LobData) -> Fut,
        Fut: std::future::Future<Output = Result<()>>,
    {
        self.ensure_ready().await?;

        let total_size = locator.size();
        if total_size == 0 {
            return Ok(());
        }

        let chunk_size = if chunk_size == 0 {
            self.lob_chunk_size(locator).await?.max(8192) as u64
        } else {
            chunk_size
        };

        let mut offset = 1u64;
        while offset <= total_size {
            let remaining = total_size - offset + 1;
            let amount = std::cmp::min(remaining, chunk_size);

            let chunk = self.read_lob_internal(locator, offset, amount).await?;
            callback(chunk).await?;

            offset += amount;
        }

        Ok(())
    }

    /// Get the optimal chunk size for a LOB
    pub async fn lob_chunk_size(&self, locator: &LobLocator) -> Result<u32> {
        self.ensure_ready().await?;

        let mut inner = self.inner.lock().await;
        let large_sdu = inner.large_sdu;

        let mut lob_msg = LobOpMessage::new_get_chunk_size(locator);
        let seq_num = inner.next_sequence_number();
        lob_msg.set_sequence_number(seq_num);

        let request = lob_msg.build_request(&inner.capabilities, large_sdu)?;
        inner.send(&request).await?;

        let response = inner.receive().await?;
        if response.len() <= PACKET_HEADER_SIZE {
            return Err(Error::Protocol("Empty LOB chunk size response".to_string()));
        }

        let packet_type = response[4];
        if packet_type == PacketType::Marker as u8 {
            let error_response = inner.handle_marker_reset().await?;
            let payload = &error_response[PACKET_HEADER_SIZE..];
            let mut buf = crate::buffer::ReadBuffer::from_slice(payload);
            buf.skip(2)?;
            return self.parse_lob_error(&mut buf);
        }

        self.parse_lob_amount_response(&response[PACKET_HEADER_SIZE..], locator)
            .map(|v| v as u32)
    }

    /// Internal LOB read implementation
    pub(crate) async fn read_lob_internal(
        &self,
        locator: &LobLocator,
        offset: u64,
        amount: u64,
    ) -> Result<LobData> {
        let mut inner = self.inner.lock().await;
        let large_sdu = inner.large_sdu;

        let mut lob_msg = LobOpMessage::new_read(locator, offset, amount);
        let seq_num = inner.next_sequence_number();
        lob_msg.set_sequence_number(seq_num);

        let request = lob_msg.build_request(&inner.capabilities, large_sdu)?;
        inner.send(&request).await?;

        let response = inner.receive_response().await?;
        if response.len() <= PACKET_HEADER_SIZE {
            return Err(Error::Protocol("Empty LOB read response".to_string()));
        }

        let packet_type = response[4];
        if packet_type == PacketType::Marker as u8 {
            let error_response = inner.handle_marker_reset().await?;
            let payload = &error_response[PACKET_HEADER_SIZE..];
            let mut buf = crate::buffer::ReadBuffer::from_slice(payload);
            buf.skip(2)?;
            return self.parse_lob_error(&mut buf);
        }

        let payload = &response[PACKET_HEADER_SIZE..];
        self.parse_lob_read_response(payload, locator)
    }

    /// Parse LOB read response
    pub(crate) fn parse_lob_read_response(&self, payload: &[u8], locator: &LobLocator) -> Result<LobData> {
        use crate::buffer::ReadBuffer;

        let mut buf = ReadBuffer::from_slice(payload);

        buf.skip(2)?;

        let mut lob_data: Option<Vec<u8>> = None;

        while buf.remaining() > 0 {
            let msg_type = buf.read_u8()?;

            match msg_type {
                x if x == MessageType::LobData as u8 => {
                    let data = buf.read_raw_bytes_chunked()?;
                    lob_data = Some(data);
                }

                x if x == MessageType::Parameter as u8 => {
                    let locator_len = locator.locator_bytes().len();
                    buf.skip(locator_len)?;
                    let _returned_amount = buf.read_ub8()?;
                }

                x if x == MessageType::Error as u8 => {
                    if let Ok((code, msg, _)) = self.parse_error_info(&mut buf) {
                        if code != 0 {
                            let message = msg.unwrap_or_else(|| "LOB error".to_string());
                            return Err(Error::OracleError { code, message });
                        }
                    }
                }

                x if x == MessageType::EndOfResponse as u8 => {
                    break;
                }

                _ => continue,
            }
        }

        match lob_data {
            Some(data) => {
                if locator.is_blob() || locator.is_bfile() {
                    Ok(LobData::Bytes(bytes::Bytes::from(data)))
                } else {
                    let text = if locator.uses_var_length_charset() {
                        let chars: Vec<u16> = data
                            .chunks_exact(2)
                            .map(|c| u16::from_be_bytes([c[0], c[1]]))
                            .collect();
                        String::from_utf16_lossy(&chars)
                    } else {
                        String::from_utf8_lossy(&data).to_string()
                    };
                    Ok(LobData::String(text))
                }
            }
            None => {
                if locator.is_blob() || locator.is_bfile() {
                    Ok(LobData::Bytes(bytes::Bytes::new()))
                } else {
                    Ok(LobData::String(String::new()))
                }
            }
        }
    }

    /// Write data to a LOB
    pub async fn write_lob(&self, locator: &LobLocator, offset: u64, data: &[u8]) -> Result<()> {
        self.ensure_ready().await?;

        let mut inner = self.inner.lock().await;
        let large_sdu = inner.large_sdu;
        let sdu_size = inner.sdu_size as usize;

        let encoded_data: Vec<u8>;
        let write_data = if locator.is_clob() && locator.uses_var_length_charset() {
            let text = String::from_utf8_lossy(data);
            encoded_data = text
                .encode_utf16()
                .flat_map(|c| c.to_be_bytes())
                .collect();
            &encoded_data[..]
        } else {
            data
        };

        let mut lob_msg = LobOpMessage::new_write(locator, offset, write_data);
        let seq_num = inner.next_sequence_number();
        lob_msg.set_sequence_number(seq_num);

        let message = lob_msg.build_message_only(&inner.capabilities)?;

        let max_single_packet_payload = sdu_size.saturating_sub(PACKET_HEADER_SIZE + 2);

        let is_multi_packet = message.len() > max_single_packet_payload;

        if is_multi_packet {
            inner.send_multi_packet(&message, 0).await?;
        } else {
            let request = lob_msg.build_request(&inner.capabilities, large_sdu)?;
            inner.send(&request).await?;
        }

        let response = inner.receive_response().await?;
        if response.len() <= PACKET_HEADER_SIZE {
            return Err(Error::Protocol("Empty LOB write response".to_string()));
        }

        let packet_type = response[4];
        if packet_type == PacketType::Marker as u8 {
            let error_response = inner.handle_marker_reset().await?;
            let payload = &error_response[PACKET_HEADER_SIZE..];
            let mut buf = crate::buffer::ReadBuffer::from_slice(payload);
            buf.skip(2)?;
            return self.parse_lob_error(&mut buf);
        }

        self.parse_lob_simple_response(&response[PACKET_HEADER_SIZE..], locator)
    }

    /// Write string data to a CLOB
    pub async fn write_clob(&self, locator: &LobLocator, offset: u64, text: &str) -> Result<()> {
        if locator.is_blob() || locator.is_bfile() {
            return Err(Error::Protocol(
                "Cannot write string to BLOB/BFILE, use write_blob instead".to_string(),
            ));
        }
        self.write_lob(locator, offset, text.as_bytes()).await
    }

    /// Write binary data to a BLOB
    pub async fn write_blob(&self, locator: &LobLocator, offset: u64, data: &[u8]) -> Result<()> {
        if locator.is_clob() {
            return Err(Error::Protocol(
                "Cannot write bytes to CLOB, use write_clob instead".to_string(),
            ));
        }
        self.write_lob(locator, offset, data).await
    }

    /// Get the length of a LOB
    pub async fn lob_length(&self, locator: &LobLocator) -> Result<u64> {
        self.ensure_ready().await?;

        let mut inner = self.inner.lock().await;
        let large_sdu = inner.large_sdu;

        let mut lob_msg = LobOpMessage::new_get_length(locator);
        let seq_num = inner.next_sequence_number();
        lob_msg.set_sequence_number(seq_num);

        let request = lob_msg.build_request(&inner.capabilities, large_sdu)?;
        inner.send(&request).await?;

        let response = inner.receive().await?;
        if response.len() <= PACKET_HEADER_SIZE {
            return Err(Error::Protocol("Empty LOB get_length response".to_string()));
        }

        let packet_type = response[4];
        if packet_type == PacketType::Marker as u8 {
            let error_response = inner.handle_marker_reset().await?;
            let payload = &error_response[PACKET_HEADER_SIZE..];
            let mut buf = crate::buffer::ReadBuffer::from_slice(payload);
            buf.skip(2)?;
            return self.parse_lob_error(&mut buf);
        }

        self.parse_lob_amount_response(&response[PACKET_HEADER_SIZE..], locator)
    }

    /// Trim a LOB to a specified length
    pub async fn lob_trim(&self, locator: &LobLocator, new_size: u64) -> Result<()> {
        self.ensure_ready().await?;

        let mut inner = self.inner.lock().await;
        let large_sdu = inner.large_sdu;

        let mut lob_msg = LobOpMessage::new_trim(locator, new_size);
        let seq_num = inner.next_sequence_number();
        lob_msg.set_sequence_number(seq_num);

        let request = lob_msg.build_request(&inner.capabilities, large_sdu)?;
        inner.send(&request).await?;

        let response = inner.receive().await?;
        if response.len() <= PACKET_HEADER_SIZE {
            return Err(Error::Protocol("Empty LOB trim response".to_string()));
        }

        let packet_type = response[4];
        if packet_type == PacketType::Marker as u8 {
            let error_response = inner.handle_marker_reset().await?;
            let payload = &error_response[PACKET_HEADER_SIZE..];
            let mut buf = crate::buffer::ReadBuffer::from_slice(payload);
            buf.skip(2)?;
            return self.parse_lob_error(&mut buf);
        }

        self.parse_lob_simple_response(&response[PACKET_HEADER_SIZE..], locator)
    }

    /// Create a temporary LOB on the server
    pub async fn create_temp_lob(&self, oracle_type: OracleType) -> Result<LobLocator> {
        use crate::buffer::ReadBuffer;

        match oracle_type {
            OracleType::Clob | OracleType::Blob => {}
            _ => {
                return Err(Error::Protocol(format!(
                    "create_temp_lob: invalid type {:?}, must be Clob or Blob",
                    oracle_type
                )));
            }
        }

        self.ensure_ready().await?;

        let mut inner = self.inner.lock().await;
        let large_sdu = inner.large_sdu;

        let mut lob_msg = LobOpMessage::new_create_temp(oracle_type);
        let seq_num = inner.next_sequence_number();
        lob_msg.set_sequence_number(seq_num);

        let request = lob_msg.build_request(&inner.capabilities, large_sdu)?;
        inner.send(&request).await?;

        let response = inner.receive().await?;
        if response.len() <= PACKET_HEADER_SIZE {
            return Err(Error::Protocol("Empty CREATE_TEMP LOB response".to_string()));
        }

        let packet_type = response[4];
        if packet_type == PacketType::Marker as u8 {
            let error_response = inner.handle_marker_reset().await?;
            let payload = &error_response[PACKET_HEADER_SIZE..];
            let mut buf = ReadBuffer::from_slice(payload);
            buf.skip(2)?;
            return self.parse_lob_error(&mut buf);
        }

        let payload = &response[PACKET_HEADER_SIZE..];
        let mut buf = ReadBuffer::from_slice(payload);
        buf.skip(2)?;

        let mut locator_bytes: Option<Vec<u8>> = None;

        while buf.remaining() > 0 {
            let msg_type = buf.read_u8()?;

            match msg_type {
                x if x == MessageType::Parameter as u8 => {
                    let loc_data = buf.read_bytes_vec(40)?;
                    locator_bytes = Some(loc_data);
                    buf.skip_ub2()?;
                    buf.skip(1)?;
                }

                x if x == MessageType::Error as u8 => {
                    if let Ok((code, msg, _)) = self.parse_error_info(&mut buf) {
                        if code != 0 {
                            let message = msg.unwrap_or_else(|| "CREATE_TEMP LOB error".to_string());
                            return Err(Error::OracleError { code, message });
                        }
                    }
                }

                x if x == MessageType::EndOfResponse as u8 => {
                    break;
                }

                _ => continue,
            }
        }

        let loc_bytes = locator_bytes.ok_or_else(|| {
            Error::Protocol("CREATE_TEMP LOB response did not contain locator".to_string())
        })?;

        let locator = LobLocator::new(
            bytes::Bytes::from(loc_bytes),
            0,
            0,
            oracle_type,
            1,
        );

        Ok(locator)
    }

    // ==================== BFILE Operations ====================

    /// Check if a BFILE exists on the server
    pub async fn bfile_exists(&self, locator: &LobLocator) -> Result<bool> {
        self.ensure_ready().await?;

        if !locator.is_bfile() {
            return Err(Error::Protocol("bfile_exists called on non-BFILE locator".to_string()));
        }

        let mut inner = self.inner.lock().await;
        let large_sdu = inner.large_sdu;

        let mut lob_msg = LobOpMessage::new_file_exists(locator);
        let seq_num = inner.next_sequence_number();
        lob_msg.set_sequence_number(seq_num);

        let request = lob_msg.build_request(&inner.capabilities, large_sdu)?;
        inner.send(&request).await?;

        let response = inner.receive().await?;
        if response.len() <= PACKET_HEADER_SIZE {
            return Err(Error::Protocol("Empty BFILE exists response".to_string()));
        }

        let packet_type = response[4];
        if packet_type == PacketType::Marker as u8 {
            let error_response = inner.handle_marker_reset().await?;
            let payload = &error_response[PACKET_HEADER_SIZE..];
            let mut buf = crate::buffer::ReadBuffer::from_slice(payload);
            buf.skip(2)?;
            return self.parse_lob_error(&mut buf);
        }

        self.parse_lob_bool_response(&response[PACKET_HEADER_SIZE..], locator)
    }

    /// Open a BFILE for reading
    pub async fn bfile_open(&self, locator: &LobLocator) -> Result<()> {
        self.ensure_ready().await?;

        if !locator.is_bfile() {
            return Err(Error::Protocol("bfile_open called on non-BFILE locator".to_string()));
        }

        let mut inner = self.inner.lock().await;
        let large_sdu = inner.large_sdu;

        let mut lob_msg = LobOpMessage::new_file_open(locator);
        let seq_num = inner.next_sequence_number();
        lob_msg.set_sequence_number(seq_num);

        let request = lob_msg.build_request(&inner.capabilities, large_sdu)?;
        inner.send(&request).await?;

        let response = inner.receive().await?;
        if response.len() <= PACKET_HEADER_SIZE {
            return Err(Error::Protocol("Empty BFILE open response".to_string()));
        }

        let packet_type = response[4];
        if packet_type == PacketType::Marker as u8 {
            let error_response = inner.handle_marker_reset().await?;
            let payload = &error_response[PACKET_HEADER_SIZE..];
            let mut buf = crate::buffer::ReadBuffer::from_slice(payload);
            buf.skip(2)?;
            return self.parse_lob_error(&mut buf);
        }

        self.parse_lob_simple_response(&response[PACKET_HEADER_SIZE..], locator)
    }

    /// Close a BFILE after reading
    pub async fn bfile_close(&self, locator: &LobLocator) -> Result<()> {
        self.ensure_ready().await?;

        if !locator.is_bfile() {
            return Err(Error::Protocol("bfile_close called on non-BFILE locator".to_string()));
        }

        let mut inner = self.inner.lock().await;
        let large_sdu = inner.large_sdu;

        let mut lob_msg = LobOpMessage::new_file_close(locator);
        let seq_num = inner.next_sequence_number();
        lob_msg.set_sequence_number(seq_num);

        let request = lob_msg.build_request(&inner.capabilities, large_sdu)?;
        inner.send(&request).await?;

        let response = inner.receive().await?;
        if response.len() <= PACKET_HEADER_SIZE {
            return Err(Error::Protocol("Empty BFILE close response".to_string()));
        }

        let packet_type = response[4];
        if packet_type == PacketType::Marker as u8 {
            let error_response = inner.handle_marker_reset().await?;
            let payload = &error_response[PACKET_HEADER_SIZE..];
            let mut buf = crate::buffer::ReadBuffer::from_slice(payload);
            buf.skip(2)?;
            return self.parse_lob_error(&mut buf);
        }

        self.parse_lob_simple_response(&response[PACKET_HEADER_SIZE..], locator)
    }

    /// Check if a BFILE is currently open
    pub async fn bfile_is_open(&self, locator: &LobLocator) -> Result<bool> {
        self.ensure_ready().await?;

        if !locator.is_bfile() {
            return Err(Error::Protocol("bfile_is_open called on non-BFILE locator".to_string()));
        }

        let mut inner = self.inner.lock().await;
        let large_sdu = inner.large_sdu;

        let mut lob_msg = LobOpMessage::new_file_is_open(locator);
        let seq_num = inner.next_sequence_number();
        lob_msg.set_sequence_number(seq_num);

        let request = lob_msg.build_request(&inner.capabilities, large_sdu)?;
        inner.send(&request).await?;

        let response = inner.receive().await?;
        if response.len() <= PACKET_HEADER_SIZE {
            return Err(Error::Protocol("Empty BFILE is_open response".to_string()));
        }

        let packet_type = response[4];
        if packet_type == PacketType::Marker as u8 {
            let error_response = inner.handle_marker_reset().await?;
            let payload = &error_response[PACKET_HEADER_SIZE..];
            let mut buf = crate::buffer::ReadBuffer::from_slice(payload);
            buf.skip(2)?;
            return self.parse_lob_error(&mut buf);
        }

        self.parse_lob_bool_response(&response[PACKET_HEADER_SIZE..], locator)
    }

    /// Read BFILE data
    pub async fn read_bfile(&self, locator: &LobLocator) -> Result<bytes::Bytes> {
        if !locator.is_bfile() {
            return Err(Error::Protocol("read_bfile called on non-BFILE locator".to_string()));
        }

        let should_close = if !self.bfile_is_open(locator).await? {
            self.bfile_open(locator).await?;
            true
        } else {
            false
        };

        let result = self.read_blob(locator).await;

        if should_close {
            let _ = self.bfile_close(locator).await;
        }

        result
    }

    /// Parse a LOB operation response that returns a boolean
    pub(crate) fn parse_lob_bool_response(&self, payload: &[u8], locator: &LobLocator) -> Result<bool> {
        use crate::buffer::ReadBuffer;

        let mut buf = ReadBuffer::from_slice(payload);
        buf.skip(2)?;

        let mut bool_result: bool = false;

        while buf.remaining() > 0 {
            let msg_type = buf.read_u8()?;

            match msg_type {
                x if x == MessageType::Parameter as u8 => {
                    let locator_len = locator.locator_bytes().len();
                    buf.skip(locator_len)?;
                    let flag = buf.read_u8()?;
                    bool_result = flag > 0;
                }

                x if x == MessageType::Error as u8 => {
                    if let Ok((code, msg, _)) = self.parse_error_info(&mut buf) {
                        if code != 0 {
                            let message = msg.unwrap_or_else(|| "LOB error".to_string());
                            return Err(Error::OracleError { code, message });
                        }
                    }
                }

                x if x == MessageType::EndOfResponse as u8 => {
                    break;
                }

                _ => continue,
            }
        }

        Ok(bool_result)
    }

    /// Parse a simple LOB operation response (write, trim)
    pub(crate) fn parse_lob_simple_response(&self, payload: &[u8], locator: &LobLocator) -> Result<()> {
        use crate::buffer::ReadBuffer;

        let mut buf = ReadBuffer::from_slice(payload);
        buf.skip(2)?;

        while buf.remaining() > 0 {
            let msg_type = buf.read_u8()?;

            match msg_type {
                x if x == MessageType::Parameter as u8 => {
                    let locator_len = locator.locator_bytes().len();
                    buf.skip(locator_len)?;
                }

                x if x == MessageType::Error as u8 => {
                    if let Ok((code, msg, _)) = self.parse_error_info(&mut buf) {
                        if code != 0 {
                            let message = msg.unwrap_or_else(|| "LOB error".to_string());
                            return Err(Error::OracleError { code, message });
                        }
                    }
                }

                x if x == MessageType::EndOfResponse as u8 => {
                    break;
                }

                _ => continue,
            }
        }

        Ok(())
    }

    /// Parse a LOB operation response that returns an amount
    pub(crate) fn parse_lob_amount_response(&self, payload: &[u8], locator: &LobLocator) -> Result<u64> {
        use crate::buffer::ReadBuffer;

        let mut buf = ReadBuffer::from_slice(payload);
        buf.skip(2)?;

        let mut returned_amount: u64 = 0;

        while buf.remaining() > 0 {
            let msg_type = buf.read_u8()?;

            match msg_type {
                x if x == MessageType::Parameter as u8 => {
                    let locator_len = locator.locator_bytes().len();
                    buf.skip(locator_len)?;
                    returned_amount = buf.read_ub8()?;
                }

                x if x == MessageType::Error as u8 => {
                    if let Ok((code, msg, _)) = self.parse_error_info(&mut buf) {
                        if code != 0 {
                            let message = msg.unwrap_or_else(|| "LOB error".to_string());
                            return Err(Error::OracleError { code, message });
                        }
                    }
                }

                x if x == MessageType::EndOfResponse as u8 => {
                    break;
                }

                _ => continue,
            }
        }

        Ok(returned_amount)
    }

    /// Parse LOB error response
    pub(crate) fn parse_lob_error<T>(&self, buf: &mut crate::buffer::ReadBuffer) -> Result<T> {
        if let Ok((code, msg, _)) = self.parse_error_info(buf) {
            let message = msg.unwrap_or_else(|| "Unknown LOB error".to_string());
            Err(Error::OracleError { code, message })
        } else {
            Err(Error::Protocol("LOB operation failed".to_string()))
        }
    }
}
