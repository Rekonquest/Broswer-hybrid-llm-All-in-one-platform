use common::{messages::OrchestratorMessage, errors::Result};
use tokio::sync::{mpsc, broadcast};
use tracing::{debug, error};
use uuid::Uuid;

/// Message bus for inter-component communication
/// Uses a broadcast channel for pub-sub pattern
pub struct MessageBus {
    /// Broadcast sender for all messages
    sender: broadcast::Sender<OrchestratorMessage>,
    /// Capacity of the broadcast channel
    capacity: usize,
}

impl MessageBus {
    /// Create a new message bus with specified capacity
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender, capacity }
    }

    /// Publish a message to all subscribers
    pub fn publish(&self, message: OrchestratorMessage) -> Result<()> {
        debug!("📤 Publishing message: {:?}", message);

        self.sender
            .send(message)
            .map_err(|e| common::errors::HybridLLMError::Other(anyhow::anyhow!("Failed to send message: {}", e)))?;

        Ok(())
    }

    /// Subscribe to messages
    /// Returns a receiver that will get all future messages
    pub fn subscribe(&self) -> broadcast::Receiver<OrchestratorMessage> {
        self.sender.subscribe()
    }

    /// Get the number of active subscribers
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::messages::OrchestratorMessage;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_message_bus() {
        let bus = MessageBus::new(100);
        let mut rx = bus.subscribe();

        let msg = OrchestratorMessage::UserRequest {
            id: Uuid::new_v4(),
            content: "Test".to_string(),
            context: HashMap::new(),
        };

        bus.publish(msg.clone()).unwrap();

        let received = rx.recv().await.unwrap();
        match received {
            OrchestratorMessage::UserRequest { content, .. } => {
                assert_eq!(content, "Test");
            }
            _ => panic!("Wrong message type"),
        }
    }
}
