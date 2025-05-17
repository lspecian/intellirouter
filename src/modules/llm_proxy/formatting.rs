//! LLM Proxy Response Formatting
//!
//! This module provides unified formatting functions for LLM responses,
//! ensuring all responses match OpenAI API format specifications exactly.

use chrono::Utc;
use uuid::Uuid;

use super::routes::{
    ChatCompletionChoice, ChatCompletionChunk, ChatCompletionChunkChoice, ChatCompletionRequest,
    ChatCompletionResponse, ChatMessage, ChatMessageDelta, TokenUsage,
};

/// Format a non-streaming response
pub fn format_completion_response(
    model: &str,
    messages: &[ChatMessage],
    content: &str,
    finish_reason: &str,
) -> ChatCompletionResponse {
    ChatCompletionResponse {
        id: generate_response_id(),
        object: "chat.completion".to_string(),
        created: Utc::now().timestamp() as u64,
        model: normalize_model_name(model),
        choices: vec![ChatCompletionChoice {
            index: 0,
            message: ChatMessage {
                role: "assistant".to_string(),
                content: content.to_string(),
                name: None,
            },
            finish_reason: finish_reason.to_string(),
        }],
        usage: calculate_token_usage(messages, content),
    }
}

/// Format a streaming response chunk
pub fn format_completion_chunk(
    model: &str,
    chunk_index: u32,
    content: Option<String>,
    role: Option<String>,
    finish_reason: Option<String>,
) -> ChatCompletionChunk {
    ChatCompletionChunk {
        id: generate_response_id(),
        object: "chat.completion.chunk".to_string(),
        created: Utc::now().timestamp() as u64,
        model: normalize_model_name(model),
        choices: vec![ChatCompletionChunkChoice {
            index: 0,
            delta: ChatMessageDelta { role, content },
            finish_reason,
        }],
    }
}

/// Generate a unique response ID in OpenAI format
pub fn generate_response_id() -> String {
    format!("chatcmpl-{}", Uuid::new_v4().to_string().replace("-", ""))
}

/// Calculate token usage for a request and response
pub fn calculate_token_usage(messages: &[ChatMessage], response_content: &str) -> TokenUsage {
    let prompt_tokens = calculate_prompt_tokens(messages);
    let completion_tokens = calculate_completion_tokens(response_content);

    TokenUsage {
        prompt_tokens,
        completion_tokens,
        total_tokens: prompt_tokens + completion_tokens,
    }
}

/// Calculate approximate token count for the prompt
pub fn calculate_prompt_tokens(messages: &[ChatMessage]) -> u32 {
    // More accurate token counting
    let mut count = 0;
    for message in messages {
        // Roughly 4 chars per token
        count += (message.content.len() / 4) as u32;
        // Add overhead for message format
        count += 4; // For role and message overhead
    }
    // Add overhead for the overall messages structure
    count += 2;
    count
}

/// Calculate approximate token count for the completion
pub fn calculate_completion_tokens(content: &str) -> u32 {
    // More accurate token counting
    // Roughly 4 chars per token
    (content.len() / 4) as u32 + 2 // Add overhead for message format
}

/// Normalize a model name to a standard format
pub fn normalize_model_name(model: &str) -> String {
    // Handle different model name formats
    if model.starts_with("gpt-") || model.starts_with("text-") {
        // OpenAI model names are already normalized
        model.to_string()
    } else if model.starts_with("claude-") {
        // Map Anthropic model names to OpenAI-like format
        format!("anthropic/{}", model)
    } else if model.starts_with("mistral-") {
        // Map Mistral model names to OpenAI-like format
        format!("mistral/{}", model)
    } else if model.starts_with("llama-") {
        // Map Llama model names to OpenAI-like format
        format!("meta/{}", model)
    } else {
        // Default case
        format!("unknown/{}", model)
    }
}

