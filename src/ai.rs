use std::time::Duration;

/// Trait for AI services that can generate responses
#[async_trait::async_trait]
pub trait AIService: Send + Sync {
    /// Send a message and get a response (async)
    async fn send_message(&self, message: &str) -> String;

    /// Stream a response with thinking time (async)
    /// Returns (thinking_duration, response)
    async fn send_message_with_thinking(&self, message: &str) -> (Duration, String);
}

/// Mock AI service for testing
pub struct MockAI {
    pub delay_ms: u64,
}

impl MockAI {
    pub fn new() -> Self {
        Self { delay_ms: 2000 }
    }

    pub fn with_delay(delay_ms: u64) -> Self {
        Self { delay_ms }
    }
}

#[async_trait::async_trait]
impl AIService for MockAI {
    async fn send_message(&self, message: &str) -> String {
        // Simulate processing delay
        tokio::time::sleep(Duration::from_millis(self.delay_ms)).await;

        // Generate a mock response
        format!(
            "I received your message: \"{}\". This is a mock AI response!",
            message
        )
    }

    async fn send_message_with_thinking(&self, message: &str) -> (Duration, String) {
        let start = std::time::Instant::now();

        // Simulate thinking time
        tokio::time::sleep(Duration::from_millis(self.delay_ms)).await;

        let thinking_duration = start.elapsed();
        let response = format!(
            "After careful consideration of \"{}\", here's my thoughtful response!",
            message
        );

        (thinking_duration, response)
    }
}
