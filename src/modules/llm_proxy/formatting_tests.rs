//! Tests for the LLM Proxy formatting module
//!
//! This file contains comprehensive tests for the response formatting functionality.

#[cfg(test)]
mod tests {
    use super::super::formatting::*;
    use super::super::routes::ChatCompletionRequest;
    use crate::modules::model_registry::connectors::ChatMessage;

    #[test]
    fn test_generate_response_id() {
        let id = generate_response_id();
        assert!(id.starts_with("chatcmpl-"));
        assert_eq!(id.len(), 8 + 32); // "chatcmpl-" + 32 chars UUID without hyphens
    }

    #[test]
    fn test_normalize_model_name() {
        assert_eq!(normalize_model_name("gpt-3.5-turbo"), "gpt-3.5-turbo");
        assert_eq!(normalize_model_name("text-davinci-003"), "text-davinci-003");
        assert_eq!(normalize_model_name("claude-2"), "anthropic/claude-2");
        assert_eq!(
            normalize_model_name("mistral-medium"),
            "mistral/mistral-medium"
        );
        assert_eq!(normalize_model_name("llama-7b"), "meta/llama-7b");
        assert_eq!(normalize_model_name("custom-model"), "unknown/custom-model");
    }

    #[test]
    fn test_token_calculation() {
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a helpful assistant.".to_string(),
                name: None,
                function_call: None,
                tool_calls: None,
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Hello, how are you?".to_string(),
                name: None,
                function_call: None,
                tool_calls: None,
            },
        ];

        let prompt_tokens = calculate_prompt_tokens(&messages);
        assert!(prompt_tokens > 0);

        let completion = "I'm doing well, thank you for asking!";
        let completion_tokens = calculate_completion_tokens(completion);
        assert!(completion_tokens > 0);

        let usage = calculate_token_usage(&messages, completion);
        assert_eq!(
            usage.total_tokens,
            usage.prompt_tokens + usage.completion_tokens
        );
    }

    #[test]
    fn test_format_completion_response() {
        let model = "gpt-3.5-turbo";
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
            name: None,
            function_call: None,
            tool_calls: None,
        }];
        let content = "Hi there! How can I help you today?";
        let finish_reason = "stop";

        let response = format_completion_response(model, &messages, content, finish_reason);

        assert!(response.id.starts_with("chatcmpl-"));
        assert_eq!(response.object, "chat.completion");
        assert_eq!(response.model, "gpt-3.5-turbo");
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.choices[0].index, 0);
        assert_eq!(response.choices[0].message.role, "assistant");
        assert_eq!(response.choices[0].message.content, content);
        assert_eq!(response.choices[0].finish_reason, "stop");
        assert!(response.usage.prompt_tokens > 0);
        assert!(response.usage.completion_tokens > 0);
        assert_eq!(
            response.usage.total_tokens,
            response.usage.prompt_tokens + response.usage.completion_tokens
        );
    }

    #[test]
    fn test_format_completion_chunk() {
        let model = "gpt-3.5-turbo";
        let content = Some("Hello".to_string());
        let role = Some("assistant".to_string());
        let finish_reason = Some("stop".to_string());

        let chunk = format_completion_chunk(
            model,
            1,
            content.clone(),
            role.clone(),
            finish_reason.clone(),
        );

        assert!(chunk.id.starts_with("chatcmpl-"));
        assert_eq!(chunk.object, "chat.completion.chunk");
        assert_eq!(chunk.model, "gpt-3.5-turbo");
        assert_eq!(chunk.choices.len(), 1);
        assert_eq!(chunk.choices[0].index, 0);
        assert_eq!(chunk.choices[0].delta.role, role);
        assert_eq!(chunk.choices[0].delta.content, content);
        assert_eq!(chunk.choices[0].finish_reason, finish_reason);
    }

    #[test]
    fn test_create_streaming_chunks() {
        let model = "gpt-3.5-turbo";
        let content = "This is a test response that should be split into multiple chunks.";

        // Test with 3 chunks
        let chunks = create_streaming_chunks(model, content, 3);

        // Should have 4 chunks (1 for role + 3 for content)
        assert_eq!(chunks.len(), 4);

        // First chunk should have role but no content
        assert_eq!(
            chunks[0].choices[0].delta.role,
            Some("assistant".to_string())
        );
        assert_eq!(chunks[0].choices[0].delta.content, None);
        assert_eq!(chunks[0].choices[0].finish_reason, None);

        // Middle chunks should have content but no role or finish_reason
        assert_eq!(chunks[1].choices[0].delta.role, None);
        assert!(chunks[1].choices[0].delta.content.is_some());
        assert_eq!(chunks[1].choices[0].finish_reason, None);

        // Last chunk should have content and finish_reason but no role
        assert_eq!(chunks[3].choices[0].delta.role, None);
        assert!(chunks[3].choices[0].delta.content.is_some());
        assert_eq!(chunks[3].choices[0].finish_reason, Some("stop".to_string()));
    }

    #[test]
    fn test_contextual_response_generation() {
        // Test hello response
        let hello_response = generate_contextual_response("Hello there!");
        assert!(hello_response.contains("Hello"));
        assert!(hello_response.contains("mock assistant"));

        // Test help response
        let help_response = generate_contextual_response("Can you help me?");
        assert!(help_response.contains("help"));

        // Test weather response
        let weather_response = generate_contextual_response("What's the weather like?");
        assert!(weather_response.contains("weather"));

        // Test code response
        let code_response = generate_contextual_response("Write some code for me");
        assert!(code_response.contains("code"));

        // Test explain response
        let explain_response = generate_contextual_response("What is quantum computing?");
        assert!(explain_response.contains("explain"));

        // Test default response
        let default_response = generate_contextual_response("Something completely different");
        assert!(default_response.contains("mock response"));
    }

    #[test]
    fn test_temperature_effects() {
        let content = "This is a test response.";

        // No temperature
        let no_temp = apply_temperature_effects(content, None);
        assert_eq!(no_temp, content);

        // Low temperature
        let low_temp = apply_temperature_effects(content, Some(0.5));
        assert_eq!(low_temp, content);

        // High temperature
        let high_temp = apply_temperature_effects(content, Some(1.5));
        assert!(high_temp.contains(content));
        assert!(high_temp.contains("temperature"));
    }

    #[test]
    fn test_max_tokens_truncation() {
        let content = "This is a test response with multiple words to test truncation.";

        // No max_tokens
        let no_max = apply_max_tokens_truncation(content, None);
        assert_eq!(no_max, content);

        // High max_tokens (no truncation)
        let high_max = apply_max_tokens_truncation(content, Some(100));
        assert_eq!(high_max, content);

        // Low max_tokens (with truncation)
        let low_max = apply_max_tokens_truncation(content, Some(3));
        assert!(low_max.len() < content.len());
        assert!(low_max.contains("..."));
    }
}