/// Generate a contextual response based on the user's message
pub fn generate_contextual_response(user_message: &str) -> String {
    // Simple keyword-based response generation
    if user_message.to_lowercase().contains("hello") || user_message.to_lowercase().contains("hi") {
        "Hello! I'm a mock assistant from the IntelliRouter LLM proxy. How can I help you today?"
            .to_string()
    } else if user_message.to_lowercase().contains("help") {
        "I'm here to help! However, I'm currently just a mock response from the IntelliRouter LLM proxy. In the future, I'll be able to provide real assistance by routing to actual LLM providers.".to_string()
    } else if user_message.to_lowercase().contains("weather") {
        "I don't have access to real-time weather data as I'm just a mock response. In a real implementation, this request would be routed to an actual LLM that might have access to such information.".to_string()
    } else if user_message.to_lowercase().contains("code")
        || user_message.to_lowercase().contains("program")
    {
        "I can help with programming questions, but as a mock response, I can't provide actual code assistance yet. When implemented with a real LLM, I'll be able to help with coding tasks across various languages.".to_string()
    } else if user_message.to_lowercase().contains("explain")
        || user_message.to_lowercase().contains("what is")
    {
        "I'd be happy to explain that topic. As a mock response, I don't have the actual knowledge to provide a detailed explanation. When connected to a real LLM, I'll be able to provide comprehensive explanations on a wide range of topics.".to_string()
    } else {
        "This is a mock response from the IntelliRouter LLM proxy. Your message has been received, but I'm not yet connected to a real LLM provider. This functionality will be implemented in future tasks.".to_string()
    }
}

/// Apply temperature effects to a response
pub fn apply_temperature_effects(content: &str, temperature: Option<f32>) -> String {
    if let Some(temp) = temperature {
        if temp > 1.0 {
            // Add some randomness for high temperature
            format!(
                "{} (With some creative variations due to high temperature setting.)",
                content
            )
        } else {
            content.to_string()
        }
    } else {
        content.to_string()
    }
}

/// Apply max_tokens truncation to a response
pub fn apply_max_tokens_truncation(content: &str, max_tokens: Option<u32>) -> String {
    if let Some(max_tokens) = max_tokens {
        // Simulate token truncation (very simplified)
        let approx_tokens = content.split_whitespace().count() as u32;
        if approx_tokens > max_tokens {
            let words: Vec<&str> = content.split_whitespace().collect();
            let truncated_words = &words[0..max_tokens as usize];
            truncated_words.join(" ") + "..."
        } else {
            content.to_string()
        }
    } else {
        content.to_string()
    }
}

/// Create a sequence of streaming chunks from a complete response
pub fn create_streaming_chunks(
    model: &str,
    content: &str,
    num_chunks: usize,
) -> Vec<ChatCompletionChunk> {
    let mut chunks = Vec::with_capacity(num_chunks + 1); // +1 for the initial role chunk

    // First chunk with role only
    chunks.push(format_completion_chunk(
        model,
        0,
        None,
        Some("assistant".to_string()),
        None,
    ));

    if num_chunks <= 1 {
        // If only one content chunk requested, send it all at once
        chunks.push(format_completion_chunk(
            model,
            1,
            Some(content.to_string()),
            None,
            Some("stop".to_string()),
        ));
    } else {
        // Split content into multiple chunks
        let words: Vec<&str> = content.split_whitespace().collect();
        let chunk_size = (words.len() as f32 / num_chunks as f32).ceil() as usize;

        for i in 0..num_chunks {
            let start = i * chunk_size;
            let end = (start + chunk_size).min(words.len());

            if start >= words.len() {
                break;
            }

            let chunk_content = words[start..end].join(" ");
            let is_last_chunk = i == num_chunks - 1 || end == words.len();

            chunks.push(format_completion_chunk(
                model,
                (i + 1) as u32,
                Some(if i > 0 {
                    format!(" {}", chunk_content)
                } else {
                    chunk_content
                }),
                None,
                if is_last_chunk {
                    Some("stop".to_string())
                } else {
                    None
                },
            ));
        }
    }

    chunks
}

// Tests for this module are in formatting_tests.rs
