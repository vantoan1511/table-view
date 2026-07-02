use futures_util::{SinkExt, StreamExt, stream::{SplitSink, SplitStream}};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Auth info received from Neutralino via stdin or CLI flags.
#[derive(Debug, Default, Deserialize)]
pub struct AuthInfo {
    #[serde(rename = "nlPort", default)]
    pub nl_port: String,
    #[serde(rename = "nlToken", default)]
    pub nl_token: String,
    #[serde(rename = "nlConnectToken", default)]
    pub nl_connect_token: String,
    #[serde(rename = "nlExtensionId", default)]
    pub nl_ext_id: String,
}

impl AuthInfo {
    pub fn from_stdin() -> Self {
        use std::io::Read;
        let mut buf = String::new();
        let stdin = std::io::stdin();
        let mut handle = stdin.lock();
        let mut small_buf = [0u8; 4096];
        if let Ok(n) = handle.read(&mut small_buf) {
            buf.push_str(&String::from_utf8_lossy(&small_buf[..n]));
        }
        serde_json::from_str(&buf).unwrap_or_default()
    }

    pub fn override_from_cli(&mut self) {
        let args: Vec<String> = std::env::args().collect();
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--nl-port" if i + 1 < args.len() => { self.nl_port = args[i + 1].clone(); i += 2; }
                "--nl-token" if i + 1 < args.len() => { self.nl_token = args[i + 1].clone(); i += 2; }
                "--nl-extension-id" if i + 1 < args.len() => { self.nl_ext_id = args[i + 1].clone(); i += 2; }
                "--nl-connect-token" if i + 1 < args.len() => { self.nl_connect_token = args[i + 1].clone(); i += 2; }
                _ => {
                    if let Some(val) = args[i].strip_prefix("--nl-port=") { self.nl_port = val.to_string(); }
                    else if let Some(val) = args[i].strip_prefix("--nl-token=") { self.nl_token = val.to_string(); }
                    else if let Some(val) = args[i].strip_prefix("--nl-extension-id=") { self.nl_ext_id = val.to_string(); }
                    else if let Some(val) = args[i].strip_prefix("--nl-connect-token=") { self.nl_connect_token = val.to_string(); }
                    i += 1;
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(rename = "accessToken", skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

pub type WsWriter = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
pub type WsReader = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

/// The Bridge wraps the WebSocket connection to Neutralino.
pub struct Bridge {
    pub writer: Arc<Mutex<WsWriter>>,
    pub reader: WsReader,
    pub token: String,
}

impl Bridge {
    pub async fn connect(auth: &AuthInfo) -> Result<Self, Box<dyn std::error::Error>> {
        let url = format!(
            "ws://localhost:{}?extensionId={}&connectToken={}",
            auth.nl_port,
            urlencoding::encode(&auth.nl_ext_id),
            urlencoding::encode(&auth.nl_connect_token)
        );

        log::info!("Connecting to WebSocket: {}", url);
        let (ws, _) = connect_async(&url).await?;
        let (writer, reader) = ws.split();

        Ok(Bridge {
            writer: Arc::new(Mutex::new(writer)),
            reader,
            token: auth.nl_token.clone(),
        })
    }

    pub async fn broadcast(
        writer: &Arc<Mutex<WsWriter>>,
        token: &str,
        event: &str,
        data: Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let payload = serde_json::json!({
            "event": event,
            "data": data,
        });

        let msg = WsMessage {
            id: Some(uuid::Uuid::new_v4().to_string()),
            method: Some("app.broadcast".to_string()),
            access_token: Some(token.to_string()),
            event: None,
            data: Some(payload),
        };

        let text = serde_json::to_string(&msg)?;
        log::info!("Broadcasting event: {}", event);
        let mut w = writer.lock().await;
        w.send(Message::Text(text)).await?;
        Ok(())
    }

    pub async fn read_message(reader: &mut WsReader, writer: &Arc<Mutex<WsWriter>>) -> Result<WsMessage, Box<dyn std::error::Error>> {
        loop {
            match reader.next().await {
                Some(Ok(Message::Text(text))) => {
                    let msg: WsMessage = serde_json::from_str(&text)?;
                    return Ok(msg);
                }
                Some(Ok(Message::Ping(data))) => {
                    let mut w = writer.lock().await;
                    w.send(Message::Pong(data)).await?;
                }
                Some(Ok(_)) => continue,
                Some(Err(e)) => return Err(Box::new(e)),
                None => return Err("WebSocket closed".into()),
            }
        }
    }
}
