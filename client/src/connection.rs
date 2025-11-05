use anyhow::{Context, Result};
use futures::{SinkExt, StreamExt};
use terma_shared::{ClientMessage, ServerMessage};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub struct Connection {
    tx: mpsc::UnboundedSender<ClientMessage>,
}

impl Connection {
    pub async fn connect(
        host: &str,
        room_id: &str,
        user_id: String,
    ) -> Result<(Self, mpsc::UnboundedReceiver<ServerMessage>)> {
        let url = if host.starts_with("localhost") || host.starts_with("127.0.0.1") {
            format!("ws://{}/ws/{}", host, room_id)
        } else {
            format!("wss://{}/ws/{}", host, room_id)
        };

        let (ws_stream, _) = connect_async(&url)
            .await
            .context("Failed to connect to server")?;

        let (mut write, mut read) = ws_stream.split();

        // Send Join message immediately
        let join_msg = ClientMessage::Join { user_id };
        let join_json = join_msg.to_json()?;
        write.send(Message::Text(join_json)).await?;

        // Create channels
        let (outgoing_tx, mut outgoing_rx) = mpsc::unbounded_channel::<ClientMessage>();
        let (incoming_tx, incoming_rx) = mpsc::unbounded_channel::<ServerMessage>();

        // Spawn task to send messages
        tokio::spawn(async move {
            while let Some(msg) = outgoing_rx.recv().await {
                if let Ok(json) = msg.to_json() {
                    if write.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
            }
        });

        // Spawn task to receive messages
        tokio::spawn(async move {
            while let Some(Ok(msg)) = read.next().await {
                if let Message::Text(text) = msg {
                    if let Ok(server_msg) = ServerMessage::from_json(&text) {
                        if incoming_tx.send(server_msg).is_err() {
                            break;
                        }
                    }
                }
            }
        });

        Ok((Connection { tx: outgoing_tx }, incoming_rx))
    }

    pub fn send(&self, message: ClientMessage) -> Result<()> {
        self.tx
            .send(message)
            .context("Failed to send message")?;
        Ok(())
    }
}
